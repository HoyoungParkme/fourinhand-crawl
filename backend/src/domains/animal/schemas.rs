//! 화면과 주고받는 모양. 필드는 camelCase, 빈 값은 null(API 1장)

use chrono::{NaiveDate, SecondsFormat};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::models::{
    Animal, LinkKind, Neutered, NoticeStatus, Period, QueryError, RegionCount, RegionFilter, SearchCondition,
    SearchResult, Sex, Snapshot, SortKey,
};

/// 받는 중 진행 상태 — 쪽마다 한 번
#[derive(Debug, Clone, Serialize)]
pub struct FetchProgressDto {
    pub received: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadResultDto {
    pub fetched_at: String,
    pub total: usize,
}

impl LoadResultDto {
    pub fn from_snapshot(snapshot: &Snapshot) -> Self {
        Self {
            fetched_at: snapshot.fetched_at.to_rfc3339_opts(SecondsFormat::Secs, false),
            total: snapshot.animals.len(),
        }
    }
}

/// 조회 조건. 칸이 빠졌거나 모양이 틀려도 serde가 먼저 거절하지 않게 값으로 받고,
/// `into_condition`이 `invalid-condition`으로 거절한다 — serde가 거절하면 오류 모양이 `{ code }`가 아니게 된다
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionDto {
    #[serde(default)]
    pub min_weight_kg: Value,
    #[serde(default)]
    pub period: Value,
    #[serde(default)]
    pub region: Value,
    #[serde(default)]
    pub sort: Value,
}

impl ConditionDto {
    pub fn into_condition(self) -> Result<SearchCondition, QueryError> {
        let invalid = QueryError::InvalidCondition;
        let min_weight_kg = self.min_weight_kg.as_f64().ok_or(invalid)?;
        let period = match self.period.as_str() {
            Some("all") => Period::All,
            Some("1y") => Period::OneYear,
            Some("6m") => Period::SixMonths,
            Some("3m") => Period::ThreeMonths,
            _ => return Err(invalid),
        };
        let kind = self.region.get("kind").and_then(Value::as_str);
        let name = self.region.get("name").and_then(Value::as_str);
        let region = match (kind, name) {
            (Some("all"), _) => RegionFilter::All,
            (Some("sido"), Some(name)) => RegionFilter::Sido(name.to_string()),
            (Some("unknown"), _) => RegionFilter::Unknown,
            _ => return Err(invalid),
        };
        let sort = match self.sort.as_str() {
            Some("weight") => SortKey::Weight,
            Some("registered") => SortKey::Registered,
            _ => return Err(invalid),
        };
        Ok(SearchCondition { min_weight_kg, period, region, sort })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResultDto {
    pub count: usize,
    pub all_count: usize,
    pub region_counts: Vec<RegionCountDto>,
    pub region_reset: bool,
    pub animals: Vec<AnimalDto>,
}

impl QueryResultDto {
    pub fn from_result(result: &SearchResult, today: NaiveDate) -> Self {
        Self {
            count: result.count,
            all_count: result.all_count,
            region_counts: result.region_counts.iter().map(RegionCountDto::from_model).collect(),
            region_reset: result.region_reset,
            animals: result.animals.iter().map(|animal| AnimalDto::from_model(animal, today)).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RegionCountDto {
    Sido { name: String, count: usize },
    Unknown { count: usize },
}

impl RegionCountDto {
    fn from_model(region_count: &RegionCount) -> Self {
        match &region_count.sido {
            Some(name) => Self::Sido { name: name.clone(), count: region_count.count },
            None => Self::Unknown { count: region_count.count },
        }
    }
}

/// 동물 한 마리(API 2.4). 상세 패널도 이 모양을 그대로 쓴다
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimalDto {
    pub notice_no: String,
    pub registered_on: NaiveDate,
    pub weight: WeightDto,
    pub notice: NoticeDto,
    pub region: RegionDto,
    pub photos: Vec<String>,
    pub has_source_notice: bool,
    pub breed: Option<String>,
    pub color: Option<String>,
    pub age: Option<String>,
    pub sex: Option<&'static str>,
    pub neutered: Option<&'static str>,
    pub feature: Option<String>,
    pub found_at: Option<String>,
    pub shelter: ShelterDto,
    pub office: OfficeDto,
}

#[derive(Debug, Serialize)]
pub struct WeightDto {
    /// 결과에 나오는 동물은 기준 이상이라 kg가 늘 있다
    pub kg: f64,
    pub raw: String,
}

#[derive(Debug, Serialize)]
pub struct NoticeDto {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RegionDto {
    pub sido: Option<String>,
    pub sigungu: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ShelterDto {
    pub name: Option<String>,
    pub address: Option<String>,
    pub tel: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OfficeDto {
    pub name: Option<String>,
    pub tel: Option<String>,
}

impl AnimalDto {
    /// 공고 상태는 부를 때의 오늘로 정한다
    pub fn from_model(animal: &Animal, today: NaiveDate) -> Self {
        Self {
            notice_no: animal.notice_no.clone(),
            registered_on: animal.registered_on,
            weight: WeightDto { kg: animal.weight.kg.unwrap_or_default(), raw: animal.weight.raw.clone() },
            notice: NoticeDto {
                start: animal.notice.start,
                end: animal.notice.end,
                status: match animal.notice.status(today) {
                    NoticeStatus::Notice => "notice",
                    NoticeStatus::Protected => "protected",
                },
            },
            region: RegionDto { sido: animal.region.sido.clone(), sigungu: animal.region.sigungu.clone() },
            photos: animal.photos.iter().map(|photo| photo.url.clone()).collect(),
            has_source_notice: animal.source_no.is_some(),
            breed: animal.breed.clone(),
            color: animal.color.clone(),
            age: animal.age.clone(),
            sex: animal.sex.map(|sex| match sex {
                Sex::M => "M",
                Sex::F => "F",
                Sex::Q => "Q",
            }),
            neutered: animal.neutered.map(|neutered| match neutered {
                Neutered::Y => "Y",
                Neutered::N => "N",
                Neutered::U => "U",
            }),
            feature: animal.feature.clone(),
            found_at: animal.found_at.clone(),
            shelter: ShelterDto {
                name: animal.shelter_name.clone(),
                address: animal.shelter_address.clone(),
                tel: animal.shelter_tel.clone(),
            },
            office: OfficeDto { name: animal.office_name.clone(), tel: animal.office_tel.clone() },
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKindDto {
    Pawinhand,
    Source,
}

impl From<LinkKindDto> for LinkKind {
    fn from(kind: LinkKindDto) -> Self {
        match kind {
            LinkKindDto::Pawinhand => LinkKind::Pawinhand,
            LinkKindDto::Source => LinkKind::Source,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OpenLinkResultDto {
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::animal::adapters::pawinhand::PawinhandSource;
    use serde_json::json;

    fn condition(value: Value) -> Result<SearchCondition, QueryError> {
        serde_json::from_value::<ConditionDto>(value).unwrap().into_condition()
    }

    #[test]
    fn condition_dto_reads_the_api_shape() {
        let parsed = condition(json!({
            "minWeightKg": 7.5, "period": "3m", "region": { "kind": "sido", "name": "경기도" }, "sort": "registered"
        }))
        .unwrap();
        assert_eq!(
            parsed,
            SearchCondition {
                min_weight_kg: 7.5,
                period: Period::ThreeMonths,
                region: RegionFilter::Sido("경기도".into()),
                sort: SortKey::Registered,
            }
        );
        let unknown = condition(json!({ "minWeightKg": 8, "period": "all", "region": { "kind": "unknown" }, "sort": "weight" }));
        assert_eq!(unknown.unwrap().region, RegionFilter::Unknown);
    }

    #[test]
    fn condition_dto_rejects_values_outside_the_lists() {
        let base = json!({ "minWeightKg": 8, "period": "all", "region": { "kind": "all" }, "sort": "weight" });
        let with = |key: &str, value: Value| {
            let mut v = base.clone();
            v[key] = value;
            condition(v)
        };
        assert!(with("period", json!("2y")).is_err());
        assert!(with("sort", json!("name")).is_err());
        assert!(with("region", json!({ "kind": "sido" })).is_err());
        assert!(with("region", json!({ "kind": "city", "name": "경기도" })).is_err());
        assert!(with("minWeightKg", Value::Null).is_err());
        assert!(with("minWeightKg", json!("8")).is_err(), "글자로 온 수");
        assert!(with("region", json!("all")).is_err(), "객체가 아닌 지역");
        assert!(with("minWeightKg", json!(8)).is_ok());
        let missing: ConditionDto = serde_json::from_value(json!({ "minWeightKg": 8 })).unwrap();
        assert!(missing.into_condition().is_err(), "빠진 칸");
    }

    #[test]
    fn animal_dto_matches_the_api_example() {
        let item = serde_json::from_str(
            r#"{"image": "https://www.animal.go.kr/files/shelter/2026/07/202608311508874.jpg", "image2": "https://www.animal.go.kr/files/shelter/2026/07/202608311508400.jpg", "image3": null, "notify_number": "경기-화성-2026-01287", "notify_sdt": "20260830", "notify_edt": "20260830", "sex": "M", "color": "레몬색&흰색", "age": "2017(년생)", "neutral": "U", "feature": "덩치가 매우 몹시큰 고양이/ 거대냥이/ 순하고 겁이 조금 있음/ 지금은 숨고싶어함 ", "registration_date": "20260831", "find_location": "팔탄면 삼천병마로 인근", "shelter_name": "화성시민동물보호센터", "shelter_address": "경기도 화성시 서신면 전곡항로 492  ", "shelter_tel": "010-3969-1034", "office_name": "경기도 화성시", "office_tel": null, "city": "경기도", "country": "화성시", "s_breeds": "한국고양이", "detail_url": "http://www.animal.go.kr/portal_rnl/abandonment/public_view.jsp?desertion_no=441553202602482", "weight": "15(Kg)", "more_image1": null}"#,
        )
        .unwrap();
        let animal = PawinhandSource::to_animal(&item).unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 9, 30).unwrap();
        let dto = serde_json::to_value(AnimalDto::from_model(&animal, today)).unwrap();
        assert_eq!(
            dto,
            json!({
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
            })
        );
        let on_the_end_date = serde_json::to_value(AnimalDto::from_model(&animal, animal.notice.end)).unwrap();
        assert_eq!(on_the_end_date["notice"]["status"], "notice");
    }

    #[test]
    fn region_counts_are_tagged_by_kind() {
        let counts = [RegionCount { sido: Some("경기도".into()), count: 4 }, RegionCount { sido: None, count: 1 }];
        let dto: Vec<_> = counts.iter().map(RegionCountDto::from_model).collect();
        assert_eq!(
            serde_json::to_value(dto).unwrap(),
            json!([{ "kind": "sido", "name": "경기도", "count": 4 }, { "kind": "unknown", "count": 1 }])
        );
    }
}
