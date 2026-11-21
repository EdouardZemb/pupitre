//! Les tests des opérations nommées de l'import (§ 30 · 9.2 ; § 31 · 2 ; questions 18 à 22, 25,
//! 26 et 31) : ouvrir, verser un fichier, verser une prise en stems, clore, déclarer une origine
//! après l'import, choisir le type d'un stem, et le nom d'une copie de l'import.

use std::collections::{BTreeMap, BTreeSet};

use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::entete::{FaitsMidi, FaitsWav};
use noyau::donnees::gestes::{Appareil, Impossible, Sorte};
use noyau::donnees::idee::empreintes_nommees;
use noyau::donnees::import::{
    ChoixDuType, Cible, DejaDeLaPrise, DejaLu, Faits, ImportOuvert, MidiDuStem, StemAVerser,
    choisir_le_type, clore, copie_de, declarer_une_origine, nom_de_la_copie, ouvrir,
    verser_un_fichier, verser_une_prise_en_stems,
};
use noyau::donnees::origine::{Ajout, Declaration, OriginesLues};
use noyau::donnees::signature::{FormatSonore, FormatTexte};
use noyau::donnees::stems::TypeDeStem;

/// L'ULID de l'import.
const I: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ";
const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023
// L'idée qui naît, un ULID de test ; les empreintes, des chaînes de 64 caractères hexadécimaux
// écrites en dur, distinctes : le noyau ne vérifie pas qu'une empreinte est celle d'un contenu,
// et `empreinte(b"…")` n'est pas une constante (balayage P15).
const IDEE: &str = "01HF7YAT000000000000000002";
/// Trois fichiers.
const E1: &str = "e100000000000000000000000000000000000000000000000000000000000001";
const E2: &str = "e200000000000000000000000000000000000000000000000000000000000002";
const E3: &str = "e300000000000000000000000000000000000000000000000000000000000003";
/// Le manifeste d'une prise en stems.
const P: &str = "fa00000000000000000000000000000000000000000000000000000000000001";
/// Les stems voix et basse.
const S1: &str = "5a00000000000000000000000000000000000000000000000000000000000001";
const S2: &str = "5a00000000000000000000000000000000000000000000000000000000000002";
/// Le MIDI de la voix.
const M1: &str = "3d00000000000000000000000000000000000000000000000000000000000001";
/// Le rapport.
const R: &str = "ab00000000000000000000000000000000000000000000000000000000000001";

fn ouvert() -> ImportOuvert<'static> {
    ImportOuvert {
        import: I,
        ouvert_le: HEURE,
        par: Appareil::Etabli,
    }
}

// Les faits des stems d'essai, en `static` : `stems_d_essai()` rend des `StemAVerser<'static>`,
// qui empruntent leurs `Faits` (balayage P15).
static FAITS_D_UN_STEM: std::sync::LazyLock<Faits> = std::sync::LazyLock::new(|| {
    Faits::du_wav(&FaitsWav {
        codage: 1,
        canaux: 1,
        frequence: 8_000,
        bits: 8,
        trames: 8_000,
    })
});
static FAITS_DU_MIDI: std::sync::LazyLock<Faits> = std::sync::LazyLock::new(|| {
    Faits::du_midi(&FaitsMidi {
        format: 1,
        pistes: 3,
        division: 480,
    })
});

/// `Jeu A/voix.wav` (`S1`, 1 000 ms), avec `Jeu A/voix.mid` (`M1`, `smf1`, 480) ; puis
/// `Jeu A/basse.wav` (`S2`), sans MIDI.
fn stems_d_essai() -> [StemAVerser<'static>; 2] {
    [
        StemAVerser {
            empreinte: S1,
            faits: &FAITS_D_UN_STEM,
            chemin: Some("Jeu A/voix.wav"),
            midi: Some(MidiDuStem {
                empreinte: M1,
                faits: &FAITS_DU_MIDI,
                chemin: Some("Jeu A/voix.mid"),
            }),
        },
        StemAVerser {
            empreinte: S2,
            faits: &FAITS_D_UN_STEM,
            chemin: Some("Jeu A/basse.wav"),
            midi: None,
        },
    ]
}

/// Les changements `Poser` de l'opération, chacun sous son chemin, ses segments joints par `|`.
fn cles_posees(op: &Operation) -> BTreeMap<String, Simple> {
    op.changements
        .iter()
        .filter_map(|changement| match changement {
            Changement::Poser { chemin, valeur } => Some((chemin.join("|"), valeur.clone())),
            _ => None,
        })
        .collect()
}

