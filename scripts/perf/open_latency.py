#!/usr/bin/env python3
"""Measure warm and cold palette open latency from the app log.

The palette reports `palette_open_ms=<n> cold=<bool>` after its first paint.
This script drives the global hotkey, hides between iterations, and forces the
webview to be destroyed for cold runs using the `--palette-destroy` test hook.

Windows only. The foreground window must not be elevated: UIPI drops synthetic
input into high-integrity windows. Pass `--focus-pid` to activate a normal app
before each iteration.

Usage:
  python scripts/perf/open_latency.py --warm 10 --cold 10 \
    --focus-pid 1234 --out docs/perf/open-latency.json
"""

import argparse
import ctypes
import ctypes.wintypes as wt
import json
import os
import re
import statistics
import subprocess
import sys
import time

user32 = ctypes.WinDLL("user32", use_last_error=True)
user32.keybd_event.argtypes = [wt.BYTE, wt.BYTE, wt.DWORD, ctypes.c_size_t]
user32.GetForegroundWindow.restype = wt.HWND
user32.SetForegroundWindow.argtypes = [wt.HWND]
user32.EnumWindows.argtypes = [ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM), wt.LPARAM]

VK_CONTROL = 0x11
VK_SHIFT = 0x10
VK_SPACE = 0x20
VK_ESCAPE = 0x1B

LINE = re.compile(r"palette_open_ms=(\d+) cold=(true|false)")


def press(vk: int) -> None:
    user32.keybd_event(vk, 0, 0, 0)


def release(vk: int) -> None:
    user32.keybd_event(vk, 0, 2, 0)


def hotkey() -> None:
    press(VK_CONTROL)
    press(VK_SHIFT)
    press(VK_SPACE)
    time.sleep(0.05)
    release(VK_SPACE)
    release(VK_SHIFT)
    release(VK_CONTROL)


def escape() -> None:
    press(VK_ESCAPE)
    release(VK_ESCAPE)


def main_window_for(pid: int) -> int:
    result = {"hwnd": 0}

    def callback(hwnd, _):
        owner = wt.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
        if owner.value == pid and user32.IsWindowVisible(hwnd):
            length = user32.GetWindowTextLengthW(hwnd)
            if length > 0 and result["hwnd"] == 0:
                result["hwnd"] = hwnd
        return True

    enum = ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)(callback)
    user32.EnumWindows(enum, 0)
    return result["hwnd"]


def log_path() -> str:
    local = os.environ.get("LOCALAPPDATA")
    if not local:
        raise SystemExit("LOCALAPPDATA is not set")
    return os.path.join(local, "com.copyfie.app", "logs", "Copyfie.log")


def read_logs() -> str:
    """Read every rotated Copyfie log, oldest first.

    The log plugin may rotate during a run; reading an offset from a single file
    loses samples. Metrics are the most recent activity, so concatenating all
    files in modification order and taking the last N of each kind is stable.
    """
    directory = os.path.dirname(log_path())
    if not os.path.isdir(directory):
        return ""

    files = [
        os.path.join(directory, name)
        for name in os.listdir(directory)
        if name.startswith("Copyfie.log")
    ]
    files.sort(key=lambda path: os.path.getmtime(path))

    chunks: list[str] = []
    for path in files:
        try:
            with open(path, "r", encoding="utf-8", errors="replace") as handle:
                chunks.append(handle.read())
        except OSError:
            continue
    return "\n".join(chunks)


def collect(samples: list[float]) -> dict:
    if not samples:
        return {"samples": 0}
    ordered = sorted(samples)
    return {
        "samples": len(ordered),
        "p50_ms": round(statistics.median(ordered), 1),
        "p95_ms": round(ordered[min(len(ordered) - 1, int(len(ordered) * 0.95))], 1),
        "max_ms": round(ordered[-1], 1),
        "all_ms": [round(value, 1) for value in ordered],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--warm", type=int, default=10)
    parser.add_argument("--cold", type=int, default=10)
    parser.add_argument("--focus-pid", type=int, default=0)
    parser.add_argument("--exe", default="target/release/copyfie.exe")
    parser.add_argument("--out", default=None)
    args = parser.parse_args()

    focus_hwnd = main_window_for(args.focus_pid) if args.focus_pid else 0
    if args.focus_pid and not focus_hwnd:
        print(f"could not find a visible window for pid {args.focus_pid}", file=sys.stderr)
        return 2

    def activate() -> None:
        if focus_hwnd:
            user32.SetForegroundWindow(focus_hwnd)
            time.sleep(0.25)

    # Warm opens: the webview already exists.
    for index in range(args.warm):
        activate()
        hotkey()
        time.sleep(1.0)
        escape()
        time.sleep(0.35)
        print(f"warm {index + 1}/{args.warm}")

    # Cold opens: destroy the hidden webview first.
    for index in range(max(0, args.cold)):
        escape()
        time.sleep(0.35)
        subprocess.run(
            [os.path.abspath(args.exe), "--palette-destroy"],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        time.sleep(0.6)
        activate()
        hotkey()
        time.sleep(1.6)
        print(f"cold {index + 1}/{args.cold}")

    escape()
    time.sleep(0.4)

    text = read_logs()
    warm: list[float] = []
    cold: list[float] = []
    for line in text.splitlines():
        match = LINE.search(line)
        if not match:
            continue
        value = float(match.group(1))
        (cold if match.group(2) == "true" else warm).append(value)

    report = {
        "platform": "windows",
        "warm": collect(warm[-args.warm :] if args.warm else warm),
        "cold": collect(cold[-args.cold :] if args.cold else cold),
    }
    print(json.dumps(report, indent=2))

    if args.out:
        directory = os.path.dirname(args.out)
        if directory:
            os.makedirs(directory, exist_ok=True)
        with open(args.out, "w", encoding="utf-8") as handle:
            json.dump(report, handle, indent=2)

    return 0 if report["warm"].get("samples") or report["cold"].get("samples") else 1


if __name__ == "__main__":
    raise SystemExit(main())
