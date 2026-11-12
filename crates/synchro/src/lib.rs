//! Le cœur de la couche de synchronisation de pupitre (§ 40 · 3) : comparer deux copies d'un
//! document, puis porter entre elles le protocole de synchronisation d'Automerge. Au lot 3, il
//! lit aussi les conflits et les relève, selon la règle du noyau (§ 70 · 1). Il n'a aucun
//! transport : le réseau se branche dessus, ou à côté.

use automerge::sync::{ChunkList, Message, State, SyncDoc};
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

    /// Le prochain message, sans aucune modification (§ 31 · 2.4) : celui de
    /// `generate_sync_message`, `changes` remplacé par `ChunkList::empty()` et `heads` par celles
    /// de l'autre copie (`State::their_heads`, champ public d'automerge 0.12.0, vide tant qu'elle
    /// n'a rien dit). Sans ces heads, l'autre copie redemanderait sans fin ce qu'elle voit
    /// annoncé, et l'échange ne finirait pas ; vides, elles la font répondre par son document
    /// entier. `generate_sync_message` compte comme envoyé (`State::sent_hashes`) ce que le
    /// message aurait porté : un échange qui a préparé un tel message ne revient pas à
    /// `prochain_message`, il se rouvre, sinon les modifications de la copie, comptées comme
    /// envoyées, ne partiraient jamais.
    ///
    /// Quand l'autre copie a annoncé des heads vides : `None`, avant `generate_sync_message`.
    /// Seule une copie qui envoie sans modification, avant d'avoir rien entendu, en annonce : un
    /// fonds né porte toujours sa genèse, et une copie qui reprend l'échange après une réception
    /// refusée (`reprendre`) a déjà reçu un message de l'autre copie, qui dit connaître le
    /// drapeau `SYNC_RESET` : elle annonce ses heads, avec ce drapeau. Deux copies qui ne
    /// transmettent aucune modification n'ont rien à se dire. Sans cet arrêt, chacune prendrait
    /// les heads vides de l'autre pour une copie qui a tout perdu, lui répondrait par des heads
    /// vides, et l'échange ne finirait pas.
    pub fn prochain_message_sans_modification(&mut self, doc: &Automerge) -> Option<Vec<u8>> {
        if self.etat.their_heads.as_ref().is_some_and(Vec::is_empty) {
            return None;
        }
        let mut message = doc.generate_sync_message(&mut self.etat)?;
        message.changes = ChunkList::empty();
        message.heads = self.etat.their_heads.clone().unwrap_or_default();
        Some(message.encode())
    }

    /// `Message::decode`, puis `SyncDoc::receive_sync_message`. Un message illisible est
    /// refusé avant toute réception.
    pub fn recevoir(&mut self, doc: &mut Automerge, message: &[u8]) -> Result<(), Erreur> {
        let message = Message::decode(message)
            .map_err(|erreur| Erreur::MessageIllisible(erreur.to_string()))?;
        doc.receive_sync_message(&mut self.etat, message)
            .map_err(Erreur::Automerge)
    }

    /// § 31 · 2.1 : la copie locale a perdu ce qu'elle avait reçu (une réception dont l'écriture
    /// est refusée, puis la relecture du disque). Son prochain message demande à l'autre copie
    /// d'oublier ce qu'elle croit avoir envoyé (`State::needs_reset`, champ public d'automerge
    /// 0.12.0 : le drapeau `SYNC_RESET`, ou des heads vides devant une copie qui ne le connaît
    /// pas, que `prochain_message_sans_modification` remplace par celles de l'autre copie, ne
    /// gardant que le drapeau), et l'autre copie le renvoie.
    pub fn reprendre(&mut self) {
        self.etat.needs_reset = true;
    }
}
