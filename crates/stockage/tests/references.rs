//! La genèse et la teneur figées (§ 31 · 10, test 11) : les octets de la genèse et la teneur de
//! deux documents témoins, engendrés à l'exécution, à leurs empreintes de référence : l'un
//! ordinaire, né par les opérations, l'autre qui porte une valeur de chaque type que la teneur
//! écrit ; et deux documents qui ne diffèrent que par leur lignée ont la même teneur (§ 31 · 2.6).
//!
//! Ce fichier est le seul des empreintes de référence. Le test ne lit aucune étiquette, que la
//! CI ne récupère pas. Il inclut la suite des commandes, où les gestes du témoin sont écrits.

#[path = "../../services/tests/suite/mod.rs"]
mod suite;

mod commun;

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{Automerge, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use commun::{HEURE, dossier_jetable, ecrire, faite, ouvrir};
use noyau::donnees::fonds::{GENESE, LIGNEE, LIGNEES_REMPLACEES};
use noyau::donnees::gestes;
use services::{Alea, Contexte, Services};
use stockage::{Captations, FondsSurFichiers, RacineDesDonnees, empreinte, teneur};
use suite::AleaDeTest;

/// Les empreintes de référence du test 11. Une ligne se change ou se retire avant la `v0.3.0`, et
/// la PR le dit ; dès elle, seule une ligne ajoutée passe (§ 31 · 10, test 11 ; ADR-023).
const GENESE_DU_FONDS: &str = "a02890acd15f12619da7f1f5abea9a8056db7188f909a467896ea50d4b5a6fc1";
const TENEUR_DU_TEMOIN: &str = "291a0df280e2274fe96fc10d24a232a57213a0e938fa0a7b844be8f056657dbc";
const TENEUR_DES_TYPES: &str = "35d51798213927730a9c636b9d6b72760400f3be0f674b9566b7d012ae7f5d30";

/// Les services de l'établi sur la paire de fichiers d'une racine des données.
type ServicesSurFichiers<'r> = Services<FondsSurFichiers<'r>, Captations<'r>>;

/// L'aléa d'un témoin d'une autre lignée : les tirages d'`AleaDeTest { graine: 0, appels: 0 }`,
/// sauf le deuxième, celui de la lignée, dont l'octet 1 vaut 1.
struct AutreLignee(AleaDeTest);

impl Alea for AutreLignee {
    fn remplir(&mut self, octets: &mut [u8]) {
        self.0.remplir(octets);
        if self.0.appels == 2 {
            octets[1] = 1;
        }
    }
}

/// Le document témoin, né dans `racine` par les opérations : les services de l'établi, sur la
/// paire de fichiers de `racine`, jouent les gestes du témoin (`suite::temoin`), à l'heure des
/// tests, sous l'aléa `alea`, chacun fait sans avertissement. Rend les services, ouverts.
fn faire_naitre_le_temoin<'r>(
    racine: &'r RacineDesDonnees,
    alea: &mut dyn Alea,
) -> ServicesSurFichiers<'r> {
    let mut temoin = Services::new(
        FondsSurFichiers::ouvrir(racine).expect("ouvrir le fonds"),
        racine.captations(),
        gestes::Appareil::Etabli,
    );
    suite::temoin(
        &mut temoin,
        &mut Contexte {
            heure: suite::HEURE,
            alea,
        },
    );
    temoin
}

/// Le fonds du témoin `temoin`, qui est né.
fn fonds<'a>(temoin: &'a ServicesSurFichiers<'_>) -> &'a Automerge {
    temoin
        .documents()
        .document()
        .expect("le témoin est né")
        .automerge()
}

