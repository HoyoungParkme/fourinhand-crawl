//! 화면에 여는 커맨드 셋(API 2장). 입출력만 한다 — 인자를 모델로 바꾸고, 오늘 날짜를 읽고,
//! 서비스를 부르고, 결과를 화면 모양으로 바꾼다

use chrono::Local;
use tauri::ipc::Channel;
use tauri::State;

use crate::core::error::CommandError;

use super::adapters::opener::TauriOpener;
use super::adapters::pawinhand::PawinhandSource;
use super::schemas::{ConditionDto, FetchProgressDto, LinkKindDto, LoadResultDto, OpenLinkResultDto, QueryResultDto};
use super::service::AnimalService;

/// 앱이 Tauri 상태로 하나만 두는 서비스
pub type AppService = AnimalService<PawinhandSource, TauriOpener>;

#[tauri::command]
pub async fn load_animals(
    service: State<'_, AppService>,
    on_progress: Channel<FetchProgressDto>,
) -> Result<LoadResultDto, CommandError> {
    let snapshot = service
        .load(|received| {
            // 화면이 닫혀 채널이 끊겨도 받기는 끝까지 한다
            let _ = on_progress.send(FetchProgressDto { received });
        })
        .await?;
    Ok(LoadResultDto::from_snapshot(&snapshot))
}

/// 걸린 동물이 많으면 옮기기·직렬화가 무거워서 창이 멈추지 않게 메인 스레드 밖(async)에서 돈다
#[tauri::command]
pub async fn query_animals(
    service: State<'_, AppService>,
    condition: ConditionDto,
) -> Result<QueryResultDto, CommandError> {
    let today = Local::now().date_naive();
    let result = service.query(&condition.into_condition()?, today)?;
    Ok(QueryResultDto::from_result(&result, today))
}

/// 브라우저를 띄우는 동안 창이 멈추지 않게 메인 스레드 밖(async)에서 돈다
#[tauri::command]
pub async fn open_link(
    service: State<'_, AppService>,
    notice_no: String,
    kind: LinkKindDto,
) -> Result<OpenLinkResultDto, CommandError> {
    let link = service.open_link(&notice_no, kind.into())?;
    Ok(OpenLinkResultDto { url: link.url })
}
