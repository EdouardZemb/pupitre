//! Le bandeau de l'établi (I29 ; question 31) : l'état du fonds, le refus d'un geste et les
//! avertissements, dits en tête de l'établi, chacun par un mot en tête, jamais par la seule
//! couleur ; et la page d'ouverture, l'établi ou le seul bandeau (question 4).

#[cfg(test)]
mod tests {
    use super::*;
    use commandes::Raison;

    fn refus(raison: Raison, message: &str) -> Refus {
        Refus {
            raison,
            message: message.to_string(),
        }
    }

    /// Un état du fonds né, sans geste à annuler.
    fn etat(lecture_seule: Option<Refus>, avertissements: &[&str]) -> EtatDuFonds {
        EtatDuFonds {
            ne: true,
            lecture_seule,
            avertissements: avertissements.iter().map(|a| a.to_string()).collect(),
            a_annuler: None,
        }
    }

    fn message(source: Source, texte: &str, fermable: bool) -> MessageDuBandeau {
        MessageDuBandeau {
            source,
            texte: texte.to_string(),
            fermable,
        }
    }

    #[test]
    fn i29_une_ecriture_refusee_s_affiche_en_bandeau() {
        let texte = "Le disque a refusé l'écriture du fonds : rien n'a changé.";
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Err(refus(Raison::Disque, texte)));
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Refus, texte, true)]
        );
    }

    #[test]
    fn un_refus_reste_jusqu_au_geste_reussi_suivant() {
        let texte = "Le disque a refusé l'écriture du fonds : rien n'a changé.";
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Err(refus(Raison::Disque, texte)));
        // Une lecture réussie de l'état du fonds ne l'efface pas.
        bandeau.suivre(&etat(None, &[]), false);
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Refus, texte, true)]
        );
        bandeau.apres_un_geste(&Ok(None));
        assert_eq!(bandeau.messages(), Vec::new());
        // « Fermer » l'efface aussi, comme les avertissements.
        bandeau.apres_un_geste(&Ok(Some("Un avertissement inventé.".to_string())));
        bandeau.apres_un_geste(&Err(refus(Raison::Disque, texte)));
        bandeau.fermer(Source::Refus);
        assert_eq!(
            bandeau.messages(),
            vec![message(
                Source::Avertissement,
                "Un avertissement inventé.",
                true
            )]
        );
        bandeau.fermer(Source::Avertissement);
        assert_eq!(bandeau.messages(), Vec::new());
    }

    #[test]
    fn un_etat_ne_se_ferme_pas() {
        let illisible = "Un fichier du fonds est altéré : le fonds se lit sans écrire.";
        let mut bandeau = Bandeau::default();
        bandeau.suivre(&etat(Some(refus(Raison::Illisible, illisible)), &[]), true);
        bandeau.fermer(Source::Etat);
        // Ni « Fermer » ni un geste réussi ne l'effacent.
        bandeau.apres_un_geste(&Ok(None));
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Etat, illisible, false)]
        );

        let plus_recente =
            "Le fonds porte une version de schéma plus récente : il se lit sans écrire.";
        let mut bandeau = Bandeau::default();
        bandeau.suivre(
            &etat(
                Some(refus(
                    Raison::VersionPlusRecente { version: 4 },
                    plus_recente,
                )),
                &[],
            ),
            false,
        );
        bandeau.fermer(Source::Etat);
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Etat, plus_recente, false)]
        );
        // Le fonds qui écrit de nouveau : l'état s'efface.
        bandeau.suivre(&etat(None, &[]), false);
        assert_eq!(bandeau.messages(), Vec::new());
    }

    #[test]
    fn les_etats_passent_avant_les_refus() {
        let mut bandeau = Bandeau::default();
        // À l'ouverture, l'état et deux avertissements ; puis un refus. Les avertissements font
        // un seul message, un par source, et viennent après le refus.
        bandeau.suivre(
            &etat(
                Some(refus(Raison::Illisible, "Un état inventé.")),
                &[
                    "Un avertissement inventé.",
                    "Un second avertissement inventé.",
                ],
            ),
            true,
        );
        bandeau.apres_un_geste(&Err(refus(Raison::Impossible, "Un refus inventé.")));
        assert_eq!(
            bandeau.messages(),
            vec![
                message(Source::Etat, "Un état inventé.", false),
                message(Source::Refus, "Un refus inventé.", true),
                message(
                    Source::Avertissement,
                    "Un avertissement inventé. Un second avertissement inventé.",
                    true
                ),
            ]
        );
    }

    #[test]
    fn l_avertissement_de_la_naissance_ne_parait_qu_une_fois() {
        // L'avertissement d'une naissance non durable revient dans l'issue du geste et dans
        // l'état du fonds relu après la commande : le bandeau ne le dit qu'une fois.
        let t = "Le fonds est né, mais le disque n'a pas confirmé son écriture.";
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Ok(Some(t.to_string())));
        bandeau.suivre(&etat(None, &[t]), false);
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Avertissement, t, true)]
        );
    }

    #[test]
    fn chaque_source_a_son_mot_et_son_role() {
        // La source se dit par un mot, jamais par la seule couleur ; un refus est une alerte.
        assert_eq!(Source::Etat.mot(), "Lecture seule :");
        assert_eq!(Source::Refus.mot(), "Refusé :");
        assert_eq!(Source::Avertissement.mot(), "Avertissement :");
        assert_eq!(Source::Etat.role(), "status");
        assert_eq!(Source::Refus.role(), "alert");
        assert_eq!(Source::Avertissement.role(), "status");
    }

    #[test]
    fn la_page_d_ouverture_d_une_racine_tenue_n_a_que_le_bandeau() {
        let tenue = "Un autre processus tient la racine des données.";
        assert_eq!(
            page_d_ouverture(&Err(refus(Raison::Impossible, tenue)), true),
            Page::BandeauSeul(tenue.to_string())
        );
        assert_eq!(page_d_ouverture(&Ok(etat(None, &[])), true), Page::Etabli);
        // Un fonds en lecture seule ouvre l'établi : le bandeau le dit.
        assert_eq!(
            page_d_ouverture(
                &Ok(etat(
                    Some(refus(Raison::Illisible, "Un état inventé.")),
                    &[]
                )),
                true
            ),
            Page::Etabli
        );
        let hors = "Hors de l'application, aucune commande ne passe.";
        assert_eq!(
            page_d_ouverture(&Err(refus(Raison::Impossible, hors)), false),
            Page::Etabli
        );
    }
}
