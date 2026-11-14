//! Les types des commandes, tels que l'interface et l'établi les échangent en JSON : chacun fait
//! l'aller-retour, les noms des commandes sont uniques, et chaque entier d'une réponse s'écrit
//! en nombre entier que `JSON.parse` relit à l'identique.

use std::collections::BTreeSet;
use std::fmt::Debug;

use commandes::{
    Appareil, Captation, Completer, ENTIER_SUR, EntreeDuJournal, Etiqueter, Faite, NOMS, Notee,
    Noter, Provenance, Raison, Refus, Regrouper, Relier, Reponse, Sorte, Viser, VueDUneIdee,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

const IDEE: &str = "01HF7YAT000000000000000001";
const AUTRE_IDEE: &str = "01HF7YAT000000000000000002";
const IMPORT: &str = "01HF7YAT000000000000000003";
const EMPREINTE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const AUTRE_EMPREINTE: &str = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023, 22 h 13 min 20 s UTC
const PREMIER_JANVIER_2100: i64 = 4_102_444_800; // le 1er janvier 2100, 0 h UTC

/// Sérialise `valeur` en JSON, la relit, et compare.
fn aller_retour<T>(valeur: T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let json = serde_json::to_string(&valeur).expect("la valeur se sérialise");
    let relue: T = serde_json::from_str(&json).expect("le JSON se relit");
    assert_eq!(relue, valeur, "relue de {json}");
}

fn vue(le: i64, duree_ms: Option<u64>, retiree: Option<i64>) -> VueDUneIdee {
    VueDUneIdee {
        idee: IDEE.to_string(),
        provenance: Provenance::Captee {
            par: Appareil::Etabli,
        },
        le,
        texte: Some("Un refrain qui monte, en ré".to_string()),
        etiquettes: vec!["refrain".to_string(), "à finir".to_string()],
        liens: vec![AUTRE_IDEE.to_string()],
        captations: vec![
            Captation {
                sorte: Sorte::Prise,
                empreinte: EMPREINTE.to_string(),
            },
            Captation {
                sorte: Sorte::TexteVerse,
                empreinte: AUTRE_EMPREINTE.to_string(),
            },
        ],
        reliee: true,
        duree_ms,
        retiree,
    }
}

fn refus() -> Refus {
    Refus {
        raison: Raison::VersionPlusRecente { version: 2 },
        message: "Ce fonds vient d'une version plus récente de l'application.".to_string(),
    }
}

#[test]
fn chaque_type_fait_l_aller_retour_par_serde_json() {
    aller_retour(Noter {
        texte: "Un refrain qui monte, en ré".to_string(),
        decalage_minutes: 60,
    });
    aller_retour(Completer {
        idee: IDEE.to_string(),
        texte: "Le pont descend d'un ton".to_string(),
        decalage_minutes: -300,
    });
    aller_retour(Etiqueter {
        idee: IDEE.to_string(),
        etiquette: "à finir".to_string(),
        decalage_minutes: 0,
    });
    aller_retour(Relier {
        a: IDEE.to_string(),
        b: AUTRE_IDEE.to_string(),
        decalage_minutes: 60,
    });
    for sorte in [Sorte::Prise, Sorte::MidiVerseSeul, Sorte::TexteVerse] {
        aller_retour(sorte);
    }
    aller_retour(Captation {
        sorte: Sorte::Prise,
        empreinte: EMPREINTE.to_string(),
    });
    aller_retour(Regrouper {
        captation: Captation {
            sorte: Sorte::MidiVerseSeul,
            empreinte: AUTRE_EMPREINTE.to_string(),
        },
        cible: AUTRE_IDEE.to_string(),
        decalage_minutes: 120,
    });
    aller_retour(Viser {
        idee: IDEE.to_string(),
        decalage_minutes: 60,
    });
    for appareil in [Appareil::Etabli, Appareil::Carnet] {
        aller_retour(appareil);
    }
    aller_retour(Provenance::Captee {
        par: Appareil::Carnet,
    });
    aller_retour(Provenance::Importee {
        par: Appareil::Etabli,
        depuis: IMPORT.to_string(),
    });
    aller_retour(Notee {
        idee: IDEE.to_string(),
        avertissement: None,
    });
    aller_retour(Notee {
        idee: IDEE.to_string(),
        avertissement: Some("Un avertissement inventé.".to_string()),
    });
    aller_retour(Faite {
        avertissement: None,
    });
    aller_retour(Faite {
        avertissement: Some("Un autre avertissement inventé.".to_string()),
    });
    for raison in [
        Raison::Impossible,
        Raison::RienAAnnuler,
        Raison::Illisible,
        Raison::VersionPlusRecente { version: 2 },
        Raison::Disque,
    ] {
        aller_retour(raison);
    }
    aller_retour(refus());
    aller_retour::<Reponse<Notee>>(Ok(Notee {
        idee: IDEE.to_string(),
        avertissement: None,
    }));
    aller_retour::<Reponse<Faite>>(Err(refus()));
    aller_retour(vue(HEURE, Some(42_000), None));
    aller_retour(VueDUneIdee {
        provenance: Provenance::Importee {
            par: Appareil::Carnet,
            depuis: IMPORT.to_string(),
        },
        texte: None,
        etiquettes: Vec::new(),
        liens: Vec::new(),
        captations: Vec::new(),
        reliee: false,
        ..vue(HEURE, None, Some(HEURE + 3_600))
    });
    aller_retour(EntreeDuJournal {
        message: "Idée du 14 novembre notée".to_string(),
        heure: HEURE,
    });
}

#[test]
fn les_noms_sont_uniques_et_en_snake_case() {
    assert_eq!(NOMS.len(), 13);
    let uniques: BTreeSet<&str> = NOMS.iter().copied().collect();
    assert_eq!(uniques.len(), NOMS.len(), "un nom en double dans {NOMS:?}");
    for nom in NOMS {
        assert!(
            !nom.is_empty() && nom.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "« {nom} » n'est pas en snake_case"
        );
    }
}

/// L'entier que porte `valeur`, un nombre JSON écrit sans `.` ni exposant, et sous 2⁵³ en valeur
/// absolue.
fn entier_sur(valeur: &Value) -> i128 {
    let texte = valeur.to_string();
    assert!(valeur.is_number(), "{texte} n'est pas un nombre");
    assert!(
        !texte.contains(['.', 'e', 'E']),
        "{texte} n'est pas écrit en entier"
    );
    let entier: i128 = texte.parse().expect("un entier écrit en chiffres");
    assert!(
        entier.unsigned_abs() <= u128::from(ENTIER_SUR),
        "{texte} dépasse ENTIER_SUR"
    );
    entier
}

/// La sérialisation de `valeur`, relue en arbre JSON.
fn en_json<T: Serialize>(valeur: &T) -> Value {
    let texte = serde_json::to_string(valeur).expect("la valeur se sérialise");
    serde_json::from_str(&texte).expect("le JSON se relit")
}

#[test]
fn les_entiers_d_une_reponse_restent_sous_2_puissance_53() {
    assert_eq!(ENTIER_SUR, (1u64 << 53) - 1);

    let json = en_json(&vue(
        PREMIER_JANVIER_2100,
        Some(86_400_000),
        Some(PREMIER_JANVIER_2100),
    ));
    assert_eq!(entier_sur(&json["le"]), 4_102_444_800);
    assert_eq!(entier_sur(&json["duree_ms"]), 86_400_000);
    assert_eq!(entier_sur(&json["retiree"]), 4_102_444_800);

    let json = en_json(&EntreeDuJournal {
        message: "Idée du 1er janvier notée".to_string(),
        heure: PREMIER_JANVIER_2100,
    });
    assert_eq!(entier_sur(&json["heure"]), 4_102_444_800);
}
