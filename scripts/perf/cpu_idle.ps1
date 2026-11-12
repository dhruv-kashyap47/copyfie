param(
  [int]$DurationSeconds = 60,
  [int]$IntervalSeconds = 2,
  [string]$Out = "docs/perf/cpu-idle.json",
  [string]$Label = "idle",
  [int]$CopyfieProcessId = 0
)

# Samples CPU time for the Copyfie process tree (core + WebView2 children).
# Reports average percent of one core and percent of the whole machine.
# Run with the palette in the state being compared (hidden, or visible and idle).

$ErrorActionPreference = "Stop"

if ($CopyfieProcessId -eq 0) {
  $app = Get-Process copyfie -ErrorAction Stop | Select-Object -First 1
} else {
  $app = Get-Process -Id $CopyfieProcessId -ErrorAction Stop
}

function Get-TreeIds {
  param([int]$RootId)

  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId
  $ids = [System.Collections.Generic.HashSet[int]]::new()
  [void]$ids.Add($RootId)
  $changed = $true

  while ($changed) {
    $changed = $false
    foreach ($process in $all) {
      if ($ids.Contains([int]$process.ParentProcessId) -and -not $ids.Contains([int]$process.ProcessId)) {
        [void]$ids.Add([int]$process.ProcessId)
        $changed = $true
      }
    }
  }

  return $ids
}

function Get-TreeCpuSeconds {
  param([System.Collections.Generic.HashSet[int]]$Ids)

  $total = 0.0
  foreach ($id in $Ids) {
    $process = Get-Process -Id $id -ErrorAction SilentlyContinue
    if ($process) {
      $total += $process.CPU
    }
  }
  return $total
}

$cores = [Environment]::ProcessorCount
$samples = [System.Collections.Generic.List[object]]::new()
$deadline = (Get-Date).AddSeconds($DurationSeconds)

Write-Host "Sampling CPU for '$Label' (pid $($app.Id)) for $DurationSeconds s every $IntervalSeconds s"

$lastWall = Get-Date
$lastCpu = Get-TreeCpuSeconds -Ids (Get-TreeIds -RootId $app.Id)

while ((Get-Date) -lt $deadline) {
  Start-Sleep -Seconds $IntervalSeconds
  $now = Get-Date
  $ids = Get-TreeIds -RootId $app.Id
  $cpu = Get-TreeCpuSeconds -Ids $ids
  $wall = ($now - $lastWall).TotalSeconds
  $delta = [Math]::Max(0.0, $cpu - $lastCpu)
  $percent = if ($wall -gt 0) { $delta / $wall * 100.0 } else { 0.0 }

  $samples.Add([pscustomobject]@{
    at = $now.ToString("o")
    deltaCpuSeconds = [Math]::Round($delta, 3)
    percentOneCore = [Math]::Round($percent, 2)
    processes = $ids.Count
  })

  $lastWall = $now
  $lastCpu = $cpu
}

$values = $samples | ForEach-Object { $_.percentOneCore }
$avg = if ($samples.Count) { ($values | Measure-Object -Average).Average } else { 0 }
$max = if ($samples.Count) { ($values | Measure-Object -Maximum).Maximum } else { 0 }

$summary = [pscustomobject]@{
  capturedAt = (Get-Date).ToString("o")
  platform = "windows"
  label = $Label
  durationSeconds = $DurationSeconds
  intervalSeconds = $IntervalSeconds
  logicalProcessors = $cores
  samples = $samples.Count
  avgPercentOneCore = [Math]::Round($avg, 2)
  maxPercentOneCore = [Math]::Round($max, 2)
  avgPercentMachine = [Math]::Round($avg / $cores, 3)
  series = $samples
}

$directory = Split-Path -Parent $Out
if ($directory) {
  New-Item -ItemType Directory -Force $directory | Out-Null
}

$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $Out -Encoding utf8
Write-Host "Wrote $Out (avg/max of one core: $($summary.avgPercentOneCore)/$($summary.maxPercentOneCore) %)"
