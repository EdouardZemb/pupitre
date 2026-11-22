//! Contrôle statique d'I30 (§ 70 · 3) : hors du module `systeme` de l'établi, aucun code ne devine
//! un chemin, ne lit l'horloge, ne tire l'aléa, ne lit la place libre du disque ni n'est non sûr
//! (`unsafe`) ; l'acteur d'Automerge ne naît et ne change que dans les fichiers que le contrôle
//! admet (`ACTEUR_ADMIS`). Le test lit sur le disque chaque `src/` de `crates/` et celui de
//! l'établi ; une racine dont aucune source n'est lue est une faute. L'interface a sa propre règle.

use std::fs;
use std::path::{Path, PathBuf};

/// Les dossiers de `crates/` que le contrôle doit trouver : une crate ajoutée est lue sans
/// retouche, mais ce plancher garde un dossier renommé de passer sans rien lire.
const AU_MOINS_DES_CRATES: usize = 6;

/// L'établi, dont les sources sont lues, nommées depuis la racine du dépôt.
const ETABLI: &str = "app/src-tauri/src/";

/// Le manifeste de l'établi, nommé depuis la racine du dépôt, que lit le contrôle d'I29.
const MANIFESTE_DE_L_ETABLI: &str = "app/src-tauri/Cargo.toml";

/// Le seul fichier qui devine un chemin, lit l'horloge, tire l'aléa, lit la place libre du disque
/// et porte du code non sûr.
const SYSTEME: &str = "app/src-tauri/src/systeme.rs";

/// Le fichier de l'établi qui porte `dossier_jetable`, seul admis à nommer `temp_dir`, une fois.
const LIB_DE_L_ETABLI: &str = "app/src-tauri/src/lib.rs";

/// Les chemins devinés : le dossier de l'utilisateur, les dossiers du système, leurs variables
/// d'environnement. Admis dans `SYSTEME` ; `temp_dir` l'est aussi, une fois, dans
/// `LIB_DE_L_ETABLI`. Un chemin se reçoit.
const CHEMINS_DEVINES: &[&str] = &[
    "home_dir",
    "temp_dir",
    "data_dir",
    "data_local_dir",
    "config_dir",
    "cache_dir",
    "document_dir",
    "app_data_dir",
    "app_local_data_dir",
    "app_config_dir",
    "app_cache_dir",
    "app_log_dir",
    "download_dir",
    "desktop_dir",
    "audio_dir",
    "picture_dir",
    "video_dir",
    "public_dir",
    "template_dir",
    "font_dir",
    "runtime_dir",
    "executable_dir",
    "resource_dir",
    "BaseDirectory",
    "SHGetKnownFolderPath",
    "GetTempPath",
    "\"USERPROFILE\"",
    "\"HOME\"",
    "\"APPDATA\"",
    "\"LOCALAPPDATA\"",
    "\"TEMP\"",
    "\"TMP\"",
    "dirs::",
];

/// L'horloge, admise dans `SYSTEME`.
const HORLOGE: &[&str] = &["SystemTime", "Instant", "UNIX_EPOCH"];

/// L'aléa, admis dans `SYSTEME`.
const ALEA: &[&str] = &["getrandom", "rand::", "OsRng", "thread_rng"];

/// La place libre du disque, admise dans `SYSTEME` : l'appel de Windows, celui d'Unix, et ce que
/// des crates offrent pour la lire.
const PLACE_LIBRE: &[&str] = &[
    "GetDiskFreeSpaceEx",
    "statvfs",
    "available_space",
    "fs2::",
    "sysinfo",
];

/// Le code non sûr, admis dans `SYSTEME` seul, où la place libre se lit par un appel de Windows
/// déclaré à la main.
const NON_SUR: &[&str] = &["unsafe"];

/// L'acteur d'Automerge : il naît de l'un de ces mots, ou d'une copie du document, et change par
/// `set_actor`, d'un acteur fait d'octets (`ActorId::from`).
const ACTEUR: &[&str] = &[
    "ActorId::random",
    "Automerge::new",
    ".fork(",
    "fork_at",
    "Automerge::load",
    "load_with_options",
    "set_actor",
    "ActorId::from",
];

/// Les seuls fichiers où l'acteur naît ou change : les documents sur fichiers, le démarrage du
/// fonds, et l'application des opérations nommées avec son adaptateur en mémoire, qui change
/// d'acteur toutes les quelques opérations pour le test 13 (question 29).
const ACTEUR_ADMIS: &[&str] = &[
    "crates/stockage/src/documents.rs",
    "crates/stockage/src/demarrage.rs",
    "crates/services/src/application.rs",
    "crates/services/src/en_memoire.rs",
];

