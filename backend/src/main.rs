// 릴리스 빌드에서 콘솔 창 없이 뜨게 한다
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod core;
mod domains;

use tauri::Manager;

use domains::animal::adapters::opener::TauriOpener;
use domains::animal::adapters::pawinhand::PawinhandSource;
use domains::animal::commands::{self, AppService};
use domains::animal::service::AnimalService;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // User-Agent에 밝히는 버전은 tauri.conf.json의 버전이다
            let user_agent = core::config::user_agent(&app.package_info().version.to_string());
            let service: AppService =
                AnimalService::new(PawinhandSource::new(&user_agent), TauriOpener::new(app.handle().clone()));
            app.manage(service);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::load_animals, commands::query_animals, commands::open_link])
        .run(tauri::generate_context!())
        .expect("앱을 시작하지 못했다");
}
