---
doc_id: PAW-INFRA-001
type: INFRA
title: 포인핸드 대형묘 찾기 — 인프라 아키텍처
status: approved
upstream: [PAW-PRD-001, PAW-UC-001]
---

# 인프라 아키텍처

## 0. 이 문서가 다루는 것

앱이 어떤 부품으로 되어 있고, 어디와 통신하고, 무엇을 어디에 두고, 설치 파일을 어떻게 만들어 나눠 주는지를 정한다. 코드 폴더·클래스는 클래스 명세(DOM)가, 화면은 UI 문서가, 화면과 코어 사이 호출의 정확한 모양은 API 문서가 정한다.

버전·주소·건수는 모두 **2026-09-30에 직접 확인한 값**이다.

| 이름 | 값 |
|---|---|
| 보이는 이름(시작 메뉴·설치 마법사·창 제목) | 포인핸드 대형묘 찾기 |
| 영문 이름(실행 파일·설치 파일) | `PawinhandBigCat` |
| 앱 식별자(identifier) | `io.github.hoyoungparkme.pawinhandbigcat` — 이후 바꾸지 않는다(C4) |

## 1. 제약

설계를 묶는 조건이다. 각 제약의 출처를 따라가면 왜 그런지 나온다.

#### C1 서버 없이 사용자 PC 안에서 끝난다

