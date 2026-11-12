//! Les documents sur fichiers (§ 31 · 2.1) : chacun dans son dossier, sous `documents/`, porté
//! par son fichier complet et par un fichier d'ajout par écriture faite depuis, que la
//! compaction fond dans un fichier complet neuf (§ 31 · 2.2). Ce que la synchronisation reçoit
//! s'écrit de même, en un fichier d'ajout, avant que l'échange ne continue. L'essai ne connaît
//! que le fonds, qui naît une seule fois (§ 31 · 2.3).
//!
//! Une écriture est faite ; faite, avec un avertissement, quand son renommage n'est pas rendu
//! durable ou que la compaction qui la suit échoue ; ou refusée, avant la fin de son renommage,
//! et le document se relit alors du disque. Une relecture du disque qui échoue met le document
//! en lecture seule, comme un fichier illisible à l'ouverture ; une compaction d'ouverture qui
//! échoue vient en avertissement, et le document s'ouvre.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{
    ActorId, Automerge, AutomergeError, Change, ChangeHash, LoadOptions, OnPartialLoad, ROOT,
    ReadDoc,
};
use noyau::donnees::acces::Acces;
use noyau::donnees::compaction::{self, Lu};
use noyau::donnees::fonds::{GENESE, IDENTIFIANT, LIGNEE, MESSAGE_DE_LA_NAISSANCE, PROPRIETAIRE};
use synchro::{Copie, Echange};

use crate::fichier::{self, Echec, ecrire_a_cote};
use crate::pannes::{self, Etape};
use crate::{Avertissement, Erreur, LectureSeule, RacineDesDonnees, disque, empreinte};

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
pub struct Document<'r> {
    racine: &'r RacineDesDonnees,
    /// Le dossier du document : `documents/fonds/`.
    dossier: PathBuf,
    automerge: Automerge,
    /// Pourquoi le document est en lecture seule ; `None` quand il est entier.
    lecture_seule: Option<LectureSeule>,
    /// Une compaction a échoué, ou un renommage n'a pas été rendu durable : l'écriture suivante
    /// compacte le document. Une compaction qui réussit la baisse.
    compaction_due: bool,
    /// Les avertissements de la naissance ou de l'ouverture, et ceux que des réceptions refusées
    /// en chemin n'ont pu rendre.
    avertissements: Vec<Avertissement>,
}

impl Drop for Document<'_> {
    /// Un `Document` du fonds vit tant que la garde de la racine des données est prise : il la
    /// rend à sa fin.
    fn drop(&mut self) {
        self.racine.rendre_la_garde();
    }
}

