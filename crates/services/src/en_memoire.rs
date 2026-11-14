//! La paire d'adaptateurs en mémoire (§ 40 · 3), sous la fonction `en-memoire` : les documents
//! et les captations sans fichier, que la suite des commandes éprouve comme ceux du stockage. Le
//! fonds s'y écrit par `application`, comme sur fichiers, et s'ouvre de ses octets selon les
//! mêmes règles de la version de schéma (§ 31 · 2.4).

use std::cell::RefCell;
use std::collections::BTreeMap;

use automerge::{Automerge, AutomergeError};
use noyau::donnees::changement::Operation;
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::version::{self, Lecture, Migration};

use crate::application::{self, Inapplicable};
use crate::lecture;
use crate::{Captations, Documents, Ecriture, Raison, Refus};

/// Le message d'un fonds de version de schéma plus récente que celle que connaît l'application.
const VERSION_PLUS_RECENTE: &str =
    "Ce fonds vient d'une version plus récente de Pupitre ; celle-ci le lit sans y écrire.";
/// Le message d'un fonds dont la version de schéma ne se lit pas.
const VERSION_ILLISIBLE: &str =
    "La version de schéma du document ne se lit pas : il se lit sans écrire ni échanger.";

/// Les documents en mémoire : le fonds, s'il est né, et pourquoi il n'écrit pas.
pub struct DocumentsEnMemoire {
    automerge: Option<Automerge>,
    lecture_seule: Option<Refus>,
}

impl DocumentsEnMemoire {
    /// Des documents sans fonds, qui écrivent.
    pub fn neuf() -> Self {
        DocumentsEnMemoire {
            automerge: None,
            lecture_seule: None,
        }
    }

    /// Ouvre les octets d'un fonds selon `migrations` : `version::lecture(version_de_schema, connue)`
    /// décide ; `Lecture::Migrer` applique `a_appliquer` par `application::migrer`, et une
    /// migration refusée, qui n'a pas fait migrer le fonds, met en lecture seule
    /// (`Raison::Illisible`) ; `Lecture::LireSansEcrire` met en lecture seule
    /// (`Raison::VersionPlusRecente { version }`, ou `Raison::Illisible` sans version lue) ;
    /// `Lecture::Illisible` aussi (`Raison::Illisible`). Des octets qu'`Automerge::load` refuse :
    /// `Err`, `Raison::Illisible`.
    pub fn de(octets: &[u8], migrations: &[Migration<'_>]) -> Result<Self, Refus> {
        let mut automerge = Automerge::load(octets).map_err(refus_illisible)?;
        let lue = lecture::version_de_schema(&automerge);
        let lecture_seule = match version::lecture(lue, version::connue(migrations)) {
            Lecture::Ecrire => None,
            Lecture::Migrer { de, .. } => {
                let a_appliquer = version::a_appliquer(migrations, de);
                application::migrer(&mut automerge, a_appliquer)
                    .err()
                    .map(refus_illisible)
            }
            // `version::lecture` ne rend `LireSansEcrire` que d'une version lue ; sans elle, le
            // fonds n'écrit pas non plus.
            Lecture::LireSansEcrire => Some(match lue {
                Some(version) => Refus {
                    raison: Raison::VersionPlusRecente { version },
                    message: VERSION_PLUS_RECENTE.to_string(),
                },
                None => version_illisible(),
            }),
            Lecture::Illisible => Some(version_illisible()),
        };
        Ok(DocumentsEnMemoire {
            automerge: Some(automerge),
            lecture_seule,
        })
    }

    /// `save()` du fonds ; `None` sans fonds.
    pub fn octets(&self) -> Option<Vec<u8>> {
        self.automerge.as_ref().map(Automerge::save)
    }
}

impl Documents for DocumentsEnMemoire {
    fn fonds(&self) -> Option<&Automerge> {
        self.automerge.as_ref()
    }

    fn lecture_seule(&self) -> Option<Refus> {
        self.lecture_seule.clone()
    }

    /// Le fonds ne naît qu'une fois : déjà né, `Raison::Impossible`, et rien ne change.
    fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture {
        if self.automerge.is_some() {
            return Err(Refus {
                raison: Raison::Impossible,
                message: "Le fonds est déjà né : il ne naît qu'une fois.".to_string(),
            });
        }
        let automerge = application::faire_naitre(naissance, geste).map_err(refus_inapplicable)?;
        self.automerge = Some(automerge);
        Ok(None)
    }

    /// En lecture seule, le refus de `lecture_seule()` ; sans fonds, `Raison::Impossible`.
    fn appliquer(&mut self, operation: &Operation) -> Ecriture {
        if let Some(refus) = &self.lecture_seule {
            return Err(refus.clone());
        }
        let Some(automerge) = self.automerge.as_mut() else {
            return Err(Refus {
                raison: Raison::Impossible,
                message: "Aucun fonds n'est né.".to_string(),
            });
        };
        application::appliquer(automerge, operation).map_err(refus_inapplicable)?;
        Ok(None)
    }
}

/// Les captations en mémoire, chacune sous son empreinte.
#[derive(Default)]
pub struct CaptationsEnMemoire {
    objets: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl Captations for CaptationsEnMemoire {
    /// L'empreinte de `octets` (`noyau::donnees::identifiants::empreinte`) ; une captation déjà
    /// versée ne se range pas une seconde fois.
    fn verser(&self, octets: &[u8]) -> Result<String, Refus> {
        let e = empreinte(octets);
        self.objets
            .borrow_mut()
            .entry(e.clone())
            .or_insert_with(|| octets.to_vec());
        Ok(e)
    }

    /// Une empreinte qu'aucune captation ne porte : `Raison::Impossible`.
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus> {
        self.objets
            .borrow()
            .get(empreinte)
            .cloned()
            .ok_or_else(|| Refus {
                raison: Raison::Impossible,
                message: format!("Aucune captation ne porte l'empreinte {empreinte}."),
            })
    }
}

/// Le refus d'octets qu'Automerge ne charge pas, ou d'une migration qui ne s'y applique pas :
/// `Raison::Illisible`, son message le `Debug` de l'erreur.
fn refus_illisible(erreur: AutomergeError) -> Refus {
    Refus {
        raison: Raison::Illisible,
        message: format!("{erreur:?}"),
    }
}

/// Le refus d'un fonds dont la version de schéma ne se lit pas : `Raison::Illisible`.
fn version_illisible() -> Refus {
    Refus {
        raison: Raison::Illisible,
        message: VERSION_ILLISIBLE.to_string(),
    }
}

/// Le refus d'une opération inapplicable : `Raison::Impossible`, son message le `Debug` de
/// l'erreur.
fn refus_inapplicable(erreur: Inapplicable) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: format!("{erreur:?}"),
    }
}
