//! Les services de pupitre (§ 40 · 3) : les opérations nommées et les lectures du fonds,
//! conduites sur les contrats `Documents` et `Captations`, que réalisent les adaptateurs, sur
//! fichiers (le stockage) ou en mémoire. `Services` conduit chaque commande : il tire de l'aléa
//! du `Contexte` les identifiants qu'elle forme, demande l'opération au noyau, l'écrit par
//! `Documents`, et rend la réponse aux types de `commandes`. `application` applique les
//! changements que le noyau calcule ; `lecture` tire du document le fonds lu, et l'arbre de sa
//! teneur, que le noyau reçoit ; `mesure` engendre le fonds de mesure (§ 31 · 10, test 13) par
//! les seules opérations de `Services`. Le troisième contrat, la source à verser (`Source`), a
//! sa réalisation en mémoire dans `source_en_memoire`, hors de la fonction `en-memoire` : le
//! générateur du fonds de mesure l'emploie.
//!
//! La fonction `en-memoire`, que seuls les tests demandent, compile le module `en_memoire`, la
//! paire d'adaptateurs en mémoire.

pub mod application;
mod contrats;
#[cfg(feature = "en-memoire")]
pub mod en_memoire;
pub mod lecture;
pub mod mesure;
pub mod source_en_memoire;

pub use commandes::{Raison, Refus};
pub use contrats::{
    Alea, Captations, Documents, Ecriture, Entree, FONDS_NON_NE, SorteDEntree, Source, Versement,
};

use std::cell::RefCell;
use std::collections::BTreeSet;

use automerge::{Automerge, ChangeHash};
use commandes::{
    Captation, Completer, Enregistrer, EntreeDuJournal, EtatDuFonds, Etiqueter, FENETRE_MAX, Faite,
    Fenetre, LONGUEUR_D_UNE_LIGNE, LigneDeLaBoite, Notee, Noter, Regrouper, Relier, Renommer,
    Reponse, RequeteEnregistree, Supprimer, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees,
    VueDuJournal,
};
use noyau::donnees::boite::{self, EnBoite};
use noyau::donnees::changement::Operation;
use noyau::donnees::gestes::{self, CaptationLue, GesteDeTri, IdeeLue, Impossible};
use noyau::donnees::identifiants::{ulid, uuid_v4};
use noyau::donnees::index::{self, Index};
use noyau::donnees::journal;
use noyau::donnees::requete;

/// Ce que le côté natif donne à une commande qui écrit : l'heure du geste et l'aléa, des
/// arguments, jamais l'horloge ni le hasard des services.
pub struct Contexte<'a> {
    /// L'heure du geste, en secondes depuis 1970.
    pub heure: i64,
    /// L'aléa d'où se tirent les identifiants que forme la commande.
    pub alea: &'a mut dyn Alea,
}

/// Ce que rend le versement d'une fiche : l'empreinte de sa captation, l'idée née avec elle, et
/// un avertissement, lisible : le versement est fait, mais Sacha doit en savoir quelque chose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versee {
    /// L'empreinte de la captation versée.
    pub empreinte: String,
    /// L'identifiant de l'idée née avec elle, un ULID.
    pub idee: String,
    /// Un avertissement, lisible.
    pub avertissement: Option<String>,
}

/// Les services d'un appareil : ses commandes, conduites sur ses documents et ses captations.
///
/// Un geste de tri (compléter, poser ou ôter une étiquette, relier, délier, regrouper, retirer,
/// rétablir) : en lecture seule, le refus de `lecture_seule()`, tel quel ; sans fonds, aucune idée
/// ne porte l'identifiant que vise la commande, le premier qu'elle nomme, ou la cible d'un
/// regroupement (`Raison::Impossible`) ; puis le fonds lu : chaque idée par
/// `lecture::lire_l_idee`, et, d'un regroupement, la captation par `lecture::lire_la_captation` et
/// celles de sa source par `lecture::captations_de` ; puis le geste, que calcule le noyau
/// (`noyau::donnees::gestes`), à l'heure du contexte, et qui, impossible, est refusé de
/// `Raison::Impossible`, son message le constat du noyau ; puis `Documents::appliquer`. Fait, il
/// devient le dernier geste de tri ; refusé, le dernier geste de tri ne change pas. Un geste de
/// tri ne tire rien de l'aléa.
///
/// Une opération sur une requête enregistrée (enregistrer, renommer, supprimer) : en lecture
/// seule, le refus de `lecture_seule()`, tel quel ; sans fonds, `Raison::Impossible`, « Aucun
/// fonds n'est né. », sans rien faire naître (question 20) ; puis, pour renommer et supprimer, la
/// requête que nomme la commande, qui ne se lit pas : `Impossible::RequeteAbsente` ; puis
/// l'opération, que calcule le noyau (`noyau::donnees::requete`), à l'heure du contexte, et qui,
/// impossible, est refusée de `Raison::Impossible`, son message le constat du noyau ; puis
/// `Documents::appliquer`. Enregistrer tire l'ULID de la requête ; renommer et supprimer ne tirent
/// rien. Aucune ne touche au dernier geste de tri : annuler ne la vise pas.
///
/// Les services tiennent l'index des idées interrogées (`noyau::donnees::index::Index`), un dérivé
/// jetable (I13), avec les têtes du fonds (`Automerge::get_heads`) dont il est bâti. Chaque lecture
/// de la boîte compare les têtes du fonds aux têtes gardées : différentes, ou sans index tenu,
/// l'index se rebâtit en entier du fonds lu (`en_boite`), jamais par morceaux ; sans fonds, la
/// lecture prend un index vide, et rien n'est gardé. Aucune écriture ne le touche (question 14) :
/// une écriture faite change les têtes, et la lecture qui suit le rebâtit. Les têtes ne suffisent
/// à dire l'index à jour que tant qu'`en_boite` ne lit que le document : ce qu'elle lirait
/// ailleurs pourrait changer sans changer les têtes.
pub struct Services<D: Documents, C: Captations> {
    documents: D,
    captations: C,
    appareil: gestes::Appareil,
    /// Le dernier geste de tri fait par ces services, que l'annulation défait (§ 30 · 11) ; aucun
    /// à leur ouverture, ni après une annulation faite.
    dernier_geste: Option<GesteDeTri>,
    /// L'index tenu, et les têtes du fonds dont il est bâti ; aucun à l'ouverture des services.
    index: RefCell<Option<(Vec<ChangeHash>, Index)>>,
}

