//! 기본 브라우저로 열기. 여는 것은 코어뿐이다 — 화면에는 opener 권한이 없다(INFRA 5장)

use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::domains::animal::ports::{LinkOpener, OpenError};

pub struct TauriOpener {
    app: AppHandle,
}

impl TauriOpener {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl LinkOpener for TauriOpener {
    fn open(&self, url: &str) -> Result<(), OpenError> {
        self.app.opener().open_url(url, None::<&str>).map_err(|_| OpenError)
    }
}
