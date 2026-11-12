//! Le démarrage dans `documents/` (§ 31 · 2.1) : à chaque ouverture du fonds, la garde prise,
//! chaque `.partiel` reçoit le sort que lui donnent les règles du noyau
//! (`noyau::donnees::demarrage`). La lecture du fonds et sa naissance lisent le voisin du fichier
//! complet selon ce même sort.

use std::path::Path;

use automerge::Automerge;
use noyau::donnees::demarrage::{self, Partiel, Sort};

use crate::documents::{nom_de, partiels};
use crate::fichier;
use crate::{Avertissement, Erreur, disque, empreinte};

/// § 31 · 2.1 : le démarrage dans `documents/`, la garde prise, avant toute lecture du fonds.
/// Chaque `.partiel` reçoit son sort (`sort_d_un_partiel`), dans l'ordre des noms : ceux de
/// `documents` (`Ailleurs`), puis ceux du dossier du fonds `dossier` (`DansLeDossier`, lisible
/// s'il n'est pas vide et qu'`Automerge::load` le relit sans erreur), puis ceux de son `ajouts/`
/// (`DansAjouts`, avec l'empreinte de son contenu) ; chaque `.partiel` dont le sort demande le
/// contenu se lit par `fichier::lire`.
///
/// - `MettreEnPlace` : le voisin est renommé en son fichier d'ajout (`fichier_du_voisin`), dans
///   `ajouts/`, puis le renommage est rendu durable. Un renommage refusé ne fait pas échouer
///   l'ouverture : le voisin reste où il est, `lire_le_dossier` le lit avec les fichiers
///   d'ajout, et `Avertissement::VoisinRestant` le nomme. Un renommage non rendu durable laisse
///   le fichier d'ajout en place, avec `Avertissement::NonDurable`.
/// - `Lire` : rien ; `lire_le_dossier` le lit.
/// - `Retirer` : `fichier::retirer` ; un refus est tu, et l'ouverture suivante le retente.
///
/// Un dossier absent ne contient aucun `.partiel`, et le démarrage ne crée aucun dossier dans
/// `documents/` : un `ajouts/` absent se recrée à la première écriture. Toute autre erreur de
/// lecture d'un dossier, ou un refus du disque à la lecture d'un `.partiel` du dossier du fonds ou
/// de `ajouts/` (ceux de `documents/` ne se lisent pas), rend `Erreur::Disque` : un voisin non lu
/// peut porter une opération dite faite, ou le premier fichier complet. Un fichier qui n'est pas
/// un `.partiel` n'est ni lu ni retiré.
pub(crate) fn demarrer(documents: &Path, dossier: &Path) -> Result<Vec<Avertissement>, Erreur> {
    let mut avertissements = Vec::new();
    for chemin in partiels(documents)? {
        let sort = demarrage::sort_d_un_partiel(Partiel::Ailleurs);
        donner_son_sort(&chemin, sort, &mut avertissements);
    }
    for chemin in partiels(dossier)? {
        let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        let sort = sort_dans_le_dossier(&chemin, &octets);
        donner_son_sort(&chemin, sort, &mut avertissements);
    }
    for chemin in partiels(&dossier.join("ajouts"))? {
        let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        let sort = sort_dans_ajouts(&chemin, &octets);
        donner_son_sort(&chemin, sort, &mut avertissements);
    }
    Ok(avertissements)
}

/// Fait du `.partiel` `chemin` ce que dit `sort`, comme [`demarrer`] le dit ; un avertissement
/// rejoint `avertissements`.
fn donner_son_sort(chemin: &Path, sort: Sort, avertissements: &mut Vec<Avertissement>) {
    match sort {
        Sort::Lire => {}
        Sort::Retirer => {
            // Un voisin qui reste se retire à l'ouverture suivante.
            let _ = fichier::retirer(chemin);
        }
        Sort::MettreEnPlace => {
            let nom = nom_de(chemin);
            let nom_du_fichier = demarrage::fichier_du_voisin(&nom)
                .expect("un voisin à mettre en place nomme son fichier d'ajout");
            let cible = chemin.with_file_name(nom_du_fichier);
            if let Err(erreur) = fichier::renommer(chemin, &cible) {
                avertissements.push(Avertissement::VoisinRestant {
                    chemin: chemin.to_path_buf(),
                    erreur,
                });
            } else if let Err(erreur) = fichier::rendre_durable_apres_le_renommage(&cible) {
                avertissements.push(Avertissement::NonDurable {
                    chemin: cible,
                    erreur,
                });
            }
        }
    }
}

/// Le sort du `.partiel` `chemin` du dossier d'un document, de contenu `octets`.
pub(crate) fn sort_dans_le_dossier(chemin: &Path, octets: &[u8]) -> Sort {
    demarrage::sort_d_un_partiel(Partiel::DansLeDossier {
        nom: &nom_de(chemin),
        lisible: voisin_lisible(octets),
    })
}

/// Le sort du `.partiel` `chemin` d'`ajouts/`, de contenu `octets`.
pub(crate) fn sort_dans_ajouts(chemin: &Path, octets: &[u8]) -> Sort {
    demarrage::sort_d_un_partiel(Partiel::DansAjouts {
        nom: &nom_de(chemin),
        empreinte: &empreinte(octets),
    })
}

/// Le voisin du fichier complet de contenu `octets` est lisible : il n'est pas vide, et
/// `Automerge::load` le relit sans erreur. Vide, Automerge le lirait comme un document neuf.
pub(crate) fn voisin_lisible(octets: &[u8]) -> bool {
    !octets.is_empty() && Automerge::load(octets).is_ok()
}

/// Le nombre de voisins du fichier complet du dossier `dossier` dont le sort est `Lire` : non
/// vides, et qu'`Automerge::load` relit sans erreur. Chaque `.partiel` se lit par
/// `fichier::lire`.
pub(crate) fn voisins_du_complet_lisibles(dossier: &Path) -> Result<usize, Erreur> {
    let mut lisibles = 0;
    for chemin in partiels(dossier)? {
        let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        if sort_dans_le_dossier(&chemin, &octets) == Sort::Lire {
            lisibles += 1;
        }
    }
    Ok(lisibles)
}
