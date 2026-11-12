//! La racine des données d'un fonds et son verrou (§ 31 · 2.1).

use std::fs::{self, File, OpenOptions, TryLockError};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

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
    /// Un [`Document`] du fonds vit dans ce processus. Un `AtomicBool`, pour que la racine des
    /// données reste `Sync` (l'état de l'application, en tranche 3).
    fonds_ouvert: AtomicBool,
}

/// La garde du fonds, tenue le temps d'une ouverture : tant que le [`Document`] n'est pas né,
/// elle se rend d'elle-même à la fin de la portée, quelle que soit la sortie (`Err`, `Ok(None)`).
/// Le [`Document`] la reprend par [`Garde::passer_au_document`] et la rend à sa fin.
pub(crate) struct Garde<'r> {
    racine: &'r RacineDesDonnees,
}

impl Garde<'_> {
    /// Le document né tient la garde : il la rendra à sa fin, pas cette garde-ci.
    pub(crate) fn passer_au_document(self) {
        mem::forget(self);
    }
}

impl Drop for Garde<'_> {
    fn drop(&mut self) {
        self.racine.rendre_la_garde();
    }
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
            fonds_ouvert: AtomicBool::new(false),
        })
    }

    /// Prend la garde du fonds (`compare_exchange(false, true)`) : un seul [`Document`] à la
    /// fois. Prise, elle est à qui l'a prise ; sinon [`Erreur::FondsDejaOuvert`], et la garde
    /// reste à son titulaire.
    pub(crate) fn prendre_la_garde(&self) -> Result<Garde<'_>, Erreur> {
        self.fonds_ouvert
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| Erreur::FondsDejaOuvert)?;
        Ok(Garde { racine: self })
    }

    /// Rend la garde : à n'appeler que pour la garde qu'on a prise.
    pub(crate) fn rendre_la_garde(&self) {
        self.fonds_ouvert.store(false, Ordering::Release);
    }

    /// Le chemin de la racine des données, tel qu'il a été reçu.
    pub fn chemin(&self) -> &Path {
        &self.chemin
    }

    /// Les captations de la racine des données, dans son dossier `captations/` (§ 31 · 6.1).
    pub fn captations(&self) -> Captations<'_> {
        Captations::de(self)
    }

    /// § 31 · 2.1 : le démarrage, puis le fonds, rouvert s'il est né ; sinon `None`.
    ///
    /// La garde prise, le démarrage donne son sort à chaque `.partiel` de `documents/`, du
    /// dossier du fonds et de son `ajouts/`, dans l'ordre des noms : le voisin d'un fichier
    /// d'ajout dont le contenu donne l'empreinte que porte le nom de ce fichier prend sa place ;
    /// le voisin du fichier complet, s'il n'est pas vide et qu'`Automerge::load` le relit, se
    /// lit ; tout autre se retire. Un voisin d'un fichier d'ajout qui ne peut prendre sa place
    /// se lit où il est, et le fonds s'ouvre avec
    /// [`Avertissement::VoisinRestant`](crate::Avertissement::VoisinRestant) ; un renommage du
    /// démarrage non rendu durable, avec
    /// [`Avertissement::NonDurable`](crate::Avertissement::NonDurable). Un retrait refusé est tu,
    /// et se retente à l'ouverture suivante. Le démarrage ne crée aucun dossier dans
    /// `documents/`, et laisse tel quel un fichier qui n'est pas un `.partiel` ; un `ajouts/`
    /// absent se recrée à la première écriture ou réception. Un refus du disque à la lecture d'un
    /// dossier qui n'est pas absent, ou d'un `.partiel` du dossier du fonds ou de `ajouts/` (ceux
    /// de `documents/` ne se lisent pas), rend [`Erreur::Disque`] : un voisin non lu peut porter
    /// une opération dite faite, ou le premier fichier complet.
    ///
    /// Le fonds est né quand son `complet.automerge` existe, ou un voisin lisible de celui-ci.
    /// Le fichier complet se lit, puis chaque voisin du fichier complet, puis chaque fichier
    /// d'ajout, chacun dans l'ordre des noms (`load_incremental`) ; chaque ouverture tire un
    /// acteur neuf (`set_actor(ActorId::random())`) ; puis le fonds se compacte quand
    /// `a_l_ouverture` le dit, c'est-à-dire dès qu'il a un fichier d'ajout ou un voisin du
    /// fichier complet, s'il écrit, et la compaction retire le voisin qu'elle absorbe.
    ///
    /// Enfin, même en lecture seule, chaque nom provisoire de `captations/` dont le contenu
    /// entier donne une empreinte que le fonds nomme, tel qu'il se lit, et qu'aucune captation
    /// rangée ne porte encore, prend ce nom (§ 31 · 6.1) ; de deux noms provisoires du même
    /// contenu, le premier dans l'ordre des noms. Aucun nom provisoire n'est retiré. Un nom
    /// provisoire qui ne se lit pas, ou ne peut prendre ce nom, reste, et le fonds s'ouvre avec
    /// [`Avertissement::NomProvisoireRestant`](crate::Avertissement::NomProvisoireRestant) ; il
    /// se range à l'ouverture suivante.
    ///
    /// Un fichier illisible ouvre le fonds en lecture seule ([`Document::lecture_seule`]), en se
    /// nommant : le fonds montre ce qui se lit, n'écrit ni ne se compacte, n'envoie ni ne reçoit.
    /// C'est un fichier complet vide, ou qu'`Automerge::load` refuse, ou un voisin lisible du
    /// fichier complet que `load_incremental` refuse (`CompletIllisible`) ; un fichier d'ajout
    /// dont le contenu ne donne pas l'empreinte de son nom (`AjoutAltere`), ou que
    /// `load_incremental` refuse (`AjoutIllisible`) ; tout lu, une modification qui attend encore
    /// ses dépendances (`DependancesManquantes`). La première raison trouvée l'emporte, dans cet
    /// ordre. Un refus du disque à la lecture rend [`Erreur::Disque`]. Une compaction qui échoue
    /// n'empêche pas d'ouvrir : son erreur vient en avertissement ([`Document::avertissements`]),
    /// rien n'est perdu, et l'écriture suivante la retente, et retire ce qu'elle n'a pas retiré.
    ///
    /// Un seul [`Document`] du fonds doit vivre à la fois : chacun se compacte de sa seule
    /// mémoire, et deux `Document` qui écrivent puis se compactent perdent les écritures de l'un.
    /// Le verrou de la racine des données écarte les autres processus, pas un second `Document`
    /// du même processus : la garde de la racine des données s'en charge. Un `Document` déjà
    /// vivant : [`Erreur::FondsDejaOuvert`], avant toute lecture. La garde se prend avant la
    /// lecture et se rend si elle rend `Err` ou `None` ; le `Document` la rend à sa fin.
    pub fn fonds(&self) -> Result<Option<Document<'_>>, Erreur> {
        documents::rouvrir_le_fonds(self)
    }

    /// § 31 · 2.3 : `GENESE`, la naissance (`MESSAGE_DE_LA_NAISSANCE`, qui écrit `IDENTIFIANT`,
    /// `LIGNEE` et `PROPRIETAIRE`), puis le premier geste, avec son message ; les trois en un
    /// seul fichier, `documents/fonds/complet.automerge`, écrit à côté puis renommé, et
    /// `documents/fonds/ajouts/` créé. Le geste refusé, rien ne s'écrit. Le fichier complet
    /// refusé avant la fin de son renommage rend [`Erreur::Disque`], et seul `ajouts/` reste,
    /// vide ; en place, mais son renommage non rendu durable, le fonds naît, avec cet
    /// avertissement ([`Document::avertissements`]).
    ///
    /// Un fonds déjà né rend [`Erreur::FondsDejaNe`], sans que le geste soit fait : son
    /// `complet.automerge` existe, ou un voisin lisible de celui-ci, que la naissance ne retire
    /// pas, et que l'ouverture lit. Un `Document` du fonds déjà vivant rend
    /// [`Erreur::FondsDejaOuvert`], avant toute lecture.
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
