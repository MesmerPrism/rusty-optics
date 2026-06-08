param(
    [string]$MatterRepoRoot = "",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [string]$File,
        [string[]]$Arguments = @()
    )

    & $File @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
if ([string]::IsNullOrWhiteSpace($MatterRepoRoot)) {
    $MatterRepoRoot = Resolve-Path (Join-Path $RepoRoot "..\rusty-matter")
} else {
    $MatterRepoRoot = Resolve-Path $MatterRepoRoot
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "local-artifacts\matter_surface_field_wasm"
}

$MatterBuildScript = Join-Path $MatterRepoRoot "tools\Build-SurfaceFieldWasmRuntime.ps1"
if (-not (Test-Path $MatterBuildScript)) {
    throw "Matter surface-field Wasm build script not found: $MatterBuildScript"
}

Invoke-Checked "Matter surface-field Wasm build" "powershell" @(
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    $MatterBuildScript,
    "-OutputDir",
    $OutputDir
)
