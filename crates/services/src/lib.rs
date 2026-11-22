//! Les services de pupitre (§ 40 · 3) : les opérations nommées et les lectures du fonds,
//! conduites sur les contrats `Documents` et `Captations`, que réalisent les adaptateurs, sur
//! fichiers (le stockage) ou en mémoire. `Services` conduit chaque commande : il tire de l'aléa
//! du `Contexte` les identifiants qu'elle forme, demande l'opération au noyau, l'écrit par
//! `Documents`, et rend la réponse aux types de `commandes`. `application` applique les
//! changements que le noyau calcule ; `lecture` tire du document le fonds lu, et l'arbre de sa
//! teneur, que le noyau reçoit ; `mesure` engendre le fonds de mesure (§ 31 · 10, test 13) par
//! les seules opérations de `Services`. Le troisième contrat, la source à verser (`Source`), a
//! sa réalisation en mémoire dans `source_en_memoire`, hors de la fonction `en-memoire` : le
//! générateur du fonds de mesure l'emploie. `etat_de_l_import` tient l'état d'un import : ce que
//! l'inspection a lu de la source, puis l'import ouvert, et les conversions de ses types.
//!
//! La fonction `en-memoire`, que seuls les tests demandent, compile le module `en_memoire`, la
//! paire d'adaptateurs en mémoire.

pub mod application;
mod contrats;
#[cfg(feature = "en-memoire")]
pub mod en_memoire;
mod etat_de_l_import;
pub mod lecture;
pub mod mesure;
pub mod source_en_memoire;

pub use commandes::{Raison, Refus};
pub use contrats::{
    Alea, Captations, Documents, Ecriture, Entree, FONDS_NON_NE, SorteDEntree, Source, Versement,
    l_entree, le_fichier,
};

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use automerge::{Automerge, ChangeHash};
use commandes::{
    Captation, CaptationVue, ChoisirLeType, CloreLImport, Completer, DeclarerUneOrigine,
    ENTIER_SUR, Enregistrer, EntreeDuJournal, EtatDuFonds, Etiqueter, FENETRE_MAX, Faite, Fenetre,
    FichierVerse, ImportClos, Inspection, Issue, LONGUEUR_D_UNE_LIGNE, LigneDeLaBoite,
    LireUnRapport, MidiVu, Notee, Noter, OrigineADeclarer, OuvrirLImport, Portee, RapportDImport,
    Regrouper, Relier, Renommer, Reponse, RequeteEnregistree, StemVu, Supprimer, TexteLu, TypeVu,
    VerserUnFichier, VerserUnePriseEnStems, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees,
    VueDuJournal,
};
use noyau::donnees::boite::{self, EnBoite};
use noyau::donnees::changement::Operation;
use noyau::donnees::demarrage::copies_a_retirer;
use noyau::donnees::entete;
use noyau::donnees::fonds::MIDI_VERSES;
use noyau::donnees::gestes::{self, CaptationLue, GesteDeTri, IdeeLue, Impossible};
use noyau::donnees::identifiants::{ulid, uuid_v4};
use noyau::donnees::import::{self, MidiDuStem, StemAVerser};
use noyau::donnees::index::{self, Index};
use noyau::donnees::journal;
use noyau::donnees::origine;
use noyau::donnees::rapport;
use noyau::donnees::requete;
use noyau::donnees::stems;