/// L'heure de naissance de l'idée `IDEE`, si l'opération la fait naître : son objet, avec celui
/// de ses étiquettes, `provenance` à `importee`, `par` à `etabli`, `depuis` à l'import, et `le`.
fn idee_nee(op: &Operation) -> Option<i64> {
    let cree = op.changements.iter().any(|c| {
        matches!(c, Changement::Creer { chemin, conteneurs }
            if chemin.join("|") == format!("idees|{IDEE}") && conteneurs == &["etiquettes"])
    });
    let cles = cles_posees(op);
    let sous_l_idee = |cle: &str| cles.get(&format!("idees|{IDEE}|{cle}")).cloned();
    let nee = cree
        && sous_l_idee("provenance") == Some(Simple::Chaine("importee".into()))
        && sous_l_idee("par") == Some(Simple::Chaine("etabli".into()))
        && sous_l_idee("depuis") == Some(Simple::Chaine(I.into()));
    match sous_l_idee("le") {
        Some(Simple::Entier(le)) if nee => Some(le),
        _ => None,
    }
}

/// La déclaration d'un dossier qui ne porte que son origine.
fn origine(ajout: Ajout) -> Declaration {
    Declaration {
        stems: false,
        origine: Some(ajout),
    }
}

#[test]
fn ouvrir_ecrit_l_heure_et_les_declarations() {
    // Le dossier versé, son origine ; `Jeu A`, jeu de stems ; `Jeu B`, jeu de stems et généré à
    // la fois ; `Vide`, ni l'un ni l'autre, n'écrit rien.
    let declarations = BTreeMap::from([
        (String::new(), origine(Ajout::Main)),
        (
            "Jeu A".into(),
            Declaration {
                stems: true,
                origine: None,
            },
        ),
        (
            "Jeu B".into(),
            Declaration {
                stems: true,
                origine: Some(Ajout::Generee("Atelier".into())),
            },
        ),
        ("Vide".into(), Declaration::default()),
    ]);
    let op = ouvrir(I, &declarations, HEURE, 0).unwrap();
    assert_eq!(op.message, "Import du 14 novembre ouvert");
    assert_eq!(
        cles_posees(&op),
        BTreeMap::from([
            (format!("imports|{I}/ouvert"), Simple::Entier(HEURE)),
            (
                format!("imports|{I}/declaration/"),
                Simple::Chaine("main".into())
            ),
            (format!("imports|{I}/jeu/Jeu A"), Simple::Vrai),
            (
                format!("imports|{I}/declaration/Jeu B"),
                Simple::Chaine("generee:Atelier".into())
            ),
            (format!("imports|{I}/jeu/Jeu B"), Simple::Vrai),
        ])
    );
    let mauvais = BTreeMap::from([(
        String::new(),
        Declaration {
            stems: true,
            origine: Some(Ajout::Generee("a\nb".into())),
        },
    )]);
    assert_eq!(
        ouvrir(I, &mauvais, HEURE, 0).unwrap_err(),
        Impossible::NomDeGenerateurIllisible
    );
}

#[test]
fn verser_un_wav_neuf_fait_naitre_son_idee_et_ecrit_ses_faits() {
    let faits = Faits::du_wav(&FaitsWav {
        codage: 1,
        canaux: 2,
        frequence: 44_100,
        bits: 16,
        trames: 44_100,
    });
    let op = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E1,
        &faits,
        Some("Démos/Démo 1.wav"),
        Some(&Ajout::Main),
        &DejaLu::default(),
        IDEE,
        HEURE + 5,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(op.message, "Idée du 14 novembre : prise versée");
    let cles = cles_posees(&op);
    assert_eq!(
        cles[&format!("prises|{E1}/idee")],
        Simple::Chaine(IDEE.into())
    );
    assert_eq!(idee_nee(&op), Some(HEURE + 5));
    assert_eq!(
        cles[&format!("prises|{E1}/format")],
        Simple::Chaine("wav".into())
    );
    assert_eq!(cles[&format!("prises|{E1}/duree")], Simple::Naturel(1_000));
    for (cle, valeur) in [
        ("codage", 1),
        ("canaux", 2),
        ("frequence", 44_100),
        ("bits", 16),
    ] {
        assert_eq!(
            cles[&format!("prises|{E1}/{cle}")],
            Simple::Naturel(valeur),
            "{cle}"
        );
    }
    assert_eq!(
        cles[&format!("prises|{E1}/nom/Démos/Démo 1.wav")],
        Simple::Chaine(I.into())
    );
    assert_eq!(cles[&format!("origines_declarees|{E1}/main")], Simple::Vrai);
}

