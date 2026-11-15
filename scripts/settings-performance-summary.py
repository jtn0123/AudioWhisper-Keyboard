#!/usr/bin/env python3
"""Summarize numeric renderer logs, dumpsys categories and the same Perfetto display boundary."""
import argparse
import csv
import io
import json
from pathlib import Path
import re
import statistics
import subprocess


def distribution(values):
    values = sorted(values)
    if not values:
        return {'count': 0}
    return {'count': len(values), 'p50_ms': statistics.median(values) / 1e6,
            'p95_ms': values[int((len(values)-1)*.95)] / 1e6}


def numeric(path):
    return [{k: int(v) for k, v in re.findall(r'(\w+)=([0-9]+)', line)}
            for line in path.read_text().splitlines()]


def memory(path):
    rows = {}
    for line in path.read_text().splitlines():
        if line.strip().startswith('TOTAL'):
            break
        match = re.match(r'^\s*(.+?)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)(?:\s|$)', line)
        if match:
            rows[match[1].strip()] = {'pss_kib': int(match[2]), 'rss_kib': int(match[6])}
    text = path.read_text()
    total = re.search(r'TOTAL PSS:\s+(\d+)\s+TOTAL RSS:\s+(\d+)', text)
    return {'categories': rows, 'pss_kib': int(total[1]), 'rss_kib': int(total[2])}


def query(processor, trace, sql):
    result = subprocess.run([str(processor.resolve()), str(trace), '-Q', sql], check=True,
                            capture_output=True, text=True)
    return list(csv.DictReader(io.StringIO(result.stdout)))


APP = 'com.jtn0123.audiowhisper.keyboard'


