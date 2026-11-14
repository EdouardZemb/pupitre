//! La lecture du fonds (§ 31 · 3) : le fonds lu, que le noyau reçoit (`IdeeLue`, `CaptationLue`),
//! le journal, la version de schéma, les empreintes que nomme le document et sa teneur, tirés du
//! document sans y écrire. Une valeur se reconnaît à sa variante, jamais par conversion : une idée
//! dont une valeur n'a pas le type du schéma ne se lit pas. Un conteneur absent de la racine du
//! document se lit vide.

use std::collections::{BTreeMap, BTreeSet};

use automerge::{Automerge, ObjId, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use noyau::donnees::fonds::{
    CAPTEE, CONTENEURS_NOMMANT_UNE_CAPTATION, DEPUIS, DUREE, ETIQUETTES, IDEE, IDEES, IMPORTEE, LE,
    LIENS, PAR, PROVENANCE, RETIREE, TEXTE, VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes::{Appareil, CaptationLue, IdeeLue, Provenance, Sorte};
use noyau::donnees::idee;
use noyau::donnees::teneur::{ErreurDeTeneur, Valeur, ecrire_la_teneur};

/// L'idée `id`, telle qu'elle se lit : son identifiant, sa provenance (`provenance`, `par`, et
/// `depuis` d'une idée importée), sa date (`le`), son texte, ses étiquettes, la date de son
/// retrait et les clés des liens dont elle est un bout. `None` quand l'idée manque, ou qu'une de
/// ses valeurs n'a pas le type du schéma : une idée illisible n'est pas une idée.
pub fn lire_l_idee(doc: &Automerge, id: &str) -> Option<IdeeLue> {
    let idee = objet(doc, &objet(doc, &ROOT, IDEES)?, id)?;
    // `??` : le premier refuse une valeur d'un autre type, le second une valeur absente ; le
    // dernier `?`, un nom qu'aucun appareil ne porte.
    let par = Appareil::depuis_nom(&valeur(doc, &idee, PAR, chaine)??)?;
    let provenance = match valeur(doc, &idee, PROVENANCE, chaine)??.as_str() {
        CAPTEE => Provenance::Captee { par },
        IMPORTEE => Provenance::Importee {
            par,
            depuis: valeur(doc, &idee, DEPUIS, chaine)??,
        },
        _ => return None,
    };
    let le = valeur(doc, &idee, LE, entier)??;
    let texte = valeur(doc, &idee, TEXTE, chaine)?;
    let retiree = valeur(doc, &idee, RETIREE, entier)?;
    let etiquettes = match doc.get(&idee, ETIQUETTES) {
        Ok(None) => BTreeSet::new(),
        Ok(Some((Value::Object(ObjType::Map), etiquettes))) => doc.keys(&etiquettes).collect(),
        _ => return None,
    };
    Some(IdeeLue {
        id: id.to_string(),
        provenance,
        le,
        texte,
        etiquettes,
        retiree,
        liens: liens_de(doc, id),
    })
}

/// La captation `empreinte` de la sorte `sorte`, sous les clés `<empreinte>/idee` et, d'une prise,
/// `<empreinte>/duree`, dans le conteneur de la sorte. `None` quand le conteneur manque, ou que
/// `<empreinte>/idee` n'est pas une chaîne.
pub fn lire_la_captation(doc: &Automerge, sorte: Sorte, empreinte: &str) -> Option<CaptationLue> {
    let conteneur = objet(doc, &ROOT, sorte.conteneur())?;
    let idee = valeur(doc, &conteneur, &format!("{empreinte}/{IDEE}"), chaine)??;
    let duree = valeur(doc, &conteneur, &format!("{empreinte}/{DUREE}"), naturel).flatten();
    Some(captation(sorte, empreinte.to_string(), idee, duree))
}

/// Les captations dont `<empreinte>/idee` vaut `id`, dans les trois conteneurs, ordre (sorte,
/// empreinte). Chaque conteneur se parcourt par `map_range`, en un passage, chaque clé coupée au
/// premier `/` ; un conteneur absent de la racine du document se saute.
pub fn captations_de(doc: &Automerge, id: &str) -> Vec<CaptationLue> {
    let mut captations = Vec::new();
    for sorte in Sorte::TOUTES {
        let Some(conteneur) = objet(doc, &ROOT, sorte.conteneur()) else {
            continue;
        };
        // De chaque empreinte, son idée et sa durée, telles qu'elles se lisent.
        let mut faits: BTreeMap<String, (Option<String>, Option<u64>)> = BTreeMap::new();
        for element in doc.map_range(&conteneur, ..) {
            let Value::Scalar(simple) = element.value.into_value() else {
                continue;
            };
            let Some((empreinte, fait)) = element.key.split_once('/') else {
                continue;
            };
            let entree = faits.entry(empreinte.to_string()).or_default();
            match fait {
                IDEE => entree.0 = chaine(&simple),
                DUREE => entree.1 = naturel(&simple),
                _ => {}
            }
        }
        captations.extend(faits.into_iter().filter_map(|(empreinte, (idee, duree))| {
            let idee = idee.filter(|idee| idee == id)?;
            Some(captation(sorte, empreinte, idee, duree))
        }));
    }
    captations
}

/// Chaque idée qui se lit, retirée comprise, dans l'ordre des identifiants.
pub fn idees(doc: &Automerge) -> Vec<IdeeLue> {
    let Some(idees) = objet(doc, &ROOT, IDEES) else {
        return Vec::new();
    };
    doc.keys(&idees)
        .filter_map(|id| lire_l_idee(doc, &id))
        .collect()
}

/// (message, heure) de chaque modification, dans l'ordre de `get_changes(&[])` ; une modification
/// sans message en a un vide.
pub fn journal(doc: &Automerge) -> Vec<(String, i64)> {
    doc.get_changes(&[])
        .iter()
        .map(|modification| {
            (
                modification.message().unwrap_or_default().to_string(),
                modification.timestamp(),
            )
        })
        .collect()
}

/// La version de schéma que porte la racine du document (`VERSION_DE_SCHEMA`), un entier non signé
/// (`ScalarValue::Uint`) ; absente, ou de tout autre type, `None`.
pub fn version_de_schema(doc: &Automerge) -> Option<u64> {
    valeur(doc, &ROOT, VERSION_DE_SCHEMA, naturel).flatten()
}

/// Les empreintes que nomme le document (§ 31 · 6.1) : les clés `<empreinte>/<clé>` des
/// `CONTENEURS_NOMMANT_UNE_CAPTATION`, par `idee::empreintes_nommees` ; un conteneur absent de la
/// racine du document, ou qui n'est pas un objet à clés, se saute.
pub fn empreintes_nommees(doc: &Automerge) -> BTreeSet<String> {
    CONTENEURS_NOMMANT_UNE_CAPTATION
        .into_iter()
        .filter_map(|conteneur| objet(doc, &ROOT, conteneur))
        .flat_map(|conteneur| {
            let cles: Vec<String> = doc.keys(&conteneur).collect();
            idee::empreintes_nommees(cles.iter().map(String::as_str))
        })
        .collect()
}

/// La teneur de `doc` (§ 31 · 2.6) : son arbre, lu sur la valeur que présente Automerge
/// (`ReadDoc`, jamais `get_all`), puis écrit par le noyau, sans la lignée ni les lignées qu'elle
/// remplace.
pub fn teneur(doc: &Automerge) -> Result<String, ErreurDeTeneur> {
    ecrire_la_teneur(&objet_a_cles(doc, &ROOT))
}

/// Un objet `Map` ou `Table` : une seule sorte.
fn objet_a_cles(doc: &Automerge, objet: &ObjId) -> BTreeMap<String, Valeur> {
    doc.keys(objet)
        .map(|cle| {
            let (valeur, id) = doc
                .get(objet, cle.as_str())
                .expect("lire une clé que le document vient de nommer")
                .expect("une clé nommée a une valeur");
            let valeur = valeur_de(doc, valeur, &id);
            (cle, valeur)
        })
        .collect()
}

fn valeur_de(doc: &Automerge, valeur: Value<'_>, id: &ObjId) -> Valeur {
    match valeur {
        // En automerge 0.12.0, une `Table` n'est jamais indexée : `put` y rend `NotAnObject`, et
        // `keys` avale cette erreur. Une `Table` non vide, chargée d'un document d'une autre
        // implémentation, se lit donc vide, sans erreur.
        Value::Object(ObjType::Map | ObjType::Table) => Valeur::ObjetACles(objet_a_cles(doc, id)),
        Value::Object(ObjType::List) => Valeur::Liste(
            (0..doc.length(id))
                .map(|indice| {
                    let (valeur, id) = doc
                        .get(id, indice)
                        .expect("lire un indice que le document vient de compter")
                        .expect("un indice compté a une valeur");
                    valeur_de(doc, valeur, &id)
                })
                .collect(),
        ),
        Value::Object(ObjType::Text) => {
            Valeur::Texte(doc.text(id).expect("lire un objet texte du document"))
        }
        Value::Scalar(scalaire) => match scalaire.as_ref() {
            ScalarValue::Str(texte) => Valeur::Str(texte.to_string()),
            ScalarValue::Int(n) => Valeur::Int(*n),
            ScalarValue::Uint(n) => Valeur::Uint(*n),
            ScalarValue::F64(x) => Valeur::F64(*x),
            ScalarValue::Counter(compteur) => Valeur::Counter(i64::from(compteur)),
            ScalarValue::Timestamp(n) => Valeur::Timestamp(*n),
            ScalarValue::Boolean(b) => Valeur::Boolean(*b),
            ScalarValue::Bytes(octets) => Valeur::Bytes(octets.clone()),
            ScalarValue::Null => Valeur::Null,
            ScalarValue::Unknown { type_code, .. } => Valeur::Inconnu {
                type_code: *type_code,
            },
        },
    }
}

/// L'objet à clés que porte `cle` dans `parent` ; absent, ou de tout autre type, `None`.
fn objet(doc: &Automerge, parent: &ObjId, cle: &str) -> Option<ObjId> {
    match doc.get(parent, cle) {
        Ok(Some((Value::Object(ObjType::Map), id))) => Some(id),
        _ => None,
    }
}

/// La valeur que porte `cle` dans `objet`, telle que `typer` la reconnaît à sa variante :
/// `Some(None)` absente, `Some(Some(_))` du type attendu, `None` de tout autre type, un objet
/// compris.
fn valeur<T>(
    doc: &Automerge,
    objet: &ObjId,
    cle: &str,
    typer: fn(&ScalarValue) -> Option<T>,
) -> Option<Option<T>> {
    match doc.get(objet, cle) {
        Ok(None) => Some(None),
        Ok(Some((Value::Scalar(valeur), _))) => typer(&valeur).map(Some),
        Ok(Some((Value::Object(_), _))) | Err(_) => None,
    }
}

/// Une chaîne (`ScalarValue::Str`).
fn chaine(valeur: &ScalarValue) -> Option<String> {
    match valeur {
        ScalarValue::Str(texte) => Some(texte.to_string()),
        _ => None,
    }
}

/// Un entier signé (`ScalarValue::Int`).
fn entier(valeur: &ScalarValue) -> Option<i64> {
    match valeur {
        ScalarValue::Int(entier) => Some(*entier),
        _ => None,
    }
}

/// Un entier non signé (`ScalarValue::Uint`).
fn naturel(valeur: &ScalarValue) -> Option<u64> {
    match valeur {
        ScalarValue::Uint(naturel) => Some(*naturel),
        _ => None,
    }
}

/// La captation lue : sa durée, d'une prise seulement.
fn captation(sorte: Sorte, empreinte: String, idee: String, duree: Option<u64>) -> CaptationLue {
    CaptationLue {
        sorte,
        empreinte,
        idee,
        duree_ms: duree.filter(|_| sorte == Sorte::Prise),
    }
}

/// Les clés de `LIENS` dont un bout est `id` : `apparente:<a>:<b>`, coupée aux deux premiers `:`.
/// Sans liens à la racine du document, aucune.
fn liens_de(doc: &Automerge, id: &str) -> BTreeSet<String> {
    let Some(liens) = objet(doc, &ROOT, LIENS) else {
        return BTreeSet::new();
    };
    doc.keys(&liens)
        .filter(|cle| {
            cle.split_once(':')
                .and_then(|(_, bouts)| bouts.split_once(':'))
                .is_some_and(|(a, b)| a == id || b == id)
        })
        .collect()
}