#[test]
fn un_illisible_n_a_aucun_fait() {
    let op = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E1,
        &Faits::default(),
        Some("a.wav"),
        None,
        &DejaLu::default(),
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    let cles = cles_posees(&op);
    // Sa captation et son nom sont écrits, et rien d'autre sous elle (balayage P20).
    assert_eq!(
        cles[&format!("prises|{E1}/idee")],
        Simple::Chaine(IDEE.into())
    );
    assert_eq!(
        cles[&format!("prises|{E1}/nom/a.wav")],
        Simple::Chaine(I.into())
    );
    assert!(
        !cles.keys().any(|c| c.starts_with(&format!("prises|{E1}/"))
            && !c.ends_with("/idee")
            && !c.contains("/nom/")),
        "{cles:?}"
    );
}

#[test]
fn un_contenu_connu_ne_prend_que_ses_noms_et_son_origine() {
    let deja = DejaLu {
        captation: true,
        noms: BTreeSet::from(["a.wav".to_string()]),
        origines: OriginesLues::default(),
    };
    assert_eq!(
        verser_un_fichier(
            &ouvert(),
            Sorte::Prise,
            E1,
            &Faits::default(),
            Some("a.wav"),
            None,
            &deja,
            IDEE,
            HEURE,
            0
        )
        .unwrap(),
        None
    );
    let op = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E1,
        &Faits::default(),
        Some("b.wav"),
        None,
        &deja,
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(op.message, "Import du 14 novembre : nom d'origine ajouté");
    assert!(!cles_posees(&op).keys().any(|c| c.starts_with("idees|")));
    let generee = DejaLu {
        origines: OriginesLues {
            main: false,
            generees: BTreeSet::from(["Atelier".into()]),
        },
        ..deja.clone()
    };
    assert_eq!(
        verser_un_fichier(
            &ouvert(),
            Sorte::Prise,
            E1,
            &Faits::default(),
            Some("a.wav"),
            Some(&Ajout::Main),
            &generee,
            IDEE,
            HEURE,
            0
        )
        .unwrap(),
        None
    );
    let op = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E1,
        &Faits::default(),
        Some("c.wav"),
        Some(&Ajout::Generee("Forge".into())),
        &generee,
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        op.message,
        "Import du 14 novembre : nom d'origine ajouté, origine déclarée"
    );
}

#[test]
fn un_midi_et_un_texte_verses_ont_leurs_faits() {
    let midi = Faits::du_midi(&FaitsMidi {
        format: 1,
        pistes: 3,
        division: 480,
    });
    let cles = cles_posees(
        &verser_un_fichier(
            &ouvert(),
            Sorte::MidiVerseSeul,
            E2,
            &midi,
            Some("m.mid"),
            None,
            &DejaLu::default(),
            IDEE,
            HEURE,
            0,
        )
        .unwrap()
        .unwrap(),
    );
    assert_eq!(
        (
            cles[&format!("midi_verses|{E2}/format")].clone(),
            cles[&format!("midi_verses|{E2}/division")].clone()
        ),
        (Simple::Chaine("smf1".into()), Simple::Naturel(480))
    );
    let cles = cles_posees(
        &verser_un_fichier(
            &ouvert(),
            Sorte::TexteVerse,
            E3,
            &Faits::d_un_texte(FormatTexte::Chordpro),
            Some("t.cho"),
            None,
            &DejaLu::default(),
            IDEE,
            HEURE,
            0,
        )
        .unwrap()
        .unwrap(),
    );
    assert_eq!(
        cles[&format!("textes_verses|{E3}/format")],
        Simple::Chaine("chordpro".into())
    );
}

#[test]
fn un_audio_que_l_import_ne_decode_pas_n_a_que_son_format() {
    // Un FLAC : son format seul, sans durée ni fait d'en-tête (ADR-024, décision 6).
    let faits = Faits::d_un_audio(FormatSonore::Flac);
    assert_eq!(
        faits,
        Faits {
            format: Some("flac".into()),
            ..Faits::default()
        }
    );
    let op = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E2,
        &faits,
        Some("b.flac"),
        None,
        &DejaLu::default(),
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    let cles = cles_posees(&op);
    let prefixe = format!("prises|{E2}/");
    let sous_la_captation: Vec<&str> = cles
        .keys()
        .filter_map(|c| c.strip_prefix(prefixe.as_str()))
        .collect();
    assert_eq!(sous_la_captation, ["format", "idee", "nom/b.flac"]);
    assert_eq!(
        cles[&format!("prises|{E2}/format")],
        Simple::Chaine("flac".into())
    );
}

