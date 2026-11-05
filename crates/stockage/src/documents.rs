//! Les documents sur fichiers (§ 31 · 2.1) : chacun dans son dossier, sous `documents/`, porté
//! par son fichier complet et par un fichier d'ajout par écriture faite depuis. L'essai ne
//! connaît que le fonds, qui naît une seule fois (§ 31 · 2.3).

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{ActorId, Automerge, AutomergeError, Change, ROOT, ReadDoc};
use noyau::donnees::fonds::{GENESE, IDENTIFIANT, LIGNEE, MESSAGE_DE_LA_NAISSANCE, PROPRIETAIRE};

use crate::fichier::ecrire_a_cote;
use crate::{Erreur, RacineDesDonnees, disque, empreinte};

/// Ce que la naissance du fonds écrit sur la genèse (§ 31 · 2.3) : son identifiant, sa lignée
/// (§ 31 · 2.5) et l'identifiant de propriétaire.
#[derive(Debug, Clone, Copy)]
pub struct Naissance<'a> {
    pub identifiant: &'a str,
    pub lignee: &'a str,
    pub proprietaire: &'a str,
}

/// Un document ouvert, dans une racine des données dont ce processus tient le verrou : il ne
/// survit pas à la racine des données.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "la racine des données ne se lit pas encore : elle tient le document dans sa vie"
)]
pub struct Document<'r> {
    racine: &'r RacineDesDonnees,
    /// Le dossier du document : `documents/fonds/`.
    dossier: PathBuf,
    automerge: Automerge,
    /// Une écriture a échoué : le document refuse d'écrire jusqu'à sa réouverture.
    a_relire: bool,
}

impl Document<'_> {
    /// Le document Automerge, tel qu'il est en mémoire.
    pub fn automerge(&self) -> &Automerge {
        &self.automerge
    }

    /// § 31 · 2.1 : une modification atomique, avec son message et son heure (`commit_with`),
    /// faite une fois son fichier d'ajout en place : `save_after` des heads d'avant, sous
    /// `ajouts/<empreinte>.automerge`, écrit à côté puis renommé. Une écriture sans modification
    /// n'écrit rien. Le geste refusé, la transaction est annulée (`rollback`), `Erreur::Automerge`
    /// se rend, et le document reste ouvert. L'écriture du fichier échoue : l'erreur se rend, et
    /// le document est à relire.
    pub fn ecrire(
        &mut self,
        message: &str,
        heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
    ) -> Result<(), Erreur> {
        if self.a_relire {
            return Err(Erreur::ARelire);
        }
        let avant = self.automerge.get_heads();
        let mut tx = self.automerge.transaction();
        if let Err(erreur) = geste(&mut tx) {
            tx.rollback();
            return Err(Erreur::Automerge(erreur));
        }
        let (tete, _) = tx.commit_with(a_l_heure(message, heure));
        if tete.is_none() {
            return Ok(());
        }
        let octets = self.automerge.save_after(&avant);
        let chemin = self
            .dossier
            .join("ajouts")
            .join(format!("{}.automerge", empreinte(&octets)));
        if let Err(erreur) = ecrire_a_cote(&chemin, &octets) {
            // La modification est en mémoire, et son fichier peut être en place : l'erreur peut
            // venir après le renommage, quand il ne devient pas durable. Le document ne sait
            // plus ce que porte le disque.
            self.a_relire = true;
            return Err(disque(&chemin, erreur));
        }
        Ok(())
    }
}

/// Le fonds de `racine`, rouvert de son fichier complet puis de ses fichiers d'ajout s'il est
/// né ; sinon `None`.
pub(crate) fn rouvrir_le_fonds(racine: &RacineDesDonnees) -> Result<Option<Document<'_>>, Erreur> {
    let dossier = dossier_du_fonds(racine);
    let complet = dossier.join("complet.automerge");
    let octets = match fs::read(&complet) {
        Ok(octets) => octets,
        Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(erreur) => return Err(disque(&complet, erreur)),
    };
    let mut automerge = Automerge::load(&octets).map_err(Erreur::Automerge)?;
    lire_les_ajouts(&mut automerge, &dossier.join("ajouts"))?;
    automerge.set_actor(ActorId::random());
    Ok(Some(Document {
        racine,
        dossier,
        automerge,
        a_relire: false,
    }))
}