impl<D: Documents, C: Captations> Services<D, C> {
    /// Les services de l'appareil `appareil`, sur `documents` et `captations`, sans dernier geste
    /// de tri ni index tenu.
    pub fn new(documents: D, captations: C, appareil: gestes::Appareil) -> Self {
        Services {
            documents,
            captations,
            appareil,
            dernier_geste: None,
            index: RefCell::new(None),
        }
    }

    /// Les documents.
    pub fn documents(&self) -> &D {
        &self.documents
    }

    /// Les documents, à modifier.
    pub fn documents_mut(&mut self) -> &mut D {
        &mut self.documents
    }

    /// Les captations.
    pub fn captations(&self) -> &C {
        &self.captations
    }

    /// Les documents et les captations, rendus : les services sont fermés.
    pub fn en_parties(self) -> (D, C) {
        (self.documents, self.captations)
    }

    /// Noter une idée, sur l'appareil, à l'heure du contexte. En lecture seule, le refus de
    /// `lecture_seule()`, tel quel, sans rien tirer. Sans fonds, la naissance : l'ULID du
    /// document, puis celui de la lignée (dix octets chacun), l'identifiant de propriétaire
    /// (seize), l'ULID de l'idée (dix), dans cet ordre ; puis `Documents::faire_naitre`. Avec un
    /// fonds, l'ULID de l'idée, puis `Documents::appliquer`. Un texte vide, une fois ses blancs
    /// ôtés : `Raison::Impossible`, et rien ne s'écrit. Ne touche pas au dernier geste de tri.
    pub fn noter(&mut self, commande: &Noter, contexte: &mut Contexte<'_>) -> Reponse<Notee> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let naissance = self
            .documents
            .fonds()
            .is_none()
            .then(|| naissance_tiree(contexte));
        let idee = ulid_tire(contexte);
        let geste = gestes::noter(
            &idee,
            &commande.texte,
            self.appareil,
            contexte.heure,
            commande.decalage_minutes,
        )
        .map_err(refus_impossible)?;
        let avertissement = match naissance {
            Some(naissance) => self.documents.faire_naitre(&naissance, &geste)?,
            None => self.documents.appliquer(&geste)?,
        };
        Ok(Notee {
            idee,
            avertissement,
        })
    }

    /// Compléter le texte de l'idée `commande.idee` (`gestes::completer`) : un geste de tri.
    pub fn completer(
        &mut self,
        commande: &Completer,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        self.trier(&commande.idee, contexte.heure, |fonds, heure| {
            let idee = idee_lue(fonds, &commande.idee)?;
            gestes::completer(&idee, &commande.texte, heure, commande.decalage_minutes)
        })
    }

    /// Poser l'étiquette `commande.etiquette` sur l'idée `commande.idee`
    /// (`gestes::poser_une_etiquette`) : un geste de tri.
    pub fn poser_une_etiquette(
        &mut self,
        commande: &Etiqueter,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        self.trier(&commande.idee, contexte.heure, |fonds, heure| {
            let idee = idee_lue(fonds, &commande.idee)?;
            gestes::poser_une_etiquette(
                &idee,
                &commande.etiquette,
                heure,
                commande.decalage_minutes,
            )
        })
    }

    /// Ôter l'étiquette `commande.etiquette` de l'idée `commande.idee`
    /// (`gestes::oter_une_etiquette`) : un geste de tri.
    pub fn oter_une_etiquette(
        &mut self,
        commande: &Etiqueter,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        self.trier(&commande.idee, contexte.heure, |fonds, heure| {
            let idee = idee_lue(fonds, &commande.idee)?;
            gestes::oter_une_etiquette(&idee, &commande.etiquette, heure, commande.decalage_minutes)
        })
    }

    /// Relier les idées `commande.a` et `commande.b` (`gestes::relier`), lues dans cet ordre,
    /// chacune datée à son décalage (`decalage_minutes`, `decalage_b_minutes`) : un geste de tri.
    pub fn relier(&mut self, commande: &Relier, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        self.trier(&commande.a, contexte.heure, |fonds, heure| {
            let a = idee_lue(fonds, &commande.a)?;
            let b = idee_lue(fonds, &commande.b)?;
            gestes::relier(
                &a,
                &b,
                heure,
                commande.decalage_minutes,
                commande.decalage_b_minutes,
            )
        })
    }

