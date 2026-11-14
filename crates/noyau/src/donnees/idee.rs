//! Les règles d'une idée et de ses captations, sans entrée-sortie : les empreintes que nomment
//! les clés `<empreinte>/<fait>` des conteneurs des captations (§ 31 · 6.1), le lien
//! « apparenté », la durée, l'assiette d'un compte (§ 30 · 9).

use std::collections::BTreeSet;

use super::gestes::IdeeLue;

/// Le préfixe de la clé d'un lien « apparenté ».
const PREFIXE_DU_LIEN: &str = "apparente:";

/// Les empreintes que nomment des clés `<empreinte>/<fait>` : ce qui précède le premier `/` ; une
/// clé sans `/` ne nomme rien.
pub fn empreintes_nommees<'a>(cles: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
    cles.into_iter()
        .filter_map(|cle| cle.split_once('/'))
        .map(|(empreinte, _)| empreinte.to_string())
        .collect()
}

/// La clé d'un lien « apparenté » : `apparente:<a>:<b>`, les deux bouts dans l'ordre de leurs
/// octets, si bien que deux appareils qui posent le même lien en font la même clé (§ 30 · 9.3).
pub fn cle_du_lien(a: &str, b: &str) -> String {
    let (p, q) = (a.min(b), a.max(b));
    format!("{PREFIXE_DU_LIEN}{p}:{q}")
}

/// Les deux bouts d'une clé de lien ; `None` si la clé n'en est pas une.
pub fn bouts_du_lien(cle: &str) -> Option<(&str, &str)> {
    cle.strip_prefix(PREFIXE_DU_LIEN)?.split_once(':')
}

/// Reliée : au moins un lien dont l'autre bout n'est pas une idée retirée (§ 30 · 9.1).
pub fn reliee(idee: &IdeeLue, retiree: impl Fn(&str) -> bool) -> bool {
    idee.liens.iter().any(|cle| {
        bouts_du_lien(cle).is_some_and(|(a, b)| {
            let autre = if a == idee.id { b } else { a };
            !retiree(autre)
        })
    })
}

/// La durée d'une idée, des durées de ses prises : la plus longue ; aucune prise, ou une prise
/// sans durée lue, `None` (§ 30 · 9.2).
pub fn duree(prises: &[Option<u64>]) -> Option<u64> {
    prises
        .iter()
        .copied()
        .try_fold(None, |plus_longue: Option<u64>, prise| {
            prise.map(|d| Some(plus_longue.map_or(d, |p| p.max(d))))
        })
        .flatten()
}

/// Sans rien : ni captation, ni texte, ni étiquette, ni lien (§ 30 · 9.3).
pub fn est_vide(idee: &IdeeLue, captations: usize) -> bool {
    captations == 0 && idee.texte.is_none() && idee.etiquettes.is_empty() && idee.liens.is_empty()
}

/// L'assiette d'un compte sur les idées.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assiette {
    /// Les idées considérées : les idées retirées n'y entrent pas.
    pub considerees: usize,
    /// Celles qui portent la donnée.
    pub portant: usize,
    /// Celles dont la donnée est décidée.
    pub decidees: usize,
}

/// L'assiette d'un compte sur les idées : les idées retirées n'y entrent pas ; `porte` dit si une
/// idée porte la donnée ; au lot 1, toute donnée portée est décidée.
pub fn assiette<'a>(
    idees: impl IntoIterator<Item = &'a IdeeLue>,
    porte: impl Fn(&IdeeLue) -> bool,
) -> Assiette {
    let mut a = Assiette {
        considerees: 0,
        portant: 0,
        decidees: 0,
    };
    for idee in idees.into_iter().filter(|i| i.retiree.is_none()) {
        a.considerees += 1;
        if porte(idee) {
            a.portant += 1;
            a.decidees += 1;
        }
    }
    a
}
