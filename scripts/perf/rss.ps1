param(
  [int]$DurationSeconds = 600,
  [int]$IntervalSeconds = 5,
  [string]$Out = "docs/perf/rss.json",
  [int]$CopyfieProcessId = 0
)

# Samples the RSS (working set) of the Copyfie process tree every few seconds.
#
# The tree includes WebView2 child processes (browser, GPU, renderer, utility),
# which are the bulk of total memory. Run with the palette hidden and the app
# idle for the M1-12 gate.

$ErrorActionPreference = "Stop"

if ($CopyfieProcessId -eq 0) {
  $app = Get-Process copyfie -ErrorAction Stop | Select-Object -First 1
} else {
  $app = Get-Process -Id $CopyfieProcessId -ErrorAction Stop
}

function Get-TreeProcesses {
  param([int]$RootId)

  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name, WorkingSetSize, CommandLine
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

  $all | Where-Object { $ids.Contains([int]$_.ProcessId) }
}

$samples = [System.Collections.Generic.List[object]]::new()
$deadline = (Get-Date).AddSeconds($DurationSeconds)

Write-Host "Sampling Copyfie (pid $($app.Id)) for $DurationSeconds s every $IntervalSeconds s"

while ((Get-Date) -lt $deadline) {
  $tree = Get-TreeProcesses -RootId $app.Id
  $total = ($tree | Measure-Object -Property WorkingSetSize -Sum).Sum

  $samples.Add([pscustomobject]@{
    at = (Get-Date).ToString("o")
    totalMb = [math]::Round($total / 1MB, 1)
    processes = @($tree | ForEach-Object {
      [pscustomobject]@{
        pid = $_.ProcessId
        name = $_.Name
        mb = [math]::Round($_.WorkingSetSize / 1MB, 1)
      }
    })
  })

  Start-Sleep -Seconds $IntervalSeconds
}

$totals = $samples | ForEach-Object { $_.totalMb }
$summary = [pscustomobject]@{
  capturedAt = (Get-Date).ToString("o")
  platform = "windows"
  arch = $env:PROCESSOR_ARCHITECTURE
  durationSeconds = $DurationSeconds
  intervalSeconds = $IntervalSeconds
  samples = $samples.Count
  totalMbMin = ($totals | Measure-Object -Minimum).Minimum
  totalMbAvg = [math]::Round((($totals | Measure-Object -Average).Average), 1)
  totalMbMax = ($totals | Measure-Object -Maximum).Maximum
  firstSample = $samples[0]
  lastSample = $samples[$samples.Count - 1]
  series = $samples
}

$directory = Split-Path -Parent $Out
if ($directory) {
  New-Item -ItemType Directory -Force $directory | Out-Null
}

$summary | ConvertTo-Json -Depth 6 | Set-Content -Path $Out -Encoding utf8
Write-Host "Wrote $Out (total min/avg/max: $($summary.totalMbMin)/$($summary.totalMbAvg)/$($summary.totalMbMax) MB)"
