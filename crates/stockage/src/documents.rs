//! Les documents sur fichiers (§ 31 · 2.1) : chacun dans son dossier, sous `documents/`, porté
//! par son fichier complet et par un fichier d'ajout par écriture faite depuis, que la
//! compaction fond dans un fichier complet neuf (§ 31 · 2.2). Ce que la synchronisation reçoit
//! s'écrit de même, en un fichier d'ajout, avant que l'échange ne continue. L'essai ne connaît
//! que le fonds, qui naît une seule fois (§ 31 · 2.3).
//!
//! À chaque ouverture, la garde prise, le démarrage donne son sort à chaque `.partiel` de
//! `documents/` (§ 31 · 2.1) : le voisin d'un fichier d'ajout dont le contenu donne l'empreinte
//! prend sa place, ou se lit où il est ; le voisin lisible du fichier complet se lit, puis la
//! compaction le fond ; tout autre se retire. Lu entier, le fonds lit sa version de schéma
//! (§ 31 · 2.4) : plus ancienne que celle que connaît l'application, il migre, et ses migrations
//! s'écrivent ensemble en un seul fichier d'ajout, ou, si elles ne s'appliquent pas, il s'ouvre en
//! lecture seule ; plus récente, il se lit sans écrire, et reçoit, sans transmettre aucune
//! modification ; absente ou illisible, il s'ouvre en lecture seule. Après sa compaction
//! d'ouverture, chaque nom provisoire de `captations/` dont le contenu donne une empreinte que le
//! fonds nomme prend ce nom (§ 31 · 6.1).
//!
//! Les opérations, la genèse, la naissance et les migrations s'appliquent par
//! `services::application`, le code que partagent les deux adaptateurs `Documents`.
//!
//! Une écriture est faite ; faite, avec un avertissement, quand son renommage n'est pas rendu
//! durable ou que la compaction qui la suit échoue ; ou refusée, avant la fin de son renommage,
//! et le document se relit alors du disque. Une relecture du disque qui échoue met le document
//! en lecture seule, comme un fichier illisible à l'ouverture ; une compaction d'ouverture qui
//! échoue vient en avertissement, et le document s'ouvre.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "pannes")]
use automerge::transaction::{CommitOptions, Transaction};
use automerge::{
    ActorId, Automerge, AutomergeError, ChangeHash, LoadOptions, ObjType, OnPartialLoad, ROOT,
    ReadDoc, ScalarValue, Value,
};
use noyau::donnees::acces::Acces;
use noyau::donnees::changement::Operation;
use noyau::donnees::compaction::{self, Lu};
use noyau::donnees::demarrage::{self, Sort};
use noyau::donnees::fonds::{
    IDENTIFIANT, LIGNEE, MIGRATIONS, PRISES, PROPRIETAIRE, VERSION_DE_SCHEMA,
};
#[cfg(feature = "pannes")]
use noyau::donnees::gestes;
use noyau::donnees::version::{self, Lecture, Migration};
use services::application;
use synchro::{Copie, Echange};

use crate::demarrage::{
    demarrer, sort_dans_ajouts, sort_dans_le_dossier, voisin_lisible, voisins_du_complet_lisibles,
};
use crate::fichier::{self, Echec, ecrire_a_cote};
use crate::pannes::{self, Etape};
use crate::{Avertissement, Erreur, LectureSeule, RacineDesDonnees, disque, empreinte};

