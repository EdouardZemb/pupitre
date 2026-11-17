//! La fenêtre de l'établi s'ouvre agrandie : sur tout écran, elle prend la place que laisse la
//! barre des tâches. Sa taille rétablie, 1440 × 900 (le profil « Établi » du § 70 · 2), Windows la
//! borne à l'écran, non à cette place : sur un écran de 1366 × 768, son bas passait sous la barre
//! des tâches (l'essai installé de la tranche 3).

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn la_fenetre_de_l_etabli_s_ouvre_agrandie() {
    let chemin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let texte = fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("lecture de {} : {e}", chemin.display()));
    let conf: Value = serde_json::from_str(&texte).expect("tauri.conf.json est du JSON");
    let fenetres = conf["app"]["windows"]
        .as_array()
        .expect("app.windows est une liste");
    assert_eq!(fenetres.len(), 1, "l'établi a une seule fenêtre");
    assert_eq!(
        fenetres[0]["maximized"],
        Value::Bool(true),
        "la fenêtre de l'établi ne s'ouvre pas agrandie"
    );
}
