//! La genèse et la teneur figées (§ 31 · 10, test 11) : les octets de la genèse et la teneur d'un
//! document témoin, engendré à l'exécution, à leurs empreintes de référence ; et deux documents
//! qui ne diffèrent que par leur lignée ont la même teneur (§ 31 · 2.6).
//!
//! Ce fichier est le seul des empreintes de référence. Le test ne lit aucune étiquette, que la
//! CI ne récupère pas.

mod commun;

use automerge::transaction::Transactable;
use automerge::{ObjType, ROOT, ReadDoc, ScalarValue, Value};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_jetable, ecrire, etiqueter, faite, noter, ouvrir,
};
use noyau::donnees::fonds::{GENESE, LIGNEE, LIGNEES_REMPLACEES};
use stockage::{Document, Naissance, RacineDesDonnees, empreinte, teneur};

/// Les empreintes de référence du test 11. Une ligne se change ou se retire avant la `v0.3.0`, et
/// la PR le dit ; dès elle, seule une ligne ajoutée passe (§ 31 · 10, test 11 ; ADR-023).
const GENESE_DU_FONDS: &str = "aa03e0cee230ee1aab90c1d53a6a00e51da250b8b5944b67560dd4f9b301c105";
const TENEUR_DU_TEMOIN: &str = "a349d135957809fcbf348db20cfb69a8752fe5a32b28296f7b8d9744900a58a4";

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

// § 31 · 10, test 11 : les octets de la genèse et la teneur d'un document témoin, à leurs références
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
