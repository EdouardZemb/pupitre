//! Les types des commandes, tels que l'interface et l'établi les échangent en JSON : chacun fait
//! l'aller-retour, les noms des commandes sont uniques, et chaque entier d'une réponse s'écrit
//! en nombre entier que `JSON.parse` relit à l'identique.

use std::collections::BTreeSet;
use std::fmt::Debug;

use commandes::{
    Appareil, Assiette, BOITE, Boite, Bornes, CHOISIR_LE_TYPE_D_UN_STEM, CLORE_L_IMPORT, Captation,
    CaptationVue, ChoisirLeType, ChoixDuType, CloreLImport, Completer, Compte, Contenu,
    DECLARER_UNE_ORIGINE, Declaration, DeclarationDUnDossier, DeclarerUneOrigine, DossierInspecte,
    ENREGISTRER_UNE_REQUETE, ENTIER_SUR, ETAT_DU_FONDS, EXPORTER_CAHIER, Enregistrer,
    EntreeDuJournal, Etat, EtatDuFonds, Etiqueter, FENETRE_MAX, Faite, FaitsDUneCaptation, Fenetre,
    FichierInspecte, FichierVerse, GENERATEURS, INSPECTER_UN_DOSSIER, ImportClos, ImportOuvert,
    Inspection, Issue, JeuAVerser, Jour, LIRE_UN_RAPPORT, LIRE_UN_TEXTE_VERSE, LigneDeLaBoite,
    LireUnRapport, MidiVu, NOMS, NatureDuFichier, Notee, Noter, OUVRIR_L_IMPORT, Ordre,
    OrigineADeclarer, OrigineDeclaree, OuvrirLImport, Portee, Provenance, ProvenanceDeLIdee,
    RENOMMER_UNE_REQUETE, REQUETES_ENREGISTREES, RETIREES, Raison, RapportDImport, Refus,
    Regrouper, Relier, Renommer, Reponse, Requete, RequeteEnregistree, SUPPRIMER_UNE_REQUETE, Sans,
    Sorte, StemVu, Supprimer, TexteLu, TypeDeStem, TypeVu, VERSER_UN_FICHIER,
    VERSER_UNE_PRISE_EN_STEMS, VerserUnFichier, VerserUnePriseEnStems, Viser, VueDUneIdee,
    VueDeLaBoite, VueDesRetirees, VueDuJournal,
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

/// Une assiette dont chaque compte vaut `n`, ses illisibles comprises.
fn assiette(n: u64) -> Assiette {
    Assiette {
        considerees: n,
        portant: n,
        decidees: n,
        illisibles: n,
    }
}

/// Une requête dont chaque critère est posé.
fn requete_entiere() -> Requete {
    Requete {
        texte: "Église du port".to_string(),
        du: Some(Jour {
            annee: 2026,
            mois: 10,
            jour: 1,
        }),
        au: Some(Jour {
            annee: 2026,
            mois: 10,
            jour: 31,
        }),
        etiquette: Some("à finir".to_string()),
        plus_de_secondes: Some(60),
        etat: Some(Etat::NonReliee),
        provenance: Some(ProvenanceDeLIdee::Importee),
        contenu: Some(Contenu::Prise),
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
        decalage_minutes: 120,
        decalage_b_minutes: 60,
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
        decalage_source_minutes: -300,
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
    aller_retour(VueDuJournal {
        total: 3,
        empreinte: EMPREINTE.to_string(),
        lignes: vec![
            EntreeDuJournal {
                message: "Idée du 14 novembre notée".to_string(),
                heure: HEURE,
            },
            EntreeDuJournal {
                message: "Genèse du fonds".to_string(),
                heure: 0,
            },
        ],
    });
    aller_retour(VueDuJournal {
        total: 0,
        empreinte: AUTRE_EMPREINTE.to_string(),
        lignes: Vec::new(),
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
    aller_retour(Jour {
        annee: 2026,
        mois: 2,
        jour: 28,
    });
    let bornes = Bornes {
        des: Some(HEURE),
        avant: Some(HEURE + 86_400),
    };
    aller_retour(bornes.clone());
    aller_retour(Bornes::default());
    for etat in [Etat::Reliee, Etat::NonReliee] {
        aller_retour(etat);
    }
    for provenance in [ProvenanceDeLIdee::Captee, ProvenanceDeLIdee::Importee] {
        aller_retour(provenance);
    }
    aller_retour(requete_entiere());
    aller_retour(Requete::default());
    aller_retour(Sans {
        etiquette: Some(1),
        duree: Some(0),
        texte: Some(2),
    });
    aller_retour(Sans::default());
    aller_retour(Boite {
        ordre: Ordre::Duree,
        requete: requete_entiere(),
        bornes,
        debut: 40,
        nombre: 20,
    });
    aller_retour(Boite {
        ordre: Ordre::Date,
        requete: Requete::default(),
        bornes: Bornes::default(),
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
        illisibles: 1,
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
        assiette_de_la_requete: assiette(2),
        sans: Sans {
            etiquette: None,
            duree: Some(1),
            texte: Some(0),
        },
        etiquettes_du_fonds: vec!["refrain".to_string(), "à finir".to_string()],
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
    aller_retour(RequeteEnregistree {
        id: IDEE.to_string(),
        nom: "Ponts lents".to_string(),
        requete: Some(requete_entiere()),
    });
    aller_retour(RequeteEnregistree {
        id: AUTRE_IDEE.to_string(),
        nom: "Église".to_string(),
        requete: None,
    });
    aller_retour(Enregistrer {
        nom: "Ponts lents".to_string(),
        requete: requete_entiere(),
    });
    aller_retour(Renommer {
        requete: IDEE.to_string(),
        nom: "Ponts du soir".to_string(),
    });
    aller_retour(Supprimer {
        requete: IDEE.to_string(),
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
    assert_eq!(NOMS.len(), 31);
    assert_eq!(
        [BOITE, RETIREES, ETAT_DU_FONDS],
        ["boite", "retirees", "etat_du_fonds"]
    );
    for nom in [BOITE, RETIREES, ETAT_DU_FONDS] {
        assert!(NOMS.contains(&nom), "« {nom} » manque à {NOMS:?}");
    }
    // Les quatre commandes des requêtes enregistrées, après `ETAT_DU_FONDS`, avant
    // `EXPORTER_CAHIER`, dans cet ordre.
    assert_eq!(
        [
            REQUETES_ENREGISTREES,
            ENREGISTRER_UNE_REQUETE,
            RENOMMER_UNE_REQUETE,
            SUPPRIMER_UNE_REQUETE,
        ],
        [
            "requetes_enregistrees",
            "enregistrer_une_requete",
            "renommer_une_requete",
            "supprimer_une_requete",
        ]
    );
    assert_eq!(
        NOMS[15..],
        [
            ETAT_DU_FONDS,
            REQUETES_ENREGISTREES,
            ENREGISTRER_UNE_REQUETE,
            RENOMMER_UNE_REQUETE,
            SUPPRIMER_UNE_REQUETE,
            INSPECTER_UN_DOSSIER,
            OUVRIR_L_IMPORT,
            VERSER_UN_FICHIER,
            VERSER_UNE_PRISE_EN_STEMS,
            CLORE_L_IMPORT,
            DECLARER_UNE_ORIGINE,
            CHOISIR_LE_TYPE_D_UN_STEM,
            LIRE_UN_TEXTE_VERSE,
            LIRE_UN_RAPPORT,
            GENERATEURS,
            EXPORTER_CAHIER,
        ]
    );
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

    // Le nombre d'entrées du journal à `FENETRE_MAX` : ce n'est pas sa borne, que donne le nombre
    // des modifications du document, mais un entier de la même sorte, écrit de même.
    let json = en_json(&VueDuJournal {
        total: u64::from(FENETRE_MAX),
        empreinte: EMPREINTE.to_string(),
        lignes: vec![EntreeDuJournal {
            message: "Idée du 1er janvier notée".to_string(),
            heure: PREMIER_JANVIER_2100,
        }],
    });
    assert_eq!(entier_sur(&json["total"]), 200);
    assert_eq!(entier_sur(&json["lignes"][0]["heure"]), 4_102_444_800);

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
        assiette_de_la_requete: assiette(compte),
        sans: Sans {
            etiquette: Some(compte),
            duree: Some(compte),
            texte: Some(compte),
        },
        etiquettes_du_fonds: vec!["refrain".to_string()],
        lignes: vec![la_ligne.clone()],
    });
    assert_eq!(entier_sur(&json["non_reliees"]), 200);
    assert_eq!(entier_sur(&json["retenues"]), 200);
    for cle in ["assiette_du_compte", "assiette_de_la_requete"] {
        for champ in ["considerees", "portant", "decidees", "illisibles"] {
            assert_eq!(entier_sur(&json[cle][champ]), 200, "{cle}.{champ}");
        }
    }
    for part in ["etiquette", "duree", "texte"] {
        assert_eq!(entier_sur(&json["sans"][part]), 200, "sans.{part}");
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

/// Des faits dont chaque champ est posé.
fn faits_complets() -> FaitsDUneCaptation {
    FaitsDUneCaptation {
        format: Some("wav".to_string()),
        duree_ms: Some(215_000),
        codage: Some(1),
        canaux: Some(2),
        frequence: Some(44_100),
        bits: Some(24),
        division: Some(480),
    }
}

/// Un compte dont chaque champ vaut `n`.
fn compte_de(n: u64) -> Compte {
    Compte {
        fichiers: n,
        octets: n,
    }
}

/// Un rapport dont chaque compte vaut `n`.
fn rapport_de(n: u64) -> RapportDImport {
    RapportDImport {
        import: IMPORT.to_string(),
        vus: compte_de(n),
        verses: compte_de(n),
        dedoublonnes: compte_de(n),
        illisibles: compte_de(n),
        ignores: compte_de(n),
        sans_nom: compte_de(n),
        origines_changees: n,
        fichiers_sans_nom: vec![n],
    }
}

#[test]
fn les_types_de_l_import_font_l_aller_retour() {
    let declaration = Declaration {
        stems: true,
        origine: OrigineADeclarer::Generee {
            nom: "Atelier inventé".to_string(),
        },
    };
    aller_retour(declaration.clone());
    aller_retour(Declaration {
        stems: false,
        origine: OrigineADeclarer::Aucune,
    });
    aller_retour(Declaration {
        stems: false,
        origine: OrigineADeclarer::Main,
    });
    for nature in [
        NatureDuFichier::Audio {
            format: "wav".to_string(),
        },
        NatureDuFichier::Midi,
        NatureDuFichier::Texte {
            format: "txt".to_string(),
        },
        NatureDuFichier::Ignore,
    ] {
        aller_retour(nature);
    }
    aller_retour(FichierInspecte {
        rang: 0,
        chemin: None,
        octets: 12,
        nature: NatureDuFichier::Ignore,
        connu: false,
    });
    aller_retour(FichierInspecte {
        rang: 3,
        chemin: Some("jeu/basse.wav".to_string()),
        octets: 4_096,
        nature: NatureDuFichier::Audio {
            format: "wav".to_string(),
        },
        connu: true,
    });
    aller_retour(DossierInspecte {
        chemin: String::new(),
        audio: 4,
        jeu_propose: true,
        reproposee: Some(declaration.clone()),
    });
    aller_retour(DossierInspecte {
        chemin: "jeu".to_string(),
        audio: 0,
        jeu_propose: false,
        reproposee: None,
    });
    let declaree = DeclarationDUnDossier {
        chemin: "jeu".to_string(),
        declaration: declaration.clone(),
    };
    aller_retour(declaree.clone());
    aller_retour(Inspection {
        reprise: Some(IMPORT.to_string()),
        nom_du_dossier: Some("l'atelier".to_string()),
        fichiers: vec![FichierInspecte {
            rang: 1,
            chemin: Some("a.wav".to_string()),
            octets: 10,
            nature: NatureDuFichier::Midi,
            connu: false,
        }],
        dossiers: vec![DossierInspecte {
            chemin: String::new(),
            audio: 1,
            jeu_propose: false,
            reproposee: None,
        }],
        a_verser: 10,
        libre: 1_000_000,
        manque: Some("La place manque.".to_string()),
        generateurs: vec!["Atelier d'essai".to_string(), "Générateur".to_string()],
        declarations_reprises: vec![declaree.clone()],
        ouvert_ailleurs: true,
    });
    aller_retour(Inspection {
        reprise: None,
        nom_du_dossier: None,
        fichiers: Vec::new(),
        dossiers: Vec::new(),
        a_verser: 0,
        libre: 0,
        manque: None,
        generateurs: Vec::new(),
        declarations_reprises: Vec::new(),
        ouvert_ailleurs: false,
    });
    aller_retour(OuvrirLImport {
        declarations: vec![declaree],
        decalage_minutes: -60,
    });
    aller_retour(ImportOuvert {
        import: IMPORT.to_string(),
        repris: true,
        rangs: vec![0, 2],
        jeux: vec![JeuAVerser {
            dossier: "jeu".to_string(),
            rangs: vec![3, 4],
        }],
        avertissement: Some("Un fichier a changé.".to_string()),
    });
    aller_retour(VerserUnFichier {
        import: IMPORT.to_string(),
        rang: 2,
        decalage_minutes: 120,
    });
    for issue in [
        Issue::Verse,
        Issue::Dedoublonne,
        Issue::Stem,
        Issue::DejaFait,
    ] {
        aller_retour(FichierVerse {
            rang: 2,
            issue,
            avertissement: None,
        });
    }
    aller_retour(FichierVerse {
        rang: 2,
        issue: Issue::Verse,
        avertissement: Some("Un avertissement.".to_string()),
    });
    aller_retour(VerserUnePriseEnStems {
        import: IMPORT.to_string(),
        dossier: "jeu".to_string(),
        decalage_minutes: 0,
    });
    aller_retour(CloreLImport {
        import: IMPORT.to_string(),
        decalage_minutes: 60,
    });
    aller_retour(compte_de(3));
    aller_retour(rapport_de(5));
    aller_retour(ImportClos {
        rapport: rapport_de(1),
        avertissement: Some("Le rapport est resté court.".to_string()),
    });
    aller_retour(DeclarerUneOrigine {
        portee: Portee::Captation(Captation {
            sorte: Sorte::Prise,
            empreinte: EMPREINTE.to_string(),
        }),
        origine: OrigineADeclarer::Main,
    });
    aller_retour(DeclarerUneOrigine {
        portee: Portee::Prefixe {
            import: IMPORT.to_string(),
            prefixe: "jeu".to_string(),
        },
        origine: OrigineADeclarer::Generee {
            nom: "L'atelier".to_string(),
        },
    });
    for type_de_stem in [
        TypeDeStem::Voix,
        TypeDeStem::Choeurs,
        TypeDeStem::Batterie,
        TypeDeStem::Percussions,
        TypeDeStem::Basse,
        TypeDeStem::Guitare,
        TypeDeStem::Clavier,
        TypeDeStem::Cordes,
        TypeDeStem::Cuivres,
        TypeDeStem::Synthe,
        TypeDeStem::Effets,
        TypeDeStem::Autre,
    ] {
        aller_retour(ChoisirLeType {
            prise: EMPREINTE.to_string(),
            stem: AUTRE_EMPREINTE.to_string(),
            choix: ChoixDuType::Choisir(type_de_stem),
        });
    }
    aller_retour(ChoisirLeType {
        prise: EMPREINTE.to_string(),
        stem: AUTRE_EMPREINTE.to_string(),
        choix: ChoixDuType::Garder,
    });
    aller_retour(TexteLu {
        texte: Some("Un couplet, puis un refrain.".to_string()),
    });
    aller_retour(TexteLu { texte: None });
    aller_retour(LireUnRapport {
        import: IMPORT.to_string(),
    });
    for origine in [
        OrigineDeclaree::Aucune,
        OrigineDeclaree::Main,
        OrigineDeclaree::Generee {
            noms: vec!["Atelier d'essai".to_string(), "Un autre".to_string()],
        },
    ] {
        aller_retour(origine);
    }
    aller_retour(faits_complets());
    aller_retour(FaitsDUneCaptation {
        format: None,
        duree_ms: None,
        codage: None,
        canaux: None,
        frequence: None,
        bits: None,
        division: None,
    });
    let midi = MidiVu {
        empreinte: AUTRE_EMPREINTE.to_string(),
        faits: faits_complets(),
        noms: vec!["basse.mid".to_string()],
    };
    aller_retour(midi.clone());
    for type_vu in [
        TypeVu::Propose(TypeDeStem::Choeurs),
        TypeVu::Garde(TypeDeStem::Basse),
        TypeVu::Choisi(TypeDeStem::Autre),
    ] {
        aller_retour(StemVu {
            empreinte: EMPREINTE.to_string(),
            faits: faits_complets(),
            noms: vec!["basse.wav".to_string(), "basse (2).wav".to_string()],
            type_de_stem: type_vu,
            midi: Some(midi.clone()),
        });
    }
    let stem = StemVu {
        empreinte: EMPREINTE.to_string(),
        faits: faits_complets(),
        noms: Vec::new(),
        type_de_stem: TypeVu::Propose(TypeDeStem::Choeurs),
        midi: None,
    };
    aller_retour(stem.clone());
    aller_retour(CaptationVue {
        captation: Captation {
            sorte: Sorte::Prise,
            empreinte: EMPREINTE.to_string(),
        },
        faits: faits_complets(),
        noms: vec!["prise.wav".to_string()],
        origine: OrigineDeclaree::Generee {
            noms: vec!["Atelier inventé".to_string()],
        },
        stems: vec![stem],
    });
    // Les deux champs neufs des vues d'une idée et de la boîte.
    aller_retour(VueDUneIdee {
        captations_vues: vec![CaptationVue {
            captation: Captation {
                sorte: Sorte::Prise,
                empreinte: EMPREINTE.to_string(),
            },
            faits: faits_complets(),
            noms: vec!["prise.wav".to_string()],
            origine: OrigineDeclaree::Main,
            stems: Vec::new(),
        }],
        ..vue(HEURE, None, None)
    });
    aller_retour(LigneDeLaBoite {
        nom_de_fichier: Some("prise.wav".to_string()),
        ..ligne(HEURE, None, None)
    });
}

#[test]
fn les_noms_de_l_import_sont_ceux_du_plan() {
    assert_eq!(
        [
            INSPECTER_UN_DOSSIER,
            OUVRIR_L_IMPORT,
            VERSER_UN_FICHIER,
            VERSER_UNE_PRISE_EN_STEMS,
            CLORE_L_IMPORT,
            DECLARER_UNE_ORIGINE,
            CHOISIR_LE_TYPE_D_UN_STEM,
            LIRE_UN_TEXTE_VERSE,
            LIRE_UN_RAPPORT,
            GENERATEURS,
        ],
        [
            "inspecter_un_dossier",
            "ouvrir_l_import",
            "verser_un_fichier",
            "verser_une_prise_en_stems",
            "clore_l_import",
            "declarer_une_origine",
            "choisir_le_type_d_un_stem",
            "lire_un_texte_verse",
            "lire_un_rapport",
            "generateurs",
        ]
    );
}

#[test]
fn les_entiers_de_l_import_restent_sous_2_puissance_53() {
    let borne = i128::from(ENTIER_SUR);

    let json = en_json(&FichierInspecte {
        rang: ENTIER_SUR,
        chemin: None,
        octets: ENTIER_SUR,
        nature: NatureDuFichier::Ignore,
        connu: false,
    });
    assert_eq!(entier_sur(&json["rang"]), borne);
    assert_eq!(entier_sur(&json["octets"]), borne);

    let json = en_json(&Inspection {
        reprise: None,
        nom_du_dossier: None,
        fichiers: Vec::new(),
        dossiers: vec![DossierInspecte {
            chemin: String::new(),
            audio: ENTIER_SUR,
            jeu_propose: false,
            reproposee: None,
        }],
        a_verser: ENTIER_SUR,
        libre: ENTIER_SUR,
        manque: None,
        generateurs: Vec::new(),
        declarations_reprises: Vec::new(),
        ouvert_ailleurs: false,
    });
    assert_eq!(entier_sur(&json["a_verser"]), borne);
    assert_eq!(entier_sur(&json["libre"]), borne);
    assert_eq!(entier_sur(&json["dossiers"][0]["audio"]), borne);

    let json = en_json(&ImportOuvert {
        import: IMPORT.to_string(),
        repris: false,
        rangs: vec![ENTIER_SUR],
        jeux: vec![JeuAVerser {
            dossier: "jeu".to_string(),
            rangs: vec![ENTIER_SUR],
        }],
        avertissement: None,
    });
    assert_eq!(entier_sur(&json["rangs"][0]), borne);
    assert_eq!(entier_sur(&json["jeux"][0]["rangs"][0]), borne);

    let json = en_json(&FichierVerse {
        rang: ENTIER_SUR,
        issue: Issue::Verse,
        avertissement: None,
    });
    assert_eq!(entier_sur(&json["rang"]), borne);

    let json = en_json(&compte_de(ENTIER_SUR));
    assert_eq!(entier_sur(&json["fichiers"]), borne);
    assert_eq!(entier_sur(&json["octets"]), borne);

    let json = en_json(&ImportClos {
        rapport: rapport_de(ENTIER_SUR),
        avertissement: None,
    });
    for cle in [
        "vus",
        "verses",
        "dedoublonnes",
        "illisibles",
        "ignores",
        "sans_nom",
    ] {
        for champ in ["fichiers", "octets"] {
            assert_eq!(
                entier_sur(&json["rapport"][cle][champ]),
                borne,
                "{cle}.{champ}"
            );
        }
    }
    assert_eq!(entier_sur(&json["rapport"]["origines_changees"]), borne);
    assert_eq!(entier_sur(&json["rapport"]["fichiers_sans_nom"][0]), borne);

    let json = en_json(&FaitsDUneCaptation {
        duree_ms: Some(ENTIER_SUR),
        codage: Some(u16::MAX),
        canaux: Some(u16::MAX),
        frequence: Some(u32::MAX),
        bits: Some(u16::MAX),
        division: Some(u16::MAX),
        ..faits_complets()
    });
    assert_eq!(entier_sur(&json["duree_ms"]), borne);
    assert_eq!(entier_sur(&json["codage"]), i128::from(u16::MAX));
    assert_eq!(entier_sur(&json["canaux"]), i128::from(u16::MAX));
    assert_eq!(entier_sur(&json["frequence"]), i128::from(u32::MAX));
    assert_eq!(entier_sur(&json["bits"]), i128::from(u16::MAX));
    assert_eq!(entier_sur(&json["division"]), i128::from(u16::MAX));
}
