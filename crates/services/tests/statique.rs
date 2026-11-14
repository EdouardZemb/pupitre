//! Contrôles statiques des services et d'I28 : la source des services ne touche ni fichier, ni
//! fil, ni réseau, ni horloge, ni environnement, ni processus, ni table au hacheur tiré du
//! système ; hors des trois fichiers qui appliquent les opérations nommées, aucun code n'écrit
//! dans un document (I28, § 70 · 3) ; l'identifiant de propriétaire ne se forme qu'à un seul
//! endroit. Les tests lisent sur le disque chaque `src/` de `crates/` et d'`app/`.

use std::fs;
use std::path::{Path, PathBuf};

/// Les mots que la source des services (`crates/services/src/`) ne porte jamais. Les services
/// conduisent les commandes sur les contrats des adaptateurs (§ 40 · 3) : ni fichier, ni fil, ni
/// réseau, ni processus, ni environnement ; l'heure du geste y arrive en argument (`Contexte`) :
/// ni horloge. L'aléa n'y est pas interdit : l'acteur d'Automerge s'y tire (`Automerge::new` en
/// tire un au hasard). Les chemins courts (`fs::`, `thread::`, `env::`…) attrapent un import
/// groupé, où le chemin long ne paraît pas : `use std::{fs, path::Path};`, puis `fs::read`.
///
/// `HashMap`, `HashSet` et `RandomState` : sous leur hacheur par défaut, `new()`, `default()` et
/// `collect()` tirent leur graine du système sans qu'aucun mot de l'aléa ne paraisse, et l'ordre
/// d'itération change d'un passage à l'autre ; une règle qui en dépendrait ne serait plus
/// déterministe. `BTreeMap` et `BTreeSet` restent permis.
///
/// Chaque recherche de ce fichier se fait en sous-chaîne, sensible à la casse, commentaires
/// compris : un identifiant qui contient l'un de ces mots la fait échouer, et c'est voulu.
const INTERDITS_AUX_SERVICES: &[&str] = &[
    "std::fs",
    "fs::",
    "File",
    "std::thread",
    "thread::",
    "spawn",
    "std::net",
    "net::",
    "TcpStream",
    "UdpSocket",
    "SystemTime",
    "Instant",
    "std::time",
    "time::",
    "std::env",
    "env::",
    "env!",
    "std::process",
    "process::",
    "HashMap",
    "HashSet",
    "RandomState",
];

/// Ce qui écrit dans un document Automerge, c'est-à-dire y crée une modification : une
/// transaction, ce qu'elle pose ou retire, et son commit (I28). `.insert(` n'y est pas, puisque
/// c'est aussi celui d'un `BTreeSet` ; mais aucune insertion ne s'écrit sans `Transactable` ou
/// `automerge::transaction`, qui y sont. La synchronisation, elle, ne crée aucune modification :
/// elle reprend celles nées d'opérations nommées (`receive_sync_message`, dans `synchro`), ce que
/// le § 70 · 3 admet, et aucun de ces mots ne la nomme.
const ECRITURES: &[&str] = &[
    ".put(",
    ".put_object(",
    ".delete(",
    ".increment(",
    ".splice(",
    ".splice_text(",
    ".insert_object(",
    ".transaction(",
    ".transact(",
    ".transact_with(",
    ".empty_commit(",
    ".commit_with(",
    ".commit(",
    "AutoCommit",
    "Transactable",
    "automerge::transaction",
    "Transaction<",
];

/// Les seuls fichiers qui écrivent dans un document : `services::application`, qui applique les
/// opérations nommées, et les documents sur fichiers du stockage, qui l'appellent (`documents.rs`,
/// et `racine_des_donnees.rs`, où naît le fonds), et gardent pour leurs tests l'écriture par une
/// fermeture, sous la fonction `pannes`.
const ECRIVENT: &[&str] = &[
    "crates/services/src/application.rs",
    "crates/stockage/src/documents.rs",
    "crates/stockage/src/racine_des_donnees.rs",
];

/// Les appels des fonctions publiques de `services::application` qui écrivent : la parenthèse
/// les distingue d'une doc qui les nomme.
const APPELS_QUI_ECRIVENT: &[&str] = &[
    "application::appliquer(",
    "application::naitre(",
    "application::faire_naitre(",
    "application::migrer(",
];

/// Où ces appels, et une ligne `use` qui porte `application`, sont permis : les services, et les
/// documents sur fichiers du stockage.
const APPELLENT: &[&str] = &[
    "crates/services/src/",
    "crates/stockage/src/documents.rs",
    "crates/stockage/src/racine_des_donnees.rs",
];

/// Le seul fichier qui forme l'identifiant de propriétaire (§ 30 · 2), par le seul appel
/// d'`uuid_v4` hors de sa définition, au noyau.
const PROPRIETAIRE: &str = "crates/services/src/lib.rs";

