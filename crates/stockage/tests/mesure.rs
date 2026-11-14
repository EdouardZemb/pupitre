//! Le fonds de mesure (§ 31 · 10, test 13), dans la part qu'en écrit la tranche 2 : engendré sur
//! la paire d'adaptateurs en mémoire, écrit en fichier complet dans une racine des données
//! jetable, puis rouvert sur fichiers. Le test est ignoré : une étape à part de `verifier` le
//! lance, en `--release`, et imprime le poids du fichier complet et la durée.

mod commun;

use std::fs;
use std::time::Instant;

use noyau::donnees::gestes;
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::lecture::lire_l_idee;
use services::mesure::{Engendre, engendrer_le_fonds_de_mesure};
use services::{Documents, Services};
use stockage::FondsSurFichiers;

/// Le fichier complet du fonds de mesure pèse moins que ces octets : le fonds tient sous 4,5 Mo,
/// historique compris (§ 31 · 2).
const POIDS_MAX: u64 = 4_500_000;

/// Chaque texte du fonds de mesure en fait au moins autant d'octets ; leur moyenne aussi.
const OCTETS_D_UN_TEXTE: usize = 200;

// § 31 · 10, test 13 : 2 000 idées, 200 octets de texte en moyenne, chaque texte écrit trois fois, trois étiquettes et un lien par idée
#[test]
#[ignore = "le fonds de mesure : une étape à part de verifier, en --release"]
fn le_fonds_de_mesure_pese_moins_de_4_500_000_octets() {
    let debut = Instant::now();
    let mut en_memoire = Services::new(
        DocumentsEnMemoire::neuf(),
        CaptationsEnMemoire::default(),
        gestes::Appareil::Etabli,
    );
    let engendre = engendrer_le_fonds_de_mesure(&mut en_memoire, commun::HEURE)
        .expect("le fonds de mesure s'engendre");
    let generation = debut.elapsed();
    assert_eq!(
        engendre,
        Engendre {
            idees_visibles: 2_000,
            idees_retirees: 4_000,
            operations: 23_000,
        }
    );
    let octets = en_memoire.documents().octets().expect("le fonds est né");

    let chemin = commun::dossier_jetable("mesure").join("donnees");
    let racine = commun::ouvrir(&chemin);
    let dossier = commun::dossier_du_fonds(&chemin);
    fs::create_dir_all(&dossier).expect("créer documents/fonds/");
    let complet = dossier.join("complet.automerge");
    fs::write(&complet, &octets).expect("écrire le fichier complet");

    let debut = Instant::now();
    let fonds = FondsSurFichiers::ouvrir(&racine).expect("rouvrir le fonds");
    let reouverture = debut.elapsed();
    // Rouvert entier : ni en lecture seule, ni avec un avertissement.
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds rouvert est en lecture seule"
    );
    let document = fonds.document().expect("le fonds est né");
    assert!(
        document.avertissements().is_empty(),
        "le fonds rouvert a des avertissements : {:?}",
        document.avertissements()
    );
    let rouvert = Services::new(fonds, racine.captations(), gestes::Appareil::Etabli);
    let idees = rouvert.idees().expect("les idées se lisent");
    assert_eq!(idees.len(), 2_000);
    let doc = rouvert.documents().fonds().expect("le fonds est né");
    let octets_de_texte: usize = idees
        .iter()
        .map(|idee| {
            lire_l_idee(doc, idee)
                .and_then(|lue| lue.texte)
                .map_or(0, |texte| texte.len())
        })
        .sum();
    let moyenne = octets_de_texte as f64 / idees.len() as f64;
    let poids = commun::poids(&complet);
    println!(
        "Le fonds de mesure : {poids} octets, {moyenne:.1} octets de texte en moyenne ; engendré \
         en {generation:.1?}, rouvert en {reouverture:.1?}."
    );
    assert!(
        octets_de_texte >= OCTETS_D_UN_TEXTE * idees.len(),
        "{moyenne:.1} octets de texte en moyenne, moins de {OCTETS_D_UN_TEXTE}"
    );
    assert!(
        poids < POIDS_MAX,
        "le fonds de mesure pèse {poids} octets, pas moins de {POIDS_MAX}"
    );
}
