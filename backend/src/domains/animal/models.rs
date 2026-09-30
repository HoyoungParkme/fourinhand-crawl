//! 보호 동물 도메인의 개념과 자기 값을 만드는 규칙(DOM-002 2장)
//!
//! 순수하다 — 네트워크·시계·전역 상태를 쓰지 않는다. 오늘 날짜도 인자로 받는다

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{DateTime, FixedOffset, Months, NaiveDate};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

use crate::core::config::{MAX_KG_AS_KG, MAX_PAGES, PAGE_OVERLAP, PAGE_SIZE};

/// 보호 동물 한 마리. 공고번호가 식별자다
#[derive(Debug, Clone, PartialEq)]
pub struct Animal {
    pub notice_no: String,
    pub registered_on: NaiveDate,
    pub weight: Weight,
    pub notice: NoticePeriod,
    pub region: Region,
    pub photos: Vec<Photo>,
    /// 국가동물보호정보시스템 원문 번호. 없으면 원문 링크가 없다
    pub source_no: Option<String>,
    pub breed: Option<String>,
    pub color: Option<String>,
    pub age: Option<String>,
    pub sex: Option<Sex>,
    pub neutered: Option<Neutered>,
    pub feature: Option<String>,
    pub found_at: Option<String>,
    pub shelter_name: Option<String>,
    pub shelter_address: Option<String>,
    pub shelter_tel: Option<String>,
    pub office_name: Option<String>,
    pub office_tel: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sex {
    M,
    F,
    Q,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Neutered {
    Y,
    N,
    U,
}

/// 몸무게 — 받은 글자와 해석한 kg
#[derive(Debug, Clone, PartialEq)]
pub struct Weight {
    /// 받은 글자 그대로. 바꾸지 않는다
    pub raw: String,
    /// 숫자로 바뀌지 않으면 없다
    pub kg: Option<f64>,
}

impl Weight {
    /// PRD R2의 여섯 단계를 순서대로 밟는다. 규칙은 이 함수 한 곳에만 있다
    pub fn parse(raw: &str) -> Weight {
        Weight { raw: raw.to_string(), kg: parse_kg(raw) }
    }

    pub fn at_least(&self, min_kg: f64) -> bool {
        self.kg.is_some_and(|kg| kg >= min_kg)
    }
}

fn parse_kg(raw: &str) -> Option<f64> {
    // 1. (Kg)와 모든 공백을 지운다
    let text: String = raw.replace("(Kg)", "").chars().filter(|c| !c.is_whitespace()).collect();
    // 2. 쉼표는 점으로, 이어진 점은 하나로
    let mut s = String::with_capacity(text.len() + 1);
    for c in text.chars().map(|c| if c == ',' { '.' } else { c }) {
        if c == '.' && s.ends_with('.') {
            continue;
        }
        s.push(c);
    }
    // 3. 점으로 시작하면 앞에 0
    if s.starts_with('.') {
        s.insert(0, '0');
    }
    // 4. 0 뒤에 숫자만 있으면(017) 0 뒤에 점 — 0.17kg
    if s.len() > 1 && s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit()) {
        s.insert(1, '.');
    }
    // 5. 수로 읽는다. NaN·무한대는 없는 값
    let value = s.parse::<f64>().ok().filter(|v| v.is_finite())?;
    // 6. 30보다 크면 그램으로 본다
    Some(if value > MAX_KG_AS_KG { value / 1000.0 } else { value })
}

/// 공고 기간
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoticePeriod {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeStatus {
    /// 공고중 — 종료일 당일까지
    Notice,
    /// 보호중 — 종료일 다음 날부터
    Protected,
}

impl NoticePeriod {
    /// 저장하지 않고 부를 때마다 오늘로 계산한다
    pub fn status(&self, today: NaiveDate) -> NoticeStatus {
        if today > self.end {
            NoticeStatus::Protected
        } else {
            NoticeStatus::Notice
        }
    }
}

/// 지역. 시·도가 없으면 「지역 미상」
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub sido: Option<String>,
    pub sigungu: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionFilter {
    All,
    Sido(String),
    Unknown,
}

impl Region {
    /// 이름은 글자 그대로 비교한다(`강원특별자치도`와 `강원도`는 다르다)
    pub fn matches(&self, filter: &RegionFilter) -> bool {
        match filter {
            RegionFilter::All => true,
            RegionFilter::Sido(name) => self.sido.as_deref() == Some(name.as_str()),
            RegionFilter::Unknown => self.sido.is_none(),
        }
    }
}

/// 포인핸드가 상대 경로로 주는 사진의 앞 주소(INFRA C10)
const PHOTO_CDN: &str = "https://d12l2mexpetzlh.cloudfront.net/images/shelter/";
/// 화면이 사진을 불러올 수 있는 두 곳 — CSP와 같다
const PHOTO_HOSTS: [&str; 2] = ["https://www.animal.go.kr/", "https://d12l2mexpetzlh.cloudfront.net/"];

/// 화면이 불러올 https 사진 주소
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Photo {
    pub url: String,
}

impl Photo {
    pub fn from_raw(raw: &str) -> Option<Photo> {
        let raw = raw.trim();
        if raw.is_empty() {
            return None;
        }
        let url = if let Some(rest) = raw.strip_prefix("http://") {
            format!("https://{rest}")
        } else if raw.starts_with("https://") {
            raw.to_string()
        } else if raw.contains("://") {
            return None;
        } else {
            format!("{PHOTO_CDN}{}", raw.trim_start_matches('/'))
        };
        PHOTO_HOSTS.iter().any(|host| url.starts_with(host)).then_some(Photo { url })
    }
}

/// 포인핸드 상세 페이지의 앞 주소
const PAWINHAND_DETAIL: &str = "https://pawinhand.kr/shelter/animal/detail/";
/// 국가동물보호정보시스템 공고의 앞 주소 — 받은 옛 주소는 첫 화면으로 가서 새로 만든다(INFRA C9)
const SOURCE_NOTICE: &str = "https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo=";
/// 코어가 여는 주소는 이 두 곳뿐이다(INFRA 5장)
const LINK_PREFIXES: [&str; 2] = ["https://pawinhand.kr/", "https://www.animal.go.kr/"];
/// RFC 3986의 예약되지 않은 글자(A-Z a-z 0-9 - _ . ~) 말고는 모두 인코딩한다 — 괄호도
const NOT_UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC.remove(b'-').remove(b'_').remove(b'.').remove(b'~');

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    /// 포인핸드 상세
    Pawinhand,
    /// 국가동물보호정보시스템 원문 공고
    Source,
}

/// 기본 브라우저로 열 공고 주소
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub kind: LinkKind,
    pub url: String,
}