#[test]
fn verser_une_prise_en_stems_ecrit_son_manifeste_ses_stems_et_leurs_midi() {
    // deux stems, le premier accompagné d'un MIDI ; la prise prend le nom de son dossier
    let op = verser_une_prise_en_stems(
        &ouvert(),
        P,
        Some("Jeu A"),
        &stems_d_essai(),
        None,
        &DejaDeLaPrise::default(),
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(op.message, "Idée du 14 novembre : prise en stems versée");
    let cles = cles_posees(&op);
    assert_eq!(
        cles[&format!("prises|{P}/idee")],
        Simple::Chaine(IDEE.into())
    );
    assert_eq!(idee_nee(&op), Some(HEURE));
    assert_eq!(cles[&format!("prises|{P}/manifeste")], Simple::Vrai);
    // L'entrée de chaque stem, avant ses autres clés.
    let rang = |voulu: &dyn Fn(&str) -> bool| {
        op.changements.iter().position(|c| {
            matches!(c, Changement::Poser { chemin, .. }
                if chemin[0] == "prises" && voulu(chemin[1].as_str()))
        })
    };
    for stem in [S1, S2] {
        let entree = format!("{P}/stem/{stem}");
        assert_eq!(cles[&format!("prises|{entree}")], Simple::Vrai);
        let avant = rang(&|cle: &str| cle == entree);
        let apres = rang(&|cle: &str| cle.starts_with(&format!("{entree}/")));
        assert!(avant.is_some() && avant < apres, "{stem}");
    }
    assert_eq!(
        cles[&format!("prises|{P}/nom/Jeu A")],
        Simple::Chaine(I.into())
    );
    assert!(
        !cles.contains_key(&format!("prises|{P}/format"))
            && !cles.contains_key(&format!("prises|{P}/duree"))
    );
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/duree")],
        Simple::Naturel(1_000)
    );
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/midi/{M1}")],
        Simple::Vrai
    );
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/nom/Jeu A/voix.wav")],
        Simple::Chaine(I.into())
    );
    assert_eq!(
        cles[&format!("midi_verses|{M1}/division")],
        Simple::Naturel(480)
    );
    assert!(!cles.contains_key(&format!("midi_verses|{M1}/idee")));
    assert!(
        !cles.keys().any(|c| c.contains("/type")),
        "aucun type décidé à l'import (CAP-03 · 7)"
    );
    assert_eq!(
        verser_une_prise_en_stems(
            &ouvert(),
            P,
            Some("Jeu A"),
            &[],
            None,
            &DejaDeLaPrise::default(),
            IDEE,
            HEURE,
            0
        )
        .unwrap_err(),
        Impossible::JeuSansAudio
    );
}

