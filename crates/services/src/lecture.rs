//! La lecture du fonds (§ 31 · 3) : le fonds lu, que le noyau reçoit (`IdeeLue`, `CaptationLue`,
//! les clés d'une requête enregistrée), idée par idée ou en un seul passage pour la boîte, les
//! requêtes enregistrées, le journal, la version de schéma, les empreintes que nomme le document et
//! sa teneur, tirés du document sans y écrire. Une valeur se reconnaît à sa variante, jamais par
//! conversion : une idée dont une valeur n'a pas le type du schéma ne se lit pas. Un conteneur
//! absent de la racine du document se lit vide.
//!
//! Ce que l'import écrit se lit ici aussi (§ 31 · 2 ; questions 9, 18, 20 à 22) : les noms
//! d'origine d'un contenu et son premier nom vu, ses origines déclarées, ce que le fonds porte déjà
//! d'un contenu ou d'une prise en stems, les noms qu'a posés un import, les stems d'une prise, les
//! imports, la tête de l'ouverture de chacun, leurs déclarations et les générateurs ; et, pour le
//! protocole des captations, le format d'une prise et celui de chaque stem. Les clés
//! d'une captation sont à plat, `<empreinte>/<clé>` : chacune se coupe à sa place (`lire_la_cle`),
//! et le chemin d'un nom garde ses `/`. Une clé marque, à `true` (l'entrée d'un stem,
//! `manifeste`, le lien d'un stem à son MIDI, un jeu déclaré, une origine), se lit à sa seule
//! présence.

use std::collections::{BTreeMap, BTreeSet};

