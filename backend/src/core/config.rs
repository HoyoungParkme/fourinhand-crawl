//! 코어 상수. 포인핸드 요청 조건과 받아오기 한도를 한 곳에 둔다

use std::time::Duration;

/// 포인핸드 보호 동물 조건 검색 주소. 공개·문서화된 API가 아니다(INFRA C6)
pub const API_URL: &str = "https://pawinhand.net/bridge/animals/condition";

/// 요청 조건 — 모든 지역의 보호중 고양이. 폼 인코딩으로 공백은 `+`가 된다(INFRA C8)
pub const API_CONDITION: [(&str, &str); 7] = [
    ("city", "모든 지역"),
    ("country", "전체"),
    ("species", "고양이"),
    ("breeds", "전체"),
    ("state", "보호중"),
    ("sex", "전체"),
    ("neutral", "전체"),
];

/// 조회 시작일(`start_date`). 끝 날은 늘 오늘이다
pub const START_DATE: &str = "20200101";

/// 한 쪽에 받는 건수
pub const PAGE_SIZE: usize = 1000;

/// 다음 쪽을 앞 쪽과 이만큼 겹쳐 받는다 — 받는 사이 앞쪽에서 빠진 아이가 있어도 뒤 아이를 놓치지 않게
pub const PAGE_OVERLAP: usize = 50;

/// 앞 요청이 끝나고 다음 쪽을 부를 때까지 쉬는 시간(INFRA C7)
pub const PAGE_GAP: Duration = Duration::from_millis(300);

/// 한 요청의 시간 제한(INFRA C7)
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// 이만큼 받고도 쪽이 꽉 차 있으면 비정상으로 보고 멈춘다(2만 건)
pub const MAX_PAGES: usize = 20;

/// 이보다 큰 몸무게 값은 그램으로 본다(PRD R2)
pub const MAX_KG_AS_KG: f64 = 30.0;

/// 저장소 주소 — User-Agent에 밝힌다
pub const REPO_URL: &str = "https://github.com/HoyoungParkme/fourinhand-crawl";

/// 요청 머리의 User-Agent. 앱 이름·버전·저장소 주소를 밝힌다(INFRA C7)
pub fn user_agent(version: &str) -> String {
    format!("PawinhandBigCat/{version} (+{REPO_URL})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_agent_names_app_version_and_repo() {
        assert_eq!(
            user_agent("0.1.0"),
            "PawinhandBigCat/0.1.0 (+https://github.com/HoyoungParkme/fourinhand-crawl)"
        );
    }
}