/// Ce que la naissance du fonds écrit sur la genèse (§ 31 · 2.3), que pose
/// `noyau::donnees::gestes::naissance` : son identifiant, sa lignée (§ 31 · 2.5) et l'identifiant
/// de propriétaire. La naissance de `RacineDesDonnees::faire_naitre_le_fonds`, un point d'entrée
/// des tests, sous la fonction `pannes`.
#[cfg(feature = "pannes")]
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
    /// La version de schéma que connaît l'application qui l'a ouvert, ou fait naître : celle des
    /// migrations de l'ouverture (§ 31 · 2.4). La relecture du disque et la réception la relisent
    /// (`raison_de_la_memoire`).
    version_connue: u64,
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

    /// Le prochain message de l'échange pour l'autre copie, ou `None` quand il n'y a plus rien à
    /// dire. Un document entier transmet ses modifications (`Echange::prochain_message`) ; un
    /// document de version de schéma plus récente n'en transmet aucune (§ 31 · 2.4,
    /// `Echange::prochain_message_sans_modification`). Un document en lecture seule que son accès
    /// prive d'échange n'envoie rien : `None` aussi.
    pub fn prochain_message(&self, echange: &mut Echange) -> Option<Vec<u8>> {
        let acces = self.acces();
        if !acces.echange() {
            return None;
        }
        if acces.ecrit() {
            echange.prochain_message(&self.automerge)
        } else {
            echange.prochain_message_sans_modification(&self.automerge)
        }
    }

    /// § 31 · 2.1 : ce que la synchronisation reçoit ne l'est qu'une fois écrit. Le message
    /// s'applique (`Echange::recevoir`) ; si les heads ont changé, que la réception ait réussi ou
    /// non, `save_after` des heads d'avant s'écrit en un fichier d'ajout avant la réponse, avec
    /// les issues de [`Document::appliquer`] : faite, faite avec un avertissement, ou refusée.
    /// Rien de neuf, rien d'écrit.
    ///
    /// Une réception dont l'écriture est refusée relit le document du disque, qui perd ce que
    /// l'échange croit reçu : l'échange reprend (`Echange::reprendre`), pour que l'autre copie
    /// renvoie ce qu'elle a envoyé, et `Erreur::Disque` se rend. Une réception refusée en chemin,
    /// que l'échange refuse, rend `Erreur::Synchro`, ce qu'elle a apporté écrit ; l'avertissement
    /// de cette écriture, s'il en vient un, rejoint [`Document::avertissements`]. Un document en
    /// lecture seule que son accès prive d'échange : `Erreur::LectureSeule`, et le message ne
    /// s'applique pas. Un document de version de schéma plus récente (§ 31 · 2.4) reçoit, et ce
    /// qu'il reçoit s'écrit, sans compaction. Un document entier, ou de version de schéma plus
    /// récente, relit sa version de schéma à chaque réception qui change ses heads, avant que ce
    /// qu'il a reçu ne s'écrive : plus récente que la version de schéma connue, il est en lecture
    /// seule (`VersionPlusRecente`, qui dit la version de schéma relue), et ne se compacte pas ;
    /// illisible, il passe en lecture seule (`VersionIllisible`), sans échange. Devant la version
    /// de schéma connue, ou une plus ancienne, son accès ne change pas : un document plus récent
    /// le reste, et ne redevient entier qu'à sa réouverture ; son échange n'envoie, jusque-là,
    /// aucune modification. Une relecture du disque, si l'écriture est refusée, relit la version
    /// de schéma du disque.
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
        // § 31 · 2.4 : ce qui est reçu peut porter une version de schéma plus récente que la
        // version de schéma connue, ou illisible : la mémoire la lit comme la relecture du disque,
        // avant que l'écriture de ce qui est reçu ne décide de compacter, chez un document entier
        // comme chez un document plus récent. Un document devenu plus récent ne se compacte pas,
        // et ce qu'il a reçu s'écrit quand même ; une version de schéma illisible le prive
        // d'échange. Seule une raison remplace celle d'un document plus récent : il le reste
        // devant la version de schéma connue, ou une plus ancienne, jusqu'à sa réouverture, son
        // échange ne passant pas, en chemin, à `Echange::prochain_message`.
        if matches!(
            self.lecture_seule,
            None | Some(LectureSeule::VersionPlusRecente { .. })
        ) && let Some(raison) = self.raison_de_la_memoire()
        {
            self.lecture_seule = Some(raison);
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

    /// § 31 · 2.1 : `operation`, appliquée par `services::application::appliquer`, en une
    /// modification atomique, avec son message et son heure, faite une fois son fichier d'ajout en
    /// place : `save_after` des heads d'avant, sous `ajouts/<empreinte>.automerge`, écrit à côté
    /// puis renommé. Une opération inapplicable (un objet manque sur son chemin, elle ne change
    /// rien, ou Automerge la refuse) : `Erreur::Inapplicable`, la modification annulée, et rien ne
    /// s'écrit ; le document reste ouvert. Un document en lecture seule n'écrit pas :
    /// `Erreur::LectureSeule`, et l'opération ne s'applique pas.
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
    ///   `Erreur::Disque` se rend ; de même, avant tout voisin, quand `ajouts/`, absent, ne peut
    ///   être recréé. Une relecture qui échoue met le document en lecture seule
    ///   (`LectureSeule::RelectureManquee`), sur sa mémoire d'avant l'écriture refusée.
    ///
    /// Un `ajouts/` absent, que l'ouverture ne crée pas, se recrée avant le voisin du fichier
    /// d'ajout.
    ///
    /// Aucun message ne se prépare pendant une écriture : `appliquer` tient le document emprunté
    /// en écriture jusqu'à son retour, et une lecture du document tenue pendant ce temps, pour
    /// l'échange, ne compile pas. Le premier exemple ne compile pas ; le second, qui prépare le
    /// message après l'écriture, compile.
    ///
    // § 31 · 10, test 8 : une synchronisation lancée pendant l'écriture ne voit pas l'opération en cours
    /// ```compile_fail,E0502
    /// fn pendant(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange,
    ///            operation: &noyau::donnees::changement::Operation) {
    ///     let lu = document.automerge();
    ///     let _ = document.appliquer(operation);
    ///     let _ = echange.prochain_message(lu);
    /// }
    /// ```
    ///
    /// ```
    /// fn apres(document: &mut stockage::Document<'_>, echange: &mut synchro::Echange,
    ///          operation: &noyau::donnees::changement::Operation) {
    ///     let _ = document.appliquer(operation);
    ///     let _ = document.prochain_message(echange);
    /// }
    /// ```
    pub fn appliquer(&mut self, operation: &Operation) -> Result<Option<Avertissement>, Erreur> {
        self.exiger(Acces::ecrit)?;
        let avant = self.automerge.get_heads();
        application::appliquer(&mut self.automerge, operation).map_err(Erreur::Inapplicable)?;
        self.ecrire_l_ajout(&avant)
    }

    /// L'écriture de [`Document::appliquer`], d'un geste que donne une fermeture sur la
    /// transaction, sous `message`, à `heure` (`commit_with`), avec les mêmes issues : un point
    /// d'entrée des tests, hors de l'API livrée, sous la fonction `pannes`. Une écriture sans
    /// modification n'écrit rien, et rend `Ok(None)`. Le geste refusé, la transaction est annulée
    /// (`rollback`), `Erreur::Automerge` se rend, et le document reste ouvert. Un document en
    /// lecture seule n'écrit pas : `Erreur::LectureSeule`, et le geste n'est pas fait.
    #[cfg(feature = "pannes")]
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
    /// le renommer ; puis chaque voisin du fichier complet que le dossier du document garde
    /// (`complet.automerge.<processus>[-<n>].partiel`), et chaque fichier d'ajout, se lit par
    /// `fichier::lire`, et ceux que désigne `a_retirer` sont retirés par `fichier::retirer`. Un
    /// document en lecture seule ne se compacte pas : `Erreur::LectureSeule`, et rien ne s'écrit.
    ///
    /// Les octets de `save()` qui ne se lisent pas, ou ne mènent pas aux mêmes heads, ne
    /// s'écrivent pas : `CompactionInfidele`, et l'ancien fichier complet garde l'histoire qu'il
    /// porte seul (« elle ne retire jamais d'histoire »). Un voisin relu du disque qui ne rend
    /// pas ces mêmes octets ne prend pas la place : `Erreur::Disque` se rend
    /// (`io::ErrorKind::InvalidData`), l'ancien fichier complet reste en place, et rien n'est
    /// retiré. Un fichier d'ajout est lisible quand son contenu donne l'empreinte de son nom et
    /// que `load_incremental` le charge, dans le fichier complet relu ; un voisin du fichier
    /// complet, quand il n'est pas vide et qu'`Automerge::load` le relit sans erreur ; l'un et
    /// l'autre sont absorbés quand, chargés dans le fichier complet relu, ils n'y ajoutent rien.
    /// Le voisin d'un fichier d'ajout que le démarrage n'a pu mettre en place n'est pas des
    /// siens : le démarrage suivant le met en place. Un dossier `ajouts/` absent n'a aucun
    /// fichier d'ajout, et la compaction ne le crée pas : l'écriture suivante le recrée. Une
    /// compaction qui échoue est due : l'écriture suivante la retente, même sans fichier complet,
    /// qui pèse alors 0 octet ; une compaction qui réussit ne l'est plus.
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

        let mut lus = Vec::new();
        for chemin in partiels(&self.dossier)? {
            let nom = nom_de(&chemin);
            if demarrage::fichier_du_voisin(&nom) != Some("complet.automerge") {
                continue;
            }
            let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
            lus.push(Lu {
                lisible: voisin_lisible(&octets),
                absorbe: charger_contre(&relu, &octets) == Some(true),
                nom,
            });
        }
        for chemin in fichiers_d_ajout(&self.dossier.join("ajouts"))? {
            let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
            lus.push(lu_contre(&relu, &chemin, &octets));
        }
        for nom in compaction::a_retirer(&lus) {
            let chemin = dans_le_dossier(&self.dossier, &nom);
            fichier::retirer(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        }
        Ok(())
    }

    /// Le fichier d'ajout des heads `avant` (`ecrire_le_fichier_d_ajout`) ; puis la compaction,
    /// quand `apres_une_ecriture` le dit (§ 31 · 2.2). Ses trois issues sont dites à
    /// [`Document::appliquer`], qui l'appelle comme [`Document::recevoir`], et comme `ecrire`,
    /// sous la fonction `pannes`. Un refus, de recréer `ajouts/` comme du voisin, est une écriture
    /// refusée : la mémoire porte déjà la modification, et se relit.
    fn ecrire_l_ajout(&mut self, avant: &[ChangeHash]) -> Result<Option<Avertissement>, Erreur> {
        match self.ecrire_le_fichier_d_ajout(avant) {
            // Rien n'est en place : la mémoire porte ce que le disque n'a pas, et se relit.
            Err(refus) => {
                self.relire_du_disque(avant);
                Err(refus)
            }
            // Le fichier est en place : l'écriture est faite. La compaction, due, réécrira à
            // l'écriture suivante un fichier complet qui porte ce fichier d'ajout.
            Ok(Some(avertissement)) => Ok(Some(avertissement)),
            // L'écriture est faite : ni une compaction qui échoue, ni la pesée des fichiers qui la
            // précède, ne la défont.
            Ok(None) => Ok(self
                .compacter_apres_une_ecriture()
                .err()
                .map(Avertissement::Compaction)),
        }
    }

    /// `save_after` des heads `avant`, sous `ajouts/<empreinte>.automerge`, écrit à côté puis
    /// renommé, `ajouts/` recréé d'abord s'il manque (`recreer_les_ajouts`), sans relecture du
    /// disque ni compaction : `ecrire_l_ajout` et la migration à l'ouverture l'appellent. Refusé
    /// avant la fin du renommage, le voisin est ôté, et `Erreur::Disque` se rend ; en place, mais
    /// son renommage non rendu durable, `Avertissement::NonDurable`, et la compaction est due ;
    /// sinon `Ok(None)`.
    fn ecrire_le_fichier_d_ajout(
        &mut self,
        avant: &[ChangeHash],
    ) -> Result<Option<Avertissement>, Erreur> {
        self.recreer_les_ajouts()?;
        let octets = self.automerge.save_after(avant);
        let chemin = self
            .dossier
            .join("ajouts")
            .join(format!("{}.automerge", empreinte(&octets)));
        match ecrire_a_cote(&chemin, &octets) {
            Err(Echec::Refusee(erreur)) => Err(disque(&chemin, erreur)),
            Err(Echec::NonDurable(erreur)) => {
                self.compaction_due = true;
                Ok(Some(Avertissement::NonDurable { chemin, erreur }))
            }
            Ok(()) => Ok(None),
        }
    }

    /// § 31 · 2.4 : applique les migrations `a_appliquer`, dans l'ordre de la suite des
    /// migrations, par `services::application::migrer`, puis écrit `save_after` des heads d'avant
    /// en un seul fichier d'ajout (`ecrire_le_fichier_d_ajout`), comme une écriture : refusé,
    /// `Erreur::Disque` se rend, et le fonds ne s'ouvre pas ; en place, mais non durable,
    /// l'avertissement rejoint ceux du document, et la compaction est due. Une migration que
    /// `migrer` refuse, qui ne se lit pas ou attend ses dépendances, n'a pas fait migrer le
    /// document : il passe en lecture seule (`LectureSeule::VersionIllisible`), et rien ne
    /// s'écrit, ni la migration, ni, ensuite, une opération à l'ancienne version de schéma.
    fn migrer(&mut self, a_appliquer: &[Migration<'_>]) -> Result<(), Erreur> {
        let avant = self.automerge.get_heads();
        if application::migrer(&mut self.automerge, a_appliquer).is_err() {
            self.lecture_seule = Some(LectureSeule::VersionIllisible);
            return Ok(());
        }
        let avertissement = self.ecrire_le_fichier_d_ajout(&avant)?;
        self.avertissements.extend(avertissement);
        Ok(())
    }

    /// Recrée `ajouts/` dans le dossier du document s'il manque (`fs::create_dir`) : l'ouverture
    /// ne crée aucun dossier dans `documents/`, et un fonds peut s'ouvrir sans lui (un voisin du
    /// fichier complet seul, ou une copie qui ne garde pas un dossier vide). Créé, hors de
    /// Windows, il ne survit à une coupure qu'une fois le dossier du document synchronisé
    /// (`fichier::rendre_durable`), comme à la naissance ; un dossier ne se synchronise pas sous
    /// Windows. Si cette synchronisation échoue, `ajouts/`, vide, se retire (`fs::remove_dir`, son
    /// refus tu) avant que le refus se rende : un `ajouts/` présent est ainsi un dossier dont la
    /// création a été rendue durable, et l'écriture suivante le recrée, puis le synchronise.
    /// Présent, ou remplacé par un fichier, il est laissé tel quel : l'écriture à côté refusera
    /// alors son voisin. Un refus du disque rend `Erreur::Disque`.
    fn recreer_les_ajouts(&self) -> Result<(), Erreur> {
        let ajouts = self.dossier.join("ajouts");
        match fs::create_dir(&ajouts) {
            Err(erreur) if erreur.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(erreur) => Err(disque(&ajouts, erreur)),
            Ok(()) => {
                #[cfg(not(windows))]
                if let Err(erreur) = fichier::rendre_durable(&ajouts) {
                    let _ = fs::remove_dir(&ajouts);
                    return Err(disque(&self.dossier, erreur));
                }
                Ok(())
            }
        }
    }

    /// La compaction qui suit une écriture faite, quand `apres_une_ecriture` la dit. Une pesée
    /// des fichiers qui échoue la rend due, comme une compaction qui échoue. Un document qui
    /// n'écrit pas ne se compacte jamais (`apres_une_ecriture`) : rien ne se pèse, et le disque
    /// n'est pas lu.
    fn compacter_apres_une_ecriture(&mut self) -> Result<(), Erreur> {
        if !self.acces().ecrit() {
            return Ok(());
        }
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
    /// d'ajout. Un fichier complet absent (un fonds ouvert par ses seuls voisins, sa compaction
    /// d'ouverture en échec) pèse 0 octet : la compaction se retente.
    fn faut_il_compacter(&self) -> Result<bool, Erreur> {
        let complet = self.dossier.join("complet.automerge");
        let octets_du_complet = match fs::metadata(&complet) {
            Ok(metadonnees) => metadonnees.len(),
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => 0,
            Err(erreur) => return Err(disque(&complet, erreur)),
        };
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

    /// § 31 · 2.1 : après une écriture refusée, la mémoire se relit du disque (`complet_present`,
    /// puis `lire_le_dossier`, sans démarrage : rien n'est retiré ni renommé), sans toucher à la
    /// garde, et tire un acteur neuf ; une raison de lecture seule que la relecture trouve devient
    /// celle du document, sur ce qui se lit. Lue entière, la mémoire relue lit sa version de
    /// schéma comme l'ouverture, devant la version de schéma connue du document, sans migrer
    /// (`raison_de_la_memoire`, § 31 · 2.4) : une copie plus récente le reste. Une relecture qui
    /// échoue laisse la mémoire d'avant l'écriture refusée (`fork_at` des heads `avant`), en
    /// lecture seule `RelectureManquee`, sur le fichier qu'elle n'a pu lire, ou sur le dossier du
    /// document si son erreur n'en nomme aucun.
    fn relire_du_disque(&mut self, avant: &[ChangeHash]) {
        let relu = complet_present(&self.dossier)
            .and_then(|present| lire_le_dossier(&self.dossier, present));
        match relu {
            Ok((mut automerge, lecture_seule, _, _)) => {
                automerge.set_actor(ActorId::random());
                self.automerge = automerge;
                let lecture_seule = lecture_seule.or_else(|| self.raison_de_la_memoire());
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

    /// § 31 · 2.4 : la raison de lecture seule que donne la version de schéma de la mémoire,
    /// devant la version de schéma connue du document, sans migrer (`raison_de_la_version`). La
    /// relecture du disque et la réception la lisent ainsi, quand la mémoire se lit entière.
    fn raison_de_la_memoire(&self) -> Option<LectureSeule> {
        raison_de_la_version(version_de_schema(&self.automerge), self.version_connue)
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

/// Le fonds de `racine`, la garde prise, selon les migrations `migrations` : d'abord le démarrage
/// (`demarrer`), puis, s'il est né, sa lecture (`lire_le_dossier`), avec un acteur neuf, en
/// lecture seule si un fichier ne se lit pas ; puis, s'il se lit entier, sa version de schéma
/// devant celle que donnent les migrations (`version::connue`, § 31 · 2.4) : la même, il est
/// entier ; plus ancienne, il migre (`Document::migrer`, les migrations de
/// `version::a_appliquer`) : une migration que le disque refuse rend `Erreur::Disque`, sans
/// document, et une migration qui ne s'applique pas le met en lecture seule (`VersionIllisible`),
/// sans rien écrire ; plus récente, absente ou illisible, il est en lecture seule
/// (`raison_de_la_version`) ; un fonds illisible ne migre pas. Puis sa compaction, quand
/// `a_l_ouverture` le dit, sur l'accès qui suit la lecture de la version de schéma et sur ce que
/// la lecture a lu, avant la migration : un fichier d'ajout lu, ou un voisin du fichier complet.
/// Pupitre neuf (`pupitre_neuf` : ni fichier complet, ni voisin du fichier complet lu) : `None`.
/// Les avertissements du démarrage (`VoisinRestant`, `NonDurable`) sont ceux du document ; un
/// renommage du démarrage non rendu durable rend la compaction due. Une compaction qui échoue
/// n'empêche pas d'ouvrir : son erreur vient en avertissement (`Avertissement::Compaction`), rien
/// n'est perdu, et la compaction est due : l'écriture suivante retire ce qu'elle n'a pas retiré.
/// Enfin, même en lecture seule, chaque nom provisoire de `captations/` dont le contenu donne une
/// empreinte que nomme le document lu (`empreintes_nommees`) prend ce nom
/// (`Captations::ranger_les_noms_provisoires`, § 31 · 6.1) ; ses avertissements
/// (`NomProvisoireRestant`, `NonDurable`) rejoignent ceux du document, sans empêcher d'ouvrir. Une
/// erreur du démarrage, de la lecture ou de la migration rend la garde : le document qui n'est pas
/// rendu la rend à sa fin.
///
/// `fonds()` l'appelle avec `MIGRATIONS`, et `fonds_selon`, sous la fonction `pannes`, avec les
/// migrations que lui donne un test.
pub(crate) fn rouvrir_le_fonds<'r>(
    racine: &'r RacineDesDonnees,
    migrations: &[Migration<'_>],
) -> Result<Option<Document<'r>>, Erreur> {
    let garde = racine.prendre_la_garde()?;
    let dossier = dossier_du_fonds(racine);
    let avertissements = demarrer(&racine.chemin().join("documents"), &dossier)?;
    let present = complet_present(&dossier)?;
    let (mut automerge, lecture_seule, ajouts, voisins) = lire_le_dossier(&dossier, present)?;
    if demarrage::pupitre_neuf(present, voisins) {
        return Ok(None);
    }
    automerge.set_actor(ActorId::random());
    // Le document né rend la garde à sa fin, même si la migration ou la compaction qui suivent
    // échouent.
    garde.passer_au_document();
    let compaction_due = avertissements
        .iter()
        .any(|avertissement| matches!(avertissement, Avertissement::NonDurable { .. }));
    let connue = version::connue(migrations);
    let mut document = Document {
        racine,
        dossier,
        automerge,
        lecture_seule,
        compaction_due,
        avertissements,
        version_connue: connue,
    };
    // Un document illisible ne migre pas : sa version de schéma ne se lit que s'il se lit entier.
    if document.lecture_seule.is_none() {
        let lue = version_de_schema(&document.automerge);
        match version::lecture(lue, connue) {
            Lecture::Migrer { de, .. } => document.migrer(version::a_appliquer(migrations, de))?,
            Lecture::Ecrire | Lecture::LireSansEcrire | Lecture::Illisible => {
                document.lecture_seule = raison_de_la_version(lue, connue);
            }
        }
    }
    // Les fichiers lus avant la migration : celui de la migration ne compte pas.
    if compaction::a_l_ouverture(document.acces(), ajouts, voisins > 0)
        && let Err(erreur) = document.compacter()
    {
        // `compacter` a rendu la compaction due : l'écriture suivante la retente.
        document
            .avertissements
            .push(Avertissement::Compaction(erreur));
    }
    // Le rangement suit la compaction d'ouverture, même en lecture seule : rien n'y fait échouer
    // l'ouverture.
    let nommees = empreintes_nommees(&document.automerge);
    let restants = racine.captations().ranger_les_noms_provisoires(&nommees);
    document.avertissements.extend(restants);
    Ok(Some(document))
}

/// Les empreintes que nomme le document `automerge`, tel qu'il se lit : les clés de ses prises
/// (`PRISES`, à sa racine). Sans prises qui se lisent comme un objet à clés, aucune.
fn empreintes_nommees(automerge: &Automerge) -> BTreeSet<String> {
    match automerge.get(ROOT, PRISES) {
        Ok(Some((Value::Object(ObjType::Map), prises))) => automerge.keys(&prises).collect(),
        _ => BTreeSet::new(),
    }
}

/// § 31 · 2.4 : la version de schéma que porte la racine du document `automerge`
/// (`VERSION_DE_SCHEMA`), un entier non signé (`ScalarValue::Uint`) ; absente, ou de tout autre
/// type, `None`.
fn version_de_schema(automerge: &Automerge) -> Option<u64> {
    match automerge.get(ROOT, VERSION_DE_SCHEMA) {
        Ok(Some((Value::Scalar(valeur), _))) => match *valeur {
            ScalarValue::Uint(version) => Some(version),
            _ => None,
        },
        _ => None,
    }
}

/// La raison de lecture seule que donne la version de schéma lue `lue`, devant la version de
/// schéma connue `connue` (`version::lecture`, § 31 · 2.4) : une version de schéma plus récente,
/// `VersionPlusRecente` ; une version de schéma absente, nulle ou d'un autre type,
/// `VersionIllisible` ; la version de schéma connue, ou une version de schéma plus ancienne,
/// aucune (l'ouverture migre celle-ci, la relecture du disque et la réception non).
fn raison_de_la_version(lue: Option<u64>, connue: u64) -> Option<LectureSeule> {
    match version::lecture(lue, connue) {
        Lecture::Ecrire | Lecture::Migrer { .. } => None,
        Lecture::LireSansEcrire => lue.map(|version| LectureSeule::VersionPlusRecente { version }),
        Lecture::Illisible => Some(LectureSeule::VersionIllisible),
    }
}

/// Le fichier complet du dossier `dossier` d'un document existe (`try_exists`). Un refus du disque
/// rend `Erreur::Disque`, sur le fichier complet.
fn complet_present(dossier: &Path) -> Result<bool, Erreur> {
    let complet = dossier.join("complet.automerge");
    complet
        .try_exists()
        .map_err(|erreur| disque(&complet, erreur))
}

/// Lit le dossier `dossier` d'un document, sans compaction ni démarrage : son fichier complet
/// (`lire_le_complet`), s'il existe (`present`, que l'appelant tient de `complet_present`), puis
/// chaque voisin du fichier complet que le démarrage lit (`lire_les_voisins_du_complet`), puis
/// ses fichiers d'ajout et chaque voisin d'un fichier d'ajout que le démarrage met en place, dans
/// l'ordre des noms (`lire_les_ajouts`), chacun par `fichier::lire` ; tout lu, elle cherche les
/// dépendances manquantes (`dependances_manquantes`). Elle ne construit aucun `Document`, ne
/// touche pas à la garde, et ne retire ni ne renomme rien : l'ouverture et la relecture du disque
/// l'appellent. Sans fichier complet ni voisin lu, la mémoire est un document neuf, sans genèse,
/// qui n'est jamais entier : `CompletIllisible`, comme un fichier complet vide.
///
/// Rend la mémoire lue, ce qui se lit ; la première raison de lecture seule qu'elle trouve, dans
/// cet ordre : le fichier complet, puis ses voisins, puis les fichiers d'ajout, puis les
/// dépendances ; le nombre de fichiers d'ajout lus, un voisin d'un fichier d'ajout resté compris ;
/// et le nombre de voisins du fichier complet lus. Un refus du disque à la lecture d'un fichier,
/// ou d'un dossier qui n'est pas absent, rend `Erreur::Disque`.
fn lire_le_dossier(
    dossier: &Path,
    present: bool,
) -> Result<(Automerge, Option<LectureSeule>, usize, usize), Erreur> {
    let complet = dossier.join("complet.automerge");
    let (mut automerge, du_complet) = if present {
        lire_le_complet(&complet)?
    } else {
        (Automerge::new(), None)
    };
    let voisins = lire_les_voisins_du_complet(&mut automerge, dossier)?;
    let du_complet = if !present && voisins.lus == 0 {
        Some(LectureSeule::CompletIllisible {
            chemin: complet.clone(),
        })
    } else {
        du_complet
    };
    let ajouts = lire_les_ajouts(&mut automerge, &dossier.join("ajouts"))?;
    let raison = du_complet
        .or(voisins.raison)
        .or(ajouts.raison)
        .or_else(|| dependances_manquantes(&automerge, &ajouts.charges, &complet));
    Ok((automerge, raison, ajouts.nombre, voisins.lus))
}

/// Ce que `lire_les_voisins_du_complet` rend : le nombre de voisins du fichier complet lus, et la
/// raison de lecture seule du premier que `load_incremental` refuse.
struct VoisinsLus {
    lus: usize,
    raison: Option<LectureSeule>,
}

/// Charge dans `automerge` (`load_incremental`), après le fichier complet, ou sans lui, chaque
/// voisin du fichier complet du dossier `dossier` dont le sort est `Lire` (non vide, et
/// qu'`Automerge::load` relit sans erreur), dans l'ordre des noms, chacun par `fichier::lire`.
/// Un autre `.partiel` n'est ni chargé ni compté. Un voisin lisible que `load_incremental`
/// refuse n'en est pas moins lu, et compté : il donne `CompletIllisible`, sur son chemin ; le
/// premier, dans l'ordre des noms, l'emporte.
fn lire_les_voisins_du_complet(
    automerge: &mut Automerge,
    dossier: &Path,
) -> Result<VoisinsLus, Erreur> {
    let mut voisins = VoisinsLus {
        lus: 0,
        raison: None,
    };
    for chemin in partiels(dossier)? {
        let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        if sort_dans_le_dossier(&chemin, &octets) != Sort::Lire {
            continue;
        }
        voisins.lus += 1;
        if automerge.load_incremental(&octets).is_err() {
            voisins.raison = voisins
                .raison
                .or(Some(LectureSeule::CompletIllisible { chemin }));
        }
    }
    Ok(voisins)
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
/// des noms ; la raison de lecture seule du premier qui ne l'est pas ; et le nombre de fichiers
/// d'ajout lus, un voisin d'un fichier d'ajout resté compris.
struct AjoutsLus {
    charges: Vec<(PathBuf, Vec<u8>)>,
    raison: Option<LectureSeule>,
    nombre: usize,
}

/// Lit, dans l'ordre des noms, chaque fichier d'ajout de `ajouts` et chaque voisin d'un fichier
/// d'ajout que le démarrage met en place (`Sort::MettreEnPlace` : son contenu donne l'empreinte
/// que porte le nom de son fichier), resté là quand son renommage a été refusé ; chacun par
/// `fichier::lire`, et charge (`load_incremental`) ceux qui se lisent. Un autre `.partiel` n'est
/// ni chargé ni compté. Un fichier d'ajout dont le contenu ne donne pas l'empreinte de son nom
/// n'est pas chargé, et donne `AjoutAltere` ; un fichier qui la donne, mais que
/// `load_incremental` refuse, ne l'est pas non plus, et donne `AjoutIllisible`. Le premier, dans
/// l'ordre des noms, l'emporte. Rien n'est retiré ni réécrit. Un dossier `ajouts/` absent n'a
/// aucun fichier.
fn lire_les_ajouts(automerge: &mut Automerge, ajouts: &Path) -> Result<AjoutsLus, Erreur> {
    let mut lus = AjoutsLus {
        charges: Vec::new(),
        raison: None,
        nombre: 0,
    };
    for chemin in fichiers_de(ajouts, &["automerge", "partiel"])? {
        let octets = fichier::lire(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
        let conforme = if chemin.extension() == Some(OsStr::new("partiel")) {
            // Le voisin d'un fichier d'ajout ne se lit que s'il en prend la place : son contenu
            // donne alors l'empreinte du nom de ce fichier.
            if sort_dans_ajouts(&chemin, &octets) != Sort::MettreEnPlace {
                continue;
            }
            true
        } else {
            est_conforme(&chemin, &octets)
        };
        lus.nombre += 1;
        // `load_incremental` s'arrête sans erreur au premier tronçon altéré : seule l'empreinte
        // du nom dit qu'un fichier d'ajout a changé.
        let refus = if !conforme {
            Some(LectureSeule::AjoutAltere { chemin })
        } else if automerge.load_incremental(&octets).is_err() {
            Some(LectureSeule::AjoutIllisible { chemin })
        } else {
            lus.charges.push((chemin, octets));
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
    charges: &[(PathBuf, Vec<u8>)],
    complet: &Path,
) -> Option<LectureSeule> {
    let manquantes = automerge.get_missing_deps(&[]);
    if manquantes.is_empty() {
        return None;
    }
    let chemin = charges
        .iter()
        .find(|(_, octets)| attend_l_une_de(&manquantes, octets))
        .map_or(complet, |(chemin, _)| chemin.as_path());
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

/// Les fichiers d'ajout de `ajouts`, `<empreinte>.automerge`, triés par nom. Un `.partiel` n'en
/// est pas un : le démarrage lui donne son sort. Un dossier `ajouts/` absent n'en a aucun.
fn fichiers_d_ajout(ajouts: &Path) -> Result<Vec<PathBuf>, Erreur> {
    fichiers_de(ajouts, &["automerge"])
}

/// Les `.partiel` de `dossier`, triés par nom. Un dossier absent n'en a aucun.
pub(crate) fn partiels(dossier: &Path) -> Result<Vec<PathBuf>, Erreur> {
    fichiers_de(dossier, &["partiel"])
}

/// Ce que contient `dossier` sous l'une des `extensions`, trié par nom ; un autre nom n'est ni
/// lu ni retiré. Un dossier absent (`io::ErrorKind::NotFound`) ne contient rien, et n'est pas
/// créé ; toute autre erreur de lecture du dossier rend `Erreur::Disque`.
fn fichiers_de(dossier: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>, Erreur> {
    let entrees = match fs::read_dir(dossier) {
        Ok(entrees) => entrees,
        Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(erreur) => return Err(disque(dossier, erreur)),
    };
    let mut fichiers = Vec::new();
    for entree in entrees {
        let chemin = entree.map_err(|erreur| disque(dossier, erreur))?.path();
        if extensions
            .iter()
            .any(|extension| chemin.extension() == Some(OsStr::new(extension)))
        {
            fichiers.push(chemin);
        }
    }
    fichiers.sort();
    Ok(fichiers)
}

/// Le nom du fichier `chemin`.
pub(crate) fn nom_de(chemin: &Path) -> String {
    chemin
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Le chemin de `nom`, relatif au dossier du document `dossier`, `/` pour séparateur, tel que la
/// compaction le nomme (`Lu`) : chaque segment s'ajoute à `dossier`.
fn dans_le_dossier(dossier: &Path, nom: &str) -> PathBuf {
    nom.split('/')
        .fold(dossier.to_path_buf(), |chemin, segment| {
            chemin.join(segment)
        })
}

/// Le contenu `octets` du fichier d'ajout `chemin` donne l'empreinte de son nom.
fn est_conforme(chemin: &Path, octets: &[u8]) -> bool {
    chemin.file_stem() == Some(OsStr::new(&empreinte(octets)))
}

/// Ce que la compaction lit du fichier d'ajout `chemin`, de contenu `octets`, contre le fichier
/// complet relu `relu` (`charger_contre`) : son nom, `ajouts/<empreinte>.automerge` ; lisible,
/// son contenu donne l'empreinte de son nom (`est_conforme`), et il s'est chargé sans erreur ;
/// absorbé, chargé, il n'ajoute rien à `relu`.
fn lu_contre(relu: &Automerge, chemin: &Path, octets: &[u8]) -> Lu {
    let charge = charger_contre(relu, octets);
    Lu {
        nom: format!("ajouts/{}", nom_de(chemin)),
        lisible: charge.is_some() && est_conforme(chemin, octets),
        absorbe: charge == Some(true),
    }
}

/// Le contenu `octets` d'un fichier d'ajout ou d'un voisin du fichier complet, chargé
/// (`load_incremental`) dans un `fork()` du fichier complet relu `relu` : `None` s'il ne se
/// charge pas ; sinon, s'il est absorbé : il laisse les heads de `relu` inchangées, et aucune
/// modification n'y attend ses dépendances.
fn charger_contre(relu: &Automerge, octets: &[u8]) -> Option<bool> {
    let mut avec = relu.fork();
    avec.load_incremental(octets).ok()?;
    Some(avec.get_heads() == relu.get_heads() && avec.get_missing_deps(&[]).is_empty())
}

/// Le poids de `chemin` sur le disque, en octets.
fn poids(chemin: &Path) -> Result<u64, Erreur> {
    Ok(fs::metadata(chemin)
        .map_err(|erreur| disque(chemin, erreur))?
        .len())
}

/// Fait naître le fonds de `racine` de la genèse, de `naissance` et du premier geste `geste`,
/// appliqués par `services::application::faire_naitre`, par `naitre`. Une opération
/// inapplicable rend `Erreur::Inapplicable`, et rien ne s'écrit.
pub(crate) fn faire_naitre<'r>(
    racine: &'r RacineDesDonnees,
    naissance: &Operation,
    geste: &Operation,
) -> Result<Document<'r>, Erreur> {
    naitre(racine, || {
        application::faire_naitre(naissance, geste).map_err(Erreur::Inapplicable)
    })
}

/// Fait naître le fonds de `racine`, par `naitre`, de la genèse et de la naissance que pose
/// `noyau::donnees::gestes::naissance` à l'heure `heure` (`services::application::naitre`), puis
/// du premier geste que donne la fermeture `geste`, sous `message`, à `heure` : un point d'entrée
/// des tests, sous la fonction `pannes`. Le geste refusé, sa transaction est annulée,
/// `Erreur::Automerge` se rend, et rien ne s'écrit.
#[cfg(feature = "pannes")]
pub(crate) fn faire_naitre_le_fonds<'r>(
    racine: &'r RacineDesDonnees,
    naissance: &Naissance<'_>,
    message: &str,
    heure: i64,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Document<'r>, Erreur> {
    naitre(racine, || {
        let operation = gestes::naissance(
            naissance.identifiant,
            naissance.lignee,
            naissance.proprietaire,
            heure,
        );
        let mut automerge = application::naitre(&operation).map_err(Erreur::Inapplicable)?;
        let mut tx = automerge.transaction();
        geste(&mut tx).map_err(Erreur::Automerge)?;
        tx.commit_with(a_l_heure(message, heure));
        Ok(automerge)
    })
}

/// Fait naître le fonds de `racine` : `construire` bâtit en mémoire la genèse, la naissance et le
/// premier geste, dans un document Automerge neuf, dont l'acteur est neuf ; puis un seul fichier
/// complet, écrit à côté puis renommé, une fois `ajouts/` créé. Refusé avant la fin de son
/// renommage, le fichier complet ne laisse que `ajouts/`, vide, et `Erreur::Disque` se rend ; en
/// place, mais son renommage non rendu durable, le document naît avec cet avertissement
/// (`Avertissement::NonDurable`), et sa compaction est due.
///
/// Le fonds ne naît que si Pupitre est neuf (`pupitre_neuf`) : sinon `Erreur::FondsDejaNe`, sans
/// que `construire` soit appelé. Un fichier complet, ou un voisin du fichier complet lisible (non
/// vide, et qu'`Automerge::load` relit sans erreur), chacun lu par `fichier::lire`, dit le fonds
/// né ; la naissance ne retire aucun voisin, et ne fait pas le démarrage. Un refus du disque à la
/// lecture du dossier du fonds, s'il n'est pas absent, ou d'un `.partiel` de ce dossier, rend
/// `Erreur::Disque`. Le refus de `construire` se rend tel quel, et rien ne s'écrit.
fn naitre<'r>(
    racine: &'r RacineDesDonnees,
    construire: impl FnOnce() -> Result<Automerge, Erreur>,
) -> Result<Document<'r>, Erreur> {
    let garde = racine.prendre_la_garde()?;
    let dossier = dossier_du_fonds(racine);
    let complet = dossier.join("complet.automerge");
    let present = complet_present(&dossier)?;
    if !demarrage::pupitre_neuf(present, voisins_du_complet_lisibles(&dossier)?) {
        return Err(Erreur::FondsDejaNe);
    }
    // Rien ne s'écrit avant que le document soit bâti : refusé, il ne laisse rien.
    let automerge = construire()?;
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
        // § 31 · 2.4 : la naissance ne migre pas. Le fonds naît à la version de schéma de la
        // genèse, 1, qui n'est la version de schéma connue que tant que `MIGRATIONS` est vide :
        // le jour où `MIGRATIONS` reçoit sa première migration, la naissance doit migrer, et
        // `le_fonds_nait_a_la_version_de_schema_connue` (`tests/version.rs`) devient rouge.
        version_connue: version::connue(MIGRATIONS),
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

/// Le message et l'heure d'une modification.
#[cfg(feature = "pannes")]
fn a_l_heure(message: &str, heure: i64) -> CommitOptions {
    CommitOptions::default()
        .with_message(message)
        .with_time(heure)
}
