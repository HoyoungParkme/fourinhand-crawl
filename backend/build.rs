fn main() {
    // 화면에 여는 앱 커맨드를 이 셋으로 못 박는다 — capabilities/default.json이 하나씩 허용한다
    let manifest = tauri_build::AppManifest::new().commands(&["load_animals", "query_animals", "open_link"]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest)).expect("tauri build");
}
