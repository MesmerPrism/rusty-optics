param(
    [string]$OutputDir = "",
    [string]$ThreeVersion = "0.184.0"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "local-artifacts\web3d"
}

New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$threeModuleUrl = "https://unpkg.com/three@$ThreeVersion/build/three.module.js"
$threeCoreUrl = "https://unpkg.com/three@$ThreeVersion/build/three.core.js"
$threePath = Join-Path $OutputDir "three.module.js"
$threeCorePath = Join-Path $OutputDir "three.core.js"
$metadataPath = Join-Path $OutputDir "three.module.meta.json"

Invoke-WebRequest -Uri $threeModuleUrl -OutFile $threePath
Invoke-WebRequest -Uri $threeCoreUrl -OutFile $threeCorePath

$hash = Get-FileHash -Path $threePath -Algorithm SHA256
$coreHash = Get-FileHash -Path $threeCorePath -Algorithm SHA256
$metadata = [ordered]@{
    package = "three"
    version = $ThreeVersion
    module_url = $threeModuleUrl
    core_url = $threeCoreUrl
    license = "MIT"
    module_sha256 = $hash.Hash
    core_sha256 = $coreHash.Hash
    fetched_at_utc = (Get-Date).ToUniversalTime().ToString("o")
}
$metadata | ConvertTo-Json | Set-Content -Path $metadataPath -Encoding UTF8

Get-Item -Path $threePath, $threeCorePath, $metadataPath | Select-Object FullName, Length
