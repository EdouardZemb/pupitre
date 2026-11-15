//! Le pont natif de l'interface : les commandes de l'établi, par l'IPC de Tauri (décision 11), et
//! le décalage local d'un geste ou d'un instant.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_rejet_qui_n_est_pas_un_refus_se_dit_par_un_constat() {
        let constat = Refus {
            raison: Raison::Impossible,
            message: "Le geste n'a pas abouti.".to_string(),
        };
        // Le refus des droits de Tauri, une erreur JavaScript (`{}` en JSON), aucun rejet lisible.
        assert_eq!(
            refus_du_rejet(Some("\"Command noter not allowed by ACL\"".into())),
            constat
        );
        assert_eq!(refus_du_rejet(Some("{}".into())), constat);
        assert_eq!(refus_du_rejet(None), constat);
        // Le JSON d'un refus : ce refus, tel quel.
        let refus = Refus {
            raison: Raison::VersionPlusRecente { version: 4 },
            message: "Le fonds porte une version de schéma plus récente.".to_string(),
        };
        let json = serde_json::to_string(&refus).expect("un refus s'écrit en JSON");
        assert_eq!(refus_du_rejet(Some(json)), refus);
    }

    #[test]
    fn le_decalage_de_javascript_change_de_signe() {
        // `getTimezoneOffset` vaut UTC moins l'heure locale : -60 pour une heure locale en avance
        // d'une heure sur UTC, dont le décalage est de 60 minutes.
        assert_eq!(decalage_depuis_js(-60.0), 60);
        assert_eq!(decalage_depuis_js(-120.0), 120);
        assert_eq!(decalage_depuis_js(300.0), -300);
        assert_eq!(decalage_depuis_js(0.0), 0);
    }
}