/// Les fautes d'I30 sur `sources`, des paires (chemin, texte) nommées depuis la racine du dépôt.
/// Une ligne dont le début, blancs ôtés, est `//` (commentaire, doc) ne compte pas ; tout le reste
/// se lit en sous-chaîne, sensible à la casse. Chaque faute nomme le fichier, la ligne et le mot.
fn controler(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    for (chemin, texte) in sources {
        let mut temp_dir_vus = 0;
        for (indice, ligne) in texte.lines().enumerate() {
            if ligne.trim_start().starts_with("//") {
                continue;
            }
            let mut faute = |mot: &str| {
                fautes.push(format!("{chemin}:{} contient « {mot} »", indice + 1));
            };
            if chemin != SYSTEME {
                for mot in CHEMINS_DEVINES.iter().filter(|mot| ligne.contains(**mot)) {
                    if *mot == "temp_dir" && chemin == LIB_DE_L_ETABLI {
                        temp_dir_vus += ligne.matches(mot).count();
                        if temp_dir_vus <= 1 {
                            continue;
                        }
                    }
                    faute(mot);
                }
                for mot in HORLOGE
                    .iter()
                    .chain(ALEA)
                    .chain(PLACE_LIBRE)
                    .chain(NON_SUR)
                    .filter(|mot| ligne.contains(**mot))
                {
                    faute(mot);
                }
            }
            if !ACTEUR_ADMIS.contains(&chemin.as_str()) {
                for mot in ACTEUR.iter().filter(|mot| ligne.contains(**mot)) {
                    faute(mot);
                }
            }
        }
    }
    fautes
}

/// Les fautes d'I29 (§ 70 · 3) : côté natif, ni greffon ni API de notification, ni boîte de
/// message. `manifeste` est le `Cargo.toml` de l'établi, `sources` ses sources, nommées depuis la
/// racine du dépôt. Une ligne de source dont le début, blancs ôtés, est `//`, et une ligne du
/// manifeste dont le début est `#`, ne comptent pas. `notification` se lit sans égard à la casse,
/// `.message(` et `MessageDialog` en sous-chaîne ; la boîte d'enregistrement, `.file()`, reste.
fn controler_i29(manifeste: &str, sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    let mut examiner = |chemin: &str, texte: &str, commentaire: &str, mots: &[&str]| {
        for (indice, ligne) in texte.lines().enumerate() {
            if ligne.trim_start().starts_with(commentaire) {
                continue;
            }
            for mot in mots {
                let trouve = if *mot == "notification" {
                    ligne.to_lowercase().contains(mot)
                } else {
                    ligne.contains(mot)
                };
                if trouve {
                    fautes.push(format!("{chemin}:{} contient « {mot} »", indice + 1));
                }
            }
        }
    };
    examiner(MANIFESTE_DE_L_ETABLI, manifeste, "#", &["notification"]);
    for (chemin, texte) in sources {
        examiner(
            chemin,
            texte,
            "//",
            &["notification", ".message(", "MessageDialog"],
        );
    }
    fautes
}

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

