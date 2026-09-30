---
doc_id: PAW-API-001
type: API
title: 포인핸드 대형묘 찾기 — 코어 커맨드 명세 (MCP 형식)
status: draft
upstream: [PAW-UC-001, PAW-DOM-001, PAW-INFRA-001, PAW-UI-001]
---

# 코어 커맨드 명세 (MCP 형식) — 포인핸드 대형묘 찾기

## 0. 이 문서가 다루는 것

화면(React)이 코어(Rust)에 부르는 커맨드 셋의 이름, 인자, 결과, 오류, 부르는 차례를 정한다. 커맨드가 셋이라는 것과 각자 맡는 일은 [[PAW-INFRA-001]] 4장이 정했다.

**왜 MCP 형식인가.** 싱크독의 API 문서는 REST와 MCP 두 형식뿐이다. 이 앱에는 HTTP 서버가 없고, 화면이 Tauri의 `invoke(이름, 인자)`로 코어 함수를 부른다. 이름 붙은 함수에 JSON 인자를 주고 JSON 결과를 받는 모양이 MCP 도구와 같아서 이 형식으로 적는다. **이 앱이 MCP 서버인 것은 아니다.** 이 문서의 「도구」는 Tauri 커맨드이고, 「에이전트」는 커맨드를 부르는 화면이다.

