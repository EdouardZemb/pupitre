//! L'application des changements sur un document Automerge, sa lecture, et la paire d'adaptateurs
//! en mémoire : une opération s'applique en une seule modification, avec son message et son
//! heure, ou ne laisse rien ; une migration qui attend ses dépendances ne fait pas migrer ; le
//! fonds lu et la version de schéma ne prennent que les valeurs du type du schéma ; la paire en
//! mémoire migre à son ouverture, ouvre en lecture seule un fonds dont la version de schéma ne se
//! lit pas, et rend la même empreinte d'une captation versée deux fois. La migration de test et
//! celle qui attend ses dépendances viennent de la suite des commandes, qui les porte une seule
//! fois, et qui éprouve sur chaque paire le fonds qu'une telle migration laisse en lecture seule.

mod suite;

use std::collections::BTreeSet;

use automerge::{Automerge, AutomergeError, ObjId, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::fonds::{
    ETIQUETTES, IDEES, IDENTIFIANT, LE, MIGRATIONS, TEXTE, VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes::{Appareil, IdeeLue, Provenance, naissance, noter};
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::version::message_de_la_migration;
use services::application::{Inapplicable, appliquer, faire_naitre, migrer, naitre};
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::lecture::{idees, journal, lire_l_idee, version_de_schema};
use services::{Captations, Documents, Raison, Refus};
use suite::{
    CONTENEUR_DE_LA_MIGRATION, migration_de_test, migration_sur_une_autre_genese,
    migrations_de_test,
};

const HEURE: i64 = 1_700_000_000; // le 14 novembre 2023, 22 h 13 min 20 s UTC
/// L'idée que notent les tests.
const I: &str = "01HF7YAT000000000000000004";
/// Une idée créée sans provenance ni date.
const J: &str = "01HF7YAT000000000000000005";
/// Le message des opérations de test.
const MESSAGE_DE_TEST: &str = "Opération de test";
/// L'heure des opérations de test, une minute après la note.
const HEURE_DE_TEST: i64 = HEURE + 60;

fn chemin(segments: &[&str]) -> Vec<String> {
    segments.iter().map(|segment| segment.to_string()).collect()
}

fn poser(segments: &[&str], valeur: Simple) -> Changement {
    Changement::Poser {
        chemin: chemin(segments),
        valeur,
    }
}

/// Une opération de test : `changements`, sous `MESSAGE_DE_TEST`, à `HEURE_DE_TEST`.
fn operation(changements: Vec<Changement>) -> Operation {
    Operation {
        changements,
        message: MESSAGE_DE_TEST.to_string(),
        heure: HEURE_DE_TEST,
    }
}

/// La naissance des tests, à `HEURE` : des identifiants écrits, jamais tirés.
fn naissance_de_test() -> Operation {
    naissance(
        "01HF7YAT000000000000000001",
        "01HF7YAT000000000000000002",
        "00000000-0000-4000-8000-000000000003",
        HEURE,
    )
}

/// Noter `I`, « Un refrain », à l'établi, à `HEURE`.
fn noter_i() -> Operation {
    noter(I, "Un refrain", Appareil::Etabli, HEURE, 0).expect("le texte de la note n'est pas vide")
}

/// Le fonds né de la naissance des tests, puis de la note de `I`.
fn fonds_note() -> Automerge {
    faire_naitre(&naissance_de_test(), &noter_i()).expect("le fonds naît")
}

/// L'objet à clés que porte `cle` dans `parent`.
fn objet(doc: &Automerge, parent: &ObjId, cle: &str) -> ObjId {
    match doc.get(parent, cle) {
        Ok(Some((Value::Object(ObjType::Map), id))) => id,
        autre => panic!("« {cle} » ne porte pas un objet à clés : {autre:?}"),
    }
}

/// La valeur simple que porte `cle` dans `parent`.
fn scalaire(doc: &Automerge, parent: &ObjId, cle: &str) -> ScalarValue {
    match doc.get(parent, cle) {
        Ok(Some((Value::Scalar(valeur), _))) => valeur.into_owned(),
        autre => panic!("« {cle} » ne porte pas une valeur simple : {autre:?}"),
    }
}

/// L'objet de l'idée `id`, sous `IDEES`.
fn idee(doc: &Automerge, id: &str) -> ObjId {
    objet(doc, &objet(doc, &ROOT, IDEES), id)
}

#[test]
fn appliquer_pose_cree_et_ote() {
    let mut doc = fonds_note();
    appliquer(
        &mut doc,
        &operation(vec![poser(&[IDEES, I, ETIQUETTES, "pont"], Simple::Vrai)]),
    )
    .expect("l'étiquette se pose");
    let etiquettes = objet(&doc, &idee(&doc, I), ETIQUETTES);
    assert_eq!(
        scalaire(&doc, &etiquettes, "pont"),
        ScalarValue::Boolean(true)
    );

    let avant = doc.get_heads();
    appliquer(
        &mut doc,
        &operation(vec![
            poser(
                &[IDEES, I, TEXTE],
                Simple::Chaine("Un refrain qui descend".into()),
            ),
            Changement::Creer {
                chemin: chemin(&[IDEES, J]),
                conteneurs: vec![ETIQUETTES.into()],
            },
            Changement::Oter {
                chemin: chemin(&[IDEES, I, ETIQUETTES, "pont"]),
            },
        ]),
    )
    .expect("les trois changements s'appliquent");
    // Les trois changements, en une seule modification.
    assert_eq!(doc.get_changes(&avant).len(), 1);

    let lue = IdeeLue {
        id: I.into(),
        provenance: Provenance::Captee {
            par: Appareil::Etabli,
        },
        le: HEURE,
        texte: Some("Un refrain qui descend".into()),
        etiquettes: BTreeSet::new(),
        retiree: None,
        liens: BTreeSet::new(),
    };
    assert_eq!(lire_l_idee(&doc, I), Some(lue.clone()));
    // Ni `le` ni `provenance` : une idée qui ne se lit pas.
    assert_eq!(lire_l_idee(&doc, J), None);
    let j = idee(&doc, J);
    objet(&doc, &j, ETIQUETTES);
    assert_eq!(doc.keys(&j).collect::<Vec<_>>(), [ETIQUETTES]);
    assert_eq!(idees(&doc), [lue]);
}

#[test]
fn un_objet_absent_annule_toute_l_operation() {
    let mut doc = fonds_note();
    let avant = doc.get_heads();
    let refus = appliquer(
        &mut doc,
        &operation(vec![
            poser(
                &[IDEES, I, TEXTE],
                Simple::Chaine("Un refrain qui monte".into()),
            ),
            poser(&[IDEES, "absente", TEXTE], Simple::Chaine("Un pont".into())),
        ]),
    );
    match refus {
        Err(Inapplicable::ObjetAbsent { chemin }) => assert_eq!(chemin, ["idees", "absente"]),
        autre => panic!("attendu : ObjetAbsent ; rendu : {autre:?}"),
    }
    assert_eq!(doc.get_heads(), avant);
    // Le premier changement, valide, est annulé avec le second.
    assert_eq!(
        lire_l_idee(&doc, I)
            .expect("l'idée se lit")
            .texte
            .as_deref(),
        Some("Un refrain")
    );
}

#[test]
fn une_operation_sans_effet_est_inapplicable() {
    let mut doc = fonds_note();
    let avant = doc.get_heads();
    let refus = appliquer(
        &mut doc,
        &operation(vec![Changement::Oter {
            chemin: chemin(&[IDEES, I, ETIQUETTES, "absente"]),
        }]),
    );
    assert!(matches!(refus, Err(Inapplicable::Vide)), "{refus:?}");
    assert_eq!(doc.get_heads(), avant);
}

#[test]
fn l_operation_porte_son_message_et_son_heure() {
    let mut doc = fonds_note();
    assert_eq!(
        journal(&doc).last(),
        Some(&("Idée du 14 novembre notée".to_string(), HEURE))
    );
    appliquer(
        &mut doc,
        &operation(vec![poser(&[IDEES, I, ETIQUETTES, "pont"], Simple::Vrai)]),
    )
    .expect("l'étiquette se pose");
    assert_eq!(
        journal(&doc).last(),
        Some(&(MESSAGE_DE_TEST.to_string(), HEURE_DE_TEST))
    );
}

#[test]
fn la_genese_puis_la_naissance() {
    let doc = naitre(&naissance_de_test()).expect("le fonds naît");
    assert_eq!(
        journal(&doc),
        [
            ("Genèse du fonds".to_string(), 0),
            ("Fonds créé".to_string(), HEURE)
        ]
    );
    assert_eq!(version_de_schema(&doc), Some(1));
    assert_eq!(
        scalaire(&doc, &ROOT, IDENTIFIANT),
        ScalarValue::Str("01HF7YAT000000000000000001".into())
    );
}

#[test]
fn une_valeur_d_un_autre_type_rend_l_idee_illisible() {
    let mut doc = fonds_note();
    assert!(lire_l_idee(&doc, I).is_some());
    appliquer(
        &mut doc,
        &operation(vec![poser(&[IDEES, I, LE], Simple::Naturel(1_700_000_000))]),
    )
    .expect("la date se pose");
    assert_eq!(
        scalaire(&doc, &idee(&doc, I), LE),
        ScalarValue::Uint(1_700_000_000)
    );
    assert_eq!(lire_l_idee(&doc, I), None);
}

/// Le fonds noté, sa version de schéma reposée en entier signé (`ScalarValue::Int(1)`), de la même
/// valeur que la version de schéma de la genèse, mais d'un autre type.
fn fonds_a_la_version_de_schema_signee() -> Automerge {
    let mut doc = fonds_note();
    appliquer(
        &mut doc,
        &operation(vec![poser(&[VERSION_DE_SCHEMA], Simple::Entier(1))]),
    )
    .expect("la version de schéma se pose");
    assert_eq!(
        scalaire(&doc, &ROOT, VERSION_DE_SCHEMA),
        ScalarValue::Int(1)
    );
    doc
}

#[test]
fn une_version_de_schema_d_un_autre_type_ne_se_lit_pas() {
    // La valeur se reconnaît à sa variante : un entier signé n'est pas une version de schéma.
    assert_eq!(
        version_de_schema(&fonds_a_la_version_de_schema_signee()),
        None
    );
}

#[test]
fn la_paire_en_memoire_ouvre_en_lecture_seule_un_fonds_de_version_de_schema_illisible() {
    let octets = fonds_a_la_version_de_schema_signee().save();
    let mut documents = DocumentsEnMemoire::de(&octets, MIGRATIONS).expect("le fonds s'ouvre");
    let refus = documents
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(refus.raison, Raison::Illisible);
    let avant = documents.fonds().expect("le fonds se lit").get_heads();
    assert_eq!(
        documents.appliquer(&operation(vec![poser(
            &[IDEES, I, ETIQUETTES, "pont"],
            Simple::Vrai
        )])),
        Err(refus)
    );
    let fonds = documents.fonds().expect("le fonds se lit");
    assert_eq!(fonds.get_heads(), avant);
    assert!(lire_l_idee(fonds, I).is_some());
}

#[test]
fn la_paire_en_memoire_migre_a_son_ouverture() {
    let mut documents = DocumentsEnMemoire::neuf();
    assert!(documents.fonds().is_none());
    assert_eq!(
        documents.faire_naitre(&naissance_de_test(), &noter_i()),
        Ok(None)
    );
    let octets = documents.octets().expect("le fonds est né");

    let migration = migration_de_test();
    let migre =
        DocumentsEnMemoire::de(&octets, &migrations_de_test(&migration)).expect("le fonds s'ouvre");
    assert_eq!(migre.lecture_seule(), None);
    let fonds = migre.fonds().expect("le fonds se lit");
    assert_eq!(version_de_schema(fonds), Some(2));
    objet(fonds, &ROOT, CONTENEUR_DE_LA_MIGRATION);
    // La migration, concurrente de la naissance et de la note, est au journal.
    assert!(journal(fonds).contains(&(message_de_la_migration(2), 0)));
    assert!(lire_l_idee(fonds, I).is_some());

    // Rouvert par une application qui ne connaît que la version de schéma 1.
    let octets = migre.octets().expect("le fonds migré");
    let mut rouvert = DocumentsEnMemoire::de(&octets, MIGRATIONS).expect("le fonds s'ouvre");
    let refus = rouvert
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(refus.raison, Raison::VersionPlusRecente { version: 2 });
    assert_eq!(
        rouvert.appliquer(&operation(vec![poser(
            &[IDEES, I, ETIQUETTES, "pont"],
            Simple::Vrai
        )])),
        Err(refus)
    );
    let fonds = rouvert.fonds().expect("le fonds se lit");
    assert_eq!(version_de_schema(fonds), Some(2));
    assert!(lire_l_idee(fonds, I).is_some());
}

#[test]
fn migrer_refuse_une_migration_qui_attend_ses_dependances() {
    let migration = migration_sur_une_autre_genese();
    // `apply_changes` garde la migration en attente, sans erreur : `migrer` le dit.
    let mut doc = fonds_note();
    let migre = migrer(&mut doc, &migrations_de_test(&migration));
    assert!(
        matches!(migre, Err(AutomergeError::MissingDeps)),
        "{migre:?}"
    );
    assert_eq!(version_de_schema(&doc), Some(1));
}

#[test]
fn des_octets_qui_ne_sont_pas_un_fonds_ne_s_ouvrent_pas() {
    let Err(refus) = DocumentsEnMemoire::de(b"pas un fonds", MIGRATIONS) else {
        panic!("des octets qui ne sont pas un fonds se sont ouverts");
    };
    assert_eq!(refus.raison, Raison::Illisible);
}

#[test]
fn rien_ne_s_applique_avant_la_naissance_du_fonds() {
    let mut documents = DocumentsEnMemoire::neuf();
    assert_eq!(documents.lecture_seule(), None);
    assert_eq!(
        documents.appliquer(&noter_i()),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Aucun fonds n'est né.".to_string(),
        })
    );
    assert!(documents.fonds().is_none());
    assert_eq!(documents.octets(), None);
}

