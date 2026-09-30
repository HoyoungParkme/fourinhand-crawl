//! 화면에 가는 오류 모양과 코드. 코드 목록은 이 파일 한 곳에만 있다(API 1장)

use serde::Serialize;

use crate::domains::animal::models::{FetchError, OpenLinkError, QueryError};

/// 커맨드가 실패하면 promise가 이 모양으로 거절된다. `message`는 개발용이라 화면에 그대로 보이지 않는다
#[derive(Debug, Serialize, PartialEq)]
pub struct CommandError {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl CommandError {
    fn new(code: &'static str, message: &str) -> Self {
        Self { code, message: message.to_string(), url: None }
    }
}

impl From<FetchError> for CommandError {
    fn from(error: FetchError) -> Self {
        match error {
            FetchError::ConnectionFailed => Self::new("connection-failed", "could not reach pawinhand"),
            FetchError::Timeout => Self::new("timeout", "a request took longer than 15 seconds"),
            FetchError::BadFormat => Self::new("bad-format", "error status or the body is not a JSON array"),
            FetchError::TooManyPages => Self::new("too-many-pages", "still full after 20 pages"),
        }
    }
}

impl From<QueryError> for CommandError {
    fn from(error: QueryError) -> Self {
        match error {
            QueryError::NoSnapshot => Self::new("no-snapshot", "nothing has been loaded yet"),
            QueryError::InvalidCondition => Self::new("invalid-condition", "condition value is out of range"),
        }
    }
}

impl From<OpenLinkError> for CommandError {
    fn from(error: OpenLinkError) -> Self {
        match error {
            OpenLinkError::NoSnapshot => Self::new("no-snapshot", "nothing has been loaded yet"),
            OpenLinkError::NotFound => Self::new("not-found", "no animal with that notice number"),
            OpenLinkError::NoSource => Self::new("no-source", "no source notice number"),
            OpenLinkError::NotAllowed => Self::new("not-allowed", "the address is not on the allowed list"),
            OpenLinkError::OpenFailed(url) => {
                Self { url: Some(url), ..Self::new("open-failed", "default browser did not start") }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn each_domain_error_has_one_code() {
        let codes: Vec<_> = [
            FetchError::ConnectionFailed,
            FetchError::Timeout,
            FetchError::BadFormat,
            FetchError::TooManyPages,
        ]
        .into_iter()
        .map(|e| CommandError::from(e).code)
        .chain([QueryError::NoSnapshot, QueryError::InvalidCondition].into_iter().map(|e| CommandError::from(e).code))
        .collect();
        assert_eq!(
            codes,
            ["connection-failed", "timeout", "bad-format", "too-many-pages", "no-snapshot", "invalid-condition"]
        );
    }

    #[test]
    fn error_without_url_leaves_it_out() {
        assert_eq!(
            serde_json::to_value(CommandError::from(FetchError::Timeout)).unwrap(),
            json!({ "code": "timeout", "message": "a request took longer than 15 seconds" })
        );
    }

    #[test]
    fn open_failed_carries_the_url() {
        let codes: Vec<_> = [OpenLinkError::NoSnapshot, OpenLinkError::NotFound, OpenLinkError::NoSource, OpenLinkError::NotAllowed]
            .into_iter()
            .map(|e| CommandError::from(e).code)
            .collect();
        assert_eq!(codes, ["no-snapshot", "not-found", "no-source", "not-allowed"]);
        let url = "https://pawinhand.kr/shelter/animal/detail/%EA%B2%BD%EA%B8%B0-%ED%99%94%EC%84%B1-2026-01287";
        assert_eq!(
            serde_json::to_value(CommandError::from(OpenLinkError::OpenFailed(url.into()))).unwrap(),
            json!({ "code": "open-failed", "message": "default browser did not start", "url": url })
        );
    }
}
