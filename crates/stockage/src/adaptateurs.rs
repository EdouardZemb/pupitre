//! Les adaptateurs du stockage (§ 40 · 3) : le fonds sur fichiers réalise le contrat `Documents`
//! des services, et les captations de la racine des données, le contrat `Captations`. Chaque
//! erreur du stockage s'y rend en refus, sa raison dite par `raison`, son message le texte de
//! l'erreur ; un avertissement, de même, en texte. Chaque chemin s'y dit relatif à la racine des
//! données (`Relatif`), sauf dans le refus d'ouvrir la racine des données elle-même
//! (`refus_a_l_ouverture`), qui la nomme par son chemin entier.

use automerge::Automerge;
use noyau::donnees::changement::Operation;
#[cfg(feature = "pannes")]
use noyau::donnees::version::Migration;
use services::{Documents, Ecriture, Raison, Refus};

use crate::{Avertissement, Captations, Document, Erreur, LectureSeule, RacineDesDonnees, Relatif};

/// Le fonds d'une racine des données, sur fichiers : son document, s'il est né, que les services
/// lisent et écrivent par le contrat `Documents`.
pub struct FondsSurFichiers<'r> {
    racine: &'r RacineDesDonnees,
    document: Option<Document<'r>>,
}

impl<'r> FondsSurFichiers<'r> {
    /// Le fonds de `racine`, par [`RacineDesDonnees::fonds`] : son document, ou aucun tant que
    /// Pupitre est neuf. Une erreur d'ouverture se rend en refus.
    pub fn ouvrir(racine: &'r RacineDesDonnees) -> Result<Self, Refus> {
        let document = racine.fonds().map_err(|erreur| refus(erreur, racine))?;
        Ok(FondsSurFichiers { racine, document })
    }

    /// Le fonds de `racine`, ouvert selon les migrations `migrations` qu'un test lui donne, au
    /// lieu de `MIGRATIONS` (`RacineDesDonnees::fonds_selon`) : un point d'entrée des tests, hors
    /// de l'API livrée, sous la fonction `pannes`. Une erreur d'ouverture se rend en refus.
    #[cfg(feature = "pannes")]
    pub fn ouvrir_selon(
        racine: &'r RacineDesDonnees,
        migrations: &[Migration<'_>],
    ) -> Result<Self, Refus> {
        let document = racine
            .fonds_selon(migrations)
            .map_err(|erreur| refus(erreur, racine))?;
        Ok(FondsSurFichiers { racine, document })
    }

    /// Le document du fonds ; `None` tant qu'il n'est pas né.
    pub fn document(&self) -> Option<&Document<'r>> {
        self.document.as_ref()
    }

    /// Le document du fonds, à modifier : pour les tests à plusieurs répliques, qui réunissent
    /// deux documents par la synchronisation.
    pub fn document_mut(&mut self) -> Option<&mut Document<'r>> {
        self.document.as_mut()
    }
}

impl Documents for FondsSurFichiers<'_> {
    fn fonds(&self) -> Option<&Automerge> {
        self.document.as_ref().map(Document::automerge)
    }

    /// Le refus de la raison de lecture seule du document ; sans document, aucun : le fonds
    /// naîtra.
    fn lecture_seule(&self) -> Option<Refus> {
        let raison = self.document.as_ref()?.lecture_seule()?;
        Some(refus(Erreur::LectureSeule(raison.clone()), self.racine))
    }

    /// [`RacineDesDonnees::faire_naitre`] : le document né, et l'avertissement de sa naissance,
    /// s'il en vient un. Un document déjà ouvert ([`Erreur::FondsDejaOuvert`]), ou un fonds déjà
    /// né ([`Erreur::FondsDejaNe`]) : `Raison::Impossible`, et rien ne change.
    fn faire_naitre(&mut self, naissance: &Operation, geste: &Operation) -> Ecriture {
        let document = self
            .racine
            .faire_naitre(naissance, geste)
            .map_err(|erreur| refus(erreur, self.racine))?;
        let avertissement = document
            .avertissements()
            .first()
            .map(|avertissement| texte_de(avertissement, self.racine));
        self.document = Some(document);
        Ok(avertissement)
    }

    /// [`Document::appliquer`] ; sans document, `Raison::Impossible`, et rien ne s'écrit.
    fn appliquer(&mut self, operation: &Operation) -> Ecriture {
        let Some(document) = self.document.as_mut() else {
            return Err(Refus {
                raison: Raison::Impossible,
                message: "Aucun fonds n'est né.".to_string(),
            });
        };
        let racine = self.racine;
        let avertissement = document
            .appliquer(operation)
            .map_err(|erreur| refus(erreur, racine))?;
        Ok(avertissement
            .as_ref()
            .map(|avertissement| texte_de(avertissement, racine)))
    }
}