#[test]
fn l_entree_d_un_stem_sans_fait_ni_nom_ni_origine_s_ecrit() {
    // Un stem illisible, au nom qui n'est pas en UTF-8, sans origine : son entrée le nomme
    // parmi les empreintes du fonds, et sa copie ne se retire pas.
    let rien = Faits::default();
    let muet = [StemAVerser {
        empreinte: S2,
        faits: &rien,
        chemin: None,
        midi: None,
    }];
    let op = verser_une_prise_en_stems(
        &ouvert(),
        P,
        Some("Jeu A"),
        &muet,
        None,
        &DejaDeLaPrise::default(),
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    let cles = cles_posees(&op);
    assert_eq!(cles[&format!("prises|{P}/stem/{S2}")], Simple::Vrai);
    let sous_la_prise = cles.keys().filter_map(|c| c.strip_prefix("prises|"));
    assert!(empreintes_nommees(sous_la_prise, []).contains(S2));
}

#[test]
fn une_prise_en_stems_sans_nom_garde_les_noms_de_ses_stems() {
    // un jeu versé seul dont le dossier n'a pas de nom en UTF-8 (R23)
    let op = verser_une_prise_en_stems(
        &ouvert(),
        P,
        None,
        &stems_d_essai(),
        None,
        &DejaDeLaPrise::default(),
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    let cles = cles_posees(&op);
    assert!(
        !cles
            .keys()
            .any(|c| c.starts_with(&format!("prises|{P}/nom/")))
    );
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/nom/Jeu A/voix.wav")],
        Simple::Chaine(I.into())
    );
    assert_eq!(cles[&format!("prises|{P}/manifeste")], Simple::Vrai);
}

#[test]
fn une_prise_en_stems_connue_ne_prend_que_ses_noms_et_ses_origines() {
    // La prise, la voix et son MIDI déjà dans le fonds, sous leurs noms, le lien de la voix à son
    // MIDI posé ; la basse, sans le sien.
    let deja = DejaDeLaPrise {
        prise: DejaLu {
            captation: true,
            noms: BTreeSet::from(["Jeu A".to_string()]),
            origines: OriginesLues::default(),
        },
        stems: BTreeMap::from([(
            S1.to_string(),
            DejaLu {
                captation: true,
                noms: BTreeSet::from(["Jeu A/voix.wav".to_string()]),
                origines: OriginesLues::default(),
            },
        )]),
        midis: BTreeMap::from([(
            M1.to_string(),
            DejaLu {
                captation: true,
                noms: BTreeSet::from(["Jeu A/voix.mid".to_string()]),
                origines: OriginesLues::default(),
            },
        )]),
        liens_midi: BTreeSet::from([(S1.to_string(), M1.to_string())]),
    };
    let forge = Ajout::Generee("Forge".into());
    let op = verser_une_prise_en_stems(
        &ouvert(),
        P,
        Some("Jeu A"),
        &stems_d_essai(),
        Some(&forge),
        &deja,
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        op.message,
        "Import du 14 novembre : nom d'origine ajouté, origine déclarée"
    );
    // Ni idée, ni manifeste, ni fait, ni lien : le nom de la basse, et l'origine de chaque
    // captation, la prise, ses stems et le MIDI de la voix.
    assert_eq!(
        cles_posees(&op),
        BTreeMap::from([
            (
                format!("prises|{P}/stem/{S2}/nom/Jeu A/basse.wav"),
                Simple::Chaine(I.into())
            ),
            (
                format!("origines_declarees|{P}/generee/Forge"),
                Simple::Vrai
            ),
            (
                format!("origines_declarees|{S1}/generee/Forge"),
                Simple::Vrai
            ),
            (
                format!("origines_declarees|{S2}/generee/Forge"),
                Simple::Vrai
            ),
            (
                format!("origines_declarees|{M1}/generee/Forge"),
                Simple::Vrai
            ),
        ])
    );
    // Chaque nom déjà là, sans origine : rien ne s'ajoute.
    let mut nommee = deja.clone();
    nommee.stems.insert(
        S2.to_string(),
        DejaLu {
            captation: true,
            noms: BTreeSet::from(["Jeu A/basse.wav".to_string()]),
            origines: OriginesLues::default(),
        },
    );
    assert_eq!(
        verser_une_prise_en_stems(
            &ouvert(),
            P,
            Some("Jeu A"),
            &stems_d_essai(),
            None,
            &nommee,
            IDEE,
            HEURE,
            0
        )
        .unwrap(),
        None
    );
    // Les origines seules.
    let op = verser_une_prise_en_stems(
        &ouvert(),
        P,
        Some("Jeu A"),
        &stems_d_essai(),
        Some(&Ajout::Main),
        &nommee,
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(op.message, "Import du 14 novembre : origine déclarée");
    assert_eq!(
        cles_posees(&op)[&format!("origines_declarees|{M1}/main")],
        Simple::Vrai
    );
    // Une prise neuve dont le MIDI d'un stem a déjà son entrée sous `midi_verses` : le lien et son
    // nom, sans ses faits.
    let midi_connu = DejaDeLaPrise {
        midis: BTreeMap::from([(
            M1.to_string(),
            DejaLu {
                captation: true,
                ..DejaLu::default()
            },
        )]),
        ..DejaDeLaPrise::default()
    };
    let cles = cles_posees(
        &verser_une_prise_en_stems(
            &ouvert(),
            P,
            Some("Jeu A"),
            &stems_d_essai(),
            None,
            &midi_connu,
            IDEE,
            HEURE,
            0,
        )
        .unwrap()
        .unwrap(),
    );
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/midi/{M1}")],
        Simple::Vrai
    );
    assert_eq!(
        cles[&format!("midi_verses|{M1}/nom/Jeu A/voix.mid")],
        Simple::Chaine(I.into())
    );
    assert!(
        !cles.contains_key(&format!("midi_verses|{M1}/format"))
            && !cles.contains_key(&format!("midi_verses|{M1}/division"))
    );
}

#[test]
fn une_prise_en_stems_connue_accueille_le_midi_nouveau_d_un_stem() {
    // Une prise déjà versée sans MIDI ; `voix.mid` ajouté à son dossier, puis le jeu réimporté :
    // le manifeste, qui ne compte que les stems audio, la dit connue (ADR-024, décision 1).
    let sans_midi = DejaDeLaPrise {
        prise: DejaLu {
            captation: true,
            noms: BTreeSet::from(["Jeu A".to_string()]),
            origines: OriginesLues::default(),
        },
        stems: BTreeMap::from([
            (
                S1.to_string(),
                DejaLu {
                    captation: true,
                    noms: BTreeSet::from(["Jeu A/voix.wav".to_string()]),
                    origines: OriginesLues::default(),
                },
            ),
            (
                S2.to_string(),
                DejaLu {
                    captation: true,
                    noms: BTreeSet::from(["Jeu A/basse.wav".to_string()]),
                    origines: OriginesLues::default(),
                },
            ),
        ]),
        ..DejaDeLaPrise::default()
    };
    let op = verser_une_prise_en_stems(
        &ouvert(),
        P,
        Some("Jeu A"),
        &stems_d_essai(),
        None,
        &sans_midi,
        IDEE,
        HEURE,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        op.message,
        "Import du 14 novembre : nom d'origine ajouté, MIDI d'un stem ajouté"
    );
    // Le lien de la voix à son MIDI, les faits du MIDI sans `idee`, et son nom : rien d'autre.
    assert_eq!(
        cles_posees(&op),
        BTreeMap::from([
            (format!("prises|{P}/stem/{S1}/midi/{M1}"), Simple::Vrai),
            (
                format!("midi_verses|{M1}/format"),
                Simple::Chaine("smf1".into())
            ),
            (format!("midi_verses|{M1}/division"), Simple::Naturel(480)),
            (
                format!("midi_verses|{M1}/nom/Jeu A/voix.mid"),
                Simple::Chaine(I.into())
            ),
        ])
    );
    // Le même versement, ce lien posé et le MIDI dans le fonds sous son nom : rien de plus.
    let avec_midi = DejaDeLaPrise {
        midis: BTreeMap::from([(
            M1.to_string(),
            DejaLu {
                captation: true,
                noms: BTreeSet::from(["Jeu A/voix.mid".to_string()]),
                origines: OriginesLues::default(),
            },
        )]),
        liens_midi: BTreeSet::from([(S1.to_string(), M1.to_string())]),
        ..sans_midi.clone()
    };
    assert_eq!(
        verser_une_prise_en_stems(
            &ouvert(),
            P,
            Some("Jeu A"),
            &stems_d_essai(),
            None,
            &avec_midi,
            IDEE,
            HEURE,
            0
        )
        .unwrap(),
        None
    );
    // Le lien posé, le MIDI pas encore dans le fonds : ses faits seuls, sans lien de plus.
    let lien_seul = DejaDeLaPrise {
        liens_midi: BTreeSet::from([(S1.to_string(), M1.to_string())]),
        ..sans_midi.clone()
    };
    let cles = cles_posees(
        &verser_une_prise_en_stems(
            &ouvert(),
            P,
            Some("Jeu A"),
            &stems_d_essai(),
            None,
            &lien_seul,
            IDEE,
            HEURE,
            0,
        )
        .unwrap()
        .unwrap(),
    );
    assert!(!cles.contains_key(&format!("prises|{P}/stem/{S1}/midi/{M1}")));
    assert_eq!(
        cles[&format!("midi_verses|{M1}/division")],
        Simple::Naturel(480)
    );
}

#[test]
fn l_idee_se_date_a_sa_naissance_et_l_import_a_son_ouverture() {
    // Ouvert à `HEURE`, le 14 novembre 2023 à 22:13:20 UTC ; versé deux heures plus tard, le 15 à
    // 00:13:20, au décalage 0.
    let neuf = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E1,
        &FAITS_D_UN_STEM,
        Some("a.wav"),
        None,
        &DejaLu::default(),
        IDEE,
        HEURE + 7_200,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(neuf.message, "Idée du 15 novembre : prise versée");
    let connu = DejaLu {
        captation: true,
        ..DejaLu::default()
    };
    let ajout = verser_un_fichier(
        &ouvert(),
        Sorte::Prise,
        E1,
        &FAITS_D_UN_STEM,
        Some("a.wav"),
        None,
        &connu,
        IDEE,
        HEURE + 7_200,
        0,
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        ajout.message,
        "Import du 14 novembre : nom d'origine ajouté"
    );
}

#[test]
fn clore_ecrit_l_empreinte_du_rapport() {
    let op = clore(&ouvert(), R, HEURE + 3_600, 0);
    assert_eq!(op.message, "Import du 14 novembre clos");
    assert_eq!(
        cles_posees(&op)[&format!("imports|{I}/rapport")],
        Simple::Chaine(R.into())
    );
}

#[test]
fn declarer_une_origine_apres_l_import() {
    let generee = OriginesLues {
        main: false,
        generees: BTreeSet::from(["Atelier".into()]),
    };
    let rien = OriginesLues::default();
    let cibles = [
        Cible {
            empreinte: E1,
            origines: &generee,
            nommee: "« a.wav »",
        },
        Cible {
            empreinte: E2,
            origines: &rien,
            nommee: "« b.wav »",
        },
    ];
    let faite = declarer_une_origine(&cibles, Some(&Ajout::Main), "« Démos/ »", HEURE).unwrap();
    assert_eq!((faite.deja_portees, faite.generees_gardees), (0, 1));
    let op = faite.operation.unwrap();
    assert_eq!(
        op.message,
        "Origine déclarée pour « Démos/ » : de la main de Sacha"
    );
    assert_eq!(cles_posees(&op).len(), 1);
    assert_eq!(
        declarer_une_origine(&cibles, None, "x", HEURE).unwrap_err(),
        Impossible::OrigineNonRetiree
    );
    assert_eq!(
        declarer_une_origine(&cibles, Some(&Ajout::Generee(String::new())), "x", HEURE)
            .unwrap_err(),
        Impossible::NomDeGenerateurVide
    );
    let seule =
        declarer_une_origine(&cibles[..1], Some(&Ajout::Main), "cette prise", HEURE).unwrap();
    assert_eq!(
        (seule.operation, seule.deja_portees, seule.generees_gardees),
        (None, 0, 1)
    );
    let op = declarer_une_origine(
        &cibles[1..],
        Some(&Ajout::Generee("Forge".into())),
        "« b.wav »",
        HEURE,
    )
    .unwrap()
    .operation
    .unwrap();
    assert_eq!(
        op.message,
        "Origine déclarée pour « b.wav » : générée par Forge"
    );
    // déjà portée : rien ne change, et rien ne vaut un avertissement (R16)
    let main = OriginesLues {
        main: true,
        generees: BTreeSet::new(),
    };
    let deja = declarer_une_origine(
        &[Cible {
            empreinte: E3,
            origines: &main,
            nommee: "« c.wav »",
        }],
        Some(&Ajout::Main),
        "« c.wav »",
        HEURE,
    )
    .unwrap();
    assert_eq!(
        (deja.operation, deja.deja_portees, deja.generees_gardees),
        (None, 1, 0)
    );
    // de la main de Sacha, puis générée : « de la main de Sacha » redéclarée ne change rien, et
    // l'écran la montre générée ; elle vaut l'avertissement (question 22 ; R16)
    let main_et_generee = OriginesLues {
        main: true,
        generees: BTreeSet::from(["Forge".into()]),
    };
    let redeclaree = declarer_une_origine(
        &[Cible {
            empreinte: E3,
            origines: &main_et_generee,
            nommee: "« c.wav »",
        }],
        Some(&Ajout::Main),
        "« c.wav »",
        HEURE,
    )
    .unwrap();
    assert_eq!(
        (
            redeclaree.operation,
            redeclaree.deja_portees,
            redeclaree.generees_gardees
        ),
        (None, 0, 1)
    );
    let atelier = declarer_une_origine(
        &cibles[..1],
        Some(&Ajout::Generee("Atelier".into())),
        "« a.wav »",
        HEURE,
    )
    .unwrap();
    assert_eq!(
        (
            atelier.operation,
            atelier.deja_portees,
            atelier.generees_gardees
        ),
        (None, 1, 0)
    );
}

#[test]
fn un_nom_de_generateur_fait_de_blancs_est_vide() {
    // Vide une fois ses blancs ôtés, l'espace insécable comprise, comme le nom d'une requête.
    let blancs = Ajout::Generee(" \u{a0}\t".into());
    let declarations = BTreeMap::from([(String::new(), origine(blancs.clone()))]);
    assert_eq!(
        ouvrir(I, &declarations, HEURE, 0).unwrap_err(),
        Impossible::NomDeGenerateurVide
    );
    let rien = OriginesLues::default();
    let cible = [Cible {
        empreinte: E1,
        origines: &rien,
        nommee: "« a.wav »",
    }];
    assert_eq!(
        declarer_une_origine(&cible, Some(&blancs), "« a.wav »", HEURE).unwrap_err(),
        Impossible::NomDeGenerateurVide
    );
}

#[test]
fn garder_puis_choisir_le_type() {
    let gardee = cles_posees(
        &choisir_le_type(
            P,
            S1,
            true,
            TypeDeStem::Voix,
            ChoixDuType::Garder,
            Some("voix.wav"),
            HEURE,
        )
        .unwrap(),
    );
    assert_eq!(
        gardee[&format!("prises|{P}/stem/{S1}/type")],
        Simple::Chaine("voix".into())
    );
    assert_eq!(
        gardee[&format!("prises|{P}/stem/{S1}/type/provenance")],
        Simple::Chaine("importee".into())
    );
    assert_eq!(
        gardee[&format!("prises|{P}/stem/{S1}/type/par")],
        Simple::Chaine("nom".into())
    );
    assert_eq!(
        gardee[&format!("prises|{P}/stem/{S1}/type/depuis")],
        Simple::Chaine(S1.into())
    );
    let op = choisir_le_type(
        P,
        S1,
        true,
        TypeDeStem::Voix,
        ChoixDuType::Choisir(TypeDeStem::Choeurs),
        None,
        HEURE,
    )
    .unwrap();
    assert_eq!(op.message, "Type de ce stem : chœurs");
    let choisie = cles_posees(&op);
    assert_eq!(
        choisie[&format!("prises|{P}/stem/{S1}/type/provenance")],
        Simple::Chaine("saisie".into())
    );
    assert!(
        !choisie.contains_key(&format!("prises|{P}/stem/{S1}/type/par")),
        "saisie : ni par ni depuis (R5)"
    );
    for ote in ["/type/par", "/type/depuis"] {
        assert!(
            op.changements.iter().any(|c| matches!(
                c,
                Changement::Oter { chemin } if chemin.last().is_some_and(|s| s.ends_with(ote))
            )),
            "{ote}"
        );
    }
    assert_eq!(
        choisir_le_type(
            P,
            S1,
            false,
            TypeDeStem::Voix,
            ChoixDuType::Garder,
            None,
            HEURE
        )
        .unwrap_err(),
        Impossible::StemAbsent
    );
}

#[test]
fn un_type_garde_se_date_et_nomme_son_stem() {
    let op = choisir_le_type(
        P,
        S1,
        true,
        TypeDeStem::Synthe,
        ChoixDuType::Garder,
        Some("pad.wav"),
        HEURE + 7,
    )
    .unwrap();
    assert_eq!(op.message, "Type de « pad.wav » : synthé");
    assert_eq!(op.heure, HEURE + 7);
    let cles = cles_posees(&op);
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/type")],
        Simple::Chaine("synthe".into())
    );
    assert_eq!(
        cles[&format!("prises|{P}/stem/{S1}/type/le")],
        Simple::Entier(HEURE + 7)
    );
    assert!(
        op.changements
            .iter()
            .all(|c| matches!(c, Changement::Poser { .. })),
        "gardé : rien ne s'ôte"
    );
}

#[test]
fn le_nom_d_une_copie_se_forme_et_se_relit() {
    assert_eq!(
        nom_de_la_copie(I, 12),
        "import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12"
    );
    assert_eq!(
        copie_de("import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12.4242.partiel"),
        Some((I, 12))
    );
    assert_eq!(
        copie_de("import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12.4242-2.partiel"),
        Some((I, 12))
    );
    let capture = format!("{E1}.4242.partiel");
    for nom in [
        capture.as_str(),
        "import-court-1.1.partiel",
        "import-01HF7YAT00ZZZZZZZZZZZZZZZZ-x.1.partiel",
        "import-01HF7YAT00ZZZZZZZZZZZZZZZZ-12",
        "import-01HF7YAT00ZZZZZZZZZZZZZZZu-1.1.partiel",
        "import-01HF7YAT00ZZZZZZZZZZZZZZZI-1.1.partiel",
    ] {
        assert_eq!(copie_de(nom), None, "{nom}");
    }
}