출처: [[PAW-PRD-001#G2]]

PRD 비목표 「자체 서버·DB」. 앱이 포인핸드에 직접 묻고, 받은 데이터는 앱 메모리에만 둔다. 운영할 서버·DB·계정이 없다.

#### C2 Windows 10·11 64비트, 개발 도구 없는 PC에서 돈다

출처: [[PAW-PRD-001#N4]] · [[PAW-PRD-001#G2]]

받는 사람 PC에 Node·Rust·파이썬이 없다고 본다. 앱이 필요로 하는 것은 WebView2 하나이고, 없으면 설치 파일이 받아 깐다(C4).

#### C3 Tauri로 만들고 Windows에서 개발한다

출처: [[PAW-RFQ-001#Q8]]

WSL에서는 리눅스용 결과물이 나오고 화면을 그리는 엔진도 달라서 쓰지 않는다. 개발 PC와 CI 모두 Windows에서 빌드한다.

#### C4 설치 파일은 20MB 이하, 관리자 권한 없이, 덮어 설치된다

출처: [[PAW-PRD-001#R11]] · [[PAW-PRD-001#N3]]

- 현재 사용자 설치(Tauri NSIS `installMode: currentUser`, 기본값)라 관리자 권한이 필요 없다
- WebView2는 필요할 때만 내려받는다(`webviewInstallMode: downloadBootstrapper`, 기본값). 오프라인 설치본을 넣으면 크기가 20MB를 크게 넘는다
- 덮어 설치는 앱 식별자가 같아야 된다. 식별자는 `io.github.hoyoungparkme.pawinhandbigcat`이고 **한 번 정한 뒤 바꾸지 않는다** — 바꾸면 이미 설치한 사람에게 새 버전이 덮어 설치되지 않고 따로 하나 더 깔린다
- 첫 빌드에서 확인했다(2026-09-30): 설치 파일 1.95MB. 0.1.1을 0.1.0 위에 설치하면 「설치된 앱」 항목이 하나로 남고 버전만 바뀐다. 설치 마법사는 깔린 이전 버전을 알아보고 「설치하기 전에 제거하기」를 골라 둔 채 묻는다(조용히 설치(`/S`)하면 묻지 않고 덮어쓴다)

#### C5 코드 서명을 하지 않는다

출처: [[PAW-RFQ-001#Q8]]

설치할 때 「Windows의 PC 보호」 경고가 뜬다. README에 넘기는 법을 그림과 함께 적는다([[PAW-PRD-001#R12]]) — 그림은 실제 창을 본떠 그린 두 장(`.github/images/smartscreen-1.png`·`-2.png`)이고, 실제 창과 모양이 조금 다를 수 있다고 그림에 적어 둔다. 자동 업데이트 기능은 쓰지 않는다(PRD 비목표).

#### C6 데이터 원천은 포인핸드의 비공식 API다

출처: [[PAW-RFQ-001]]

`https://pawinhand.net/bridge/animals/condition`은 포인핸드 사이트가 내부적으로 쓰는 주소이고, 공개·문서화된 API가 아니다. 주소나 응답 형식이 예고 없이 바뀌거나 막힐 수 있다. 앱은 응답을 검사해 실패로 알리고([[PAW-UC-001#UC-S1]] 2b), 배포자가 고쳐 새 버전을 낸다([[PAW-UC-001#UC-A1]]). 포인핸드에 사용 문의는 하지 않기로 했다(고객 결정, 9.1) — 이 위험은 그대로 남는다. 참고로 `pawinhand.kr/robots.txt`는 `Allow: /`이고, `pawinhand.net`에는 robots.txt가 없다(404).

#### C7 포인핸드에 부담을 주지 않는다

출처: [[PAW-PRD-001#N2]]

- 요청은 한 번에 하나씩, 앞 요청이 끝나고 0.3초 뒤에 보낸다. 한 요청은 15초가 넘으면 실패로 본다
- 요청은 켤 때와 새로고침·다시 시도 때만 나간다. 받는 중에 또 받으라고 하면 요청을 새로 보내지 않고 받던 결과를 함께 쓴다
- 한 번에 5번 안팎이다. 쪽마다 1000건을 받되 앞 쪽과 50건을 겹쳐 950건씩 넘긴다 — 받는 사이 목록이 줄어도 빠뜨리지 않게. 4,225건이면 5번이다
- 요청 머리의 User-Agent에 앱 이름·버전·저장소 주소를 밝힌다: `PawinhandBigCat/{버전} (+https://github.com/HoyoungParkme/fourinhand-crawl)`. 이런 형식(앱 이름/버전 + 저장소 주소)으로 보냈을 때 정상 응답하는 것을 확인했다

#### C8 조건 값의 공백은 + 기호로 인코딩한다

출처: [[PAW-PRD-001#R1]]

`모든 지역`의 공백을 `%20`으로 보내면 빈 배열이 온다. 쿼리는 폼 인코딩(공백 → `+`)으로 만들고, 인코딩 결과를 단위 테스트로 고정한다.

#### C9 원문 공고 주소는 받은 그대로 쓰지 않고 새로 만든다

출처: [[PAW-UC-001#UC-S4]] · [[PAW-PRD-001#R9]]

받아온 `detail_url`은 세 종류다.

| 종류 | 건수 | 예 | 열면 |
|---|---|---|---|
| 국가동물보호정보시스템 옛 주소 | 3,715 | `http://www.animal.go.kr/portal_rnl/abandonment/public_view.jsp?desertion_no=445471202600372` | 그 아이가 아니라 **첫 화면**(`/front/index.do`)으로 넘어간다 |
| 포인핸드 상세 | 503 | `https://pawinhand.kr/animal/detail/{공고번호}` | 포인핸드 링크와 같은 곳 |
| `pasm.kr` | 7 | `pasm.kr` | 주소가 아니다 |

옛 주소의 `desertion_no`로 `https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo={번호}`를 열면 그 아이 공고가 나온다(무작위 4건 확인). 그래서 코어는 옛 주소에서 번호를 꺼내 새 주소를 만들고, 번호가 없는 나머지 510건에는 원문 링크를 두지 않는다. [[PAW-UC-001#UC-S4]] 1단계가 이 규칙을 따른다.

#### C10 사진 주소를 https로 맞추고 두 곳만 허용한다

출처: [[PAW-PRD-001#R7]] · [[PAW-PRD-001#R8]]

| 출처 | 건수(사진 칸 기준) | 처리 |
|---|---|---|
| `https://www.animal.go.kr/…` | 6,934 | 그대로 |
| `http://www.animal.go.kr/…` | 871 | https로 바꾼다(바꿔도 열리는 것 확인) |
| `http://d12l2mexpetzlh.cloudfront.net/…` (포인핸드 CDN) | 21 | https로 바꾼다(확인) |
| `more/more_….jpg` 같은 상대 경로 | 25 | 모두 `more_image1` 칸이다. 포인핸드 사이트처럼 `https://d12l2mexpetzlh.cloudfront.net/images/shelter/`를 앞에 붙인다(확인) |

모든 아이가 사진을 하나 이상 가지고 있다. 화면이 불러올 수 있는 외부 주소는 이 두 곳뿐이다(5장).

#### C11 개인정보를 모으지 않는다

출처: [[PAW-PRD-001#N5]]

로그인·사용 통계·오류 보고 서비스가 없다. 로그 파일도 남기지 않는다. 밖으로 나가는 요청은 포인핸드 API와 사진 두 곳, 그리고 사용자가 누른 링크뿐이다.

#### C12 포인핸드 공식 앱이 아님을 밝힌다

출처: 앱 이름 고객 결정(9.1)

앱 이름에 「포인핸드」가 들어가서, 포인핸드가 만든 앱으로 오해할 수 있다. README·설치 안내·앱 정보 화면에 「포인핸드 공식 앱이 아닙니다. 포인핸드에 공개된 보호 동물 공고를 모아 보여줍니다」를 적는다. 포인핸드 로고·상표 그림은 쓰지 않는다 — 앱 아이콘도 따로 만든다.

## 2. 구성도

### 2.1 앱이 돌 때

```mermaid
flowchart LR
    subgraph PC["사용자 PC · Windows 10·11"]
        subgraph APP["앱 (Tauri)"]
            UI["화면<br/>React · WebView2"]
            CORE["코어<br/>Rust"]
        end
        BR["기본 브라우저"]
    end
    API["포인핸드 API<br/>pawinhand.net/bridge"]
    IMG["사진<br/>www.animal.go.kr<br/>포인핸드 CDN"]
    PAGE["공고 페이지<br/>pawinhand.kr<br/>www.animal.go.kr"]
    UI -->|"커맨드 호출"| CORE
    CORE -->|"진행 상태 · 결과"| UI
    CORE -->|"HTTPS GET"| API
    UI -->|"사진 불러오기"| IMG
    CORE -->|"링크 열기"| BR
    BR --> PAGE
```

화면은 포인핸드 API를 직접 부르지 않는다. 받아오기·해석·거르기·링크 만들기는 모두 코어가 하고, 화면은 코어가 준 결과를 그린다. 화면이 직접 여는 외부 연결은 사진뿐이다.

### 2.2 새 버전을 낼 때

```mermaid
flowchart LR
    DEV["배포자<br/>버전 태그 push"] --> GH["GitHub 저장소"]
    GH --> ACT["GitHub Actions<br/>windows-latest"]
    ACT -->|"setup.exe"| REL["GitHub Releases"]
    REL -->|"링크"| USER["입양 희망자 PC<br/>설치 · 덮어 설치"]
```

## 3. 기술 스택

| 영역 | 선택 | 확인한 최신 안정판 | 이유 |
|---|---|---|---|
| 앱 틀 | Tauri | 2.12 | C3. 설치 파일이 작다(C4) |
| 코어 언어 | Rust stable, MSVC 툴체인(`x86_64-pc-windows-msvc`) | — | Tauri의 Windows 빌드 조건 |
| HTTP | reqwest | 0.13 | 쿼리 폼 인코딩(C8), 요청별 시간 제한(C7). TLS는 Windows 기본(Schannel, `native-tls` 기능)을 쓴다 — 0.13 기본값(rustls + aws-lc)은 빌드 도구가 더 필요하다 |
| JSON | serde · serde_json | — | 응답 검사와 변환 |
| 날짜 | chrono | — | 8자리 날짜 읽기, 기간 계산(없는 날은 그 달 말일) |
| 주소 인코딩 | percent-encoding | — | 공고번호를 괄호까지 인코딩해 링크를 만든다(C9) |
| 링크 열기 | tauri-plugin-opener | 2.7 | 코어에서 기본 브라우저를 연다(5장) |
| 진행 상태 전달 | Tauri 채널(`tauri::ipc::Channel`) | — | 받는 중 건수를 화면에 흘려보낸다([[PAW-PRD-001#R10]]) |
| 화면 | React + TypeScript | 19.3 · 7.0 | 카드 목록·상세·필터 상태 |
| 화면 빌드 | Vite | 8.3 | Tauri 공식 안내 조합 |
| 웹 엔진 | WebView2 (Edge) | 이 PC 154 | Windows 11 기본 포함. 없으면 설치 중 받음(C4) |
| 설치 파일 | Tauri 번들러 NSIS | — | 현재 사용자 설치·덮어 설치·한국어 설치 마법사 |
| CI | GitHub Actions + `scripts/build.ps1` + `gh release` | — | [[PAW-UC-001#UC-A1]]. 20MB 확인이 Release를 만들기 전에 끝나야 한다 — `tauri-action`은 빌드와 Release 올리기를 한 단계로 해서 쓰지 않는다. 로컬과 CI가 같은 스크립트로 빌드한다 |
| 테스트 | `cargo test` | — | 몸무게 해석·쿼리 인코딩·주소 정리·거르기를 코어에서 검증. 화면은 1차에서 손으로 확인 |

정확한 버전은 구현할 때 잠금 파일(`Cargo.lock`·`package-lock.json`)로 고정한다.

## 4. 내부 구조

| 부분 | 자리 | 맡는 일 | 유스케이스 |
|---|---|---|---|
| 코어 | `backend/` (Rust, Tauri 설정 포함) | 받아오기, 몸무게 해석, 주소 정리(C9·C10), 조건 거르기·정렬, 링크 열기, 받아온 목록 보관 | [[PAW-UC-001#UC-S1]] · [[PAW-UC-001#UC-S2]] · [[PAW-UC-001#UC-S3]] · [[PAW-UC-001#UC-S4]] |
| 화면 | `frontend/` (React) | 카드·상세·필터 그리기, 지금 조건 들고 있기, 코어 호출 | [[PAW-UC-001#UC-H1]] ~ [[PAW-UC-001#UC-H5]] |
| 설치 파일 | Tauri 번들러 설정(`backend/`) | 설치·덮어 설치·제거 | [[PAW-UC-001#UC-H6]] |
| CI | `.github/workflows/` | 버전 대조·빌드·Release | [[PAW-UC-001#UC-A1]] |

코어가 화면에 여는 커맨드는 셋이다. 이름과 입출력 모양은 API 문서에서 정한다.

1. **받아오기** — 진행 상태를 채널로 보내며 전부 받는다. 다 받았을 때만 보관 중인 목록을 새것으로 바꾼다. 실패하면 이전 목록을 그대로 둔다([[PAW-UC-001#UC-H1]] 2b · [[PAW-UC-001#UC-H5]] 3a). 받는 중에 또 부르면 새로 받지 않고 받던 결과를 함께 받는다(C7)
2. **조회** — 조건(기준 kg·기간·시·도·정렬)을 받아 걸린 목록, 마릿수, 시·도별 마릿수를 돌려준다
3. **링크 열기** — 공고번호와 종류(포인핸드·원문)를 받아 코어가 주소를 만들고, 허용된 곳인지 확인한 뒤 기본 브라우저로 연다

거르기를 화면이 아니라 코어에 두는 이유: 받아온 목록이 코어에 있고, 몸무게 해석과 거르기 규칙을 한 언어로 함께 테스트할 수 있다. 걸린 결과는 보통 수십~수백 건이라 조건을 바꿀 때마다 주고받아도 바로 끝난다. 기준을 0으로 낮춰 4,000마리 넘게 걸려도 0.3초 남짓이고, 화면은 카드를 60장씩 나눠 그린다.

## 5. 인증과 접근

| 무엇 | 어떻게 |
|---|---|
| 사용자 | 로그인이 없다. 설치한 사람이 곧 사용자다 |
| 포인핸드 API | 인증 없는 공개 GET. User-Agent로 앱을 밝힌다(C7) |
| 화면의 권한 | 화면에는 앱이 정의한 커맨드 셋과 앱 버전 읽기(`core:app:allow-version` — 앱 정보에 버전을 보인다)만 연다. 파일·셸·HTTP·링크 열기 플러그인 권한을 화면에 주지 않는다 — 링크는 코어가 연다 |
| 링크 허용 목록 | 코어가 여는 주소는 `https://pawinhand.kr/…`와 `https://www.animal.go.kr/…`뿐이다([[PAW-UC-001#UC-S4]] 2a) |
| 콘텐츠 보안 정책(CSP) | 화면은 앱 자체 파일과 사진 두 곳(`https://www.animal.go.kr`, `https://d12l2mexpetzlh.cloudfront.net`)에서만 불러온다. 나머지 외부 스크립트·연결은 막는다 |
| GitHub Actions | 기본 `GITHUB_TOKEN`에 `contents: write`만 준다(Release 만들기). 서명 키 같은 비밀값이 없다(C5) |
| 저장소 | 공개 저장소. 비밀값이 없어 `.env`를 두지 않는다 |

## 6. 데이터가 사는 곳

| 데이터 | 어디 | 얼마나 | 지우는 법 |
|---|---|---|---|
| 받아온 고양이 목록 | 코어 메모리 | 앱이 켜져 있는 동안 | 앱을 끄면 사라진다 |
| 화면 조건(기준·기간·지역·정렬) | 화면 메모리 | 앱이 켜져 있는 동안. 다시 켜면 기본값 | 앱을 끄면 사라진다 |
| 사진 캐시 | WebView2 데이터 폴더(`%LOCALAPPDATA%\io.github.hoyoungparkme.pawinhandbigcat`) | WebView2가 관리 | 앱 제거 때 「애플리케이션 데이터 삭제하기」를 고르면 함께 지운다. 고르지 않으면 남는다 |
| 앱 파일 | `%LOCALAPPDATA%\포인핸드 대형묘 찾기`(현재 사용자 설치) — `PawinhandBigCat.exe`·`uninstall.exe`. 시작 메뉴 바로 가기, 바탕 화면 바로 가기(설치 마지막 화면에서 고름) | 설치부터 제거까지 | Windows 「앱 제거」 |
| 설치 파일 | GitHub Releases | 버전마다 남는다 | 배포자가 Release를 지운다 |
| 소스와 명세 | GitHub 저장소 `HoyoungParkme/fourinhand-crawl` | 계속 | — |

DB·설정 파일·로그 파일이 없다(C1·C11).

## 7. 외부 변경 감지

앱이 기대는 바깥 형식은 셋이다. 바뀌었을 때 어떻게 알게 되는지 정한다.

| 바뀔 수 있는 것 | 앱에서 드러나는 모습 | 배포자가 아는 법 |
|---|---|---|
| 포인핸드 API 주소·응답 형식(C6) | 받아오기 실패 → 「포인핸드에 연결하지 못했어요」([[PAW-UC-001#UC-S1]] 2b) | 실데이터 점검 명령 |
| 국가동물보호정보시스템 공고 주소 형식(C9) | 원문 링크가 엉뚱한 곳을 연다 | 실데이터 점검 명령 |
| 사진 주소(C10) | 사진 자리에 빈 그림 | 실데이터 점검 명령 |

**실데이터 점검 명령** — 평소 테스트에서는 빠지고 배포자가 부를 때만 돈다(`cargo test -- --ignored`). 실제 API로 한 번 받아 건수·필수 칸을 확인하고, 원문 주소 하나를 열어 그 공고번호가 페이지에 있는지, 사진 하나가 열리는지 본다. 새 버전을 내기 전에 돌린다. 예약 실행(주기 점검)은 1차에 두지 않는다.

## 8. 배치와 운영

### 8.1 개발 PC 준비 (Windows, 한 번)

| 도구 | 설치 | 이 PC 상태(2026-09-30) |
|---|---|---|
| Node.js | — | 24.15 있음 |
| WebView2 | — | 154 있음 |
| Rust (rustup, stable MSVC) | `winget install Rustlang.Rustup` | 설치함(rustup 1.29.1, stable) |
| Visual Studio 2022 Build Tools + 「C++를 사용한 데스크톱 개발」(MSVC·Windows SDK) | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"` | 설치함. 관리자 권한(UAC) 확인이 한 번 뜬다. 약 3~6GB |

Build Tools는 2026판(`Microsoft.VisualStudio.BuildTools` 18.x)도 있지만, Rust·Tauri 조합으로 오래 검증된 2022판을 쓴다.

### 8.2 개발 명령

Tauri CLI는 `frontend/`의 `package.json`에 들어 있고, Tauri 설정은 `backend/`에 있다. CLI는 `TAURI_APP_PATH` 환경 변수가 있으면 그 폴더에서 설정을 찾는다(Tauri CLI 소스 `crates/tauri-cli/src/helpers/app_paths.rs`로 확인 — 변수가 없으면 현재 폴더에서 3단계 아래까지만 찾는다). 이 변수를 매번 치지 않게 명령을 스크립트로 묶는다.

| 할 일 | 명령(저장소 루트에서) | 하는 일 |
|---|---|---|
| 개발 실행 | `pwsh scripts/dev.ps1` | `TAURI_APP_PATH=backend`로 두고 `frontend/`에서 `npm run tauri dev` — Vite 개발 서버를 띄우고 코어를 빌드해 창을 연다 |
| 설치 파일 만들기 | `pwsh scripts/build.ps1` | 같은 방식으로 `npm run tauri build`. 설치 파일 이름을 영문으로 바꾸고(8.3) 20MB를 넘으면 실패한다. 결과는 `backend/target/release/bundle/nsis/` |
| 코어 테스트 | `backend/`에서 `cargo test` | 네트워크 없이 도는 테스트만 |
| 실데이터 점검 | `backend/`에서 `cargo test -- --ignored` | 7장 |

`tauri.conf.json`은 화면을 `../frontend/dist`에서 읽고, 개발 때는 `http://localhost:5173`을 본다. 빌드 전후 명령(`npm run dev`·`npm run build`)은 Tauri가 화면 폴더(`frontend/`)에서 돌린다.

처음 설치 파일을 만들 때 Tauri가 NSIS를 `%LOCALAPPDATA%\tauri`에 받아 푼다. 백신이 막 푼 파일을 검사하느라 폴더 이름 바꾸기가 막히면(「액세스가 거부되었습니다(os error 5)」) `build.ps1`이 반쯤 풀린 폴더(`nsis-*`)를 지우고 10초 뒤 한 번 더 만든다 — Tauri가 NSIS를 처음부터 다시 받는다. 그래도 막히면 조금 뒤 다시 실행한다.

### 8.3 설치 파일 설정

- 대상: NSIS만 만든다(MSI는 만들지 않는다)
- `installMode: currentUser`, `webviewInstallMode: downloadBootstrapper`(C4)
- 설치 마법사 언어: 한국어
- 보이는 이름 「포인핸드 대형묘 찾기」, 실행 파일 `PawinhandBigCat.exe`
- 설치 파일 이름: `PawinhandBigCat_{버전}_x64-setup.exe`. Tauri는 보이는 이름으로 `포인핸드 대형묘 찾기_{버전}_x64-setup.exe`를 만든다. `scripts/build.ps1`이 이것을 영문 이름으로 바꾸고, 20MB를 넘으면 실패로 끝낸다. 한글 보이는 이름으로 NSIS 빌드·설치·시작 메뉴 모두 문제없다(첫 빌드에서 확인)

### 8.4 버전과 배포

1. 버전은 **한 곳**(`backend/`의 Tauri 설정 `version`)에서만 관리한다
2. 배포자가 실데이터 점검(7장)을 돌리고, 버전을 올려 같은 값으로 태그 `v{버전}`을 올린다
3. GitHub Actions(`windows-latest`)가 돈다
   1. 태그와 설정의 버전이 다르면 멈춘다([[PAW-UC-001#UC-A1]] 1a)
   2. Node·Rust를 준비하고 캐시를 쓴다(Rust 캐시 대상 `backend`). 코어 테스트(`cargo test --locked`)를 돈다
   3. `scripts/build.ps1`로 빌드한다 — 로컬과 같은 스크립트다. 설치 파일 이름을 영문으로 바꾸고, 20MB를 넘으면 실패로 끝낸다([[PAW-PRD-001#N3]])
   4. 다 되면 `gh release create`로 태그 이름의 Release를 만들어 `setup.exe`를 올린다. 초안이 아니라 바로 공개하고, 설명에 설치 안내와 비공식 안내(C12)를 적는다
4. 어느 단계든 실패하면 Release가 생기지 않는다([[PAW-UC-001#UC-A1]] 2a)

## 9. 미결사항

### 9.1 사용자가 정할 것

- [x] **앱 이름과 식별자** — 보이는 이름 「포인핸드 대형묘 찾기」, 영문 이름 `PawinhandBigCat`, 식별자 `io.github.hoyoungparkme.pawinhandbigcat`(고객 결정). 이름에 「포인핸드」가 들어가 C12를 둔다
- [x] **포인핸드에 사용 문의** — 하지 않는다(고객 결정). C6의 위험은 그대로 남는다

### 9.2 후속 수정·구현 때 확인 (사용자 결정 불필요)

- [x] [[PAW-UC-001#UC-S4]] 1단계를 C9에 맞게 고친다 — UC v3에서 반영했다
- [x] Tauri CLI를 어느 폴더에서 실행할지 — `frontend/`에서 실행하고 `TAURI_APP_PATH`로 `backend/`를 알려 준다. 8.2에 명령으로 확정했다
- [x] 한글 보이는 이름으로 NSIS 빌드·시작 메뉴가 문제없는지, 설치 파일 이름을 영문으로 맞추는 법 — 첫 빌드에서 빌드·설치·시작 메뉴 모두 문제없었다. 설치 파일 이름은 `build.ps1`이 영문으로 바꾼다(8.3)
- [x] 이전 버전 위에 새 `setup.exe`를 실행했을 때 덮어 설치되는지, 앱 제거 때 WebView2 데이터를 지우는 선택지가 있는지 — 덮어 설치된다(「설치된 앱」 항목 하나, C4). 제거 마법사에 「애플리케이션 데이터 삭제하기」가 있고, 고르면 WebView2 데이터를 지운다(6장)
- [x] GitHub Actions의 빌드 방식 — `tauri-action`은 시험하지 않고 `scripts/build.ps1` + `gh release`로 바꿨다. 20MB 확인이 Release를 만들기 전에 끝나야 하는데 `tauri-action`은 빌드와 Release 올리기를 한 단계로 한다. 3장·8.4를 고쳤다
