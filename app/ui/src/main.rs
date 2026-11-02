//! L'interface de pupitre : l'établi, sa table de la tonalité, sa palette et l'export du cahier
//! des tables.

mod app;
mod entete;
mod export;
mod panneau_palette;
mod vue_table;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
