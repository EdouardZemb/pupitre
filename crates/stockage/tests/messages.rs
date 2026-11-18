//! Les messages du stockage et de la synchronisation : chaque erreur, chaque raison de la lecture
//! seule, chaque avertissement et chaque échec d'une écriture à côté se disent en français, par
//! un constat, jamais par un ordre (§ 50 · 5). Un chemin s'y écrit par ses composants joints par
//! `/`, le même texte sous Linux et sous Windows, et le texte d'une erreur du système ou
//! d'Automerge n'y paraît jamais : le genre de l'erreur du système en forme le constat.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use automerge::AutomergeError;
use noyau::donnees::copie::Difference;
use noyau::donnees::gestes::Impossible;
use noyau::donnees::teneur::ErreurDeTeneur;
use services::application::Inapplicable;
use stockage::fichier::Echec;
use stockage::{Avertissement, Erreur, LectureSeule, empreinte};

/// `documents/fonds/<nom>`, par ses composants : `\` les sépare sous Windows, `/` ailleurs.
fn dans_le_fonds(nom: &str) -> PathBuf {
    Path::new("documents").join("fonds").join(nom)
}

/// `documents/fonds/ajouts/<nom>`, par ses composants.
fn dans_les_ajouts(nom: &str) -> PathBuf {
    dans_le_fonds("ajouts").join(nom)
}

/// Une erreur du système, de ce genre, et de ce texte, en anglais, qui ne doit jamais paraître.
fn refus_du_systeme(genre: ErrorKind, texte: &str) -> io::Error {
    io::Error::new(genre, texte)
}

