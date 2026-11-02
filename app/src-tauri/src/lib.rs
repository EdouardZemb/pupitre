//! L'établi : l'enveloppe Tauri de l'interface.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("l'établi ne démarre pas");
}
