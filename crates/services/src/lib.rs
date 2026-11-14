//! Les services de pupitre (§ 40 · 3) : les opérations nommées et les lectures du fonds,
//! conduites sur les contrats `Documents` et `Captations`, que réalisent les adaptateurs, sur
//! fichiers (le stockage) ou en mémoire. `Services` conduit chaque commande : il tire de l'aléa
//! du `Contexte` les identifiants qu'elle forme, demande l'opération au noyau, l'écrit par
//! `Documents`, et rend la réponse aux types de `commandes`. `application` applique les
//! changements que le noyau calcule ; `lecture` tire du document le fonds lu, que le noyau
//! reçoit.
//!
//! La fonction `en-memoire`, que seuls les tests demandent, compile le module `en_memoire`, la
//! paire d'adaptateurs en mémoire.

pub mod application;
mod contrats;
#[cfg(feature = "en-memoire")]
pub mod en_memoire;
pub mod lecture;

pub use commandes::{Raison, Refus};
pub use contrats::{Alea, Captations, Documents, Ecriture};

use commandes::{Captation, EntreeDuJournal, Notee, Noter, Reponse, VueDUneIdee};
use noyau::donnees::changement::Operation;
use noyau::donnees::gestes::{self, Impossible};
use noyau::donnees::identifiants::{ulid, uuid_v4};

/// Ce que le côté natif donne à une commande qui écrit : l'heure du geste et l'aléa, des
/// arguments, jamais l'horloge ni le hasard des services.
pub struct Contexte<'a> {
    /// L'heure du geste, en secondes depuis 1970.
    pub heure: i64,
    /// L'aléa d'où se tirent les identifiants que forme la commande.
    pub alea: &'a mut dyn Alea,
}

/// Les services d'un appareil : ses commandes, conduites sur ses documents et ses captations.
pub struct Services<D: Documents, C: Captations> {
    documents: D,
    captations: C,
    appareil: gestes::Appareil,
}

impl<D: Documents, C: Captations> Services<D, C> {
    /// Les services de l'appareil `appareil`, sur `documents` et `captations`.
    pub fn new(documents: D, captations: C, appareil: gestes::Appareil) -> Self {
        Services {
            documents,
            captations,
            appareil,
        }
    }

    /// Les documents.
    pub fn documents(&self) -> &D {
        &self.documents
    }

    /// Les documents, à modifier.
    pub fn documents_mut(&mut self) -> &mut D {
        &mut self.documents
    }

    /// Les captations.
    pub fn captations(&self) -> &C {
        &self.captations
    }

    /// Les documents et les captations, rendus : les services sont fermés.
    pub fn en_parties(self) -> (D, C) {
        (self.documents, self.captations)
    }

    /// Noter une idée, sur l'appareil, à l'heure du contexte. En lecture seule, le refus de
    /// `lecture_seule()`, tel quel, sans rien tirer. Sans fonds, la naissance : l'ULID du
    /// document, puis celui de la lignée (dix octets chacun), l'identifiant de propriétaire
    /// (seize), l'ULID de l'idée (dix), dans cet ordre ; puis `Documents::faire_naitre`. Avec un
    /// fonds, l'ULID de l'idée, puis `Documents::appliquer`. Un texte vide, une fois ses blancs
    /// ôtés : `Raison::Impossible`, et rien ne s'écrit.
    pub fn noter(&mut self, commande: &Noter, contexte: &mut Contexte<'_>) -> Reponse<Notee> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let naissance = self
            .documents
            .fonds()
            .is_none()
            .then(|| naissance_tiree(contexte));
        let idee = ulid_tire(contexte);
        let geste = gestes::noter(
            &idee,
            &commande.texte,
            self.appareil,
            contexte.heure,
            commande.decalage_minutes,
        )
        .map_err(refus_impossible)?;
        let avertissement = match naissance {
            Some(naissance) => self.documents.faire_naitre(&naissance, &geste)?,
            None => self.documents.appliquer(&geste)?,
        };
        Ok(Notee {
            idee,
            avertissement,
        })
    }