/// Chaque cas d'`Erreur`, de `LectureSeule`, d'`Avertissement`, d'`Echec` et de
/// `synchro::Erreur`, et un cas d'`Erreur::Disque` par genre d'erreur du système : son texte,
/// puis le texte attendu, en entier.
fn messages() -> Vec<(String, String)> {
    let e = empreinte(b"une captation de test");
    let complet = dans_le_fonds("complet.automerge");
    let ajout = dans_les_ajouts(&format!("{e}.automerge"));
    let voisin = dans_les_ajouts(&format!("{e}.automerge.4242.partiel"));
    let provisoire = Path::new("captations").join(format!("{e}.4242.partiel"));
    let c_ajout = format!("documents/fonds/ajouts/{e}.automerge");
    let c_voisin = format!("documents/fonds/ajouts/{e}.automerge.4242.partiel");
    let c_provisoire = format!("captations/{e}.4242.partiel");
    let disque = |chemin: &Path, genre: ErrorKind, texte: &str| Erreur::Disque {
        chemin: chemin.to_path_buf(),
        erreur: refus_du_systeme(genre, texte),
    };
    let lectures_seules = [
        (
            LectureSeule::RelectureManquee {
                chemin: ajout.clone(),
            },
            format!(
                "La relecture de {c_ajout}, après une écriture refusée, a échoué : le \
                 document montre ce qu'il était avant elle, et se lit sans écrire ni échanger."
            ),
        ),
        (
            LectureSeule::AjoutAltere {
                chemin: ajout.clone(),
            },
            format!(
                "Le contenu de {c_ajout} ne donne pas l'empreinte que porte son nom : il n'est \
                 pas chargé, et le document se lit sans écrire ni échanger."
            ),
        ),
        (
            LectureSeule::AjoutIllisible {
                chemin: ajout.clone(),
            },
            format!(
                "{c_ajout} donne l'empreinte de son nom, mais ne se charge pas : le document se \
                 lit sans lui, sans écrire ni échanger."
            ),
        ),
        (
            LectureSeule::CompletIllisible {
                chemin: complet.clone(),
            },
            "documents/fonds/complet.automerge ne se lit pas en entier : le document montre ce \
             qui s'en lit, sans écrire ni échanger."
                .to_string(),
        ),
        (
            LectureSeule::DependancesManquantes {
                chemin: ajout.clone(),
            },
            format!(
                "{c_ajout} attend une modification qu'aucun fichier lu ne porte : un fichier \
                 d'ajout manque, et le document se lit sans écrire ni échanger."
            ),
        ),
        (
            LectureSeule::VersionPlusRecente { version: 2 },
            "Ce fonds vient d'une version plus récente de Pupitre ; cette version-ci le lit \
             sans y écrire."
                .to_string(),
        ),
        (
            LectureSeule::VersionIllisible,
            "La version de schéma du document ne se lit pas : il se lit sans écrire ni échanger."
                .to_string(),
        ),
        (
            LectureSeule::MigrationRefusee,
            "Une migration de ce fonds ne s'applique pas : il se lit sans écrire ni échanger."
                .to_string(),
        ),
    ];

    let mut messages = vec![
        (
            Erreur::RacineDesDonneesTenue.to_string(),
            "Un autre processus tient la racine des données.".to_string(),
        ),
        // Un cas par genre d'erreur du système, sur un fichier ou sur un dossier : le texte du
        // système, en anglais, ne paraît pas.
        (
            disque(&complet, ErrorKind::NotFound, "No such file or directory").to_string(),
            "documents/fonds/complet.automerge est absent.".to_string(),
        ),
        (
            disque(
                &dans_le_fonds("ajouts"),
                ErrorKind::NotFound,
                "No such file or directory",
            )
            .to_string(),
            "documents/fonds/ajouts est absent.".to_string(),
        ),
        (
            disque(
                &ajout,
                ErrorKind::PermissionDenied,
                "Access is denied. (os error 5)",
            )
            .to_string(),
            format!("{c_ajout} est refusé par le système."),
        ),
        (
            disque(&ajout, ErrorKind::StorageFull, "No space left on device").to_string(),
            format!("{c_ajout} ne tient plus sur le disque."),
        ),
        (
            disque(
                &complet,
                ErrorKind::InvalidData,
                "stream did not contain valid data",
            )
            .to_string(),
            "documents/fonds/complet.automerge ne se relit pas tel qu'il a été écrit.".to_string(),
        ),
        // Le refus d'une lecture, à l'ouverture : un constat neutre quant au geste, qui ne dit
        // pas « s'écrire ».
        (
            disque(&complet, ErrorKind::Other, "read failed").to_string(),
            "documents/fonds/complet.automerge, le disque l'a refusé.".to_string(),
        ),
        (
            Erreur::CaptationAlteree {
                empreinte: e.clone(),
            }
            .to_string(),
            format!("Le contenu relu de la captation {e} ne donne pas son empreinte."),
        ),
        (
            Erreur::CaptationAbsente {
                empreinte: e.clone(),
            }
            .to_string(),
            format!("Aucune captation ne porte l'empreinte {e}."),
        ),
        (
            Erreur::FondsDejaOuvert.to_string(),
            "Le fonds est déjà ouvert dans ce processus.".to_string(),
        ),
        (
            Erreur::FondsDejaNe.to_string(),
            "Le fonds est déjà né : il ne naît qu'une fois.".to_string(),
        ),
        (
            Erreur::Automerge(AutomergeError::Fail).to_string(),
            "Le document a refusé ce geste, ou sa racine ne se lit pas.".to_string(),
        ),
        (
            Erreur::CompactionInfidele.to_string(),
            "Le fichier complet de la compaction, relu, ne mène pas au document en mémoire : \
             aucun fichier d'ajout n'est retiré."
                .to_string(),
        ),
        (
            Erreur::ChampAbsent { champ: "lignee" }.to_string(),
            "La racine du document ne porte pas de chaîne sous « lignee » : la copie ne se \
             situe pas."
                .to_string(),
        ),
        (
            Erreur::Synchro(synchro::Erreur::MessageIllisible(
                "not enough input".to_string(),
            ))
            .to_string(),
            "Le message reçu ne se lit pas : le document n'a pas changé.".to_string(),
        ),
        (
            Erreur::Inapplicable(Inapplicable::Vide).to_string(),
            "L'opération ne change rien au fonds.".to_string(),
        ),
        (
            synchro::Erreur::MessageIllisible("not enough input".to_string()).to_string(),
            "Le message reçu ne se lit pas : le document n'a pas changé.".to_string(),
        ),
        (
            synchro::Erreur::Automerge(AutomergeError::Fail).to_string(),
            "Le message reçu ne s'applique pas au document.".to_string(),
        ),
        (
            Avertissement::NonDurable {
                chemin: ajout.clone(),
                erreur: refus_du_systeme(ErrorKind::Other, "sync failed"),
            }
            .to_string(),
            format!("{c_ajout} est en place, mais son renommage n'est pas rendu durable."),
        ),
        (
            Avertissement::Compaction(disque(&complet, ErrorKind::Other, "write failed"))
                .to_string(),
            "La compaction a échoué, sans rien perdre, et se retente à l'écriture suivante : \
             documents/fonds/complet.automerge, le disque l'a refusé."
                .to_string(),
        ),
        (
            Avertissement::Compaction(Erreur::CompactionInfidele).to_string(),
            "La compaction a échoué, sans rien perdre, et se retente à l'écriture suivante : \
             Le fichier complet de la compaction, relu, ne mène pas au document en mémoire : \
             aucun fichier d'ajout n'est retiré."
                .to_string(),
        ),
        (
            Avertissement::VoisinRestant {
                chemin: voisin,
                erreur: refus_du_systeme(ErrorKind::PermissionDenied, "Access is denied."),
            }
            .to_string(),
            format!(
                "Le voisin {c_voisin} n'a pas pris la place de son fichier : il se lit où il est, \
                 et la prend à l'ouverture suivante."
            ),
        ),
        (
            Avertissement::NomProvisoireRestant {
                chemin: provisoire,
                erreur: refus_du_systeme(ErrorKind::Other, "rename failed"),
            }
            .to_string(),
            format!(
                "{c_provisoire} n'a pas pu se lire, ou prendre le nom de son empreinte : il \
                 reste, et l'ouverture suivante le relit."
            ),
        ),
        (
            Echec::Refusee(refus_du_systeme(ErrorKind::Other, "rename failed")).to_string(),
            "Le fichier écrit à côté, le disque l'a refusé : celui qu'il devait remplacer n'a \
             pas changé."
                .to_string(),
        ),
        (
            Echec::Refusee(refus_du_systeme(
                ErrorKind::StorageFull,
                "No space left on device",
            ))
            .to_string(),
            "Le fichier écrit à côté ne tient plus sur le disque : celui qu'il devait \
             remplacer n'a pas changé."
                .to_string(),
        ),
        (
            Echec::NonDurable(refus_du_systeme(ErrorKind::Other, "sync failed")).to_string(),
            "Le fichier est en place, mais son renommage n'est pas rendu durable.".to_string(),
        ),
    ];
    for (lecture_seule, attendu) in lectures_seules {
        // `Erreur::LectureSeule` dit le texte de sa raison.
        messages.push((
            Erreur::LectureSeule(lecture_seule.clone()).to_string(),
            attendu.clone(),
        ));
        messages.push((lecture_seule.to_string(), attendu));
    }
    messages
}

