---
doc_id: PAW-CODE-001
type: CODE
title: 포인핸드 대형묘 찾기 — 구현 계획
status: draft
upstream: [PAW-MS-001, PAW-DOM-002, PAW-API-001, PAW-UI-001, PAW-SEQ-001, PAW-SCN-001, PAW-INFRA-001]
---

# 구현 계획 — 포인핸드 대형묘 찾기

## 0. 이 문서가 다루는 것

명세를 코드로 옮기는 순서를 시나리오 단위의 세로 슬라이스로 정한다. 슬라이스 하나가 끝나면 그 시나리오가 앱에서 실제로 돈다. 함수의 내용은 [[PAW-MS-001]], 파일 자리는 [[PAW-DOM-002]] 1장이 정본이다.

- 작업 브랜치 `feat/app`에서 슬라이스마다 커밋하고, 다 되면 `main`에 squash 머지한다(저장소 `CLAUDE.md`의 브랜치 전략)
- 커밋 메시지에 에이전트 표시 줄을 넣지 않는다(규약 1.10)
- 완료 칸에는 그 슬라이스의 커밋을 적는다

## 1. 슬라이스

#### A 기반

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-INFRA-001]] · [[PAW-DOM-002]] 1장 |
| 구현 | `backend/` 크레이트(Cargo.toml · build.rs · tauri.conf.json · capabilities · main.rs · core/config.rs · core/error.rs · 빈 domains/animal 모듈), `frontend/`(Vite + React + TypeScript, styles.css에 UI 문서 3장 토큰, SUIT 글꼴 파일과 OFL 라이선스), `scripts/dev.ps1`·`build.ps1`, `.gitignore`, `README.md`·`AGENTS.md` |
| API | — |
| 화면 | 빈 창(제목 「포인핸드 대형묘 찾기」, 1280×800, 최소 1024×680) |
| 테스트 | `cargo build` · `npm run build` 통과, `scripts/dev.ps1`로 창이 뜬다 |
| 선행 | 없음 |
| 완료 | — |