| 커맨드 | 한 줄 | 유스케이스 |
|---|---|---|
| `load_animals` | 포인핸드에서 보호중 고양이를 전부 받아 코어의 목록을 바꾼다 | [[PAW-UC-001#UC-S1]] |
| `query_animals` | 들고 있는 목록을 조건으로 거르고 정렬해 돌려준다 | [[PAW-UC-001#UC-S3]] |
| `open_link` | 한 아이의 공고 페이지를 기본 브라우저로 연다 | [[PAW-UC-001#UC-S4]] |

## 1. 규칙

모든 커맨드에 걸리는 약속이다.

| 무엇 | 약속 |
|---|---|
| 부르는 곳 | 화면은 `frontend/src/api/` 한 곳에서만 `invoke`를 부른다(규약 1.9). 화면에 열린 커맨드는 이 셋뿐이다([[PAW-INFRA-001]] 5장) |
| 누구의 권한 | 로그인이 없다. 코어는 설치한 사람의 PC에서 그 사람 권한으로 돈다 |
| 이름 | 커맨드는 snake_case. 인자와 결과의 필드는 camelCase — Tauri가 Rust의 snake_case 인자를 camelCase로 바꿔 받는 기본 규칙을 따른다 |
| 날짜·시각 | 날짜는 `YYYY-MM-DD`, 시각은 ISO 8601에 시간대까지(`2026-09-30T14:02:11+09:00`). 포인핸드의 8자리 날짜는 코어가 바꾼다 |
| 「오늘」 | 기간 거르기와 공고 상태의 「오늘」은 PC의 현지 날짜다. 조회할 때마다 새로 읽는다([[PAW-DOM-001#NoticePeriod]]) |
| 빈 값 | 빈 문자열과 공백뿐인 값은 `null`로 보낸다. 글자 앞뒤 공백은 지운다. 몸무게 원래 값(`weight.raw`)만은 받은 글자 그대로 둔다([[PAW-DOM-001#Weight]]) |
| 상태 | 코어는 받아온 목록 하나([[PAW-DOM-001#Snapshot]])와 받는 중 여부만 들고 있다. `load_animals`가 도는 동안 `query_animals`·`open_link`는 이전 목록으로 답한다 |
| 네트워크 | 포인핸드에 요청하는 커맨드는 `load_animals` 하나다. 나머지는 코어 메모리만 본다 |

**오류 형식.** 커맨드가 실패하면 promise가 아래 모양으로 거절된다. 화면은 `code`로 보여줄 문구를 고른다([[PAW-UI-001#UI-1]] 규칙). `message`는 개발용 설명이라 화면에 그대로 보이지 않는다.

```json
{ "code": "open-failed", "message": "default browser did not start", "url": "https://pawinhand.kr/shelter/animal/detail/%EA%B2%BD%EA%B8%B0-%ED%99%94%EC%84%B1-2026-01287" }
```

| code | 커맨드 | 뜻 | 화면 |
|---|---|---|---|
| `busy` | load_animals | 이미 받는 중이다 | 알리지 않는다 — 버튼이 이미 막혀 있다. 그래도 생기면 1초 뒤 다시 부른다 |
| `connection-failed` | load_animals | 포인핸드에 닿지 못했다 | UI-1의 7·8 |
| `timeout` | load_animals | 한 요청이 15초를 넘었다 | UI-1의 7·8 |
| `bad-format` | load_animals | 오류 응답이거나 JSON 배열이 아니다 | UI-1의 7·8 |
| `too-many-pages` | load_animals | 20쪽(2만 건)을 넘었다 | UI-1의 7·8 |
| `no-snapshot` | query_animals, open_link | 아직 다 받은 목록이 없다 | 순서를 지키면 생기지 않는다 |
| `invalid-condition` | query_animals | 조건 값이 틀렸다 | 화면이 먼저 막는다(UI-1의 11) |
| `not-found` | open_link | 지금 목록에 없는 공고번호다 | 순서를 지키면 생기지 않는다 |
| `no-source` | open_link | 원문 번호가 없는 아이다 | 원문 링크가 안 보이므로 생기지 않는다 |
| `not-allowed` | open_link | 만든 주소가 허용된 곳이 아니다 | UI-2의 「열 수 없는 주소예요」 |
| `open-failed` | open_link | 기본 브라우저를 열지 못했다. `url`이 함께 온다 | UI-2의 6 |

## 2. 도구

#### load_animals 보호중 고양이 받아오기

포인핸드에서 보호중 고양이를 전부 받아, 다 받았을 때만 코어의 목록을 새것으로 바꾼다.

```json
{
  "name": "load_animals",
  "description": "포인핸드에서 보호중 고양이를 전부 받아 코어의 목록을 바꾼다. 쪽을 받을 때마다 누적 마릿수를 onProgress로 보낸다. 실패하면 이전 목록을 그대로 둔다.",
  "inputSchema": {
    "type": "object",
    "required": ["onProgress"],
    "properties": {
      "onProgress": {
        "description": "Tauri 채널. 쪽을 받을 때마다 { \"received\": 누적 마릿수 }를 보낸다",
        "x-tauri": "Channel<FetchProgress>"
      }
    }
  }
}
```

**진행 상태** — 쪽마다 한 번

```json
{ "received": 2000 }
```

**결과**

```json
{ "fetchedAt": "2026-09-30T14:02:11+09:00", "total": 4225 }
```

`total`은 공고번호 중복을 뺀 마릿수다.

| 에러 | 언제 |
|---|---|
| `busy` | 이미 받는 중일 때. 새로 시작하지 않는다 |
| `connection-failed` | 인터넷이 끊겼거나 포인핸드 주소에 닿지 못할 때 |
| `timeout` | 한 요청이 15초를 넘을 때 |
| `bad-format` | 오류 응답이거나, 응답이 JSON 배열이 아닐 때 |
| `too-many-pages` | 20쪽을 넘게 받을 때 |

**코어가 하는 일** ([[PAW-UC-001#UC-S1]])
- 요청: `GET https://pawinhand.net/bridge/animals/condition` — `city=모든 지역`, `country=전체`, `species=고양이`, `breeds=전체`, `state=보호중`, `sex=전체`, `neutral=전체`, `start_date=20200101`, `end_date=오늘`, `offset`, `limit=1000`. 폼 인코딩으로 공백은 `+`([[PAW-INFRA-001#C8]]). User-Agent는 `PawinhandBigCat/{버전} (+https://github.com/HoyoungParkme/fourinhand-crawl)`([[PAW-INFRA-001#C7]])
- 한 번에 한 요청. 앞 요청이 끝나고 0.3초 뒤 다음 쪽. 포인핸드가 준 건수가 1000보다 적으면 끝
- 한 건씩 옮긴다: 몸무게 해석([[PAW-DOM-001#Weight]]), 사진 주소 정리([[PAW-DOM-001#Photo]]), `detail_url`에서 원문 번호 꺼내기([[PAW-DOM-001#Link]]), 빈 값 `null`. 공고번호가 없거나 날짜(등록일·공고 기간)를 못 읽는 건은 버린다
- 다 받으면 공고번호 중복을 빼고 목록을 통째로 바꾼 뒤 결과를 돌려준다. 중간에 실패하면 받은 일부를 버린다([[PAW-UC-001#UC-H1]] 2b)

**연관**: [[PAW-UC-001#UC-H1]] · [[PAW-UC-001#UC-H5]] · [[PAW-DOM-001#Fetch]] · [[PAW-DOM-001#Snapshot]] · [[PAW-UI-001#UI-1]]

#### query_animals 거르고 정렬하기

들고 있는 목록을 조건으로 거르고 정렬해 돌려준다. 포인핸드에 요청하지 않는다.

```json
{
  "name": "query_animals",
  "description": "코어가 들고 있는 목록을 조건으로 거르고 정렬한다. 걸린 동물, 마릿수, 시·도별 마릿수를 돌려준다.",
  "inputSchema": {
    "type": "object",
    "required": ["condition"],
    "properties": {
      "condition": {
        "type": "object",
        "required": ["minWeightKg", "period", "region", "sort"],
        "properties": {
          "minWeightKg": { "type": "number", "minimum": 0, "description": "기준 kg. 이 값 이상이 걸린다. 화면은 0.1 단위로 보낸다" },
          "period": { "enum": ["all", "1y", "6m", "3m"], "description": "등록일 기간" },
          "region": {
            "type": "object",
            "required": ["kind"],
            "properties": {
              "kind": { "enum": ["all", "sido", "unknown"], "description": "전국 · 시·도 하나 · 지역 미상" },
              "name": { "type": "string", "description": "kind가 sido일 때 시·도 이름. 예: 경기도" }
            }
          },
          "sort": { "enum": ["weight", "registered"], "description": "무거운 순 · 최근 등록 순" }
        }
      }
    }
  }
}
```

**결과** — `animals`에는 걸린 동물이 모두 온다. 여기서는 한 마리만 적었다(모양은 2.4).

```json
{
  "count": 15,
  "allCount": 15,
  "regionCounts": [
    { "kind": "sido", "name": "경기도", "count": 4 },
    { "kind": "sido", "name": "서울특별시", "count": 4 },
    { "kind": "sido", "name": "경상남도", "count": 0 },
    { "kind": "unknown", "count": 1 }
  ],
  "regionReset": false,
  "animals": [
    {
      "noticeNo": "경기-화성-2026-01287",
      "registeredOn": "2026-08-31",
      "weight": { "kg": 15.0, "raw": "15(Kg)" },
      "notice": { "start": "2026-08-30", "end": "2026-08-30", "status": "protected" },
      "region": { "sido": "경기도", "sigungu": "화성시" },
      "photos": [
        "https://www.animal.go.kr/files/shelter/2026/07/202608311508874.jpg",
        "https://www.animal.go.kr/files/shelter/2026/07/202608311508400.jpg"
      ],
      "hasSourceNotice": true,
      "breed": "한국고양이",
      "color": "레몬색&흰색",
      "age": "2017(년생)",
      "sex": "M",
      "neutered": "U",
      "feature": "덩치가 매우 몹시큰 고양이/ 거대냥이/ 순하고 겁이 조금 있음/ 지금은 숨고싶어함",
      "foundAt": "팔탄면 삼천병마로 인근",
      "shelter": { "name": "화성시민동물보호센터", "address": "경기도 화성시 서신면 전곡항로 492", "tel": "010-3969-1034" },
      "office": { "name": "경기도 화성시", "tel": null }
    }
  ]
}
```

| 필드 | 뜻 |
|---|---|
| `count` | 조건 전부에 걸린 수. 화면의 「N마리」 |
| `allCount` | 지역 조건만 뺀 나머지 조건에 걸린 수. 지역 목록의 「전국」 숫자 |
| `regionCounts` | 목록에 있는 시·도마다 하나, 지역 조건만 뺀 나머지 조건으로 센 수. 0마리인 시·도도 온다. 시·도가 빈 아이가 목록에 있으면 `unknown` 하나. 순서는 정하지 않는다 — 화면이 [[PAW-UI-001#UI-1]] 규칙대로 늘어놓는다 |
| `regionReset` | 고른 시·도가 목록에 아예 없어 지역을 전국으로 바꿔 걸렀으면 `true`([[PAW-UC-001#UC-H5]] 4a) |
| `animals` | 걸린 동물, 정렬된 순서 |

| 에러 | 언제 |
|---|---|
| `no-snapshot` | 아직 다 받은 목록이 없을 때 |
| `invalid-condition` | `minWeightKg`가 숫자가 아니거나 0보다 작을 때, 값이 목록 밖일 때, `kind`가 `sido`인데 `name`이 없을 때 |

**코어가 하는 일** ([[PAW-UC-001#UC-S3]] · [[PAW-DOM-001#SearchResult]])
- 거르는 순서: 몸무게 없음 빼고 `kg >= minWeightKg` → 기간 → 지역
- 기간: 오늘에서 12·6·3개월을 뺀 날짜 **이상**인 등록일만 남긴다. 뺀 날이 그 달에 없으면 그 달 말일로 한다(5월 31일 − 3개월 → 2월 28일 또는 29일)
- 정렬: `weight`면 kg 내림차순, 같으면 등록일 최신순. `registered`면 등록일 내림차순, 같으면 kg 내림차순. 그래도 같으면 공고번호 순
- `notice.status`는 부를 때의 오늘로 정한다: 오늘이 종료일보다 뒤면 `protected`(보호중), 아니면 `notice`(공고중)

**연관**: [[PAW-UC-001#UC-H1]] · [[PAW-UC-001#UC-H2]] · [[PAW-UC-001#UC-H5]] · [[PAW-DOM-001#SearchCondition]] · [[PAW-DOM-001#SearchResult]] · [[PAW-UI-001#UI-1]] · [[PAW-UI-001#UI-2]]

#### open_link 공고 페이지 열기

한 아이의 포인핸드 상세나 국가동물보호정보시스템 원문 공고를 기본 브라우저로 연다. 주소는 코어가 만든다 — 화면은 공고번호와 종류만 넘긴다.

```json
{
  "name": "open_link",
  "description": "공고번호와 링크 종류를 받아 코어가 주소를 만들고, 허용된 곳인지 확인한 뒤 기본 브라우저로 연다.",
  "inputSchema": {
    "type": "object",
    "required": ["noticeNo", "kind"],
    "properties": {
      "noticeNo": { "type": "string", "description": "공고번호. 예: 경기-화성-2026-01287" },
      "kind": { "enum": ["pawinhand", "source"], "description": "포인핸드 상세 · 국가동물보호정보시스템 원문 공고" }
    }
  }
}
```

**결과**

```json
{ "url": "https://pawinhand.kr/shelter/animal/detail/%EA%B2%BD%EA%B8%B0-%ED%99%94%EC%84%B1-2026-01287" }
```

| 에러 | 언제 |
|---|---|
| `no-snapshot` | 아직 다 받은 목록이 없을 때 |
| `not-found` | 지금 목록에 없는 공고번호일 때 |
| `no-source` | `kind`가 `source`인데 원문 번호가 없는 아이일 때 |
| `not-allowed` | 만든 주소가 `https://pawinhand.kr/`나 `https://www.animal.go.kr/`로 시작하지 않을 때 |
| `open-failed` | 기본 브라우저를 열지 못했을 때. 오류에 `url`이 함께 온다 |

**코어가 하는 일** ([[PAW-UC-001#UC-S4]] · [[PAW-DOM-001#Link]])
- `pawinhand`: `https://pawinhand.kr/shelter/animal/detail/` 뒤에 공고번호 전체를 주소용으로 인코딩해 붙인다(괄호까지)
- `source`: `https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo=` 뒤에 원문 번호를 붙인다
- 허용된 곳인지 확인한 뒤 연다. 화면에는 링크 열기 권한이 없다 — 여는 것은 코어뿐이다([[PAW-INFRA-001]] 5장)

**연관**: [[PAW-UC-001#UC-H4]] · [[PAW-UI-001#UI-1]] · [[PAW-UI-001#UI-2]]

### 2.4 결과에 나오는 동물 한 마리

`query_animals`의 `animals` 한 칸이다. 상세 패널도 이 모양을 그대로 쓴다 — 상세를 따로 묻는 커맨드는 없다.

| 필드 | 형식 | 뜻 · 받는 칸 |
|---|---|---|
| `noticeNo` | 문자열 | 공고번호(`notify_number`). 받은 그대로 |
| `registeredOn` | 날짜 | 등록일(`registration_date`) |
| `weight` | `{ kg: 수, raw: 문자열 }` | 해석값과 원래 값(`weight`). 결과에 나오는 동물은 `kg`가 늘 있다 |
| `notice` | `{ start: 날짜, end: 날짜, status: "notice" 또는 "protected" }` | 공고 기간(`notify_sdt`·`notify_edt`)과 상태 |
| `region` | `{ sido: 문자열 또는 null, sigungu: 문자열 또는 null }` | `city`·`country`. 빈 값은 null(1장 빈 값 규칙) |
| `photos` | 문자열 배열(0~4) | https로 정리한 사진 주소. [[PAW-DOM-001#Photo]] 순서. 화면은 앞 3장을 쓴다 |
| `hasSourceNotice` | 참·거짓 | 원문 번호가 있는지. 번호 자체는 보내지 않는다 |
| `breed` · `color` · `age` · `feature` · `foundAt` | 문자열 또는 null | `s_breeds` · `color` · `age` · `feature` · `find_location`. `age`는 받은 글자 그대로 — 모양 바꾸기는 화면이 한다 |
| `sex` | `"M"`·`"F"`·`"Q"` 또는 null | `sex`. 그 밖의 값은 null |
| `neutered` | `"Y"`·`"N"`·`"U"` 또는 null | `neutral`. 그 밖의 값은 null |
| `shelter` | `{ name, address, tel }` | `shelter_name`·`shelter_address`·`shelter_tel`. 각각 null일 수 있다 |
| `office` | `{ name, tel }` | `office_name`·`office_tel`. 각각 null일 수 있다 |

## 3. 에이전트 순서

이 앱에서 에이전트는 커맨드를 부르는 화면이다. 화면은 아래 차례로 부른다. 어느 커맨드에 대한 결과인지 꼬이지 않게, 같은 커맨드를 다시 불렀으면 **가장 마지막에 부른 호출의 결과만** 그린다.

| 때 | 부르는 것 | 성공하면 | 실패하면 |
|---|---|---|---|
| 창이 뜰 때 | `load_animals` | 1.2에 `fetchedAt`·`total`을 적고 기본 조건(8, `all`, `all`, `weight`)으로 `query_animals` | UI-1의 7. 다시 시도를 누르면 처음부터 |
| 받는 동안 | — | `onProgress`마다 UI-1의 6에 `received`를 적는다 | — |
| 조건을 바꿀 때 | `query_animals` | 목록·마릿수·지역 숫자를 새로 그린다 | `invalid-condition`이면 마지막으로 올바르던 결과를 그대로 둔다 |
| 새로고침·다시 시도 | `load_animals` | 지금 조건으로 `query_animals`. `regionReset`이면 지역 칸을 전국으로 바꾸고 안내 띠를 보여준다 | UI-1의 8. `query_animals`는 부르지 않는다 — 보던 목록 그대로 |
| 링크를 누를 때 | `open_link` | 할 일 없음. 브라우저가 열린다 | `open-failed`면 UI-2의 6(`url`로 주소 칸을 채운다), `not-allowed`면 「열 수 없는 주소예요」 |

- 숫자 칸(UI-1의 2.1)은 입력이 멈추고 150ms 뒤, 올바른 값일 때만 `query_animals`를 부른다. 눈금자(2.2)는 끄는 동안 값이 바뀔 때마다 불러도 된다 — 코어 메모리만 보는 호출이라 가볍다
- `load_animals`가 도는 동안에도 `query_animals`와 `open_link`는 부를 수 있다. 이전 목록으로 답한다
- `busy`·`no-snapshot`·`not-found`·`no-source`는 이 차례를 지키면 생기지 않는다. 생기면 화면은 아무것도 바꾸지 않는다 — `busy`는 1초 뒤 `load_animals`를 다시 불러 받는 중 화면에 멈추지 않게 하고(개발 중 화면만 다시 그려졌을 때), 나머지는 개발 중 콘솔에만 남긴다

## 4. 미결사항

- [x] **UI 문서 후속 수정** — 반영했다: 카드의 「포인핸드에서 보기」가 `open-failed`·`not-allowed`로 실패하면 그 아이의 UI-2를 열어 알린다([[PAW-UI-001#UI-1]] 규칙 · [[PAW-UI-001#UI-2]] 진입)
