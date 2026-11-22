//! La boîte d'idées à l'écran, zone du fonds (CAP-02, CAP-06, CAP-07 ; questions 25 à 30). En
//! tête, le champ « Noter une idée », toujours présent, le compte, et sous une requête qui pose un
//! critère sa réponse, « Ranger par : », les critères de la requête en cours (« Contenu : »,
//! « Date : », « Étiquette : », « Durée : », « État : », « Provenance : »), les requêtes
//! enregistrées (les choisir, en enregistrer une, la renommer, la supprimer) et les gestes :
//! relier, annuler, les idées retirées, le journal du fonds. Dessous, la liste en fenêtre, réponse
//! de la requête en cours, des lignes de hauteur fixe que suivent son défilement et la taille de
//! la fenêtre de l'application, jamais une minuterie (I18) ; sans idée interrogée, la page du
//! fonds vide la cache et dit, par des constats, ce qu'il en est. À droite de la liste, le volet
//! de l'idée choisie, celui des idées retirées, ou celui du journal du fonds, en lecture seule ;
//! Échap ferme le volet ouvert comme « Fermer », sauf dans la recherche. Après chaque commande,
//! l'interface relit l'état du fonds et les fenêtres, et le bandeau reçoit l'issue ; seul le
//! refus d'une lecture y passe, jamais une lecture réussie. `Ctrl+Z`, hors d'un champ de texte,
//! annule le dernier geste de tri.

mod ecran;
mod etat;
mod fenetre;
mod focus;
mod gestes;
mod requetes;
mod textes;
mod volets;
mod vue;

pub(crate) use etat::EtatDeLaBoite;
pub(crate) use gestes::dans_un_champ_de_texte;
pub(crate) use textes::CONTENUS;
pub(crate) use vue::BoiteDIdees;

#[cfg(test)]
mod tests {
    use commandes::{Appareil, LigneDeLaBoite, Provenance, Requete, RequeteEnregistree};

    /// Une ligne inventée, captée sur l'établi, sans durée ni lien.
    pub(super) fn ligne(texte: Option<&str>, etiquettes: &[&str]) -> LigneDeLaBoite {
        LigneDeLaBoite {
            idee: "idee-inventee".to_string(),
            le: 1_793_750_400,
            texte: texte.map(str::to_string),
            etiquettes: etiquettes.iter().map(|e| e.to_string()).collect(),
            duree_ms: None,
            reliee: false,
            provenance: Provenance::Captee {
                par: Appareil::Etabli,
            },
            retiree: None,
            nom_de_fichier: None,
        }
    }

    /// Une ligne inventée de l'idée `idee`.
    pub(super) fn ligne_de(idee: &str) -> LigneDeLaBoite {
        LigneDeLaBoite {
            idee: idee.to_string(),
            ..ligne(Some("Une idée de pont"), &[])
        }
    }

    /// Le décalage local de l'heure de Paris, en minutes : l'heure d'été (120) jusqu'au
    /// 25 octobre 2026 à 1 h UTC, l'heure d'hiver (60) ensuite.
    pub(super) fn decalage_de_paris(le: i64) -> i32 {
        if le < 1_792_890_000 { 120 } else { 60 }
    }

    /// Une requête enregistrée inventée, lisible ou, sans requête, d'une version plus récente.
    pub(super) fn enregistree(nom: &str, requete: Option<Requete>) -> RequeteEnregistree {
        RequeteEnregistree {
            id: "requete-inventee".to_string(),
            nom: nom.to_string(),
            requete,
        }
    }
}
