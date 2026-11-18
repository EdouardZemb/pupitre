//! La requête de l'écran et ses touches, en fonctions pures (CAP-06 ; questions 24 et 25).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_porte_le_focus_hors_d_un_champ_seulement() {
        assert!(slash_cherche("/", false, false, false, false, false, false)); // AZERTY : Maj + :
        assert!(!slash_cherche("/", false, false, false, false, false, true)); // dans un champ, elle s'écrit
        assert!(!slash_cherche("/", true, false, false, false, false, false));
        assert!(!slash_cherche("/", false, true, false, false, false, false));
        assert!(!slash_cherche("/", true, true, false, false, false, false)); // AltGr
        assert!(!slash_cherche("/", false, false, true, false, false, false));
        assert!(!slash_cherche("/", false, false, false, true, false, false)); // tenue
        assert!(!slash_cherche("/", false, false, false, false, true, false)); // en composition
        assert!(!slash_cherche(
            "!", false, false, false, false, false, false
        ));
        assert!(!slash_cherche(
            ":", false, false, false, false, false, false
        ));
    }

    #[test]
    fn echap_ne_ferme_que_le_volet_ouvert() {
        assert!(echap_ferme("Escape", false, true, false));
        assert!(!echap_ferme("Escape", false, false, false));
        assert!(!echap_ferme("Escape", true, true, false));
        assert!(!echap_ferme("Esc", false, true, false));
        assert!(!echap_ferme("Escape", false, true, true)); // dans la recherche, Échap vide le champ
        assert!(est_la_recherche(Some(ID_DE_LA_RECHERCHE)));
        assert!(!est_la_recherche(Some("note")));
        assert!(!est_la_recherche(Some("")));
        assert!(!est_la_recherche(None));
    }

    #[test]
    fn un_champ_de_date_vide_ne_borne_pas() {
        assert_eq!(jour_du_champ(""), None);
        assert_eq!(jour_du_champ("2026-10"), None);
        assert_eq!(
            jour_du_champ("2026-10-25"),
            Some(Jour {
                annee: 2026,
                mois: 10,
                jour: 25
            })
        );
    }

    #[test]
    fn les_bornes_de_la_requete_se_tirent_de_ses_jours() {
        let paris = |t: i64| {
            if (1_774_746_000..1_792_890_000).contains(&t) {
                120
            } else {
                60
            }
        };
        let jour = jour_du_champ("2026-10-25");
        let requete = Requete {
            du: jour.clone(),
            au: jour,
            ..Requete::default()
        };
        assert_eq!(
            bornes_de_la_requete(&requete, paris),
            Bornes {
                des: Some(1_792_879_200),
                avant: Some(1_792_969_200)
            }
        );
        assert_eq!(
            bornes_de_la_requete(&Requete::default(), paris),
            Bornes::default()
        );
    }

    #[test]
    fn la_requete_de_l_ecran_suit_ses_reglages_et_s_y_repose() {
        let owner = Owner::new();
        owner.set();
        let etat = EtatDeLaRequete::nouveau();
        assert_eq!(etat.requete(), Requete::default());
        let requete = Requete {
            texte: "eglise".into(),
            du: jour_du_champ("2026-10-25"),
            etiquette: Some("pont".into()),
            plus_de_secondes: Some(60),
            etat: Some(Etat::NonReliee),
            provenance: Some(ProvenanceDeLIdee::Importee),
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        };
        etat.poser(&requete);
        assert_eq!(etat.requete(), requete);
        etat.poser(&Requete {
            plus_de_secondes: Some(45),
            ..Requete::default()
        });
        assert_eq!(etat.requete().plus_de_secondes, None);
    }
}