use automerge::iter::MapRangeItem;
use automerge::{Automerge, ChangeHash, ObjId, ObjType, ROOT, ReadDoc, ScalarValue, Value};
use noyau::donnees::changement::Simple;
use noyau::donnees::fonds::{
    BITS, CANAUX, CAPTEE, CODAGE, CONTENEURS_NOMMANT_UNE_CAPTATION, DECLARATION, DEPUIS, DIVISION,
    DUREE, ETIQUETTES, FORMAT, FREQUENCE, GENEREE, IDEE, IDEES, IMPORTEE, IMPORTS, JEU, LE, LIENS,
    MAIN, MANIFESTE, MIDI, MIDI_VERSES, NOM, ORIGINES_DECLAREES, OUVERT, PAR, PHOTOS, PRISES,
    PROVENANCE, RAPPORT, REQUETES_ENREGISTREES, RETIREE, SAISIE, STEM, TEXTE, TYPE,
    VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes::{Appareil, CaptationLue, IdeeLue, Provenance, Sorte};
use noyau::donnees::idee;
use noyau::donnees::import::{DejaDeLaPrise, DejaLu, Faits};
use noyau::donnees::origine::{self, Ajout, Declaration, OriginesLues};
use noyau::donnees::requete::{RequeteEnregistreeLue, lire_une_requete};
use noyau::donnees::stems::{TypeDeStem, duree_de_la_prise_en_stems};
use noyau::donnees::teneur::{ErreurDeTeneur, Valeur, ecrire_la_teneur};

/// L'idée `id`, telle qu'elle se lit : son identifiant, sa provenance (`provenance`, `par`, et
/// `depuis` d'une idée importée), sa date (`le`), son texte, ses étiquettes, la date de son
/// retrait et les clés des liens dont elle est un bout. `None` quand l'idée manque, ou qu'elle est
/// illisible (`idee_de`) : une idée illisible ne se lit pas, `idees_et_illisibles` la compte à
/// part.
pub fn lire_l_idee(doc: &Automerge, id: &str) -> Option<IdeeLue> {
    let idee = objet(doc, &objet(doc, &ROOT, IDEES)?, id)?;
    idee_de(doc, &idee, id, liens_de(doc, id))
}

/// L'idée `id`, telle qu'elle se lit de son objet `idee`, comme `lire_l_idee`, avec `liens`, les
/// clés des liens dont elle est un bout. `None` quand l'une de ses valeurs manque (`par`,
/// `provenance`, `le`, et `depuis` d'une idée importée), n'a pas le type du schéma, ou sort des
/// valeurs qu'il connaît : un `par` qu'aucun appareil ne porte, une `provenance` ni `captee` ni
/// `importee`.
fn idee_de(doc: &Automerge, idee: &ObjId, id: &str, liens: BTreeSet<String>) -> Option<IdeeLue> {
    // `??` : le premier refuse une valeur d'un autre type, le second une valeur absente ; le
    // dernier `?`, un nom qu'aucun appareil ne porte.
    let par = Appareil::depuis_nom(&valeur(doc, idee, PAR, chaine)??)?;
    let provenance = match valeur(doc, idee, PROVENANCE, chaine)??.as_str() {
        CAPTEE => Provenance::Captee { par },
        IMPORTEE => Provenance::Importee {
            par,
            depuis: valeur(doc, idee, DEPUIS, chaine)??,
        },
        _ => return None,
    };
    let le = valeur(doc, idee, LE, entier)??;
    let texte = valeur(doc, idee, TEXTE, chaine)?;
    let retiree = valeur(doc, idee, RETIREE, entier)?;
    let etiquettes = match doc.get(idee, ETIQUETTES) {
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
        liens,
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

/// Les faits que porte le contenu `empreinte` sous `conteneur`, chacun sous `<empreinte>/<fait>` :
/// son format, sa durée, son codage, ses canaux, sa fréquence, ses bits et sa division. Un fait
/// absent, d'un autre type que celui du schéma, ou qui ne tient pas dans le sien, se lit `None` ;
/// un conteneur absent, aucun. La durée d'une prise en stems n'est pas sous son manifeste : elle
/// vient de ses stems (`captations_de`).
pub fn faits_de(doc: &Automerge, conteneur: &str, empreinte: &str) -> Faits {
    let Some(objet) = objet(doc, &ROOT, conteneur) else {
        return Faits::default();
    };
    let naturel_de =
        |fait: &str| valeur(doc, &objet, &format!("{empreinte}/{fait}"), naturel).flatten();
    Faits {
        format: valeur(doc, &objet, &format!("{empreinte}/{FORMAT}"), chaine).flatten(),
        duree_ms: naturel_de(DUREE),
        codage: naturel_de(CODAGE).and_then(|n| u16::try_from(n).ok()),
        canaux: naturel_de(CANAUX).and_then(|n| u16::try_from(n).ok()),
        frequence: naturel_de(FREQUENCE).and_then(|n| u32::try_from(n).ok()),
        bits: naturel_de(BITS).and_then(|n| u16::try_from(n).ok()),
        division: naturel_de(DIVISION).and_then(|n| u16::try_from(n).ok()),
    }
}

/// La tête de l'ouverture de l'import `import` : le hash de la modification qui a posé
/// `imports/<i>/ouvert`, et de chacune, si la clé est en conflit (`get_all`, puis
/// `Automerge::hash_for_opid`, d'après l'identifiant de chaque valeur). Aucun si la clé ne se lit
/// pas. Une reprise y lit, comme l'ouverture de l'import, les captations connues de son rapport.
pub fn tete_de_l_ouverture(doc: &Automerge, import: &str) -> Vec<ChangeHash> {
    let Some(imports) = objet(doc, &ROOT, IMPORTS) else {
        return Vec::new();
    };
    doc.get_all(&imports, format!("{import}/{OUVERT}").as_str())
        .map(|valeurs| {
            valeurs
                .iter()
                .filter_map(|(_, id)| doc.hash_for_opid(id))
                .collect()
        })
        .unwrap_or_default()
}

/// Les captations dont `<empreinte>/idee` vaut `id`, dans les trois conteneurs, ordre (sorte,
/// empreinte). Chaque conteneur se parcourt par `map_range`, en un passage, chaque clé coupée au
/// premier `/` ; un conteneur absent de la racine du document se saute.
pub fn captations_de(doc: &Automerge, id: &str) -> Vec<CaptationLue> {
    let mut captations = Vec::new();
    for sorte in Sorte::TOUTES {
        captations.extend(faits_de_la_sorte(doc, sorte).into_iter().filter_map(
            |(empreinte, (idee, duree))| {
                let idee = idee.filter(|idee| idee == id)?;
                Some(captation(sorte, empreinte, idee, duree))
            },
        ));
    }
    captations
}

/// Les captations de chaque idée, en un seul passage sur les trois conteneurs : sous
/// l'identifiant que porte `<empreinte>/idee`, ses captations dans l'ordre (sorte, empreinte),
/// comme `captations_de`. Une empreinte dont `<empreinte>/idee` n'est pas une chaîne n'est la
/// captation d'aucune idée ; un conteneur absent de la racine du document se saute.
pub fn captations_par_idee(doc: &Automerge) -> BTreeMap<String, Vec<CaptationLue>> {
    let mut par_idee: BTreeMap<String, Vec<CaptationLue>> = BTreeMap::new();
    for sorte in Sorte::TOUTES {
        for (empreinte, (idee, duree)) in faits_de_la_sorte(doc, sorte) {
            if let Some(idee) = idee {
                par_idee
                    .entry(idee.clone())
                    .or_default()
                    .push(captation(sorte, empreinte, idee, duree));
            }
        }
    }
    par_idee
}

/// Les idées que nomme une clé `<empreinte>/idee` du conteneur `photos`, sa valeur une chaîne
/// (vide au lot 1) ; un conteneur absent de la racine du document, aucune.
pub fn idees_avec_une_photo(doc: &Automerge) -> BTreeSet<String> {
    let Some(photos) = objet(doc, &ROOT, PHOTOS) else {
        return BTreeSet::new();
    };
    doc.map_range(&photos, ..)
        .filter_map(|element| {
            let Value::Scalar(simple) = element.value.into_value() else {
                return None;
            };
            let (_, fait) = element.key.split_once('/')?;
            if fait == IDEE { chaine(&simple) } else { None }
        })
        .collect()
}

/// Chaque idée qui se lit, retirée comprise, dans l'ordre des identifiants
/// (`idees_et_illisibles`).
pub fn idees(doc: &Automerge) -> Vec<IdeeLue> {
    idees_et_illisibles(doc).0
}

/// Chaque idée qui se lit, retirée comprise, dans l'ordre des identifiants, et le nombre de
/// celles qui ne se lisent pas : les clés d'`idees` dont la valeur n'est pas un objet à clés, ou
/// qu'`idee_de` ne lit pas. Les liens se lisent en un seul passage sur `liens`, rangés par bout,
/// et non par une recherche pour chaque idée : sur deux mille idées, la taille visée du fonds,
/// chacune parcourrait toutes les clés des liens. Sans `idees` à la racine du document, aucune,
/// et aucune illisible.
pub fn idees_et_illisibles(doc: &Automerge) -> (Vec<IdeeLue>, usize) {
    let Some(idees) = objet(doc, &ROOT, IDEES) else {
        return (Vec::new(), 0);
    };
    let mut liens = liens_par_bout(doc);
    let mut lues = Vec::new();
    let mut illisibles = 0;
    for id in doc.keys(&idees) {
        let liens_de_l_idee = liens.remove(&id).unwrap_or_default();
        match objet(doc, &idees, &id).and_then(|idee| idee_de(doc, &idee, &id, liens_de_l_idee)) {
            Some(lue) => lues.push(lue),
            None => illisibles += 1,
        }
    }
    (lues, illisibles)
}

/// Chaque requête enregistrée de `doc` qui a un nom (`lire_une_requete`), dans l'ordre des clés de
/// `requetes_enregistrees` : de chaque objet à clés qu'il porte, chaque clé et sa valeur simple
/// (`simple_du_noyau`), `None` pour toute autre valeur, un objet compris. Une clé dont la valeur
/// n'est pas un objet à clés n'est la requête enregistrée de personne ; sans
/// `requetes_enregistrees` à la racine du document, aucune.
pub fn requetes_enregistrees(doc: &Automerge) -> Vec<RequeteEnregistreeLue> {
    let Some(conteneur) = objet(doc, &ROOT, REQUETES_ENREGISTREES) else {
        return Vec::new();
    };
    doc.keys(&conteneur)
        .filter_map(|id| {
            let requete = objet(doc, &conteneur, &id)?;
            let champs: BTreeMap<String, Option<Simple>> = doc
                .keys(&requete)
                .map(|cle| {
                    let valeur = match doc.get(&requete, cle.as_str()) {
                        Ok(Some((Value::Scalar(valeur), _))) => simple_du_noyau(&valeur),
                        _ => None,
                    };
                    (cle, valeur)
                })
                .collect();
            lire_une_requete(&id, &champs)
        })
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
/// `CONTENEURS_NOMMANT_UNE_CAPTATION`, et chaque valeur de `imports/<i>/rapport`, celles d'un
/// conflit comprises, par `idee::empreintes_nommees` ; un conteneur absent de la racine du
/// document, ou qui n'est pas un objet à clés, se saute.
pub fn empreintes_nommees(doc: &Automerge) -> BTreeSet<String> {
    nommees_selon(Lecture::courante(doc))
}

/// Les empreintes nommées à la tête `tete` (`keys_at`, `map_range_at`, `get_all_at`), comme
/// `empreintes_nommees` : les captations connues à l'ouverture d'un import.
pub fn empreintes_nommees_a(doc: &Automerge, tete: &[ChangeHash]) -> BTreeSet<String> {
    nommees_selon(Lecture {
        doc,
        tete: Some(tete),
    })
}

/// Les empreintes nommées, selon `lecture` (`empreintes_nommees`).
fn nommees_selon(lecture: Lecture<'_>) -> BTreeSet<String> {
    let mut rapports = Vec::new();
    if let Some(imports) = lecture.conteneur(IMPORTS) {
        for element in lecture.sous(&imports, "") {
            let valeur = element.value.into_value();
            let cle: &str = &element.key;
            if cle.split_once('/').is_some_and(|(_, fait)| fait == RAPPORT) {
                rapports.extend(chaines(lecture, &imports, cle, valeur, element.conflict));
            }
        }
    }
    let mut nommees = idee::empreintes_nommees(
        std::iter::empty::<&str>(),
        rapports.iter().map(String::as_str),
    );
    for conteneur in CONTENEURS_NOMMANT_UNE_CAPTATION
        .into_iter()
        .filter_map(|conteneur| lecture.conteneur(conteneur))
    {
        let cles = lecture.cles(&conteneur);
        nommees.extend(idee::empreintes_nommees(
            cles.iter().map(String::as_str),
            [],
        ));
    }
    nommees
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

/// La valeur simple du noyau que porte `valeur`, à sa variante : une chaîne (`Str`) en `Chaine`, un
/// entier signé (`Int`) en `Entier`, un entier non signé (`Uint`) en `Naturel`, `true`
/// (`Boolean(true)`) en `Vrai` ; toute autre, `None`.
fn simple_du_noyau(valeur: &ScalarValue) -> Option<Simple> {
    match valeur {
        ScalarValue::Str(texte) => Some(Simple::Chaine(texte.to_string())),
        ScalarValue::Int(entier) => Some(Simple::Entier(*entier)),
        ScalarValue::Uint(naturel) => Some(Simple::Naturel(*naturel)),
        ScalarValue::Boolean(true) => Some(Simple::Vrai),
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

/// Les clés de `LIENS` qui sont celles d'un lien dont un bout est `id`, selon la règle du noyau
/// (`idee::bouts_du_lien`) : une clé d'une autre forme n'est le lien d'aucune idée. Sans liens à
/// la racine du document, aucune.
fn liens_de(doc: &Automerge, id: &str) -> BTreeSet<String> {
    let Some(liens) = objet(doc, &ROOT, LIENS) else {
        return BTreeSet::new();
    };
    doc.keys(&liens)
        .filter(|cle| idee::bouts_du_lien(cle).is_some_and(|(a, b)| a == id || b == id))
        .collect()
}

/// Les clés de `LIENS`, rangées sous chacun de leurs deux bouts, en un seul passage, selon la même
/// règle que `liens_de` : sous un identifiant, les clés que `liens_de` rendrait pour lui. Sans
/// liens à la racine du document, aucune.
fn liens_par_bout(doc: &Automerge) -> BTreeMap<String, BTreeSet<String>> {
    let mut par_bout: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let Some(liens) = objet(doc, &ROOT, LIENS) else {
        return par_bout;
    };
    for cle in doc.keys(&liens) {
        if let Some((a, b)) = idee::bouts_du_lien(&cle) {
            for bout in [a, b] {
                par_bout
                    .entry(bout.to_string())
                    .or_default()
                    .insert(cle.clone());
            }
        }
    }
    par_bout
}

/// De chaque empreinte du conteneur de `sorte`, son idée et sa durée, telles qu'elles se lisent :
/// le conteneur se parcourt par `map_range`, en un passage, chaque clé coupée à sa place
/// (`lire_la_cle`), et une valeur qui n'est pas simple se saute. Une empreinte sans `idee`, le
/// MIDI d'un stem sous `midi_verses`, n'est la captation d'aucune idée (question 21). D'une prise
/// en stems (`<p>/manifeste`), la durée est celle de ses stems (`duree_de_la_prise_en_stems`),
/// énumérés par leur entrée `<p>/stem/<s>`, jamais par leurs clés `…/duree` : un stem sans
/// `duree` la rend `None` (balayage Q4). Un conteneur absent de la racine du document, aucune.
fn faits_de_la_sorte(
    doc: &Automerge,
    sorte: Sorte,
) -> BTreeMap<String, (Option<String>, Option<u64>)> {
    let mut faits: BTreeMap<String, (Option<String>, Option<u64>)> = BTreeMap::new();
    let Some(conteneur) = objet(doc, &ROOT, sorte.conteneur()) else {
        return faits;
    };
    let mut manifestes: BTreeSet<String> = BTreeSet::new();
    let mut entrees: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut durees_des_stems: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for element in doc.map_range(&conteneur, ..) {
        let Value::Scalar(simple) = element.value.into_value() else {
            continue;
        };
        match lire_la_cle(&element.key) {
            CleDeCaptation::Fait { empreinte, fait } => {
                let entree = faits.entry(empreinte.to_string()).or_default();
                match fait {
                    IDEE => entree.0 = chaine(&simple),
                    DUREE => entree.1 = naturel(&simple),
                    MANIFESTE => {
                        manifestes.insert(empreinte.to_string());
                    }
                    _ => {}
                }
            }
            CleDeCaptation::EntreeDuStem { prise, stem } => {
                entrees
                    .entry(prise.to_string())
                    .or_default()
                    .insert(stem.to_string());
            }
            CleDeCaptation::FaitDuStem {
                prise,
                stem,
                fait: DUREE,
            } => {
                if let Some(duree) = naturel(&simple) {
                    durees_des_stems
                        .entry(prise.to_string())
                        .or_default()
                        .insert(stem.to_string(), duree);
                }
            }
            _ => {}
        }
    }
    for prise in manifestes {
        let durees = durees_des_stems.get(&prise);
        let des_stems: Vec<Option<u64>> = match entrees.get(&prise) {
            Some(stems) => stems
                .iter()
                .map(|stem| durees.and_then(|durees| durees.get(stem)).copied())
                .collect(),
            None => Vec::new(),
        };
        faits.entry(prise).or_default().1 = duree_de_la_prise_en_stems(&des_stems);
    }
    faits
}

/// Une clé à plat d'un conteneur de captations, coupée à sa place : le chemin d'un nom garde ses
/// `/`, et un segment ne se lit jamais dans un nom. La prise d'un nom ou d'un MIDI de stem ne
/// s'y garde pas : aucune lecture ne s'en sert, chacune parcourant une prise à la fois, ou toutes.
#[derive(Clone, Copy, Debug)]
enum CleDeCaptation<'c> {
    /// `<empreinte>/<fait>`, le fait sans `/` : `idee`, `format`, `duree`, `manifeste`…
    Fait { empreinte: &'c str, fait: &'c str },
    /// `<empreinte>/nom/<chemin>` : un nom d'origine.
    Nom { empreinte: &'c str, chemin: &'c str },
    /// `<p>/stem/<s>` : l'entrée du stem `<s>` dans la prise en stems `<p>`.
    EntreeDuStem { prise: &'c str, stem: &'c str },
    /// `<p>/stem/<s>/nom/<chemin>` : un nom d'origine du stem.
    NomDuStem { stem: &'c str, chemin: &'c str },
    /// `<p>/stem/<s>/midi/<m>` : le MIDI `<m>` qui accompagne le stem.
    MidiDuStem { stem: &'c str, midi: &'c str },
    /// `<p>/stem/<s>/<fait>`, hors d'un nom et d'un MIDI : `format`, `duree`, `type`,
    /// `type/provenance`…
    FaitDuStem {
        prise: &'c str,
        stem: &'c str,
        fait: &'c str,
    },
    /// Toute autre forme, une clé sans `/` comprise.
    Autre,
}

/// La clé `cle` d'un conteneur de captations, coupée à sa place (`CleDeCaptation`) : son premier
/// segment, l'empreinte ; puis, si le deuxième vaut `nom`, le reste est un chemin ; s'il vaut
/// `stem`, le troisième est le stem, et ce qui le suit, s'il vaut `nom/<chemin>`, `midi/<m>` ou un
/// autre fait, se coupe de même.
fn lire_la_cle(cle: &str) -> CleDeCaptation<'_> {
    let mut segments = cle.splitn(3, '/');
    let (Some(empreinte), Some(deuxieme)) = (segments.next(), segments.next()) else {
        return CleDeCaptation::Autre;
    };
    match (deuxieme, segments.next()) {
        (fait, None) => CleDeCaptation::Fait { empreinte, fait },
        (NOM, Some(chemin)) => CleDeCaptation::Nom { empreinte, chemin },
        (STEM, Some(sous_la_prise)) => {
            let prise = empreinte;
            let Some((stem, fait)) = sous_la_prise.split_once('/') else {
                return CleDeCaptation::EntreeDuStem {
                    prise,
                    stem: sous_la_prise,
                };
            };
            match fait.split_once('/') {
                Some((NOM, chemin)) => CleDeCaptation::NomDuStem { stem, chemin },
                Some((MIDI, midi)) => CleDeCaptation::MidiDuStem { stem, midi },
                _ => CleDeCaptation::FaitDuStem { prise, stem, fait },
            }
        }
        _ => CleDeCaptation::Autre,
    }
}

/// Une lecture du document : à sa tête courante (`tete` à `None`), ou à la tête `tete`, chaque
/// appel d'Automerge par sa forme `_at` (`get_at`, `keys_at`, `map_range_at`, `get_all_at`).
#[derive(Clone, Copy)]
struct Lecture<'d> {
    doc: &'d Automerge,
    tete: Option<&'d [ChangeHash]>,
}

impl<'d> Lecture<'d> {
    /// La lecture de `doc` à sa tête courante.
    fn courante(doc: &'d Automerge) -> Self {
        Lecture { doc, tete: None }
    }

    /// L'objet à clés que porte `cle` à la racine du document ; absent, ou de tout autre type,
    /// `None`.
    fn conteneur(self, cle: &str) -> Option<ObjId> {
        let lu = match self.tete {
            None => self.doc.get(ROOT, cle),
            Some(tete) => self.doc.get_at(ROOT, cle, tete),
        };
        match lu {
            Ok(Some((Value::Object(ObjType::Map), id))) => Some(id),
            _ => None,
        }
    }

    /// Les clés de `objet`, dans leur ordre.
    fn cles(self, objet: &ObjId) -> Vec<String> {
        match self.tete {
            None => self.doc.keys(objet).collect(),
            Some(tete) => self.doc.keys_at(objet, tete).collect(),
        }
    }

    /// Les éléments de `objet` dont la clé commence par `prefixe`, dans l'ordre des clés : un seul
    /// passage de `map_range`, depuis `prefixe`, arrêté à la première clé qui ne le porte plus ;
    /// le préfixe vide les prend tous.
    fn sous(
        self,
        objet: &ObjId,
        prefixe: &str,
    ) -> impl Iterator<Item = MapRangeItem<'d>> + use<'d> {
        let depuis = prefixe.to_string();
        let elements = match self.tete {
            None => self.doc.map_range(objet, depuis..),
            Some(tete) => self.doc.map_range_at(objet, depuis.., tete),
        };
        let prefixe = prefixe.to_string();
        elements.take_while(move |element| element.key.starts_with(prefixe.as_str()))
    }

    /// Chaque valeur de la clé `cle` de `objet`, celles d'un conflit comprises (`get_all`) ; une
    /// clé qui ne se lit pas n'en a aucune.
    fn toutes(self, objet: &ObjId, cle: &str) -> Vec<Value<'d>> {
        let lues = match self.tete {
            None => self.doc.get_all(objet, cle),
            Some(tete) => self.doc.get_all_at(objet, cle, tete),
        };
        lues.map(|valeurs| valeurs.into_iter().map(|(valeur, _)| valeur).collect())
            .unwrap_or_default()
    }
}

/// Les chaînes de la clé `cle` de `objet`, de valeur `valeur` : elle seule, ou, en conflit
/// (`conflit`), chacune de ses valeurs (`Lecture::toutes`) ; une valeur d'un autre type se saute.
fn chaines(
    lecture: Lecture<'_>,
    objet: &ObjId,
    cle: &str,
    valeur: Value<'_>,
    conflit: bool,
) -> Vec<String> {
    if conflit {
        lecture
            .toutes(objet, cle)
            .iter()
            .filter_map(|valeur| simple_de(valeur, chaine))
            .collect()
    } else {
        simple_de(&valeur, chaine).into_iter().collect()
    }
}

/// La valeur simple que porte `valeur`, telle que `typer` la reconnaît à sa variante ; un objet,
/// ou une valeur d'un autre type, `None`.
fn simple_de<T>(valeur: &Value<'_>, typer: fn(&ScalarValue) -> Option<T>) -> Option<T> {
    match valeur {
        Value::Scalar(simple) => typer(simple),
        Value::Object(_) => None,
    }
}

/// Les noms d'origine du contenu `empreinte` sous `conteneur` (ou, `stem` donné, sous l'entrée de
/// ce stem dans la prise `empreinte`) : chaque couple (chemin, ULID), chaque valeur d'une clé lue
/// par `get_all` (question 20), dans l'ordre des chemins. Un conteneur absent, aucun.
pub fn noms_de(
    doc: &Automerge,
    conteneur: &str,
    empreinte: &str,
    stem: Option<&str>,
) -> Vec<(String, String)> {
    let lecture = Lecture::courante(doc);
    let Some(objet) = lecture.conteneur(conteneur) else {
        return Vec::new();
    };
    let prefixe = match stem {
        None => format!("{empreinte}/{NOM}/"),
        Some(stem) => format!("{empreinte}/{STEM}/{stem}/{NOM}/"),
    };
    let mut noms = Vec::new();
    for element in lecture.sous(&objet, &prefixe) {
        let valeur = element.value.into_value();
        let cle: &str = &element.key;
        let chemin = &cle[prefixe.len()..];
        for ulid in chaines(lecture, &objet, cle, valeur, element.conflict) {
            noms.push((chemin.to_string(), ulid));
        }
    }
    noms
}

/// Le premier nom vu d'un contenu, d'après `origine::premier_nom_vu` (question 20), parmi tous ses
/// noms : sous les trois conteneurs (`noms_de`), puis sous l'entrée de chaque stem
/// (`<p>/stem/<empreinte>/nom/<chemin>`, cherchée dans toutes les prises) ; aucun nom, `None`.
/// C'est `premiers_noms_vus` d'un seul contenu.
pub fn premier_nom_vu(doc: &Automerge, empreinte: &str) -> Option<String> {
    premiers_noms_vus(doc, &BTreeSet::from([empreinte.to_string()])).remove(empreinte)
}

/// Le premier nom vu de chaque contenu de `empreintes`, comme `premier_nom_vu`, en un seul passage
/// sur `prises` pour les noms de leurs stems, quel que soit leur nombre : la fenêtre de la boîte
/// d'un fonds juste importé, dont aucune idée n'a de texte, en nomme chaque ligne sans parcourir
/// `prises` une fois par ligne (C1 de la revue de la PR (b)). Les noms sous les trois conteneurs
/// se lisent par `noms_de`, chacun depuis son préfixe ; un contenu sans aucun nom n'a pas
/// d'entrée.
pub fn premiers_noms_vus(
    doc: &Automerge,
    empreintes: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    let lecture = Lecture::courante(doc);
    let mut noms: BTreeMap<&str, Vec<(String, String)>> = empreintes
        .iter()
        .map(|empreinte| {
            let siens = CONTENEURS_NOMMANT_UNE_CAPTATION
                .into_iter()
                .flat_map(|conteneur| noms_de(doc, conteneur, empreinte, None))
                .collect();
            (empreinte.as_str(), siens)
        })
        .collect();
    if let Some(prises) = lecture.conteneur(PRISES) {
        for element in lecture.sous(&prises, "") {
            let cle: &str = &element.key;
            if let CleDeCaptation::NomDuStem { stem, chemin } = lire_la_cle(cle)
                && let Some(siens) = noms.get_mut(stem)
            {
                let valeur = element.value.into_value();
                for ulid in chaines(lecture, &prises, cle, valeur, element.conflict) {
                    siens.push((chemin.to_string(), ulid));
                }
            }
        }
    }
    noms.into_iter()
        .filter_map(|(empreinte, siens)| {
            let premier = origine::premier_nom_vu(
                siens
                    .iter()
                    .map(|(chemin, ulid)| (chemin.as_str(), ulid.as_str())),
            )?;
            Some((empreinte.to_string(), premier.to_string()))
        })
        .collect()
}

/// L'ensemble des origines du contenu `empreinte`, sous `origines_declarees` : `<e>/main`, et
/// chaque `<e>/generee/<nom>`, le nom non vide.
pub fn origines_de(doc: &Automerge, empreinte: &str) -> OriginesLues {
    origines_selon(Lecture::courante(doc), empreinte)
}

/// La même lecture qu'`origines_de`, à la tête `tete` (`get_at`, `map_range_at`).
pub fn origines_a(doc: &Automerge, empreinte: &str, tete: &[ChangeHash]) -> OriginesLues {
    origines_selon(
        Lecture {
            doc,
            tete: Some(tete),
        },
        empreinte,
    )
}

/// Les origines du contenu `empreinte`, selon `lecture` (`origines_de`).
fn origines_selon(lecture: Lecture<'_>, empreinte: &str) -> OriginesLues {
    let mut lues = OriginesLues::default();
    let Some(origines) = lecture.conteneur(ORIGINES_DECLAREES) else {
        return lues;
    };
    let prefixe = format!("{empreinte}/");
    for element in lecture.sous(&origines, &prefixe) {
        let element_d_origine = &element.key[prefixe.len()..];
        if element_d_origine == MAIN {
            lues.main = true;
        } else if let Some(nom) = nom_genere(element_d_origine) {
            lues.generees.insert(nom.to_string());
        }
    }
    lues
}

/// Le nom de `generee/<nom>`, s'il n'est pas vide.
fn nom_genere(element_d_origine: &str) -> Option<&str> {
    element_d_origine
        .strip_prefix(GENEREE)?
        .strip_prefix('/')
        .filter(|nom| !nom.is_empty())
}

/// Ce que le fonds porte du contenu `empreinte` : nommé par le fonds, d'après `nommees`, les
/// empreintes nommées que l'état de l'import lit une fois à l'ouverture et tient à jour à chaque
/// versement (R26) ; ses noms sous le conteneur de `sorte` (les chemins de `noms_de`) ; ses
/// origines (`origines_de`).
pub fn deja_lu(
    doc: &Automerge,
    sorte: Sorte,
    empreinte: &str,
    nommees: &BTreeSet<String>,
) -> DejaLu {
    DejaLu {
        captation: nommees.contains(empreinte),
        noms: chemins_des_noms(doc, sorte.conteneur(), empreinte),
        origines: origines_de(doc, empreinte),
    }
}

/// Les noms qu'a posés un import (R2, R7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NomsPoses {
    /// Les noms de fichiers que l'import a posés, chacun sous son chemin : l'empreinte du contenu,
    /// et vrai s'il est illisible, aucun `format` ne se lisant pour ce contenu dans son conteneur,
    /// ni, sous `prises`, sous l'entrée d'un stem (R7) ; un fichier versé seul et le stem d'un jeu
    /// du même contenu ont ainsi le même sort.
    pub fichiers: BTreeMap<String, (String, bool)>,
    /// Les noms de ses prises en stems (R2) : le chemin de leur dossier, ou le dernier segment
    /// d'un jeu versé seul.
    pub jeux: BTreeSet<String>,
}

/// Les noms qu'a posés l'import `import`, ceux dont une valeur est son ULID, celles d'un conflit
/// comprises (question 20) : ceux des fichiers, `<e>/nom/<chemin>` sous `prises` (hors d'une prise
/// marquée `manifeste`), `midi_verses` et `textes_verses`, et `<p>/stem/<s>/nom/<chemin>` ; et, à
/// part, ceux de ses prises en stems (R2). Chaque nom de fichier va avec son empreinte, et si sa
/// captation n'a pas de `format` (un illisible, R7 ; `NomsPoses::fichiers`). L'entrée
/// `<p>/stem/<s>`, à `true`, n'est pas un nom : elle se saute. Chaque conteneur se parcourt une
/// fois.
pub fn noms_poses_par(doc: &Automerge, import: &str) -> NomsPoses {
    let lecture = Lecture::courante(doc);
    let mut poses = NomsPoses::default();
    for sorte in Sorte::TOUTES {
        let Some(objet) = lecture.conteneur(sorte.conteneur()) else {
            continue;
        };
        let mut lisibles: BTreeSet<String> = BTreeSet::new();
        let mut manifestes: BTreeSet<String> = BTreeSet::new();
        let mut noms: Vec<(String, String)> = Vec::new();
        for element in lecture.sous(&objet, "") {
            let valeur = element.value.into_value();
            let cle: &str = &element.key;
            let (empreinte, chemin) = match lire_la_cle(cle) {
                CleDeCaptation::Fait {
                    empreinte,
                    fait: FORMAT,
                }
                | CleDeCaptation::FaitDuStem {
                    stem: empreinte,
                    fait: FORMAT,
                    ..
                } => {
                    if simple_de(&valeur, chaine).is_some() {
                        lisibles.insert(empreinte.to_string());
                    }
                    continue;
                }
                CleDeCaptation::Fait {
                    empreinte,
                    fait: MANIFESTE,
                } => {
                    manifestes.insert(empreinte.to_string());
                    continue;
                }
                CleDeCaptation::Nom { empreinte, chemin }
                | CleDeCaptation::NomDuStem {
                    stem: empreinte,
                    chemin,
                } => (empreinte, chemin),
                _ => continue,
            };
            let ulids = chaines(lecture, &objet, cle, valeur, element.conflict);
            if ulids.iter().any(|ulid| ulid == import) {
                noms.push((empreinte.to_string(), chemin.to_string()));
            }
        }
        for (empreinte, chemin) in noms {
            if manifestes.contains(&empreinte) {
                poses.jeux.insert(chemin);
            } else {
                let illisible = !lisibles.contains(&empreinte);
                poses.fichiers.insert(chemin, (empreinte, illisible));
            }
        }
    }
    poses
}

/// Les chemins des noms d'origine du contenu `empreinte` sous `conteneur` (`noms_de`).
fn chemins_des_noms(doc: &Automerge, conteneur: &str, empreinte: &str) -> BTreeSet<String> {
    noms_de(doc, conteneur, empreinte, None)
        .into_iter()
        .map(|(chemin, _)| chemin)
        .collect()
}

/// Ce que le fonds porte d'une prise en stems, de ses stems et de leurs MIDI, et les liens d'un
/// stem à son MIDI déjà posés sous elle : chaque clé `<p>/stem/<s>/midi/<m>` de `prises`, en
/// `DejaDeLaPrise::liens_midi` (revue de la tâche 6). Comme la doc d'`import::DejaDeLaPrise` :
/// `stems[s].origines` et `midis[m].origines` sont les origines du contenu, à quelque rôle que ce
/// soit ; elles se remplissent pour chaque stem et chaque MIDI du versement, même sous une prise
/// neuve ; `captation` n'y sert que pour les MIDI (balayage Q14). L'entrée `<p>/stem/<s>`, à
/// `true`, n'est pas un nom (balayage Q4). La prise est portée (`prise.captation`) dès qu'une clé
/// de `prises` commence par `<manifeste>/` ; un stem, dès que son entrée est sous elle ; un MIDI,
/// dès qu'une clé de `midi_verses` commence par `<m>/`. `prises` se parcourt une fois, depuis
/// `<manifeste>/`.
pub fn deja_de_la_prise(
    doc: &Automerge,
    manifeste: &str,
    stems: &[&str],
    midis: &[&str],
) -> DejaDeLaPrise {
    let lecture = Lecture::courante(doc);
    let mut deja = DejaDeLaPrise {
        prise: DejaLu {
            origines: origines_de(doc, manifeste),
            ..DejaLu::default()
        },
        ..DejaDeLaPrise::default()
    };
    for stem in stems {
        deja.stems.insert(
            stem.to_string(),
            DejaLu {
                origines: origines_de(doc, stem),
                ..DejaLu::default()
            },
        );
    }
    let midi_verses = lecture.conteneur(MIDI_VERSES);
    for midi in midis {
        let captation = midi_verses
            .as_ref()
            .is_some_and(|objet| lecture.sous(objet, &format!("{midi}/")).next().is_some());
        deja.midis.insert(
            midi.to_string(),
            DejaLu {
                captation,
                noms: chemins_des_noms(doc, MIDI_VERSES, midi),
                origines: origines_de(doc, midi),
            },
        );
    }
    let Some(prises) = lecture.conteneur(PRISES) else {
        return deja;
    };
    for element in lecture.sous(&prises, &format!("{manifeste}/")) {
        deja.prise.captation = true;
        match lire_la_cle(&element.key) {
            CleDeCaptation::Nom { chemin, .. } => {
                deja.prise.noms.insert(chemin.to_string());
            }
            CleDeCaptation::EntreeDuStem { stem, .. } => {
                if let Some(du_stem) = deja.stems.get_mut(stem) {
                    du_stem.captation = true;
                }
            }
            CleDeCaptation::NomDuStem { stem, chemin } => {
                if let Some(du_stem) = deja.stems.get_mut(stem) {
                    du_stem.noms.insert(chemin.to_string());
                }
            }
            CleDeCaptation::MidiDuStem { stem, midi } => {
                deja.liens_midi.insert((stem.to_string(), midi.to_string()));
            }
            _ => {}
        }
    }
    deja
}

/// D'où vient le type d'un stem (question 18 ; § 30 · 8.1), tel que `…/type/provenance` le dit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvenanceDuType {
    /// Gardé : `importee`, proposé d'après le nom du stem ; `par` (`nom`) et `depuis` (le stem),
    /// tels qu'ils se lisent sous `…/type/par` et `…/type/depuis`.
    Garde {
        par: Option<String>,
        depuis: Option<String>,
    },
    /// Choisi : `saisie`, qui n'a ni `par` ni `depuis`.
    Choisi,
}

/// Le type gardé ou choisi d'un stem, sous `prises/<p>/stem/<s>/type…`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeLu {
    /// Le type, `…/type`, lu par `TypeDeStem::lire`.
    pub type_de_stem: TypeDeStem,
    /// Sa provenance, `…/type/provenance`.
    pub provenance: ProvenanceDuType,
    /// L'heure où il fut gardé ou choisi, `…/type/le`, en secondes.
    pub le: Option<i64>,
}