/// Le second document témoin, bâti par Automerge, en mémoire : une valeur de chaque type que la
/// teneur écrit, à la racine du document. Une chaîne qui porte une barre et un guillemet ; un
/// compteur posé à 3, puis augmenté de 2 par une seconde modification ; un entier négatif ; une
/// liste de flottants, 0,1, 1e300, 100, qui s'écrit `100` et non `1e2`, 0, -0, l'infini et moins
/// l'infini ; un instant négatif ; une liste d'un entier non signé et d'une chaîne ; un objet à
/// clés imbriqué, qui porte un booléen et un objet à clés vide ; deux octets ; rien ; un objet
/// `Text`.
fn batir_le_temoin_des_types() -> Automerge {
    let mut temoin = Automerge::new();
    let mut tx = temoin.transaction();
    tx.put(ROOT, "chaine", "a\\b\"c").expect("poser la chaîne");
    tx.put(ROOT, "compteur", ScalarValue::counter(3))
        .expect("poser le compteur");
    tx.put(ROOT, "entier", ScalarValue::Int(-7))
        .expect("poser l'entier");
    let flottants = tx
        .put_object(ROOT, "flottants", ObjType::List)
        .expect("créer la liste des flottants");
    let valeurs = [
        0.1,
        1e300,
        100.0,
        0.0,
        -0.0,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for (indice, flottant) in valeurs.into_iter().enumerate() {
        tx.insert(&flottants, indice, flottant)
            .expect("insérer un flottant");
    }
    tx.put(ROOT, "instant", ScalarValue::Timestamp(-5))
        .expect("poser l'instant");
    let liste = tx
        .put_object(ROOT, "liste", ObjType::List)
        .expect("créer la liste");
    tx.insert(&liste, 0, 1u64)
        .expect("insérer l'entier non signé");
    tx.insert(&liste, 1, "a").expect("insérer la chaîne");
    let objet = tx
        .put_object(ROOT, "objet", ObjType::Map)
        .expect("créer l'objet à clés");
    tx.put(&objet, "faux", false).expect("poser le booléen");
    tx.put_object(&objet, "vide", ObjType::Map)
        .expect("créer l'objet à clés vide");
    tx.put(ROOT, "octets", ScalarValue::Bytes(vec![0x00, 0xab]))
        .expect("poser les octets");
    tx.put(ROOT, "rien", ScalarValue::Null).expect("poser rien");
    let texte = tx
        .put_object(ROOT, "texte", ObjType::Text)
        .expect("créer l'objet texte");
    tx.splice_text(&texte, 0, 0, "abc")
        .expect("écrire l'objet texte");
    tx.commit_with(CommitOptions::default().with_time(HEURE));
    let mut tx = temoin.transaction();
    tx.increment(ROOT, "compteur", 2)
        .expect("augmenter le compteur");
    tx.commit_with(CommitOptions::default().with_time(HEURE + 1));
    temoin
}

/// La lignée que porte le fonds `fonds`, à la racine du document.
fn lignee(fonds: &Automerge) -> String {
    match fonds.get(ROOT, LIGNEE).expect("lire la lignée") {
        Some((Value::Scalar(valeur), _)) => match valeur.as_ref() {
            ScalarValue::Str(texte) => texte.to_string(),
            autre => panic!("la lignée n'est pas une chaîne : {autre:?}"),
        },
        autre => panic!("la lignée n'est pas une valeur simple : {autre:?}"),
    }
}

// § 31 · 10, test 11 : comparer les octets de chaque genèse, et la teneur de deux documents témoins, l'un ordinaire, […] à leurs empreintes de référence
#[test]
fn genese_et_teneur_figees() {
    assert_eq!(empreinte(GENESE), GENESE_DU_FONDS);
    let racine = ouvrir(&dossier_jetable("references-temoin"));
    let temoin = faire_naitre_le_temoin(
        &racine,
        &mut AleaDeTest {
            graine: 0,
            appels: 0,
        },
    );
    let teneur = teneur(fonds(&temoin)).unwrap();
    // La teneur attendue, 1 708 octets sur une seule ligne, coupée ici après une virgule, une
    // accolade ouvrante ou un deux-points :
    // {"idees":{"01HF7YAT000000000000000004":{"etiquettes":{"refrain":b1},"le":i1700000000,
    //  "par":s"etabli","provenance":s"captee","texte":s"Un refrain qui monte, en ré"},
    //  "01HF7YAT000000000000000005":{"etiquettes":{},"le":i1700000000,"par":s"etabli",
    //  "provenance":s"captee","texte":s"Un pont en mi mineur"},"01HF7YAT000000000000000006":{
    //  "depuis":s"01HF7YAT00ZZZZZZZZZZZZZZZZ","etiquettes":{},"le":i1700000000,"par":s"etabli",
    //  "provenance":s"importee"},"01HF7YAT000000000000000007":{"depuis":
    //  s"01HF7YAT00ZZZZZZZZZZZZZZZZ","etiquettes":{},"le":i1700000000,"par":s"etabli",
    //  "provenance":s"importee","retiree":i1700000000},"01HF7YAT000000000000000008":{"depuis":
    //  s"01HF7YAT00ZZZZZZZZZZZZZZZZ","etiquettes":{},"le":i1700000000,"par":s"etabli",
    //  "provenance":s"importee"}},"identifiant":s"01HF7YAT000000000000000001","imports":{},"liens":
    //  {"apparente:01HF7YAT000000000000000004:01HF7YAT000000000000000005":b1},"midi_verses":{
    //  "dee9bf9e8aaeb03dce938f41ee65b33989d3436072478a95514ca26df98f39d3/format":s"smf1",
    //  "dee9bf9e8aaeb03dce938f41ee65b33989d3436072478a95514ca26df98f39d3/idee":
    //  s"01HF7YAT000000000000000004"},"origines_declarees":{},"photos":{},"prises":{
    //  "8c12b061b3b4734f4f62d617674f892c49cb31c9212878903601d7ca369a3299/duree":u30000,
    //  "8c12b061b3b4734f4f62d617674f892c49cb31c9212878903601d7ca369a3299/format":s"wav",
    //  "8c12b061b3b4734f4f62d617674f892c49cb31c9212878903601d7ca369a3299/idee":
    //  s"01HF7YAT000000000000000006"},"proprietaire":s"00000000-0000-4000-8000-000000000003",
    //  "requetes_enregistrees":{},"textes_verses":{
    //  "05474c7f2b8f6390a4a215fa6edca1ac61a8f97aa9cad1e08652909008b565b8/format":s"brut",
    //  "05474c7f2b8f6390a4a215fa6edca1ac61a8f97aa9cad1e08652909008b565b8/idee":
    //  s"01HF7YAT000000000000000008"},"version_de_schema":u1}
    assert_eq!(
        empreinte(teneur.as_bytes()),
        TENEUR_DU_TEMOIN,
        "teneur : {teneur}"
    );
}

// § 31 · 10, test 11 : comparer […] la teneur de deux documents témoins, […] l'autre qui porte une valeur de chaque type que la teneur écrit, à leurs empreintes de référence
#[test]
fn la_teneur_de_chaque_type_est_figee() {
    // La teneur du second témoin, en mémoire puis relu de ses octets, se compare d'abord au texte
    // attendu, pour qu'un écart se lise, puis l'empreinte de ce texte à sa référence.
    let temoin = batir_le_temoin_des_types();
    let relu = Automerge::load(&temoin.save()).expect("relire le témoin des types");
    let attendue = r#"{"chaine":s"a\\b\"c","compteur":c5,"entier":i-7,"flottants":[f0.1,f1e300,f100,f0,f-0,finf,f-inf],"instant":d-5,"liste":[u1,s"a"],"objet":{"faux":b0,"vide":{}},"octets":x00ab,"rien":n,"texte":t"abc"}"#;
    for (quoi, document) in [("en mémoire", &temoin), ("relu", &relu)] {
        assert_eq!(
            teneur(document).unwrap(),
            attendue,
            "la teneur du témoin des types, {quoi}"
        );
    }
    assert_eq!(empreinte(attendue.as_bytes()), TENEUR_DES_TYPES);
}

// § 31 · 10, test 11 : deux documents qui ne diffèrent que par leur lignée ont la même teneur
#[test]
fn la_lignee_ne_change_pas_la_teneur() {
    // Le témoin sous `AleaDeTest { graine: 0, appels: 0 }`, et sous `AutreLignee`, dont seul le
    // tirage de la lignée diffère, donc seule la lignée, une écriture de plus y posant
    // `LIGNEES_REMPLACEES = ["01HF7YAT000000000000000002"]` : deux teneurs égales. Une autre
    // graine changerait aussi l'identifiant du document, l'identifiant de propriétaire et chaque
    // idée, que la teneur porte.
    let d = dossier_jetable("references-lignee");
    let racine_a = ouvrir(&d.join("a"));
    let racine_b = ouvrir(&d.join("b"));
    let a = faire_naitre_le_temoin(
        &racine_a,
        &mut AleaDeTest {
            graine: 0,
            appels: 0,
        },
    );
    let mut b = faire_naitre_le_temoin(
        &racine_b,
        &mut AutreLignee(AleaDeTest {
            graine: 0,
            appels: 0,
        }),
    );
    faite(
        ecrire(
            b.documents_mut()
                .document_mut()
                .expect("le témoin B est né"),
            |tx| {
                let remplacees = tx.put_object(ROOT, LIGNEES_REMPLACEES, ObjType::List)?;
                tx.insert(&remplacees, 0, "01HF7YAT000000000000000002")?;
                Ok(())
            },
        ),
        "poser les lignées remplacées",
    );

    assert_eq!(lignee(fonds(&a)), "01HF7YAT000000000000000002");
    assert_eq!(lignee(fonds(&b)), "01HF7YAT00000G000000000002");
    assert!(
        fonds(&a)
            .get(ROOT, LIGNEES_REMPLACEES)
            .expect("lire les lignées remplacées d'A")
            .is_none(),
        "A porte des lignées remplacées"
    );
    match fonds(&b)
        .get(ROOT, LIGNEES_REMPLACEES)
        .expect("lire les lignées remplacées de B")
    {
        Some((Value::Object(ObjType::List), remplacees)) => {
            assert_eq!(fonds(&b).length(&remplacees), 1, "une lignée remplacée");
        }
        autre => panic!("B ne porte pas la liste des lignées remplacées : {autre:?}"),
    }
    assert_eq!(
        teneur(fonds(&a)).unwrap(),
        teneur(fonds(&b)).unwrap(),
        "deux documents qui ne diffèrent que par leur lignée ont des teneurs différentes"
    );
}
