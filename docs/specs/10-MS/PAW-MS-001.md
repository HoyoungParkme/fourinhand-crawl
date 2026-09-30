---
doc_id: PAW-MS-001
type: MS
title: 포인핸드 대형묘 찾기 — 미니스펙
status: draft
upstream: [PAW-DOM-002, PAW-API-001, PAW-SEQ-001, PAW-PRD-001]
---

# MINISPEC — 포인핸드 대형묘 찾기

## 0. 이 문서가 다루는 것

코어(Rust) 함수 하나하나의 입력·처리·출력·예외와 테스트 관점을 정한다. 클래스와 파일 자리는 [[PAW-DOM-002]], 커맨드의 바깥 모양은 [[PAW-API-001]], 부르는 차례는 [[PAW-SEQ-001]]이 정본이다.

- 항목 이름은 `타입.함수`다. `commands.rs`의 커맨드는 구조체가 없어 모듈 이름으로 `commands.load_animals`처럼 쓴다
- 분기는 `if 조건 → 결과 · else → 결과`로 적는다
- 도메인 오류 열거형은 `models.rs`에 있다: `FetchError { Busy, ConnectionFailed, Timeout, BadFormat, TooManyPages }` · `QueryError { NoSnapshot, InvalidCondition }` · `OpenLinkError { NoSnapshot, NotFound, NoSource, NotAllowed, OpenFailed(url) }` · `LinkError { NoSource, NotAllowed }`. 화면에 가는 코드로 바꾸는 곳은 `core/error.rs` 한 곳이다([[PAW-DOM-002#CommandError]])
- 상수는 `core/config.rs`: `PAGE_SIZE = 1000` · `PAGE_GAP = 300ms` · `REQUEST_TIMEOUT = 15s` · `MAX_PAGES = 20` · `START_DATE = 20200101` · `MAX_KG_AS_KG = 30.0`

## 1. 함수 목록

| 함수 | 파일 | 한 줄 |
|---|---|---|
| `Weight.parse` | models.rs | 몸무게 글자 → kg |
| `Weight.at_least` | models.rs | 기준 이상인지 |
| `NoticePeriod.status` | models.rs | 오늘 기준 공고중·보호중 |
| `Region.matches` | models.rs | 지역 조건에 맞는지 |
| `Photo.from_raw` | models.rs | 사진 주소 정리·허용 확인 |
| `Link.for_animal` | models.rs | 공고 링크 주소 만들기 |
| `Snapshot.new` | models.rs | 받은 목록 → 스냅숏(중복 빼기) |
| `Snapshot.find` | models.rs | 공고번호로 한 마리 |
| `Fetch.record_page` | models.rs | 한 쪽 세고 더 받을지 |
| `SearchCondition.validate` | models.rs | 조건 값 검사 |
| `SearchCondition.since` | models.rs | 기간의 첫날 |
| `AnimalStore.current` | store.rs | 지금 스냅숏 |
| `AnimalStore.replace` | store.rs | 스냅숏 통째로 갈기 |
| `AnimalStore.try_begin_load` | store.rs | 「받는 중」 잡기 |
| `AnimalService.load` | service.rs | 전부 받아 스냅숏 갈기 |
| `AnimalService.query` | service.rs | 거르기·세기·정렬 |
| `AnimalService.open_link` | service.rs | 링크 만들고 열기 |
| `PawinhandSource.fetch_page` | adapters/pawinhand.rs | 한 쪽 받기 |
| `PawinhandSource.page_url` | adapters/pawinhand.rs | 요청 주소 만들기 |
| `PawinhandSource.to_animal` | adapters/pawinhand.rs | 응답 한 건 → Animal |
| `PawinhandSource.source_no` | adapters/pawinhand.rs | 원문 번호 꺼내기 |
| `TauriOpener.open` | adapters/opener.rs | 기본 브라우저로 열기 |
| `commands.load_animals` · `commands.query_animals` · `commands.open_link` | commands.rs | 화면 입출력 |

화면(TypeScript) 쪽 함수는 이름이 camelCase라 항목으로 두지 않고 4장 표에 적는다.

## 2. 함수

#### Weight.parse 몸무게 해석

**시그니처** `fn parse(raw: &str) -> Weight`

**입력** 포인핸드 `weight` 칸의 글자(빈 글자 포함)

**처리** — [[PAW-PRD-001#R2]]의 순서 그대로
1. `(Kg)`를 지우고 모든 공백을 지운다
2. `,` → `.`, 이어진 `.`은 하나로
3. `.`으로 시작하면 앞에 `0`을 붙인다
4. `if 0 뒤에 숫자만 있다(0\d+) → 0 뒤에 . 을 넣는다`
5. `f64`로 읽는다. `if 못 읽거나 유한하지 않다(NaN·inf) → kg 없음`
6. `if 값 > 30 → 값 ÷ 1000 · else → 그대로`

**출력** `Weight { raw: 받은 글자 그대로, kg }`

**테스트 관점** `9.00(Kg)`→9.0 · `0,39(Kg)`→0.39 · ` 0,31 (Kg)`→0.31 · `0..8(Kg)`→0.8 · `0.,53(Kg)`→0.53 · `.9(Kg)`→0.9 · `017(Kg)`→0.17 · `0312(Kg)`→0.312 · `0(Kg)`→0 · `800(Kg)`→0.8 · `850(Kg)`→0.85 · `15(Kg)`→15 · 빈 글자·`https://ww(Kg)`·`0.2~0.3(Kg)`·`3-4(Kg)`·`0.3.(Kg)`·`inf`→없음 · `raw`는 늘 받은 글자와 같다. 2026-09-30 데이터에서 못 읽는 29건은 모두 빈 값·범위·오타이고 8kg 이상이 없다

근거: [[PAW-UC-001#UC-S2]] · [[PAW-DOM-002#Weight]]

#### Weight.at_least 기준 이상

**시그니처** `fn at_least(&self, min_kg: f64) -> bool`

**처리** `if kg가 있고 kg >= min_kg → 참 · else → 거짓`

**테스트 관점** 8.0은 8 이상 · 7.99는 아님 · kg 없음은 기준 0에도 거짓

#### NoticePeriod.status 공고 상태

**시그니처** `fn status(&self, today: NaiveDate) -> NoticeStatus`

**처리** `if today > end → Protected(보호중) · else → Notice(공고중)`

**테스트 관점** 종료일 당일은 공고중 · 다음 날은 보호중 · 포인핸드 `isNotifyEnd`(종료일 < 오늘이면 끝)와 같은 결과

근거: [[PAW-DOM-001#NoticePeriod]]

#### Region.matches 지역 비교

**시그니처** `fn matches(&self, filter: &RegionFilter) -> bool`

**처리** `if All → 참 · if Sido(이름) → sido == Some(이름) · if Unknown → sido가 없다`

**테스트 관점** 시·도 없는 아이는 `Unknown`에만 걸린다 · 이름은 글자 그대로 비교한다(`강원특별자치도`와 `강원도`는 다르다)

#### Photo.from_raw 사진 주소 정리

**시그니처** `fn from_raw(raw: &str) -> Option<Photo>`

**처리**
1. 앞뒤 공백을 지운다. `if 비었다 → 없음`
2. `if http:// 로 시작 → https:// 로 바꾼다 · if https:// 로 시작 → 그대로 · if 다른 :// 가 있다 → 없음 · else(상대 경로) → CDN https://d12l2mexpetzlh.cloudfront.net/images/shelter/ + 앞의 / 를 뗀 글자`
3. `if https://www.animal.go.kr/ 나 https://d12l2mexpetzlh.cloudfront.net/ 로 시작 → Photo · else → 없음`

**테스트 관점** `http://www.animal.go.kr/files/a.jpg`→https · `more/more_1.jpg`→CDN 주소 · `ftp://x`·`https://evil.example/a.jpg`·빈 글자→없음

근거: [[PAW-INFRA-001#C10]] · [[PAW-DOM-002#Photo]]

#### Link.for_animal 공고 링크 만들기

**시그니처** `fn for_animal(animal: &Animal, kind: LinkKind) -> Result<Link, LinkError>`

**처리**
1. `if Pawinhand → https://pawinhand.kr/shelter/animal/detail/ + 공고번호를 A-Z a-z 0-9 - _ . ~ 말고 모두 퍼센트 인코딩`
2. `if Source → source_no가 있으면 https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo= + source_no · else → NoSource`
3. `if 만든 주소가 https://pawinhand.kr/ 나 https://www.animal.go.kr/ 로 시작 → Link · else → NotAllowed`

**예외** | 원문 번호 없음 | `NoSource` | · | 허용 밖 주소 | `NotAllowed` |

**테스트 관점** `경기-화성-2026-01287` → `…/detail/%EA%B2%BD%EA%B8%B0-%ED%99%94%EC%84%B1-2026-01287` · `광주-남구-2026-00054(구)`의 괄호가 `%28`·`%29`가 된다 · 원문 번호 없는 아이의 Source는 `NoSource`

근거: [[PAW-UC-001#UC-S4]] · [[PAW-INFRA-001#C9]]

#### Snapshot.new 스냅숏 만들기

**시그니처** `fn new(animals: Vec<Animal>, fetched_at: DateTime<FixedOffset>) -> Snapshot`

**처리** 공고번호를 본 적이 있으면 뺀다(처음 것을 남긴다). 남은 것을 `Arc`로 싼다

**테스트 관점** 같은 공고번호 두 건 → 한 건 · 순서는 받은 순서

#### Snapshot.find 한 마리 찾기

**시그니처** `fn find(&self, notice_no: &str) -> Option<&Arc<Animal>>`

**처리** 처음부터 훑어 공고번호가 같은 것을 돌려준다(5천 건 안팎이라 인덱스를 두지 않는다 — [[PAW-DOM-003]] 3장)

#### Fetch.record_page 한 쪽 세기

**시그니처** `fn record_page(&mut self, raw_count: usize) -> Result<bool, FetchError>`

**입력** 이번 쪽에서 포인핸드가 준 **원래 건수**(옮기다 버린 건을 빼기 전)

**처리**
1. `pages += 1`, `received += raw_count`
2. `if raw_count == PAGE_SIZE 이고 pages >= MAX_PAGES → TooManyPages · if raw_count == PAGE_SIZE → 참(더 받는다) · else → 거짓(끝)`

**테스트 관점** 1000·1000·225 → 참·참·거짓, received 2225 · 1000을 스무 번 → 스무 번째에 `TooManyPages` · 0건 → 거짓

#### SearchCondition.validate 조건 검사

**시그니처** `fn validate(&self) -> Result<(), QueryError>`

**처리** `if min_weight_kg가 유한하지 않거나 0보다 작다 → InvalidCondition · if Sido(이름)인데 이름이 비었다 → InvalidCondition · else → Ok`

#### SearchCondition.since 기간의 첫날

**시그니처** `fn since(&self, today: NaiveDate) -> Option<NaiveDate>`

**처리** `if All → 없음 · else → today.checked_sub_months(12·6·3개월)` — 그 달에 없는 날은 chrono가 말일로 맞춘다

**테스트 관점** 2026-09-30의 3개월 → 2026-06-30 · 2026-05-31의 3개월 → 2026-02-28 · 2028-05-31의 3개월 → 2028-02-29

근거: [[PAW-API-001]] `query_animals`

#### AnimalStore.current 지금 스냅숏

**시그니처** `fn current(&self) -> Option<Arc<Snapshot>>`

**처리** 읽기 잠금을 잡고 `Arc`를 복사해 돌려준다(목록은 복사하지 않는다)

#### AnimalStore.replace 스냅숏 갈기

**시그니처** `fn replace(&self, snapshot: Snapshot) -> Arc<Snapshot>`

**처리** 쓰기 잠금을 잡고 `Arc`로 싼 새 스냅숏을 통째로 넣는다. 넣은 `Arc`를 돌려준다

**테스트 관점** 갈기 전에 받아 둔 `Arc`는 옛 목록 그대로다 — 읽는 쪽이 반쯤 바뀐 목록을 보지 않는다

#### AnimalStore.try_begin_load 받는 중 잡기

**시그니처** `fn try_begin_load(&self) -> Option<LoadGuard<'_>>`

**처리** `if 「받는 중」을 거짓→참으로 바꾸는 데 성공 → LoadGuard · else → 없음`. `LoadGuard`가 떨어지면 거짓으로 되돌린다

**테스트 관점** 두 번 연달아 → 두 번째는 없음 · 가드를 버린 뒤에는 다시 잡힌다

#### AnimalService.load 전부 받기

**시그니처** `async fn load(&self, on_page: impl Fn(usize)) -> Result<Arc<Snapshot>, FetchError>`

**처리**
1. `if store.try_begin_load()가 없다 → Busy`
2. `offset = 0`, 빈 목록, `Fetch::default()`
3. 되풀이: `source.fetch_page(offset, PAGE_SIZE)` → 옮긴 동물을 목록에 더한다 → `fetch.record_page(page.raw_count)` → `on_page(fetch.received)` → `if 더 받는다 → offset += PAGE_SIZE, PAGE_GAP만큼 쉬고 다시 · else → 멈춘다`
4. `Snapshot::new(목록, 지금 시각)` → `store.replace` → 그 `Arc`를 돌려준다

**예외** | 받는 중 | `Busy` | · | 한 쪽이라도 실패 | 그 오류 그대로(받은 목록은 버리고 `replace`를 부르지 않는다) |

**호출하는 것** [[#AnimalStore.try_begin_load]] · `AnimalSource.fetch_page`(앱에서는 [[#PawinhandSource.fetch_page]]) · [[#Fetch.record_page]] · [[#Snapshot.new]] · [[#AnimalStore.replace]]

**테스트 관점**(가짜 원천으로) 1000·225 두 쪽 → 스냅숏 1225건, `on_page`가 1000·1225로 두 번 · 둘째 쪽 `Timeout` → 오류, 옛 스냅숏 그대로 · 받는 동안 다시 부르면 `Busy` · 끝난 뒤 다시 부를 수 있다 · **옮기다 버린 건이 있어도 원래 건수가 1000이면 다음 쪽을 받는다**

근거: [[PAW-SEQ-001#SEQ-1]] · [[PAW-SEQ-001#SEQ-3]] · [[PAW-UC-001#UC-S1]]

#### AnimalService.query 거르기

**시그니처** `fn query(&self, condition: &SearchCondition, today: NaiveDate) -> Result<SearchResult, QueryError>`

**처리**
1. `condition.validate()` → `if 스냅숏이 없다 → NoSnapshot`
2. `since = condition.since(today)`
3. 기본 걸림 = `weight.at_least(min)` 이고 `if since가 있으면 registered_on >= since`
4. 시·도 목록 = 스냅숏 전체에 있는 시·도들. 시·도별 마릿수 = 기본 걸림 가운데 그 시·도인 수(0 포함). `if 스냅숏에 시·도 없는 아이가 있다 → Unknown 한 줄 더`. `all_count` = 기본 걸림 수
5. `if 조건이 Sido(이름)인데 시·도 목록에 없다 → 지역을 All로 바꾸고 region_reset = 참`
6. 기본 걸림을 `region.matches`로 거른다
7. 정렬: `if Weight → kg 내림 → 등록일 내림 → 공고번호 오름 · if Registered → 등록일 내림 → kg 내림 → 공고번호 오름`

**출력** `SearchResult { animals, count, all_count, region_counts, region_reset }`

**호출하는 것** [[#SearchCondition.validate]] · [[#AnimalStore.current]] · [[#SearchCondition.since]] · [[#Weight.at_least]] · [[#Region.matches]]

**테스트 관점** 2026-09-30 표본 15마리: 기준 8·전체 → 15, 무거운 순 첫째 15.0kg · 경기도 → 4 · 경기도+3개월 → 2 · 전국+3개월 → 6 · 기준 7 → 15보다 많다 · `Sido("없는도")` → 전국으로 바꿔 15, `region_reset` 참 · 경상남도(목록에 있으나 0마리) → 0, 초기화 없음 · 스냅숏 없음 → `NoSnapshot` · 기준 -1 → `InvalidCondition`

근거: [[PAW-SEQ-001#SEQ-2]] · [[PAW-UC-001#UC-S3]]

#### AnimalService.open_link 링크 열기

**시그니처** `fn open_link(&self, notice_no: &str, kind: LinkKind) -> Result<Link, OpenLinkError>`

**처리**
1. `if 스냅숏이 없다 → NoSnapshot`
2. `if find(공고번호)가 없다 → NotFound`
3. `Link::for_animal` — `if NoSource → NoSource · if NotAllowed → NotAllowed`
4. `opener.open(url)` — `if 실패 → OpenFailed(url)`

**호출하는 것** [[#AnimalStore.current]] · [[#Snapshot.find]] · [[#Link.for_animal]] · `LinkOpener.open`(앱에서는 [[#TauriOpener.open]])

**테스트 관점**(가짜 열개로) 성공하면 연 주소가 적힌다 · 열개가 실패하면 `OpenFailed`에 주소가 담긴다 · 없는 공고번호 → `NotFound`

근거: [[PAW-SEQ-001#SEQ-4]]

#### PawinhandSource.fetch_page 한 쪽 받기

**시그니처** `async fn fetch_page(&self, offset: usize, limit: usize) -> Result<Page, FetchError>` — `Page { raw_count, animals }`

**처리**
1. `page_url(offset, limit, 오늘)`로 GET. 시간 제한 15초, User-Agent `PawinhandBigCat/{버전} (+저장소 주소)`
2. `if 시간 초과 → Timeout · if 연결·요청 실패 → ConnectionFailed · if 상태 코드가 2xx가 아니다 → BadFormat`
3. 본문을 JSON으로 읽는다. `if JSON 배열이 아니다 → BadFormat`
4. 배열의 객체마다 `to_animal`. 객체가 아니거나 옮기지 못한 건은 버린다
5. `Page { raw_count: 배열 길이, animals }`

**호출하는 것** [[#PawinhandSource.page_url]] · [[#PawinhandSource.to_animal]]

**테스트 관점** 실데이터 점검(`#[ignore]`): 실제로 한 쪽 받아 1000건, 필수 칸이 다 있다

근거: [[PAW-API-001]] `load_animals` · [[PAW-INFRA-001#C7]]

#### PawinhandSource.page_url 요청 주소

**시그니처** `fn page_url(offset: usize, limit: usize, today: NaiveDate) -> String`

**처리** `https://pawinhand.net/bridge/animals/condition?` + 폼 인코딩(공백 → `+`) 쿼리: `city=모든 지역` · `country=전체` · `species=고양이` · `breeds=전체` · `state=보호중` · `sex=전체` · `neutral=전체` · `start_date=20200101` · `end_date=오늘(YYYYMMDD)` · `offset` · `limit`

**테스트 관점** 결과에 `city=%EB%AA%A8%EB%93%A0+%EC%A7%80%EC%97%AD`가 있고 `%20`이 없다([[PAW-INFRA-001#C8]])

#### PawinhandSource.to_animal 한 건 옮기기

**시그니처** `fn to_animal(item: &serde_json::Map<String, Value>) -> Option<Animal>`

**처리**
1. 칸 읽기: 문자열이면 그대로, 수면 글자로. 앞뒤 공백을 지우고 `if 비었다 → 없음`
2. `if notify_number가 없다 → 버린다`
3. `registration_date`·`notify_sdt`·`notify_edt`를 8자리 날짜로. `if 하나라도 못 읽는다 → 버린다`
4. `Weight::parse(weight 칸의 원래 글자)` — 공백을 지우지 않은 글자
5. 사진: `more_image1` → `image` → `image2` → `image3` 순서로 `Photo::from_raw`, 같은 주소는 뺀다
6. `source_no = source_no(detail_url)`
7. `sex`는 M·F·Q, `neutral`은 Y·N·U일 때만. 나머지 칸은 [[PAW-DOM-003]]의 `animal` 표대로

**호출하는 것** [[#Weight.parse]] · [[#Photo.from_raw]] · [[#PawinhandSource.source_no]]

**테스트 관점** 2026-09-30 실제 응답 한 건(`경기-화성-2026-01287`)을 JSON 그대로 넣어 모든 칸 확인 · 공고번호 없는 건 → 없음 · 날짜 `2026-13-01` → 없음 · `sex: "X"` → 성별 없음 · 주소 끝 공백이 지워진다

#### PawinhandSource.source_no 원문 번호

**시그니처** `fn source_no(detail_url: Option<&str>) -> Option<String>`

**처리** `if 주소에 desertion_no= 가 있다 → 그 뒤 & 전까지 · else → 없음`. `if 꺼낸 값이 숫자만이 아니다 → 없음`

**테스트 관점** 옛 주소 → `441553202602482` · `https://pawinhand.kr/animal/detail/…` → 없음 · `pasm.kr` → 없음 · `desertion_no=12a` → 없음

근거: [[PAW-INFRA-001#C9]]

#### TauriOpener.open 브라우저 열기

**시그니처** `fn open(&self, url: &str) -> Result<(), OpenError>`

**처리** `app.opener().open_url(url, None)` — `if 실패 → OpenError`

#### commands.load_animals 받아오기 커맨드

**시그니처** `async fn load_animals(state: State<AppService>, on_progress: Channel<FetchProgressDto>) -> Result<LoadResultDto, CommandError>`

**처리** `service.load(|n| on_progress.send({ received: n }))` → `if 성공 → { fetchedAt: RFC 3339, total: 스냅숏 건수 } · else → CommandError`

**호출하는 것** [[#AnimalService.load]]

#### commands.query_animals 조회 커맨드

**시그니처** `fn query_animals(state: State<AppService>, condition: ConditionDto) -> Result<QueryResultDto, CommandError>`

**처리** DTO → `SearchCondition` · 오늘 = `Local::now().date_naive()` · `service.query` → 동물마다 `AnimalDto::from_model(animal, 오늘)`(여기서 `notice.status`를 계산)

**호출하는 것** [[#AnimalService.query]] · [[#NoticePeriod.status]]

#### commands.open_link 링크 커맨드

**시그니처** `fn open_link(state: State<AppService>, notice_no: String, kind: LinkKindDto) -> Result<OpenLinkResultDto, CommandError>`

**처리** `service.open_link` → `if 성공 → { url } · else → CommandError`(`OpenFailed`면 `url` 함께)

**호출하는 것** [[#AnimalService.open_link]]

## 3. 테스트 묶음

| 무엇 | 어디 | 언제 |
|---|---|---|
| 순수 규칙(몸무게·사진·링크·기간·쪽 세기·보관소) | 각 파일의 `#[cfg(test)]` | `cargo test` 때마다 |
| 서비스(가짜 원천·가짜 열개) | `service.rs` | `cargo test` 때마다 |
| 옮기기(실제 응답 한 건 JSON) | `adapters/pawinhand.rs` | `cargo test` 때마다 |
| 조회 표본(2026-09-30의 8kg 이상 15마리와 그 주변) | `service.rs` | `cargo test` 때마다 |
| 실데이터 점검 | `adapters/pawinhand.rs`의 `#[ignore]` | 배포 전 `cargo test -- --ignored`([[PAW-INFRA-001]] 7장) |

## 4. 화면 쪽 함수

| 함수 | 파일 | 처리 |
|---|---|---|
| `loadAnimals(onProgress)` | api/animals.ts | `Channel`을 만들어 `onmessage`로 `received`를 넘기고 `invoke('load_animals', { onProgress })` |
| `queryAnimals(condition)` | api/animals.ts | `invoke('query_animals', { condition })` |
| `openLink(noticeNo, kind)` | api/animals.ts | `invoke('open_link', { noticeNo, kind })` |
| `refresh()` | App.tsx | `if 처음 → UI-1의 6 · else → 1.3을 「받는 중…」으로`. 성공하면 지금 조건으로 조회, `regionReset`이면 지역을 전국으로 돌리고 초록 띠. 실패하면 `if 처음 → 7 · else → 8` |
| `changeCondition(next)` | App.tsx | 요청 번호를 올리고 조회. 돌아온 결과의 번호가 마지막 번호일 때만 그린다 |
| `formatKg(kg)` | text/format.ts | 소수 한 자리(`15` → `15.0`) |
| `formatAge(raw)` | text/format.ts | `if 2017(년생) → 2017년생 · if 2026(60일미만)(년생) → 2026년생 (60일 미만) · else → 받은 글자` |
| `formatSex(code)` · `formatNeutered(code)` | text/format.ts | `M 수컷 · F 암컷 · Q 성별 미상` · `Y 했음 · N 안 했음 · U 미확인` |
| `formatPeriod(start, end, short)` | text/format.ts | `if short이고 같은 해 → 2026.08.30 ~ 08.30 · else → 2026.08.30 ~ 2026.08.30` |
| `formatFetchedAt(iso, now)` | text/format.ts | `if 같은 날 → 오늘 14:02 · else → 9월 29일 14:02` |
| `withCopula(word)` | text/format.ts | `if 끝 글자가 받침 있는 한글 → 이에요 · else → 예요` |
| `failureText(code)` | text/format.ts | [[PAW-UI-001#UI-1]] 규칙의 실패 표 그대로 제목·이유 |

## 5. 미결사항

- [x] 없음. 쓰면서 찾은 것은 반영했다: `fetch_page`가 원래 건수(`raw_count`)를 함께 돌려주고, 더 받을지는 그 수로 정한다 — 옮기다 버린 건 때문에 1000보다 적어져 다음 쪽을 놓치지 않게
