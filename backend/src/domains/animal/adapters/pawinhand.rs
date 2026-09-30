//! 포인핸드 API 호출과 응답 칸 → 개념 옮기기. 포인핸드 칸 이름은 이 파일 밖에 나오지 않는다

use chrono::{Local, NaiveDate};
use serde_json::{Map, Value};

use crate::core::config::{API_CONDITION, API_URL, REQUEST_TIMEOUT, START_DATE};
use crate::domains::animal::models::{
    Animal, FetchError, Neutered, NoticePeriod, Photo, Region, Sex, Weight,
};
use crate::domains::animal::ports::{AnimalSource, Page};

/// 사진 칸을 보는 순서(DOM-001 Photo)
const PHOTO_FIELDS: [&str; 4] = ["more_image1", "image", "image2", "image3"];

pub struct PawinhandSource {
    client: reqwest::Client,
}

impl PawinhandSource {
    pub fn new(user_agent: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("HTTP 클라이언트를 만들지 못했다");
        Self { client }
    }

    /// 한 쪽의 요청 주소. 폼 인코딩이라 공백은 `+`다 — `%20`이면 빈 배열이 온다(INFRA C8)
    pub fn page_url(offset: usize, limit: usize, today: NaiveDate) -> String {
        let mut query = form_urlencoded::Serializer::new(String::new());
        query
            .extend_pairs(API_CONDITION)
            .append_pair("start_date", START_DATE)
            .append_pair("end_date", &today.format("%Y%m%d").to_string())
            .append_pair("offset", &offset.to_string())
            .append_pair("limit", &limit.to_string());
        format!("{API_URL}?{}", query.finish())
    }

    /// 응답 한 건 → Animal. 공고번호가 없거나 날짜를 못 읽으면 버린다
    pub fn to_animal(item: &Map<String, Value>) -> Option<Animal> {
        let text = |key| text(item, key);
        let notice_no = text("notify_number")?;
        let registered_on = date(item, "registration_date")?;
        let notice = NoticePeriod { start: date(item, "notify_sdt")?, end: date(item, "notify_edt")? };
        let mut photos: Vec<Photo> = Vec::new();
        for photo in PHOTO_FIELDS.iter().filter_map(|key| item.get(*key)?.as_str()).filter_map(Photo::from_raw) {
            if !photos.contains(&photo) {
                photos.push(photo);
            }
        }
        Some(Animal {
            notice_no,
            registered_on,
            weight: Weight::parse(&raw_text(item, "weight")),
            notice,
            region: Region { sido: text("city"), sigungu: text("country") },
            photos,
            source_no: Self::source_no(item.get("detail_url").and_then(Value::as_str)),
            breed: text("s_breeds"),
            color: text("color"),
            age: text("age"),
            sex: text("sex").and_then(|sex| match sex.as_str() {
                "M" => Some(Sex::M),
                "F" => Some(Sex::F),
                "Q" => Some(Sex::Q),
                _ => None,
            }),
            neutered: text("neutral").and_then(|neutral| match neutral.as_str() {
                "Y" => Some(Neutered::Y),
                "N" => Some(Neutered::N),
                "U" => Some(Neutered::U),
                _ => None,
            }),
            feature: text("feature"),
            found_at: text("find_location"),
            shelter_name: text("shelter_name"),
            shelter_address: text("shelter_address"),
            shelter_tel: text("shelter_tel"),
            office_name: text("office_name"),
            office_tel: text("office_tel"),
        })
    }

    /// 국가동물보호정보시스템 옛 주소의 `desertion_no`. 숫자만일 때만 쓴다(INFRA C9)
    pub fn source_no(detail_url: Option<&str>) -> Option<String> {
        let (_, rest) = detail_url?.split_once("desertion_no=")?;
        let no = rest.split('&').next().unwrap_or_default();
        (!no.is_empty() && no.bytes().all(|b| b.is_ascii_digit())).then(|| no.to_string())
    }
}

impl AnimalSource for PawinhandSource {
    async fn fetch_page(&self, offset: usize, limit: usize) -> Result<Page, FetchError> {
        let url = Self::page_url(offset, limit, Local::now().date_naive());
        let response = self.client.get(url).send().await.map_err(request_error)?;
        if !response.status().is_success() {
            return Err(FetchError::BadFormat);
        }
        let body = response.bytes().await.map_err(request_error)?;
        let items: Vec<Value> = serde_json::from_slice(&body).map_err(|_| FetchError::BadFormat)?;
        page_from(&items)
    }
}

