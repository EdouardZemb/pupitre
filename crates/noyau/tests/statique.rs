//! Contrôles statiques du noyau : sa source ne nomme ni fichier, ni flux, ni environnement, ni
//! fil, ni réseau, ni processus, ni horloge, ni aléa, ni table au hacheur tiré du système, et
//! `ulid` y entre sans ses fonctions par défaut. Les tests lisent les sources sur le disque.

use std::fs;
use std::path::{Path, PathBuf};

fn lire(chemin: &Path) -> String {
    fs::read_to_string(chemin).unwrap_or_else(|e| panic!("lecture de {} : {e}", chemin.display()))
}

/// Tous les fichiers `*.rs` sous `dossier`, en descendant dans les sous-dossiers.
fn fichiers_rs(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    for entree in fs::read_dir(dossier).expect("lecture du dossier") {
        let chemin = entree.expect("entrée").path();
        if chemin.is_dir() {
            fichiers_rs(&chemin, sortie);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            sortie.push(chemin);
        }
    }
}

/// `chemin`, relatif à `racine`, ses composants joints par `/`, sous Windows comme sous Linux.
fn relatif(racine: &Path, chemin: &Path) -> String {
    let reste = chemin
        .strip_prefix(racine)
        .expect("un chemin sous la racine");
    reste
        .components()
        .map(|composant| composant.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Les sources du noyau, lues sur le disque : chaque `.rs` de `src/`, en descendant, puis son
/// `Cargo.toml`, nommés depuis la racine du dépôt.
fn sources_du_noyau() -> Vec<(String, String)> {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut fichiers = Vec::new();
    fichiers_rs(&racine.join("src"), &mut fichiers);
    assert!(!fichiers.is_empty(), "aucune source Rust trouvée");
    fichiers.sort();
    fichiers.push(racine.join("Cargo.toml"));
    fichiers
        .iter()
        .map(|f| (format!("crates/noyau/{}", relatif(racine, f)), lire(f)))
        .collect()
}

/// Une source inventée, nommée depuis la racine du dépôt.
fn source(chemin: &str, texte: &str) -> (String, String) {
    (chemin.to_string(), texte.to_string())
}

/// Une source saine du noyau, et un `Cargo.toml` réduit à la ligne d'`ulid` exigée.
fn saines() -> Vec<(String, String)> {
    vec![
        source(
            "crates/noyau/src/lib.rs",
            "use std::collections::BTreeMap;\n\npub fn compte(table: &BTreeMap<u8, u8>) -> usize {\n    table.len()\n}\n",
        ),
        source(
            "crates/noyau/Cargo.toml",
            "ulid = { version = \"3.0.0\", default-features = false }\n",
        ),
    ]
}

/// Les sources saines, où `chemin` porte `texte`, à la place de sa source saine ou en plus.
fn avec(chemin: &str, texte: &str) -> Vec<(String, String)> {
    let mut sources: Vec<_> = saines().into_iter().filter(|(c, _)| c != chemin).collect();
    sources.push(source(chemin, texte));
    sources
}

/// `controler` refuse `sources` : chaque faute nomme `chemin`, et l'une nomme `mot`.
fn refuse(sources: &[(String, String)], chemin: &str, mot: &str) {
    let Err(fautes) = controler(sources) else {
        panic!("{chemin} : « {mot} » admis");
    };
    assert!(fautes.iter().all(|f| f.contains(chemin)), "{fautes:?}");
    assert!(
        fautes.iter().any(|f| f.contains(&format!("« {mot} »"))),
        "{chemin} : « {mot} » non nommé : {fautes:?}"
    );
}

#[test]
fn chaque_controle_refuse_ce_qu_on_y_plante() {
    // La source saine, et la ligne d'`ulid` seule, `default-features = false` compris : admises.
    assert_eq!(controler(&saines()), Ok(()));
    // Un contrôle qui ne lit aucune source échoue.
    let Err(fautes) = controler(&[]) else {
        panic!("aucune source lue, et le contrôle passe");
    };
    assert!(
        fautes.iter().any(|f| f == "aucune source Rust lue"),
        "{fautes:?}"
    );
    let plante = "crates/noyau/src/plante.rs";
    refuse(&avec(plante, "use std::fs;\n"), plante, "std::fs");
    // Un import groupé ne nomme pas `std::fs` : le chemin court l'attrape.
    refuse(
        &avec(
            plante,
            "use std::{fs, path::Path};\nfn f() { let _ = fs::read(\"x\"); }\n",
        ),
        plante,
        "fs::",
    );
    refuse(
        &avec(
            plante,
            "use std::collections::HashMap;\nfn f() -> HashMap<u8, u8> { HashMap::new() }\n",
        ),
        plante,
        "HashMap",
    );
    // La ligne d'`ulid` qui porte aussi `features = ["std"]` : une clé `features`.
    let cargo = "crates/noyau/Cargo.toml";
    refuse(
        &avec(
            cargo,
            "ulid = { version = \"3.0.0\", default-features = false, features = [\"std\"] }\n",
        ),
        cargo,
        "features",
    );
    // `ulid` avec ses fonctions par défaut : la ligne exigée manque.
    refuse(
        &avec(cargo, "ulid = \"3.0.0\"\n"),
        cargo,
        r#"ulid = { version = "3.0.0", default-features = false }"#,
    );
}

#[test]
fn le_depot_passe_chaque_controle() {
    assert_eq!(controler(&sources_du_noyau()), Ok(()));
}