/// Lit chaque `<empreinte>.automerge` de `ajouts`, dans l'ordre que donne le dossier : son
/// contenu doit donner l'empreinte de son nom, sinon [`Erreur::AjoutAltere`], puis il s'ajoute
/// au document (`load_incremental`). Tout lu, aucune modification n'attend ses dépendances,
/// sinon [`Erreur::DependancesManquantes`]. Rien n'est retiré ni réécrit, et un autre nom, un
/// `.partiel` par exemple, est laissé tel quel.
fn lire_les_ajouts(automerge: &mut Automerge, ajouts: &Path) -> Result<(), Erreur> {
    let entrees = fs::read_dir(ajouts).map_err(|erreur| disque(ajouts, erreur))?;
    for entree in entrees {
        let chemin = entree.map_err(|erreur| disque(ajouts, erreur))?.path();
        if chemin.extension() != Some(OsStr::new("automerge")) {
            continue;
        }
        let octets = fs::read(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        // `load_incremental` s'arrête sans erreur au premier tronçon altéré : seule l'empreinte
        // du nom dit qu'un fichier d'ajout a changé.
        if chemin.file_stem() != Some(OsStr::new(&empreinte(&octets))) {
            return Err(Erreur::AjoutAltere { chemin });
        }
        automerge
            .load_incremental(&octets)
            .map_err(Erreur::Automerge)?;
    }
    if !automerge.get_missing_deps(&[]).is_empty() {
        return Err(Erreur::DependancesManquantes);
    }
    Ok(())
}

/// Fait naître le fonds de `racine` : la genèse, la naissance et le premier geste, en mémoire,
/// puis un seul fichier complet, écrit à côté puis renommé, une fois `ajouts/` créé.
pub(crate) fn faire_naitre_le_fonds<'r>(
    racine: &'r RacineDesDonnees,
    naissance: &Naissance<'_>,
    message: &str,
    heure: i64,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Document<'r>, Erreur> {
    let dossier = dossier_du_fonds(racine);
    let complet = dossier.join("complet.automerge");
    if complet
        .try_exists()
        .map_err(|erreur| disque(&complet, erreur))?
    {
        return Err(Erreur::FondsDejaNe);
    }
    // Rien ne s'écrit avant que le geste soit fait : refusé, il ne laisse rien.
    let automerge = naitre_en_memoire(naissance, message, heure, geste)?;
    let ajouts = dossier.join("ajouts");
    fs::create_dir_all(&ajouts).map_err(|erreur| disque(&ajouts, erreur))?;
    // Hors de Windows, le dossier du fonds ne survit à une coupure qu'une fois `documents/`
    // synchronisé ; `ajouts/`, qu'une fois le dossier du fonds synchronisé, ce que fait
    // l'écriture du fichier complet. Un dossier ne se synchronise pas sous Windows.
    #[cfg(not(windows))]
    crate::fichier::rendre_durable(&dossier)
        .map_err(|erreur| disque(&racine.chemin().join("documents"), erreur))?;
    ecrire_a_cote(&complet, &automerge.save()).map_err(|erreur| disque(&complet, erreur))?;
    Ok(Document {
        racine,
        dossier,
        automerge,
        a_relire: false,
    })
}

/// `documents/fonds/`, dans la racine des données.
fn dossier_du_fonds(racine: &RacineDesDonnees) -> PathBuf {
    racine.chemin().join("documents").join("fonds")
}

/// La genèse, puis la naissance, qui écrit l'identifiant, la lignée et le propriétaire, puis le
/// premier geste : trois modifications, dans un document Automerge neuf, dont l'acteur est
/// neuf. Le geste refusé, le document est perdu avec lui.
fn naitre_en_memoire(
    naissance: &Naissance<'_>,
    message: &str,
    heure: i64,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Automerge, Erreur> {
    let genese = Change::try_from(GENESE).map_err(|erreur| Erreur::Genese(erreur.to_string()))?;
    let mut automerge = Automerge::new();
    automerge
        .apply_changes([genese])
        .map_err(|erreur| Erreur::Genese(erreur.to_string()))?;

    let mut tx = automerge.transaction();
    for (cle, valeur) in [
        (IDENTIFIANT, naissance.identifiant),
        (LIGNEE, naissance.lignee),
        (PROPRIETAIRE, naissance.proprietaire),
    ] {
        tx.put(ROOT, cle, valeur).map_err(Erreur::Automerge)?;
    }
    tx.commit_with(a_l_heure(MESSAGE_DE_LA_NAISSANCE, heure));

    let mut tx = automerge.transaction();
    geste(&mut tx).map_err(Erreur::Automerge)?;
    tx.commit_with(a_l_heure(message, heure));
    Ok(automerge)
}

/// Le message et l'heure d'une modification.
fn a_l_heure(message: &str, heure: i64) -> CommitOptions {
    CommitOptions::default()
        .with_message(message)
        .with_time(heure)
}
