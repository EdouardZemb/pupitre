//! Les contrats des adaptateurs (§ 40 · 3) : ce que les services demandent aux documents, aux
//! captations et à l'aléa, que chaque côté réalise, sur fichiers ou en mémoire.

use automerge::Automerge;
use noyau::donnees::changement::Operation;

use crate::Refus;

/// Ce que rend une écriture : faite, sans avertissement (`None`) ou avec un avertissement, en
/// français ; ou refusée.
pub type Ecriture = Result<Option<String>, Refus>;

/// Les documents : le fonds, tel qu'il se lit, et ce qui s'y écrit, une opération nommée à la fois.
pub trait Documents {
    /// Le fonds tel qu'il se lit ; `None` tant qu'aucun fonds n'est né.
    fn fonds(&self) -> Option<&Automerge>;
    /// Pourquoi le fonds n'écrit pas ; `None` s'il écrit.
    fn lecture_seule(&self) -> Option<Refus>;
    /// La genèse, `naissance`, puis `geste`, en un premier fichier complet ; refusé, rien ne reste.
    fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture;
    /// `operation`, en une modification atomique, avec son message et son heure.
    fn appliquer(&mut self, operation: &Operation) -> Ecriture;
}

/// Les captations, rangées par leur empreinte.
pub trait Captations {
    /// Verse `octets`, et rend leur empreinte.
    fn verser(&self, octets: &[u8]) -> Result<String, Refus>;
    /// Les octets de la captation qui porte `empreinte`.
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus>;
}

/// L'aléa, que le côté natif tire et que les tests écrivent.
pub trait Alea {
    /// Remplit `octets` d'aléa.
    fn remplir(&mut self, octets: &mut [u8]);
}