impl Link {
    pub fn for_animal(animal: &Animal, kind: LinkKind) -> Result<Link, LinkError> {
        let url = match kind {
            LinkKind::Pawinhand => {
                format!("{PAWINHAND_DETAIL}{}", utf8_percent_encode(&animal.notice_no, NOT_UNRESERVED))
            }
            LinkKind::Source => format!("{SOURCE_NOTICE}{}", animal.source_no.as_deref().ok_or(LinkError::NoSource)?),
        };
        if !LINK_PREFIXES.iter().any(|prefix| url.starts_with(prefix)) {
            return Err(LinkError::NotAllowed);
        }
        Ok(Link { kind, url })
    }
}

/// 받아온 목록 한 벌. 다 받은 뒤에만 만들고, 바꿀 때는 통째로 간다
#[derive(Debug)]
pub struct Snapshot {
    /// 공고번호로 유일하다. 조회 결과가 복사하지 않고 가리키게 `Arc`로 든다
    pub animals: Vec<Arc<Animal>>,
    pub fetched_at: DateTime<FixedOffset>,
}

impl Snapshot {
    /// 같은 공고번호는 처음 것만 남긴다. 순서는 받은 순서
    pub fn new(animals: Vec<Animal>, fetched_at: DateTime<FixedOffset>) -> Snapshot {
        let mut seen = HashSet::new();
        let animals = animals
            .into_iter()
            .filter(|animal| seen.insert(animal.notice_no.clone()))
            .map(Arc::new)
            .collect();
        Snapshot { animals, fetched_at }
    }

    /// 처음부터 훑는다 — 5천 건 안팎이라 인덱스를 두지 않는다(DOM-003 3장)
    pub fn find(&self, notice_no: &str) -> Option<&Arc<Animal>> {
        self.animals.iter().find(|animal| animal.notice_no == notice_no)
    }
}

