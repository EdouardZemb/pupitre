//! Le fonds de mesure, le test 13 du § 31 · 10, entier (question 28) : engendré sur la paire
//! d'adaptateurs en mémoire par les opérations de l'import (`services::mesure`, qui dit ce qui y
//! reste absent), les documents changeant d'acteur toutes les sept opérations (question 29), écrit
//! en fichier complet dans une racine des données jetable, puis rouvert sur fichiers ; puis 101
//! notes sur le fonds rouvert, dont la dernière compacte ses 101 fichiers d'ajout (question 30).
//! Le test est ignoré : une étape à part de `verifier`, « Le fonds de mesure (test 13) », le
//! lance, en `--release`, et imprime le poids du fichier complet, les acteurs et la durée d'une
//! compaction.

mod commun;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use automerge::{ROOT, ReadDoc, Value};
use commandes::Noter;
use noyau::donnees::compaction::AJOUTS_AU_PLUS;
use noyau::donnees::fonds::{PRISES, STEM};
use noyau::donnees::gestes;
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::lecture::lire_l_idee;
use services::mesure::{AleaDuGenerateur, Engendre, GRAINE, engendrer_le_fonds_de_mesure};
use services::{Contexte, Documents, Services};
use stockage::FondsSurFichiers;

/// Le fichier complet du fonds de mesure pèse moins que ces octets : le fonds tient sous 4,5 Mo,
/// historique compris (§ 31 · 2).
const POIDS_MAX: u64 = 4_500_000;

/// Chaque texte du fonds de mesure en fait au moins autant d'octets ; leur moyenne aussi.
const OCTETS_D_UN_TEXTE: usize = 200;

/// Les documents en mémoire changent d'acteur après chaque série de ces opérations (question 29).
const OPERATIONS_PAR_ACTEUR: usize = 7;

/// Les acteurs que porte au moins le fonds de mesure : « un acteur par ouverture, 3 000 en tout »
/// (§ 31 · 10, test 13).
const ACTEURS_AU_MOINS: usize = 3_000;

/// Les entrées `prises/<p>/stem/<s>` du fonds de mesure : douze stems dans chacune de ses 100
/// prises en stems ; elles pèsent dans le fichier complet sans opération de plus (balayage Q4).
const ENTREES_DE_STEMS: usize = 1_200;

/// Une compaction de `AJOUTS_AU_PLUS` + 1 fichiers d'ajout, en `--release`, est trop lente
/// au-delà (question 30).
const COMPACTION_AU_PLUS: Duration = Duration::from_secs(2);

/// Les opérations d'un import du fonds visé (§ 31 · 10) : 6 000 versements, l'ouverture et la
/// clôture.
const OPERATIONS_D_UN_IMPORT_VISE: usize = 6_002;