    /// Délier les idées `commande.a` et `commande.b` (`gestes::delier`), lues dans cet ordre,
    /// chacune datée à son décalage, comme `relier` : un geste de tri.
    pub fn delier(&mut self, commande: &Relier, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        self.trier(&commande.a, contexte.heure, |fonds, heure| {
            let a = idee_lue(fonds, &commande.a)?;
            let b = idee_lue(fonds, &commande.b)?;
            gestes::delier(
                &a,
                &b,
                heure,
                commande.decalage_minutes,
                commande.decalage_b_minutes,
            )
        })
    }

    /// Regrouper la captation `commande.captation` dans l'idée `commande.cible`
    /// (`gestes::regrouper`) : un geste de tri. La cible se lit d'abord ; puis la captation, qui
    /// ne se lit pas : `Impossible::CaptationAbsente` ; puis sa source, l'idée que la captation
    /// nomme, jamais une autre, et le compte des captations de la source, elle comprise. La cible
    /// se date à `decalage_minutes`, la source retirée à `decalage_source_minutes`.
    pub fn regrouper(
        &mut self,
        commande: &Regrouper,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        self.trier(&commande.cible, contexte.heure, |fonds, heure| {
            let cible = idee_lue(fonds, &commande.cible)?;
            let empreinte = &commande.captation.empreinte;
            let captation = lecture::lire_la_captation(
                fonds,
                sorte_du_noyau(&commande.captation.sorte),
                empreinte,
            )
            .ok_or_else(|| Impossible::CaptationAbsente {
                empreinte: empreinte.clone(),
            })?;
            let source = idee_lue(fonds, &captation.idee)?;
            let captations_de_la_source = lecture::captations_de(fonds, &source.id).len();
            gestes::regrouper(
                &captation,
                &source,
                captations_de_la_source,
                &cible,
                heure,
                commande.decalage_minutes,
                commande.decalage_source_minutes,
            )
        })
    }

    /// Retirer l'idée `commande.idee` (`gestes::retirer`) : un geste de tri.
    pub fn retirer(&mut self, commande: &Viser, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        self.trier(&commande.idee, contexte.heure, |fonds, heure| {
            let idee = idee_lue(fonds, &commande.idee)?;
            gestes::retirer(&idee, heure, commande.decalage_minutes)
        })
    }

    /// Rétablir l'idée retirée `commande.idee` (`gestes::retablir`) : un geste de tri.
    pub fn retablir(&mut self, commande: &Viser, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        self.trier(&commande.idee, contexte.heure, |fonds, heure| {
            let idee = idee_lue(fonds, &commande.idee)?;
            gestes::retablir(&idee, heure, commande.decalage_minutes)
        })
    }

    /// Annuler le dernier geste de tri fait par ces services (§ 30 · 11) : son inverse, sous
    /// « Annulé : » suivi de son message, à l'heure du contexte (`gestes::annulation`), par
    /// `Documents::appliquer`. En lecture seule, le refus de `lecture_seule()`, tel quel, avant
    /// tout autre refus. Faite, l'annulation vide le dernier geste de tri : elle ne s'annule pas ;
    /// refusée par l'adaptateur, elle le garde, et se refait. Sans dernier geste de tri, depuis
    /// l'ouverture des services ou depuis la dernière annulation : `Raison::RienAAnnuler`,
    /// « Aucun geste de tri à annuler depuis l'ouverture. », et rien ne s'écrit. Ne tire rien de
    /// l'aléa. Le refus d'une annulation après une synchronisation qui a changé l'élément
    /// (§ 30 · 11) vient au lot 3.
    pub fn annuler(&mut self, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let geste = self.dernier_geste.as_ref().ok_or_else(rien_a_annuler)?;
        let avertissement = self
            .documents
            .appliquer(&gestes::annulation(geste, contexte.heure))?;
        self.dernier_geste = None;
        Ok(Faite { avertissement })
    }

    /// Enregistrer la requête `commande.requete` sous le nom `commande.nom`
    /// (`noyau::donnees::requete::enregistrer_une_requete`), sous un ULID tiré à l'heure du
    /// contexte, après les refus de la lecture seule et du fonds non né (voir `Services`).
    pub fn enregistrer_une_requete(
        &mut self,
        commande: &Enregistrer,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        if self.documents.fonds().is_none() {
            return Err(fonds_non_ne());
        }
        let id = ulid_tire(contexte);
        let operation = requete::enregistrer_une_requete(
            &id,
            &commande.nom,
            &requete_du_noyau(&commande.requete),
            contexte.heure,
        )
        .map_err(refus_impossible)?;
        let avertissement = self.documents.appliquer(&operation)?;
        Ok(Faite { avertissement })
    }

    /// Renommer `commande.nom` la requête enregistrée `commande.requete`
    /// (`noyau::donnees::requete::renommer_une_requete`) : une opération sur une requête
    /// enregistrée (voir `Services`).
    pub fn renommer_une_requete(
        &mut self,
        commande: &Renommer,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        let heure = contexte.heure;
        self.sur_une_requete(&commande.requete, |lue| {
            requete::renommer_une_requete(lue, &commande.nom, heure)
        })
    }

    /// Supprimer la requête enregistrée `commande.requete`, sans confirmation (question 19 ;
    /// `noyau::donnees::requete::supprimer_une_requete`) : une opération sur une requête
    /// enregistrée (voir `Services`).
    pub fn supprimer_une_requete(
        &mut self,
        commande: &Supprimer,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        let heure = contexte.heure;
        self.sur_une_requete(&commande.requete, |lue| {
            Ok(requete::supprimer_une_requete(lue, heure))
        })
    }

