#!/usr/bin/env python3
"""Measure clipboard-change -> stored-row latency.

Windows only. Writes CF_UNICODETEXT directly through user32, then polls the
Copyfie SQLite database every 5 ms until the row appears.

Usage:
  python scripts/perf/capture_latency.py [--db PATH] [--iterations 20] [--out FILE]
"""

import argparse
import ctypes
import ctypes.wintypes as wintypes
import json
import os
import sqlite3
import statistics
import sys
import time

CF_UNICODETEXT = 13
GMEM_MOVEABLE = 0x0002

user32 = ctypes.WinDLL("user32", use_last_error=True)
kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)

user32.OpenClipboard.argtypes = [wintypes.HWND]
user32.SetClipboardData.argtypes = [wintypes.UINT, wintypes.HANDLE]
user32.SetClipboardData.restype = wintypes.HANDLE
kernel32.GlobalAlloc.argtypes = [wintypes.UINT, ctypes.c_size_t]
kernel32.GlobalAlloc.restype = wintypes.HGLOBAL
kernel32.GlobalLock.argtypes = [wintypes.HGLOBAL]
kernel32.GlobalLock.restype = ctypes.c_void_p
kernel32.GlobalUnlock.argtypes = [wintypes.HGLOBAL]


def set_clipboard_text(text: str) -> None:
    data = (text + "\0").encode("utf-16-le")
    handle = kernel32.GlobalAlloc(GMEM_MOVEABLE, len(data))
    if not handle:
        raise OSError("GlobalAlloc failed")
    pointer = kernel32.GlobalLock(handle)
    if not pointer:
        raise OSError("GlobalLock failed")
    ctypes.memmove(pointer, data, len(data))
    kernel32.GlobalUnlock(handle)

    if not user32.OpenClipboard(None):
        raise OSError(f"OpenClipboard failed: {ctypes.get_last_error()}")
    try:
        user32.EmptyClipboard()
        if not user32.SetClipboardData(CF_UNICODETEXT, handle):
            raise OSError("SetClipboardData failed")
    finally:
        user32.CloseClipboard()


def default_db() -> str:
    appdata = os.environ.get("APPDATA")
    if not appdata:
        raise SystemExit("APPDATA is not set")
    return os.path.join(appdata, "com.copyfie.app", "clipboard.db")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--db", default=default_db())
    parser.add_argument("--iterations", type=int, default=20)
    parser.add_argument("--timeout", type=float, default=5.0)
    parser.add_argument("--out", default=None)
    args = parser.parse_args()

    if not os.path.exists(args.db):
        raise SystemExit(f"database not found: {args.db}")

    connection = sqlite3.connect(f"file:{args.db}?mode=ro", uri=True, isolation_level=None)
    samples = []
    run_id = int(time.time() * 1000)

    for index in range(args.iterations):
        text = f"copyfie-latency-{run_id}-{index}"
        set_clipboard_text(text)
        started = time.perf_counter()
        deadline = started + args.timeout

        while time.perf_counter() < deadline:
            row = connection.execute(
                "select 1 from items where preview = ? limit 1", (text,)
            ).fetchone()
            if row:
                break
            time.sleep(0.005)
        else:
            print(f"timeout waiting for {text}", file=sys.stderr)
            continue

        samples.append((time.perf_counter() - started) * 1000.0)

    connection.close()

    if not samples:
        raise SystemExit("no samples collected; is Copyfie running and unpaused?")

    samples.sort()
    p50 = statistics.median(samples)
    p95 = samples[min(len(samples) - 1, int(len(samples) * 0.95))]
    report = {
        "platform": "windows",
        "db": args.db,
        "iterations": len(samples),
        "p50_ms": round(p50, 1),
        "p95_ms": round(p95, 1),
        "max_ms": round(max(samples), 1),
        "samples_ms": [round(value, 1) for value in samples],
    }

    print(json.dumps(report, indent=2))
    if args.out:
        with open(args.out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=2)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
