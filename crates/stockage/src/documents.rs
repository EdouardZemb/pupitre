//! Les documents sur fichiers (§ 31 · 2.1) : chacun dans son dossier, sous `documents/`, porté
//! par son fichier complet et par un fichier d'ajout par écriture faite depuis, que la
//! compaction fond dans un fichier complet neuf (§ 31 · 2.2). Ce que la synchronisation reçoit
//! s'écrit de même, en un fichier d'ajout, avant que l'échange ne continue. L'essai ne connaît
//! que le fonds, qui naît une seule fois (§ 31 · 2.3).

use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{ActorId, Automerge, AutomergeError, Change, ChangeHash, ROOT, ReadDoc};
use noyau::donnees::compaction::{self, Lu};
use noyau::donnees::fonds::{GENESE, IDENTIFIANT, LIGNEE, MESSAGE_DE_LA_NAISSANCE, PROPRIETAIRE};
use synchro::{Copie, Echange};

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
    /// Une écriture a échoué : le document refuse d'écrire, d'envoyer, de recevoir et de se
    /// compacter jusqu'à sa réouverture.
    a_relire: bool,
}

impl Document<'_> {
    /// Le document Automerge, tel qu'il est en mémoire.
    pub fn automerge(&self) -> &Automerge {
        &self.automerge
    }

    /// § 31 · 2.5 : la copie que situe la racine du document, l'identifiant de propriétaire,
    /// l'identifiant du document et sa lignée, écrits à la naissance. Un champ absent, ou qui ne
    /// porte pas une chaîne : `ChampAbsent`.
    pub fn copie(&self) -> Result<Copie, Erreur> {
        Ok(Copie {
            proprietaire: self.champ(PROPRIETAIRE)?,
            document: self.champ(IDENTIFIANT)?,
            lignee: self.champ(LIGNEE)?,
        })
    }

    /// Le prochain message de l'échange pour l'autre copie (`Echange::prochain_message`), ou
    /// `None` quand il n'y a plus rien à dire. Un document à relire n'envoie rien : `None` aussi.
    pub fn prochain_message(&self, echange: &mut Echange) -> Option<Vec<u8>> {
        if self.a_relire {
            return None;
        }
        echange.prochain_message(&self.automerge)
    }

    /// § 31 · 2.1 : ce que la synchronisation reçoit ne l'est qu'une fois écrit. Le message
    /// s'applique (`Echange::recevoir`) ; si les heads ont changé, que la réception ait réussi ou
    /// non (ce que l'essai n'éprouve pas), `save_after` des heads d'avant s'écrit en un fichier
    /// d'ajout avant la réponse, puis la compaction suit `apres_une_ecriture`, comme après
    /// [`Document::ecrire`]. Rien de neuf, rien d'écrit.
    ///
    /// L'écriture échoue : le document est à relire, et l'erreur du disque se rend. Une
    /// compaction infidèle se rend comme après `ecrire`, ce qui est reçu écrit. L'une ou l'autre
    /// se rend avant le refus de la réception, `Synchro`. Un document à relire : `ARelire`, et le
    /// message ne s'applique pas.
    pub fn recevoir(&mut self, echange: &mut Echange, message: &[u8]) -> Result<(), Erreur> {
        if self.a_relire {
            return Err(Erreur::ARelire);
        }
        let avant = self.automerge.get_heads();
        let reception = echange.recevoir(&mut self.automerge, message);
        if self.automerge.get_heads() != avant {
            // Une réception refusée en chemin peut avoir appliqué une part du message : ce que
            // porte la mémoire s'écrit.
            self.ecrire_l_ajout(&avant)?;
        }
        reception.map_err(Erreur::Synchro)
    }

    /// § 31 · 2.1 : une modification atomique, avec son message et son heure (`commit_with`),
    /// faite une fois son fichier d'ajout en place : `save_after` des heads d'avant, sous
    /// `ajouts/<empreinte>.automerge`, écrit à côté puis renommé. Une écriture sans modification
    /// n'écrit rien. Le geste refusé, la transaction est annulée (`rollback`), `Erreur::Automerge`
    /// se rend, et le document reste ouvert. L'écriture du fichier échoue : l'erreur se rend, et
    /// le document est à relire.
    ///
    /// Le fichier d'ajout en place, le document se compacte quand `apres_une_ecriture` le dit
    /// (§ 31 · 2.2). Une compaction qui échoue ne défait pas l'écriture, qui est faite : rien
    /// n'est retiré, et la compaction se retente à l'écriture suivante. Une erreur du disque, à
    /// la pesée des fichiers ou à la compaction, se tait. Toute autre erreur de la compaction,
    /// `CompactionInfidele`, se rend, l'écriture faite : le format fait défaut, et la retenter
    /// n'y changerait rien.
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
        self.ecrire_l_ajout(&avant)
    }

    /// `save_after` des heads `avant`, sous `ajouts/<empreinte>.automerge`, écrit à côté puis
    /// renommé ; puis la compaction, quand `apres_une_ecriture` le dit (§ 31 · 2.2). Ce que rend
    /// cette écriture est dit à [`Document::ecrire`], qui l'appelle comme [`Document::recevoir`].
    fn ecrire_l_ajout(&mut self, avant: &[ChangeHash]) -> Result<(), Erreur> {
        let octets = self.automerge.save_after(avant);
        let chemin = self
            .dossier
            .join("ajouts")
            .join(format!("{}.automerge", empreinte(&octets)));
        if let Err(erreur) = ecrire_a_cote(&chemin, &octets) {
            // Ce qui s'écrit, écrit ou reçu, est en mémoire, et son fichier peut être en place :
            // l'erreur peut venir après le renommage, quand il ne devient pas durable. Le
            // document ne sait plus ce que porte le disque.
            self.a_relire = true;
            return Err(disque(&chemin, erreur));
        }
        // L'écriture est faite : ni une compaction qui échoue, ni la pesée des fichiers qui la
        // précède, ne la défont.
        let compaction = match self.a_compacter_apres_une_ecriture() {
            Ok(true) => self.compacter(),
            pesee => pesee.map(|_| ()),
        };
        match compaction {
            // Le disque peut le permettre à l'écriture suivante, qui retente la compaction.
            Err(Erreur::Disque { .. }) => Ok(()),
            // Le reste : `Ok`, ou une compaction infidèle, qui n'a rien retiré mais que rien ne
            // retenterait mieux. Qui écrit doit la voir.
            autre => autre,
        }
    }

    /// § 31 · 2.2 : `save()` dans un fichier complet neuf, écrit à côté puis renommé ; relu par
    /// `Automerge::load` ; puis les fichiers que désigne `a_retirer` sont retirés. Le fichier
    /// complet relu ne mène pas aux mêmes heads : rien n'est retiré, et `CompactionInfidele`.
    /// Un document à relire : `ARelire`, et rien ne s'écrit.
    ///
    /// Les octets de `save()` sont relus avant d'être écrits : s'ils ne se lisent pas, ou ne
    /// mènent pas aux mêmes heads, rien ne s'écrit, et l'ancien fichier complet garde
    /// l'histoire qu'il porte seul (« elle ne retire jamais d'histoire »). Écrit, le fichier
    /// complet se relit du disque : il doit rendre ces mêmes octets.
    pub fn compacter(&mut self) -> Result<(), Erreur> {
        if self.a_relire {
            return Err(Erreur::ARelire);
        }
        let heads = self.automerge.get_heads();
        let octets = self.automerge.save();
        let relu = match Automerge::load(&octets) {
            Ok(relu) if relu.get_heads() == heads => relu,
            // Écrit, ce fichier complet prendrait la place de l'ancien, qui porte seul les
            // compactions d'avant : il n'est pas écrit.
            _ => return Err(Erreur::CompactionInfidele),
        };
        let complet = self.dossier.join("complet.automerge");
        ecrire_a_cote(&complet, &octets).map_err(|erreur| disque(&complet, erreur))?;
        // Relu du disque, il rend les octets relus plus haut, et mène donc aux mêmes heads ; le
        // disque en rend d'autres, et rien n'est retiré.
        let meme_point = fs::read(&complet).map_err(|erreur| disque(&complet, erreur))? == octets;

        let ajouts = self.dossier.join("ajouts");
        let mut lus = Vec::new();
        for chemin in fichiers_d_ajout(&ajouts)? {
            let octets = fs::read(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
            lus.push(Lu {
                nom: chemin
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                conforme: est_conforme(&chemin, &octets),
                absorbe: est_absorbe(&relu, &octets),
            });
        }
        for nom in compaction::a_retirer(meme_point, &lus) {
            let chemin = ajouts.join(nom);
            fs::remove_file(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        }
        if !meme_point {
            return Err(Erreur::CompactionInfidele);
        }
        Ok(())
    }

    /// La chaîne que porte `champ`, à la racine du document ; sinon `ChampAbsent`.
    fn champ(&self, champ: &'static str) -> Result<String, Erreur> {
        let (valeur, _) = self
            .automerge
            .get(ROOT, champ)
            .map_err(Erreur::Automerge)?
            .ok_or(Erreur::ChampAbsent { champ })?;
        valeur
            .into_string()
            .map_err(|_| Erreur::ChampAbsent { champ })
    }

    /// Ce que dit `apres_une_ecriture` des fichiers sur le disque : le poids du fichier complet,
    /// le poids et le nombre des fichiers d'ajout.
    fn a_compacter_apres_une_ecriture(&self) -> Result<bool, Erreur> {
        let octets_du_complet = poids(&self.dossier.join("complet.automerge"))?;
        let ajouts = fichiers_d_ajout(&self.dossier.join("ajouts"))?;
        let mut octets_des_ajouts = 0;
        for chemin in &ajouts {
            octets_des_ajouts += poids(chemin)?;
        }
        Ok(compaction::apres_une_ecriture(
            octets_du_complet,
            octets_des_ajouts,
            ajouts.len(),
        ))
    }
}

/// Le fonds de `racine`, rouvert de son fichier complet puis de ses fichiers d'ajout s'il est
/// né, puis compacté quand `a_l_ouverture` le dit ; sinon `None`. Une compaction qui échoue
/// fait échouer l'ouverture. L'essai ne lit pas le voisin du fichier complet : le lot 1 le
/// fond.
pub(crate) fn rouvrir_le_fonds(racine: &RacineDesDonnees) -> Result<Option<Document<'_>>, Erreur> {
    let dossier = dossier_du_fonds(racine);
    let complet = dossier.join("complet.automerge");
    let octets = match fs::read(&complet) {
        Ok(octets) => octets,
        Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(erreur) => return Err(disque(&complet, erreur)),
    };
    let mut automerge = Automerge::load(&octets).map_err(Erreur::Automerge)?;
    let ajouts = lire_les_ajouts(&mut automerge, &dossier.join("ajouts"))?;
    automerge.set_actor(ActorId::random());
    let mut document = Document {
        racine,
        dossier,
        automerge,
        a_relire: false,
    };
    if compaction::a_l_ouverture(ajouts, false) {
        document.compacter()?;
    }
    Ok(Some(document))
}