#[test]
fn le_fonds_ne_nait_qu_une_fois() {
    let mut documents = DocumentsEnMemoire::neuf();
    assert_eq!(
        documents.faire_naitre(&naissance_de_test(), &noter_i()),
        Ok(None)
    );
    let avant = documents.fonds().expect("le fonds est né").get_heads();
    assert_eq!(
        documents.faire_naitre(&naissance_de_test(), &noter_i()),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Le fonds est déjà né : il ne naît qu'une fois.".to_string(),
        })
    );
    assert_eq!(
        documents.fonds().expect("le fonds est né").get_heads(),
        avant
    );
}

#[test]
fn une_captation_versee_deux_fois_rend_la_meme_empreinte() {
    let captations = CaptationsEnMemoire::default();
    let octets = "une captation versée deux fois".as_bytes();
    let e = captations.verser(octets).expect("la captation se verse");
    assert_eq!(e, empreinte(octets));
    assert_eq!(captations.verser(octets), Ok(e.clone()));
    assert_eq!(captations.relire(&e), Ok(octets.to_vec()));

    let autre = empreinte("une captation jamais versée".as_bytes());
    assert_eq!(
        captations.relire(&autre),
        Err(Refus {
            raison: Raison::Impossible,
            message: format!("Aucune captation ne porte l'empreinte {autre}."),
        })
    );
}