impl Document<'_> {
    /// Le document Automerge, tel qu'il est en mémoire.
    pub fn automerge(&self) -> &Automerge {
        &self.automerge
    }

    /// Pourquoi le document est en lecture seule (§ 31 · 2.1) ; `None` quand il est entier.
    pub fn lecture_seule(&self) -> Option<&LectureSeule> {
        self.lecture_seule.as_ref()
    }

    /// Les avertissements de la naissance ou de l'ouverture, et celui qu'une réception refusée en
    /// chemin n'a pu rendre, son refus se rendant à sa place.
    pub fn avertissements(&self) -> &[Avertissement] {
        &self.avertissements
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
    /// `None` quand il n'y a plus rien à dire. Un document en lecture seule que son accès prive
    /// d'échange n'envoie rien : `None` aussi.
    pub fn prochain_message(&self, echange: &mut Echange) -> Option<Vec<u8>> {
        if !self.acces().echange() {
            return None;
        }
        echange.prochain_message(&self.automerge)
    }

    /// § 31 · 2.1 : ce que la synchronisation reçoit ne l'est qu'une fois écrit. Le message
    /// s'applique (`Echange::recevoir`) ; si les heads ont changé, que la réception ait réussi ou
    /// non, `save_after` des heads d'avant s'écrit en un fichier d'ajout avant la réponse, avec
    /// les issues de [`Document::ecrire`] : faite, faite avec un avertissement, ou refusée. Rien
    /// de neuf, rien d'écrit.
    ///
    /// Une réception dont l'écriture est refusée relit le document du disque, qui perd ce que
    /// l'échange croit reçu : l'échange reprend (`Echange::reprendre`), pour que l'autre copie
    /// renvoie ce qu'elle a envoyé, et `Erreur::Disque` se rend. Une réception refusée en chemin,
    /// que l'échange refuse, rend `Erreur::Synchro`, ce qu'elle a apporté écrit ; l'avertissement
    /// de cette écriture, s'il en vient un, rejoint [`Document::avertissements`]. Un document en
    /// lecture seule que son accès prive d'échange : `Erreur::LectureSeule`, et le message ne
    /// s'applique pas.
    pub fn recevoir(
        &mut self,
        echange: &mut Echange,
        message: &[u8],
    ) -> Result<Option<Avertissement>, Erreur> {
        self.exiger(Acces::echange)?;
        let avant = self.automerge.get_heads();
        let mut reception = echange.recevoir(&mut self.automerge, message);
        if self.automerge.get_heads() == avant {
            return reception.map(|()| None).map_err(Erreur::Synchro);
        }
        if pannes::point(Etape::Reception).is_err() {
            // Seule une panne injectée rend ce cas : la réception a changé le document, ce que ne
            // fait pas un message illisible.
            reception = reception.and(Err(synchro::Erreur::Automerge(AutomergeError::Fail)));
        }
        // Une réception refusée en chemin peut avoir appliqué une part du message : ce que porte
        // la mémoire s'écrit.
        let ecriture = match self.ecrire_l_ajout(&avant) {
            Ok(avertissement) => avertissement,
            Err(refus) => {
                echange.reprendre();
                return Err(refus);
            }
        };
        match reception {
            Ok(()) => Ok(ecriture),
            Err(refus) => {
                // Le refus de la réception se rend : l'avertissement ne se perd pas.
                self.avertissements.extend(ecriture);
                Err(Erreur::Synchro(refus))
            }
        }
    }

    /// § 31 · 2.1 : une modification atomique, avec son message et son heure (`commit_with`),
    /// faite une fois son fichier d'ajout en place : `save_after` des heads d'avant, sous
    /// `ajouts/<empreinte>.automerge`, écrit à côté puis renommé. Une écriture sans modification
    /// n'écrit rien. Le geste refusé, la transaction est annulée (`rollback`), `Erreur::Automerge`
    /// se rend, et le document reste ouvert. Un document en lecture seule n'écrit pas :
    /// `Erreur::LectureSeule`, et le geste n'est pas fait.
    ///
    /// La modification faite en mémoire, l'écriture a trois issues :
    ///
    /// - faite : `Ok(None)` ;
    /// - faite, avec un avertissement : le fichier d'ajout est renommé, mais son renommage n'est
    ///   pas rendu durable (`Avertissement::NonDurable`), et la compaction est due ; ou la
    ///   compaction qui suit, quand `apres_une_ecriture` la dit (§ 31 · 2.2), échoue, la pesée
    ///   des fichiers comprise (`Avertissement::Compaction`) : rien n'est perdu, et elle se
    ///   retente à l'écriture suivante, qui retire ce qu'elle n'a pas retiré ;
    /// - refusée, avant la fin du renommage : le voisin est ôté, vidé puis retiré
    ///   ([`fichier::ecrire_a_cote`]), le document se relit du disque, avec un acteur neuf, et
    ///   `Erreur::Disque` se rend. Une relecture qui échoue met le document en lecture seule
    ///   (`LectureSeule::RelectureManquee`), sur sa mémoire d'avant l'écriture refusée.
    ///
    /// Aucun message ne se prépare pendant une écriture : `ecrire` tient le document emprunté en
    /// écriture jusqu'à son retour, et le geste ne peut pas l'emprunter pour l'échange. Le premier
    /// exemple ne compile pas ; le second, qui prépare le message après l'écriture, compile.
    ///
    // § 31 · 10, test 8 : une synchronisation lancée pendant l'écriture ne voit pas l'opération en cours
    /// ```compile_fail,E0502
    /// fn pendant(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange) {
    ///     let _ = document.ecrire("Idée notée", 0, |_tx| {
    ///         let _ = document.prochain_message(echange);
    ///         Ok(())
    ///     });
    /// }
    /// ```
    ///
    /// ```
    /// fn apres(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange) {
    ///     let _ = document.ecrire("Idée notée", 0, |_tx| Ok(()));
    ///     let _ = document.prochain_message(echange);
    /// }
    /// ```
    pub fn ecrire(
        &mut self,
        message: &str,
        heure: i64,
        geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
    ) -> Result<Option<Avertissement>, Erreur> {
        self.exiger(Acces::ecrit)?;
        let avant = self.automerge.get_heads();
        let mut tx = self.automerge.transaction();
        if let Err(erreur) = geste(&mut tx) {
            tx.rollback();
            return Err(Erreur::Automerge(erreur));
        }
        let (tete, _) = tx.commit_with(a_l_heure(message, heure));
        if tete.is_none() {
            return Ok(None);
        }
        self.ecrire_l_ajout(&avant)
    }

    /// § 31 · 2.2 : les octets de `save()`, relus en mémoire par `Automerge::load`, puis écrits
    /// dans un fichier complet neuf par `ecrire_a_cote`, qui relit son voisin du disque avant de
    /// le renommer ; puis chaque fichier d'ajout se lit par `fichier::lire`, et ceux que désigne
    /// `a_retirer` sont retirés par `fichier::retirer`. Un document en lecture seule ne se
    /// compacte pas : `Erreur::LectureSeule`, et rien ne s'écrit.
    ///
    /// Les octets de `save()` qui ne se lisent pas, ou ne mènent pas aux mêmes heads, ne
    /// s'écrivent pas : `CompactionInfidele`, et l'ancien fichier complet garde l'histoire qu'il
    /// porte seul (« elle ne retire jamais d'histoire »). Un voisin relu du disque qui ne rend
    /// pas ces mêmes octets ne prend pas la place : `Erreur::Disque` se rend
    /// (`io::ErrorKind::InvalidData`), l'ancien fichier complet reste en place, et rien n'est
    /// retiré. Un fichier d'ajout est lisible quand son contenu donne l'empreinte de son nom et
    /// que `load_incremental` le charge, dans le fichier complet relu ; absorbé, quand il n'y
    /// ajoute rien. Une compaction qui échoue est due : l'écriture suivante la retente ; une
    /// compaction qui réussit ne l'est plus.
    pub fn compacter(&mut self) -> Result<(), Erreur> {
        self.exiger(Acces::ecrit)?;
        let compaction = self.fondre_les_ajouts();
        self.compaction_due = compaction.is_err();
        compaction
    }

    /// La compaction de [`Document::compacter`], une fois son accès vérifié.
    fn fondre_les_ajouts(&self) -> Result<(), Erreur> {
        let heads = self.automerge.get_heads();
        let octets = self.automerge.save();
        // Une panne injectée après `save()` vaut des octets qui ne se relisent pas.
        if pannes::point(Etape::Save).is_err() {
            return Err(Erreur::CompactionInfidele);
        }
        let relu = match Automerge::load(&octets) {
            Ok(relu) if relu.get_heads() == heads => relu,
            // Écrit, ce fichier complet prendrait la place de l'ancien, qui porte seul les
            // compactions d'avant : il n'est pas écrit.
            _ => return Err(Erreur::CompactionInfidele),
        };
        let complet = self.dossier.join("complet.automerge");
        // Avant son renommage, le voisin a rendu, relu du disque, les octets relus plus haut :
        // le fichier complet en place mène aux mêmes heads.
        ecrire_a_cote(&complet, &octets).map_err(|echec| disque_d_un_echec(&complet, echec))?;

        let ajouts = self.dossier.join("ajouts");
        let mut lus = Vec::new();
        for chemin in fichiers_d_ajout(&ajouts)? {
            let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
            lus.push(lu_contre(&relu, &chemin, &octets));
        }
        for nom in compaction::a_retirer(&lus) {
            let chemin = ajouts.join(nom);
            fichier::retirer(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        }
        Ok(())
    }

    /// `save_after` des heads `avant`, sous `ajouts/<empreinte>.automerge`, écrit à côté puis
    /// renommé ; puis la compaction, quand `apres_une_ecriture` le dit (§ 31 · 2.2). Ses trois
    /// issues sont dites à [`Document::ecrire`], qui l'appelle comme [`Document::recevoir`].
    fn ecrire_l_ajout(&mut self, avant: &[ChangeHash]) -> Result<Option<Avertissement>, Erreur> {
        let octets = self.automerge.save_after(avant);
        let chemin = self
            .dossier
            .join("ajouts")
            .join(format!("{}.automerge", empreinte(&octets)));
        match ecrire_a_cote(&chemin, &octets) {
            // Le voisin est ôté : la mémoire porte ce que le disque n'a pas, et se relit.
            Err(Echec::Refusee(erreur)) => {
                self.relire_du_disque(avant);
                Err(disque(&chemin, erreur))
            }
            // Le fichier est en place : l'écriture est faite. La compaction, due, réécrira à
            // l'écriture suivante un fichier complet qui porte ce fichier d'ajout.
            Err(Echec::NonDurable(erreur)) => {
                self.compaction_due = true;
                Ok(Some(Avertissement::NonDurable { chemin, erreur }))
            }
            // L'écriture est faite : ni une compaction qui échoue, ni la pesée des fichiers qui la
            // précède, ne la défont.
            Ok(()) => Ok(self
                .compacter_apres_une_ecriture()
                .err()
                .map(Avertissement::Compaction)),
        }
    }

    /// La compaction qui suit une écriture faite, quand `apres_une_ecriture` la dit. Une pesée
    /// des fichiers qui échoue la rend due, comme une compaction qui échoue.
    fn compacter_apres_une_ecriture(&mut self) -> Result<(), Erreur> {
        match self.faut_il_compacter() {
            Ok(true) => self.compacter(),
            Ok(false) => Ok(()),
            Err(erreur) => {
                self.compaction_due = true;
                Err(erreur)
            }
        }
    }

    /// Ce que dit `apres_une_ecriture` de l'accès du document, de sa compaction due et des
    /// fichiers sur le disque : le poids du fichier complet, le poids et le nombre des fichiers
    /// d'ajout.
    fn faut_il_compacter(&self) -> Result<bool, Erreur> {
        let octets_du_complet = poids(&self.dossier.join("complet.automerge"))?;
        let ajouts = fichiers_d_ajout(&self.dossier.join("ajouts"))?;
        let mut octets_des_ajouts = 0;
        for chemin in &ajouts {
            octets_des_ajouts += poids(chemin)?;
        }
        Ok(compaction::apres_une_ecriture(
            self.acces(),
            self.compaction_due,
            octets_du_complet,
            octets_des_ajouts,
            ajouts.len(),
        ))
    }

    /// § 31 · 2.1 : après une écriture refusée, la mémoire se relit du disque
    /// (`lire_le_dossier`), sans toucher à la garde, et tire un acteur neuf ; une raison de
    /// lecture seule que la relecture trouve devient celle du document, sur ce qui se lit. Une
    /// relecture qui échoue laisse la mémoire d'avant l'écriture refusée (`fork_at` des heads
    /// `avant`), en lecture seule `RelectureManquee`, sur le fichier qu'elle n'a pu lire, ou sur
    /// le dossier du document si son erreur n'en nomme aucun.
    fn relire_du_disque(&mut self, avant: &[ChangeHash]) {
        match lire_le_dossier(&self.dossier) {
            Ok((mut automerge, lecture_seule, _)) => {
                automerge.set_actor(ActorId::random());
                self.automerge = automerge;
                self.lecture_seule = lecture_seule;
            }
            Err(erreur) => {
                let chemin = fichier_nomme(&erreur)
                    .unwrap_or(&self.dossier)
                    .to_path_buf();
                // Les heads d'avant sont celles de cette mémoire, qui ne fait que croître.
                self.automerge = self
                    .automerge
                    .fork_at(avant)
                    .expect("les heads d'avant l'écriture sont dans la mémoire");
                self.lecture_seule = Some(LectureSeule::RelectureManquee { chemin });
            }
        }
    }

    /// Ce que permet le document : tout, s'il est entier ; sinon, l'accès de sa raison de lecture
    /// seule.
    fn acces(&self) -> Acces {
        self.lecture_seule
            .as_ref()
            .map_or(Acces::Entier, LectureSeule::acces)
    }

    /// `Erreur::LectureSeule`, avec sa raison, quand l'accès du document ne permet pas `permis`.
    fn exiger(&self, permis: impl FnOnce(Acces) -> bool) -> Result<(), Erreur> {
        match &self.lecture_seule {
            Some(raison) if !permis(self.acces()) => Err(Erreur::LectureSeule(raison.clone())),
            _ => Ok(()),
        }
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
}

/// Le fonds de `racine`, rouvert s'il est né (`lire_le_dossier`), avec un acteur neuf, en
/// lecture seule si un fichier ne se lit pas, puis compacté quand `a_l_ouverture` le dit ;
/// sinon `None`. Une compaction qui échoue n'empêche pas d'ouvrir : son erreur vient en
/// avertissement (`Avertissement::Compaction`), rien n'est perdu, et la compaction est due :
/// l'écriture suivante retire ce qu'elle n'a pas retiré. L'essai ne lit pas le voisin du fichier
/// complet : le lot 1 le fond.
pub(crate) fn rouvrir_le_fonds(racine: &RacineDesDonnees) -> Result<Option<Document<'_>>, Erreur> {
    let garde = racine.prendre_la_garde()?;
    let dossier = dossier_du_fonds(racine);
    let complet = dossier.join("complet.automerge");
    if !complet
        .try_exists()
        .map_err(|erreur| disque(&complet, erreur))?
    {
        return Ok(None);
    }
    let (mut automerge, lecture_seule, ajouts) = lire_le_dossier(&dossier)?;
    automerge.set_actor(ActorId::random());
    // Le document né rend la garde à sa fin, même si la compaction qui suit échoue.
    garde.passer_au_document();
    let mut document = Document {
        racine,
        dossier,
        automerge,
        lecture_seule,
        compaction_due: false,
        avertissements: Vec::new(),
    };
    if compaction::a_l_ouverture(document.acces(), ajouts, false)
        && let Err(erreur) = document.compacter()
    {
        // `compacter` a rendu la compaction due : l'écriture suivante la retente.
        document
            .avertissements
            .push(Avertissement::Compaction(erreur));
    }
    Ok(Some(document))
}

/// Lit le dossier `dossier` d'un document, sans compaction ni démarrage : son fichier complet
/// (`lire_le_complet`), puis ses fichiers d'ajout, dans l'ordre des noms (`lire_les_ajouts`),
/// chacun par `fichier::lire` ; tout lu, elle cherche les dépendances manquantes
/// (`dependances_manquantes`). Elle ne construit aucun `Document` et ne touche pas à la garde :
/// l'ouverture et la relecture du disque l'appellent. Rend la mémoire lue, ce qui se lit, la
/// première raison de lecture seule qu'elle trouve, dans cet ordre : le fichier complet, puis
/// les fichiers d'ajout, puis les dépendances ; et le nombre de fichiers d'ajout. Un refus du
/// disque à la lecture rend `Erreur::Disque`.
fn lire_le_dossier(dossier: &Path) -> Result<(Automerge, Option<LectureSeule>, usize), Erreur> {
    let complet = dossier.join("complet.automerge");
    let (mut automerge, du_complet) = lire_le_complet(&complet)?;
    let fichiers = fichiers_d_ajout(&dossier.join("ajouts"))?;
    let ajouts = lire_les_ajouts(&mut automerge, &fichiers)?;
    let raison = du_complet
        .or(ajouts.raison)
        .or_else(|| dependances_manquantes(&automerge, &ajouts.charges, &complet));
    Ok((automerge, raison, fichiers.len()))
}

/// Le fichier complet `complet`, lu par `fichier::lire`, puis `Automerge::load`. Vide, ou
/// refusé, il se relit sans ce qui ne se lit pas (`load_with_options`, `OnPartialLoad::Ignore`),
/// ou, refusé encore, laisse un document neuf ; la raison est alors `CompletIllisible`. Un
/// fichier complet vide, qu'Automerge lit comme un document neuf, sans genèse, n'est jamais
/// entier : les écritures naîtraient sans elle.
fn lire_le_complet(complet: &Path) -> Result<(Automerge, Option<LectureSeule>), Erreur> {
    let octets = fichier::lire(complet).map_err(|erreur| disque(complet, erreur))?;
    if !octets.is_empty()
        && let Ok(automerge) = Automerge::load(&octets)
    {
        return Ok((automerge, None));
    }
    let ce_qui_se_lit = Automerge::load_with_options(
        &octets,
        LoadOptions::new().on_partial_load(OnPartialLoad::Ignore),
    )
    .unwrap_or_else(|_| Automerge::new());
    let raison = LectureSeule::CompletIllisible {
        chemin: complet.to_path_buf(),
    };
    Ok((ce_qui_se_lit, Some(raison)))
}

/// Ce que `lire_les_ajouts` rend : les fichiers d'ajout chargés, avec leur contenu, dans l'ordre
/// des noms, et la raison de lecture seule du premier qui ne l'est pas.
struct AjoutsLus<'f> {
    charges: Vec<(&'f Path, Vec<u8>)>,
    raison: Option<LectureSeule>,
}

/// Lit les fichiers d'ajout `fichiers`, triés par nom, chacun par `fichier::lire`, et charge
/// (`load_incremental`) ceux qui se lisent. Un fichier dont le contenu ne donne pas l'empreinte
/// de son nom n'est pas chargé, et donne `AjoutAltere` ; un fichier qui la donne, mais que
/// `load_incremental` refuse, ne l'est pas non plus, et donne `AjoutIllisible`. Le premier, dans
/// l'ordre des noms, l'emporte. Rien n'est retiré ni réécrit.
fn lire_les_ajouts<'f>(
    automerge: &mut Automerge,
    fichiers: &'f [PathBuf],
) -> Result<AjoutsLus<'f>, Erreur> {
    let mut lus = AjoutsLus {
        charges: Vec::new(),
        raison: None,
    };
    for chemin in fichiers {
        let octets = fichier::lire(chemin).map_err(|erreur| disque(chemin, erreur))?;
        // `load_incremental` s'arrête sans erreur au premier tronçon altéré : seule l'empreinte
        // du nom dit qu'un fichier d'ajout a changé.
        let refus = if !est_conforme(chemin, &octets) {
            Some(LectureSeule::AjoutAltere {
                chemin: chemin.clone(),
            })
        } else if automerge.load_incremental(&octets).is_err() {
            Some(LectureSeule::AjoutIllisible {
                chemin: chemin.clone(),
            })
        } else {
            lus.charges.push((chemin.as_path(), octets));
            None
        };
        lus.raison = lus.raison.or(refus);
    }
    Ok(lus)
}

