//! Les règles des données : quand compacter, quels fichiers retirer (§ 31 · 2.2), et quelles
//! copies s'échangent (§ 31 · 2.5).

use noyau::donnees::compaction::{Lu, a_l_ouverture, a_retirer, apres_une_ecriture};
use noyau::donnees::copie::{Copie, Difference, comparer};

fn lu(nom: &str, conforme: bool, absorbe: bool) -> Lu {
    Lu {
        nom: nom.to_string(),
        conforme,
        absorbe,
    }
}

/// Quatre fichiers lus : a et d conformes et absorbés, b absorbé mais non conforme à son
/// empreinte, c conforme mais non absorbé.
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
fn compacter_a_l_ouverture() {
    assert!(
        !a_l_ouverture(0, false),
        "ni ajout ni voisin : rien à fondre"
    );
    assert!(a_l_ouverture(1, false), "un fichier d'ajout");
    assert!(
        a_l_ouverture(0, true),
        "le voisin entier du fichier complet"
    );
}

#[test]
fn compacter_apres_une_ecriture() {
    assert!(
        !apres_une_ecriture(100, 100, 1),
        "des ajouts aussi lourds que le fichier complet : l'égalité ne suffit pas"
    );
    assert!(
        apres_une_ecriture(100, 101, 1),
        "des ajouts plus lourds que le fichier complet"
    );
    assert!(
        !apres_une_ecriture(10_000, 10, 100),
        "cent ajouts ne passent pas la centaine"
    );
    assert!(
        apres_une_ecriture(10_000, 10, 101),
        "cent un ajouts passent la centaine"
    );
}

#[test]
fn a_retirer_ne_retire_rien_si_le_complet_relu_mene_ailleurs() {
    assert_eq!(a_retirer(false, &quatre_lus()), Vec::<String>::new());
}

#[test]
fn a_retirer_garde_un_fichier_non_conforme() {
    assert_eq!(a_retirer(true, &quatre_lus()), ["a", "d"]);
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