/// Lit chaque fichier d'ajout de `ajouts`, dans l'ordre que donne le dossier : son contenu doit
/// donner l'empreinte de son nom, sinon [`Erreur::AjoutAltere`], puis il s'ajoute au document
/// (`load_incremental`). Tout lu, aucune modification n'attend ses dépendances, sinon
/// [`Erreur::DependancesManquantes`]. Rien n'est retiré ni réécrit. Rend le nombre de fichiers
/// d'ajout lus.
fn lire_les_ajouts(automerge: &mut Automerge, ajouts: &Path) -> Result<usize, Erreur> {
    let fichiers = fichiers_d_ajout(ajouts)?;
    for chemin in &fichiers {
        let octets = fs::read(chemin).map_err(|erreur| disque(chemin, erreur))?;
        // `load_incremental` s'arrête sans erreur au premier tronçon altéré : seule l'empreinte
        // du nom dit qu'un fichier d'ajout a changé.
        if !est_conforme(chemin, &octets) {
            return Err(Erreur::AjoutAltere {
                chemin: chemin.clone(),
            });
        }
        automerge
            .load_incremental(&octets)
            .map_err(Erreur::Automerge)?;
    }
    if !automerge.get_missing_deps(&[]).is_empty() {
        return Err(Erreur::DependancesManquantes);
    }
    Ok(fichiers.len())
}