#### B1 켜서 큰 고양이 목록 보기

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-SCN-001#S1]] · [[PAW-UC-001#UC-H1]] · [[PAW-SEQ-001#SEQ-1]] |
| 구현 함수 | [[PAW-MS-001#Weight.parse]] · [[PAW-MS-001#Weight.at_least]] · [[PAW-MS-001#NoticePeriod.status]] · [[PAW-MS-001#Photo.from_raw]] · [[PAW-MS-001#Snapshot.new]] · [[PAW-MS-001#Fetch.record_page]] · [[PAW-MS-001#AnimalStore.current]] · [[PAW-MS-001#AnimalStore.replace]] · [[PAW-MS-001#AnimalStore.try_begin_load]] · [[PAW-MS-001#AnimalService.load]] · [[PAW-MS-001#AnimalService.query]] · [[PAW-MS-001#PawinhandSource.fetch_page]] · [[PAW-MS-001#PawinhandSource.page_url]] · [[PAW-MS-001#PawinhandSource.to_animal]] · [[PAW-MS-001#PawinhandSource.source_no]] · [[PAW-MS-001#commands.load_animals]] · [[PAW-MS-001#commands.query_animals]] |
| API | [[PAW-API-001#load_animals]] · [[PAW-API-001#query_animals]] |
| 화면 | [[PAW-UI-001#UI-1]] — 윗줄, 카드 격자, 받는 중(6), 첫 실패(7), 사진 없음(10) |
| 테스트 | 위 함수의 테스트 관점 전부, 실데이터로 켜서 15마리 안팎이 무거운 순으로 뜬다 |
| 선행 | A |
| 완료 | — |

#### B2 조건을 바꿔 찾기

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-SCN-001#S2]] · [[PAW-UC-001#UC-H2]] · [[PAW-SEQ-001#SEQ-2]] |
| 구현 함수 | [[PAW-MS-001#SearchCondition.validate]] · [[PAW-MS-001#SearchCondition.since]] · [[PAW-MS-001#Region.matches]] · [[PAW-MS-001#AnimalService.query]](시·도별 마릿수·지역 초기화·정렬 둘) |
| API | [[PAW-API-001#query_animals]] |
| 화면 | [[PAW-UI-001#UI-1]] — 몸무게 기준 칸과 눈금자, 등록 기간, 지역 목록, 정렬, 0마리(9), 잘못된 기준(11) |
| 테스트 | 조회 표본(경기도 4 · 경기도+3개월 2 · 전국+3개월 6), 가장 마지막 요청만 그리기, 숫자 칸 150ms |
| 선행 | B1 |
| 완료 | — |

#### B3 자세히 보고 포인핸드로

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-SCN-001#S3]] · [[PAW-UC-001#UC-H3]] · [[PAW-UC-001#UC-H4]] · [[PAW-SEQ-001#SEQ-4]] |
| 구현 함수 | [[PAW-MS-001#Link.for_animal]] · [[PAW-MS-001#Snapshot.find]] · [[PAW-MS-001#AnimalService.open_link]] · [[PAW-MS-001#TauriOpener.open]] · [[PAW-MS-001#commands.open_link]] |
| API | [[PAW-API-001#open_link]] |
| 화면 | [[PAW-UI-001#UI-2]] 전부, [[PAW-UI-001#UI-1]]의 카드 바로 가기(5.6)와 그 실패 |
| 테스트 | 링크 주소(괄호 인코딩 포함), 가짜 열개로 실패 → `open-failed`와 주소, 실제 앱에서 포인핸드·원문 페이지가 기본 브라우저로 열린다 |
| 선행 | B1 |
| 완료 | — |

#### B4 새로고침

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-SCN-001#S4]] · [[PAW-UC-001#UC-H5]] · [[PAW-SEQ-001#SEQ-3]] |
| 구현 함수 | [[PAW-MS-001#AnimalService.load]](받는 중 `busy`, 실패 때 옛 목록 유지) |
| API | [[PAW-API-001#load_animals]] |
| 화면 | [[PAW-UI-001#UI-1]] — 새로고침(1.3), 새로고침 실패 띠(8), 지역 초기화 안내 띠 |
| 테스트 | 가짜 원천으로 둘째 쪽 실패 → 옛 스냅숏 그대로, 받는 중 다시 부르기 → `busy` |
| 선행 | B2 |
| 완료 | — |

#### B5 앱 정보

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-INFRA-001#C12]] |
| 구현 함수 | — (화면만) |
| API | — |
| 화면 | [[PAW-UI-001#UI-3]] |
| 테스트 | 버전이 설정의 버전과 같다, 「포인핸드 공식 앱이 아닙니다」가 보인다 |
| 선행 | B1 |
| 완료 | — |

#### C 설치 파일과 배포

| 항목 | 내용 |
|---|---|
| 근거 | [[PAW-SCN-001#S1]] 설치 · [[PAW-SCN-001#S5]] · [[PAW-UC-001#UC-H6]] · [[PAW-UC-001#UC-A1]] · [[PAW-INFRA-001]] 8장 |
| 구현 | 번들 설정(NSIS, 현재 사용자 설치, 한국어, WebView2 필요할 때 받기), 앱 아이콘(포인핸드 로고 아님), 설치 파일 이름 영문화, `.github/workflows/release.yml`(버전 대조 · 빌드 · 20MB 확인 · Release), README 설치 안내(PC 보호 경고 넘기기) |
| API | — |
| 화면 | 설치 마법사(Tauri NSIS) |
| 테스트 | 로컬 `scripts/build.ps1`로 setup.exe가 20MB 이하, 조용히 설치 → 실행 → 제거, 덮어 설치, 태그를 올려 Release에 setup.exe가 생긴다 |
| 선행 | A · B1 |
| 완료 | — |

## 2. 통합 테스트

| 시나리오 | 슬라이스 | 검증하는 것 |
|---|---|---|
| [[PAW-SCN-001#S1]] 처음 설치하고 첫 목록 | C · B1 | setup.exe로 설치한 앱을 켜서 실데이터 목록이 10초 안에 뜬다 |
| [[PAW-SCN-001#S2]] 조건 바꿔 찾기 | B2 | 지역·기간·기준·정렬을 바꾸면 다시 받지 않고 바로 바뀐다 |
| [[PAW-SCN-001#S3]] 자세히 보고 포인핸드로 | B3 | 상세의 정보·원래 값, 링크가 그 공고번호의 페이지를 연다 |
| [[PAW-SCN-001#S4]] 다시 켜서 확인 | B4 | 새로고침으로 받은 시각이 바뀌고, 인터넷을 끊으면 실패 띠만 뜨고 목록은 그대로 |
| [[PAW-SCN-001#S5]] 새 버전 배포 | C | 태그 push → Actions → Release에 setup.exe |

화면은 1차에서 손으로 확인한다([[PAW-INFRA-001]] 3장). 실제 앱 창을 띄워 캡처해 UI 문서의 시안과 맞춰 본다.

## 3. 커밋·PR 목록

슬라이스 카드의 `완료` 행에 기록한다. 작업 브랜치 `feat/app`의 커밋을 슬라이스마다 나누고, `main`에는 squash 머지 한 번으로 올린다.

## 4. 미결사항

- [ ] **클래스 명세 후속** — [[PAW-DOM-002#AnimalSource]]의 `fetch_page`가 `Vec<Animal>` 대신 `Page { raw_count, animals }`를 돌려주게 맞춘다([[PAW-MS-001#AnimalService.load]]에서 찾은 것). 오류 열거형의 자리(`models.rs`)도 적는다. 구현을 마친 뒤 코드와 함께 맞춘다
