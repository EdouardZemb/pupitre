//! L'index des idées interrogées et la réponse d'une requête (CAP-06) : un dérivé jetable (I13),
//! bâti du fonds lu, sans rien écrire. Chaque requête le parcourt idée par idée, sans index
//! inversé (décision 17 ; LE-6, règle 14), puis range ce qu'elle trouve par `boite::ordonner`.

use std::collections::BTreeSet;

use super::boite::{EnBoite, Ordre, ordonner};
use super::gestes::{IdeeLue, Provenance};
use super::idee::Assiette;
use super::repli::{mots, replier, trouve};
use super::requete::{Bornes, Etat, ProvenanceDeLIdee, Requete, Sans};

/// Une idée de l'index : l'idée telle que la boîte la reçoit, et son texte replié.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexee {
    /// L'idée telle que la boîte la reçoit.
    pub en_boite: EnBoite,
    /// Son texte replié (`repli::replier`), si elle en a un.
    pub texte_replie: Option<String>,
}

/// L'index des idées interrogées : un dérivé jetable (I13), bâti du fonds lu, sans rien écrire.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Index {
    /// Les idées non retirées, indexées.
    idees: Vec<Indexee>,
    /// Le nombre des idées qui ne se lisent pas.
    illisibles: usize,
}

/// La réponse d'une requête : le compte des non reliées et son assiette, qui ne suivent pas la
/// requête ; l'assiette de la requête (considérées : les lisibles et les illisibles ; portant et
/// décidées : les lisibles qui portent chaque attribut que la requête interroge), ses illisibles, ses
/// parts « sans », et les idées trouvées, rangées par `ordonner`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reponse {
    /// Les idées de l'index non reliées, que la requête ne change pas.
    pub non_reliees: usize,
    /// L'assiette de ce compte : les idées de l'index portent la donnée, les illisibles comptent
    /// parmi les seules considérées.
    pub assiette_du_compte: Assiette,
    /// L'assiette de la requête.
    pub assiette: Assiette,
    /// Les idées qui ne se lisent pas.
    pub illisibles: usize,
    /// Les parts « sans … ».
    pub sans: Sans,
    /// Les idées trouvées, rangées.
    pub trouvees: Vec<EnBoite>,
}

/// Ce que dit d'une idée un critère sur un attribut qu'elle peut manquer : l'étiquette, la durée,
/// le texte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    /// Le critère n'est pas posé, ou l'idée porte l'attribut et satisfait le critère.
    Satisfait,
    /// L'idée porte l'attribut et ne satisfait pas le critère.
    NonSatisfait,
    /// L'idée manque l'attribut.
    Manque,
}

impl Verdict {
    /// Le verdict d'un critère posé : `Manque` sans `attribut`, sinon selon `satisfait`.
    fn sur<T>(attribut: Option<T>, satisfait: impl FnOnce(T) -> bool) -> Verdict {
        match attribut.map(satisfait) {
            None => Verdict::Manque,
            Some(true) => Verdict::Satisfait,
            Some(false) => Verdict::NonSatisfait,
        }
    }
}

/// La provenance de `idee`, pour une requête : captée ou importée.
fn provenance_de(idee: &IdeeLue) -> ProvenanceDeLIdee {
    match idee.provenance {
        Provenance::Captee { .. } => ProvenanceDeLIdee::Captee,
        Provenance::Importee { .. } => ProvenanceDeLIdee::Importee,
    }
}

/// `e` satisfait les critères que toute idée porte : la date, dans `bornes` ; l'état, la
/// provenance et le contenu, s'ils sont posés.
fn satisfait_les_criteres_toujours_portes(requete: &Requete, bornes: Bornes, e: &EnBoite) -> bool {
    bornes.contient(e.idee.le)
        && requete.etat.is_none_or(|etat| match etat {
            Etat::Reliee => e.reliee,
            Etat::NonReliee => !e.reliee,
        })
        && requete
            .provenance
            .is_none_or(|provenance| provenance == provenance_de(&e.idee))
        && requete
            .contenu
            .is_none_or(|contenu| e.contenus.contains(&contenu))
}