#[test]
fn chaque_erreur_a_son_message_en_francais() {
    for (rendu, attendu) in messages() {
        assert_eq!(rendu, attendu);
    }
}

#[test]
fn aucun_message_n_est_un_ordre() {
    let e = empreinte(b"une captation de test");
    let mut textes: Vec<String> = messages().into_iter().map(|(rendu, _)| rendu).collect();
    textes.extend(
        [
            Impossible::TexteVide,
            Impossible::IdeeAbsente {
                idee: "01HF7YAT000000000000000004".to_string(),
            },
            Impossible::IdeeRetiree {
                idee: "01HF7YAT000000000000000004".to_string(),
            },
            Impossible::IdeeNonRetiree {
                idee: "01HF7YAT000000000000000004".to_string(),
            },
            Impossible::MemeTexte,
            Impossible::EtiquetteVide,
            Impossible::EtiquetteDejaPosee {
                etiquette: "pont".to_string(),
            },
            Impossible::EtiquetteAbsente {
                etiquette: "pont".to_string(),
            },
            Impossible::RelieeAElleMeme,
            Impossible::LienDejaPose,
            Impossible::LienAbsent,
            Impossible::CaptationAbsente {
                empreinte: e.clone(),
            },
            Impossible::DejaDansLIdee,
            Impossible::CaptationDejaVersee {
                empreinte: e.clone(),
            },
            Impossible::DureeHorsDUnePrise,
            Impossible::DureeTropLongue {
                duree_ms: 9_007_199_254_740_992,
            },
            Impossible::HorsDeLImport,
            Impossible::NomVide,
            Impossible::MemeNom,
            Impossible::RequeteSansCritere,
            Impossible::RequeteAbsente {
                requete: "01HF7YAT000000000000000005".to_string(),
            },
            Impossible::SeuilTropLong {
                secondes: 9_007_199_254_740_992,
            },
        ]
        .iter()
        .map(ToString::to_string),
    );
    textes.extend(
        [
            Inapplicable::ObjetAbsent {
                chemin: vec!["idees".to_string(), "absente".to_string()],
            },
            Inapplicable::Vide,
            Inapplicable::Automerge(AutomergeError::Fail),
        ]
        .iter()
        .map(ToString::to_string),
    );
    textes.extend(
        [
            Difference::AutreProprietaire,
            Difference::AutreDocument,
            Difference::AutreLignee,
        ]
        .iter()
        .map(ToString::to_string),
    );
    textes.extend(
        [
            ErreurDeTeneur::NaN {
                chemin: vec!["a".to_string(), "b".to_string()],
            },
            ErreurDeTeneur::Inconnu {
                chemin: vec!["k".to_string(), "1".to_string()],
                type_code: 9,
            },
        ]
        .iter()
        .map(ToString::to_string),
    );

    let ordres = [
        "il faut",
        "veuillez",
        "vous devez",
        "mettez",
        "essayez",
        "réessayez",
        "vérifiez",
        "relancez",
        "choisissez",
        "ouvrez",
        "fermez",
    ];
    for texte in &textes {
        let minuscules = texte.to_lowercase();
        for ordre in ordres {
            assert!(!minuscules.contains(ordre), "« {ordre} » dans « {texte} »");
        }
    }
}