/// 받아오기 한 번의 진행 기록. 요청과 기다리기는 서비스가 하고, 쪽을 세는 규칙만 여기 있다
#[derive(Debug, Default)]
pub struct Fetch {
    /// 지금까지 받은 줄 수(겹쳐 받은 줄은 한 번만) — 화면의 진행 상태
    pub received: usize,
    pub pages: usize,
}

impl Fetch {
    /// 한 쪽을 센다. `offset`은 이번 쪽의 시작, `raw_count`는 포인핸드가 준 원래 건수(옮기다 버린 건을 빼기 전)다.
    /// 돌려주는 값은 「더 받을지」 — 옮기다 버린 건 때문에 다음 쪽을 놓치지 않게 원래 건수로 정한다
    pub fn record_page(&mut self, offset: usize, raw_count: usize) -> Result<bool, FetchError> {
        self.pages += 1;
        self.received = offset + raw_count;
        let full = raw_count == PAGE_SIZE;
        if full && self.pages >= MAX_PAGES {
            return Err(FetchError::TooManyPages);
        }
        Ok(full)
    }

    /// 다음 쪽의 시작. 앞 쪽과 `PAGE_OVERLAP`만큼 겹친다 — 받는 사이 앞쪽에서 빠진 아이가 있어
    /// 줄이 당겨져도 뒤 아이를 놓치지 않게. 겹쳐 받은 아이는 스냅숏을 만들 때 한 번만 남는다
    pub fn next_offset(offset: usize) -> usize {
        offset + PAGE_SIZE - PAGE_OVERLAP
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    All,
    OneYear,
    SixMonths,
    ThreeMonths,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    /// 무거운 순
    Weight,
    /// 최근 등록 순
    Registered,
}

/// 조회 조건
#[derive(Debug, Clone, PartialEq)]
pub struct SearchCondition {
    pub min_weight_kg: f64,
    pub period: Period,
    pub region: RegionFilter,
    pub sort: SortKey,
}

impl SearchCondition {
    pub fn validate(&self) -> Result<(), QueryError> {
        if !self.min_weight_kg.is_finite() || self.min_weight_kg < 0.0 {
            return Err(QueryError::InvalidCondition);
        }
        if matches!(&self.region, RegionFilter::Sido(name) if name.trim().is_empty()) {
            return Err(QueryError::InvalidCondition);
        }
        Ok(())
    }