// § 31 · 10, test 13 : 2 000 idées, 6 000 prises par vingt imports, dont 100 prises en stems, 200 octets de texte en moyenne, chaque texte écrit trois fois, trois étiquettes et un lien par idée, 3 000 acteurs ; une compaction de 101 fichiers d'ajout en 2 s au plus
#[test]
#[ignore = "le fonds de mesure : une étape à part de verifier, en --release"]
fn le_fonds_de_mesure_pese_moins_de_4_500_000_octets() {
    let debut = Instant::now();
    let mut documents = DocumentsEnMemoire::neuf();
    documents.changer_d_acteur_tous_les(
        OPERATIONS_PAR_ACTEUR,
        Box::new(AleaDuGenerateur::de(GRAINE + 1)),
    );
    let mut en_memoire = Services::new(
        documents,
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
            operations: 23_090,
        }
    );
    let acteurs: BTreeSet<_> = en_memoire
        .documents()
        .fonds()
        .expect("le fonds est né")
        .get_changes(&[])
        .iter()
        .map(|modification| modification.actor_id().clone())
        .collect();
    let acteurs = acteurs.len();
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
    let mut rouvert = Services::new(fonds, racine.captations(), gestes::Appareil::Etabli);
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
    // Les entrées de stems : sous `prises`, les clés de trois segments, `<p>/stem/<s>` ; un nom
    // d'origine, `<e>/nom/<chemin>`, en a plus, chaque chemin portant au moins un dossier.
    let prises = match doc.get(&ROOT, PRISES) {
        Ok(Some((Value::Object(_), prises))) => prises,
        autre => panic!("le fonds n'a pas de prises : {autre:?}"),
    };
    let stems = doc
        .keys(&prises)
        .filter(|cle| {
            let segments: Vec<&str> = cle.split('/').collect();
            segments.len() == 3 && segments[1] == STEM
        })
        .count();
    assert_eq!(stems, ENTREES_DE_STEMS);

    // Les notes 1 à 100 laissent chacune leur fichier d'ajout ; la 101e, qui en fait passer le
    // nombre au-delà d'`AJOUTS_AU_PLUS`, compacte (§ 31 · 2.2) : sa durée, son propre fichier
    // d'ajout compris, est celle d'une compaction de 101 fichiers d'ajout.
    let ajouts = dossier.join("ajouts");
    let mut alea = AleaDuGenerateur::de(GRAINE + 2);
    let mut heure = commun::HEURE + 1_000_000;
    let mut compaction = Duration::ZERO;
    for rang in 1..=AJOUTS_AU_PLUS + 1 {
        heure += 1;
        let note = Noter {
            texte: format!("Une note de mesure, la {rang}e"),
            decalage_minutes: 0,
        };
        let mut contexte = Contexte {
            heure,
            alea: &mut alea,
        };
        let debut = Instant::now();
        rouvert
            .noter(&note, &mut contexte)
            .unwrap_or_else(|refus| panic!("la note n° {rang} ne s'écrit pas : {refus:?}"));
        let duree = debut.elapsed();
        let restants = fichiers_d_ajout(&ajouts);
        if rang <= AJOUTS_AU_PLUS {
            assert_eq!(restants, rang, "après la note n° {rang}");
        } else {
            assert_eq!(restants, 0, "la note n° {rang} n'a pas compacté");
            compaction = duree;
        }
    }
    let compactions = OPERATIONS_D_UN_IMPORT_VISE / (AJOUTS_AU_PLUS + 1);
    let pour_un_import = compaction * u32::try_from(compactions).expect("un compte de compactions");

    let poids = commun::poids(&complet);
    println!(
        "Le fonds de mesure : {poids} octets ; {acteurs} acteurs ; une compaction de {} fichiers \
         d'ajout : {compaction:.1?} ; {compactions} compactions pour un import du fonds visé : \
         {pour_un_import:.1?}.",
        AJOUTS_AU_PLUS + 1
    );
    println!(
        "Il porte {stems} entrées de stems et {moyenne:.1} octets de texte en moyenne ; engendré \
         en {generation:.1?}, rouvert en {reouverture:.1?}."
    );
    assert!(
        acteurs >= ACTEURS_AU_MOINS,
        "{acteurs} acteurs, moins de {ACTEURS_AU_MOINS}"
    );
    assert!(
        octets_de_texte >= OCTETS_D_UN_TEXTE * idees.len(),
        "{moyenne:.1} octets de texte en moyenne, moins de {OCTETS_D_UN_TEXTE}"
    );
    assert!(
        poids < POIDS_MAX,
        "le fonds de mesure pèse {poids} octets, pas moins de {POIDS_MAX}"
    );
    assert!(
        compaction <= COMPACTION_AU_PLUS,
        "une compaction de {} fichiers d'ajout dure {compaction:.1?}, plus de {COMPACTION_AU_PLUS:?}",
        AJOUTS_AU_PLUS + 1
    );
}

/// Les fichiers d'ajout du dossier `ajouts`, `<empreinte>.automerge` : leur nombre ; un dossier
/// absent n'en a aucun.
fn fichiers_d_ajout(ajouts: &Path) -> usize {
    if !ajouts.exists() {
        return 0;
    }
    commun::entrees(ajouts)
        .iter()
        .filter(|nom| nom.ends_with(".automerge"))
        .count()
}
