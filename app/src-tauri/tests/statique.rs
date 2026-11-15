//! Contrôle statique d'I30 (§ 70 · 3) : hors du module `systeme` de l'établi, aucun code ne devine
//! un chemin, ne lit l'horloge ni ne tire l'aléa ; l'acteur d'Automerge ne naît que dans quatre
//! fichiers. Le test lit sur le disque chaque `src/` de `crates/` et celui de l'établi ; une
//! racine dont aucune source n'est lue est une faute. L'interface a sa propre règle.

use std::fs;
use std::path::{Path, PathBuf};

/// Les dossiers de `crates/` que le contrôle doit trouver : une crate ajoutée est lue sans
/// retouche, mais ce plancher garde un dossier renommé de passer sans rien lire.
const AU_MOINS_DES_CRATES: usize = 6;

/// L'établi, dont les sources sont lues, nommées depuis la racine du dépôt.
const ETABLI: &str = "app/src-tauri/src/";

/// Une source inventée, nommée depuis la racine du dépôt.
fn source(chemin: &str, texte: &str) -> (String, String) {
    (chemin.to_string(), texte.to_string())
}

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

/// La racine du dépôt, deux dossiers au-dessus de l'établi.
fn racine_du_depot() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("la racine du dépôt")
}

/// Les `src/` lus : celui de chaque dossier de `crates/`, listés à l'exécution, et celui de
/// l'établi.
fn dossiers_src(racine: &Path) -> Vec<PathBuf> {
    let mut dossiers: Vec<PathBuf> = fs::read_dir(racine.join("crates"))
        .expect("lecture de crates/")
        .map(|entree| entree.expect("entrée").path().join("src"))
        .filter(|src| src.is_dir())
        .collect();
    dossiers.push(racine.join("app/src-tauri/src"));
    dossiers.sort();
    dossiers
}

/// Les sources du dépôt, lues sur le disque, nommées depuis la racine du dépôt.
fn sources_du_depot() -> Vec<(String, String)> {
    let racine = racine_du_depot();
    let mut fichiers = Vec::new();
    for src in dossiers_src(&racine) {
        fichiers_rs(&src, &mut fichiers);
    }
    fichiers.sort();
    fichiers
        .iter()
        .map(|f| (relatif(&racine, f), lire(f)))
        .collect()
}

/// `controler` refuse `sources` : chaque faute nomme `chemin`, et l'une nomme `mot`.
fn refuse(sources: &[(String, String)], chemin: &str, mot: &str) {
    let fautes = controler(sources);
    assert!(!fautes.is_empty(), "{chemin} : « {mot} » admis");
    assert!(fautes.iter().all(|f| f.contains(chemin)), "{fautes:?}");
    assert!(
        fautes.iter().any(|f| f.contains(&format!("« {mot} »"))),
        "{chemin} : « {mot} » non nommé : {fautes:?}"
    );
}

#[test]
fn i30_hors_de_systeme_ni_chemin_devine_ni_horloge_ni_alea() {
    let fautes = controler(&sources_du_depot());
    assert!(fautes.is_empty(), "{fautes:#?}");
}

#[test]
fn i30_chaque_racine_lue_porte_des_sources() {
    let racine = racine_du_depot();
    let sources = sources_du_depot();
    let crates: Vec<_> = dossiers_src(&racine)
        .into_iter()
        .map(|src| relatif(&racine, &src) + "/")
        .filter(|src| src.starts_with("crates/"))
        .collect();
    assert!(
        crates.len() >= AU_MOINS_DES_CRATES,
        "moins de {AU_MOINS_DES_CRATES} crates lues : {crates:?}"
    );
    for src in crates.iter().map(String::as_str).chain([ETABLI]) {
        assert!(
            sources.iter().any(|(chemin, _)| chemin.starts_with(src)),
            "aucune source lue sous {src}"
        );
    }
}

#[test]
fn i30_chaque_controle_refuse_ce_qu_on_y_plante() {
    let lib = "app/src-tauri/src/lib.rs";
    let systeme = "app/src-tauri/src/systeme.rs";
    // Le seul `temp_dir` admis de l'établi : un second est refusé.
    assert!(
        controler(&[source(lib, "let d = std::env::temp_dir().join(nom);\n")]).is_empty(),
        "le temp_dir de dossier_jetable est refusé"
    );
    refuse(
        &[source(
            lib,
            "let d = std::env::temp_dir().join(nom);\nlet e = std::env::temp_dir();\n",
        )],
        lib,
        "temp_dir",
    );
    // Un import groupé, qui ne nomme pas `std::env::temp_dir`.
    let cahier = "crates/cahier/src/main.rs";
    refuse(
        &[source(
            cahier,
            "use std::{env, path::Path};\nfn f() { let _ = env::temp_dir(); }\n",
        )],
        cahier,
        "temp_dir",
    );
    refuse(
        &[source(lib, "let d = app.path().home_dir();\n")],
        lib,
        "home_dir",
    );
    refuse(
        &[source(
            lib,
            "let d = app.path().resolve(\"x\", BaseDirectory::Download);\n",
        )],
        lib,
        "BaseDirectory",
    );
    let stockage = "crates/stockage/src/lib.rs";
    refuse(
        &[source(
            stockage,
            "let d = std::env::var(\"LOCALAPPDATA\");\n",
        )],
        stockage,
        "\"LOCALAPPDATA\"",
    );
    let services = "crates/services/src/lib.rs";
    refuse(
        &[source(services, "let t = SystemTime::now();\n")],
        services,
        "SystemTime",
    );
    let documents = "crates/stockage/src/documents.rs";
    refuse(
        &[source(
            documents,
            "getrandom::fill(&mut octets).unwrap();\n",
        )],
        documents,
        "getrandom",
    );
    let synchro = "crates/synchro/src/lib.rs";
    refuse(
        &[source(synchro, "let doc = Automerge::new();\n")],
        synchro,
        "Automerge::new",
    );
    let lecture = "crates/services/src/lecture.rs";
    refuse(
        &[source(lecture, "let copie = doc.fork();\n")],
        lecture,
        ".fork(",
    );
    // L'acteur n'est pas admis dans `systeme`, qui n'admet que les chemins, l'horloge et l'aléa.
    refuse(
        &[source(systeme, "let a = ActorId::random();\n")],
        systeme,
        "ActorId::random",
    );
    // `systeme` admet l'horloge ; les quatre fichiers admettent l'acteur.
    assert!(
        controler(&[source(systeme, "let t = SystemTime::now();\n")]).is_empty(),
        "systeme refuse l'horloge"
    );
    assert!(
        controler(&[source(documents, "let doc = Automerge::new();\n")]).is_empty(),
        "documents.rs refuse l'acteur"
    );
    // Un mot dans un commentaire ou une doc ne compte pas.
    assert!(
        controler(&[source(
            lecture,
            "/// home_dir\n    // SystemTime\nfn f() {}\n"
        )])
        .is_empty(),
        "un commentaire est refusé"
    );
}