    /// Verse `octets` par le contrat `Captations` (toujours, même devant un fonds en lecture
    /// seule), puis la fiche, sous une idée qui naît avec elle (son ULID tiré) ; sans fonds, la
    /// naissance d'abord, comme `noter`. En lecture seule, la captation versée, le refus de
    /// `lecture_seule()`, tel quel, sans rien tirer ; une fiche déjà versée, ou que le noyau
    /// refuse (`gestes::verser`) : `Raison::Impossible`, et rien ne s'écrit dans le fonds. Pas
    /// encore une commande : l'import la recevra. Le décalage, celui du message (« Idée du … :
    /// prise versée »), passe tel quel à `gestes::verser` ; l'import le recevra de sa commande.
    /// Ne touche pas au dernier geste de tri.
    pub fn verser_une_fiche(
        &mut self,
        sorte: gestes::Sorte,
        octets: &[u8],
        fiche: &gestes::Fiche,
        provenance: &gestes::Provenance,
        decalage_minutes: i32,
        contexte: &mut Contexte<'_>,
    ) -> Result<Versee, Refus> {
        let empreinte = self.captations.verser(octets)?;
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let naissance = self
            .documents
            .fonds()
            .is_none()
            .then(|| naissance_tiree(contexte));
        let idee = ulid_tire(contexte);
        let existante = self
            .documents
            .fonds()
            .and_then(|fonds| lecture::lire_la_captation(fonds, sorte, &empreinte));
        let operation = gestes::verser(
            sorte,
            &empreinte,
            fiche,
            &idee,
            provenance,
            existante.as_ref(),
            contexte.heure,
            decalage_minutes,
        )
        .map_err(refus_impossible)?;
        let avertissement = match naissance {
            Some(naissance) => self.documents.faire_naitre(&naissance, &operation)?,
            None => self.documents.appliquer(&operation)?,
        };
        Ok(Versee {
            empreinte,
            idee,
            avertissement,
        })
    }

    /// Les idées non retirées, dans l'ordre de leurs identifiants, en lecture seule comme
    /// ailleurs ; sans fonds, aucune.
    pub fn idees(&self) -> Reponse<Vec<String>> {
        Ok(self
            .documents
            .fonds()
            .map(lecture::idees)
            .unwrap_or_default()
            .into_iter()
            .filter(|idee| idee.retiree.is_none())
            .map(|idee| idee.id)
            .collect())
    }

    /// La vue de l'idée `idee`, telle qu'elle se lit, en lecture seule comme ailleurs : sa
    /// provenance, sa date, son texte, ses étiquettes, l'autre bout de chacun de ses liens
    /// (§ 30 · 9.3), dans l'ordre de leurs octets, ses captations, son état de reliée
    /// (§ 30 · 9.1), sa durée (§ 30 · 9.2) et la date de son retrait. Les bouts de chaque lien,
    /// l'état de reliée et la durée, des seules prises, se tirent des règles du noyau
    /// (`noyau::donnees::idee`). Sans fonds, ou sans idée qui se lise sous cet identifiant :
    /// `Raison::Impossible`.
    pub fn vue_d_une_idee(&self, idee: &str) -> Reponse<VueDUneIdee> {
        let fonds = self.documents.fonds().ok_or_else(|| idee_absente(idee))?;
        let lue = lecture::lire_l_idee(fonds, idee).ok_or_else(|| idee_absente(idee))?;
        let captations = lecture::captations_de(fonds, idee);
        let duree_ms = duree_des_prises(&captations);
        let mut liens: Vec<String> = lue
            .liens
            .iter()
            .filter_map(|cle| noyau::donnees::idee::bouts_du_lien(cle.as_str()))
            .map(|(a, b)| if a == lue.id { b } else { a })
            .map(str::to_string)
            .collect();
        liens.sort();
        let reliee = noyau::donnees::idee::reliee(&lue, |autre| {
            lecture::lire_l_idee(fonds, autre).is_some_and(|bout| bout.retiree.is_some())
        });
        Ok(VueDUneIdee {
            idee: lue.id,
            provenance: provenance_des_commandes(lue.provenance),
            le: lue.le,
            texte: lue.texte,
            etiquettes: lue.etiquettes.into_iter().collect(),
            liens,
            captations: captations
                .into_iter()
                .map(|captation| Captation {
                    sorte: sorte_des_commandes(captation.sorte),
                    empreinte: captation.empreinte,
                })
                .collect(),
            reliee,
            duree_ms,
            retiree: lue.retiree,
        })
    }

    /// Le journal du fonds : le message et l'heure de chaque modification, dans l'ordre de
    /// l'historique, en lecture seule comme ailleurs ; sans fonds, aucune entrée.
    pub fn journal(&self) -> Reponse<Vec<EntreeDuJournal>> {
        Ok(self
            .documents
            .fonds()
            .map(lecture::journal)
            .unwrap_or_default()
            .into_iter()
            .map(|(message, heure)| EntreeDuJournal { message, heure })
            .collect())
    }

    /// Le journal du fonds en fenêtre (question 29) : ses entrées (`lecture::journal`), leur
    /// nombre, leur empreinte (`noyau::donnees::journal::empreinte_du_journal`), puis la fenêtre
    /// de leurs lignes, rangées la plus récente d'abord
    /// (`noyau::donnees::journal::ranger_le_journal`), `nombre` borné à `FENETRE_MAX`. Sans
    /// fonds : aucune entrée, et l'empreinte du journal vide. En lecture seule comme ailleurs.
    pub fn journal_en_fenetre(&self, commande: &Fenetre) -> Reponse<VueDuJournal> {
        let entrees = self
            .documents
            .fonds()
            .map(lecture::journal)
            .unwrap_or_default();
        let total = compte(entrees.len());
        let empreinte = journal::empreinte_du_journal(&entrees);
        let lignes = dans_la_fenetre(
            journal::ranger_le_journal(entrees),
            commande.debut,
            commande.nombre,
        )
        .map(|(message, heure)| EntreeDuJournal { message, heure })
        .collect();
        Ok(VueDuJournal {
            total,
            empreinte,
            lignes,
        })
    }

