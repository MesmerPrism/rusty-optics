$ErrorActionPreference = "Stop"

function Invoke-Checked {
    param(
        [Parameter(Mandatory=$true)]
        [string]$Name,
        [Parameter(Mandatory=$true)]
        [string]$File,
        [string[]]$Arguments = @()
    )

    & $File @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot
try {
    Invoke-Checked "cargo fmt" "cargo" @("fmt", "--all", "--check")
    Invoke-Checked "cargo test" "cargo" @("test", "--workspace")
    Invoke-Checked "fixture export check" "cargo" @("run", "-p", "rusty-optics-fixtures", "--", "export", "--check")
    Invoke-Checked "schema export check" "cargo" @("run", "-p", "rusty-optics-schema", "--", "export", "--check")
    Invoke-Checked "Optics boundary scan" "python" @("tools\check_optics_boundaries.py")
} finally {
    Pop-Location
}

