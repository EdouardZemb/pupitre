//! La paire d'adaptateurs en mémoire (§ 40 · 3), sous la fonction `en-memoire` : les documents
//! et les captations sans fichier, que la suite des commandes éprouve comme ceux du stockage. Le
//! fonds s'y écrit par `application`, comme sur fichiers, et s'ouvre de ses octets selon les
//! mêmes règles de la version de schéma (§ 31 · 2.4). Les documents peuvent changer d'acteur
//! toutes les quelques opérations (`changer_d_acteur_tous_les`) : le test 13 y trouve ses 3 000
//! acteurs, sans 3 000 réouvertures (question 29).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;

use automerge::{ActorId, Automerge, AutomergeError};
use noyau::donnees::changement::Operation;
use noyau::donnees::demarrage::Provisoire;
use noyau::donnees::identifiants::Hacheur;
use noyau::donnees::version::{self, Lecture, Migration};

use crate::application::{self, Inapplicable};
use crate::contrats::plage;
use crate::lecture;
use crate::source_en_memoire::FICHIER_A_VERSER_REFUSE;
use crate::{Alea, Captations, Documents, Ecriture, FONDS_NON_NE, Raison, Refus, Versement};

/// Le message d'une captation qu'aucune ne porte, sans son empreinte (question 26), comme sur
/// fichiers.
const CAPTATION_ABSENTE: &str = "Cette captation n'est pas dans le fonds.";

/// Le message d'un fonds de version de schéma plus récente que celle que connaît l'application.
const VERSION_PLUS_RECENTE: &str =
    "Ce fonds vient d'une version plus récente de Pupitre ; cette version-ci le lit sans y écrire.";
/// Le message d'un fonds dont la version de schéma ne se lit pas.
const VERSION_ILLISIBLE: &str =
    "La version de schéma du document ne se lit pas : il se lit sans écrire ni échanger.";
/// Le message d'un fonds dont une migration ne s'applique pas, à son ouverture.
const MIGRATION_REFUSEE: &str =
    "Une migration de ce fonds ne s'applique pas : il se lit sans écrire ni échanger.";

/// Les documents en mémoire : le fonds, s'il est né, pourquoi il n'écrit pas, et ses acteurs
/// tournants, s'il en a.
pub struct DocumentsEnMemoire {
    automerge: Option<Automerge>,
    lecture_seule: Option<Refus>,
    acteurs: Option<ActeursTournants>,
}

/// Les acteurs tournants (question 29) : un acteur neuf après chaque série de `pas` opérations
/// appliquées, fait de seize octets tirés d'`alea` ; `appliquees`, les opérations appliquées
/// depuis le dernier acteur neuf, ou depuis le premier.
struct ActeursTournants {
    pas: usize,
    alea: Box<dyn Alea>,
    appliquees: usize,
}

impl DocumentsEnMemoire {
    /// Des documents sans fonds, qui écrivent.
    pub fn neuf() -> Self {
        DocumentsEnMemoire {
            automerge: None,
            lecture_seule: None,
            acteurs: None,
        }
    }

    /// Change d'acteur après chaque série de `operations` opérations appliquées, une au moins,
    /// l'acteur fait de seize octets tirés de `alea` (`set_actor`, `ActorId::from`) (question 29).
    /// Compte, comme une opération, chaque naissance faite (`faire_naitre`, la genèse et la
    /// naissance venant avec son geste) et chaque `appliquer` fait ; une opération refusée ne
    /// compte pas. Le compte part de zéro à chaque appel. Rouverts de leurs octets (`de`), les
    /// documents ne changent plus d'acteur.
    pub fn changer_d_acteur_tous_les(&mut self, operations: usize, alea: Box<dyn Alea>) {
        self.acteurs = Some(ActeursTournants {
            pas: operations.max(1),
            alea,
            appliquees: 0,
        });
    }

