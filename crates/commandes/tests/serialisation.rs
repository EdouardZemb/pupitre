//! Les types des commandes, tels que l'interface et l'établi les échangent en JSON : chacun fait
//! l'aller-retour, les noms des commandes sont uniques, et chaque entier d'une réponse s'écrit
//! en nombre entier que `JSON.parse` relit à l'identique.

use std::collections::BTreeSet;
use std::fmt::Debug;

use commandes::{
    Appareil, Assiette, BOITE, Boite, Captation, Completer, Contenu, ENTIER_SUR, ETAT_DU_FONDS,
    EntreeDuJournal, EtatDuFonds, Etiqueter, FENETRE_MAX, Faite, Fenetre, LigneDeLaBoite, NOMS,
    Notee, Noter, Ordre, Provenance, RETIREES, Raison, Refus, Regrouper, Relier, Reponse, Sorte,
    Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees,
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

fn ligne(le: i64, duree_ms: Option<u64>, retiree: Option<i64>) -> LigneDeLaBoite {
    LigneDeLaBoite {
        idee: IDEE.to_string(),
        le,
        texte: Some("Un refrain qui monte, en ré".to_string()),
        etiquettes: vec!["refrain".to_string(), "à finir".to_string()],
        duree_ms,
        reliee: true,
        provenance: Provenance::Importee {
            par: Appareil::Etabli,
            depuis: IMPORT.to_string(),
        },
        retiree,
    }
}

/// Une assiette dont chaque compte vaut `n`.
fn assiette(n: u64) -> Assiette {
    Assiette {
        considerees: n,
        portant: n,
        decidees: n,
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
    for ordre in [
        Ordre::Date,
        Ordre::Etiquette,
        Ordre::Duree,
        Ordre::Reliee,
        Ordre::Provenance,
    ] {
        aller_retour(ordre);
    }
    for contenu in [
        Contenu::TexteDeLIdee,
        Contenu::Prise,
        Contenu::Photo,
        Contenu::MidiVerse,
        Contenu::TexteVerse,
    ] {
        aller_retour(contenu);
    }
    aller_retour(Boite {
        ordre: Ordre::Duree,
        filtre: Some(Contenu::Prise),
        debut: 40,
        nombre: 20,
    });
    aller_retour(Boite {
        ordre: Ordre::Date,
        filtre: None,
        debut: 0,
        nombre: FENETRE_MAX,
    });
    aller_retour(Fenetre {
        debut: 0,
        nombre: FENETRE_MAX,
    });
    aller_retour(Assiette {
        considerees: 3,
        portant: 2,
        decidees: 2,
    });
    aller_retour(ligne(HEURE, Some(42_000), None));
    aller_retour(LigneDeLaBoite {
        texte: None,
        etiquettes: Vec::new(),
        reliee: false,
        provenance: Provenance::Captee {
            par: Appareil::Carnet,
        },
        ..ligne(HEURE, None, Some(HEURE + 3_600))
    });
    aller_retour(VueDeLaBoite {
        non_reliees: 1,
        assiette_du_compte: assiette(2),
        retenues: 2,
        assiette_du_filtre: assiette(2),
        lignes: vec![ligne(HEURE, Some(42_000), None)],
    });
    aller_retour(VueDesRetirees {
        total: 1,
        lignes: vec![ligne(HEURE, None, Some(HEURE + 3_600))],
    });
    aller_retour(EtatDuFonds {
        ne: false,
        lecture_seule: None,
        avertissements: Vec::new(),
        a_annuler: None,
    });
    aller_retour(EtatDuFonds {
        ne: true,
        lecture_seule: Some(refus()),
        avertissements: vec!["Un avertissement inventé.".to_string()],
        a_annuler: Some("Idée du 14 novembre : étiquette #pont posée".to_string()),
    });
}

/// CAP-01 · 4, une part : noter, à l'établi, ne demande que le texte et le décalage local, ni
/// titre, ni étiquette. Les clés se comparent triées : l'ordre des clés d'un objet de
/// `serde_json` dépend de ses fonctions (`preserve_order`), que l'espace de travail peut unifier.
#[test]
fn cap01_4_noter_ne_demande_aucun_titre() {
    let valeur = serde_json::to_value(Noter {
        texte: "Un refrain qui monte, en ré".to_string(),
        decalage_minutes: 60,
    })
    .expect("la note se sérialise");
    let mut cles: Vec<&str> = valeur
        .as_object()
        .expect("une note est un objet")
        .keys()
        .map(String::as_str)
        .collect();
    cles.sort_unstable();
    assert_eq!(cles, ["decalage_minutes", "texte"]);
}

#[test]
fn les_noms_sont_uniques_et_en_snake_case() {
    assert_eq!(NOMS.len(), 17);
    assert_eq!(
        [BOITE, RETIREES, ETAT_DU_FONDS],
        ["boite", "retirees", "etat_du_fonds"]
    );
    for nom in [BOITE, RETIREES, ETAT_DU_FONDS] {
        assert!(NOMS.contains(&nom), "« {nom} » manque à {NOMS:?}");
    }
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

    let la_ligne = ligne(
        PREMIER_JANVIER_2100,
        Some(86_400_000),
        Some(PREMIER_JANVIER_2100),
    );
    let json = en_json(&la_ligne);
    assert_eq!(entier_sur(&json["le"]), 4_102_444_800);
    assert_eq!(entier_sur(&json["duree_ms"]), 86_400_000);
    assert_eq!(entier_sur(&json["retiree"]), 4_102_444_800);

    // Chaque compte d'idées à `FENETRE_MAX` : ce n'est pas sa borne, que donne le nombre des
    // idées du fonds, mais un entier de la même sorte, écrit de même.
    let compte = u64::from(FENETRE_MAX);
    let json = en_json(&VueDeLaBoite {
        non_reliees: compte,
        assiette_du_compte: assiette(compte),
        retenues: compte,
        assiette_du_filtre: assiette(compte),
        lignes: vec![la_ligne.clone()],
    });
    assert_eq!(entier_sur(&json["non_reliees"]), 200);
    assert_eq!(entier_sur(&json["retenues"]), 200);
    for cle in ["assiette_du_compte", "assiette_du_filtre"] {
        for champ in ["considerees", "portant", "decidees"] {
            assert_eq!(entier_sur(&json[cle][champ]), 200, "{cle}.{champ}");
        }
    }
    assert_eq!(entier_sur(&json["lignes"][0]["le"]), 4_102_444_800);
    assert_eq!(entier_sur(&json["lignes"][0]["duree_ms"]), 86_400_000);
    assert_eq!(entier_sur(&json["lignes"][0]["retiree"]), 4_102_444_800);

    let json = en_json(&VueDesRetirees {
        total: compte,
        lignes: vec![la_ligne],
    });
    assert_eq!(entier_sur(&json["total"]), 200);
    assert_eq!(entier_sur(&json["lignes"][0]["le"]), 4_102_444_800);
    assert_eq!(entier_sur(&json["lignes"][0]["duree_ms"]), 86_400_000);
    assert_eq!(entier_sur(&json["lignes"][0]["retiree"]), 4_102_444_800);
}