    /// La boîte, réponse de la requête en cours (question 21), par l'index tenu (voir
    /// `Services`) : la réponse de la requête de `commande`, sa date jugée sur ses bornes, dans son
    /// ordre (`Index::repondre`), et les étiquettes du fonds (`Index::etiquettes`), puis la
    /// fenêtre (`nombre` borné à `FENETRE_MAX`), chaque texte borné à `LONGUEUR_D_UNE_LIGNE`
    /// caractères. Sans fonds : aucune ligne, les comptes à 0, aucune étiquette. En lecture seule
    /// comme ailleurs.
    pub fn boite(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite> {
        let Some(fonds) = self.documents.fonds() else {
            return Ok(vue_de_la_boite(&Index::construire(Vec::new(), 0), commande));
        };
        let tetes = fonds.get_heads();
        let mut tenu = self.index.borrow_mut();
        if tenu.as_ref().is_some_and(|(gardees, _)| *gardees != tetes) {
            *tenu = None;
        }
        let (_, l_index) = tenu.get_or_insert_with(|| (tetes, index_du_fonds(fonds)));
        Ok(vue_de_la_boite(l_index, commande))
    }

    /// La même réponse que `boite`, d'un index neuf, bâti à l'appel du fonds lu, sans l'index
    /// tenu : « l'évaluation sans index » de la question 13 se lit ainsi, l'index tenu n'étant
    /// jamais mis à jour par morceaux, mais rebâti en entier (question 14) ; un défaut de
    /// `Index::construire` ou d'`Index::repondre` se voit aux tests du noyau. Sans fonds, un index
    /// vide. Ne touche pas à l'index tenu.
    pub fn boite_sans_index(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite> {
        let l_index = self
            .documents
            .fonds()
            .map_or_else(|| Index::construire(Vec::new(), 0), index_du_fonds);
        Ok(vue_de_la_boite(&l_index, commande))
    }

    /// Les requêtes enregistrées du fonds, chacune qui a un nom (`lecture::requetes_enregistrees`),
    /// rangées par nom replié, puis par nom, puis par ULID
    /// (`noyau::donnees::requete::ranger_les_requetes`, question 18), chaque requête aux types des
    /// commandes, `None` si elle vient d'une version plus récente de Pupitre (question 17). En
    /// lecture seule comme ailleurs ; sans fonds, aucune.
    pub fn requetes_enregistrees(&self) -> Reponse<Vec<RequeteEnregistree>> {
        let mut lues = self
            .documents
            .fonds()
            .map(lecture::requetes_enregistrees)
            .unwrap_or_default();
        requete::ranger_les_requetes(&mut lues);
        Ok(lues
            .into_iter()
            .map(|lue| RequeteEnregistree {
                id: lue.id,
                nom: lue.nom,
                requete: lue.requete.as_ref().map(requete_des_commandes),
            })
            .collect())
    }

    /// Les idées retirées, rangées par `noyau::donnees::boite::retirees`, la plus récemment
    /// retirée d'abord, leur nombre, et la fenêtre de leurs lignes, comme celle de la boîte. Sans
    /// fonds : aucune. En lecture seule comme ailleurs.
    pub fn retirees(&self, commande: &Fenetre) -> Reponse<VueDesRetirees> {
        let (idees, _) = self.documents.fonds().map(en_boite).unwrap_or_default();
        let rangees = boite::retirees(idees);
        Ok(VueDesRetirees {
            total: compte(rangees.len()),
            lignes: fenetre(rangees, commande.debut, commande.nombre),
        })
    }

    /// L'état du fonds : né ou non (`documents.fonds().is_some()`), la lecture seule
    /// (`documents.lecture_seule()`), ses avertissements (`documents.avertissements()`), et le
    /// message de l'opération du dernier geste de tri, celui qu'annuler défait. En lecture seule
    /// comme ailleurs.
    pub fn etat_du_fonds(&self) -> Reponse<EtatDuFonds> {
        Ok(EtatDuFonds {
            ne: self.documents.fonds().is_some(),
            lecture_seule: self.documents.lecture_seule(),
            avertissements: self.documents.avertissements(),
            a_annuler: self
                .dernier_geste
                .as_ref()
                .map(|geste| geste.operation.message.clone()),
        })
    }

    /// Un geste de tri, conduit (voir `Services`) : `idee`, l'identifiant que vise la commande,
    /// nomme le refus d'un geste sans fonds ; `calculer` lit le fonds et demande le geste au noyau,
    /// à l'heure `heure`.
    fn trier(
        &mut self,
        idee: &str,
        heure: i64,
        calculer: impl FnOnce(&Automerge, i64) -> Result<GesteDeTri, Impossible>,
    ) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let fonds = self.documents.fonds().ok_or_else(|| idee_absente(idee))?;
        let geste = calculer(fonds, heure).map_err(refus_impossible)?;
        let avertissement = self.documents.appliquer(&geste.operation)?;
        self.dernier_geste = Some(geste);
        Ok(Faite { avertissement })
    }

    /// Une opération sur la requête enregistrée `id`, conduite (voir `Services`) : `calculer`
    /// demande l'opération au noyau, de la requête lue. Ne touche pas au dernier geste de tri.
    fn sur_une_requete(
        &mut self,
        id: &str,
        calculer: impl FnOnce(&requete::RequeteEnregistreeLue) -> Result<Operation, Impossible>,
    ) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let fonds = self.documents.fonds().ok_or_else(fonds_non_ne)?;
        let lue = lecture::requetes_enregistrees(fonds)
            .into_iter()
            .find(|lue| lue.id == id)
            .ok_or_else(|| {
                refus_impossible(Impossible::RequeteAbsente {
                    requete: id.to_string(),
                })
            })?;
        let operation = calculer(&lue).map_err(refus_impossible)?;
        let avertissement = self.documents.appliquer(&operation)?;
        Ok(Faite { avertissement })
    }
}

