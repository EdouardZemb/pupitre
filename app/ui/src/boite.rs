//! La boîte d'idées à l'écran (CAP-02 ; questions 25 à 30).

#[cfg(test)]
mod tests {
    use super::*;
    use commandes::{Appareil, Provenance};

    /// Une ligne inventée, captée sur l'établi, sans durée ni lien.
    fn ligne(texte: Option<&str>, etiquettes: &[&str]) -> LigneDeLaBoite {
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
        }
    }

    #[test]
    fn la_fenetre_borne_les_lignes_rendues() {
        // Deux mille idées, des lignes de 40 px, une vue de 800 px : vingt lignes vues.
        let a = |defilement_px| fenetre(defilement_px, 800.0, 40.0, 2_000);
        assert_eq!(
            a(0.0),
            Fenetre {
                debut: 0,
                nombre: 25
            }
        );
        assert_eq!(
            a(40_000.0),
            Fenetre {
                debut: 995,
                nombre: 30
            }
        );
        assert_eq!(
            a(79_200.0),
            Fenetre {
                debut: 1_975,
                nombre: 25
            }
        );
        assert_eq!(a(200_000.0).nombre, 0);
        // Jamais plus de trente lignes : les vingt vues, plus deux marges.
        for defilement_px in (0..=80_000).step_by(7) {
            let rendues = a(f64::from(defilement_px)).nombre;
            assert!(
                rendues <= 20 + 2 * MARGE,
                "{rendues} lignes à {defilement_px} px"
            );
        }
        // Une vue plus haute que `FENETRE_MAX` lignes : la fenêtre s'y borne.
        assert_eq!(
            fenetre(0.0, 1_000_000.0, 40.0, 1_000_000).nombre,
            FENETRE_MAX
        );
        // Sans hauteur de ligne lisible : les `FENETRE_MAX` premières lignes, toutes s'il y en a
        // moins.
        assert_eq!(
            fenetre(40_000.0, 800.0, 0.0, 2_000),
            Fenetre {
                debut: 0,
                nombre: FENETRE_MAX
            }
        );
        assert_eq!(
            fenetre(0.0, 800.0, f64::NAN, 12),
            Fenetre {
                debut: 0,
                nombre: 12
            }
        );
    }

    #[test]
    fn la_fenetre_demandee_couvre_les_lignes_vues() {
        let demandee = fenetre(40_000.0, 800.0, 40.0, 2_000);
        let couverte = |defilement_px| couvre(&demandee, defilement_px, 800.0, 40.0, 2_000);
        // La fenêtre de 995 à 1 025 porte les lignes vues, de 1 000 à 1 020, et cinq lignes plus
        // bas encore, de 1 005 à 1 025.
        assert!(couverte(40_000.0));
        assert!(couverte(40_200.0));
        // Un demi-rang de plus, la ligne 1 025 paraît : le défilement demande une autre fenêtre.
        assert!(!couverte(40_220.0));
        // Six lignes plus haut, la ligne 994 paraît.
        assert!(!couverte(39_760.0));
        // Au bout de la liste, les lignes vues s'arrêtent à la dernière.
        let au_bout = fenetre(79_200.0, 800.0, 40.0, 2_000);
        assert!(couvre(&au_bout, 79_200.0, 800.0, 40.0, 2_000));
        // Sans hauteur de ligne lisible, rien n'est couvert.
        assert!(!couvre(&demandee, 40_000.0, 800.0, 0.0, 2_000));
    }

    #[test]
    fn ctrl_z_n_annule_un_geste_que_hors_d_un_champ() {
        assert!(ctrl_z_annule("z", true, false));
        // `Ctrl+Maj+Z` donne « Z ».
        assert!(ctrl_z_annule("Z", true, false));
        assert!(!ctrl_z_annule("z", true, true));
        assert!(!ctrl_z_annule("z", false, false));
        assert!(!ctrl_z_annule("y", true, false));
    }

    #[test]
    fn le_bouton_d_annulation_dit_le_geste_vise() {
        assert_eq!(
            libelle_d_annulation(Some("Idée du 3 novembre : étiquette #pont posée")),
            "Annuler : Idée du 3 novembre : étiquette #pont posée"
        );
        assert_eq!(libelle_d_annulation(None), "Rien à annuler");
    }

    #[test]
    fn cap01_4_noter_se_fait_en_un_geste_sans_titre() {
        // Une part de CAP-01 · 4 : à l'établi, `Entrée` dans le champ note l'idée, sans demande de
        // titre ; la capture du carnet vient au lot 2.
        assert_eq!(
            note_a_envoyer("Une idée de pont", "Enter", 60),
            Some(Noter {
                texte: "Une idée de pont".to_string(),
                decalage_minutes: 60
            })
        );
        assert_eq!(note_a_envoyer("Une idée de pont", "a", 60), None);
        assert_eq!(note_a_envoyer("Une idée de pont", "Tab", 60), None);
        assert_eq!(note_a_envoyer(" \t  ", "Enter", 60), None);
        assert_eq!(note_a_envoyer("", "Enter", 60), None);
        // La note part sans ses blancs autour, avec le décalage du geste.
        assert_eq!(
            note_a_envoyer("  Une idée de pont ", "Enter", -300),
            Some(Noter {
                texte: "Une idée de pont".to_string(),
                decalage_minutes: -300
            })
        );
    }

    #[test]
    fn un_texte_piege_se_montre_tel_quel() {
        let piege = "<img src=x onerror=alert(1)>";
        assert_eq!(texte_de_la_ligne(&ligne(Some(piege), &[])), piege);
    }

    #[test]
    fn une_case_vide_s_ecrit_d_un_tiret_cadratin() {
        let vide = ligne(None, &[]);
        assert_eq!(texte_de_la_ligne(&vide), "—");
        assert_eq!(etiquettes_de_la_ligne(&vide), "—");
        assert_eq!(texte_de_la_ligne(&ligne(Some(""), &[])), "—");
        assert_eq!(
            etiquettes_de_la_ligne(&ligne(None, &["pont", "refrain"])),
            "#pont #refrain"
        );
    }

    #[test]
    fn l_etat_et_la_provenance_se_disent_en_mots() {
        // Aucune information par la seule couleur : l'état et la provenance sont des mots.
        let mut l = ligne(Some("Une idée de pont"), &[]);
        assert_eq!(etat_de_la_ligne(&l), "non reliée");
        assert_eq!(provenance_de_la_ligne(&l), "captée");
        l.reliee = true;
        l.provenance = Provenance::Importee {
            par: Appareil::Etabli,
            depuis: "import-invente".to_string(),
        };
        assert_eq!(etat_de_la_ligne(&l), "reliée");
        assert_eq!(provenance_de_la_ligne(&l), "importée");
    }

    #[test]
    fn l_etalon_de_la_feuille_de_style_mesure_cent_lignes() {
        let css = include_str!("../style.css");
        assert!(
            css.contains("--ligne: 2.5rem;"),
            "la hauteur d'une ligne manque à la feuille de style"
        );
        let etalon = format!("height: calc({LIGNES_DE_L_ETALON} * var(--ligne));");
        assert!(
            css.contains(&etalon),
            "{etalon} manque à la feuille de style"
        );
    }
}