    /// La vue de l'idée `idee`, telle qu'elle se lit, en lecture seule comme ailleurs : sa
    /// provenance, sa date, son texte, ses étiquettes, ses captations et la date de son retrait.
    /// Ses liens (§ 30 · 9.3), son état de reliée (§ 30 · 9.1) et sa durée (§ 30 · 9.2)
    /// attendent que les services écrivent des liens et des prises : jusque-là, `liens` est
    /// vide, `reliee` vaut `false` et `duree_ms`, `None`, ce que rien ne peut démentir. Sans
    /// fonds, ou sans idée qui se lise sous cet identifiant : `Raison::Impossible`.
    pub fn vue_d_une_idee(&self, idee: &str) -> Reponse<VueDUneIdee> {
        let fonds = self.documents.fonds().ok_or_else(|| idee_absente(idee))?;
        let lue = lecture::lire_l_idee(fonds, idee).ok_or_else(|| idee_absente(idee))?;
        let captations = lecture::captations_de(fonds, idee)
            .into_iter()
            .map(|captation| Captation {
                sorte: sorte_des_commandes(captation.sorte),
                empreinte: captation.empreinte,
            })
            .collect();
        Ok(VueDUneIdee {
            idee: lue.id,
            provenance: provenance_des_commandes(lue.provenance),
            le: lue.le,
            texte: lue.texte,
            etiquettes: lue.etiquettes.into_iter().collect(),
            liens: Vec::new(),
            captations,
            reliee: false,
            duree_ms: None,
            retiree: lue.retiree,
        })
    }

    /// Le journal du fonds : le message et l'heure de chaque modification, en lecture seule
    /// comme ailleurs ; sans fonds, aucune entrée.
    pub fn journal(&self) -> Reponse<Vec<EntreeDuJournal>> {
        Ok(self
            .documents
            .fonds()
            .map(lecture::journal)
            .unwrap_or_default()
            .into_iter()
            .map(|(message, heure)| EntreeDuJournal { message, heure })
            .collect())
    }
}

/// La naissance d'un fonds, à l'heure du contexte : l'ULID du document, celui de la lignée, puis
/// l'identifiant de propriétaire, tirés dans cet ordre.
fn naissance_tiree(contexte: &mut Contexte<'_>) -> Operation {
    let identifiant = ulid_tire(contexte);
    let lignee = ulid_tire(contexte);
    let proprietaire = identifiant_de_proprietaire(contexte.alea);
    gestes::naissance(&identifiant, &lignee, &proprietaire, contexte.heure)
}

/// Un ULID à l'heure du contexte, de dix octets d'aléa (§ 30 · 2).
fn ulid_tire(contexte: &mut Contexte<'_>) -> String {
    let mut alea = [0u8; 10];
    contexte.alea.remplir(&mut alea);
    ulid(contexte.heure, alea)
}

/// La seule fonction qui forme l'identifiant de propriétaire (§ 30 · 2) : seize octets d'aléa,
/// puis `uuid_v4`.
fn identifiant_de_proprietaire(alea: &mut dyn Alea) -> String {
    let mut octets = [0u8; 16];
    alea.remplir(&mut octets);
    uuid_v4(octets)
}

/// Le refus d'une idée qu'aucun fonds ne porte, ou qui ne se lit pas : `Raison::Impossible`.
fn idee_absente(idee: &str) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: format!("Aucune idée ne porte l'identifiant {idee}."),
    }
}

/// Le refus d'un geste que le noyau dit impossible : `Raison::Impossible`, son message le
/// constat du noyau.
fn refus_impossible(impossible: Impossible) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: impossible.to_string(),
    }
}

/// L'appareil des commandes qui répond à celui du noyau.
fn appareil_des_commandes(appareil: gestes::Appareil) -> commandes::Appareil {
    match appareil {
        gestes::Appareil::Etabli => commandes::Appareil::Etabli,
        gestes::Appareil::Carnet => commandes::Appareil::Carnet,
    }
}

/// La provenance des commandes qui répond à celle du noyau.
fn provenance_des_commandes(provenance: gestes::Provenance) -> commandes::Provenance {
    match provenance {
        gestes::Provenance::Captee { par } => commandes::Provenance::Captee {
            par: appareil_des_commandes(par),
        },
        gestes::Provenance::Importee { par, depuis } => commandes::Provenance::Importee {
            par: appareil_des_commandes(par),
            depuis,
        },
    }
}

/// La sorte des commandes qui répond à celle du noyau.
fn sorte_des_commandes(sorte: gestes::Sorte) -> commandes::Sorte {
    match sorte {
        gestes::Sorte::Prise => commandes::Sorte::Prise,
        gestes::Sorte::MidiVerseSeul => commandes::Sorte::MidiVerseSeul,
        gestes::Sorte::TexteVerse => commandes::Sorte::TexteVerse,
    }
}
