//! Le cœur de la couche de synchronisation de pupitre (§ 40 · 3) : comparer deux copies d'un
//! document, puis porter entre elles le protocole de synchronisation d'Automerge. Au lot 3, il
//! lit aussi les conflits et les relève, selon la règle du noyau (§ 70 · 1). Il n'a aucun
//! transport : le réseau se branche dessus, ou à côté.

use automerge::sync::{Message, State, SyncDoc};
use automerge::{Automerge, AutomergeError};
use noyau::donnees::copie::comparer;

pub use noyau::donnees::copie::{Copie, Difference};

/// L'échange d'une copie avec une autre copie du même document (§ 31 · 4.4) : l'état de
/// synchronisation d'Automerge, vu de la copie locale. Il ne s'ouvre qu'entre deux copies qui
/// s'échangent (§ 31 · 2.5).
#[derive(Debug)]
pub struct Echange {
    etat: State,
}

/// Ce que l'échange refuse.
#[derive(Debug)]
pub enum Erreur {
    /// Les octets reçus ne se décodent pas en message de synchronisation ; le texte est celui
    /// d'Automerge. Le document ne bouge pas.
    MessageIllisible(String),
    /// Automerge a refusé ce que le message apporte.
    Automerge(AutomergeError),
}

impl Echange {
    /// § 31 · 2.5 : compare les deux copies (`comparer`) avant tout échange. Deux copies
    /// étrangères n'en ouvrent aucun : la première différence le refuse.
    pub fn ouvrir(locale: &Copie, distante: &Copie) -> Result<Echange, Difference> {
        comparer(locale, distante)?;
        Ok(Echange { etat: State::new() })
    }

    /// Le prochain message pour l'autre copie (`SyncDoc::generate_sync_message`, puis
    /// `Message::encode`), ou `None` quand il n'y a plus rien à dire.
    pub fn prochain_message(&mut self, doc: &Automerge) -> Option<Vec<u8>> {
        doc.generate_sync_message(&mut self.etat)
            .map(Message::encode)
    }

    /// `Message::decode`, puis `SyncDoc::receive_sync_message`. Un message illisible est
    /// refusé avant toute réception.
    pub fn recevoir(&mut self, doc: &mut Automerge, message: &[u8]) -> Result<(), Erreur> {
        let message = Message::decode(message)
            .map_err(|erreur| Erreur::MessageIllisible(erreur.to_string()))?;
        doc.receive_sync_message(&mut self.etat, message)
            .map_err(Erreur::Automerge)
    }

    /// § 31 · 2.1 : la copie locale a perdu ce qu'elle avait reçu (une réception refusée, puis la
    /// relecture du disque). Son prochain message demande à l'autre copie d'oublier ce qu'elle
    /// croit avoir envoyé (`State::needs_reset`, champ public d'automerge 0.12.0 : le drapeau
    /// `SYNC_RESET`, ou des heads vides devant une copie qui ne le connaît pas), et l'autre copie
    /// le renvoie.
    pub fn reprendre(&mut self) {
        self.etat.needs_reset = true;
    }
}
