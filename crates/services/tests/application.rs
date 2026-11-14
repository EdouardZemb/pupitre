//! L'application des changements sur un document Automerge, sa lecture, et la paire d'adaptateurs
//! en mémoire : une opération s'applique en une seule modification, avec son message et son
//! heure, ou ne laisse rien ; le fonds lu ne prend que les valeurs du type du schéma ; la paire en
//! mémoire migre à son ouverture, et rend la même empreinte d'une captation versée deux fois.

use std::collections::BTreeSet;

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{ActorId, Automerge, Change, ObjId, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, ETIQUETTES, GENESE, IDEES, IDENTIFIANT, LE, MIGRATIONS, TEXTE,
    VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes::{Appareil, IdeeLue, Provenance, naissance, noter};
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::version::{Migration, message_de_la_migration};
use services::application::{Inapplicable, appliquer, faire_naitre, naitre};
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::lecture::{idees, journal, lire_l_idee, version_de_schema};
use services::{Captations, Documents, Raison, Refus};

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

/// Le conteneur de la racine du document que crée la migration de test : un objet à clés, où la
/// donnée nouvelle d'un objet s'écrit en valeurs simples, sous `<objet>/<élément>` (§ 31 · 2.4).
const CONTENEUR_DE_LA_MIGRATION: &str = "elements_de_test";

/// La migration de test vers la version de schéma 2, engendrée à l'exécution depuis `GENESE`, de
/// façon déterministe, comme la recette de la genèse (`genese_par_la_recette`, dans
/// `crates/stockage/tests/genese.rs`) : l'acteur `ACTEUR_DE_LA_GENESE`, qui pose
/// `VERSION_DE_SCHEMA` à 2 et crée `CONTENEUR_DE_LA_MIGRATION`, un objet à clés, à la racine du
/// document, sous le message `message_de_la_migration(2)`, à l'heure 0. Rend les octets de sa
/// modification (`raw_bytes()`), qui ne dépend que de la genèse.
fn migration_de_test() -> Vec<u8> {
    let mut doc = Automerge::new().with_actor(ActorId::from(ACTEUR_DE_LA_GENESE));
    doc.apply_changes([Change::try_from(GENESE).expect("lire la genèse")])
        .expect("appliquer la genèse");
    let mut tx = doc.transaction();
    tx.put(ROOT, VERSION_DE_SCHEMA, 2u64)
        .expect("poser la version de schéma 2");
    tx.put_object(ROOT, CONTENEUR_DE_LA_MIGRATION, ObjType::Map)
        .expect("créer le conteneur de la migration");
    let message = message_de_la_migration(2);
    let options = CommitOptions::default()
        .with_message(message.as_str())
        .with_time(0);
    let (h, _) = tx.commit_with(options);
    doc.get_change_by_hash(&h.expect("la migration est une modification"))
        .expect("la modification de la migration")
        .raw_bytes()
        .to_vec()
}

/// Les migrations de test, une seule : `octets`, vers la version de schéma 2, apportée par la
/// version de Pupitre `0.0.0-essai`.
fn migrations_de_test(octets: &[u8]) -> [Migration<'_>; 1] {
    [Migration {
        vers: 2,
        octets,
        version_de_pupitre: "0.0.0-essai",
    }]
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
