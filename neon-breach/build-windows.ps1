$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    cargo test --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    cargo build --locked --release
    if ($LASTEXITCODE -ne 0) { throw 'Build failed' }
    New-Item -ItemType Directory -Force dist | Out-Null
    Copy-Item target/release/neon-breach.exe dist/
    Copy-Item README.md dist/
    Write-Host 'Ready: dist/neon-breach.exe'
} finally { Pop-Location }
