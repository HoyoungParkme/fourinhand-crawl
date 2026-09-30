# 개발 실행 — Vite 개발 서버를 띄우고 코어를 빌드해 창을 연다(INFRA 8.2)
# Tauri CLI는 frontend/의 package.json에 있고 설정은 backend/에 있다. TAURI_APP_PATH로 설정 폴더를 알려 준다
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$frontend = Join-Path $root 'frontend'
$env:TAURI_APP_PATH = Join-Path $root 'backend'

if (-not (Test-Path (Join-Path $frontend 'node_modules'))) {
    npm ci --prefix $frontend
    if ($LASTEXITCODE -ne 0) { throw 'npm ci 실패' }
}

Push-Location $frontend
try {
    npm run tauri -- dev @args
} finally {
    Pop-Location
}
