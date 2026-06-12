param(
    [int]$Port = 8792,
    [string]$BaseUrl = "",
    [string]$OutputDir = "",
    [int]$Width = 720,
    [int]$Height = 860,
    [int]$Fps = 12,
    [int]$DurationSeconds = 8,
    [int]$Step = 1,
    [int]$Warmup = 0,
    [string]$Views = "surface,graph",
    [ValidateSet("gif", "apng")]
    [string]$Format = "gif",
    [int]$Density = 3,
    [int]$ReadyTimeoutMs = 120000,
    [int]$ExportTimeoutMs = 300000,
    [string]$NodeExe = "",
    [string]$NodeModulePath = "",
    [string]$PythonExe = "",
    [switch]$KeepServer
)

$ErrorActionPreference = "Stop"

function Resolve-ToolPath {
    param(
        [string]$Configured,
        [string]$Bundled,
        [string]$Fallback
    )

    if (-not [string]::IsNullOrWhiteSpace($Configured)) {
        return $Configured
    }
    if (-not [string]::IsNullOrWhiteSpace($Bundled) -and (Test-Path $Bundled)) {
        return $Bundled
    }
    return $Fallback
}

function Wait-PreviewServer {
    param(
        [string]$Url,
        [int]$TimeoutMs
    )

    $deadline = [DateTimeOffset]::UtcNow.AddMilliseconds($TimeoutMs)
    while ([DateTimeOffset]::UtcNow -lt $deadline) {
        try {
            $response = Invoke-WebRequest -Uri $Url -UseBasicParsing -TimeoutSec 2
            if ($response.StatusCode -ge 200 -and $response.StatusCode -lt 500) {
                return
            }
        } catch {
            Start-Sleep -Milliseconds 500
        }
    }
    throw "Preview server did not answer before timeout: $Url"
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$BundledNode = "C:\Users\tillh\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\bin\node.exe"
$BundledNodeModules = "C:\Users\tillh\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\node_modules"
$BundledPnpmModules = "C:\Users\tillh\.cache\codex-runtimes\codex-primary-runtime\dependencies\node\node_modules\.pnpm\node_modules"
$BundledPython = "C:\Users\tillh\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe"

$NodeExe = Resolve-ToolPath $NodeExe $BundledNode "node"
$PythonExe = Resolve-ToolPath $PythonExe $BundledPython "python"
if ([string]::IsNullOrWhiteSpace($NodeModulePath) -and (Test-Path $BundledNodeModules)) {
    $NodeModulePath = $BundledNodeModules
    if (Test-Path $BundledPnpmModules) {
        $NodeModulePath = "$NodeModulePath;$BundledPnpmModules"
    }
}
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "local-artifacts\planarian3d-export-smoke"
}

$MatterWasm = Join-Path $RepoRoot "local-artifacts\matter_surface_field_wasm\rusty_matter_fields_wasm.js"
$ThreeModule = Join-Path $RepoRoot "local-artifacts\web3d\three.module.js"
if (-not (Test-Path $MatterWasm)) {
    throw "Missing Matter Wasm artifact: $MatterWasm. Build it with tools\Build-SurfaceFieldPreviewMatterWasm.ps1."
}
if (-not (Test-Path $ThreeModule)) {
    throw "Missing Three.js artifact: $ThreeModule. Build it with tools\Build-SurfaceFieldPreviewWeb3DDeps.ps1."
}

$server = $null
$previousNodePath = $env:NODE_PATH
Push-Location $RepoRoot
try {
    if ([string]::IsNullOrWhiteSpace($BaseUrl)) {
        $BaseUrl = "http://127.0.0.1:$Port/web/surface-field-preview/"
        $server = Start-Process `
            -FilePath $PythonExe `
            -ArgumentList @("-m", "http.server", $Port.ToString(), "--bind", "127.0.0.1") `
            -WorkingDirectory $RepoRoot `
            -PassThru `
            -WindowStyle Hidden
        Wait-PreviewServer -Url $BaseUrl -TimeoutMs $ReadyTimeoutMs
    }

    if (-not [string]::IsNullOrWhiteSpace($NodeModulePath)) {
        if ([string]::IsNullOrWhiteSpace($previousNodePath)) {
            $env:NODE_PATH = $NodeModulePath
        } else {
            $env:NODE_PATH = "$NodeModulePath;$previousNodePath"
        }
    }

    $nodeArgs = @(
        "tools\planarian_3d_export_smoke.cjs",
        "--url", $BaseUrl,
        "--out-dir", $OutputDir,
        "--format", $Format,
        "--views", $Views,
        "--palette", "neon-rgb",
        "--start", "reset",
        "--loop", "showcase",
        "--layer", "activity",
        "--material", "opaque",
        "--seconds", $DurationSeconds.ToString(),
        "--fps", $Fps.ToString(),
        "--width", $Width.ToString(),
        "--height", $Height.ToString(),
        "--step", $Step.ToString(),
        "--warmup", $Warmup.ToString(),
        "--density", $Density.ToString(),
        "--ready-timeout-ms", $ReadyTimeoutMs.ToString(),
        "--export-timeout-ms", $ExportTimeoutMs.ToString(),
        "--python", $PythonExe
    )
    & $NodeExe @nodeArgs
    if ($LASTEXITCODE -ne 0) {
        throw "Planarian 3D export smoke failed with exit code $LASTEXITCODE"
    }
} finally {
    $env:NODE_PATH = $previousNodePath
    if ($server -and -not $KeepServer) {
        Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
    }
    Pop-Location
}