/// L'idée `idee`, telle qu'elle se lit dans `fonds` ; aucune : `Impossible::IdeeAbsente`
/// (`gestes::presente`).
fn idee_lue(fonds: &Automerge, idee: &str) -> Result<IdeeLue, Impossible> {
    gestes::presente(idee, lecture::lire_l_idee(fonds, idee))
}

/// Chaque idée de `fonds` qui se lit, retirée comprise, telle que l'index du noyau la reçoit, et
/// le nombre de celles qui ne se lisent pas : les idées par `lecture::idees_et_illisibles`, leurs
/// captations par `lecture::captations_par_idee`, leurs photos par
/// `lecture::idees_avec_une_photo`, chacune lue une fois ; les contenus par
/// `noyau::donnees::boite::contenus_de`, la durée par `duree_des_prises`, et l'état de reliée par
/// `noyau::donnees::idee::reliee`, l'autre bout de chaque lien dit retiré s'il est l'une des idées
/// retirées lues, si bien qu'un bout illisible compte non retiré, comme dans
/// `Services::vue_d_une_idee`.
fn en_boite(fonds: &Automerge) -> (Vec<EnBoite>, usize) {
    let (idees, illisibles) = lecture::idees_et_illisibles(fonds);
    let mut captations = lecture::captations_par_idee(fonds);
    let photos = lecture::idees_avec_une_photo(fonds);
    let retirees: BTreeSet<String> = idees
        .iter()
        .filter(|idee| idee.retiree.is_some())
        .map(|idee| idee.id.clone())
        .collect();
    let lues = idees
        .into_iter()
        .map(|idee| {
            let siennes = captations.remove(&idee.id).unwrap_or_default();
            EnBoite {
                contenus: boite::contenus_de(
                    &idee,
                    siennes.iter().map(|captation| captation.sorte),
                    photos.contains(&idee.id),
                ),
                duree_ms: duree_des_prises(&siennes),
                reliee: noyau::donnees::idee::reliee(&idee, |autre| retirees.contains(autre)),
                idee,
            }
        })
        .collect();
    (lues, illisibles)
}

/// L'index des idées interrogées de `fonds`, bâti de ses idées lues (`en_boite`), les retirées
/// écartées par `Index::construire`.
fn index_du_fonds(fonds: &Automerge) -> Index {
    let (idees, illisibles) = en_boite(fonds);
    Index::construire(idees, illisibles)
}

/// La vue de la boîte qui répond à `commande` sur `l_index` : la réponse de sa requête
/// (`index::Reponse`), sa date jugée sur ses bornes, les trouvées dans son ordre ; ses comptes,
/// ses deux assiettes, chacune avec les illisibles de la réponse, et ses parts « sans … », chaque
/// compte par `compte` ; les étiquettes du fonds ; puis la fenêtre des trouvées. `boite` et
/// `boite_sans_index` la partagent.
fn vue_de_la_boite(l_index: &Index, commande: &commandes::Boite) -> VueDeLaBoite {
    let reponse: index::Reponse = l_index.repondre(
        &requete_du_noyau(&commande.requete),
        bornes_du_noyau(&commande.bornes),
        ordre_du_noyau(&commande.ordre),
    );
    VueDeLaBoite {
        non_reliees: compte(reponse.non_reliees),
        assiette_du_compte: assiette_des_commandes(reponse.assiette_du_compte, reponse.illisibles),
        retenues: compte(reponse.trouvees.len()),
        assiette_de_la_requete: assiette_des_commandes(reponse.assiette, reponse.illisibles),
        sans: sans_des_commandes(reponse.sans),
        etiquettes_du_fonds: l_index.etiquettes(),
        lignes: fenetre(reponse.trouvees, commande.debut, commande.nombre),
    }
}

/// La durée d'une idée, de ses captations : celle de ses seules prises, par
/// `noyau::donnees::idee::duree` (§ 30 · 9.2) ; un MIDI versé ou un texte versé n'y entre pas. La
/// vue d'une idée et la boîte la tirent toutes deux d'ici.
fn duree_des_prises(captations: &[CaptationLue]) -> Option<u64> {
    let prises: Vec<Option<u64>> = captations
        .iter()
        .filter(|captation| captation.sorte == gestes::Sorte::Prise)
        .map(|captation| captation.duree_ms)
        .collect();
    noyau::donnees::idee::duree(&prises)
}

