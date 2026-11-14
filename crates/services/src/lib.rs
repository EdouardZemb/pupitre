//! Les services de pupitre (§ 40 · 3) : les opérations nommées et les lectures du fonds,
//! conduites sur les contrats `Documents` et `Captations`, que réalisent les adaptateurs, sur
//! fichiers (le stockage) ou en mémoire. `application` applique les changements que le noyau
//! calcule ; `lecture` tire du document le fonds lu, que le noyau reçoit.
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
