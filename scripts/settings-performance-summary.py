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
        for name in ('lock_ns', 'copy_ns', 'unlock_ns'):
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
                        'display_flags': flags, 'app_surface_tracks': surfaces, 'trace_input_delivery': trace_delivery})
    output = args.directory / 'summary.json'
    output.write_text(json.dumps({'boundary': 'SurfaceFlinger display-frame start to present, never app/input latency',
        'emulator_limit': 'Prediction-error/early-present flags do not establish app jank. Software native window buffers have no app SurfaceFrame timeline here.',
        'trials': results}, indent=2)+'\n')
    print(output)


if __name__ == '__main__':
    main()
