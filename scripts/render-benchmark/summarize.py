"""Summarize numeric frame logs only. Run with python3 -I; never consume editor text."""
import argparse
import csv
import json
import re
from pathlib import Path


def percentiles(values):
    ordered = sorted(values)
    if not ordered:
        return {"count": 0}
    return {"count": len(ordered), **{
        name: round(ordered[min(len(ordered) - 1, int((len(ordered) - 1) * percentile))] / 1e6, 3)
        for name, percentile in (("p50_ms", .5), ("p95_ms", .95), ("p99_ms", .99))}}


def native(path):
    frames = []
    gpu_waits = []
    for line in Path(path).read_text().splitlines():
        match = re.search(r"frame draw_ns=(\d+) post_ns=(\d+) touch_ns=(\d+) choreographer_ns=(\d+)", line)
        if match:
            frames.append(tuple(map(int, match.groups())))
            gpu = re.search(r"gpu_wait_ns=(\d+)", line)
            if gpu and int(gpu[1]):
                gpu_waits.append(int(gpu[1]))
    # Touch timestamps identify the first posted update for each event. Later NLP-only updates
    # share that timestamp; counting them as input latency would inflate the measurement.
    seen = set()
    latency = []
    for _, posted, touch, _ in frames:
        if touch and touch not in seen and posted >= touch:
            seen.add(touch)
            latency.append(posted - touch)
    return {"draw": percentiles([f[0] for f in frames]), "touch_to_post": percentiles(latency),
            "gpu_submission_to_completion_wall": percentiles(gpu_waits)}


def compose(paths):
    rows = []
    for path in paths:
        text = Path(path).read_text()
        for section in text.split("---PROFILEDATA---")[1::2]:
            reader = csv.DictReader(section.strip().splitlines())
            for row in reader:
                try:
                    if int(row["Flags"]) == 0 and int(row["FrameCompleted"]) > int(row["IntendedVsync"]):
                        rows.append(row)
                except (KeyError, ValueError):
                    continue
    # gfxinfo spans the app's HWUI frames, including Compose. It cannot observe SurfaceView
    # posts, and these durations are not the same quantity as Rust's scene+raster CPU time.
    durations = [int(r["FrameCompleted"]) - int(r["IntendedVsync"]) for r in rows]
    cpu_draw = [int(r["SyncQueued"]) - int(r["DrawStart"]) for r in rows]
    input_latencies = []
    for row in rows:
        timestamp = int(row.get("OldestInputEvent", "0"))
        if 0 < timestamp < int(row["FrameCompleted"]):
            input_latencies.append(int(row["FrameCompleted"]) - timestamp)
    return {"frame_completion": percentiles(durations), "hwui_draw_recording": percentiles(cpu_draw),
            "touch_to_completion": percentiles(input_latencies)}


def compose_touch(path):
    seen = set()
    values = []
    for line in Path(path).read_text().splitlines():
        match = re.search(r"frame touch_ns=(\d+) post_ns=(\d+)", line)
        if match:
            touch, posted = map(int, match.groups())
            if touch not in seen and posted >= touch:
                seen.add(touch)
                values.append(posted - touch)
    return percentiles(values)


def profile(path):
    frames = []
    for line in Path(path).read_text().splitlines():
        if "profile draw_ns=" in line:
            frames.append({k: int(v) for k, v in re.findall(r"(\w+)=(\d+)", line)})
    tails = [f for f in frames if f["draw_ns"] > 2_000_000]
    groups = {
        "all": frames, "over_2ms": tails,
        "full": [f for f in tails if f["full"]],
        "dirty": [f for f in tails if not f["full"]],
        "press": [f for f in tails if f.get("press")],
        "release": [f for f in tails if f.get("release")],
        "surface_create": [f for f in tails if f.get("recreate")],
        "other": [f for f in tails if not any(f.get(k) for k in
                  ("press", "release", "recreate", "shift", "popup_change", "candidates"))],
        "shift": [f for f in tails if f["shift"]],
        "popup_open_or_move": [f for f in tails if f["popup_change"] and f["popup_open"]],
        "popup_close": [f for f in tails if f["popup_change"] and not f["popup_open"]],
        "candidate_change": [f for f in tails if f["candidates"]],
        "shape_cache_miss": [f for f in tails if f["layout_misses"]],
    }
    result = {}
    for name, group in groups.items():
        result[name] = {
            "draw": percentiles([f["draw_ns"] for f in group]),
            "buffer_copy_post": percentiles([f["buffer_post_ns"] for f in group
                                             if "buffer_post_ns" in f]),
            "area_px_min": min((f["area_px"] for f in group), default=0),
            "area_px_max": max((f["area_px"] for f in group), default=0),
            "totals_ms": {k: round(sum(f[k] for f in group) / 1e6, 3) for k in
                          ("encode_ns", "raster_ns", "shape_ns", "glyph_ns", "alloc_ns",
                           "cold_glyph_ns", "color_glyph_ns")},
            "explicit_alloc_sites": sum(f["alloc_sites"] for f in group),
            "layout_misses": sum(f["layout_misses"] for f in group),
        }
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--native")
    parser.add_argument("--compose", nargs="*")
    parser.add_argument("--compose-touch")
    parser.add_argument("--profile")
    args = parser.parse_args()
    result = {}
    if args.native:
        result["rust"] = native(args.native)
    if args.compose:
        result["compose"] = compose(args.compose)
    if args.compose_touch:
        result.setdefault("compose", {})["touch_to_commit"] = compose_touch(args.compose_touch)
    if args.profile:
        result["profile"] = profile(args.profile)
    print(json.dumps(result, indent=2))
