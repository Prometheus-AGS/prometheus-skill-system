# Run from the Full pack checkout in PowerShell. Windows native binaries only.
[CmdletBinding()]
param([string]$BinDir = (Join-Path $HOME '.local/bin'))
$ErrorActionPreference = 'Stop'
& node (Join-Path $PSScriptRoot 'binary-release.mjs') windows $BinDir
if ($LASTEXITCODE -ne 3) { exit $LASTEXITCODE }
Write-Host 'No Windows bundle for this pack and architecture. Trying pinned checkout sources via Git Bash.'
$env:PROMETHEUS_BINARY_RELEASE_PROBED = '1'
$env:PROMETHEUS_COMMAND_DIRECTORY = $BinDir
& bash (Join-Path $PSScriptRoot 'install-binaries.sh')
exit $LASTEXITCODE
