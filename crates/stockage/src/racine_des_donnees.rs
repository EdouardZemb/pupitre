//! La racine des données d'un fonds et son verrou (§ 31 · 2.1).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

use crate::Erreur;

/// Une racine des données ouverte, dont ce processus tient le verrou : il est seul à en écrire
/// les documents tant qu'elle reste ouverte.
#[derive(Debug)]
pub struct RacineDesDonnees {
    chemin: PathBuf,
    /// Le fichier `verrou`, tenu ouvert : le système lève le verrou quand il se ferme, donc
    /// quand la racine des données se ferme, et aussi à un arrêt brutal.
    _verrou: File,
}

impl RacineDesDonnees {
    /// § 31 · 2.1 : crée au besoin le dossier, prend le verrou de `verrou`
    /// (`OpenOptions` : `create`, `write`, `truncate(false)` ; puis `File::try_lock`), et seulement
    /// ensuite crée `documents/` et `captations/`.
    ///
    /// La seule présence du fichier `verrou`, comme en emporte une copie, ne dit rien : seul
    /// compte le verrou que tient un autre processus, qui rend
    /// [`Erreur::RacineDesDonneesTenue`].
    pub fn ouvrir(chemin: &Path) -> Result<RacineDesDonnees, Erreur> {
        fs::create_dir_all(chemin).map_err(|erreur| disque(chemin, erreur))?;
        let chemin_du_verrou = chemin.join("verrou");
        let verrou = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&chemin_du_verrou)
            .map_err(|erreur| disque(&chemin_du_verrou, erreur))?;
        verrou.try_lock().map_err(|refus| match refus {
            TryLockError::WouldBlock => Erreur::RacineDesDonneesTenue,
            TryLockError::Error(erreur) => disque(&chemin_du_verrou, erreur),
        })?;
        for nom in ["documents", "captations"] {
            let dossier = chemin.join(nom);
            fs::create_dir_all(&dossier).map_err(|erreur| disque(&dossier, erreur))?;
        }
        Ok(RacineDesDonnees {
            chemin: chemin.to_path_buf(),
            _verrou: verrou,
        })
    }

    /// Le chemin de la racine des données, tel qu'il a été reçu.
    pub fn chemin(&self) -> &Path {
        &self.chemin
    }
}

fn disque(chemin: &Path, erreur: io::Error) -> Erreur {
    Erreur::Disque {
        chemin: chemin.to_path_buf(),
        erreur,
    }
}