    /// 기간의 첫날. 그 달에 없는 날은 말일로 맞춘다(5월 31일 − 3개월 → 2월 28·29일)
    pub fn since(&self, today: NaiveDate) -> Option<NaiveDate> {
        let months = match self.period {
            Period::All => return None,
            Period::OneYear => 12,
            Period::SixMonths => 6,
            Period::ThreeMonths => 3,
        };
        today.checked_sub_months(Months::new(months))
    }
}

/// 조회 결과
#[derive(Debug)]
pub struct SearchResult {
    /// 걸린 동물, 정렬된 순서
    pub animals: Vec<Arc<Animal>>,
    pub count: usize,
    /// 지역 조건만 뺀 나머지 조건에 걸린 수
    pub all_count: usize,
    /// 스냅숏에 있는 시·도마다 하나(0 포함), 지역 미상이 있으면 하나 더
    pub region_counts: Vec<RegionCount>,
    /// 고른 시·도가 스냅숏에 아예 없어 전국으로 걸렀는지
    pub region_reset: bool,
}

/// 시·도 하나의 마릿수. `sido`가 없으면 지역 미상
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionCount {
    pub sido: Option<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchError {
    ConnectionFailed,
    Timeout,
    BadFormat,
    TooManyPages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryError {
    NoSnapshot,
    InvalidCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenLinkError {
    NoSnapshot,
    NotFound,
    NoSource,
    NotAllowed,
    /// 기본 브라우저를 열지 못했다. 화면이 보여줄 수 있게 만든 주소를 담는다
    OpenFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkError {
    NoSource,
    NotAllowed,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn weight_parse_follows_prd_r2() {
        let cases = [
            ("9.00(Kg)", 9.0),
            ("0,39(Kg)", 0.39),
            (" 0,31 (Kg)", 0.31),
            ("0..8(Kg)", 0.8),
            ("0.,53(Kg)", 0.53),
            (".9(Kg)", 0.9),
            ("017(Kg)", 0.17),
            ("0312(Kg)", 0.312),
            ("0(Kg)", 0.0),
            ("800(Kg)", 0.8),
            ("850(Kg)", 0.85),
            ("15(Kg)", 15.0),
        ];
        for (raw, kg) in cases {
            let weight = Weight::parse(raw);
            assert_eq!(weight.kg, Some(kg), "{raw}");
            assert_eq!(weight.raw, raw);
        }
    }

    #[test]
    fn weight_parse_gives_no_kg_for_text_that_is_not_a_number() {
        for raw in ["", "https://ww(Kg)", "0.2~0.3(Kg)", "3-4(Kg)", "0.3.(Kg)", "inf", "NaN(Kg)"] {
            let weight = Weight::parse(raw);
            assert_eq!(weight.kg, None, "{raw}");
            assert_eq!(weight.raw, raw);
        }
    }

    #[test]
    fn weight_at_least_needs_a_kg() {
        assert!(Weight::parse("8(Kg)").at_least(8.0));
        assert!(!Weight::parse("7.99(Kg)").at_least(8.0));
        assert!(!Weight::parse("").at_least(0.0));
    }

    #[test]
    fn notice_is_open_until_the_end_date() {
        let period = NoticePeriod { start: date("2026-09-23"), end: date("2026-10-06") };
        assert_eq!(period.status(date("2026-10-06")), NoticeStatus::Notice);
        assert_eq!(period.status(date("2026-10-07")), NoticeStatus::Protected);
    }

    #[test]
    fn region_matches_by_exact_name() {
        let gangwon = Region { sido: Some("강원특별자치도".into()), sigungu: Some("삼척시".into()) };
        let unknown = Region { sido: None, sigungu: Some("장성군".into()) };
        assert!(gangwon.matches(&RegionFilter::All));
        assert!(gangwon.matches(&RegionFilter::Sido("강원특별자치도".into())));
        assert!(!gangwon.matches(&RegionFilter::Sido("강원도".into())));
        assert!(!gangwon.matches(&RegionFilter::Unknown));
        assert!(unknown.matches(&RegionFilter::Unknown));
        assert!(!unknown.matches(&RegionFilter::Sido("전라남도".into())));
    }

    #[test]
    fn photo_from_raw_uses_https_on_two_hosts() {
        let url = |raw: &str| Photo::from_raw(raw).map(|p| p.url);
        assert_eq!(
            url("http://www.animal.go.kr/files/a.jpg").as_deref(),
            Some("https://www.animal.go.kr/files/a.jpg")
        );
        assert_eq!(
            url(" https://www.animal.go.kr/files/b.jpg ").as_deref(),
            Some("https://www.animal.go.kr/files/b.jpg")
        );
        assert_eq!(
            url("more/more_1.jpg").as_deref(),
            Some("https://d12l2mexpetzlh.cloudfront.net/images/shelter/more/more_1.jpg")
        );
        assert_eq!(
            url("http://d12l2mexpetzlh.cloudfront.net/images/shelter/abandoned/p.JPG").as_deref(),
            Some("https://d12l2mexpetzlh.cloudfront.net/images/shelter/abandoned/p.JPG")
        );
        for raw in ["", "  ", "ftp://x", "https://evil.example/a.jpg", "https://www.animal.go.kr.evil.example/a.jpg"] {
            assert_eq!(url(raw), None, "{raw}");
        }
    }

    fn animal(notice_no: &str) -> Animal {
        Animal {
            notice_no: notice_no.to_string(),
            registered_on: date("2026-09-01"),
            weight: Weight::parse("8(Kg)"),
            notice: NoticePeriod { start: date("2026-09-01"), end: date("2026-09-11") },
            region: Region { sido: None, sigungu: None },
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

    #[test]
    fn snapshot_keeps_the_first_of_each_notice_no() {
        let mut second = animal("A");
        second.color = Some("둘째".into());
        let fetched_at = DateTime::parse_from_rfc3339("2026-09-30T14:02:11+09:00").unwrap();
        let snapshot = Snapshot::new(vec![animal("A"), animal("B"), second], fetched_at);
        let nos: Vec<_> = snapshot.animals.iter().map(|a| a.notice_no.as_str()).collect();
        assert_eq!(nos, ["A", "B"]);
        assert_eq!(snapshot.animals[0].color, None);
    }

    #[test]
    fn snapshot_finds_by_notice_no() {
        let fetched_at = DateTime::parse_from_rfc3339("2026-09-30T14:02:11+09:00").unwrap();
        let snapshot = Snapshot::new(vec![animal("A"), animal("B")], fetched_at);
        assert_eq!(snapshot.find("B").map(|a| a.notice_no.as_str()), Some("B"));
        assert!(snapshot.find("C").is_none());
    }

    #[test]
    fn pawinhand_link_encodes_everything_but_unreserved() {
        let url = |no: &str| Link::for_animal(&animal(no), LinkKind::Pawinhand).unwrap().url;
        assert_eq!(
            url("경기-화성-2026-01287"),
            "https://pawinhand.kr/shelter/animal/detail/%EA%B2%BD%EA%B8%B0-%ED%99%94%EC%84%B1-2026-01287"
        );
        assert_eq!(
            url("광주-남구-2026-00054(구)"),
            "https://pawinhand.kr/shelter/animal/detail/%EA%B4%91%EC%A3%BC-%EB%82%A8%EA%B5%AC-2026-00054%28%EA%B5%AC%29"
        );
        assert!(url("서울-시립2-2026-00080-P").ends_with("-%EC%8B%9C%EB%A6%BD2-2026-00080-P"));
        assert!(url("a b/c?d#e").ends_with("/detail/a%20b%2Fc%3Fd%23e"));
    }

    #[test]
    fn source_link_needs_a_source_no() {
        let mut with_source = animal("경기-화성-2026-01287");
        with_source.source_no = Some("441553202602482".into());
        assert_eq!(
            Link::for_animal(&with_source, LinkKind::Source),
            Ok(Link {
                kind: LinkKind::Source,
                url: "https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo=441553202602482".into(),
            })
        );
        assert_eq!(Link::for_animal(&animal("서울-동대문-2025-00237"), LinkKind::Source), Err(LinkError::NoSource));
    }

    #[test]
    fn fetch_goes_on_while_pages_are_full() {
        let mut fetch = Fetch::default();
        assert_eq!(fetch.record_page(0, 1000), Ok(true));
        assert_eq!(Fetch::next_offset(0), 950);
        assert_eq!(fetch.record_page(950, 1000), Ok(true));
        assert_eq!(fetch.record_page(1900, 225), Ok(false));
        assert_eq!(fetch.received, 2125);
        assert_eq!(Fetch::default().record_page(0, 0), Ok(false));
    }

    #[test]
    fn fetch_stops_at_the_page_limit() {
        let mut fetch = Fetch::default();
        let mut offset = 0;
        for _ in 1..MAX_PAGES {
            assert_eq!(fetch.record_page(offset, PAGE_SIZE), Ok(true));
            offset = Fetch::next_offset(offset);
        }
        assert_eq!(fetch.record_page(offset, PAGE_SIZE), Err(FetchError::TooManyPages));
    }

    fn condition(min_weight_kg: f64, period: Period, region: RegionFilter) -> SearchCondition {
        SearchCondition { min_weight_kg, period, region, sort: SortKey::Weight }
    }

    #[test]
    fn condition_validate_rejects_bad_values() {
        assert_eq!(condition(8.0, Period::All, RegionFilter::All).validate(), Ok(()));
        assert_eq!(condition(0.0, Period::All, RegionFilter::Unknown).validate(), Ok(()));
        for bad in [
            condition(-1.0, Period::All, RegionFilter::All),
            condition(f64::NAN, Period::All, RegionFilter::All),
            condition(f64::INFINITY, Period::All, RegionFilter::All),
            condition(8.0, Period::All, RegionFilter::Sido(String::new())),
        ] {
            assert_eq!(bad.validate(), Err(QueryError::InvalidCondition), "{bad:?}");
        }
    }

    #[test]
    fn condition_since_clamps_to_the_month_end() {
        let since = |period, today| condition(8.0, period, RegionFilter::All).since(date(today));
        assert_eq!(since(Period::All, "2026-09-30"), None);
        assert_eq!(since(Period::ThreeMonths, "2026-09-30"), Some(date("2026-06-30")));
        assert_eq!(since(Period::ThreeMonths, "2026-05-31"), Some(date("2026-02-28")));
        assert_eq!(since(Period::ThreeMonths, "2028-05-31"), Some(date("2028-02-29")));
        assert_eq!(since(Period::SixMonths, "2026-09-30"), Some(date("2026-03-30")));
        assert_eq!(since(Period::OneYear, "2028-02-29"), Some(date("2027-02-28")));
    }
}