/// Un stem d'une prise en stems, tel qu'il se lit sous son entrée `<p>/stem/<s>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StemLu {
    /// Son empreinte.
    pub empreinte: String,
    /// Ses faits : son format (`…/format`) et sa durée (`…/duree`), les seuls qu'un stem porte ;
    /// les autres à `None`.
    pub faits: Faits,
    /// Ses noms d'origine, ceux de `<p>/stem/<s>/nom/…` seuls : chaque couple (chemin, ULID), dans
    /// l'ordre des chemins (`noms_de`).
    pub noms: Vec<(String, String)>,
    /// Son type gardé ou choisi ; `None` s'il n'en a pas, ou si `…/type` n'est pas un type, ou
    /// `…/type/provenance` ni `importee` ni `saisie` : l'écran propose alors un type d'après son
    /// nom.
    pub type_de_stem: Option<TypeLu>,
    /// Les MIDI qui l'accompagnent (`…/midi/<m>`), dans l'ordre de leurs empreintes : un seul par
    /// import (question 17), d'autres si un import suivant en verse un autre.
    pub midis: BTreeSet<String>,
}

/// Ce qui se lit d'un stem sous sa prise, en un passage, avant d'en faire un `StemLu`.
#[derive(Default)]
struct StemEnLecture {
    format: Option<String>,
    duree: Option<u64>,
    noms: Vec<(String, String)>,
    type_de_stem: Option<String>,
    provenance: Option<String>,
    par: Option<String>,
    depuis: Option<String>,
    le: Option<i64>,
    midis: BTreeSet<String>,
}

