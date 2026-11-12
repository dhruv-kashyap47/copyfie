param(
  [ValidateSet("light", "dark")][string]$Theme = "dark",
  [ValidateSet("bright", "dark")][string]$Wallpaper = "bright",
  [ValidateSet("palette", "settings")][string]$Window = "palette",
  [Parameter(Mandatory = $true)][string]$Out,
  [string]$Exe = "target/release/copyfie.exe"
)

# Captures the real Copyfie window over a generated wallpaper plate.
# Restores settings.json in `finally`; the desktop wallpaper is never changed.
# Usage:
#   powershell -File scripts/design/capture.ps1 -Theme dark -Wallpaper bright `
#     -Window palette -Out docs/design/palette-dark-bright.png

$ErrorActionPreference = "Stop"

$settingsPath = Join-Path $env:APPDATA "com.copyfie.app\settings.json"
$settingsBackup = Join-Path $env:TEMP "opencode\copyfie-settings-before-capture.json"
New-Item -ItemType Directory -Force (Split-Path $settingsBackup) | Out-Null
if (-not (Test-Path $settingsBackup)) {
  Copy-Item $settingsPath $settingsBackup -Force
}

Add-Type @'
using System;
using System.Runtime.InteropServices;
public struct COPYFIE_RECT { public int Left, Top, Right, Bottom; }
public class CopyfieCaptureWin32 {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out COPYFIE_RECT rect);
  [DllImport("user32.dll")] public static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
}
'@

function Invoke-Key([byte]$Key, [byte]$Scan = 0) {
  [CopyfieCaptureWin32]::keybd_event($Key, $Scan, 0, [UIntPtr]::Zero)
  [CopyfieCaptureWin32]::keybd_event($Key, $Scan, 2, [UIntPtr]::Zero)
}

function Save-WindowShot([IntPtr]$Handle, [string]$Path, [int]$Margin = 18) {
  $rect = New-Object COPYFIE_RECT
  if (-not [CopyfieCaptureWin32]::GetWindowRect($Handle, [ref]$rect)) {
    throw "could not read the window rectangle"
  }
  $width = ($rect.Right - $rect.Left) + ($Margin * 2)
  $height = ($rect.Bottom - $rect.Top) + ($Margin * 2)
  Add-Type -AssemblyName System.Drawing
  $bitmap = New-Object System.Drawing.Bitmap $width, $height
  $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
  $graphics.CopyFromScreen($rect.Left - $Margin, $rect.Top - $Margin, 0, 0, $bitmap.Size)
  $directory = Split-Path -Parent $Path
  if ($directory) { New-Item -ItemType Directory -Force $directory | Out-Null }
  $bitmap.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
  $graphics.Dispose()
  $bitmap.Dispose()
}

try {
  Stop-Process -Name copyfie -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 500

  $settings = Get-Content $settingsPath -Raw | ConvertFrom-Json
  $settings.theme = $Theme
  $settings.accent = "#5B9DFF"
  # Write UTF-8 without a BOM; serde_json rejects a BOM.
  [System.IO.File]::WriteAllText(
    $settingsPath,
    ($settings | ConvertTo-Json -Depth 6),
    [System.Text.UTF8Encoding]::new($false)
  )

  $plate = Start-Process powershell -ArgumentList @(
    "-ExecutionPolicy", "Bypass",
    "-File", "scripts/design/wallpaper_window.ps1",
    "-Image", (Resolve-Path (Join-Path "docs/design/wallpapers" "$Wallpaper.png")).Path
  ) -PassThru -WindowStyle Hidden
  Start-Sleep -Milliseconds 1200
  Start-Process -FilePath $Exe -ArgumentList "--show"
  Start-Sleep -Seconds 3

  # The palette focuses the search field on show. Clear any stray text so the
  # screenshot always shows the full recent list.
  [CopyfieCaptureWin32]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero)
  Invoke-Key 0x41 0
  [CopyfieCaptureWin32]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)
  Invoke-Key 0x08 0
  Invoke-Key 0x24 0
  [CopyfieCaptureWin32]::SetCursorPos(8, 8) | Out-Null
  Start-Sleep -Milliseconds 500

  if ($Window -eq "settings") {
    [CopyfieCaptureWin32]::keybd_event(0x11, 0, 0, [UIntPtr]::Zero)
    Invoke-Key 0xBC 0
    [CopyfieCaptureWin32]::keybd_event(0x11, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Seconds 2
  }

  $handle = (Get-Process copyfie -ErrorAction Stop | Select-Object -First 1).MainWindowHandle
  if ($handle -eq [IntPtr]::Zero) {
    throw "window not found: $Window"
  }

  Save-WindowShot $handle $Out
  Write-Host "Wrote $Out"
} finally {
  Stop-Process -Name copyfie -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 500
  if (Test-Path $settingsBackup) {
    Copy-Item $settingsBackup $settingsPath -Force
  }
  if ($plate) {
    Stop-Process -Id $plate.Id -Force -ErrorAction SilentlyContinue
  }}
