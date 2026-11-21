//! L'origine déclarée d'un contenu, le premier nom vu d'un fichier et la portée d'une
//! déclaration d'import (§ 30 · 9 ; questions 17, 20, 22 et 23), sans entrée-sortie.

use std::collections::{BTreeMap, BTreeSet};

/// L'ensemble des origines d'un contenu, tel qu'il se lit sous `origines_declarees/<e>/…`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OriginesLues {
    pub main: bool,
    pub generees: BTreeSet<String>,
}

/// L'origine la plus forte d'un contenu (CAP-03) : générée, avec chacun de ses noms ; puis de la
/// main de Sacha ; `None` vaut `—`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origine {
    Main,
    Generee(BTreeSet<String>),
}

/// L'origine la plus forte de `lues`.
pub fn la_plus_forte(lues: &OriginesLues) -> Option<Origine> {
    if !lues.generees.is_empty() {
        Some(Origine::Generee(lues.generees.clone()))
    } else if lues.main {
        Some(Origine::Main)
    } else {
        None
    }
}

/// Un élément d'origine qu'une déclaration ajoute.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ajout {
    Main,
    Generee(String),
}

impl Ajout {
    /// `main`, ou `generee:<nom>` : l'élément que dit le rapport.
    pub fn element(&self) -> String {
        match self {
            Ajout::Main => "main".to_string(),
            Ajout::Generee(nom) => format!("generee:{nom}"),
        }
    }
}

/// Ce que `ajout` change à `lues` : `None` s'il y est déjà, ou si c'est `Main` sur un contenu
/// généré (question 22 : fait, sans rien changer).
pub fn ajout_d_origine(lues: &OriginesLues, ajout: &Ajout) -> Option<Ajout> {
    let deja = match ajout {
        Ajout::Main => lues.main || !lues.generees.is_empty(),
        Ajout::Generee(nom) => lues.generees.contains(nom),
    };
    if deja { None } else { Some(ajout.clone()) }
}

/// La déclaration d'un dossier, pendant l'import (question 22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    Stems,
    Origine(Ajout),
}

impl Declaration {
    /// `stems`, `main` ou `generee:<nom>` : la valeur de `imports/<i>/declaration/<chemin>`.
    pub fn valeur(&self) -> String {
        match self {
            Declaration::Stems => "stems".to_string(),
            Declaration::Origine(ajout) => ajout.element(),
        }
    }

    /// L'inverse de `valeur` ; `None` pour toute autre chaîne, `generee:` sans nom compris.
    pub fn lire(valeur: &str) -> Option<Declaration> {
        match valeur {
            "stems" => Some(Declaration::Stems),
            "main" => Some(Declaration::Origine(Ajout::Main)),
            _ => valeur
                .strip_prefix("generee:")
                .filter(|nom| !nom.is_empty())
                .map(|nom| Declaration::Origine(Ajout::Generee(nom.to_string()))),
        }
    }
}

/// Ce que les déclarations de l'import donnent au fichier `chemin` : l'origine du plus proche de
/// ses dossiers déclarés, du sien au dossier versé (`""`) compris, qui déclare une origine ; et le
/// jeu de stems, son propre dossier s'il est déclaré `stems` (question 17), jamais un dossier plus
/// haut. Une déclaration `stems` n'arrête pas l'origine d'un dossier plus haut. Seuls des dossiers
/// se déclarent : le chemin du fichier lui-même n'est jamais une clé lue (balayage P22).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclarationDuFichier {
    pub origine: Option<Ajout>,
    pub jeu: Option<String>,
}

/// Les dossiers de `chemin`, du plus proche au dossier versé (`""`) : un parcours fini, borné au
/// nombre des `/` du chemin plus un, qui monte d'un segment par pas.
fn dossiers(chemin: &str) -> impl Iterator<Item = &str> {
    let premier = chemin.rsplit_once('/').map_or("", |(dossier, _)| dossier);
    let bornes = chemin.matches('/').count() + 1;
    std::iter::successors(Some(premier), |dossier| {
        Some(dossier.rsplit_once('/').map_or("", |(haut, _)| haut))
    })
    .take(bornes)
}

/// La déclaration de `chemin`, d'après `declarations` (dossier -> déclaration).
pub fn declaration_du_fichier(
    chemin: &str,
    declarations: &BTreeMap<String, Declaration>,
) -> DeclarationDuFichier {
    let origine = dossiers(chemin).find_map(|dossier| match declarations.get(dossier) {
        Some(Declaration::Origine(ajout)) => Some(ajout.clone()),
        _ => None,
    });
    let jeu = dossiers(chemin)
        .next()
        .filter(|dossier| declarations.get(*dossier) == Some(&Declaration::Stems))
        .map(str::to_string);
    DeclarationDuFichier { origine, jeu }
}

/// Le premier nom vu (question 20) : de `noms`, des couples (chemin, ULID de l'import qui l'a vu
/// le premier), une valeur par réplique comprise, celui dont l'heure de l'ULID (ses dix premiers
/// caractères) est la plus petite, puis le chemin, dans l'ordre de ses octets.
pub fn premier_nom_vu<'a>(noms: impl IntoIterator<Item = (&'a str, &'a str)>) -> Option<&'a str> {
    noms.into_iter()
        .min_by_key(|(chemin, ulid)| (ulid.get(..10).unwrap_or(ulid), chemin.as_bytes()))
        .map(|(chemin, _)| chemin)
}

/// Vrai si `chemin` est sous le préfixe `prefixe`, par segments entiers, octet pour octet
/// (question 23) ; le préfixe vide prend tout.
pub fn sous_le_prefixe(chemin: &str, prefixe: &str) -> bool {
    prefixe.is_empty()
        || chemin == prefixe
        || (chemin.starts_with(prefixe) && chemin.as_bytes().get(prefixe.len()) == Some(&b'/'))
}