impl StemEnLecture {
    /// Le type lu (`StemLu::type_de_stem`).
    fn type_lu(&self) -> Option<TypeLu> {
        let type_de_stem = TypeDeStem::lire(self.type_de_stem.as_deref()?)?;
        let provenance = match self.provenance.as_deref()? {
            IMPORTEE => ProvenanceDuType::Garde {
                par: self.par.clone(),
                depuis: self.depuis.clone(),
            },
            SAISIE => ProvenanceDuType::Choisi,
            _ => return None,
        };
        Some(TypeLu {
            type_de_stem,
            provenance,
            le: self.le,
        })
    }
}

/// Les stems de la prise `prise`, énumérés par leur entrée `<p>/stem/<s>` (balayage Q4), dans
/// l'ordre de leurs empreintes : leurs faits, leurs noms, leur type gardé ou choisi, avec sa
/// provenance, et leur MIDI. Les noms d'un stem sont ceux de `<p>/stem/<s>/nom/…` : les noms posés
/// hors du jeu, sur le même contenu versé seul (`prises/<s>/nom/…`), n'y sont pas (balayage Q19).
/// Une clé `<p>/stem/<s>/…` sans l'entrée de son stem ne fait pas un stem. `prises` se parcourt
/// une fois, depuis `<p>/stem/`.
pub fn stems_de(doc: &Automerge, prise: &str) -> Vec<StemLu> {
    let lecture = Lecture::courante(doc);
    let Some(prises) = lecture.conteneur(PRISES) else {
        return Vec::new();
    };
    let mut entrees: BTreeSet<String> = BTreeSet::new();
    let mut lus: BTreeMap<String, StemEnLecture> = BTreeMap::new();
    for element in lecture.sous(&prises, &format!("{prise}/{STEM}/")) {
        let valeur = element.value.into_value();
        let cle: &str = &element.key;
        match lire_la_cle(cle) {
            CleDeCaptation::EntreeDuStem { stem, .. } => {
                entrees.insert(stem.to_string());
            }
            CleDeCaptation::NomDuStem { stem, chemin } => {
                let ulids = chaines(lecture, &prises, cle, valeur, element.conflict);
                let lu = lus.entry(stem.to_string()).or_default();
                lu.noms
                    .extend(ulids.into_iter().map(|ulid| (chemin.to_string(), ulid)));
            }
            CleDeCaptation::MidiDuStem { stem, midi } => {
                lus.entry(stem.to_string())
                    .or_default()
                    .midis
                    .insert(midi.to_string());
            }
            CleDeCaptation::FaitDuStem { stem, fait, .. } => {
                let lu = lus.entry(stem.to_string()).or_default();
                match fait.split_once('/') {
                    None => match fait {
                        FORMAT => lu.format = simple_de(&valeur, chaine),
                        DUREE => lu.duree = simple_de(&valeur, naturel),
                        TYPE => lu.type_de_stem = simple_de(&valeur, chaine),
                        _ => {}
                    },
                    Some((TYPE, sous_le_type)) => match sous_le_type {
                        PROVENANCE => lu.provenance = simple_de(&valeur, chaine),
                        PAR => lu.par = simple_de(&valeur, chaine),
                        DEPUIS => lu.depuis = simple_de(&valeur, chaine),
                        LE => lu.le = simple_de(&valeur, entier),
                        _ => {}
                    },
                    Some(_) => {}
                }
            }
            _ => {}
        }
    }
    entrees
        .into_iter()
        .map(|stem| {
            let lu = lus.remove(&stem).unwrap_or_default();
            let type_de_stem = lu.type_lu();
            StemLu {
                empreinte: stem,
                faits: Faits {
                    format: lu.format,
                    duree_ms: lu.duree,
                    ..Faits::default()
                },
                noms: lu.noms,
                type_de_stem,
                midis: lu.midis,
            }
        })
        .collect()
}

