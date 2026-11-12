//! Les règles du démarrage (§ 31 · 2.1, § 31 · 6.1) : le nom d'un voisin, le sort de chaque
//! `.partiel` trouvé dans `documents/`, et les noms provisoires de `captations/` à ranger. Le
//! noyau décide, l'adaptateur du stockage lit et écrit.

use std::collections::BTreeSet;

/// `<nom>.<processus>.partiel`, ou `-2`, `-3`… : le premier que `pris` (les noms du dossier) n'a pas.
///
/// Une recherche bornée : les rangs 1 à `pris.len() + 1`, dont l'un est forcément libre, le rang 1
/// sans suffixe, le rang n ≥ 2 en `-n` ; le premier libre (`find`), puis `expect`. Jamais de boucle
/// sans borne (`loop`, `while`) : ses mutants (`+=` en `*=`, le `!` effacé) boucleraient sans fin,
/// et cargo-mutants, sur un TIMEOUT, sort en 3, ce qui fait échouer `mutants` ; bornée, chaque
/// mutant panique ou rend un autre nom.
pub fn voisin_libre(nom: &str, processus: u32, pris: &[String]) -> String {
    (1..=pris.len() + 1)
        .map(|rang| {
            if rang == 1 {
                format!("{nom}.{processus}.partiel")
            } else {
                format!("{nom}.{processus}-{rang}.partiel")
            }
        })
        .find(|candidat| !pris.contains(candidat))
        .expect("parmi pris.len() + 1 rangs, un est libre")
}

fn en_chiffres(texte: &str) -> bool {
    !texte.is_empty() && texte.bytes().all(|o| o.is_ascii_digit())
}

/// Le nom du fichier dont `nom` est le voisin (`<nom>.<chiffres>[-<chiffres>].partiel`), sinon `None`.
pub fn fichier_du_voisin(nom: &str) -> Option<&str> {
    let sans_partiel = nom.strip_suffix(".partiel")?;
    let (fichier, suffixe) = sans_partiel.rsplit_once('.')?;
    let (processus, rang) = match suffixe.split_once('-') {
        Some((processus, rang)) => (processus, Some(rang)),
        None => (suffixe, None),
    };
    let valide = en_chiffres(processus) && rang.is_none_or(en_chiffres);
    (valide && !fichier.is_empty()).then_some(fichier)
}

/// Un `.partiel` de `documents/`, tel que l'adaptateur l'a lu (seuls les noms en `.partiel`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Partiel<'a> {
    /// Dans `ajouts/` : son nom, et l'empreinte de son contenu. Un voisin vide a l'empreinte du
    /// vide, qui n'est jamais celle de son nom : il se retire, il ne se met jamais en place.
    DansAjouts { nom: &'a str, empreinte: &'a str },
    /// Dans le dossier d'un document : son nom, et s'il est lisible (non vide, `load` sans erreur).
    /// Un voisin de zéro octet n'est pas lisible : il ne se lit jamais.
    DansLeDossier { nom: &'a str, lisible: bool },
    /// Ailleurs dans `documents/`.
    Ailleurs,
}

/// Ce que le démarrage fait d'un `.partiel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sort {
    MettreEnPlace,
    Lire,
    Retirer,
}

/// DansAjouts : MettreEnPlace si `fichier_du_voisin(nom) == Some("<empreinte>.automerge")` ;
/// DansLeDossier : Lire si `fichier_du_voisin(nom) == Some("complet.automerge")` et lisible ;
/// sinon Retirer.
pub fn sort_d_un_partiel(partiel: Partiel<'_>) -> Sort {
    match partiel {
        Partiel::DansAjouts { nom, empreinte } => {
            if fichier_du_voisin(nom) == Some(format!("{empreinte}.automerge").as_str()) {
                Sort::MettreEnPlace
            } else {
                Sort::Retirer
            }
        }
        Partiel::DansLeDossier { nom, lisible } => {
            if lisible && fichier_du_voisin(nom) == Some("complet.automerge") {
                Sort::Lire
            } else {
                Sort::Retirer
            }
        }
        Partiel::Ailleurs => Sort::Retirer,
    }
}

/// Pupitre est neuf : ni fichier complet, ni voisin du complet lisible (§ 31 · 9.1).
pub fn pupitre_neuf(complet: bool, voisins_du_complet_lisibles: usize) -> bool {
    !complet && voisins_du_complet_lisibles == 0
}

/// Un nom provisoire de `captations/` : son nom, l'empreinte de son contenu entier, et si une
/// captation de cette empreinte est déjà rangée (`deja_rangee`, et non `rangee`, que le noyau
/// emploie pour la rangée de la table : `table::Rangee`, `Position { rangee }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provisoire {
    pub nom: String,
    pub empreinte: String,
    pub deja_rangee: bool,
}

/// (nom provisoire, empreinte) de chaque nom provisoire dont l'empreinte est nommée et non rangée ;
/// de deux du même contenu, le premier dans l'ordre reçu, que l'adaptateur donne trié par nom : le
/// second écrirait sur une captation rangée (I14 ; § 31 · 6.1).
pub fn a_ranger(provisoires: &[Provisoire], nommees: &BTreeSet<String>) -> Vec<(String, String)> {
    let mut vues = BTreeSet::new();
    provisoires
        .iter()
        .filter(|p| !p.deja_rangee && nommees.contains(&p.empreinte))
        .filter(|p| vues.insert(p.empreinte.as_str()))
        .map(|p| (p.nom.clone(), p.empreinte.clone()))
        .collect()
}