/// Tout lu, il manque au document lu `automerge` des modifications qu'une autre attend
/// (`get_missing_deps(&[])`, qui suit les modifications en attente jusqu'à celles que le
/// document n'a pas) : `DependancesManquantes`, sur le premier fichier chargé de `charges`, dans
/// l'ordre des noms, qui attend l'une d'elles (`attend_l_une_de`) ; à défaut, sur le fichier
/// complet `complet`, dont une modification attend alors. Le fichier qui manque ne se nomme pas,
/// celui qui l'attend, si ; un fichier qui n'attend qu'un fichier présent, lui-même en attente,
/// ne se nomme pas non plus. Rien ne manque : `None`.
fn dependances_manquantes(
    automerge: &Automerge,
    charges: &[(&Path, Vec<u8>)],
    complet: &Path,
) -> Option<LectureSeule> {
    let manquantes = automerge.get_missing_deps(&[]);
    if manquantes.is_empty() {
        return None;
    }
    let chemin = charges
        .iter()
        .find(|(_, octets)| attend_l_une_de(&manquantes, octets))
        .map_or(complet, |(chemin, _)| *chemin);
    Some(LectureSeule::DependancesManquantes {
        chemin: chemin.to_path_buf(),
    })
}

/// Le fichier d'ajout de contenu `octets`, seul dans un document neuf (`load_incremental`),
/// attend des modifications (`get_missing_deps(&[])`) dont l'une est dans `manquantes`, celles
/// qui manquent au document lu.
fn attend_l_une_de(manquantes: &[ChangeHash], octets: &[u8]) -> bool {
    let mut seul = Automerge::new();
    seul.load_incremental(octets).is_ok()
        && seul
            .get_missing_deps(&[])
            .iter()
            .any(|attendue| manquantes.contains(attendue))
}

