//! L'accès d'un document ouvert (§ 31 · 2.1, § 31 · 2.2, § 31 · 2.4) : ce qu'il permet, écrire
//! et se compacter, échanger avec une autre copie. L'adaptateur `Documents` le tient ; les règles
//! de la compaction le reçoivent.

/// Ce que permet un document ouvert (§ 31 · 2.1, § 31 · 2.2, § 31 · 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acces {
    /// Tout se lit.
    Entier,
    /// Un fichier ne se lit pas, la relecture du disque a échoué, ou la version de schéma ne se
    /// lit pas.
    Illisible,
    /// Sa version de schéma est plus récente que celle que connaît l'application : il se lit, et
    /// échange, sans écrire d'opération ni se compacter.
    PlusRecent,
}

impl Acces {
    /// Il écrit une opération, se compacte et transmet ses modifications : `Entier` seul.
    pub fn ecrit(self) -> bool {
        matches!(self, Acces::Entier)
    }

    /// Il envoie et reçoit des messages de synchronisation : `Entier` et `PlusRecent`.
    pub fn echange(self) -> bool {
        matches!(self, Acces::Entier | Acces::PlusRecent)
    }
}
