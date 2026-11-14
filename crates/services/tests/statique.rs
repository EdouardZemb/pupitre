//! Contrôles statiques des services et d'I28 : la source des services ne touche ni fichier, ni
//! fil, ni réseau, ni horloge, ni environnement, ni processus, ni table au hacheur tiré du
//! système ; hors des trois fichiers qui appliquent les opérations nommées, aucun code n'écrit
//! dans un document (I28, § 70 · 3) ; l'identifiant de propriétaire ne se forme qu'à un seul
//! endroit. Les tests lisent sur le disque chaque `src/` de `crates/` et d'`app/`.

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

/// Des sources saines : chacune porte ce que sa place permet, et qu'une autre place refuserait.
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
    ] {
        refuse(&avec(essai, import), essai, "application");
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
}

#[test]
fn le_depot_passe_chaque_controle() {
    assert_eq!(controler(&sources_du_depot()), Ok(()));
}
