//! 받아오기·조회·링크 열기의 흐름과 거르기 규칙. Tauri를 모른다 — 진행 상태는 클로저로 받는다

use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{Local, NaiveDate};

use crate::core::config::{PAGE_GAP, PAGE_SIZE};

use super::models::{
    Animal, Fetch, FetchError, Link, LinkError, LinkKind, OpenLinkError, QueryError, RegionCount, RegionFilter,
    SearchCondition, SearchResult, Snapshot, SortKey,
};
use super::ports::{AnimalSource, LinkOpener};
use super::store::{AnimalStore, LoadTurn};

pub struct AnimalService<S, O> {
    source: S,
    opener: O,
    store: AnimalStore,
}

impl<S: AnimalSource, O: LinkOpener> AnimalService<S, O> {
    pub fn new(source: S, opener: O) -> Self {
        Self { source, opener, store: AnimalStore::default() }
    }

    /// 전부 받아 스냅숏을 간다. 쪽마다 지금까지 받은 건수로 `on_page`를 부른다.
    /// 이미 누가 받는 중이면 새로 받지 않고 그 받기가 끝나기를 기다려 같은 결과를 돌려준다
    pub async fn load(&self, on_page: impl Fn(usize)) -> Result<Arc<Snapshot>, FetchError> {
        let guard = match self.store.try_begin_load() {
            LoadTurn::Mine(guard) => guard,
            LoadTurn::Wait(after) => return self.store.wait_for_load(after).await,
        };
        let result = self.fetch_all(on_page).await;
        guard.finish(&result);
        result
    }

    /// 쪽을 겹쳐 가며 끝까지 받는다. 한 쪽이라도 실패하거나 한 마리도 옮기지 못하면
    /// 받은 것을 버리고 옛 스냅숏을 그대로 둔다
    async fn fetch_all(&self, on_page: impl Fn(usize)) -> Result<Arc<Snapshot>, FetchError> {
        let mut fetch = Fetch::default();
        let mut animals = Vec::new();
        let mut offset = 0;
        loop {
            let page = self.source.fetch_page(offset, PAGE_SIZE).await?;
            animals.extend(page.animals);
            let more = fetch.record_page(offset, page.raw_count)?;
            on_page(fetch.received);
            if !more {
                break;
            }
            offset = Fetch::next_offset(offset);
            tokio::time::sleep(PAGE_GAP).await;
        }
        // 빈 목록은 받기 실패다 — 요청 모양이 어긋나면 포인핸드는 빈 배열을 준다(INFRA C8)
        if animals.is_empty() {
            return Err(FetchError::BadFormat);
        }
        Ok(self.store.replace(Snapshot::new(animals, Local::now().fixed_offset())))
    }

    /// 들고 있는 목록을 거르고 세고 정렬한다. 포인핸드에 묻지 않는다
    pub fn query(&self, condition: &SearchCondition, today: NaiveDate) -> Result<SearchResult, QueryError> {
        condition.validate()?;
        let snapshot = self.store.current().ok_or(QueryError::NoSnapshot)?;
        let since = condition.since(today);
        // 지역 조건만 뺀 나머지 조건에 걸린 것 — 지역별 마릿수와 「전국」 숫자의 기준
        let matched: Vec<&Arc<Animal>> = snapshot
            .animals
            .iter()
            .filter(|animal| animal.weight.at_least(condition.min_weight_kg))
            .filter(|animal| since.is_none_or(|since| animal.registered_on >= since))
            .collect();
        let region_counts = count_regions(&snapshot.animals, &matched);
        // 고른 시·도나 지역 미상이 목록에 아예 없으면(새로고침 뒤 사라짐) 전국으로 거른다
        let (region, region_reset) = match &condition.region {
            RegionFilter::Sido(name) if !region_counts.iter().any(|rc| rc.sido.as_ref() == Some(name)) => {
                (RegionFilter::All, true)
            }
            RegionFilter::Unknown if !region_counts.iter().any(|rc| rc.sido.is_none()) => (RegionFilter::All, true),
            region => (region.clone(), false),
        };
        let mut animals: Vec<Arc<Animal>> =
            matched.iter().filter(|animal| animal.region.matches(&region)).map(|animal| Arc::clone(animal)).collect();
        sort(&mut animals, condition.sort);
        Ok(SearchResult { count: animals.len(), all_count: matched.len(), region_counts, region_reset, animals })
    }