impl services::Captations for Captations<'_> {
    /// [`Captations::verser`] ; une erreur se rend en refus.
    fn verser(&self, octets: &[u8]) -> Result<String, Refus> {
        Captations::verser(self, octets).map_err(|erreur| refus(erreur, self.racine()))
    }

    /// [`Captations::relire`] ; une erreur se rend en refus, et une empreinte qu'aucune
    /// captation ne porte, en `Raison::Impossible`.
    fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Refus> {
        Captations::relire(self, empreinte).map_err(|erreur| refus(erreur, self.racine()))
    }
}

/// Le refus de `erreur` : sa raison (`raison`), et son message, le texte de l'erreur, chaque
/// chemin dit relatif à `racine`.
fn refus(erreur: Erreur, racine: &RacineDesDonnees) -> Refus {
    Refus {
        raison: raison(&erreur),
        message: Relatif(&erreur, racine.chemin()).to_string(),
    }
}

/// Le refus d'ouvrir la racine des données, qui n'a pas de racine à qui rapporter un chemin : la
/// raison que donne `refus` à chaque variante (`RacineDesDonneesTenue` : `Raison::Impossible`), et
/// le texte de l'erreur, qui nomme le dossier par son chemin entier, ses composants joints par `/`,
/// sous Windows aussi (`chemin_dit` ; question 3).
pub fn refus_a_l_ouverture(erreur: Erreur) -> Refus {
    Refus {
        raison: raison(&erreur),
        message: erreur.to_string(),
    }
}

/// La raison du refus de `erreur`, une par variante d'`Erreur`, et de `LectureSeule`.
fn raison(erreur: &Erreur) -> Raison {
    match erreur {
        Erreur::LectureSeule(LectureSeule::VersionPlusRecente { version }) => {
            Raison::VersionPlusRecente { version: *version }
        }
        Erreur::LectureSeule(
            LectureSeule::RelectureManquee { .. }
            | LectureSeule::AjoutAltere { .. }
            | LectureSeule::AjoutIllisible { .. }
            | LectureSeule::CompletIllisible { .. }
            | LectureSeule::DependancesManquantes { .. }
            | LectureSeule::VersionIllisible
            | LectureSeule::MigrationRefusee,
        ) => Raison::Illisible,
        Erreur::Disque { .. } | Erreur::CaptationAlteree { .. } => Raison::Disque,
        // Une opération qui ne s'applique pas, une empreinte qu'aucun fichier ne porte, un fonds
        // qui n'est plus neuf (`pupitre_neuf`), ou un autre `Document` de la racine des données
        // qui vit quand une commande fait naître le fonds (`FondsSurFichiers::faire_naitre`) : la
        // commande ne se fait pas sur le fonds tel qu'il est. `FondsDejaOuvert` survient aussi à
        // l'ouverture, avant les services (`FondsSurFichiers::ouvrir`).
        Erreur::Inapplicable(_)
        | Erreur::CaptationAbsente { .. }
        | Erreur::FondsDejaNe
        | Erreur::FondsDejaOuvert => Raison::Impossible,
        Erreur::Automerge(_) => Raison::Illisible,
        // Celles qui ne surviennent qu'avant les services, ou hors de leurs commandes, ont une
        // raison pour que la correspondance reste complète : la racine des données se prend avant
        // tout fonds, et son refus se dit par `refus_a_l_ouverture` ; seule `Document::compacter`,
        // que les services n'appellent pas, rend `CompactionInfidele` (après une écriture, une
        // compaction échouée est un avertissement) ; situer une copie et recevoir un message
        // relèvent de la synchronisation (§ 31 · 2.5), que les services ne conduisent pas.
        Erreur::RacineDesDonneesTenue => Raison::Impossible,
        Erreur::CompactionInfidele => Raison::Disque,
        Erreur::ChampAbsent { .. } | Erreur::Synchro(_) => Raison::Illisible,
    }
}

/// Le texte d'un avertissement, chaque chemin dit relatif à `racine`.
fn texte_de(avertissement: &Avertissement, racine: &RacineDesDonnees) -> String {
    Relatif(avertissement, racine.chemin()).to_string()
}
