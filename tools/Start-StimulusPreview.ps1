param(
    [int]$Port = 8793,
    [string]$ProfilePath = "fixtures\stimulus\interference_preview_profile.json"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot
try {
    $resolvedProfile = Resolve-Path $ProfilePath
    $repoRootText = $RepoRoot.Path.TrimEnd("\") + "\"
    $profileText = $resolvedProfile.Path
    if (!$profileText.StartsWith($repoRootText, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "ProfilePath must be inside the optics repo so the static server can read it: $ProfilePath"
    }
    $relativeProfile = $profileText.Substring($repoRootText.Length)
    $profileUrl = "/" + ($relativeProfile -replace "\\", "/")
    $url = "http://127.0.0.1:$Port/web/stimulus-preview/?profile=$([uri]::EscapeDataString($profileUrl))"

    $process = Start-Process `
        -FilePath "python" `
        -ArgumentList @("-m", "http.server", $Port.ToString(), "--bind", "127.0.0.1") `
        -WorkingDirectory $RepoRoot `
        -PassThru `
        -WindowStyle Hidden

    $pidPath = Join-Path $RepoRoot ".stimulus-preview.pid"
    Set-Content -Path $pidPath -Value $process.Id
    Write-Output "Stimulus preview: $url"
    Write-Output "Server PID written to $pidPath"
} finally {
    Pop-Location
}