/// Le fichier que nomme `erreur`, s'il en nomme un.
fn fichier_nomme(erreur: &Erreur) -> Option<&Path> {
    match erreur {
        Erreur::Disque { chemin, .. } => Some(chemin.as_path()),
        _ => None,
    }
}

/// Les fichiers d'ajout de `ajouts`, `<empreinte>.automerge`, triés par nom. Un autre nom, un
/// `.partiel` par exemple, n'en est pas un : il est laissé tel quel.
fn fichiers_d_ajout(ajouts: &Path) -> Result<Vec<PathBuf>, Erreur> {
    let mut fichiers = Vec::new();
    for entree in fs::read_dir(ajouts).map_err(|erreur| disque(ajouts, erreur))? {
        let chemin = entree.map_err(|erreur| disque(ajouts, erreur))?.path();
        if chemin.extension() == Some(OsStr::new("automerge")) {
            fichiers.push(chemin);
        }
    }
    fichiers.sort();
    Ok(fichiers)
}

/// Le contenu `octets` du fichier d'ajout `chemin` donne l'empreinte de son nom.
fn est_conforme(chemin: &Path, octets: &[u8]) -> bool {
    chemin.file_stem() == Some(OsStr::new(&empreinte(octets)))
}

/// Ce que la compaction lit du fichier d'ajout `chemin`, de contenu `octets`, contre le fichier
/// complet relu `relu`, dans un `fork()` de `relu` où `load_incremental` le charge : lisible,
/// son contenu donne l'empreinte de son nom (`est_conforme`), et il s'est chargé sans erreur ;
/// absorbé, chargé, il laisse les heads de `relu` inchangées, et aucune modification n'y attend
/// ses dépendances.
fn lu_contre(relu: &Automerge, chemin: &Path, octets: &[u8]) -> Lu {
    let mut avec_l_ajout = relu.fork();
    let charge = avec_l_ajout.load_incremental(octets).is_ok();
    Lu {
        nom: chemin
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        lisible: charge && est_conforme(chemin, octets),
        absorbe: charge
            && avec_l_ajout.get_heads() == relu.get_heads()
            && avec_l_ajout.get_missing_deps(&[]).is_empty(),
    }
}

