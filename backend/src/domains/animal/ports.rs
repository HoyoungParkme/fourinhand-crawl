//! 밖과 닿는 두 곳의 트레이트. 앱에서는 어댑터가, 테스트에서는 가짜가 구현한다

use std::future::Future;

use super::models::{Animal, FetchError};

/// 받아온 한 쪽
#[derive(Debug, Default)]
pub struct Page {
    /// 포인핸드가 준 원래 건수 — 더 받을지는 이 수로 정한다
    pub raw_count: usize,
    /// 개념으로 옮긴 동물. 옮기지 못한 건은 빠져 있다
    pub animals: Vec<Animal>,
}

/// 보호 동물을 한 쪽씩 주는 곳 — 앱에서는 포인핸드
pub trait AnimalSource {
    fn fetch_page(&self, offset: usize, limit: usize) -> impl Future<Output = Result<Page, FetchError>> + Send;
}

/// 기본 브라우저를 열지 못했다
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenError;

/// 주소를 기본 브라우저로 여는 곳
pub trait LinkOpener {
    fn open(&self, url: &str) -> Result<(), OpenError>;
}
