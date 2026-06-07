param(
    [int]$Port = 8791
)

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
    Invoke-Checked "hand mesh browser fixture export" "cargo" @(
        "run",
        "-p",
        "rusty-optics-fixtures",
        "--",
        "export-hand-mesh-browser"
    )

    $process = Start-Process `
        -FilePath "python" `
        -ArgumentList @("-m", "http.server", $Port.ToString(), "--bind", "127.0.0.1") `
        -WorkingDirectory $RepoRoot `
        -PassThru `
        -WindowStyle Hidden

    $pidPath = Join-Path $RepoRoot ".hand-mesh-browser-preview.pid"
    Set-Content -Path $pidPath -Value $process.Id
    Write-Output "Hand mesh browser preview: http://127.0.0.1:$Port/web/hand-mesh-browser-preview/"
    Write-Output "Server PID written to $pidPath"
} finally {
    Pop-Location
}
