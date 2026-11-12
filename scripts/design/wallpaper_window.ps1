param(
  [Parameter(Mandatory = $true)][string]$Image
)

# Shows a borderless, full-work-area wallpaper plate behind the Copyfie window.
# This keeps screenshots deterministic without changing the user's desktop
# wallpaper. The palette is always-on-top, so it renders above this window.

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$form = New-Object System.Windows.Forms.Form
$form.FormBorderStyle = [System.Windows.Forms.FormBorderStyle]::None
$form.WindowState = [System.Windows.Forms.FormWindowState]::Maximized
$form.ShowInTaskbar = $false
$form.TopMost = $false
$form.BackColor = [System.Drawing.Color]::Black

$picture = New-Object System.Windows.Forms.PictureBox
$picture.Dock = [System.Windows.Forms.DockStyle]::Fill
$picture.SizeMode = [System.Windows.Forms.PictureBoxSizeMode]::Zoom
$picture.Image = [System.Drawing.Image]::FromFile((Resolve-Path $Image).Path)
$form.Controls.Add($picture)

[System.Windows.Forms.Application]::Run($form)