    /// 주소는 코어가 만든다 — 화면은 공고번호와 종류만 넘긴다. 여는 데 실패하면 만든 주소를 담아 돌려준다
    pub fn open_link(&self, notice_no: &str, kind: LinkKind) -> Result<Link, OpenLinkError> {
        let snapshot = self.store.current().ok_or(OpenLinkError::NoSnapshot)?;
        let animal = snapshot.find(notice_no).ok_or(OpenLinkError::NotFound)?;
        let link = Link::for_animal(animal, kind).map_err(|error| match error {
            LinkError::NoSource => OpenLinkError::NoSource,
            LinkError::NotAllowed => OpenLinkError::NotAllowed,
        })?;
        self.opener.open(&link.url).map_err(|_| OpenLinkError::OpenFailed(link.url.clone()))?;
        Ok(link)
    }
}

/// 시·도 목록은 스냅숏 전체에서, 마릿수는 걸린 것에서 센다(0 포함). 시·도는 가나다순, 지역 미상은 끝
fn count_regions(all: &[Arc<Animal>], matched: &[&Arc<Animal>]) -> Vec<RegionCount> {
    let mut sidos: BTreeMap<&str, usize> = BTreeMap::new();
    let mut has_unknown = false;
    for animal in all {
        match animal.region.sido.as_deref() {
            Some(sido) => {
                sidos.entry(sido).or_insert(0);
            }
            None => has_unknown = true,
        }
    }
    let mut unknown = 0;
    for animal in matched {
        match animal.region.sido.as_deref() {
            Some(sido) => *sidos.entry(sido).or_insert(0) += 1,
            None => unknown += 1,
        }
    }
    let mut counts: Vec<RegionCount> =
        sidos.into_iter().map(|(sido, count)| RegionCount { sido: Some(sido.to_string()), count }).collect();
    if has_unknown {
        counts.push(RegionCount { sido: None, count: unknown });
    }
    counts
}

