---
doc_id: PAW-SEQ-001
type: SEQ
title: 포인핸드 대형묘 찾기 — 시퀀스
status: draft
upstream: [PAW-UC-001, PAW-API-001, PAW-DOM-002, PAW-UI-001, PAW-INFRA-001]
---

# SEQUENCE — 포인핸드 대형묘 찾기

## 0. 이 문서가 다루는 것

유스케이스의 흐름을 클래스 명세의 생명선 사이 호출로 그린다. 누가 먼저 부르는지, 목록이 언제 바뀌는지, 실패하면 어디서 멈추는지를 본다. 커맨드의 모양은 [[PAW-API-001]], 생명선의 정의는 [[PAW-DOM-002]]가 정본이다.

### 0.1 생명선

| 생명선 | 약어 | 실체 | 종류 | 정의한 곳 |
|---|---|---|---|---|
| 입양 희망자 | U | 앱을 쓰는 사람 | 액터 | [[PAW-UC-001]] 1장 |
| 목록 화면 | LP | `frontend/src/pages/ListPage.tsx` | 화면 | [[PAW-DOM-002#ListPage]] |
| 화면 상태 | APP | `frontend/src/App.tsx` | 화면 | [[PAW-DOM-002#App]] |
| 화면 쪽 호출 | API | `frontend/src/api/animals.ts` | 화면 | [[PAW-DOM-002#AnimalsApi]] |
| 커맨드 | CMD | `backend/src/domains/animal/commands.rs` | 코어 | [[PAW-DOM-002#AnimalCommands]] |
| 동물 서비스 | SVC | `AnimalService` | 코어 | [[PAW-DOM-002#AnimalService]] |
| 보관소 | ST | `AnimalStore` | 코어 | [[PAW-DOM-002#AnimalStore]] |
| 포인핸드 어댑터 | PS | `PawinhandSource` | 코어 | [[PAW-DOM-002#PawinhandSource]] |
| 포인핸드 API | PAW | `https://pawinhand.net/bridge/animals/condition` | 외부 | [[PAW-INFRA-001#C6]] |
| 브라우저 어댑터 | OP | `TauriOpener` | 코어 | [[PAW-DOM-002#TauriOpener]] |
| 기본 브라우저 | BR | 사용자 PC의 기본 브라우저 | 외부 | [[PAW-UC-001]] 1장 |

배포 흐름(SEQ-5)에는 배포자, GitHub 저장소·Actions·Releases가 따로 나온다.

## SEQ-1 켜서 받아오고 첫 목록 그리기

[[PAW-UC-001#UC-H1]] 기본 흐름 1~4, [[PAW-UC-001#UC-S1]] · [[PAW-UC-001#UC-S3]].

```mermaid
sequenceDiagram
    autonumber
    actor U as 입양 희망자
    participant APP as App
    participant API as api/animals.ts
    participant CMD as commands
    participant SVC as AnimalService
    participant ST as AnimalStore
    participant PS as PawinhandSource
    participant PAW as 포인핸드 API
    U->>APP: 앱을 켠다
    APP->>API: loadAnimals(onProgress)
    API->>CMD: invoke load_animals (진행 채널)
    CMD->>SVC: load(on_page)
    SVC->>ST: try_begin_load()
    ST-->>SVC: LoadGuard
    loop 1000건보다 적게 올 때까지 (최대 20쪽)
        SVC->>PS: fetch_page(offset, 1000)
        PS->>PAW: GET 고양이 · 보호중 · 2020-01-01~오늘 · offset · limit=1000
        PAW-->>PS: JSON 배열
        PS-->>SVC: Animal 목록 (칸 옮기기 · 몸무게 해석)
        SVC->>SVC: fetch.record_page(n)
        SVC-->>CMD: on_page(received)
        CMD--)API: 채널 received
        API--)APP: UI-1의 6에 받은 마릿수
        Note over SVC: 더 받을 때만 0.3초 쉰다
    end
    SVC->>ST: replace(Snapshot)
    SVC-->>CMD: Snapshot
    CMD-->>API: fetchedAt · total
    API-->>APP: LoadResult
    APP->>API: queryAnimals(8 · all · all · weight)
    API->>CMD: invoke query_animals
    CMD->>SVC: query(condition, 오늘)
    SVC->>ST: current()
    ST-->>SVC: Snapshot
    SVC-->>CMD: SearchResult
    CMD-->>API: count · regionCounts · animals
    API-->>APP: QueryResult
    APP-->>U: 카드 목록 · N마리 · 받은 시각
```

**읽을 때 볼 것**
- `replace`는 마지막 쪽까지 받은 뒤 **한 번만** 불린다. 받는 동안 보관소의 목록은 바뀌지 않는다([[PAW-DOM-002#AnimalStore]])
- `LoadGuard`는 `load`가 끝날 때 떨어져 「받는 중」이 풀린다. 성공이든 실패든 같다
- 진행 상태는 서비스가 아니라 커맨드가 채널로 보낸다 — 서비스는 Tauri를 모른다(클래스 명세 3장)
- 화면은 `load_animals`가 끝난 뒤에야 `query_animals`를 부른다([[PAW-API-001]] 3장). 오늘 날짜는 커맨드가 부를 때마다 읽는다

## SEQ-2 조건을 바꿔 다시 거르기

[[PAW-UC-001#UC-H2]] 기본 흐름 1~4, [[PAW-UC-001#UC-S3]].

```mermaid
sequenceDiagram
    autonumber
    actor U as 입양 희망자
    participant LP as ListPage
    participant APP as App
    participant API as api/animals.ts
    participant CMD as commands
    participant SVC as AnimalService
    U->>LP: 지역에서 경기도를 누른다
    LP->>APP: changeCondition(region 경기도)
    APP->>APP: 요청 번호 7을 매긴다
    APP->>API: queryAnimals(condition)
    API->>CMD: invoke query_animals
    CMD->>SVC: query(condition, 오늘)
    SVC-->>CMD: SearchResult (4마리 · 시·도별 마릿수)
    CMD-->>API: QueryResultDto
    API-->>APP: 요청 7의 결과
    alt 요청 7이 가장 마지막 번호
        APP-->>LP: 목록 · 4마리 · 지역 숫자를 그린다
    else 그 뒤에 부른 요청이 있다
        APP->>APP: 이 결과를 버린다
    end
    U->>LP: 몸무게 칸에 7을 넣는다
    Note over LP,APP: 입력이 멈추고 150ms 뒤에만 부른다
    LP->>APP: changeCondition(min 7)
    Note over APP,SVC: 같은 흐름을 한 번 더 — 포인핸드에는 요청하지 않는다
```

**읽을 때 볼 것**
- 이 흐름에는 포인핸드 API가 없다. 거르기는 코어 메모리만 본다([[PAW-API-001]] 1장 「네트워크」)
- 결과가 뒤바뀌어 도착해도 가장 마지막에 부른 요청의 결과만 그린다([[PAW-DOM-002#App]])
- 숫자 칸에 잘못된 값이 들어오면 `changeCondition`을 부르지 않는다 — 화면이 먼저 막는다(UI-1의 11)

## SEQ-3 새로고침이 실패해도 보던 목록은 그대로

[[PAW-UC-001#UC-H5]] 확장 1a · 3a, [[PAW-UC-001#UC-S1]] 확장 2a.

```mermaid
sequenceDiagram
    autonumber
    actor U as 입양 희망자
    participant APP as App
    participant API as api/animals.ts
    participant CMD as commands
    participant SVC as AnimalService
    participant ST as AnimalStore
    participant PS as PawinhandSource
    participant PAW as 포인핸드 API
    U->>APP: 새로고침을 누른다
    APP->>APP: 새로고침 버튼을 막는다
    APP->>API: loadAnimals(onProgress)
    API->>CMD: invoke load_animals
    CMD->>SVC: load(on_page)
    SVC->>ST: try_begin_load()
    alt 이미 받는 중
        ST-->>SVC: 없음
        SVC-->>CMD: Busy
        CMD-->>API: code busy
        API-->>APP: 무시한다
    else 받기 시작
        ST-->>SVC: LoadGuard
        SVC->>PS: fetch_page(0, 1000)
        PS->>PAW: GET 첫 쪽
        PAW-->>PS: 1000건
        PS-->>SVC: Animal 목록
        SVC->>PS: fetch_page(1000, 1000)
        PS->>PAW: GET 둘째 쪽
        PAW--xPS: 15초 동안 응답 없음
        PS-->>SVC: Timeout
        Note over SVC,ST: 받은 1000건은 버린다. replace를 부르지 않는다
        SVC-->>CMD: Timeout
        CMD-->>API: code timeout
        API-->>APP: ApiError
        APP-->>U: UI-1의 8 — 보던 목록과 받은 시각은 그대로
    end
    APP->>APP: 새로고침 버튼을 되살린다
```

**읽을 때 볼 것**
- 실패하면 보관소에 손대지 않는다. 그래서 화면은 `query_animals`를 다시 부를 필요가 없다([[PAW-API-001]] 3장)
- 중간 실패도 전체 실패다 — 1000건을 받아 둔 채 목록을 반쯤 바꾸지 않는다([[PAW-UC-001#UC-H1]] 2b)
- 처음 켤 때의 실패도 같은 흐름이고, 화면만 UI-1의 7로 다르다

## SEQ-4 원래 공고 페이지 열기

[[PAW-UC-001#UC-H4]] 기본 흐름·확장, [[PAW-UC-001#UC-S4]].

```mermaid
sequenceDiagram
    autonumber
    actor U as 입양 희망자
    participant APP as App
    participant API as api/animals.ts
    participant CMD as commands
    participant SVC as AnimalService
    participant ST as AnimalStore
    participant OP as TauriOpener
    participant BR as 기본 브라우저
    U->>APP: 포인핸드에서 보기를 누른다
    APP->>API: openLink(공고번호, pawinhand)
    API->>CMD: invoke open_link
    CMD->>SVC: open_link(공고번호, Pawinhand)
    SVC->>ST: current()
    ST-->>SVC: Snapshot
    SVC->>SVC: find · Link.for_animal (인코딩 · 허용 확인)
    SVC->>OP: open(url)
    alt 열렸다
        OP->>BR: 기본 브라우저로 연다
        OP-->>SVC: Ok
        SVC-->>CMD: Link
        CMD-->>API: url
        API-->>APP: 할 일 없음
    else 열지 못했다
        OP-->>SVC: OpenError
        SVC-->>CMD: OpenFailed(url)
        CMD-->>API: code open-failed · url
        API-->>APP: ApiError
        APP-->>U: UI-2의 6 — 주소 칸 · 주소 복사
    end
```

**읽을 때 볼 것**
- 주소는 서비스가 만든다. 화면은 공고번호와 종류만 넘긴다 — 화면에는 링크 열기 권한이 없다([[PAW-INFRA-001]] 5장)
- 목록 카드에서 눌러 실패했을 때도 같은 흐름이고, 화면은 그 아이의 UI-2를 열어 알린다([[PAW-UI-001#UI-1]] 규칙)
- 원문 번호가 없는 아이는 원문 링크 자체가 보이지 않아 `no-source`까지 오지 않는다

## SEQ-5 새 버전 배포

[[PAW-UC-001#UC-A1]] 기본 흐름·확장.

```mermaid
sequenceDiagram
    autonumber
    actor D as 배포자
    participant GIT as GitHub 저장소
    participant GA as GitHub Actions
    participant REL as GitHub Releases
    actor U as 입양 희망자
    D->>GIT: 설정 버전을 올리고 태그 v0.2.0을 push
    GIT->>GA: release.yml 시작 (windows-latest)
    GA->>GA: 태그와 설정의 버전을 대조
    alt 다르다
        GA-->>D: 실패 — 두 값을 보여준다
    else 같다
        GA->>GA: npm ci · Rust 준비 · tauri build (TAURI_APP_PATH=backend)
        GA->>GA: setup.exe가 20MB 이하인지 확인
        GA->>REL: Release v0.2.0 · PawinhandBigCat_0.2.0_x64-setup.exe
        D->>U: Release 링크를 나눠 준다
        U->>REL: setup.exe를 받아 설치 (UC-H6)
    end
```

**읽을 때 볼 것**
- 빌드·크기 확인 중 하나라도 실패하면 Release가 생기지 않는다([[PAW-UC-001#UC-A1]] 2a)
- 버전은 설정 한 곳에서만 오고, 태그는 그 값과 같아야 한다([[PAW-INFRA-001]] 8.4)

## 1. 대응표

| 시퀀스 | 유스케이스 | 커맨드 | 화면 |
|---|---|---|---|
| SEQ-1 | UC-H1 · UC-S1 · UC-S3 | `load_animals` → `query_animals` | UI-1 |
| SEQ-2 | UC-H2 · UC-S3 | `query_animals` | UI-1 |
| SEQ-3 | UC-H5 · UC-S1 | `load_animals` | UI-1 |
| SEQ-4 | UC-H4 · UC-S4 | `open_link` | UI-1 · UI-2 |
| SEQ-5 | UC-A1 · UC-H6 | — | — |

화면이 없는 유스케이스 UC-S2(몸무게 해석)는 SEQ-1의 「칸 옮기기 · 몸무게 해석」 안에 있고, UC-H3(상세 보기)은 코어를 부르지 않아(결과에 이미 모든 정보가 있다) 시퀀스가 없다.

## 2. 되먹일 것

그려 보니 기존 명세와 어긋난 곳은 없다. 확인한 것만 적는다.

- 진행 상태가 서비스 → 커맨드 → 채널로 가는 길이 [[PAW-DOM-002]] 3장 「service는 Tauri를 모른다」와 맞는다
- 새로고침 실패 뒤 화면이 `query_animals`를 부르지 않아도 되는 이유(보관소가 그대로다)가 [[PAW-API-001]] 3장 표와 맞는다
- 상세 보기(UC-H3)는 커맨드를 부르지 않는다 — [[PAW-API-001]] 2.4 「상세를 따로 묻는 커맨드는 없다」와 맞는다

## 3. 미결사항

- [x] 없음
