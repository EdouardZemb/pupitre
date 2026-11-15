//! Le système : l'heure, l'aléa et le dossier de l'application, les seuls que l'établi lit.

#[cfg(test)]
mod tests {
    use super::*;
    use services::Alea;

    #[test]
    fn l_heure_du_systeme_suit_l_ecriture_de_ce_plan() {
        // 1 794 700 800 : le 2026-11-15 à 0 h UTC ; 4 102 444 800 : le 2100-01-01.
        let h = heure();
        assert!(h > 1_794_700_800 && h < 4_102_444_800, "{h}");
    }

    #[test]
    fn l_alea_du_systeme_remplit_ses_octets() {
        let (mut a, mut b) = ([0u8; 32], [0u8; 32]);
        AleaDuSysteme.remplir(&mut a);
        AleaDuSysteme.remplir(&mut b);
        assert_ne!(a, [0u8; 32]);
        assert_ne!(a, b);
    }

    #[test]
    fn la_racine_des_donnees_est_donnees_sous_le_dossier_recu() {
        let dossier = crate::tests::dossier_jetable("racine-des-donnees");
        assert_eq!(racine_des_donnees(&dossier), dossier.join("donnees"));
    }
}
