//! Le système : l'heure, l'aléa et le dossier de l'application, les seuls que l'établi lit.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::Manager;

/// L'heure du système, en secondes depuis 1970 (`SystemTime`) ; une horloge d'avant 1970 compte
/// pour 0.
pub fn heure() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

/// L'aléa du système, par `getrandom::fill`. Un échec arrête l'application : l'aléa ne se
/// remplace pas, et `Alea::remplir` ne rend rien.
pub struct AleaDuSysteme;

impl services::Alea for AleaDuSysteme {
    fn remplir(&mut self, octets: &mut [u8]) {
        getrandom::fill(octets).expect("Le système n'a pas donné d'aléa.");
    }
}

/// Le dossier local de l'application, `app_local_data_dir`, jamais `app_data_dir` : le seul appel
/// du résolveur de Tauri. Seuls `run()` et le garde-fou d'I30 l'appellent.
pub fn dossier_de_l_application<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> tauri::Result<PathBuf> {
    app.path().app_local_data_dir()
}

/// La racine des données, `donnees/`, sous le dossier de l'application reçu.
pub fn racine_des_donnees(dossier: &Path) -> PathBuf {
    dossier.join("donnees")
}

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