/// Le format de la prise `empreinte`, `prises/<empreinte>/format`, que le protocole des captations
/// sert (question 27). `None` si la clé est absente, si sa valeur n'est pas une chaîne, ou si la
/// clé, coupée à sa place (`lire_la_cle`), n'est pas le fait `format` d'une empreinte : un chemin,
/// ou une empreinte qui nommerait le fait d'un stem, ne nomme aucune prise.
pub fn format_d_une_prise(doc: &Automerge, empreinte: &str) -> Option<String> {
    let cle = format!("{empreinte}/{FORMAT}");
    if !matches!(lire_la_cle(&cle), CleDeCaptation::Fait { fait: FORMAT, .. }) {
        return None;
    }
    valeur(doc, &objet(doc, &ROOT, PRISES)?, &cle, chaine).flatten()
}

/// Le format de chaque stem de chaque prise en stems, sous son empreinte : `<p>/stem/<s>/format`,
/// le stem énuméré par son entrée `<p>/stem/<s>` (balayage Q4), en un seul passage de `prises`
/// (question 27). Une valeur qui n'est pas une chaîne se saute ; un conteneur absent, aucun.
pub fn formats_des_stems(doc: &Automerge) -> BTreeMap<String, String> {
    let Some(prises) = objet(doc, &ROOT, PRISES) else {
        return BTreeMap::new();
    };
    let mut entrees: BTreeSet<String> = BTreeSet::new();
    let mut formats: BTreeMap<String, String> = BTreeMap::new();
    for element in doc.map_range(&prises, ..) {
        let Value::Scalar(simple) = element.value.into_value() else {
            continue;
        };
        match lire_la_cle(&element.key) {
            CleDeCaptation::EntreeDuStem { stem, .. } => {
                entrees.insert(stem.to_string());
            }
            CleDeCaptation::FaitDuStem {
                stem, fait: FORMAT, ..
            } => {
                if let Some(format) = chaine(&simple) {
                    formats.insert(stem.to_string(), format);
                }
            }
            _ => {}
        }
    }
    formats.retain(|stem, _| entrees.contains(stem));
    formats
}

