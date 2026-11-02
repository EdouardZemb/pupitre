// Pas de console derrière la fenêtre, en version de production sous Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    etabli_lib::run();
}
