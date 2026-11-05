//! Les documents sur fichiers (§ 31 · 2.1) : chacun dans son dossier, sous `documents/`, porté
//! par son fichier complet. L'essai ne connaît que le fonds, qui naît une seule fois (§ 31 · 2.3).

use std::fs;
use std::io;
use std::path::PathBuf;

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{ActorId, Automerge, AutomergeError, Change, ROOT};
use noyau::donnees::fonds::{GENESE, IDENTIFIANT, LIGNEE, MESSAGE_DE_LA_NAISSANCE, PROPRIETAIRE};

use crate::fichier::ecrire_a_cote;
use crate::{Erreur, RacineDesDonnees, disque};

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
    reason = "la racine des données, le dossier et l'état à relire servent aux écritures qui \
              suivent la naissance"
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
}

/// Le fonds de `racine`, rouvert de son fichier complet s'il est né ; sinon `None`.
pub(crate) fn rouvrir_le_fonds(racine: &RacineDesDonnees) -> Result<Option<Document<'_>>, Erreur> {
    let dossier = dossier_du_fonds(racine);
    let complet = dossier.join("complet.automerge");
    let octets = match fs::read(&complet) {
        Ok(octets) => octets,
        Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(erreur) => return Err(disque(&complet, erreur)),
    };
    let mut automerge = Automerge::load(&octets).map_err(Erreur::Automerge)?;
    automerge.set_actor(ActorId::random());
    Ok(Some(Document {
        racine,
        dossier,
        automerge,
        a_relire: false,
    }))
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