/// Un import, tel qu'il se lit sous `imports/<ULID>/…`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportLu {
    /// Son ULID.
    pub import: String,
    /// L'heure de son ouverture, `<i>/ouvert`, en secondes.
    pub ouvert_le: i64,
    /// Clos, l'empreinte de son rapport, `<i>/rapport` (question 25).
    pub rapport: Option<String>,
}

/// Les imports, chacun avec l'heure de son ouverture et, clos, l'empreinte de son rapport, dans
/// l'ordre de leurs ULID ; un import dont `ouvert` ne se lit pas en entier ne se lit pas.
pub fn imports(doc: &Automerge) -> Vec<ImportLu> {
    let lecture = Lecture::courante(doc);
    let Some(objet) = lecture.conteneur(IMPORTS) else {
        return Vec::new();
    };
    let mut lus: BTreeMap<String, (Option<i64>, Option<String>)> = BTreeMap::new();
    for element in lecture.sous(&objet, "") {
        let valeur = element.value.into_value();
        let Some((import, fait)) = element.key.split_once('/') else {
            continue;
        };
        match fait {
            OUVERT => lus.entry(import.to_string()).or_default().0 = simple_de(&valeur, entier),
            RAPPORT => lus.entry(import.to_string()).or_default().1 = simple_de(&valeur, chaine),
            _ => {}
        }
    }
    lus.into_iter()
        .filter_map(|(import, (ouvert_le, rapport))| {
            Some(ImportLu {
                import,
                ouvert_le: ouvert_le?,
                rapport,
            })
        })
        .collect()
}

