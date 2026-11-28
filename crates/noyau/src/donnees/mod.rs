//! Les règles des données et le schéma du fonds, sans entrée-sortie : les adaptateurs du
//! stockage et de la synchronisation s'en servent (§ 40 · 3).

pub mod acces;
pub mod boite;
pub mod changement;
pub mod compaction;
pub mod copie;
pub mod demarrage;
pub mod entete;
pub mod fonds;
pub mod gestes;
pub mod idee;
pub mod identifiants;
pub mod import;
pub mod index;
pub mod journal;
pub mod origine;
pub mod protocole;
pub mod rapport;
pub mod repli;
pub mod requete;
pub mod signature;
pub mod stems;
pub mod teneur;
pub mod version;
