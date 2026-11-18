//! L'interface de pupitre : l'établi, son bandeau, sa table de la tonalité, sa palette et l'export
//! du cahier des tables, sa boîte d'idées, et le pont des commandes de l'établi.

mod app;
mod bandeau;
mod boite;
mod entete;
mod export;
mod natif;
mod panneau_palette;
mod recherche;
mod vue_table;

fn main() {
    leptos::mount::mount_to_body(app::App);
}