/// Les déclarations des imports dont l'ULID suit `prefixe` (tous, au préfixe vide), rangées par
/// import puis par chemin : `imports/<i>/declaration/<chemin>`, lu par `Ajout::lire`, et
/// `imports/<i>/jeu/<chemin>`, réunis par chemin en une `Declaration` (balayage Q3). Une valeur
/// qui ne se lit pas ne donne pas d'origine, et un chemin dont aucune clé ne se lit n'a pas de
/// déclaration. `imports` se parcourt une fois, depuis `prefixe`.
fn declarations_par_import(
    lecture: Lecture<'_>,
    prefixe: &str,
) -> BTreeMap<String, BTreeMap<String, Declaration>> {
    let mut par_import: BTreeMap<String, BTreeMap<String, Declaration>> = BTreeMap::new();
    let Some(objet) = lecture.conteneur(IMPORTS) else {
        return par_import;
    };
    for element in lecture.sous(&objet, prefixe) {
        let valeur = element.value.into_value();
        let mut segments = element.key.splitn(3, '/');
        let (Some(import), Some(famille), Some(chemin)) =
            (segments.next(), segments.next(), segments.next())
        else {
            continue;
        };
        match famille {
            DECLARATION => {
                if let Some(ajout) = simple_de(&valeur, chaine).as_deref().and_then(Ajout::lire) {
                    declaration_de(&mut par_import, import, chemin).origine = Some(ajout);
                }
            }
            JEU => declaration_de(&mut par_import, import, chemin).stems = true,
            _ => {}
        }
    }
    par_import
}