/// `controler` refuse `sources` d'une seule faute, `attendue`, telle qu'il la dit : le fichier, la
/// ligne et le mot.
fn refuse(sources: &[(String, String)], attendue: &str) {
    assert_eq!(controler(sources), vec![attendue.to_string()]);
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
    // Le seul `temp_dir` admis de l'établi : un second est refusé, sur sa ligne à lui.
    assert!(
        controler(&[source(lib, "let d = std::env::temp_dir().join(nom);\n")]).is_empty(),
        "le temp_dir de dossier_jetable est refusé"
    );
    refuse(
        &[source(
            lib,
            "let d = std::env::temp_dir().join(nom);\nlet e = std::env::temp_dir();\n",
        )],
        "app/src-tauri/src/lib.rs:2 contient « temp_dir »",
    );
    // Un import groupé, qui ne nomme pas `std::env::temp_dir`.
    let cahier = "crates/cahier/src/main.rs";
    refuse(
        &[source(
            cahier,
            "use std::{env, path::Path};\nfn f() { let _ = env::temp_dir(); }\n",
        )],
        "crates/cahier/src/main.rs:2 contient « temp_dir »",
    );
    refuse(
        &[source(lib, "let d = app.path().home_dir();\n")],
        "app/src-tauri/src/lib.rs:1 contient « home_dir »",
    );
    refuse(
        &[source(
            lib,
            "let d = app.path().resolve(\"x\", BaseDirectory::Download);\n",
        )],
        "app/src-tauri/src/lib.rs:1 contient « BaseDirectory »",
    );
    let stockage = "crates/stockage/src/lib.rs";
    refuse(
        &[source(
            stockage,
            "let d = std::env::var(\"LOCALAPPDATA\");\n",
        )],
        "crates/stockage/src/lib.rs:1 contient « \"LOCALAPPDATA\" »",
    );
    let services = "crates/services/src/lib.rs";
    refuse(
        &[source(services, "let t = SystemTime::now();\n")],
        "crates/services/src/lib.rs:1 contient « SystemTime »",
    );
    let documents = "crates/stockage/src/documents.rs";
    refuse(
        &[source(
            documents,
            "getrandom::fill(&mut octets).unwrap();\n",
        )],
        "crates/stockage/src/documents.rs:1 contient « getrandom »",
    );
    let synchro = "crates/synchro/src/lib.rs";
    refuse(
        &[source(synchro, "let doc = Automerge::new();\n")],
        "crates/synchro/src/lib.rs:1 contient « Automerge::new »",
    );
    let lecture = "crates/services/src/lecture.rs";
    refuse(
        &[source(lecture, "let copie = doc.fork();\n")],
        "crates/services/src/lecture.rs:1 contient « .fork( »",
    );
    // L'acteur n'est pas admis dans `systeme`, qui n'admet que les chemins, l'horloge, l'aléa, la
    // place libre et le code non sûr.
    refuse(
        &[source(systeme, "let a = ActorId::random();\n")],
        "app/src-tauri/src/systeme.rs:1 contient « ActorId::random »",
    );
    // `systeme` admet l'horloge ; les fichiers d'`ACTEUR_ADMIS` admettent l'acteur.
    assert!(
        controler(&[source(systeme, "let t = SystemTime::now();\n")]).is_empty(),
        "systeme refuse l'horloge"
    );
    assert!(
        controler(&[source(documents, "let doc = Automerge::new();\n")]).is_empty(),
        "documents.rs refuse l'acteur"
    );
    // Changer d'acteur, ou en faire un d'octets, hors des fichiers admis : refusé, chacun sur sa
    // ligne ; admis dans l'adaptateur en mémoire, qui fait tourner ses acteurs (question 29).
    let mesure = "crates/services/src/mesure.rs";
    refuse(
        &[source(mesure, "doc.set_actor(acteur);\n")],
        "crates/services/src/mesure.rs:1 contient « set_actor »",
    );
    refuse(
        &[source(mesure, "let acteur = ActorId::from(octets);\n")],
        "crates/services/src/mesure.rs:1 contient « ActorId::from »",
    );
    assert!(
        controler(&[source(
            "crates/services/src/en_memoire.rs",
            "doc.set_actor(ActorId::from(octets));\n"
        )])
        .is_empty(),
        "en_memoire.rs refuse de changer d'acteur"
    );
    // La place libre et le code non sûr : refusés hors de `systeme`, chacun sur sa ligne ; admis
    // dans `systeme`.
    let place_libre = "    fn GetDiskFreeSpaceExW(dossier: *const u16) -> i32;\n";
    let non_sur = "unsafe extern \"system\" {}\n";
    refuse(
        &[source(lib, place_libre)],
        "app/src-tauri/src/lib.rs:1 contient « GetDiskFreeSpaceEx »",
    );
    refuse(
        &[source(stockage, non_sur)],
        "crates/stockage/src/lib.rs:1 contient « unsafe »",
    );
    assert!(
        controler(&[source(systeme, &format!("{non_sur}{place_libre}"))]).is_empty(),
        "systeme refuse la place libre ou le code non sûr"
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

#[test]
fn i29_ni_greffon_ni_api_de_notification_cote_natif() {
    let racine = racine_du_depot();
    let manifeste = lire(&racine.join(MANIFESTE_DE_L_ETABLI));
    let sources: Vec<_> = sources_du_depot()
        .into_iter()
        .filter(|(chemin, _)| chemin.starts_with(ETABLI))
        .collect();
    let fautes = controler_i29(&manifeste, &sources);
    assert!(fautes.is_empty(), "{fautes:#?}");
}

#[test]
fn i29_chaque_controle_refuse_ce_qu_on_y_plante() {
    let lib = "app/src-tauri/src/lib.rs";
    let sain = "[dependencies]\ntauri = \"2\"\n";
    let aucune = Vec::<(String, String)>::new();
    assert!(controler_i29(sain, &aucune).is_empty());
    // Le greffon des notifications, dans le manifeste, sans égard à la casse.
    assert_eq!(
        controler_i29(
            "[dependencies]\ntauri-plugin-notification = \"2\"\n",
            &aucune
        ),
        vec!["app/src-tauri/Cargo.toml:2 contient « notification »".to_string()]
    );
    assert_eq!(
        controler_i29("[dependencies]\nNotification = \"2\"\n", &aucune),
        vec!["app/src-tauri/Cargo.toml:2 contient « notification »".to_string()]
    );
    // Dans les sources : le greffon, une boîte de message, son constructeur.
    for (texte, mot) in [
        ("use tauri_plugin_notification::init;\n", "notification"),
        (
            "app.dialog().message(\"Bonjour\").show(|_| {});\n",
            ".message(",
        ),
        ("let b = MessageDialogBuilder::new();\n", "MessageDialog"),
    ] {
        assert_eq!(
            controler_i29(sain, &[source(lib, texte)]),
            vec![format!("{lib}:1 contient « {mot} »")],
            "{texte}"
        );
    }
    // Les lignes `//` des sources, et `#` du manifeste, ne comptent pas ; la boîte
    // d'enregistrement, `.file()`, reste admise.
    assert!(
        controler_i29(
            "# notification\n[dependencies]\n",
            &[source(
                lib,
                "/// aucune notification (I29)\n    // .message(\nlet d = app.dialog().file();\n"
            )]
        )
        .is_empty()
    );
}