/// Les verdicts des critères qu'une idée peut manquer, dans l'ordre des parts « sans … » :
/// l'étiquette, portée par une idée qui en a une, satisfaite si elle la porte, octet pour octet ;
/// la durée, portée par une idée qui a une durée, satisfaite si `duree_ms / 1 000` dépasse
/// strictement le seuil ; le texte, sans mot aucun critère, sinon porté par une idée qui a un
/// texte, satisfait si chacun de `cherches` y paraît (`repli::trouve`).
fn verdicts_de(requete: &Requete, cherches: &[String], indexee: &Indexee) -> [Verdict; 3] {
    let e = &indexee.en_boite;
    let etiquette = requete
        .etiquette
        .as_ref()
        .map_or(Verdict::Satisfait, |cherchee| {
            let etiquettes = &e.idee.etiquettes;
            Verdict::sur((!etiquettes.is_empty()).then_some(etiquettes), |portees| {
                portees.contains(cherchee)
            })
        });
    let duree = requete
        .plus_de_secondes
        .map_or(Verdict::Satisfait, |seuil| {
            Verdict::sur(e.duree_ms, |duree_ms| duree_ms / 1_000 > seuil)
        });
    let texte = if cherches.is_empty() {
        Verdict::Satisfait
    } else {
        Verdict::sur(indexee.texte_replie.as_deref(), |texte| {
            trouve(texte, cherches)
        })
    };
    [etiquette, duree, texte]
}

impl Index {
    /// L'index de `idees`, les retirées écartées (CAP-06), et de `illisibles` idées qui ne se
    /// lisent pas. Chaque texte se replie une fois.
    pub fn construire(idees: Vec<EnBoite>, illisibles: usize) -> Index {
        let idees = idees
            .into_iter()
            .filter(|e| e.idee.retiree.is_none())
            .map(|en_boite| Indexee {
                texte_replie: en_boite.idee.texte.as_deref().map(replier),
                en_boite,
            })
            .collect();
        Index { idees, illisibles }
    }

    /// La réponse de `requete`, la date jugée sur `bornes`, les trouvées rangées par `ordre`. Une
    /// idée est trouvée si elle porte chaque attribut interrogé et satisfait chaque critère ; elle
    /// compte dans une part « sans » si elle manque cet attribut et satisfait chaque autre critère
    /// qu'elle porte : une idée qui en manque deux compte dans les deux (LE-6, L5).
    pub fn repondre(&self, requete: &Requete, bornes: Bornes, ordre: Ordre) -> Reponse {
        let cherches = mots(&requete.texte);
        let mut sans = Sans {
            etiquette: requete.etiquette.as_ref().map(|_| 0),
            duree: requete.plus_de_secondes.map(|_| 0),
            texte: (!cherches.is_empty()).then_some(0),
        };
        let mut portant = 0;
        let mut trouvees = Vec::new();
        for indexee in &self.idees {
            let verdicts = verdicts_de(requete, &cherches, indexee);
            let porte_tout = !verdicts.contains(&Verdict::Manque);
            let aucun_refus =
                satisfait_les_criteres_toujours_portes(requete, bornes, &indexee.en_boite)
                    && !verdicts.contains(&Verdict::NonSatisfait);
            if porte_tout {
                portant += 1;
            }
            if porte_tout && aucun_refus {
                trouvees.push(indexee.en_boite.clone());
            }
            // Une idée ne manque un attribut que si son critère est posé : sa part est `Some`.
            for (part, verdict) in [&mut sans.etiquette, &mut sans.duree, &mut sans.texte]
                .into_iter()
                .zip(verdicts)
            {
                if aucun_refus
                    && verdict == Verdict::Manque
                    && let Some(k) = part
                {
                    *k += 1;
                }
            }
        }
        ordonner(&mut trouvees, ordre);
        let lisibles = self.idees.len();
        let considerees = lisibles + self.illisibles;
        Reponse {
            non_reliees: self.idees.iter().filter(|i| !i.en_boite.reliee).count(),
            assiette_du_compte: Assiette {
                considerees,
                portant: lisibles,
                decidees: lisibles,
            },
            assiette: Assiette {
                considerees,
                portant,
                decidees: portant,
            },
            illisibles: self.illisibles,
            sans,
            trouvees,
        }
    }

    /// Les étiquettes des idées de l'index, chacune une fois, dans l'ordre de leurs octets.
    pub fn etiquettes(&self) -> Vec<String> {
        let etiquettes: BTreeSet<&String> = self
            .idees
            .iter()
            .flat_map(|i| &i.en_boite.idee.etiquettes)
            .collect();
        etiquettes.into_iter().cloned().collect()
    }
}
