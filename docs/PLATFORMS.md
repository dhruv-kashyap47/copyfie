# Copyfie — Platforms

## Support matrix (Milestone 1)

| | Windows 10 1809+ / 11 | macOS 12+ | Linux X11 | Linux Wayland |
|---|---|---|---|---|
| Text capture | `WM_CLIPBOARDUPDATE` via `clipboard-rs` | `NSPasteboard` `changeCount` poll | XFixes via `clipboard-rs` | **Not supported in M1** |
| Concealed markers | Office/KeePassXC/Bitwarden format names, DWORD value checks | `org.nspasteboard.*`, 1Password, TypeIt4Me markers | `x-kde-passwordManagerHint` presence | n/a |
| Source app | `GetForegroundWindow` → PID → image path + title | `NSWorkspace.frontmostApplication` (bundle id + name) | `_NET_ACTIVE_WINDOW` → PID → `/proc` + `WM_CLASS` | n/a |
| Global hotkey | `RegisterHotKey` (plugin) | Carbon hotkey (plugin) | `XGrabKey` (plugin) | n/a |
| Focus restore | HWND + input-queue attach | `NSRunningApplication.activate` | EWMH `_NET_ACTIVE_WINDOW` | n/a |
| Paste chord | `SendInput` via enigo | `CGEvent` via enigo, keycode 9 for V | XTEST via enigo | n/a |
| Tray | Yes | Menu bar | AppIndicator/KSNI (desktop dependent) | partial |
| Autostart | `HKCU\...\Run` | LaunchAgent | XDG autostart | XDG autostart |
| Verified in M1 | **Yes, end-to-end** | Compile-checked only | Compile-checked only | n/a |

## Platform notes

### Windows

- The palette uses custom browser args (`--disable-background-timer-throttling`,
  `--disable-renderer-backgrounding`, `--disable-backgrounding-occluded-windows`)
  so the hidden WebView keeps painting. **Runtime-created webviews must pass the
  same args**, or WebView2 fails with `ERROR_INVALID_STATE (0x8007139F)`.
- Elevated targets: UIPI blocks synthetic input. The clipboard still receives
  the item; explicit detection and a notice are M2-11.
- The foreground lock can deny `SetForegroundWindow`. Copyfie attaches to the
  foreground thread's input queue for user-triggered shows and paste restores.

### macOS

- **Accessibility** permission is required for synthetic paste. Until M2-10,
  the enigo prompt is disabled and a denied permission falls back to copy-only.
- Secure input (`IsSecureEventInputEnabled`) can block synthetic input; M2-10.
- The palette is frameless; the settings window uses the native titlebar.
- `Key::Other(9)` (virtual keycode V) is used instead of a Unicode key, per the
  documented crash in `Key::Unicode('v')`.

### Linux

- **v1.0 is X11-only.** On Wayland the app currently logs a capability warning
  and does not start capture; the visible notice and copy-only UX are M2-12.
- The KDE password-manager hint is detected by **presence**, because
  `clipboard-rs` exposes only the MIME list, not the hint value. This can skip a
  non-secret item; it never stores a secret.
- Foreground identity opens a short-lived X connection per capture.
- Tray support depends on the desktop providing AppIndicator/KSNI.

## Permissions

| | Windows | macOS | Linux X11 |
|---|---|---|---|
| Clipboard read | none | none | none |
| Global hotkey | none | none | none |
| Synthetic input | none | Accessibility | none (XTEST) |
| Autostart | HKCU write | LaunchAgent write | XDG autostart write |
| Network | none in M1 | none in M1 | none in M1 |

## Known gaps before v1.0

- macOS/Linux runtime behaviour needs the M1-13 manual matrix and CI smoke.
- Wayland (M2-12), macOS Accessibility onboarding (M2-10), elevated-target
  detection (M2-11), image/file capture (M2-04/05), blocklist UI (M2-08).