/// 무거운 순: kg → 등록일 → 공고번호. 최근 등록 순: 등록일 → kg → 공고번호
fn sort(animals: &mut [Arc<Animal>], key: SortKey) {
    let kg = |animal: &Animal| animal.weight.kg.unwrap_or(f64::NEG_INFINITY);
    animals.sort_by(|a, b| {
        let heavier = || kg(b).total_cmp(&kg(a));
        let newer = || b.registered_on.cmp(&a.registered_on);
        let first = match key {
            SortKey::Weight => heavier().then_with(newer),
            SortKey::Registered => newer().then_with(heavier),
        };
        first.then_with(|| a.notice_no.cmp(&b.notice_no))
    });
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::future::Future;
    use std::sync::Mutex;
    use std::time::Duration;

    use chrono::DateTime;
    use serde_json::{Map, Value};

    use super::*;
    use crate::domains::animal::adapters::pawinhand::PawinhandSource;
    use crate::domains::animal::models::{NoticePeriod, Period, Region, Weight};
    use crate::domains::animal::ports::{OpenError, Page};

    /// 미리 정한 쪽을 차례로 준다. 받는 중을 흉내 내려고 쪽마다 조금 기다린다
    #[derive(Default)]
    struct FakeSource {
        pages: Mutex<VecDeque<Result<Page, FetchError>>>,
        offsets: Mutex<Vec<usize>>,
    }

    impl FakeSource {
        fn new(pages: Vec<Result<Page, FetchError>>) -> Self {
            Self { pages: Mutex::new(pages.into()), offsets: Mutex::default() }
        }

        fn offsets(&self) -> Vec<usize> {
            self.offsets.lock().unwrap().clone()
        }
    }

    impl AnimalSource for FakeSource {
        fn fetch_page(&self, offset: usize, _limit: usize) -> impl Future<Output = Result<Page, FetchError>> + Send {
            self.offsets.lock().unwrap().push(offset);
            let page = self.pages.lock().unwrap().pop_front().expect("준비한 쪽이 더 없다");
            async move {
                tokio::time::sleep(Duration::from_millis(10)).await;
                page
            }
        }
    }

    /// 연 주소를 적어 두거나 일부러 실패한다
    #[derive(Default)]
    struct FakeOpener {
        opened: Mutex<Vec<String>>,
        fail: bool,
    }

    impl LinkOpener for FakeOpener {
        fn open(&self, url: &str) -> Result<(), OpenError> {
            if self.fail {
                return Err(OpenError);
            }
            self.opened.lock().unwrap().push(url.to_string());
            Ok(())
        }
    }

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn animal(notice_no: &str) -> Animal {
        Animal {
            notice_no: notice_no.to_string(),
            registered_on: date("2026-09-01"),
            weight: Weight::parse("8(Kg)"),
            notice: NoticePeriod { start: date("2026-09-01"), end: date("2026-09-11") },
            region: Region { sido: Some("경기도".into()), sigungu: None },
            photos: vec![],
            source_no: None,
            breed: None,
            color: None,
            age: None,
            sex: None,
            neutered: None,
            feature: None,
            found_at: None,
            shelter_name: None,
            shelter_address: None,
            shelter_tel: None,
            office_name: None,
            office_tel: None,
        }
    }

    /// 원래 건수 `raw_count` 가운데 `kept`건만 옮긴 쪽
    fn page(prefix: &str, raw_count: usize, kept: usize) -> Result<Page, FetchError> {
        Ok(Page { raw_count, animals: (0..kept).map(|i| animal(&format!("{prefix}-{i}"))).collect() })
    }

    fn service(pages: Vec<Result<Page, FetchError>>) -> AnimalService<FakeSource, FakeOpener> {
        AnimalService::new(FakeSource::new(pages), FakeOpener::default())
    }

    #[tokio::test(start_paused = true)]
    async fn load_takes_overlapping_pages_until_one_is_not_full() {
        let service = service(vec![page("a", 1000, 1000), page("b", 225, 225)]);
        let progress = RefCell::new(vec![]);
        let snapshot = service.load(|received| progress.borrow_mut().push(received)).await.unwrap();
        assert_eq!(snapshot.animals.len(), 1225);
        assert_eq!(progress.into_inner(), [1000, 1175]);
        assert_eq!(service.source.offsets(), [0, 950]);
        assert!(Arc::ptr_eq(&snapshot, &service.store.current().unwrap()));
    }

    #[tokio::test(start_paused = true)]
    async fn load_keeps_each_overlapped_animal_once() {
        let first: Vec<Animal> = (0..1000).map(|i| animal(&format!("a-{i}"))).collect();
        let second: Vec<Animal> =
            (950..1000).map(|i| animal(&format!("a-{i}"))).chain((0..10).map(|i| animal(&format!("b-{i}")))).collect();
        let service =
            service(vec![Ok(Page { raw_count: 1000, animals: first }), Ok(Page { raw_count: 60, animals: second })]);
        assert_eq!(service.load(|_| {}).await.unwrap().animals.len(), 1010);
    }

    #[tokio::test(start_paused = true)]
    async fn load_decides_on_the_raw_count_not_the_kept_count() {
        let service = service(vec![page("a", 1000, 998), page("b", 5, 5)]);
        let snapshot = service.load(|_| {}).await.unwrap();
        assert_eq!(service.source.offsets(), [0, 950]);
        assert_eq!(snapshot.animals.len(), 1003);
    }

    #[tokio::test(start_paused = true)]
    async fn load_failure_keeps_the_old_snapshot() {
        let service = service(vec![page("old", 3, 3), page("a", 1000, 1000), Err(FetchError::Timeout)]);
        let old = service.load(|_| {}).await.unwrap();
        assert_eq!(service.load(|_| {}).await.unwrap_err(), FetchError::Timeout);
        assert!(Arc::ptr_eq(&old, &service.store.current().unwrap()));
    }

    #[tokio::test(start_paused = true)]
    async fn load_that_finds_nothing_fails_and_keeps_the_old_snapshot() {
        let service = service(vec![page("old", 3, 3), page("empty", 0, 0)]);
        let old = service.load(|_| {}).await.unwrap();
        assert_eq!(service.load(|_| {}).await.unwrap_err(), FetchError::BadFormat);
        assert!(Arc::ptr_eq(&old, &service.store.current().unwrap()));
    }

    #[tokio::test(start_paused = true)]
    async fn load_called_while_loading_waits_for_the_same_result() {
        let service = service(vec![page("a", 3, 3), page("b", 2, 2)]);
        let (first, second) = tokio::join!(service.load(|_| {}), service.load(|_| {}));
        assert!(Arc::ptr_eq(&first.unwrap(), &second.unwrap()));
        assert_eq!(service.source.offsets(), [0], "두 번째 부르기는 새로 받지 않는다");
        assert_eq!(service.load(|_| {}).await.unwrap().animals.len(), 2, "끝난 뒤에는 새로 받는다");
    }

    #[tokio::test(start_paused = true)]
    async fn load_called_while_loading_gets_the_same_failure() {
        let service = service(vec![Err(FetchError::Timeout)]);
        let (first, second) = tokio::join!(service.load(|_| {}), service.load(|_| {}));
        assert_eq!((first.unwrap_err(), second.unwrap_err()), (FetchError::Timeout, FetchError::Timeout));
    }

    // 조회 표본 — 2026-09-30에 받은 실데이터에서 7kg 이상 전부와, 나머지 시·도마다 한 마리
    const TODAY: &str = "2026-09-30";

    fn sample() -> AnimalService<FakeSource, FakeOpener> {
        let items: Vec<Map<String, Value>> =
            serde_json::from_str(include_str!("fixtures/sample-2026-09-30.json")).unwrap();
        let animals: Vec<Animal> = items.iter().filter_map(PawinhandSource::to_animal).collect();
        assert_eq!(animals.len(), items.len());
        let service = service(vec![]);
        service.store.replace(Snapshot::new(animals, DateTime::parse_from_rfc3339("2026-09-30T14:02:11+09:00").unwrap()));
        service
    }

    fn condition(min_weight_kg: f64, period: Period, region: RegionFilter, sort: SortKey) -> SearchCondition {
        SearchCondition { min_weight_kg, period, region, sort }
    }

    fn nos(result: &SearchResult) -> Vec<&str> {
        result.animals.iter().map(|a| a.notice_no.as_str()).collect()
    }

    fn sido(name: &str) -> RegionFilter {
        RegionFilter::Sido(name.to_string())
    }

    #[test]
    fn query_sample_default_condition_gives_15_heaviest_first() {
        let result =
            sample().query(&condition(8.0, Period::All, RegionFilter::All, SortKey::Weight), date(TODAY)).unwrap();
        assert_eq!(result.count, 15);
        assert_eq!(result.all_count, 15);
        assert!(!result.region_reset);
        assert_eq!(
            nos(&result)[..4],
            ["경기-화성-2026-01287", "서울-종로-2025-00056", "서울-종로-2025-00057", "경기-화성-2026-01286"]
        );
        assert_eq!(result.animals[0].weight.kg, Some(15.0));
        assert!(result.animals.windows(2).all(|w| w[0].weight.kg >= w[1].weight.kg));
    }

    #[test]
    fn query_sample_counts_every_sido_in_the_snapshot() {
        let result =
            sample().query(&condition(8.0, Period::All, RegionFilter::All, SortKey::Weight), date(TODAY)).unwrap();
        let count = |name: Option<&str>| {
            result.region_counts.iter().find(|rc| rc.sido.as_deref() == name).map(|rc| rc.count)
        };
        assert_eq!(result.region_counts.len(), 17, "시·도 16개 + 지역 미상");
        assert_eq!(count(Some("경기도")), Some(4));
        assert_eq!(count(Some("서울특별시")), Some(4));
        assert_eq!(count(Some("경상남도")), Some(0));
        assert_eq!(count(None), Some(1));
        assert_eq!(result.region_counts.last().unwrap().sido, None, "지역 미상은 끝");
        assert_eq!(result.region_counts.iter().map(|rc| rc.count).sum::<usize>(), result.all_count);
    }

    #[test]
    fn query_sample_narrows_by_region_period_and_weight() {
        let service = sample();
        let today = date(TODAY);
        let count = |c: SearchCondition| service.query(&c, today).unwrap().count;
        assert_eq!(count(condition(8.0, Period::All, sido("경기도"), SortKey::Weight)), 4);
        assert_eq!(count(condition(8.0, Period::ThreeMonths, sido("경기도"), SortKey::Weight)), 2);
        assert_eq!(count(condition(8.0, Period::ThreeMonths, RegionFilter::All, SortKey::Weight)), 6);
        assert_eq!(count(condition(8.0, Period::All, RegionFilter::Unknown, SortKey::Weight)), 1);
        assert_eq!(count(condition(7.0, Period::All, RegionFilter::All, SortKey::Weight)), 28);
        let zero = service.query(&condition(8.0, Period::All, sido("경상남도"), SortKey::Weight), today).unwrap();
        assert_eq!((zero.count, zero.all_count, zero.region_reset), (0, 15, false));
    }

    #[test]
    fn query_sample_resets_a_sido_that_is_gone() {
        let result =
            sample().query(&condition(8.0, Period::All, sido("없는도"), SortKey::Weight), date(TODAY)).unwrap();
        assert!(result.region_reset);
        assert_eq!(result.count, 15);
    }

    #[test]
    fn query_resets_unknown_region_when_no_animal_lacks_a_sido() {
        let service = service(vec![]);
        let fetched_at = DateTime::parse_from_rfc3339("2026-09-30T14:02:11+09:00").unwrap();
        service.store.replace(Snapshot::new(vec![animal("A")], fetched_at));
        let result =
            service.query(&condition(8.0, Period::All, RegionFilter::Unknown, SortKey::Weight), date(TODAY)).unwrap();
        assert!(result.region_reset);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn query_sample_sorts_by_registered_date() {
        let result = sample()
            .query(&condition(8.0, Period::All, RegionFilter::All, SortKey::Registered), date(TODAY))
            .unwrap();
        assert_eq!(
            nos(&result)[..4],
            ["강원-삼척-2026-00137", "충남-천안-2026-00453", "경기-화성-2026-01287", "경기-화성-2026-01286"]
        );
        assert!(result.animals.windows(2).all(|w| w[0].registered_on >= w[1].registered_on));
    }

    #[test]
    fn query_needs_a_snapshot_and_a_valid_condition() {
        let valid = condition(8.0, Period::All, RegionFilter::All, SortKey::Weight);
        assert_eq!(service(vec![]).query(&valid, date(TODAY)).unwrap_err(), QueryError::NoSnapshot);
        let negative = condition(-1.0, Period::All, RegionFilter::All, SortKey::Weight);
        assert_eq!(sample().query(&negative, date(TODAY)).unwrap_err(), QueryError::InvalidCondition);
    }

    const HWASEONG: &str = "https://pawinhand.kr/shelter/animal/detail/%EA%B2%BD%EA%B8%B0-%ED%99%94%EC%84%B1-2026-01287";

    #[test]
    fn open_link_opens_the_address_it_made() {
        let service = sample();
        let link = service.open_link("경기-화성-2026-01287", LinkKind::Pawinhand).unwrap();
        assert_eq!(link.url, HWASEONG);
        let source = service.open_link("경기-화성-2026-01287", LinkKind::Source).unwrap();
        assert_eq!(source.url, "https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo=441553202602482");
        assert_eq!(*service.opener.opened.lock().unwrap(), [HWASEONG, source.url.as_str()]);
    }

    #[test]
    fn open_link_failure_carries_the_address() {
        let mut service = sample();
        service.opener.fail = true;
        assert_eq!(
            service.open_link("경기-화성-2026-01287", LinkKind::Pawinhand),
            Err(OpenLinkError::OpenFailed(HWASEONG.to_string()))
        );
    }

    #[test]
    fn open_link_refuses_what_it_cannot_make() {
        let loaded = sample();
        assert_eq!(loaded.open_link("없는-공고-0000", LinkKind::Pawinhand), Err(OpenLinkError::NotFound));
        // 원문 번호가 없는 아이(포인핸드 상세 주소만 받은 아이)
        assert_eq!(loaded.open_link("서울-동대문-2025-00237", LinkKind::Source), Err(OpenLinkError::NoSource));
        assert!(loaded.opener.opened.lock().unwrap().is_empty());
        assert_eq!(
            service(vec![]).open_link("경기-화성-2026-01287", LinkKind::Pawinhand),
            Err(OpenLinkError::NoSnapshot)
        );
    }
}
