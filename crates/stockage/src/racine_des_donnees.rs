//! La racine des données d'un fonds et son verrou (§ 31 · 2.1).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

use automerge::AutomergeError;
use automerge::transaction::Transaction;

use crate::{Captations, Document, Erreur, Naissance, disque, documents};

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

    /// Les captations de la racine des données, dans son dossier `captations/` (§ 31 · 6.1).
    pub fn captations(&self) -> Captations<'_> {
        Captations::de(self)
    }

    /// Le fonds, rouvert s'il est né (son `complet.automerge` existe) ; sinon `None`. Le fichier
    /// complet se lit, puis chaque fichier d'ajout (`load_incremental`) ; chaque ouverture tire
    /// un acteur neuf (`set_actor(ActorId::random())`) ; puis le fonds se compacte quand
    /// `a_l_ouverture` le dit, c'est-à-dire dès qu'il a un fichier d'ajout.
    ///
    /// Un fichier d'ajout dont le contenu ne donne pas l'empreinte de son nom rend
    /// [`Erreur::AjoutAltere`] ; une modification qui attend encore ses dépendances, tout lu,
    /// [`Erreur::DependancesManquantes`] ; un fichier qu'Automerge ne lit pas,
    /// [`Erreur::Automerge`]. Une compaction qui échoue fait échouer l'ouverture, avec son erreur.
    ///
    /// Un seul [`Document`] du fonds doit vivre à la fois : chacun se compacte de sa seule
    /// mémoire, et deux `Document` qui écrivent puis se compactent perdent les écritures de l'un.
    /// Le verrou de la racine des données écarte les autres processus, pas un second `Document`
    /// du même processus ; le lot 1 pose la garde.
    pub fn fonds(&self) -> Result<Option<Document<'_>>, Erreur> {
        documents::rouvrir_le_fonds(self)
    }

    /// § 31 · 2.3 : `GENESE`, la naissance (`MESSAGE_DE_LA_NAISSANCE`, qui écrit `IDENTIFIANT`,
    /// `LIGNEE` et `PROPRIETAIRE`), puis le premier geste, avec son message ; les trois en un
    /// seul fichier, `documents/fonds/complet.automerge`, écrit à côté puis renommé, et
    /// `documents/fonds/ajouts/` créé. Le geste refusé, rien ne s'écrit.
    ///
    /// Un fonds déjà né rend [`Erreur::FondsDejaNe`], sans que le geste soit fait.
    pub fn faire_naitre_le_fonds(
        &self,
        naissance: &Naissance<'_>,
        message: &str,
        heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
    ) -> Result<Document<'_>, Erreur> {
        documents::faire_naitre_le_fonds(self, naissance, message, heure, geste)
    }
}
