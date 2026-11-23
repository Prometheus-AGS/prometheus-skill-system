# Local service providers for PowerShell. Requires Node.js; Docker selections
# require a local Docker Desktop Linux-container engine and Compose v2.
& node (Join-Path $PSScriptRoot 'service-providers/cli.mjs') @args
exit $LASTEXITCODE