/// Les éléments de la fenêtre qui commence au rang `debut` de `rangees` et en prend `nombre`,
/// borné à `FENETRE_MAX` ; une fenêtre qui commence après le dernier n'en rend aucun. Les lignes
/// de la boîte, des idées retirées et du journal se prennent ainsi.
fn dans_la_fenetre<T>(rangees: Vec<T>, debut: u32, nombre: u32) -> impl Iterator<Item = T> {
    let debut = usize::try_from(debut).unwrap_or(usize::MAX);
    let nombre = usize::try_from(nombre.min(FENETRE_MAX)).unwrap_or(usize::MAX);
    rangees.into_iter().skip(debut).take(nombre)
}

/// Les lignes de la fenêtre qui commence au rang `debut` de `rangees` et en prend `nombre`, borné
/// à `FENETRE_MAX` (`dans_la_fenetre`).
fn fenetre(rangees: Vec<EnBoite>, debut: u32, nombre: u32) -> Vec<LigneDeLaBoite> {
    dans_la_fenetre(rangees, debut, nombre).map(ligne).collect()
}

/// La ligne d'une idée de la boîte : ce qu'elle montre, son texte borné à `LONGUEUR_D_UNE_LIGNE`
/// caractères, ses étiquettes dans l'ordre de leurs octets.
fn ligne(dans_la_boite: EnBoite) -> LigneDeLaBoite {
    let idee = dans_la_boite.idee;
    LigneDeLaBoite {
        idee: idee.id,
        le: idee.le,
        texte: idee
            .texte
            .map(|texte| texte.chars().take(LONGUEUR_D_UNE_LIGNE).collect()),
        etiquettes: idee.etiquettes.into_iter().collect(),
        duree_ms: dans_la_boite.duree_ms,
        reliee: dans_la_boite.reliee,
        provenance: provenance_des_commandes(idee.provenance),
        retiree: idee.retiree,
    }
}

/// Un compte d'idées, en entier d'une réponse ; au-delà d'`u64`, que n'atteint aucun compte,
/// `u64::MAX`.
fn compte(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

/// L'assiette des commandes qui répond à celle du noyau, avec ses `illisibles`, que le noyau
/// porte à côté de l'assiette, chaque champ par `compte`.
fn assiette_des_commandes(
    assiette: noyau::donnees::idee::Assiette,
    illisibles: usize,
) -> commandes::Assiette {
    commandes::Assiette {
        considerees: compte(assiette.considerees),
        portant: compte(assiette.portant),
        decidees: compte(assiette.decidees),
        illisibles: compte(illisibles),
    }
}

/// Les parts « sans … » des commandes qui répondent à celles du noyau, chacune par `compte`.
fn sans_des_commandes(sans: requete::Sans) -> commandes::Sans {
    commandes::Sans {
        etiquette: sans.etiquette.map(compte),
        duree: sans.duree.map(compte),
        texte: sans.texte.map(compte),
    }
}

/// La requête du noyau qui répond à celle des commandes, champ pour champ.
fn requete_du_noyau(demandee: &commandes::Requete) -> requete::Requete {
    let jour = |jour: &commandes::Jour| requete::Jour {
        annee: jour.annee,
        mois: jour.mois,
        jour: jour.jour,
    };
    requete::Requete {
        texte: demandee.texte.clone(),
        du: demandee.du.as_ref().map(jour),
        au: demandee.au.as_ref().map(jour),
        etiquette: demandee.etiquette.clone(),
        plus_de_secondes: demandee.plus_de_secondes,
        etat: demandee.etat.as_ref().map(|etat| match etat {
            commandes::Etat::Reliee => requete::Etat::Reliee,
            commandes::Etat::NonReliee => requete::Etat::NonReliee,
        }),
        provenance: demandee
            .provenance
            .as_ref()
            .map(|provenance| match provenance {
                commandes::ProvenanceDeLIdee::Captee => requete::ProvenanceDeLIdee::Captee,
                commandes::ProvenanceDeLIdee::Importee => requete::ProvenanceDeLIdee::Importee,
            }),
        contenu: demandee.contenu.as_ref().map(contenu_du_noyau),
    }
}

/// La requête des commandes qui répond à celle du noyau, champ pour champ : l'inverse de
/// `requete_du_noyau`.
fn requete_des_commandes(lue: &requete::Requete) -> commandes::Requete {
    let jour = |jour: &requete::Jour| commandes::Jour {
        annee: jour.annee,
        mois: jour.mois,
        jour: jour.jour,
    };
    commandes::Requete {
        texte: lue.texte.clone(),
        du: lue.du.as_ref().map(jour),
        au: lue.au.as_ref().map(jour),
        etiquette: lue.etiquette.clone(),
        plus_de_secondes: lue.plus_de_secondes,
        etat: lue.etat.map(|etat| match etat {
            requete::Etat::Reliee => commandes::Etat::Reliee,
            requete::Etat::NonReliee => commandes::Etat::NonReliee,
        }),
        provenance: lue.provenance.map(|provenance| match provenance {
            requete::ProvenanceDeLIdee::Captee => commandes::ProvenanceDeLIdee::Captee,
            requete::ProvenanceDeLIdee::Importee => commandes::ProvenanceDeLIdee::Importee,
        }),
        contenu: lue.contenu.map(contenu_des_commandes),
    }
}

/// Les bornes du noyau qui répondent à celles des commandes.
fn bornes_du_noyau(bornes: &commandes::Bornes) -> requete::Bornes {
    requete::Bornes {
        des: bornes.des,
        avant: bornes.avant,
    }
}

/// L'ordre du noyau qui répond à celui des commandes.
fn ordre_du_noyau(ordre: &commandes::Ordre) -> boite::Ordre {
    match ordre {
        commandes::Ordre::Date => boite::Ordre::Date,
        commandes::Ordre::Etiquette => boite::Ordre::Etiquette,
        commandes::Ordre::Duree => boite::Ordre::Duree,
        commandes::Ordre::Reliee => boite::Ordre::Reliee,
        commandes::Ordre::Provenance => boite::Ordre::Provenance,
    }
}

/// Le contenu du noyau qui répond à celui des commandes.
fn contenu_du_noyau(contenu: &commandes::Contenu) -> boite::Contenu {
    match contenu {
        commandes::Contenu::TexteDeLIdee => boite::Contenu::TexteDeLIdee,
        commandes::Contenu::Prise => boite::Contenu::Prise,
        commandes::Contenu::Photo => boite::Contenu::Photo,
        commandes::Contenu::MidiVerse => boite::Contenu::MidiVerse,
        commandes::Contenu::TexteVerse => boite::Contenu::TexteVerse,
    }
}

/// Le contenu des commandes qui répond à celui du noyau.
fn contenu_des_commandes(contenu: boite::Contenu) -> commandes::Contenu {
    match contenu {
        boite::Contenu::TexteDeLIdee => commandes::Contenu::TexteDeLIdee,
        boite::Contenu::Prise => commandes::Contenu::Prise,
        boite::Contenu::Photo => commandes::Contenu::Photo,
        boite::Contenu::MidiVerse => commandes::Contenu::MidiVerse,
        boite::Contenu::TexteVerse => commandes::Contenu::TexteVerse,
    }
}

/// La naissance d'un fonds, à l'heure du contexte : l'ULID du document, celui de la lignée, puis
/// l'identifiant de propriétaire, tirés dans cet ordre.
fn naissance_tiree(contexte: &mut Contexte<'_>) -> Operation {
    let identifiant = ulid_tire(contexte);
    let lignee = ulid_tire(contexte);
    let proprietaire = identifiant_de_proprietaire(contexte.alea);
    gestes::naissance(&identifiant, &lignee, &proprietaire, contexte.heure)
}

/// Un ULID à l'heure du contexte, de dix octets d'aléa (§ 30 · 2).
fn ulid_tire(contexte: &mut Contexte<'_>) -> String {
    let mut alea = [0u8; 10];
    contexte.alea.remplir(&mut alea);
    ulid(contexte.heure, alea)
}

/// La seule fonction qui forme l'identifiant de propriétaire (§ 30 · 2) : seize octets d'aléa,
/// puis `uuid_v4`.
fn identifiant_de_proprietaire(alea: &mut dyn Alea) -> String {
    let mut octets = [0u8; 16];
    alea.remplir(&mut octets);
    uuid_v4(octets)
}

/// Le refus d'une idée qu'aucun fonds ne porte, ou qui ne se lit pas : le constat
/// `Impossible::IdeeAbsente` du noyau, en `Raison::Impossible`.
fn idee_absente(idee: &str) -> Refus {
    refus_impossible(Impossible::IdeeAbsente {
        idee: idee.to_string(),
    })
}

/// Le refus d'une opération sur une requête enregistrée sans fonds, qui n'en fait rien naître
/// (question 20) : `Raison::Impossible`, du texte que rendent les deux adaptateurs à
/// `Documents::appliquer` sans fonds, `FONDS_NON_NE`.
fn fonds_non_ne() -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: FONDS_NON_NE.to_string(),
    }
}

