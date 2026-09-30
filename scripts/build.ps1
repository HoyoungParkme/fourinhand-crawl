# 설치 파일 만들기 — backend/target/release/bundle/nsis/PawinhandBigCat_{버전}_x64-setup.exe (INFRA 8.2·8.3)
# Tauri CLI는 frontend/의 package.json에 있고 설정은 backend/에 있다. TAURI_APP_PATH로 설정 폴더를 알려 준다
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$backend = Join-Path $root 'backend'
$frontend = Join-Path $root 'frontend'
$env:TAURI_APP_PATH = $backend

if (-not (Test-Path (Join-Path $frontend 'node_modules'))) {
    npm ci --prefix $frontend
    if ($LASTEXITCODE -ne 0) { throw 'npm ci 실패' }
}

Push-Location $frontend
try {
    npm run tauri -- build
    if ($LASTEXITCODE -ne 0) {
        # 처음 NSIS를 받아 풀 때 백신이 막 푼 파일을 검사하느라 폴더 이름 바꾸기가 막힐 수 있다(os error 5).
        # 반쯤 풀린 폴더를 지우고 검사가 끝나기를 잠시 기다린 뒤 한 번 더 한다 — Tauri가 NSIS를 처음부터 다시 받는다
        $leftover = Get-ChildItem (Join-Path $env:LOCALAPPDATA 'tauri') -Directory -Filter 'nsis-*' -ErrorAction SilentlyContinue
        if (-not $leftover) { throw 'tauri build 실패' }
        Start-Sleep -Seconds 10
        $leftover | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
        npm run tauri -- build
        if ($LASTEXITCODE -ne 0) { throw 'tauri build 실패' }
    }
} finally {
    Pop-Location
}

# Tauri는 보이는 이름(한글)으로 설치 파일 이름을 짓는다 — 영문 이름으로 바꾼다
$version = (Get-Content (Join-Path $backend 'tauri.conf.json') -Raw -Encoding utf8 | ConvertFrom-Json).version
$nsis = Join-Path $backend 'target/release/bundle/nsis'
$setup = Join-Path $nsis "PawinhandBigCat_${version}_x64-setup.exe"
$built = Get-ChildItem $nsis -Filter "*_${version}_x64-setup.exe" |
    Where-Object { $_.FullName -ne $setup } |
    Sort-Object LastWriteTime |
    Select-Object -Last 1
if (-not $built) { throw "설치 파일을 찾지 못했다: $nsis" }
Move-Item $built.FullName $setup -Force

# 설치 파일은 20MB 이하(PRD N3)
$size = (Get-Item $setup).Length
if ($size -gt 20MB) { throw ('설치 파일이 20MB를 넘는다: {0:N1}MB' -f ($size / 1MB)) }
Write-Host ('설치 파일: {0} ({1:N1}MB)' -f $setup, ($size / 1MB))
