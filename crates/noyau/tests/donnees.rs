//! Les règles des données : ce que permet un document ouvert (§ 31 · 2.1), quand compacter,
//! quels fichiers retirer (§ 31 · 2.2), comment se lit la version de schéma et quelles
//! migrations s'appliquent (§ 31 · 2.4), et quelles copies s'échangent (§ 31 · 2.5).

use noyau::donnees::acces::Acces;
use noyau::donnees::compaction::{Lu, a_l_ouverture, a_retirer, apres_une_ecriture};
use noyau::donnees::copie::{Copie, Difference, comparer};
use noyau::donnees::fonds::MIGRATIONS;
use noyau::donnees::version::{
    Lecture, Migration, a_appliquer, connue, lecture, message_de_la_migration,
};

fn lu(nom: &str, lisible: bool, absorbe: bool) -> Lu {
    Lu {
        nom: nom.to_string(),
        lisible,
        absorbe,
    }
}

/// Quatre fichiers lus : a et d lisibles et absorbés, b absorbé mais illisible, c lisible mais
/// non absorbé.
fn quatre_lus() -> Vec<Lu> {
    vec![
        lu("a", true, true),
        lu("b", false, true),
        lu("c", true, false),
        lu("d", true, true),
    ]
}

// Des identifiants inventés : un UUID de propriétaire, des ULID de document et de lignée.
const PROPRIETAIRE: &str = "6f1c2b8e-3d4a-4e5f-9a7b-0c1d2e3f4a5b";
const AUTRE_PROPRIETAIRE: &str = "0a9b8c7d-6e5f-4a3b-8c1d-2e3f4a5b6c7d";
const DOCUMENT: &str = "01JD8Z3K5M7N9P2Q4R6S8T0V1W";
const AUTRE_DOCUMENT: &str = "01JD8Z3K5M7N9P2Q4R6S8T0V2X";
const LIGNEE: &str = "01JD8Z3K6A1B2C3D4E5F6G7H8J";
const AUTRE_LIGNEE: &str = "01JD8Z3K6A1B2C3D4E5F6G7H9K";

fn copie(proprietaire: &str, document: &str, lignee: &str) -> Copie {
    Copie {
        proprietaire: proprietaire.to_string(),
        document: document.to_string(),
        lignee: lignee.to_string(),
    }
}

#[test]
fn l_acces_dit_ce_que_permet_un_document() {
    assert!(Acces::Entier.ecrit() && Acces::Entier.echange());
    assert!(!Acces::Illisible.ecrit() && !Acces::Illisible.echange());
}

#[test]
fn l_acces_d_un_document_plus_recent() {
    assert!(!Acces::PlusRecent.ecrit() && Acces::PlusRecent.echange());
    assert!(!a_l_ouverture(Acces::PlusRecent, 3, true));
    // § 31 · 2.2 : en lecture seule, pas de compaction
    assert!(!apres_une_ecriture(
        Acces::PlusRecent,
        false,
        1600,
        15150,
        101
    ));
}

#[test]
fn la_version_de_schema_decide_de_la_lecture() {
    assert_eq!(lecture(Some(1), 1), Lecture::Ecrire);
    assert_eq!(lecture(Some(1), 2), Lecture::Migrer { de: 1, vers: 2 });
    assert_eq!(lecture(Some(2), 3), Lecture::Migrer { de: 2, vers: 3 });
    assert_eq!(lecture(Some(2), 1), Lecture::LireSansEcrire);
    assert_eq!(lecture(Some(3), 1), Lecture::LireSansEcrire);
    assert_eq!(lecture(None, 1), Lecture::Illisible);
    assert_eq!(lecture(Some(0), 1), Lecture::Illisible);
    let (m2, m3) = (
        Migration {
            vers: 2,
            octets: b"m2",
            version_de_pupitre: "0.4.0",
        },
        Migration {
            vers: 3,
            octets: b"m3",
            version_de_pupitre: "0.5.0",
        },
    );
    assert_eq!((connue(&[]), connue(&[m2])), (1, 2));
    assert_eq!(a_appliquer(&[m2, m3], 1), [m2, m3]);
    assert_eq!(a_appliquer(&[m2, m3], 2), [m3]);
    assert!(a_appliquer(&[m2, m3], 3).is_empty());
    assert_eq!(
        message_de_la_migration(2),
        "Fonds passé à la version de schéma 2"
    );
    assert!(
        MIGRATIONS
            .iter()
            .enumerate()
            .all(|(i, m)| m.vers == i as u64 + 2)
    );
}