use etat_de_l_import::{Donne, EtatDeLImport, FluxControle, InspectionGardee};
use lecture::{ProvenanceDuType, StemLu, TypeLu};

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
    /// La dernière inspection, gardée jusqu'à l'ouverture qui la reprend, ou jusqu'à une autre.
    inspection: Option<InspectionGardee>,
    /// L'import ouvert par ces services ; `None` hors d'un import.
    import: Option<EtatDeLImport>,
    /// Le dernier import clos, son ULID et sa table (chemin → empreinte) de chaque fichier versé
    /// ou dédoublonné, oubliés à l'inspection suivante (R6) : `Portee::Prefixe` se résout sur
    /// elle, à la déclaration d'une origine, s'il nomme cet import.
    dernier_clos: Option<(String, BTreeMap<String, String>)>,
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
            inspection: None,
            import: None,
            dernier_clos: None,
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
    /// refuse (`gestes::verser`) : `Raison::Impossible`, et rien ne s'écrit dans le fonds. La voie
    /// d'une capture, qui verse toujours : l'import a ses propres commandes (`verser_un_fichier`).
    /// Le décalage, celui du message (« Idée du … : prise versée »), passe tel quel à
    /// `gestes::verser`. Ne touche pas au dernier geste de tri.
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
    /// (`noyau::donnees::idee`). Chaque captation vue (`captation_vue`) : ses faits, ses noms vus,
    /// son origine déclarée et ses stems. Sans fonds, ou sans idée qui se lise sous cet
    /// identifiant : `Raison::Impossible`.
    pub fn vue_d_une_idee(&self, idee: &str) -> Reponse<VueDUneIdee> {
        let fonds = self.documents.fonds().ok_or_else(|| idee_absente(idee))?;
        let lue = lecture::lire_l_idee(fonds, idee).ok_or_else(|| idee_absente(idee))?;
        let captations = lecture::captations_de(fonds, idee);
        let duree_ms = duree_des_prises(&captations);
        let captations_vues = captations
            .iter()
            .map(|captation| captation_vue(fonds, captation))
            .collect();
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
            captations_vues,
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
            return Ok(vue_de_la_boite(
                &Index::construire(Vec::new(), 0),
                commande,
                None,
            ));
        };
        let tetes = fonds.get_heads();
        let mut tenu = self.index.borrow_mut();
        if tenu.as_ref().is_some_and(|(gardees, _)| *gardees != tetes) {
            *tenu = None;
        }
        let (_, l_index) = tenu.get_or_insert_with(|| (tetes, index_du_fonds(fonds)));
        Ok(vue_de_la_boite(l_index, commande, Some(fonds)))
    }

    /// La même réponse que `boite`, d'un index neuf, bâti à l'appel du fonds lu, sans l'index
    /// tenu : « l'évaluation sans index » de la question 13 se lit ainsi, l'index tenu n'étant
    /// jamais mis à jour par morceaux, mais rebâti en entier (question 14) ; un défaut de
    /// `Index::construire` ou d'`Index::repondre` se voit aux tests du noyau. Sans fonds, un index
    /// vide. Ne touche pas à l'index tenu.
    pub fn boite_sans_index(&self, commande: &commandes::Boite) -> Reponse<VueDeLaBoite> {
        let fonds = self.documents.fonds();
        let l_index = fonds.map_or_else(|| Index::construire(Vec::new(), 0), index_du_fonds);
        Ok(vue_de_la_boite(&l_index, commande, fonds))
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
        let fonds = self.documents.fonds();
        let (idees, _) = fonds.map(en_boite).unwrap_or_default();
        let rangees = boite::retirees(idees);
        Ok(VueDesRetirees {
            total: compte(rangees.len()),
            lignes: fenetre(rangees, commande.debut, commande.nombre, fonds),
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

    /// Inspecte `source` sans rien écrire : refusée en lecture seule, du refus de `lecture_seule()`.
    /// Chaque fichier lu de `SIGNATURE_AU_PLUS` octets (`signature::reconnaitre`) ; une étiquette
    /// ID3, de quatre de plus à sa suite (`signature::apres_id3`) ; un WAV, de `ENTETE_AU_PLUS`
    /// (`entete::faits_wav`) ; un lien, un ignoré, jamais au-delà
    /// (`etat_de_l_import::lire_les_fichiers`). Une lecture refusée arrête l'inspection et nomme le
    /// fichier, de `Raison::Disque` : « « x.wav » ne se lit pas : <erreur>. », sans nom « Le fichier
    /// n° <rang> ne se lit pas : <erreur>. » (question 38). L'empreinte d'un fichier audio, MIDI ou
    /// texte ne se calcule que si sa taille est dans `Captations::tailles` (question 8) ; connu, si
    /// le fonds le nomme. La taille à verser, ses fichiers, le poids du fichier complet
    /// (`Automerge::save`, 0 sans fonds) et `place_libre` : `rapport::place`, en entiers ; le
    /// manque, son texte (`rapport::ManqueDePlace`) ; la taille à verser et la place libre rendues,
    /// bornées à `ENTIER_SUR` (balayage Q6). Chaque dossier : ses fichiers audio, le jeu proposé
    /// (`stems::jeu_propose`, sur les tailles et les en-têtes lus, sans empreinte ; question 37), la
    /// déclaration reproposée (`lecture::dernieres_declarations`). La reprise (question 2 ; R2) :
    /// l'import ouvert dont au moins un nom de fichier est posé (`lecture::noms_poses_par`), dont
    /// chaque nom de fichier est le chemin d'une entrée de la source, et chaque nom de prise en
    /// stems le préfixe, par segments entiers (`<nom>/`), d'une entrée, ou `nom_du_dossier`
    /// (`etat_de_l_import::reprend`) ; elle rend les déclarations de son ouverture
    /// (`lecture::declarations_de`). Un import ouvert que la source ne reprend pas :
    /// `ouvert_ailleurs` (question 35). L'inspection se garde dans les services, avec la marque de
    /// dernière écriture de chaque entrée et chaque empreinte calculée, jusqu'à l'ouverture ou à
    /// une autre ; elle oublie l'inspection d'avant, l'état d'un import ouvert par ces services
    /// (rechoisir son dossier le reprend), et la table du dernier import clos (R6).
    pub fn inspecter<S: Source>(&mut self, source: &S, place_libre: u64) -> Reponse<Inspection> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        self.inspection = None;
        self.import = None;
        self.dernier_clos = None;
        let tailles = self.captations.tailles()?;
        let fonds = self.documents.fonds();
        let nommees = fonds.map(lecture::empreintes_nommees).unwrap_or_default();
        let fichiers = etat_de_l_import::lire_les_fichiers(source, &tailles, &nommees)?;
        let gardee = InspectionGardee::de(fichiers);
        let manque = gardee
            .place(poids_du_fonds(fonds), place_libre)
            .err()
            .map(|manque| manque.to_string());
        let dernieres = fonds
            .map(lecture::dernieres_declarations)
            .unwrap_or_default();
        let inspection = Inspection {
            reprise: None,
            nom_du_dossier: source.nom_du_dossier().map(str::to_string),
            fichiers: gardee
                .fichiers
                .iter()
                .map(etat_de_l_import::fichier_inspecte)
                .collect(),
            dossiers: etat_de_l_import::dossiers_inspectes(&gardee.fichiers, &dernieres),
            a_verser: gardee.a_verser.min(ENTIER_SUR),
            libre: place_libre.min(ENTIER_SUR),
            manque,
            generateurs: generateurs_du_fonds(fonds),
            declarations_reprises: Vec::new(),
            ouvert_ailleurs: false,
        };
        self.inspection = Some(gardee);
        Ok(inspection)
    }

    /// Ouvre l'import inspecté (question 15) : en lecture seule, le refus ; sans inspection,
    /// « Aucun dossier n'est inspecté. » ; la place, de nouveau comparée, son manque en refus
    /// (`Raison::Impossible`, le texte de `rapport::ManqueDePlace`) ; un dossier déclaré jeu de
    /// stems (`stems`) sans fichier audio, `JeuSansAudio`. Chaque `DeclarationDUnDossier` se
    /// traduit en `origine::Declaration` (`etat_de_l_import::declarations_du_noyau`) : `stems` tel
    /// quel, `Aucune` en `None`, `Main` en `Some(Ajout::Main)`, `Generee { nom }` en
    /// `Some(Ajout::Generee(nom))` ; une déclaration vide (`stems` faux, `Aucune`) n'écrit rien, et
    /// s'ôte avant de comparer les déclarations d'un import repris ; deux entrées du même chemin,
    /// la dernière l'emporte (balayage Q3). Les refus se disent avant toute écriture.
    ///
    /// Un import ouvert que la source ne reprend pas se clôt d'abord (question 35 ;
    /// `clore_un_import_ouvert_ailleurs`). Neuf : l'ULID tiré, puis `import::ouvrir`, par
    /// `faire_naitre` dans un Pupitre neuf (la naissance tirée d'abord, comme `noter`), sinon
    /// `appliquer` ; la tête qui suit, gardée. Repris : rien ne s'écrit ; les déclarations sont
    /// celles de l'ouverture, et une commande qui en porte d'autres est refusée, « Cet import
    /// reprend ses déclarations : elles ne changent pas. » ; la tête de l'ouverture, retrouvée par
    /// l'opération qui a posé `imports/<i>/ouvert` (`lecture::tete_de_l_ouverture`) ; chaque fichier
    /// dont un nom porte l'ULID de l'import, compté de nouveau sans être relu, illisible si sa
    /// captation n'a pas de `format`, ses octets ceux de son entrée (R7), et chaque jeu dont la
    /// prise en stems est nommée, versé (`EtatDeLImport::reprendre`) ; puis ses copies non nommées,
    /// retirées (`copies_a_retirer`, à la reprise). Les empreintes nommées, lues une fois et
    /// gardées dans l'état de l'import (R26). Rend les rangs à verser, hors des jeux, et chaque jeu
    /// avec les siens ; l'avertissement, le premier qu'une écriture a rendu.
    pub fn ouvrir_l_import<S: Source>(
        &mut self,
        _source: &S,
        commande: &OuvrirLImport,
        place_libre: u64,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<commandes::ImportOuvert> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let gardee = self
            .inspection
            .as_ref()
            .ok_or_else(|| refus_de_l_import("Aucun dossier n'est inspecté."))?;
        gardee
            .place(poids_du_fonds(self.documents.fonds()), place_libre)
            .map_err(|manque| refus_de_l_import(&manque.to_string()))?;
        let declarations = etat_de_l_import::declarations_du_noyau(&commande.declarations);
        let jeux =
            etat_de_l_import::jeux_de(&gardee.fichiers, &declarations).map_err(refus_impossible)?;
        let naissance = self
            .documents
            .fonds()
            .is_none()
            .then(|| naissance_tiree(contexte));
        let import = ulid_tire(contexte);
        let ouverture = import::ouvrir(
            &import,
            &declarations,
            contexte.heure,
            commande.decalage_minutes,
        )
        .map_err(refus_impossible)?;
        let avertissement = match naissance {
            Some(naissance) => self.documents.faire_naitre(&naissance, &ouverture)?,
            None => self.documents.appliquer(&ouverture)?,
        };
        let fonds = self.documents.fonds().ok_or_else(fonds_non_ne)?;
        let tete = fonds.get_heads();
        let nommees = lecture::empreintes_nommees(fonds);
        let gardee = self
            .inspection
            .take()
            .ok_or_else(|| refus_de_l_import("Aucun dossier n'est inspecté."))?;
        let etat = EtatDeLImport::nouveau(
            import,
            contexte.heure,
            tete,
            declarations,
            gardee.fichiers,
            jeux,
            nommees,
        );
        let ouvert = commandes::ImportOuvert {
            import: etat.import.clone(),
            repris: false,
            rangs: etat.rangs_hors_des_jeux(),
            jeux: etat.jeux_a_verser(),
            avertissement,
        };
        self.import = Some(etat);
        Ok(ouvert)
    }

    /// Verse le fichier `rang` de l'import ouvert : un autre import, `AucunImportOuvert` ; en
    /// lecture seule, le refus ; déjà compté, `Issue::DejaFait` ; un rang qui n'est pas un fichier
    /// à verser, « Le dossier versé n'a pas de fichier n° <rang> à verser. ». Ses octets ou sa
    /// marque de dernière écriture changés depuis l'inspection (`octets_actuels`,
    /// `marque_actuelle`, lus avant la copie, puis `marque_actuelle` relue après), ses octets
    /// copiés autres que ceux de l'inspection, ou son empreinte autre que celle qu'a calculée
    /// l'inspection (passée en `attendue` à `verser_en_flux`, qui ne range rien en désaccord ;
    /// R20) : `FichierChange` (question 14 ; R8). Une marque de dernière écriture absente ne
    /// compare rien. La copie en flux, sous `import::nom_de_la_copie`, le flux contrôlé (ses octets
    /// comptés, un texte par `ControleDuTexte` : `FluxControle`) ; ses faits, ceux de l'en-tête lu
    /// à l'inspection, ou aucun (illisible : un en-tête qui ne se lit pas, un texte que le contrôle
    /// refuse). Un fichier audio d'un jeu, et le MIDI d'un de ses stems (`stem_du_midi`) : la
    /// captation seule, `Issue::Stem`. Sinon `import::verser_un_fichier`, son origine celle des
    /// déclarations (`origine::declaration_du_fichier`), ce que le fonds en porte lu par
    /// `lecture::deja_lu`, l'ULID de l'idée tiré pour un contenu neuf seulement ; `Issue::Verse` ou
    /// `Issue::Dedoublonne` ; les empreintes nommées de l'état, tenues à jour.
    pub fn verser_un_fichier<S: Source>(
        &mut self,
        source: &S,
        commande: &VerserUnFichier,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<FichierVerse> {
        let mut etat = self.prendre_l_import(&commande.import)?;
        let verse = self.verser_dans_l_import(&mut etat, source, commande, contexte);
        self.import = Some(etat);
        verse
    }

    /// Verse la prise en stems du jeu `dossier`, chacun de ses stems déjà copié (sinon « Les stems
    /// de ce jeu ne sont pas tous versés. ») ; un dossier qui n'est pas un jeu de l'import, « Ce
    /// dossier n'est pas un jeu de stems de cet import. » ; un jeu déjà versé, fait, sans rien
    /// écrire. Le manifeste (`rapport::manifeste`) par `Captations::verser`, puis
    /// `import::verser_une_prise_en_stems`, son nom le chemin du dossier, ou
    /// `Source::nom_du_dossier` pour le dossier versé ; sans nom en UTF-8, aucun, et aucune ligne
    /// `fichier-sans-nom` au rapport, qui ne nomme que le rang d'un fichier (R23) ; son origine,
    /// celle que les déclarations donnent à ses stems. Ce que le fonds porte déjà de la prise se lit
    /// par `lecture::deja_de_la_prise`, les liens `prises/<p>/stem/<s>/midi/<m>` déjà posés
    /// compris : une prise connue, réimportée avec un MIDI nouveau d'un stem, reçoit son lien et
    /// ses faits (revue de la tâche 6). L'ULID de l'idée, tiré pour une prise neuve seulement.
    pub fn verser_une_prise_en_stems<S: Source>(
        &mut self,
        source: &S,
        commande: &VerserUnePriseEnStems,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        let mut etat = self.prendre_l_import(&commande.import)?;
        let faite = self.verser_le_jeu(&mut etat, source, commande, contexte);
        self.import = Some(etat);
        faite
    }

    /// Clôt l'import, chaque fichier et chaque jeu versés (sinon « Des fichiers restent à
    /// verser. ») : le rapport (`rapport::rapport`), les connues lues à la tête de l'ouverture
    /// (`lecture::empreintes_nommees_a`), chaque origine changée par comparaison
    /// d'`origines_a` et d'`origines_de` (`etat_de_l_import::origines_changees`) ; relu
    /// (`rapport::lire_le_rapport`), il donne le rapport rendu, ses octets bornés à `ENTIER_SUR` ;
    /// versé par `Captations::verser` ; `import::clore` ; chaque copie de l'import retirée
    /// (`copies_a_retirer`, à la clôture). L'état de l'import oublié, les services gardent son
    /// ULID et la table (chemin → empreinte) de chaque fichier versé ou dédoublonné de l'import
    /// clos, jusqu'à l'inspection suivante : `Portee::Prefixe` se résout sur elle (R6 ;
    /// `declarer_une_origine`). La source n'est pas relue. Refusé avant d'écrire, l'import reste
    /// ouvert.
    pub fn clore_l_import<S: Source>(
        &mut self,
        _source: &S,
        commande: &CloreLImport,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<ImportClos> {
        let etat = self.prendre_l_import(&commande.import)?;
        let clos = match self.ecrire_la_cloture(&etat, commande, contexte) {
            Ok(clos) => clos,
            Err(refus) => {
                self.import = Some(etat);
                return Err(refus);
            }
        };
        self.dernier_clos = Some((etat.import.clone(), etat.table()));
        self.retirer_les_copies(&etat.import)?;
        Ok(clos)
    }

    /// Déclare une origine après l'import (question 22) : sur la captation de
    /// `Portee::Captation`, que le fonds porte sous sa sorte ; ou sur les captations dont un chemin
    /// de la table du dernier import clos (chemin → empreinte, gardée par `clore_l_import`
    /// jusqu'à l'inspection suivante ; R6) est sous le préfixe de `Portee::Prefixe`
    /// (`origine::sous_le_prefixe`, par segments entiers ; balayage Q13), l'import qu'il nomme
    /// étant celui de la table, sinon « Ce dossier n'est plus inspecté. ». En lecture seule, le
    /// refus de `lecture_seule()`, tel quel ; sans cible, `Impossible::CaptationAbsente`. Chaque
    /// cible, une fois, avec ses origines (`lecture::origines_de`), passe à
    /// `import::declarer_une_origine`, à l'heure du contexte ; la portée du message, la captation
    /// nommée par `captation_nommee`, ou le préfixe suivi d'une barre `/`, qu'il en porte une ou
    /// non (« « Démos/ » »). L'opération, s'il y en a une, par `Documents::appliquer`.
    /// L'avertissement « L'origine reste générée : une origine déclarée ne se retire pas. » dès
    /// qu'une cible générée est gardée (`generees_gardees` > 0), l'opération faite pour les autres
    /// cibles, s'il y en a (V9), suivi de celui de l'écriture, s'il y en a un (`joindre`) ; une
    /// origine déjà portée par chaque cible : `Faite { avertissement: None }`, sans opération
    /// (R16). Ne tire rien de l'aléa ; ne touche pas au dernier geste de tri.
    pub fn declarer_une_origine(
        &mut self,
        commande: &DeclarerUneOrigine,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let (cibles, portee) = self.cibles_de(&commande.portee)?;
        let fonds = self.documents.fonds().ok_or_else(fonds_non_ne)?;
        let origines: Vec<origine::OriginesLues> = cibles
            .keys()
            .map(|empreinte| lecture::origines_de(fonds, empreinte))
            .collect();
        let cibles_du_noyau: Vec<import::Cible<'_>> = cibles
            .iter()
            .zip(&origines)
            .map(|((empreinte, nommee), origines)| import::Cible {
                empreinte: empreinte.as_str(),
                origines,
                nommee: nommee.as_str(),
            })
            .collect();
        let declaree = import::declarer_une_origine(
            &cibles_du_noyau,
            ajout_du_noyau(&commande.origine).as_ref(),
            &portee,
            contexte.heure,
        )
        .map_err(refus_impossible)?;
        let gardee = (declaree.generees_gardees > 0).then(|| ORIGINE_GARDEE.to_string());
        let ecrite = match &declaree.operation {
            Some(operation) => self.documents.appliquer(operation)?,
            None => None,
        };
        Ok(Faite {
            avertissement: joindre(gardee, ecrite),
        })
    }

    /// Choisit le type d'un stem, ou garde celui qui est proposé (question 18) : en lecture seule,
    /// le refus de `lecture_seule()`, tel quel ; le stem `commande.stem` de la prise
    /// `commande.prise`, lu par `lecture::stems_de` (sans fonds, ou sans ce stem sous la prise,
    /// `import::choisir_le_type` le refuse, `Impossible::StemAbsent`) ; la proposition,
    /// `stems::type_propose` de son premier nom vu, celui que montre la vue (`noms_vus`, sur les
    /// noms de son entrée dans la prise), qui le nomme aussi dans le message ; puis
    /// `import::choisir_le_type`, à l'heure du contexte, et `Documents::appliquer`. Ne tire rien
    /// de l'aléa ; ne touche pas au dernier geste de tri.
    pub fn choisir_le_type_d_un_stem(
        &mut self,
        commande: &ChoisirLeType,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let noms = self
            .documents
            .fonds()
            .and_then(|fonds| {
                lecture::stems_de(fonds, &commande.prise)
                    .into_iter()
                    .find(|stem| stem.empreinte == commande.stem)
            })
            .map(|stem| noms_vus(stem.noms));
        let premier = noms
            .as_ref()
            .and_then(|noms| noms.first())
            .map(String::as_str);
        let operation = import::choisir_le_type(
            &commande.prise,
            &commande.stem,
            noms.is_some(),
            stems::type_propose(premier),
            choix_du_noyau(&commande.choix),
            premier,
            contexte.heure,
        )
        .map_err(refus_impossible)?;
        let avertissement = self.documents.appliquer(&operation)?;
        Ok(Faite { avertissement })
    }

    /// Le texte versé `commande`, relu par `Captations::relire`, qui le relit contre son
    /// empreinte, puis décodé par la règle du noyau, `entete::decoder_le_texte`, jamais par un
    /// second décodeur dans les services (balayage Q9) ; illisible, `None`. Une captation d'une
    /// autre sorte, ou que le fonds ne porte pas sous `textes_verses` :
    /// `Impossible::CaptationAbsente`. Une lecture : en lecture seule comme ailleurs.
    pub fn lire_un_texte_verse(&self, commande: &Captation) -> Reponse<TexteLu> {
        let absente = || captation_absente(&commande.empreinte);
        let fonds = self.documents.fonds().ok_or_else(absente)?;
        let versee = commande.sorte == commandes::Sorte::TexteVerse
            && lecture::lire_la_captation(fonds, gestes::Sorte::TexteVerse, &commande.empreinte)
                .is_some();
        if !versee {
            return Err(absente());
        }
        let octets = self.captations.relire(&commande.empreinte)?;
        Ok(TexteLu {
            texte: entete::decoder_le_texte(&octets),
        })
    }

    /// Le rapport de l'import `commande.import`, l'empreinte que porte `imports/<i>/rapport`
    /// (`lecture::imports`), relu par `Captations::relire`, qui le relit contre son empreinte ; un
    /// import que le fonds ne porte pas, ou qui n'est pas clos : « Cet import n'a pas de
    /// rapport. ». Ses comptes se lisent par la règle du noyau, `rapport::lire_le_rapport`, jamais
    /// par un second lecteur de ses lignes dans les services (balayage P14) ; `None` (une autre
    /// version, une ligne manquante ou mal formée) : `Raison::Illisible`, « Ce rapport vient d'une
    /// version plus récente. ». Rendu aux types des commandes, ses octets bornés à `ENTIER_SUR`
    /// (`etat_de_l_import::rapport_des_commandes`). En lecture seule comme ailleurs.
    pub fn lire_un_rapport(&self, commande: &LireUnRapport) -> Reponse<RapportDImport> {
        let empreinte = self
            .documents
            .fonds()
            .and_then(|fonds| {
                lecture::imports(fonds)
                    .into_iter()
                    .find(|lu| lu.import == commande.import)
            })
            .and_then(|lu| lu.rapport)
            .ok_or_else(|| refus_de_l_import("Cet import n'a pas de rapport."))?;
        let octets = self.captations.relire(&empreinte)?;
        let lu = rapport::lire_le_rapport(&octets).ok_or_else(|| Refus {
            raison: Raison::Illisible,
            message: "Ce rapport vient d'une version plus récente.".to_string(),
        })?;
        Ok(etat_de_l_import::rapport_des_commandes(&lu))
    }

    /// Les noms de générateurs du fonds (question 22 : l'écran les offre, sans normaliser), ceux
    /// des origines déclarées et ceux des déclarations des imports, chacun une fois, dans l'ordre
    /// de leurs octets (`generateurs_du_fonds`) ; sans fonds, aucun. En lecture seule comme
    /// ailleurs.
    pub fn generateurs(&self) -> Reponse<Vec<String>> {
        Ok(generateurs_du_fonds(self.documents.fonds()))
    }

    /// Les cibles d'une déclaration d'origine de portée `portee`, chacune une fois, sous son
    /// empreinte, avec ce qui la nomme, et la portée du message (voir `declarer_une_origine`) :
    /// une captation, nommée par `captation_nommee` ; les fichiers de la table du dernier import
    /// clos sous le préfixe, chacun nommé par le premier de ses chemins, entre guillemets. Les
    /// refus : un préfixe d'un autre import que le dernier clos, ou sans lui, « Ce dossier n'est
    /// plus inspecté. » ; sans cible, `Impossible::CaptationAbsente`.
    fn cibles_de(&self, portee: &Portee) -> Result<(BTreeMap<String, String>, String), Refus> {
        let (cibles, dite) = match portee {
            Portee::Captation(captation) => {
                let sorte = sorte_du_noyau(&captation.sorte);
                let empreinte = &captation.empreinte;
                let fonds = self
                    .documents
                    .fonds()
                    .filter(|fonds| lecture::lire_la_captation(fonds, sorte, empreinte).is_some())
                    .ok_or_else(|| captation_absente(empreinte))?;
                let nommee = captation_nommee(fonds, sorte, empreinte);
                let cible = BTreeMap::from([(empreinte.clone(), nommee.clone())]);
                (cible, nommee)
            }
            Portee::Prefixe { import, prefixe } => {
                let (_, table) = self
                    .dernier_clos
                    .as_ref()
                    .filter(|(clos, _)| clos == import)
                    .ok_or_else(|| refus_de_l_import(PLUS_INSPECTE))?;
                let mut cibles = BTreeMap::new();
                for (chemin, empreinte) in table {
                    if origine::sous_le_prefixe(chemin, prefixe) {
                        cibles
                            .entry(empreinte.clone())
                            .or_insert_with(|| format!("« {chemin} »"));
                    }
                }
                let dite = if prefixe.ends_with('/') {
                    format!("« {prefixe} »")
                } else {
                    format!("« {prefixe}/ »")
                };
                (cibles, dite)
            }
        };
        if cibles.is_empty() {
            return Err(captation_absente(""));
        }
        Ok((cibles, dite))
    }

    /// L'état de l'import `import`, pris aux services, qui le reprennent après la commande ; un
    /// autre import, ou aucun : `Impossible::AucunImportOuvert`.
    fn prendre_l_import(&mut self, import: &str) -> Result<EtatDeLImport, Refus> {
        match self.import.take() {
            Some(etat) if etat.import == import => Ok(etat),
            autre => {
                self.import = autre;
                Err(refus_impossible(Impossible::AucunImportOuvert))
            }
        }
    }

    /// Le versement du fichier de `commande` dans l'import `etat` (voir `verser_un_fichier`).
    fn verser_dans_l_import<S: Source>(
        &mut self,
        etat: &mut EtatDeLImport,
        source: &S,
        commande: &VerserUnFichier,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<FichierVerse> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let rang = commande.rang;
        if etat.donnes.contains_key(&rang) {
            return Ok(FichierVerse {
                rang,
                issue: Issue::DejaFait,
                avertissement: None,
            });
        }
        let (fichier, sorte) = etat
            .fichier(rang)
            .and_then(|fichier| fichier.sorte().map(|sorte| (fichier.clone(), sorte)))
            .ok_or_else(|| {
                refus_de_l_import(&format!(
                    "Le dossier versé n'a pas de fichier n° {rang} à verser."
                ))
            })?;
        let change = || {
            refus_impossible(Impossible::FichierChange {
                nom: fichier.chemin.clone(),
                rang,
            })
        };
        if source.octets_actuels(rang)? != fichier.octets {
            return Err(change());
        }
        let mut flux = FluxControle::de(source.lire_en_flux(rang)?, fichier.texte());
        let versement = self.captations.verser_en_flux(
            &import::nom_de_la_copie(&etat.import, rang),
            &mut flux,
            fichier.empreinte.as_deref(),
        )?;
        let lisible = flux.lisible();
        drop(flux);
        if !versement.rangee || versement.octets != fichier.octets {
            return Err(change());
        }
        let faits = fichier.faits_de_la_copie(lisible);
        let illisible = faits.is_none();
        let faits = faits.unwrap_or_default();
        let empreinte = versement.empreinte;
        if etat.jeu_de(rang).is_some() {
            etat.donnes.insert(
                rang,
                Donne::Stem {
                    empreinte,
                    faits,
                    illisible,
                },
            );
            return Ok(FichierVerse {
                rang,
                issue: Issue::Stem,
                avertissement: None,
            });
        }
        let fonds = self.documents.fonds().ok_or_else(fonds_non_ne)?;
        let deja = lecture::deja_lu(fonds, sorte, &empreinte, &etat.nommees);
        let declaree = fichier
            .chemin
            .as_deref()
            .and_then(|chemin| origine::declaration_du_fichier(chemin, &etat.declarations).origine);
        let idee = if deja.captation {
            String::new()
        } else {
            ulid_tire(contexte)
        };
        let ouvert = import::ImportOuvert {
            import: &etat.import,
            ouvert_le: etat.ouvert_le,
            par: self.appareil,
        };
        let operation = import::verser_un_fichier(
            &ouvert,
            sorte,
            &empreinte,
            &faits,
            fichier.chemin.as_deref(),
            declaree.as_ref(),
            &deja,
            &idee,
            contexte.heure,
            commande.decalage_minutes,
        )
        .map_err(refus_impossible)?;
        let avertissement = match operation {
            Some(operation) => self.documents.appliquer(&operation)?,
            None => None,
        };
        let issue = if deja.captation {
            Issue::Dedoublonne
        } else {
            Issue::Verse
        };
        etat.nommees.insert(empreinte.clone());
        etat.donnes.insert(
            rang,
            Donne::Lu {
                empreinte,
                illisible,
            },
        );
        Ok(FichierVerse {
            rang,
            issue,
            avertissement,
        })
    }

    /// La prise en stems du jeu de `commande`, dans l'import `etat` (voir
    /// `verser_une_prise_en_stems`).
    fn verser_le_jeu<S: Source>(
        &mut self,
        etat: &mut EtatDeLImport,
        source: &S,
        commande: &VerserUnePriseEnStems,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<Faite> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        let dossier = commande.dossier.as_str();
        match etat.jeu_verse(dossier) {
            None => {
                return Err(refus_de_l_import(
                    "Ce dossier n'est pas un jeu de stems de cet import.",
                ));
            }
            Some(true) => {
                return Ok(Faite {
                    avertissement: None,
                });
            }
            Some(false) => {}
        }
        let copies = etat
            .stems_du_jeu(dossier)
            .ok_or_else(|| refus_de_l_import("Les stems de ce jeu ne sont pas tous versés."))?;
        let stems: Vec<&str> = copies
            .iter()
            .map(|(stem, _)| stem.empreinte.as_str())
            .collect();
        let midis: Vec<&str> = copies
            .iter()
            .filter_map(|(_, midi)| midi.as_ref())
            .map(|midi| midi.empreinte.as_str())
            .collect();
        let manifeste = self.captations.verser(&rapport::manifeste(&stems))?;
        let fonds = self.documents.fonds().ok_or_else(fonds_non_ne)?;
        let deja = lecture::deja_de_la_prise(fonds, &manifeste, &stems, &midis);
        let a_verser: Vec<StemAVerser<'_>> = copies
            .iter()
            .map(|(stem, midi)| StemAVerser {
                empreinte: &stem.empreinte,
                faits: &stem.faits,
                chemin: stem.chemin.as_deref(),
                midi: midi.as_ref().map(|midi| MidiDuStem {
                    empreinte: &midi.empreinte,
                    faits: &midi.faits,
                    chemin: midi.chemin.as_deref(),
                }),
            })
            .collect();
        let nom = if dossier.is_empty() {
            source.nom_du_dossier()
        } else {
            Some(dossier)
        };
        let declaree = copies
            .first()
            .and_then(|(stem, _)| stem.chemin.as_deref())
            .and_then(|chemin| origine::declaration_du_fichier(chemin, &etat.declarations).origine);
        let idee = if deja.prise.captation {
            String::new()
        } else {
            ulid_tire(contexte)
        };
        let ouvert = import::ImportOuvert {
            import: &etat.import,
            ouvert_le: etat.ouvert_le,
            par: self.appareil,
        };
        let operation = import::verser_une_prise_en_stems(
            &ouvert,
            &manifeste,
            nom,
            &a_verser,
            declaree.as_ref(),
            &deja,
            &idee,
            contexte.heure,
            commande.decalage_minutes,
        )
        .map_err(refus_impossible)?;
        let avertissement = match operation {
            Some(operation) => self.documents.appliquer(&operation)?,
            None => None,
        };
        etat.nommees.extend(
            stems
                .iter()
                .chain(&midis)
                .map(|empreinte| empreinte.to_string()),
        );
        etat.nommees.insert(manifeste);
        etat.noter_le_jeu_verse(dossier);
        Ok(Faite { avertissement })
    }

    /// La clôture de l'import `etat`, jusqu'à son écriture (voir `clore_l_import`).
    fn ecrire_la_cloture(
        &mut self,
        etat: &EtatDeLImport,
        commande: &CloreLImport,
        contexte: &mut Contexte<'_>,
    ) -> Reponse<ImportClos> {
        if let Some(refus) = self.documents.lecture_seule() {
            return Err(refus);
        }
        if !etat.tout_est_verse() {
            return Err(refus_de_l_import("Des fichiers restent à verser."));
        }
        let fonds = self.documents.fonds().ok_or_else(fonds_non_ne)?;
        let connues = lecture::empreintes_nommees_a(fonds, &etat.tete);
        let origines: Vec<rapport::OrigineChangee> = etat
            .empreintes_versees()
            .iter()
            .flat_map(|empreinte| etat_de_l_import::origines_changees(fonds, empreinte, &etat.tete))
            .collect();
        let octets = rapport::rapport(
            &etat.import,
            &etat.fichiers_du_rapport(),
            &connues,
            &origines,
        );
        let relu = rapport::lire_le_rapport(&octets).ok_or_else(|| Refus {
            raison: Raison::Illisible,
            message: "Le rapport de cet import ne se relit pas.".to_string(),
        })?;
        let empreinte = self.captations.verser(&octets)?;
        let ouvert = import::ImportOuvert {
            import: &etat.import,
            ouvert_le: etat.ouvert_le,
            par: self.appareil,
        };
        let avertissement = self.documents.appliquer(&import::clore(
            &ouvert,
            &empreinte,
            contexte.heure,
            commande.decalage_minutes,
        ))?;
        Ok(ImportClos {
            rapport: etat_de_l_import::rapport_des_commandes(&relu),
            avertissement,
        })
    }

    /// Retire les copies de l'import `import` que `copies_a_retirer` désigne à la clôture : toutes.
    fn retirer_les_copies(&self, import: &str) -> Result<(), Refus> {
        let copies = self.captations.copies_de_l_import(import)?;
        for nom in copies_a_retirer(&copies, import, &BTreeSet::new(), true) {
            self.captations.retirer_une_copie(&nom)?;
        }
        Ok(())
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
/// compte par `compte` ; les étiquettes du fonds ; puis la fenêtre des trouvées, le nom de fichier
/// de chaque ligne sans texte lu dans `fonds`. `boite` et `boite_sans_index` la partagent.
fn vue_de_la_boite(
    l_index: &Index,
    commande: &commandes::Boite,
    fonds: Option<&Automerge>,
) -> VueDeLaBoite {
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
        lignes: fenetre(reponse.trouvees, commande.debut, commande.nombre, fonds),
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
/// à `FENETRE_MAX` (`dans_la_fenetre`). Chaque ligne sans texte reçoit le premier nom vu de sa
/// première captation (`lecture::premier_nom_vu`), les captations des idées lues de `fonds` en un
/// passage (`lecture::captations_par_idee`), et seulement si une ligne de la fenêtre est sans
/// texte ; sans fonds, aucun.
fn fenetre(
    rangees: Vec<EnBoite>,
    debut: u32,
    nombre: u32,
    fonds: Option<&Automerge>,
) -> Vec<LigneDeLaBoite> {
    let lignes: Vec<EnBoite> = dans_la_fenetre(rangees, debut, nombre).collect();
    let captations = match fonds {
        Some(fonds) if lignes.iter().any(|ligne| ligne.idee.texte.is_none()) => {
            lecture::captations_par_idee(fonds)
        }
        _ => BTreeMap::new(),
    };
    lignes
        .into_iter()
        .map(|dans_la_boite| {
            let nom_de_fichier = match (fonds, &dans_la_boite.idee.texte) {
                (Some(fonds), None) => captations
                    .get(&dans_la_boite.idee.id)
                    .and_then(|siennes| siennes.first())
                    .and_then(|premiere| lecture::premier_nom_vu(fonds, &premiere.empreinte)),
                _ => None,
            };
            ligne(dans_la_boite, nom_de_fichier)
        })
        .collect()
}

/// La ligne d'une idée de la boîte : ce qu'elle montre, son texte borné à `LONGUEUR_D_UNE_LIGNE`
/// caractères, ses étiquettes dans l'ordre de leurs octets, et `nom_de_fichier`.
fn ligne(dans_la_boite: EnBoite, nom_de_fichier: Option<String>) -> LigneDeLaBoite {
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
        nom_de_fichier,
    }
}

/// Ce que la vue d'une idée montre de la captation `captation` : ses faits, sous le conteneur de
/// sa sorte (`lecture::faits_de`), sa durée celle de la captation lue (d'une prise en stems, celle
/// de ses stems) ; ses noms vus (`noms_vus`), ceux de son conteneur ; son origine déclarée, la plus
/// forte (`origine::la_plus_forte`) ; d'une prise, ses stems (`lecture::stems_de`), chacun par
/// `stem_vu`.
fn captation_vue(fonds: &Automerge, captation: &CaptationLue) -> CaptationVue {
    let conteneur = captation.sorte.conteneur();
    let empreinte = &captation.empreinte;
    let mut faits = lecture::faits_de(fonds, conteneur, empreinte);
    faits.duree_ms = captation.duree_ms;
    let stems = if captation.sorte == gestes::Sorte::Prise {
        lecture::stems_de(fonds, empreinte)
            .into_iter()
            .map(|stem| stem_vu(fonds, stem))
            .collect()
    } else {
        Vec::new()
    };
    CaptationVue {
        captation: Captation {
            sorte: sorte_des_commandes(captation.sorte),
            empreinte: empreinte.clone(),
        },
        faits: etat_de_l_import::faits_des_commandes(&faits),
        noms: noms_vus(lecture::noms_de(fonds, conteneur, empreinte, None)),
        origine: etat_de_l_import::origine_des_commandes(&lecture::origines_de(fonds, empreinte)),
        stems,
    }
}

/// Ce que la vue montre du stem `stem` : ses faits, ses noms vus, ceux de son entrée dans la
/// prise ; son type, gardé ou choisi (`TypeVu::Garde`, `TypeVu::Choisi`), ou, sans type, celui
/// que propose le premier de ses noms vus (`stems::type_propose` ; question 18) ; son MIDI, le
/// premier qu'il porte, ses faits sous `midi_verses` et ses noms vus.
fn stem_vu(fonds: &Automerge, stem: StemLu) -> StemVu {
    let noms = noms_vus(stem.noms);
    let type_de_stem = match stem.type_de_stem {
        Some(TypeLu {
            type_de_stem,
            provenance: ProvenanceDuType::Garde { .. },
            ..
        }) => TypeVu::Garde(etat_de_l_import::type_des_commandes(type_de_stem)),
        Some(TypeLu {
            type_de_stem,
            provenance: ProvenanceDuType::Choisi,
            ..
        }) => TypeVu::Choisi(etat_de_l_import::type_des_commandes(type_de_stem)),
        None => TypeVu::Propose(etat_de_l_import::type_des_commandes(stems::type_propose(
            noms.first().map(String::as_str),
        ))),
    };
    let midi = stem.midis.into_iter().next().map(|midi| MidiVu {
        faits: etat_de_l_import::faits_des_commandes(&lecture::faits_de(fonds, MIDI_VERSES, &midi)),
        noms: noms_vus(lecture::noms_de(fonds, MIDI_VERSES, &midi, None)),
        empreinte: midi,
    });
    StemVu {
        empreinte: stem.empreinte,
        faits: etat_de_l_import::faits_des_commandes(&stem.faits),
        noms,
        type_de_stem,
        midi,
    }
}

/// Les noms vus de `noms`, des couples (chemin, ULID) : le premier vu
/// (`origine::premier_nom_vu`) d'abord, puis les autres chemins, chacun une fois, dans l'ordre de
/// leurs octets.
fn noms_vus(noms: Vec<(String, String)>) -> Vec<String> {
    let premier = origine::premier_nom_vu(
        noms.iter()
            .map(|(chemin, vu_par)| (chemin.as_str(), vu_par.as_str())),
    )
    .map(str::to_string);
    let autres: BTreeSet<String> = noms
        .into_iter()
        .map(|(chemin, _)| chemin)
        .filter(|chemin| premier.as_ref() != Some(chemin))
        .collect();
    premier.into_iter().chain(autres).collect()
}

/// Le poids du fichier complet du fonds (`Automerge::save`) ; sans fonds, 0.
fn poids_du_fonds(fonds: Option<&Automerge>) -> u64 {
    fonds.map_or(0, |fonds| {
        u64::try_from(fonds.save().len()).unwrap_or(u64::MAX)
    })
}

/// Le refus d'une commande de l'import qui ne se fait pas : `Raison::Impossible`, `message`.
fn refus_de_l_import(message: &str) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: message.to_string(),
    }
}

/// L'avertissement d'une déclaration « de la main de Sacha » qu'une cible générée garde
/// (question 22 ; V9).
const ORIGINE_GARDEE: &str = "L'origine reste générée : une origine déclarée ne se retire pas.";

/// Le refus d'une déclaration par préfixe qui ne nomme pas le dernier import clos, ou qui suit
/// une inspection (R6).
const PLUS_INSPECTE: &str = "Ce dossier n'est plus inspecté.";

/// Le refus d'une captation que le fonds ne porte pas : le constat
/// `Impossible::CaptationAbsente` du noyau, qui ne nomme pas l'empreinte (question 26).
fn captation_absente(empreinte: &str) -> Refus {
    refus_impossible(Impossible::CaptationAbsente {
        empreinte: empreinte.to_string(),
    })
}

/// Ce qui nomme la captation `empreinte` de la sorte `sorte` dans un message (question 26) : son
/// premier nom vu (`lecture::premier_nom_vu`), entre guillemets ; sans nom, « cette prise », « ce
/// MIDI versé seul » ou « ce texte versé ».
fn captation_nommee(fonds: &Automerge, sorte: gestes::Sorte, empreinte: &str) -> String {
    match lecture::premier_nom_vu(fonds, empreinte) {
        Some(nom) => format!("« {nom} »"),
        None => match sorte {
            gestes::Sorte::Prise => "cette prise",
            gestes::Sorte::MidiVerseSeul => "ce MIDI versé seul",
            gestes::Sorte::TexteVerse => "ce texte versé",
        }
        .to_string(),
    }
}

/// Deux avertissements, le premier puis le second, séparés d'une espace ; un seul, tel quel ;
/// aucun, `None`.
fn joindre(premier: Option<String>, second: Option<String>) -> Option<String> {
    match (premier, second) {
        (Some(premier), Some(second)) => Some(format!("{premier} {second}")),
        (premier, second) => premier.or(second),
    }
}

/// L'élément d'origine du noyau qui répond à l'origine à déclarer des commandes, comme pour les
/// déclarations d'un import : `Aucune` en `None`, `Main` en `Some(origine::Ajout::Main)`,
/// `Generee { nom }` en `Some(origine::Ajout::Generee(nom))`.
fn ajout_du_noyau(declaree: &OrigineADeclarer) -> Option<origine::Ajout> {
    match declaree {
        OrigineADeclarer::Aucune => None,
        OrigineADeclarer::Main => Some(origine::Ajout::Main),
        OrigineADeclarer::Generee { nom } => Some(origine::Ajout::Generee(nom.clone())),
    }
}

/// Le choix du type d'un stem du noyau qui répond à celui des commandes.
fn choix_du_noyau(choix: &commandes::ChoixDuType) -> import::ChoixDuType {
    match choix {
        commandes::ChoixDuType::Garder => import::ChoixDuType::Garder,
        commandes::ChoixDuType::Choisir(choisi) => {
            import::ChoixDuType::Choisir(type_du_noyau(choisi))
        }
    }
}

/// Le type de stem du noyau qui répond à celui des commandes : l'inverse de
/// `etat_de_l_import::type_des_commandes`.
fn type_du_noyau(type_de_stem: &commandes::TypeDeStem) -> stems::TypeDeStem {
    match type_de_stem {
        commandes::TypeDeStem::Voix => stems::TypeDeStem::Voix,
        commandes::TypeDeStem::Choeurs => stems::TypeDeStem::Choeurs,
        commandes::TypeDeStem::Batterie => stems::TypeDeStem::Batterie,
        commandes::TypeDeStem::Percussions => stems::TypeDeStem::Percussions,
        commandes::TypeDeStem::Basse => stems::TypeDeStem::Basse,
        commandes::TypeDeStem::Guitare => stems::TypeDeStem::Guitare,
        commandes::TypeDeStem::Clavier => stems::TypeDeStem::Clavier,
        commandes::TypeDeStem::Cordes => stems::TypeDeStem::Cordes,
        commandes::TypeDeStem::Cuivres => stems::TypeDeStem::Cuivres,
        commandes::TypeDeStem::Synthe => stems::TypeDeStem::Synthe,
        commandes::TypeDeStem::Effets => stems::TypeDeStem::Effets,
        commandes::TypeDeStem::Autre => stems::TypeDeStem::Autre,
    }
}

/// Les noms de générateurs de `fonds` (`lecture::generateurs`), chacun une fois, dans l'ordre de
/// leurs octets ; sans fonds, aucun.
fn generateurs_du_fonds(fonds: Option<&Automerge>) -> Vec<String> {
    fonds
        .map(lecture::generateurs)
        .unwrap_or_default()
        .into_iter()
        .collect()
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