/// Les racines dont les contrôles lisent au moins une source : les services, le stockage et la
/// synchronisation, qui tiennent un document, l'interface et l'établi, qui n'en écrivent pas. Un
/// membre dont la source quitte `src/` n'y serait lu par personne, et I28 resterait vert sans
/// l'avoir vérifié.
const RACINES_LUES: &[&str] = &[
    "crates/services/src/",
    "crates/stockage/src/",
    "crates/synchro/src/",
    "app/ui/src/",
    "app/src-tauri/src/",
];

/// Les fautes des services : un mot interdit dans `crates/services/src/` ; aucune source des
/// services lue en est une.
fn fautes_des_services(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    let services: Vec<_> = sources
        .iter()
        .filter(|(chemin, _)| chemin.starts_with("crates/services/src/"))
        .collect();
    if services.is_empty() {
        fautes.push("aucune source des services lue".to_string());
    }
    for (chemin, texte) in services {
        for mot in INTERDITS_AUX_SERVICES {
            if texte.contains(mot) {
                fautes.push(format!("{chemin} contient « {mot} »"));
            }
        }
    }
    fautes
}

/// Vrai si `ligne` est une ligne `use`, `pub use` et `pub(crate) use` compris.
fn est_un_use(ligne: &str) -> bool {
    let mut reste = ligne.trim_start();
    if let Some(apres) = reste.strip_prefix("pub") {
        reste = apres.trim_start();
        if reste.starts_with('(') {
            reste = reste
                .split_once(')')
                .map_or("", |(_, suite)| suite.trim_start());
        }
    }
    reste
        .strip_prefix("use")
        .is_some_and(|suite| suite.starts_with(char::is_whitespace))
}

/// Les fautes d'I28. Hors d'`ECRIVENT`, aucune écriture dans un document. Hors d'`APPELLENT`,
/// aucun appel d'une fonction de `services::application` qui écrit, et aucune ligne `use` qui
/// porte `application`, quel que soit l'import : nommé (`use services::application::appliquer;`,
/// puis `appliquer(`), groupé (`use services::{application, lecture};`), en glob
/// (`use services::application::*;`) ou renommé (`use services::application as w;`, puis
/// `w::appliquer(`). Le chemin en ligne `services::application::Inapplicable` reste admis : ni
/// appel ni ligne `use`. Une ligne `use` se reconnaît à son début : un `use` écrit sur plusieurs
/// lignes, dont la ligne qui porte `application` ne commence pas par `use`, échappe au contrôle.
fn fautes_d_i28(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    for (chemin, texte) in sources {
        if !ECRIVENT.contains(&chemin.as_str()) {
            for mot in ECRITURES {
                if texte.contains(mot) {
                    fautes.push(format!("{chemin} contient « {mot} »"));
                }
            }
        }
        if !APPELLENT.iter().any(|permis| chemin.starts_with(permis)) {
            for appel in APPELS_QUI_ECRIVENT {
                if texte.contains(appel) {
                    fautes.push(format!("{chemin} contient « {appel} »"));
                }
            }
            for ligne in texte.lines() {
                if est_un_use(ligne) && ligne.contains("application") {
                    fautes.push(format!(
                        "{chemin} importe « application » : {}",
                        ligne.trim()
                    ));
                }
            }
        }
    }
    fautes
}

/// Les fautes de l'identifiant de propriétaire : `uuid_v4(` paraît, hors de sa définition
/// (`fn uuid_v4(`), une fois dans `PROPRIETAIRE`, et nulle part ailleurs.
fn fautes_du_proprietaire(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    let mut ici = 0;
    for (chemin, texte) in sources {
        let appels = texte.matches("uuid_v4(").count() - texte.matches("fn uuid_v4(").count();
        if chemin == PROPRIETAIRE {
            ici = appels;
        } else if appels > 0 {
            fautes.push(format!(
                "{chemin} contient « uuid_v4( », hors de {PROPRIETAIRE}"
            ));
        }
    }
    if ici != 1 {
        fautes.push(format!(
            "{PROPRIETAIRE} contient « uuid_v4( » {ici} fois, au lieu d'une"
        ));
    }
    fautes
}

