//! Les services de pupitre (§ 40 · 3) : les opérations nommées et les lectures du fonds,
//! conduites sur les contrats `Documents` et `Captations`, que réalisent les adaptateurs, sur
//! fichiers (le stockage) ou en mémoire. `Services` conduit chaque commande : il tire de l'aléa
//! du `Contexte` les identifiants qu'elle forme, demande l'opération au noyau, l'écrit par
//! `Documents`, et rend la réponse aux types de `commandes`. `application` applique les
//! changements que le noyau calcule ; `lecture` tire du document le fonds lu, que le noyau
//! reçoit.
//!
//! La fonction `en-memoire`, que seuls les tests demandent, compile le module `en_memoire`, la
//! paire d'adaptateurs en mémoire.

pub mod application;
mod contrats;
#[cfg(feature = "en-memoire")]
pub mod en_memoire;
pub mod lecture;

pub use commandes::{Raison, Refus};
pub use contrats::{Alea, Captations, Documents, Ecriture};

use automerge::Automerge;
use commandes::{
    Captation, Completer, EntreeDuJournal, Etiqueter, Faite, Notee, Noter, Regrouper, Relier,
    Reponse, Viser, VueDUneIdee,
};
use noyau::donnees::changement::Operation;
use noyau::donnees::gestes::{self, GesteDeTri, IdeeLue, Impossible};
use noyau::donnees::identifiants::{ulid, uuid_v4};

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
pub struct Services<D: Documents, C: Captations> {
    documents: D,
    captations: C,
    appareil: gestes::Appareil,
    /// Le dernier geste de tri fait par ces services, que l'annulation défait (§ 30 · 11) ; aucun
    /// à leur ouverture.
    dernier_geste: Option<GesteDeTri>,
}

impl<D: Documents, C: Captations> Services<D, C> {
    /// Les services de l'appareil `appareil`, sur `documents` et `captations`, sans dernier geste
    /// de tri.
    pub fn new(documents: D, captations: C, appareil: gestes::Appareil) -> Self {
        Services {
            documents,
            captations,
            appareil,
            dernier_geste: None,
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

    /// Relier les idées `commande.a` et `commande.b` (`gestes::relier`), lues dans cet ordre : un
    /// geste de tri.
    pub fn relier(&mut self, commande: &Relier, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        self.trier(&commande.a, contexte.heure, |fonds, heure| {
            let a = idee_lue(fonds, &commande.a)?;
            let b = idee_lue(fonds, &commande.b)?;
            gestes::relier(&a, &b, heure, commande.decalage_minutes)
        })
    }

    /// Délier les idées `commande.a` et `commande.b` (`gestes::delier`), lues dans cet ordre : un
    /// geste de tri.
    pub fn delier(&mut self, commande: &Relier, contexte: &mut Contexte<'_>) -> Reponse<Faite> {
        self.trier(&commande.a, contexte.heure, |fonds, heure| {
            let a = idee_lue(fonds, &commande.a)?;
            let b = idee_lue(fonds, &commande.b)?;
            gestes::delier(&a, &b, heure, commande.decalage_minutes)
        })
    }

    /// Regrouper la captation `commande.captation` dans l'idée `commande.cible`
    /// (`gestes::regrouper`) : un geste de tri. La cible se lit d'abord ; puis la captation, qui
    /// ne se lit pas : `Impossible::CaptationAbsente` ; puis sa source, l'idée que la captation
    /// nomme, jamais une autre, et le compte des captations de la source, elle comprise.
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
        let prises: Vec<Option<u64>> = captations
            .iter()
            .filter(|captation| captation.sorte == gestes::Sorte::Prise)
            .map(|captation| captation.duree_ms)
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
            duree_ms: noyau::donnees::idee::duree(&prises),
            retiree: lue.retiree,
        })
    }

    /// Le journal du fonds : le message et l'heure de chaque modification, en lecture seule
    /// comme ailleurs ; sans fonds, aucune entrée.
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
}

/// L'idée `idee`, telle qu'elle se lit dans `fonds` ; aucune : `Impossible::IdeeAbsente`
/// (`gestes::presente`).
fn idee_lue(fonds: &Automerge, idee: &str) -> Result<IdeeLue, Impossible> {
    gestes::presente(idee, lecture::lire_l_idee(fonds, idee))
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