/// 응답 배열 → 한 쪽. 원래 건수가 있는데 한 마리도 옮기지 못하면 칸 이름·날짜 모양이 바뀐 것으로 본다
fn page_from(items: &[Value]) -> Result<Page, FetchError> {
    let animals: Vec<Animal> =
        items.iter().filter_map(Value::as_object).filter_map(PawinhandSource::to_animal).collect();
    if !items.is_empty() && animals.is_empty() {
        return Err(FetchError::BadFormat);
    }
    Ok(Page { raw_count: items.len(), animals })
}

fn request_error(error: reqwest::Error) -> FetchError {
    if error.is_timeout() {
        FetchError::Timeout
    } else {
        FetchError::ConnectionFailed
    }
}

/// 칸의 글자. 수면 글자로 바꾼다. 앞뒤 공백을 지우고, 비면 없다
fn text(item: &Map<String, Value>, key: &str) -> Option<String> {
    let value = match item.get(key)? {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    (!value.is_empty()).then_some(value)
}

/// 칸의 원래 글자 — 공백도 그대로. 몸무게의 원래 값에만 쓴다
fn raw_text(item: &Map<String, Value>, key: &str) -> String {
    match item.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

/// 8자리 날짜(20260830)
fn date(item: &Map<String, Value>, key: &str) -> Option<NaiveDate> {
    let s = text(item, key)?;
    if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    NaiveDate::parse_from_str(&s, "%Y%m%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 2026-09-30에 받은 실제 응답 한 건 그대로
    const REAL_ITEM: &str = r#"{"idx": 0, "image": "https://www.animal.go.kr/files/shelter/2026/07/202608311508874.jpg", "image2": "https://www.animal.go.kr/files/shelter/2026/07/202608311508400.jpg", "image3": null, "notify_number": "경기-화성-2026-01287", "notify_sdt": "20260830", "notify_edt": "20260830", "sex": "M", "breeds": "[고양이] 한국 고양이", "color": "레몬색&흰색", "age": "2017(년생)", "neutral": "U", "feature": "덩치가 매우 몹시큰 고양이/ 거대냥이/ 순하고 겁이 조금 있음/ 지금은 숨고싶어함 ", "state": "보호중", "registration_date": "20260831", "find_location": "팔탄면 삼천병마로 인근", "shelter_name": "화성시민동물보호센터", "shelter_address": "경기도 화성시 서신면 전곡항로 492  ", "shelter_tel": "010-3969-1034", "office_name": "경기도 화성시", "office_tel": null, "city": "경기도", "country": "화성시", "species": "고양이", "s_breeds": "한국고양이", "detail_url": "http://www.animal.go.kr/portal_rnl/abandonment/public_view.jsp?desertion_no=441553202602482", "w_date": "2026-08-31 16:11:10", "weight": "15(Kg)", "bell_count": 32, "click_count": 1704, "comment_count": 5, "share_count": 11, "more_image1": null, "animal_name": null, "recommended": null, "recommended_date": null, "more_info_idx": 0, "shelterAnimalInfo": null}"#;

    fn item(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    fn real_item() -> Map<String, Value> {
        serde_json::from_str(REAL_ITEM).unwrap()
    }

    fn date(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn page_url_encodes_spaces_as_plus() {
        let url = PawinhandSource::page_url(2000, 1000, date("2026-09-30"));
        assert!(url.starts_with("https://pawinhand.net/bridge/animals/condition?city=%EB%AA%A8%EB%93%A0+%EC%A7%80%EC%97%AD&"), "{url}");
        assert!(!url.contains("%20"), "{url}");
        assert!(url.contains("&state=%EB%B3%B4%ED%98%B8%EC%A4%91&"), "{url}");
        assert!(url.ends_with("&start_date=20200101&end_date=20260930&offset=2000&limit=1000"), "{url}");
    }

    #[test]
    fn to_animal_moves_every_field_of_a_real_item() {
        let animal = PawinhandSource::to_animal(&real_item()).unwrap();
        assert_eq!(animal.notice_no, "경기-화성-2026-01287");
        assert_eq!(animal.registered_on, date("2026-08-31"));
        assert_eq!(animal.weight, Weight { raw: "15(Kg)".into(), kg: Some(15.0) });
        assert_eq!(animal.notice, NoticePeriod { start: date("2026-08-30"), end: date("2026-08-30") });
        assert_eq!(animal.region, Region { sido: Some("경기도".into()), sigungu: Some("화성시".into()) });
        let photos: Vec<_> = animal.photos.iter().map(|p| p.url.as_str()).collect();
        assert_eq!(
            photos,
            [
                "https://www.animal.go.kr/files/shelter/2026/07/202608311508874.jpg",
                "https://www.animal.go.kr/files/shelter/2026/07/202608311508400.jpg",
            ]
        );
        assert_eq!(animal.source_no.as_deref(), Some("441553202602482"));
        assert_eq!(animal.breed.as_deref(), Some("한국고양이"));
        assert_eq!(animal.color.as_deref(), Some("레몬색&흰색"));
        assert_eq!(animal.age.as_deref(), Some("2017(년생)"));
        assert_eq!(animal.sex, Some(Sex::M));
        assert_eq!(animal.neutered, Some(Neutered::U));
        assert_eq!(
            animal.feature.as_deref(),
            Some("덩치가 매우 몹시큰 고양이/ 거대냥이/ 순하고 겁이 조금 있음/ 지금은 숨고싶어함")
        );
        assert_eq!(animal.found_at.as_deref(), Some("팔탄면 삼천병마로 인근"));
        assert_eq!(animal.shelter_name.as_deref(), Some("화성시민동물보호센터"));
        assert_eq!(animal.shelter_address.as_deref(), Some("경기도 화성시 서신면 전곡항로 492"));
        assert_eq!(animal.shelter_tel.as_deref(), Some("010-3969-1034"));
        assert_eq!(animal.office_name.as_deref(), Some("경기도 화성시"));
        assert_eq!(animal.office_tel, None);
    }

    #[test]
    fn to_animal_drops_items_without_notice_no_or_dates() {
        let mut no_notice = real_item();
        no_notice.remove("notify_number");
        assert!(PawinhandSource::to_animal(&no_notice).is_none());
        let mut blank_notice = real_item();
        blank_notice.insert("notify_number".into(), json!("  "));
        assert!(PawinhandSource::to_animal(&blank_notice).is_none());
        for bad in ["20261301", "2026-08-30", "20251041", ""] {
            let mut bad_date = real_item();
            bad_date.insert("notify_edt".into(), json!(bad));
            assert!(PawinhandSource::to_animal(&bad_date).is_none(), "{bad}");
        }
    }

    #[test]
    fn to_animal_keeps_only_known_codes_and_empty_values_become_none() {
        let mut odd = real_item();
        odd.insert("sex".into(), json!("X"));
        odd.insert("neutral".into(), json!(""));
        odd.insert("color".into(), json!("   "));
        odd.insert("weight".into(), json!(" 0,31 (Kg)"));
        odd.insert("registration_date".into(), json!(20260831));
        let animal = PawinhandSource::to_animal(&odd).unwrap();
        assert_eq!(animal.sex, None);
        assert_eq!(animal.neutered, None);
        assert_eq!(animal.color, None);
        assert_eq!(animal.weight, Weight { raw: " 0,31 (Kg)".into(), kg: Some(0.31) });
        assert_eq!(animal.registered_on, date("2026-08-31"));
    }

    #[test]
    fn to_animal_orders_photos_and_drops_repeats() {
        let animal = PawinhandSource::to_animal(&item(json!({
            "notify_number": "서울-시립2-2026-00080-P",
            "registration_date": "20260920", "notify_sdt": "20260920", "notify_edt": "20260930",
            "more_image1": "more/more_1789950007421.JPG",
            "image": "http://d12l2mexpetzlh.cloudfront.net/images/shelter/abandoned/pasm_1.JPG",
            "image2": "https://d12l2mexpetzlh.cloudfront.net/images/shelter/abandoned/pasm_1.JPG",
            "image3": "https://evil.example/x.jpg",
            "detail_url": "pasm.kr"
        })))
        .unwrap();
        let photos: Vec<_> = animal.photos.iter().map(|p| p.url.as_str()).collect();
        assert_eq!(
            photos,
            [
                "https://d12l2mexpetzlh.cloudfront.net/images/shelter/more/more_1789950007421.JPG",
                "https://d12l2mexpetzlh.cloudfront.net/images/shelter/abandoned/pasm_1.JPG",
            ]
        );
        assert_eq!(animal.source_no, None);
        assert_eq!(animal.weight.kg, None);
    }

    #[test]
    fn page_that_converts_nothing_is_bad_format() {
        let real = Value::Object(real_item());
        let renamed = json!({ "notice_no": "경기-화성-2026-01287", "reg_date": "20260831" });
        assert_eq!(page_from(&[]).map(|p| p.raw_count), Ok(0));
        assert_eq!(page_from(&[renamed.clone(), renamed.clone()]).unwrap_err(), FetchError::BadFormat);
        let mixed = page_from(&[real, renamed]).unwrap();
        assert_eq!((mixed.raw_count, mixed.animals.len()), (2, 1));
    }

    #[test]
    fn source_no_takes_digits_from_the_old_address_only() {
        let source_no = |url| PawinhandSource::source_no(url);
        assert_eq!(
            source_no(Some("http://www.animal.go.kr/portal_rnl/abandonment/public_view.jsp?desertion_no=441553202602482")).as_deref(),
            Some("441553202602482")
        );
        assert_eq!(source_no(Some("http://x/view.jsp?desertion_no=123&page=2")).as_deref(), Some("123"));
        assert_eq!(source_no(Some("https://pawinhand.kr/animal/detail/서울-동대문-2025-00237")), None);
        assert_eq!(source_no(Some("pasm.kr")), None);
        assert_eq!(source_no(Some("http://x/view.jsp?desertion_no=12a")), None);
        assert_eq!(source_no(Some("http://x/view.jsp?desertion_no=")), None);
        assert_eq!(source_no(None), None);
    }

    /// 실데이터 점검 — 배포 전에 `cargo test -- --ignored`로 돌린다(INFRA 7장)
    #[tokio::test]
    #[ignore = "실제 포인핸드에 요청한다"]
    async fn live_pawinhand_still_answers_in_the_expected_shape() {
        let source = PawinhandSource::new(&crate::core::config::user_agent("live-check"));
        let page = source.fetch_page(0, crate::core::config::PAGE_SIZE).await.expect("첫 쪽을 받지 못했다");
        assert_eq!(page.raw_count, crate::core::config::PAGE_SIZE, "보호중 고양이가 1000마리보다 적다 — 조건이 바뀌었는지 본다");
        assert!(page.animals.len() * 100 >= page.raw_count * 99, "옮기지 못한 건이 1%를 넘는다: {}/{}", page.animals.len(), page.raw_count);
        let with_kg = page.animals.iter().filter(|a| a.weight.kg.is_some()).count();
        assert!(with_kg * 100 >= page.animals.len() * 95, "몸무게를 못 읽은 건이 5%를 넘는다");
        assert!(page.animals.iter().all(|a| !a.photos.is_empty()), "사진 없는 아이가 있다");

        let client = reqwest::Client::builder().user_agent("PawinhandBigCat/live-check").build().unwrap();
        let animal = page.animals.iter().find(|a| a.source_no.is_some()).expect("원문 번호 있는 아이가 없다");
        let no = animal.source_no.as_deref().unwrap();
        let source_page = client
            .get(format!("https://www.animal.go.kr/front/awtis/public/publicDtl.do?desertionNo={no}"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(source_page.contains(no), "원문 공고 페이지에 번호 {no}가 없다 — 주소 형식이 바뀌었는지 본다");

        let photo = client.get(&animal.photos[0].url).send().await.unwrap();
        assert!(photo.status().is_success(), "사진을 열지 못했다: {}", animal.photos[0].url);
        let content_type = photo.headers().get("content-type").map(|v| v.to_str().unwrap_or_default().to_string());
        assert!(content_type.as_deref().is_some_and(|t| t.starts_with("image/")), "사진이 아니다: {content_type:?}");
    }
}