/// Les fichiers d'ajout de `ajouts`, `<empreinte>.automerge`, dans l'ordre que donne le
/// dossier. Un autre nom, un `.partiel` par exemple, n'en est pas un : il est laissé tel quel.
fn fichiers_d_ajout(ajouts: &Path) -> Result<Vec<PathBuf>, Erreur> {
    let mut fichiers = Vec::new();
    for entree in fs::read_dir(ajouts).map_err(|erreur| disque(ajouts, erreur))? {
        let chemin = entree.map_err(|erreur| disque(ajouts, erreur))?.path();
        if chemin.extension() == Some(OsStr::new("automerge")) {
            fichiers.push(chemin);
        }
    }
    Ok(fichiers)
}

/// Le contenu `octets` du fichier d'ajout `chemin` donne l'empreinte de son nom.
fn est_conforme(chemin: &Path, octets: &[u8]) -> bool {
    chemin.file_stem() == Some(OsStr::new(&empreinte(octets)))
}

/// Le fichier d'ajout de contenu `octets` est absorbé par le fichier complet relu `relu` :
/// chargé (`load_incremental`) dans un `fork()` de `relu`, il en laisse les heads inchangées,
/// et aucune modification n'y attend ses dépendances.
fn est_absorbe(relu: &Automerge, octets: &[u8]) -> bool {
    let mut avec_l_ajout = relu.fork();
    avec_l_ajout.load_incremental(octets).is_ok()
        && avec_l_ajout.get_heads() == relu.get_heads()
        && avec_l_ajout.get_missing_deps(&[]).is_empty()
}

/// Le poids de `chemin` sur le disque, en octets.
fn poids(chemin: &Path) -> Result<u64, Erreur> {
    Ok(fs::metadata(chemin)
        .map_err(|erreur| disque(chemin, erreur))?
        .len())
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