/// Le poids de `chemin` sur le disque, en octets.
fn poids(chemin: &Path) -> Result<u64, Erreur> {
    Ok(fs::metadata(chemin)
        .map_err(|erreur| disque(chemin, erreur))?
        .len())
}

/// Fait naître le fonds de `racine` : la genèse, la naissance et le premier geste, en mémoire,
/// puis un seul fichier complet, écrit à côté puis renommé, une fois `ajouts/` créé. Refusé
/// avant la fin de son renommage, le fichier complet ne laisse que `ajouts/`, vide, et
/// `Erreur::Disque` se rend ; en place, mais son renommage non rendu durable, le document naît
/// avec cet avertissement (`Avertissement::NonDurable`), et sa compaction est due.
pub(crate) fn faire_naitre_le_fonds<'r>(
    racine: &'r RacineDesDonnees,
    naissance: &Naissance<'_>,
    message: &str,
    heure: i64,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Document<'r>, Erreur> {
    let garde = racine.prendre_la_garde()?;
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
    fichier::rendre_durable(&dossier)
        .map_err(|erreur| disque(&racine.chemin().join("documents"), erreur))?;
    let mut avertissements = Vec::new();
    match ecrire_a_cote(&complet, &automerge.save()) {
        Ok(()) => {}
        Err(Echec::Refusee(erreur)) => return Err(disque(&complet, erreur)),
        // Le fichier complet est en place : le fonds est né.
        Err(Echec::NonDurable(erreur)) => avertissements.push(Avertissement::NonDurable {
            chemin: complet,
            erreur,
        }),
    }
    garde.passer_au_document();
    Ok(Document {
        racine,
        dossier,
        automerge,
        lecture_seule: None,
        // La compaction réécrira un fichier complet, et rendra la naissance durable.
        compaction_due: !avertissements.is_empty(),
        avertissements,
    })
}