/// Le refus d'« annuler » sans dernier geste de tri : `Raison::RienAAnnuler`.
fn rien_a_annuler() -> Refus {
    Refus {
        raison: Raison::RienAAnnuler,
        message: "Aucun geste de tri à annuler depuis l'ouverture.".to_string(),
    }
}

/// Le refus d'un geste que le noyau dit impossible : `Raison::Impossible`, son message le
/// constat du noyau.
fn refus_impossible(impossible: Impossible) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: impossible.to_string(),
    }
}

/// L'appareil des commandes qui répond à celui du noyau.
fn appareil_des_commandes(appareil: gestes::Appareil) -> commandes::Appareil {
    match appareil {
        gestes::Appareil::Etabli => commandes::Appareil::Etabli,
        gestes::Appareil::Carnet => commandes::Appareil::Carnet,
    }
}

/// La provenance des commandes qui répond à celle du noyau.
fn provenance_des_commandes(provenance: gestes::Provenance) -> commandes::Provenance {
    match provenance {
        gestes::Provenance::Captee { par } => commandes::Provenance::Captee {
            par: appareil_des_commandes(par),
        },
        gestes::Provenance::Importee { par, depuis } => commandes::Provenance::Importee {
            par: appareil_des_commandes(par),
            depuis,
        },
    }
}

/// La sorte des commandes qui répond à celle du noyau.
fn sorte_des_commandes(sorte: gestes::Sorte) -> commandes::Sorte {
    match sorte {
        gestes::Sorte::Prise => commandes::Sorte::Prise,
        gestes::Sorte::MidiVerseSeul => commandes::Sorte::MidiVerseSeul,
        gestes::Sorte::TexteVerse => commandes::Sorte::TexteVerse,
    }
}

/// La sorte du noyau qui répond à celle des commandes.
fn sorte_du_noyau(sorte: &commandes::Sorte) -> gestes::Sorte {
    match sorte {
        commandes::Sorte::Prise => gestes::Sorte::Prise,
        commandes::Sorte::MidiVerseSeul => gestes::Sorte::MidiVerseSeul,
        commandes::Sorte::TexteVerse => gestes::Sorte::TexteVerse,
    }
}