/// Le contrôle des services, d'I28 et de l'identifiant de propriétaire, sur `sources`, des
/// paires (chemin, texte), nommées depuis la racine du dépôt. Rend chaque faute, qui nomme son
/// fichier et ce qu'il porte.
fn controler(sources: &[(String, String)]) -> Result<(), Vec<String>> {
    let mut fautes = fautes_des_services(sources);
    fautes.extend(fautes_d_i28(sources));
    fautes.extend(fautes_du_proprietaire(sources));
    if fautes.is_empty() {
        Ok(())
    } else {
        Err(fautes)
    }
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

/// Les sources du dépôt, lues sur le disque : chaque `.rs` de chaque `src/` de `crates/` et
/// d'`app/`, en descendant, nommé depuis la racine du dépôt, deux dossiers au-dessus de la
/// crate des services.
fn sources_du_depot() -> Vec<(String, String)> {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("la racine du dépôt");
    let mut fichiers = Vec::new();
    for groupe in ["crates", "app"] {
        for entree in fs::read_dir(racine.join(groupe)).expect("lecture du dossier") {
            let src = entree.expect("entrée").path().join("src");
            if src.is_dir() {
                fichiers_rs(&src, &mut fichiers);
            }
        }
    }
    assert!(!fichiers.is_empty(), "aucune source Rust trouvée");
    fichiers.sort();
    fichiers
        .iter()
        .map(|f| (relatif(racine, f), lire(f)))
        .collect()
}

/// Une source inventée, nommée depuis la racine du dépôt.
fn source(chemin: &str, texte: &str) -> (String, String) {
    (chemin.to_string(), texte.to_string())
}

/// Des sources saines : chacune porte ce que sa place permet, et qu'une autre place refuserait ;
/// chaque racine de `RACINES_LUES` en a au moins une.
fn saines() -> Vec<(String, String)> {
    vec![
        source(
            "crates/services/src/lib.rs",
            "use noyau::donnees::identifiants::uuid_v4;\n\nfn identifiant_de_proprietaire(octets: [u8; 16]) -> String {\n    uuid_v4(octets)\n}\n",
        ),
        source(
            "crates/services/src/application.rs",
            "use automerge::transaction::Transactable;\n\npub fn appliquer(doc: &mut Automerge) {\n    let mut tx = doc.transaction();\n    tx.put(ROOT, \"cle\", 1).unwrap();\n    tx.commit();\n}\n",
        ),
        source(
            "crates/services/src/en_memoire.rs",
            "use crate::application;\n\nfn appliquer(doc: &mut Automerge, operation: &Operation) {\n    application::appliquer(doc, operation).unwrap();\n}\n",
        ),
        source(
            "crates/stockage/src/documents.rs",
            "use services::application;\n\nfn appliquer(doc: &mut Automerge, operation: &Operation) {\n    application::appliquer(doc, operation).unwrap();\n    let mut tx = doc.transaction();\n    tx.commit();\n}\n",
        ),
        source(
            "crates/stockage/src/racine_des_donnees.rs",
            "use automerge::transaction::Transaction;\n\nfn naitre(geste: impl FnOnce(&mut Transaction<'_>)) {}\n",
        ),
        source(
            "crates/stockage/src/lib.rs",
            "pub enum Erreur {\n    Inapplicable(services::application::Inapplicable),\n}\n",
        ),
        source(
            "crates/synchro/src/lib.rs",
            "fn recevoir(doc: &mut Automerge, etat: &mut State, message: Message) {\n    doc.receive_sync_message(etat, message).unwrap();\n}\n",
        ),
        source(
            "crates/noyau/src/donnees/identifiants.rs",
            "pub fn uuid_v4(octets: [u8; 16]) -> String {\n    String::new()\n}\n",
        ),
        source(
            "app/ui/src/main.rs",
            "fn main() {\n    leptos::mount::mount_to_body(App);\n}\n",
        ),
        source(
            "app/src-tauri/src/lib.rs",
            "pub fn run() {\n    tauri::Builder::default();\n}\n",
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
    // Les sources saines, chacune à sa place : admises.
    assert_eq!(controler(&saines()), Ok(()));
    // Un contrôle qui ne lit aucune source échoue.
    let Err(fautes) = controler(&[]) else {
        panic!("aucune source lue, et le contrôle passe");
    };
    assert!(
        fautes.iter().any(|f| f == "aucune source des services lue"),
        "{fautes:?}"
    );
    // I28 : une écriture, un appel qualifié, et chaque forme d'import, hors des places permises.
    let essai = "crates/interface_essai/src/lib.rs";
    refuse(
        &avec(
            essai,
            "fn f(tx: &mut T) {\n    tx.put(ROOT, \"cle\", 1).unwrap();\n}\n",
        ),
        essai,
        ".put(",
    );
    refuse(
        &avec(
            essai,
            "fn f() {\n    services::application::appliquer(&mut doc, &operation);\n}\n",
        ),
        essai,
        "application::appliquer(",
    );
    for import in [
        "use services::application::appliquer;\nfn f() {\n    appliquer(&mut doc, &operation);\n}\n",
        "use services::application::*;\nfn f() {\n    appliquer(&mut doc, &operation);\n}\n",
        "use services::{application, lecture};\n",
        "use services::application as w;\nfn f() {\n    w::appliquer(&mut doc, &operation);\n}\n",
        "pub use services::application;\n",
        "    pub(crate) use services::application::appliquer;\n",
    ] {
        refuse(&avec(essai, import), essai, "application");
    }
    // I28 : les écritures d'automerge 0.12.0 que nul autre mot ne nomme. `init_from_hydrate`
    // ouvre et committe une transaction ; `transact_and_log_patches` se committe d'elle-même ; le
    // trait des écritures arrive par un glob dans un import groupé ; une transaction possède le
    // document. Chacune est refusée, et la faute la nomme.
    let ecritures = [
        (
            "fn f(doc: &mut Automerge, carte: &Map) {\n    doc.init_from_hydrate(&carte);\n}\n",
            ".init_from_hydrate(",
        ),
        (
            "use automerge::{transaction::*, Automerge};\n",
            "transaction::*",
        ),
        (
            "fn f(doc: &mut Automerge, liste: &ObjId) {\n    doc.transact_and_log_patches(|tx| tx.insert(&liste, 0, 1));\n}\n",
            ".transact",
        ),
        (
            "fn f(doc: Automerge) -> OwnedTransaction {\n    doc.into_transaction(None, None).unwrap()\n}\n",
            "OwnedTransaction",
        ),
    ];
    let admises: Vec<&str> = ecritures
        .iter()
        .filter(|(texte, _)| controler(&avec(essai, texte)).is_ok())
        .map(|(_, mot)| *mot)
        .collect();
    assert!(
        admises.is_empty(),
        "écritures admises hors des fichiers qui écrivent : {admises:?}"
    );
    for (texte, mot) in ecritures {
        refuse(&avec(essai, texte), essai, mot);
    }
    // Les services : un import groupé, qui ne nomme pas `std::fs`, un fil, une table hachée.
    let plante = "crates/services/src/plante.rs";
    refuse(
        &avec(
            plante,
            "use std::{fs, path::Path};\nfn f() { let _ = fs::read(\"x\"); }\n",
        ),
        plante,
        "fs::",
    );
    refuse(
        &avec(plante, "fn f() {\n    std::thread::spawn(|| ());\n}\n"),
        plante,
        "std::thread",
    );
    refuse(
        &avec(
            plante,
            "use std::collections::HashMap;\nfn f() -> HashMap<u8, u8> { HashMap::new() }\n",
        ),
        plante,
        "HashMap",
    );
    // Un second `uuid_v4(`, ailleurs, puis dans le fichier qui forme l'identifiant de propriétaire.
    refuse(
        &avec(
            essai,
            "fn f(octets: [u8; 16]) -> String {\n    uuid_v4(octets)\n}\n",
        ),
        essai,
        "uuid_v4(",
    );
    let proprietaire = "crates/services/src/lib.rs";
    refuse(
        &avec(
            proprietaire,
            "fn identifiant_de_proprietaire(octets: [u8; 16]) -> String {\n    uuid_v4(octets)\n}\n\nfn autre(octets: [u8; 16]) -> String {\n    uuid_v4(octets)\n}\n",
        ),
        proprietaire,
        "uuid_v4(",
    );
    // Aucun `uuid_v4(` : l'identifiant de propriétaire ne se forme plus.
    refuse(
        &avec(proprietaire, "fn identifiant_de_proprietaire() {}\n"),
        proprietaire,
        "uuid_v4(",
    );
}

/// Une racine de `RACINES_LUES` dont aucune source n'est lue, ou un fichier d'`ECRIVENT` non lu,
/// est une faute qui le nomme : sans elle, I28 resterait vert sans l'avoir lu.
#[test]
fn une_racine_ou_un_fichier_qui_ecrit_non_lus_sont_des_fautes() {
    let mut absentes = Vec::new();
    for racine in RACINES_LUES {
        let sources: Vec<_> = saines()
            .into_iter()
            .filter(|(chemin, _)| !chemin.starts_with(racine))
            .collect();
        let attendue = format!("aucune source lue sous {racine}");
        if !controler(&sources).is_err_and(|fautes| fautes.contains(&attendue)) {
            absentes.push(attendue);
        }
    }
    for fichier in ECRIVENT {
        let sources: Vec<_> = saines()
            .into_iter()
            .filter(|(chemin, _)| chemin != fichier)
            .collect();
        let attendue = format!("{fichier} n'est pas lu");
        if !controler(&sources).is_err_and(|fautes| fautes.contains(&attendue)) {
            absentes.push(attendue);
        }
    }
    assert!(
        absentes.is_empty(),
        "fautes attendues, absentes : {absentes:?}"
    );
}

#[test]
fn le_depot_passe_chaque_controle() {
    assert_eq!(controler(&sources_du_depot()), Ok(()));
}