/// Le refus du disque sur `chemin` d'une écriture à côté, refusée ou non durable :
/// [`Erreur::Disque`]. La compaction le rend ainsi : tant que son fichier complet n'est pas
/// durable, elle ne retire aucun fichier d'ajout.
fn disque_d_un_echec(chemin: &Path, echec: Echec) -> Erreur {
    match echec {
        Echec::Refusee(erreur) | Echec::NonDurable(erreur) => disque(chemin, erreur),
    }
}

/// `documents/fonds/`, dans la racine des données.
fn dossier_du_fonds(racine: &RacineDesDonnees) -> PathBuf {
    racine.chemin().join("documents").join("fonds")
}

/// La genèse, puis la naissance, qui écrit l'identifiant, la lignée et le propriétaire, puis le
/// premier geste : trois modifications, dans un document Automerge neuf, dont l'acteur est
/// neuf. Le geste refusé, le document est perdu avec lui. La genèse est une constante du noyau,
/// que `la_genese_se_reconstruit_a_l_octet` éprouve : elle se lit, et s'applique à un document
/// neuf.
fn naitre_en_memoire(
    naissance: &Naissance<'_>,
    message: &str,
    heure: i64,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Automerge, Erreur> {
    let genese = Change::try_from(GENESE).expect("la genèse du noyau se lit");
    let mut automerge = Automerge::new();
    automerge
        .apply_changes([genese])
        .expect("la genèse du noyau s'applique à un document neuf");

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
