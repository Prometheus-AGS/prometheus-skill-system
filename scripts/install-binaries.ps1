# Run from the Full pack checkout in PowerShell. Windows native binaries only.
[CmdletBinding()]
param([string]$BinDir = (Join-Path $HOME '.local/bin'))
$ErrorActionPreference = 'Stop'
$products = @()
if (Test-Path (Join-Path $HOME '.prometheus/service-providers/full.json')) {
    $products = @(& node (Join-Path $PSScriptRoot 'service-providers/binaries.mjs') windows)
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
& node (Join-Path $PSScriptRoot 'binary-release.mjs') windows $BinDir @products
if ($LASTEXITCODE -ne 3) { exit $LASTEXITCODE }
Write-Host 'No Windows bundle for this pack and architecture. Trying pinned checkout sources via Git Bash.'
$env:PROMETHEUS_BINARY_RELEASE_PROBED = '1'
$env:PROMETHEUS_COMMAND_DIRECTORY = $BinDir
& bash (Join-Path $PSScriptRoot 'install-binaries.sh')
exit $LASTEXITCODE
