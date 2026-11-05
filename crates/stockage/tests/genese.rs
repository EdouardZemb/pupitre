//! La genèse du fonds (§ 31 · 2.3) : la recette la reconstruit à l'octet, elle pose la version
//! de schéma 1 et les conteneurs de la racine du document, et deux copies parties d'elle gardent
//! les idées des deux côtés, ce que deux documents nés chacun de son côté ne font pas.

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{ActorId, Automerge, Change, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, GENESE, IDEES, MESSAGE_DE_LA_GENESE, PRISES, VERSION_DE_SCHEMA,
};

/// La recette de la genèse : les octets de `GENESE` en dépendent, et elle ne change plus après
/// la `v0.3.0`.
fn genese_par_la_recette() -> Vec<u8> {
    let mut doc = Automerge::new().with_actor(ActorId::from(ACTEUR_DE_LA_GENESE));
    let mut tx = doc.transaction();
    tx.put(ROOT, VERSION_DE_SCHEMA, 1u64).unwrap();
    tx.put_object(ROOT, IDEES, ObjType::Map).unwrap();
    tx.put_object(ROOT, PRISES, ObjType::Map).unwrap();
    let options = CommitOptions::default()
        .with_message(MESSAGE_DE_LA_GENESE)
        .with_time(0);
    let (h, _) = tx.commit_with(options);
    doc.get_change_by_hash(&h.unwrap())
        .unwrap()
        .raw_bytes()
        .to_vec()
}

/// Un document parti de `GENESE`, d'un acteur neuf qu'Automerge tire.
fn parti_de_la_genese() -> Automerge {
    let genese = Change::try_from(GENESE).expect("lire la genèse");
    let mut doc = Automerge::new();
    doc.apply_changes([genese]).expect("appliquer la genèse");
    doc
}

/// Écrit l'idée `idee`, un objet à clés vide, sous l'objet des idées que porte déjà `doc`.
fn ecrire_une_idee(doc: &mut Automerge, idee: &str) {
    let (_, idees) = doc
        .get(ROOT, IDEES)
        .expect("lire la racine du document")
        .expect("l'objet des idées");
    let mut tx = doc.transaction();
    tx.put_object(&idees, idee, ObjType::Map)
        .expect("écrire l'idée");
    let (h, _) = tx.commit();
    assert!(h.is_some(), "l'idée {idee} s'écrit en une modification");
}

/// Un document né de son côté, sans genèse : il crée son propre objet des idées, puis y écrit
/// l'idée `idee`.
fn ne_de_son_cote(idee: &str) -> Automerge {
    let mut doc = Automerge::new();
    let mut tx = doc.transaction();
    let idees = tx
        .put_object(ROOT, IDEES, ObjType::Map)
        .expect("créer l'objet des idées");
    tx.put_object(&idees, idee, ObjType::Map)
        .expect("écrire l'idée");
    let (h, _) = tx.commit();
    assert!(h.is_some(), "l'idée {idee} s'écrit en une modification");
    doc
}

/// Les idées que présente l'objet des idées de `doc`, dans l'ordre de leurs clés.
fn idees_visibles(doc: &Automerge) -> Vec<String> {
    let (_, idees) = doc
        .get(ROOT, IDEES)
        .expect("lire la racine du document")
        .expect("l'objet des idées");
    doc.keys(&idees).collect()
}

#[test]
fn la_genese_se_reconstruit_a_l_octet() {
    let recette = genese_par_la_recette();

    assert!(
        recette == GENESE,
        "la recette rend d'autres octets que GENESE :\n{recette:?}"
    );
}

#[test]
fn la_genese_pose_la_version_de_schema_1_et_les_conteneurs() {
    let genese = Change::try_from(GENESE).expect("lire la genèse");

    assert_eq!(*genese.actor_id(), ActorId::from(ACTEUR_DE_LA_GENESE));
    assert_eq!(genese.timestamp(), 0, "la genèse porte l'heure 0");
    assert_eq!(genese.message(), Some(MESSAGE_DE_LA_GENESE));
    assert!(
        genese.deps().is_empty(),
        "la genèse ne dépend de rien : {:?}",
        genese.deps()
    );

    let mut doc = Automerge::new();
    doc.apply_changes([genese]).expect("appliquer la genèse");
    let mut cles: Vec<String> = doc.keys(ROOT).collect();
    cles.sort();
    assert_eq!(cles, [IDEES, PRISES, VERSION_DE_SCHEMA]);
    match doc
        .get(ROOT, VERSION_DE_SCHEMA)
        .expect("lire la racine du document")
    {
        Some((Value::Scalar(version), _)) => assert_eq!(*version, ScalarValue::Uint(1)),
        autre => panic!("la version de schéma n'est pas un scalaire : {autre:?}"),
    }
    for conteneur in [IDEES, PRISES] {
        match doc
            .get(ROOT, conteneur)
            .expect("lire la racine du document")
        {
            Some((Value::Object(ObjType::Map), _)) => {}
            autre => panic!("{conteneur} n'est pas un objet à clés : {autre:?}"),
        }
    }
}

#[test]
fn avec_la_genese_les_idees_des_deux_cotes_restent() {
    let mut a = parti_de_la_genese();
    let mut b = parti_de_la_genese();
    ecrire_une_idee(&mut a, "idee-a");
    ecrire_une_idee(&mut b, "idee-b");

    a.merge(&mut b).expect("fusionner les deux documents");

    assert_eq!(idees_visibles(&a), ["idee-a", "idee-b"]);
}

/// Ce test décrit Automerge, et non notre code : c'est la raison de la genèse.
#[test]
fn sans_genese_les_idees_d_un_cote_sortent_de_la_vue() {
    let mut a = ne_de_son_cote("idee-a");
    let mut b = ne_de_son_cote("idee-b");

    a.merge(&mut b).expect("fusionner les deux documents");

    // Les deux objets des idées se disputent la même clé : Automerge en présente un et range
    // l'autre parmi les conflits. Lequel dépend de l'ordre des acteurs, tirés ici au hasard :
    // le test ne dit pas lequel.
    let visibles = idees_visibles(&a);
    assert!(
        visibles == ["idee-a"] || visibles == ["idee-b"],
        "une seule des deux idées reste visible : {visibles:?}"
    );
    let objets_des_idees = a.get_all(ROOT, IDEES).expect("lire la racine du document");
    assert_eq!(
        objets_des_idees.len(),
        2,
        "l'autre objet reste parmi les conflits"
    );
}