    /// Une opération appliquée : avec des acteurs tournants, elle compte, et la dernière d'une
    /// série fait changer d'acteur ; sans eux, ou sans fonds, rien.
    fn compter_une_operation(&mut self) {
        let (Some(automerge), Some(acteurs)) = (self.automerge.as_mut(), self.acteurs.as_mut())
        else {
            return;
        };
        acteurs.appliquees += 1;
        if acteurs.appliquees >= acteurs.pas {
            acteurs.appliquees = 0;
            let mut octets = [0u8; 16];
            acteurs.alea.remplir(&mut octets);
            automerge.set_actor(ActorId::from(octets));
        }
    }

    /// Ouvre les octets d'un fonds selon `migrations` :
    /// `version::lecture(version_de_schema, connue)` décide ; `Lecture::Migrer` applique
    /// `a_appliquer` par `application::migrer`, et une migration refusée, qui n'a pas fait migrer
    /// le fonds, met en lecture seule (`Raison::Illisible`, son message celui d'une migration qui
    /// ne s'applique pas, comme `LectureSeule::MigrationRefusee` sur fichiers) ;
    /// `Lecture::LireSansEcrire` met en lecture seule (`Raison::VersionPlusRecente { version }`,
    /// ou `Raison::Illisible` sans version lue) ; `Lecture::Illisible` aussi
    /// (`Raison::Illisible`). Des octets qu'`Automerge::load` refuse : `Err`, `Raison::Illisible`.
    pub fn de(octets: &[u8], migrations: &[Migration<'_>]) -> Result<Self, Refus> {
        let mut automerge = Automerge::load(octets).map_err(refus_illisible)?;
        let lue = lecture::version_de_schema(&automerge);
        let lecture_seule = match version::lecture(lue, version::connue(migrations)) {
            Lecture::Ecrire => None,
            Lecture::Migrer { de, .. } => {
                let a_appliquer = version::a_appliquer(migrations, de);
                application::migrer(&mut automerge, a_appliquer)
                    .is_err()
                    .then(migration_refusee)
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
            acteurs: None,
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

    /// Aucun : en mémoire, rien ne s'écrit sur un disque qui avertirait.
    fn avertissements(&self) -> Vec<String> {
        Vec::new()
    }

    /// Le fonds ne naît qu'une fois : déjà né, `Raison::Impossible`, et rien ne change. Née, une
    /// opération qui compte pour les acteurs tournants.
    fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture {
        if self.automerge.is_some() {
            return Err(Refus {
                raison: Raison::Impossible,
                message: "Le fonds est déjà né : il ne naît qu'une fois.".to_string(),
            });
        }
        let automerge = application::faire_naitre(naissance, geste).map_err(refus_inapplicable)?;
        self.automerge = Some(automerge);
        self.compter_une_operation();
        Ok(None)
    }

    /// En lecture seule, le refus de `lecture_seule()` ; sans fonds, `Raison::Impossible`.
    /// Appliquée, une opération qui compte pour les acteurs tournants.
    fn appliquer(&mut self, operation: &Operation) -> Ecriture {
        if let Some(refus) = &self.lecture_seule {
            return Err(refus.clone());
        }
        let Some(automerge) = self.automerge.as_mut() else {
            return Err(Refus {
                raison: Raison::Impossible,
                message: FONDS_NON_NE.to_string(),
            });
        };
        application::appliquer(automerge, operation).map_err(refus_inapplicable)?;
        self.compter_une_operation();
        Ok(None)
    }
}

/// Les captations en mémoire, chacune sous son empreinte. Aucune copie n'y passe par un nom
/// provisoire.
#[derive(Default)]
pub struct CaptationsEnMemoire {
    objets: RefCell<BTreeMap<String, Vec<u8>>>,
}

impl Captations for CaptationsEnMemoire {
    /// Copie `flux` dans un `Vec`, et calcule son empreinte par `Hacheur` ; `base` ne sert pas.
    /// `deja_rangee`, d'une empreinte présente ; `attendue` autre que l'empreinte calculée, rien
    /// ne se range, `rangee` faux (V7) ; sinon la captation se range, si elle ne l'est pas déjà.
    /// Un flux qui ne se lit plus : `Raison::Disque`, au constat neutre.
    fn verser_en_flux(
        &self,
        _base: &str,
        flux: &mut dyn io::Read,
        attendue: Option<&str>,
    ) -> Result<Versement, Refus> {
        let mut octets = Vec::new();
        io::Read::read_to_end(flux, &mut octets).map_err(|_| Refus {
            raison: Raison::Disque,
            message: FICHIER_A_VERSER_REFUSE.to_string(),
        })?;
        let mut hacheur = Hacheur::nouveau();
        hacheur.ajouter(&octets);
        let e = hacheur.finir();
        let deja_rangee = self.objets.borrow().contains_key(&e);
        let rangee = attendue.is_none_or(|attendue| attendue == e);
        let compte = octets.len() as u64;
        if rangee && !deja_rangee {
            self.objets.borrow_mut().insert(e.clone(), octets);
        }
        Ok(Versement {
            empreinte: e,
            octets: compte,
            deja_rangee,
            rangee,
        })
    }

    /// Une empreinte qu'aucune captation ne porte : `Raison::Impossible`, « Cette captation n'est
    /// pas dans le fonds. ».
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus> {
        self.objets
            .borrow()
            .get(empreinte)
            .cloned()
            .ok_or_else(captation_absente)
    }

    /// La plage bornée au contenu ; une empreinte qu'aucune captation ne porte, comme `relire`.
    fn relire_une_plage(
        &self,
        empreinte: &str,
        debut: u64,
        longueur: u64,
    ) -> Result<Vec<u8>, Refus> {
        let objets = self.objets.borrow();
        let octets = objets.get(empreinte).ok_or_else(captation_absente)?;
        Ok(plage(octets, debut, longueur).to_vec())
    }

    fn taille(&self, empreinte: &str) -> Result<Option<u64>, Refus> {
        Ok(self
            .objets
            .borrow()
            .get(empreinte)
            .map(|octets| octets.len() as u64))
    }

    fn tailles(&self) -> Result<BTreeSet<u64>, Refus> {
        Ok(self
            .objets
            .borrow()
            .values()
            .map(|octets| octets.len() as u64)
            .collect())
    }

    /// Aucune : en mémoire, aucune copie ne passe par un nom provisoire.
    fn copies_de_l_import(&self, _import: &str) -> Result<Vec<Provisoire>, Refus> {
        Ok(Vec::new())
    }

    /// Rien à retirer : en mémoire, aucune copie ne passe par un nom provisoire.
    fn retirer_une_copie(&self, _nom: &str) -> Result<(), Refus> {
        Ok(())
    }
}

/// Le refus d'une empreinte qu'aucune captation ne porte : `Raison::Impossible`.
fn captation_absente() -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: CAPTATION_ABSENTE.to_string(),
    }
}

/// Le refus d'octets qu'Automerge ne charge pas : `Raison::Illisible`, son message un constat,
/// sans le texte de l'erreur d'Automerge.
fn refus_illisible(_: AutomergeError) -> Refus {
    Refus {
        raison: Raison::Illisible,
        message: "Le document ne se lit pas.".to_string(),
    }
}

/// Le refus d'un fonds dont la version de schéma ne se lit pas : `Raison::Illisible`.
fn version_illisible() -> Refus {
    Refus {
        raison: Raison::Illisible,
        message: VERSION_ILLISIBLE.to_string(),
    }
}

/// Le refus d'un fonds dont une migration ne s'applique pas (`application::migrer` la refuse :
/// elle ne se lit pas, ou attend ses dépendances) : `Raison::Illisible`.
fn migration_refusee() -> Refus {
    Refus {
        raison: Raison::Illisible,
        message: MIGRATION_REFUSEE.to_string(),
    }
}

/// Le refus d'une opération inapplicable : `Raison::Impossible`, son message le texte de
/// l'erreur, celui que rend aussi la paire sur fichiers.
fn refus_inapplicable(erreur: Inapplicable) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: erreur.to_string(),
    }
}
