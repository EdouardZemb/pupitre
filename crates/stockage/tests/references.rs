//! La genèse et la teneur figées (§ 31 · 10, test 11) : les octets de la genèse et la teneur de
//! deux documents témoins, engendrés à l'exécution, à leurs empreintes de référence : l'un
//! ordinaire, l'autre qui porte une valeur de chaque type que la teneur écrit ; et deux documents
//! qui ne diffèrent que par leur lignée ont la même teneur (§ 31 · 2.6).
//!
//! Ce fichier est le seul des empreintes de référence. Le test ne lit aucune étiquette, que la
//! CI ne récupère pas.

mod commun;

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{Automerge, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_jetable, ecrire, etiqueter, faite, noter, ouvrir,
};
use noyau::donnees::fonds::{GENESE, LIGNEE, LIGNEES_REMPLACEES};
use stockage::{Document, Naissance, RacineDesDonnees, empreinte, teneur};

/// Les empreintes de référence du test 11. Une ligne se change ou se retire avant la `v0.3.0`, et
/// la PR le dit ; dès elle, seule une ligne ajoutée passe (§ 31 · 10, test 11 ; ADR-023).
const GENESE_DU_FONDS: &str = "aa03e0cee230ee1aab90c1d53a6a00e51da250b8b5944b67560dd4f9b301c105";
const TENEUR_DU_TEMOIN: &str = "a349d135957809fcbf348db20cfb69a8752fe5a32b28296f7b8d9744900a58a4";
const TENEUR_DES_TYPES: &str = "58791fdc7d27678b7177a492b6962ebbb989595b43aba79ce75bf912b39c9763";

/// Le document témoin, né dans `racine` sous `naissance` : la genèse, la naissance, puis, en un
/// seul geste, l'idée `idee-1` notée, « Un refrain qui monte, en ré », et étiquetée `refrain`. Il
/// naît sans avertissement.
fn faire_naitre_le_temoin<'r>(
    racine: &'r RacineDesDonnees,
    naissance: &Naissance<'_>,
) -> Document<'r> {
    let temoin = racine
        .faire_naitre_le_fonds(naissance, MESSAGE_DU_GESTE, HEURE, |tx| {
            noter(tx, "idee-1", "Un refrain qui monte, en ré")?;
            etiqueter(tx, "idee-1", "refrain")
        })
        .expect("faire naître le témoin");
    assert!(
        temoin.avertissements().is_empty(),
        "le témoin est né avec des avertissements : {:?}",
        temoin.avertissements()
    );
    temoin
}

/// Le second document témoin, bâti par Automerge, en mémoire : une valeur de chaque type que la
/// teneur écrit, à la racine du document. Une chaîne qui porte une barre et un guillemet ; un
/// compteur posé à 3, puis augmenté de 2 par une seconde modification ; un entier négatif ; une
/// liste de flottants, 0,1, 1e300, -0, l'infini et moins l'infini ; un instant négatif ; une
/// liste d'un entier non signé et d'une chaîne ; un objet à clés imbriqué, qui porte un booléen
/// et un objet à clés vide ; deux octets ; rien ; un objet `Text`.
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
    let valeurs = [0.1, 1e300, -0.0, f64::INFINITY, f64::NEG_INFINITY];
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

/// La lignée que porte `document`, à la racine du document.
fn lignee(document: &Document<'_>) -> String {
    match document
        .automerge()
        .get(ROOT, LIGNEE)
        .expect("lire la lignée")
    {
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
    let temoin = faire_naitre_le_temoin(&racine, &NAISSANCE);
    let teneur = teneur(temoin.automerge()).unwrap();
    // {"idees":{"idee-1":{"etiquettes":{"refrain":b1},"texte":s"Un refrain qui monte, en ré"}},
    //  "identifiant":s"fonds-1","prises":{},"proprietaire":s"proprietaire-1","version_de_schema":u1}
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
    let attendue = r#"{"chaine":s"a\\b\"c","compteur":c5,"entier":i-7,"flottants":[f0.1,f1e300,f-0,finf,f-inf],"instant":d-5,"liste":[u1,s"a"],"objet":{"faux":b0,"vide":{}},"octets":x00ab,"rien":n,"texte":t"abc"}"#;
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
    // Le témoin sous `NAISSANCE`, et sous `NAISSANCE` dont la lignée est `lignee-2`, une écriture
    // de plus y posant `LIGNEES_REMPLACEES = ["lignee-1"]` : deux teneurs égales.
    let d = dossier_jetable("references-lignee");
    let racine_a = ouvrir(&d.join("a"));
    let racine_b = ouvrir(&d.join("b"));
    let a = faire_naitre_le_temoin(&racine_a, &NAISSANCE);
    let mut b = faire_naitre_le_temoin(
        &racine_b,
        &Naissance {
            lignee: "lignee-2",
            ..NAISSANCE
        },
    );
    faite(
        ecrire(&mut b, |tx| {
            let remplacees = tx.put_object(ROOT, LIGNEES_REMPLACEES, ObjType::List)?;
            tx.insert(&remplacees, 0, "lignee-1")?;
            Ok(())
        }),
        "poser les lignées remplacées",
    );

    assert_eq!(lignee(&a), "lignee-1");
    assert_eq!(lignee(&b), "lignee-2");
    assert!(
        a.automerge()
            .get(ROOT, LIGNEES_REMPLACEES)
            .expect("lire les lignées remplacées d'A")
            .is_none(),
        "A porte des lignées remplacées"
    );
    match b
        .automerge()
        .get(ROOT, LIGNEES_REMPLACEES)
        .expect("lire les lignées remplacées de B")
    {
        Some((Value::Object(ObjType::List), remplacees)) => {
            assert_eq!(b.automerge().length(&remplacees), 1, "une lignée remplacée");
        }
        autre => panic!("B ne porte pas la liste des lignées remplacées : {autre:?}"),
    }
    assert_eq!(
        teneur(a.automerge()).unwrap(),
        teneur(b.automerge()).unwrap(),
        "deux documents qui ne diffèrent que par leur lignée ont des teneurs différentes"
    );
}
