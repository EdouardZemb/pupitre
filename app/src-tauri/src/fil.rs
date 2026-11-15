//! Le fil du fonds : un fil dédié possède la racine des données et les services de l'établi, sur
//! fichiers, et sert chaque demande qu'il reçoit par un canal, l'une après l'autre.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::dossier_jetable;
    use commandes::{Appareil, Notee, Noter, Provenance};

    /// La fermeture de `noter`, qui note `texte`, `decalage_minutes: 60`.
    fn noter_une(
        texte: &str,
    ) -> impl for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<Notee>
    + Send
    + 'static {
        let commande = Noter {
            texte: texte.to_string(),
            decalage_minutes: 60,
        };
        move |services, contexte| services.noter(&commande, contexte)
    }

    /// Ce que porte la racine des données : chaque fichier sous `documents/` et `captations/`, son
    /// chemin relatif et ses octets, dans l'ordre des chemins ; le fichier `verrou`, par sa seule
    /// présence et sa taille, jamais par ses octets : sous Windows, le verrou que tient la première
    /// ouverture peut en refuser la lecture.
    #[derive(Debug, PartialEq, Eq)]
    struct Photographie {
        fichiers: Vec<(String, Vec<u8>)>,
        verrou: Option<u64>,
    }

    fn photographie(racine: &Path) -> Photographie {
        let mut fichiers = Vec::new();
        for nom in ["documents", "captations"] {
            fichiers_sous(racine, &racine.join(nom), &mut fichiers);
        }
        fichiers.sort();
        let verrou = std::fs::metadata(racine.join("verrou"))
            .ok()
            .map(|m| m.len());
        Photographie { fichiers, verrou }
    }

    /// Chaque fichier sous `dossier`, en descendant, son chemin relatif à `racine`, ses composants
    /// joints par `/`, et ses octets.
    fn fichiers_sous(racine: &Path, dossier: &Path, fichiers: &mut Vec<(String, Vec<u8>)>) {
        let entrees = std::fs::read_dir(dossier)
            .unwrap_or_else(|e| panic!("lire {} : {e}", dossier.display()));
        for entree in entrees {
            let chemin = entree.expect("une entrée").path();
            if chemin.is_dir() {
                fichiers_sous(racine, &chemin, fichiers);
            } else {
                let relatif = chemin
                    .strip_prefix(racine)
                    .expect("un chemin sous la racine des données")
                    .components()
                    .map(|composant| composant.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let octets = std::fs::read(&chemin)
                    .unwrap_or_else(|e| panic!("lire {} : {e}", chemin.display()));
                fichiers.push((relatif, octets));
            }
        }
    }

    #[test]
    fn le_fil_note_puis_relit_l_idee() {
        let racine = dossier_jetable("fil-note").join("donnees");
        let ouverture = ouvrir(racine);
        let notee = ouverture
            .demander_bloquant(noter_une("Une idée de pont"))
            .unwrap();
        let vue = ouverture
            .demander_bloquant(move |services, _| services.vue_d_une_idee(&notee.idee))
            .unwrap();
        assert_eq!(vue.texte.as_deref(), Some("Une idée de pont"));
        assert_eq!(
            vue.provenance,
            Provenance::Captee {
                par: Appareil::Etabli
            }
        );
    }

    #[test]
    fn un_second_processus_devant_une_racine_tenue_s_arrete_en_le_disant() {
        let racine = dossier_jetable("fil-tenue").join("donnees");
        let premiere = ouvrir(racine.clone());
        premiere.demander_bloquant(noter_une("Une idée")).unwrap();
        let avant = photographie(&racine);
        let seconde = ouvrir(racine.clone());
        let Ouverture::Refusee(refus) = &seconde else {
            panic!("la seconde ouverture devait être refusée")
        };
        assert_eq!(refus.raison, Raison::Impossible);
        assert_eq!(
            refus.message,
            "Un autre processus tient la racine des données."
        );
        // Aucune commande servie, et rien d'écrit sous donnees/ (question 4 ; LE-6, règle 19).
        assert_eq!(
            seconde
                .demander_bloquant(noter_une("Une autre"))
                .unwrap_err(),
            refus.clone()
        );
        assert_eq!(photographie(&racine), avant);
    }

    #[test]
    fn le_fil_ferme_la_racine_quand_on_le_ferme() {
        let racine = dossier_jetable("fil-ferme").join("donnees");
        let premiere = ouvrir(racine.clone());
        premiere.fermer();
        assert!(matches!(ouvrir(racine), Ouverture::Ouverte(_)));
        let refus = premiere
            .demander_bloquant(noter_une("Une idée"))
            .unwrap_err();
        assert_eq!(refus.message, "Le fonds ne répond plus.");
    }

    #[test]
    fn une_racine_que_le_disque_refuse_se_dit_par_son_chemin() {
        let dossier = dossier_jetable("fil-disque");
        let racine = dossier.join("donnees");
        std::fs::write(&racine, b"un fichier, pas un dossier").unwrap();
        let Ouverture::Refusee(refus) = ouvrir(racine.clone()) else {
            panic!("l'ouverture devait être refusée")
        };
        assert_eq!(refus.raison, Raison::Disque);
        // Le stockage joint les composants du chemin par `/`, sous Windows aussi (`chemin_dit`), puis
        // dit le constat du disque (`constat_du_disque`), qui suit le genre de l'erreur : « , le
        // disque l'a refusé » pour `AlreadyExists`, attendu ici, « est absent » ou « est refusé par
        // le système » si le système en rend un autre ; le test exige le chemin entier en tête, non
        // le genre.
        let dit = racine.display().to_string().replace('\\', "/");
        assert!(
            refus.message.starts_with(&format!("{dit} "))
                || refus.message.starts_with(&format!("{dit},")),
            "{}",
            refus.message
        );
    }

    #[test]
    fn deux_demandes_concurrentes_sont_servies_l_une_apres_l_autre() {
        let racine = dossier_jetable("fil-concurrence").join("donnees");
        let ouverture = std::sync::Arc::new(ouvrir(racine));
        let fils: Vec<_> = (0..2)
            .map(|n| {
                let ouverture = ouverture.clone();
                std::thread::spawn(move || {
                    (0..10).all(|i| {
                        ouverture
                            .demander_bloquant(noter_une(&format!("Idée {n}-{i}")))
                            .is_ok()
                    })
                })
            })
            .collect();
        assert!(fils.into_iter().all(|fil| fil.join().unwrap()));
        assert_eq!(
            ouverture
                .demander_bloquant(|services, _| services.idees())
                .unwrap()
                .len(),
            20
        );
    }

    #[test]
    fn une_demande_qui_panique_laisse_un_refus_et_non_une_attente() {
        let racine = dossier_jetable("fil-panique").join("donnees");
        let ouverture = ouvrir(racine);
        let refus = ouverture
            .demander_bloquant(|_, _| -> Reponse<()> { panic!("un défaut planté") })
            .unwrap_err();
        assert_eq!(refus.message, "Le fonds ne répond plus.");
        assert_eq!(
            ouverture
                .demander_bloquant(noter_une("Une idée"))
                .unwrap_err(),
            refus
        );
    }
}