/// La déclaration du chemin `chemin` sous l'import `import`, créée vide s'il n'en a pas encore.
fn declaration_de<'p>(
    par_import: &'p mut BTreeMap<String, BTreeMap<String, Declaration>>,
    import: &str,
    chemin: &str,
) -> &'p mut Declaration {
    par_import
        .entry(import.to_string())
        .or_default()
        .entry(chemin.to_string())
        .or_default()
}

/// Les déclarations de l'import `import` : `imports/<i>/declaration/<chemin>`, lu par
/// `Ajout::lire` (une valeur qui ne se lit pas ne donne pas d'origine), et
/// `imports/<i>/jeu/<chemin>`, réunis par chemin en une `Declaration` (balayage Q3).
pub fn declarations_de(doc: &Automerge, import: &str) -> BTreeMap<String, Declaration> {
    declarations_par_import(Lecture::courante(doc), &format!("{import}/"))
        .remove(import)
        .unwrap_or_default()
}

/// Pour chaque chemin, la `Declaration` entière de l'import le plus récent qui a écrit l'une de ses
/// deux clés, et non famille par famille (question 9 ; balayage Q3) : les imports se prennent dans
/// l'ordre de leurs ULID, que leur heure ouvre, chacun remplaçant la déclaration d'un chemin qu'il
/// déclare.
pub fn dernieres_declarations(doc: &Automerge) -> BTreeMap<String, Declaration> {
    let mut dernieres = BTreeMap::new();
    for declarations in declarations_par_import(Lecture::courante(doc), "").into_values() {
        dernieres.extend(declarations);
    }
    dernieres
}

/// Les noms de générateurs que porte le fonds, chacun une fois, dans l'ordre de leurs octets : ceux
/// des origines déclarées (`origines_declarees/<e>/generee/<nom>`) et ceux des déclarations des
/// imports (`generee:<nom>`).
pub fn generateurs(doc: &Automerge) -> BTreeSet<String> {
    let lecture = Lecture::courante(doc);
    let mut noms = BTreeSet::new();
    if let Some(origines) = lecture.conteneur(ORIGINES_DECLAREES) {
        for element in lecture.sous(&origines, "") {
            if let Some((_, element_d_origine)) = element.key.split_once('/')
                && let Some(nom) = nom_genere(element_d_origine)
            {
                noms.insert(nom.to_string());
            }
        }
    }
    for declarations in declarations_par_import(lecture, "").into_values() {
        noms.extend(declarations.into_values().filter_map(
            |declaration| match declaration.origine {
                Some(Ajout::Generee(nom)) => Some(nom),
                _ => None,
            },
        ));
    }
    noms
}
