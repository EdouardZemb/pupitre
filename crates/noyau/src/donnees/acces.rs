//! L'accès d'un document ouvert (§ 31 · 2.1, § 31 · 2.2) : ce qu'il permet, écrire et se
//! compacter, échanger avec une autre copie. L'adaptateur `Documents` le tient ; les règles de
//! la compaction le reçoivent.

/// Ce que permet un document ouvert (§ 31 · 2.1, § 31 · 2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Acces {
    /// Tout se lit.
    Entier,
    /// Un fichier ne se lit pas, ou la relecture du disque a échoué.
    Illisible,
}

impl Acces {
    /// Il écrit une opération, se compacte et transmet ses modifications : `Entier` seul.
    pub fn ecrit(self) -> bool {
        matches!(self, Acces::Entier)
    }

    /// Il envoie et reçoit des messages de synchronisation : `Entier` seul.
    pub fn echange(self) -> bool {
        matches!(self, Acces::Entier)
    }
}
