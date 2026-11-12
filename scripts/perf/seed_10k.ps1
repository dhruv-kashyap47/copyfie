param(
  [int]$Items = 10000,
  [string]$Db = (Join-Path $env:APPDATA "com.copyfie.app\clipboard.db")
)

# Seeds a deterministic text fixture into a Copyfie database.
# Close Copyfie first: the store actor owns the file while the app runs.

$ErrorActionPreference = "Stop"

if (Get-Process copyfie -ErrorAction SilentlyContinue) {
  throw "Copyfie is running. Quit it before seeding $Db."
}

$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

Write-Host "Seeding $Items items into $Db"
cargo run -p copyfie-core --example perf --profile bench -- --db $Db --items $Items --queries 10
