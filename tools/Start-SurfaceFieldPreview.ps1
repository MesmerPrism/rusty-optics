param(
    [int]$Port = 8792,
    [string]$FramePath = ""
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
    Invoke-Checked "surface field preview fixture export" "cargo" @(
        "run",
        "-p",
        "rusty-optics-fixtures",
        "--",
        "export-surface-field-preview"
    )

    $url = "http://127.0.0.1:$Port/web/surface-field-preview/"
    if ($FramePath.Trim().Length -gt 0) {
        $ResolvedFrame = Resolve-Path $FramePath
        $repoRootText = $RepoRoot.Path.TrimEnd("\") + "\"
        $frameText = $ResolvedFrame.Path
        if (!$frameText.StartsWith($repoRootText, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "FramePath must be inside the optics repo so the static server can read it: $FramePath"
        }
        $relativeFrame = $frameText.Substring($repoRootText.Length)
        $frameUrl = "/" + ($relativeFrame -replace "\\", "/")
        $url = "${url}?frame=$([uri]::EscapeDataString($frameUrl))"
    }

    $process = Start-Process `
        -FilePath "python" `
        -ArgumentList @("-m", "http.server", $Port.ToString(), "--bind", "127.0.0.1") `
        -WorkingDirectory $RepoRoot `
        -PassThru `
        -WindowStyle Hidden

    $pidPath = Join-Path $RepoRoot ".surface-field-preview.pid"
    Set-Content -Path $pidPath -Value $process.Id
    Write-Output "Surface field preview: $url"
    Write-Output "Server PID written to $pidPath"
} finally {
    Pop-Location
}
