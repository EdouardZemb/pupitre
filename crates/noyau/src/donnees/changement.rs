//! Les changements (§ 31 · 3) : ce que rend le noyau pour une opération nommée, que l'adaptateur
//! `Documents` applique, en une seule modification. Le noyau les calcule sans rien écrire ; un
//! chemin nomme une clé depuis la racine du document, segment par segment.

/// Une valeur simple, posée sous une clé.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Simple {
    /// Une chaîne.
    Chaine(String),
    /// Un entier signé : une heure, en secondes.
    Entier(i64),
    /// Un entier non signé : une durée, en millisecondes.
    Naturel(u64),
    /// `true` : une étiquette ou un lien posés.
    Vrai,
}

/// Un changement d'une opération nommée.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Changement {
    /// Pose `valeur` sous la dernière clé de `chemin`, dans l'objet que nomment les autres.
    Poser { chemin: Vec<String>, valeur: Simple },
    /// Crée un objet à clés sous `chemin`, et, dans la même modification, chacun de ses `conteneurs`.
    Creer {
        chemin: Vec<String>,
        conteneurs: Vec<String>,
    },
    /// Ôte la dernière clé de `chemin`.
    Oter { chemin: Vec<String> },
}

/// Une opération nommée, telle que le noyau la calcule : ses changements, dans l'ordre où ils
/// s'appliquent, le message que le journal lira, et l'heure de sa modification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    /// Les changements, appliqués dans une seule modification.
    pub changements: Vec<Changement>,
    /// Le message de la modification, lisible (§ 31 · 3).
    pub message: String,
    /// L'heure de la modification, en secondes.
    pub heure: i64,
}