#[test]
fn compacter_a_l_ouverture() {
    assert!(
        !a_l_ouverture(Acces::Entier, 0, false),
        "ni ajout ni voisin : rien à fondre"
    );
    assert!(a_l_ouverture(Acces::Entier, 1, false), "un fichier d'ajout");
    assert!(
        a_l_ouverture(Acces::Entier, 0, true),
        "le voisin lisible du fichier complet"
    );
    assert!(
        !a_l_ouverture(Acces::Illisible, 3, false),
        "illisible, des fichiers d'ajout : pas de compaction"
    );
    assert!(
        !a_l_ouverture(Acces::Illisible, 0, true),
        "illisible, le voisin lisible du fichier complet : pas de compaction"
    );
}

#[test]
fn compacter_apres_une_ecriture() {
    assert!(
        !apres_une_ecriture(Acces::Entier, false, 100, 100, 1),
        "des ajouts aussi lourds que le fichier complet : l'égalité ne suffit pas"
    );
    assert!(
        apres_une_ecriture(Acces::Entier, false, 100, 101, 1),
        "des ajouts plus lourds que le fichier complet"
    );
    assert!(
        !apres_une_ecriture(Acces::Entier, false, 10_000, 10, 100),
        "cent ajouts ne passent pas la centaine"
    );
    assert!(
        apres_une_ecriture(Acces::Entier, false, 10_000, 10, 101),
        "cent un ajouts passent la centaine"
    );
    assert!(
        apres_une_ecriture(Acces::Entier, true, 10_000, 10, 1),
        "une compaction due, sans rien d'autre qui la demande"
    );
    assert!(
        !apres_une_ecriture(Acces::Illisible, true, 100, 101, 101),
        "illisible, tout la demande : pas de compaction"
    );
}

#[test]
fn a_retirer_garde_un_fichier_illisible() {
    // [a lisible absorbé, b illisible absorbé, c lisible non absorbé, d lisible absorbé]
    assert_eq!(a_retirer(&quatre_lus()), ["a", "d"]);
}

#[test]
fn comparer_rend_la_premiere_difference() {
    let locale = copie(PROPRIETAIRE, DOCUMENT, LIGNEE);

    assert_eq!(
        comparer(&locale, &copie(PROPRIETAIRE, DOCUMENT, LIGNEE)),
        Ok(())
    );

    // Chaque champ seul.
    assert_eq!(
        comparer(&locale, &copie(AUTRE_PROPRIETAIRE, DOCUMENT, LIGNEE)),
        Err(Difference::AutreProprietaire)
    );
    assert_eq!(
        comparer(&locale, &copie(PROPRIETAIRE, AUTRE_DOCUMENT, LIGNEE)),
        Err(Difference::AutreDocument)
    );
    assert_eq!(
        comparer(&locale, &copie(PROPRIETAIRE, DOCUMENT, AUTRE_LIGNEE)),
        Err(Difference::AutreLignee)
    );

    // Deux champs : la première différence, dans l'ordre propriétaire, document, lignée.
    assert_eq!(
        comparer(&locale, &copie(AUTRE_PROPRIETAIRE, AUTRE_DOCUMENT, LIGNEE)),
        Err(Difference::AutreProprietaire)
    );
    assert_eq!(
        comparer(&locale, &copie(PROPRIETAIRE, AUTRE_DOCUMENT, AUTRE_LIGNEE)),
        Err(Difference::AutreDocument)
    );
}