def app_frames(processor, trace, delivered):
    """Like for like on Android 13+ frame timelines: each input event's Choreographer frame
    (the doFrame it was delivered in, or the next one when delivered between frames) names
    a vsync id; the app SurfaceFrames carrying that id (HWUI's window for Compose, the
    SurfaceControl child for Rust) give its present time and its jank classification."""
    frames = query(processor, trace, f"""SELECT a.ts,a.dur,a.layer_name,a.surface_frame_token token,
        a.jank_type,a.present_type FROM actual_frame_timeline_slice a JOIN process p USING(upid)
        WHERE p.name='{APP}' AND a.surface_frame_token IS NOT NULL AND a.dur >= 0""")
    callbacks = query(processor, trace, f"""SELECT s.ts,s.dur,s.name FROM slice s
        JOIN thread_track t ON s.track_id=t.id JOIN thread th ON t.utid=th.utid JOIN process p ON th.upid=p.upid
        WHERE p.name='{APP}' AND th.tid=p.pid AND s.name GLOB 'Choreographer#doFrame *' ORDER BY s.ts""")
    by_token = {}
    for frame in frames:
        by_token.setdefault(int(frame['token']), []).append(frame)
    latencies = []
    for row in delivered:
        ts, event = int(row['ts']), int(re.search(r'eventTimeNano=(\d+)', row['name'])[1])
        owner = next((f for f in callbacks if int(f['ts']) <= ts <= int(f['ts']) + int(f['dur'])), None) \
            or next((f for f in callbacks if int(f['ts']) > ts), None)
        shown = by_token.get(int(owner['name'].split()[1]), []) if owner else []
        if shown:
            latencies.append(max(int(f['ts']) + int(f['dur']) for f in shown) - event)
    jank = {}
    for frame in frames:
        jank[frame['jank_type']] = jank.get(frame['jank_type'], 0) + 1
    # Prediction Error is SurfaceFlinger mispredicting its own schedule (on this emulator
    # nearly every frame of both renderers); only a missed app deadline is the app's jank.
    missed = sum(n for kind, n in jank.items() if 'App Deadline Missed' in kind)
    stuffed = sum(n for kind, n in jank.items() if 'Buffer Stuffing' in kind)
    layers = {}
    for frame in frames:
        layers[frame['layer_name']] = layers.get(frame['layer_name'], 0) + 1
    return {'surface_frames': len(frames), 'layers': layers, 'jank_types': jank,
            'app_deadline_missed': missed, 'buffer_stuffing': stuffed, 'input_events': len(delivered),
            'input_to_present': distribution(latencies)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('trace_processor', type=Path)
    args = parser.parse_args()
    results = []
    for trace in sorted(args.directory.glob('*.perfetto-trace')):
        label = trace.name.removesuffix('.perfetto-trace')
        values = numeric(trace.with_name(label + '-native.txt'))
        events = {v['event_ns']: v for v in values if 'event_ns' in v}
        frames = [v for v in values if 'post_ns' in v]
        first_post = {}
        first_frame = {}
        submissions = {v["frame_ns"]: v["submit_ns"] for v in values if "frame_ns" in v and "submit_ns" in v}
        for frame in frames:
            event = frame.get('touch_ns')
            if event in events:
                if event not in first_post or frame['post_ns'] < first_post[event]:
                    first_post[event] = frame['post_ns']
                    first_frame[event] = frame
        def durations(end, start):
            return distribution([v[end]-v[start] for v in values if end in v and start in v])
        phases = {'delivery': durations('delivered_ns', 'event_ns'),
                  'ui_reply': durations('reply_ns', 'delivered_ns'),
                  'frame_callback_age': durations('submit_ns', 'frame_ns'),
                  'frame_to_worker': durations('worker_ns', 'choreographer_ns'),
                  'touch_to_first_post': distribution([post-event for event, post in first_post.items()])}
        joined = [(events[event], frame, submissions[frame['choreographer_ns']])
                  for event, frame in first_frame.items() if frame['choreographer_ns'] in submissions]
        phases['reply_to_submit'] = distribution([submit-event['reply_ns'] for event, frame, submit in joined])
        phases['submit_to_worker'] = distribution([frame['worker_ns']-submit for event, frame, submit in joined])
        phases['worker_to_post'] = distribution([frame['post_ns']-frame['worker_ns'] for event, frame, submit in joined])
        for name in ('wait_ns', 'lock_ns', 'copy_ns', 'unlock_ns'):
            phases[name] = distribution([v[name] for v in values if name in v])
        phases['draw'] = distribution([v['draw_ns'] for v in frames])
        # Display frames have no surface token. This is the identical SF-start -> present
        # boundary for both renderers. It is explicitly NOT app/input -> present latency.
        display = query(args.trace_processor, trace, '''SELECT dur,present_type,jank_type FROM actual_frame_timeline_slice
            WHERE surface_frame_token IS NULL AND dur >= 0''')
        delivered = query(args.trace_processor, trace, """SELECT s.ts,s.name FROM slice s
            JOIN thread_track t ON s.track_id=t.id JOIN thread th ON t.utid=th.utid
            JOIN process p ON th.upid=p.upid WHERE s.name GLOB 'deliverInputEvent src=*'
            AND p.name='com.jtn0123.audiowhisper.keyboard'""")
        trace_delivery = distribution([int(row['ts'])-int(re.search(r'eventTimeNano=(\d+)', row['name'])[1])
                                       for row in delivered])
        surfaces = query(args.trace_processor, trace, '''SELECT layer_name,COUNT(*) frames FROM actual_frame_timeline_slice
            WHERE surface_frame_token IS NOT NULL GROUP BY layer_name''')
        flags = {}
        for frame in display:
            key = frame['present_type'] + '/' + frame['jank_type']
            flags[key] = flags.get(key, 0) + 1
        results.append({'trial': label, 'memory': memory(trace.with_name(label+'-meminfo.txt')),
                        'phases': phases, 'display_start_to_present': distribution([int(row['dur']) for row in display]),
                        'display_flags': flags, 'app_surface_tracks': surfaces, 'trace_input_delivery': trace_delivery,
                        'app_frames': app_frames(args.trace_processor, trace, delivered)})
    output = args.directory / 'summary.json'
    output.write_text(json.dumps({'boundary': 'SurfaceFlinger display-frame start to present, never app/input latency',
        'app_frames': 'Per input event: the app SurfaceFrame(s) with its doFrame vsync id, present minus event time; jank from the same SurfaceFrames (Buffer Stuffing is queueing, not a missed deadline).',
        'emulator_limit': 'Display-frame prediction-error/early-present flags do not establish app jank; use app_frames.',
        'trials': results}, indent=2)+'\n')
    print(output)


if __name__ == '__main__':
    main()
