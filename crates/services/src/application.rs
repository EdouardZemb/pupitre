//! L'application des changements (§ 31 · 3) : le code que partagent les deux adaptateurs
//! `Documents`, sur fichiers et en mémoire, et le seul des services qui écrit dans un document
//! (I28 : toute modification d'un document naît d'une opération nommée). Une opération s'applique
//! en une seule modification Automerge, avec son message et son heure, ou ne laisse rien ; la
//! genèse et les migrations, écrites d'avance, s'appliquent en octets.

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{
    Automerge, AutomergeError, Change, ChangeHash, ObjId, ObjType, ROOT, ReadDoc, ScalarValue,
    Value,
};
use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::fonds::GENESE;
use noyau::donnees::version::Migration;

/// Pourquoi une opération ne s'applique pas ; elle ne laisse alors rien dans le document.
#[derive(Debug)]
pub enum Inapplicable {
    /// Un objet que traverse le chemin d'un changement manque, ou n'est pas un objet à clés :
    /// `chemin`, du premier segment jusqu'à lui. Un chemin vide, qui ne nomme aucune clé, le rend
    /// aussi, vide.
    ObjetAbsent { chemin: Vec<String> },
    /// L'opération ne change rien au document : Automerge n'en écrit aucune modification.
    Vide,
    /// Automerge a refusé un changement, ou une modification écrite d'avance.
    Automerge(AutomergeError),
}

/// Une transaction : chaque changement, dans l'ordre, puis `commit_with(message, heure)`. Un objet
/// absent sur le chemin annule la transaction (`rollback`) ; une modification vide rend `Vide`.
pub fn appliquer(doc: &mut Automerge, operation: &Operation) -> Result<ChangeHash, Inapplicable> {
    let mut tx = doc.transaction();
    for changement in &operation.changements {
        if let Err(refus) = appliquer_le_changement(&mut tx, changement) {
            tx.rollback();
            return Err(refus);
        }
    }
    let options = CommitOptions::default()
        .with_message(operation.message.as_str())
        .with_time(operation.heure);
    let (tete, _) = tx.commit_with(options);
    tete.ok_or(Inapplicable::Vide)
}

/// Un document neuf (son acteur, tiré par Automerge), la genèse (`GENESE`, par `apply_changes`),
/// puis `naissance`.
pub fn naitre(naissance: &Operation) -> Result<Automerge, Inapplicable> {
    let genese =
        Change::try_from(GENESE).map_err(|erreur| Inapplicable::Automerge(erreur.into()))?;
    let mut doc = Automerge::new();
    doc.apply_changes([genese])
        .map_err(Inapplicable::Automerge)?;
    appliquer(&mut doc, naissance)?;
    Ok(doc)
}

/// `naitre`, puis `geste`.
pub fn faire_naitre(naissance: &Operation, geste: &Operation) -> Result<Automerge, Inapplicable> {
    let mut doc = naitre(naissance)?;
    appliquer(&mut doc, geste)?;
    Ok(doc)
}

/// Les octets de chaque migration (`Change::try_from`, puis `apply_changes`), dans l'ordre de
/// `a_appliquer`.
pub fn migrer(doc: &mut Automerge, a_appliquer: &[Migration<'_>]) -> Result<(), AutomergeError> {
    let modifications = a_appliquer
        .iter()
        .map(|migration| Change::try_from(migration.octets))
        .collect::<Result<Vec<_>, _>>()?;
    doc.apply_changes(modifications)
}

/// `changement`, dans la transaction `tx` : `Poser` par `put`, `Creer` par `put_object`, un objet
/// à clés, puis chacun de ses conteneurs, `Oter` par `delete`.
fn appliquer_le_changement(
    tx: &mut Transaction<'_>,
    changement: &Changement,
) -> Result<(), Inapplicable> {
    match changement {
        Changement::Poser { chemin, valeur } => {
            let (parent, cle) = trouver_le_parent(tx, chemin)?;
            tx.put(&parent, cle, scalaire(valeur))
                .map_err(Inapplicable::Automerge)
        }
        Changement::Creer { chemin, conteneurs } => {
            let (parent, cle) = trouver_le_parent(tx, chemin)?;
            let objet = tx
                .put_object(&parent, cle, ObjType::Map)
                .map_err(Inapplicable::Automerge)?;
            for conteneur in conteneurs {
                tx.put_object(&objet, conteneur.as_str(), ObjType::Map)
                    .map_err(Inapplicable::Automerge)?;
            }
            Ok(())
        }
        Changement::Oter { chemin } => {
            let (parent, cle) = trouver_le_parent(tx, chemin)?;
            tx.delete(&parent, cle).map_err(Inapplicable::Automerge)
        }
    }
}

/// L'objet à clés qui porte la dernière clé de `chemin`, trouvé depuis la racine du document, pas
/// à pas, chaque pas un objet à clés, et cette clé.
fn trouver_le_parent<'c>(
    tx: &Transaction<'_>,
    chemin: &'c [String],
) -> Result<(ObjId, &'c str), Inapplicable> {
    let Some((cle, pas)) = chemin.split_last() else {
        return Err(Inapplicable::ObjetAbsent { chemin: Vec::new() });
    };
    let mut objet = ROOT;
    for (rang, segment) in pas.iter().enumerate() {
        objet = match tx.get(&objet, segment.as_str()) {
            Ok(Some((Value::Object(ObjType::Map), id))) => id,
            Ok(_) => {
                return Err(Inapplicable::ObjetAbsent {
                    chemin: pas[..=rang].to_vec(),
                });
            }
            Err(erreur) => return Err(Inapplicable::Automerge(erreur)),
        };
    }
    Ok((objet, cle.as_str()))
}

/// La valeur simple d'Automerge que porte `valeur` : `Chaine` en `Str`, `Entier` en `Int`,
/// `Naturel` en `Uint`, `Vrai` en `Boolean(true)`.
fn scalaire(valeur: &Simple) -> ScalarValue {
    match valeur {
        Simple::Chaine(texte) => ScalarValue::Str(texte.as_str().into()),
        Simple::Entier(entier) => ScalarValue::Int(*entier),
        Simple::Naturel(naturel) => ScalarValue::Uint(*naturel),
        Simple::Vrai => ScalarValue::Boolean(true),
    }
}
