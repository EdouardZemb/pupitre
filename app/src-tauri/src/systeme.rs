//! Le système : l'heure, l'aléa et le dossier de l'application, les seuls que l'établi lit.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::Manager;

use crate::fil::{self, Ouverture};

/// L'heure du système, en secondes depuis 1970 (`SystemTime`) ; une horloge d'avant 1970 compte
/// pour 0.
pub fn heure() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_secs()).unwrap_or(i64::MAX),
        Err(_) => 0,
    }
}

/// L'aléa du système, par `getrandom::fill`. Un échec panique : sur le fil du fonds, seul à le
/// tirer, le fil tombe, la racine des données et son verrou avec lui, et la commande en cours,
/// puis chacune des suivantes, rendent « Le fonds ne répond plus. » ; l'aléa ne se remplace pas,
/// et `Alea::remplir` ne rend rien.
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

/// Ouvre `donnees/` sous le dossier de l'application reçu, par le fil du fonds.
pub fn ouvrir(dossier: &Path) -> Ouverture {
    fil::ouvrir(racine_des_donnees(dossier))
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

    /// Le refus d'un dossier que `admettre_le_dossier` ne verse pas.
    const DANS_PUPITRE: &str =
        "Ce dossier est celui de Pupitre, le contient ou s'y trouve : il ne se verse pas.";

    #[cfg(windows)]
    #[test]
    fn la_place_libre_se_lit_sur_un_chemin_long() {
        // GetDiskFreeSpaceExW refuse un chemin de plus de 260 unités sans `\\?\` : le chemin
        // canonique le porte.
        let mut long = crate::tests::dossier_jetable("place-libre-long");
        let mut etage = 0;
        while long.to_string_lossy().encode_utf16().count() <= 300 {
            etage += 1;
            long.push(format!("étage-{etage}-{}", "ô".repeat(40)));
        }
        std::fs::create_dir_all(&long).unwrap_or_else(|e| panic!("créer {} : {e}", long.display()));
        let place = place_libre(&long)
            .unwrap_or_else(|e| panic!("la place libre de {} : {e}", long.display()));
        assert!(
            place > 0,
            "aucune place offerte sur le volume du dossier jetable"
        );
    }

    #[cfg(windows)]
    #[test]
    fn la_place_libre_d_un_dossier_absent_est_refusee() {
        let absent = crate::tests::dossier_jetable("place-libre-absent").join("absent");
        assert!(
            place_libre(&absent).is_err(),
            "un dossier absent rend une place"
        );
    }

    #[test]
    fn un_dossier_dans_celui_de_l_application_est_refuse() {
        let jetable = crate::tests::dossier_jetable("dossier-admis");
        let application = jetable.join("app");
        let dedans = application.join("x");
        let autre = jetable.join("autre");
        // Un voisin dont le nom commence par celui de l'application : les segments se comparent
        // entiers.
        let voisin = jetable.join("app-voisin");
        for dossier in [&dedans, &autre, &voisin] {
            std::fs::create_dir_all(dossier)
                .unwrap_or_else(|e| panic!("créer {} : {e}", dossier.display()));
        }
        // Le dossier de l'application, un dossier qui s'y trouve, sous une forme qui n'est pas
        // canonique aussi, et le dossier qui le contient.
        for refuse in [&application, &dedans, &dedans.join(".."), &jetable] {
            assert_eq!(
                admettre_le_dossier(refuse, &application),
                Err(DANS_PUPITRE.to_string()),
                "{}",
                refuse.display()
            );
        }
        for admis in [&autre, &voisin] {
            assert_eq!(
                admettre_le_dossier(admis, &application),
                Ok(admis.clone()),
                "{}",
                admis.display()
            );
        }
    }

    /// L'identifiant de l'application, lu dans `tauri.conf.json`.
    fn identifiant_de_tauri_conf() -> String {
        let conf: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tauri.conf.json"
        )))
        .expect("tauri.conf.json est du JSON");
        conf["identifier"]
            .as_str()
            .expect("tauri.conf.json porte un identifiant")
            .to_string()
    }

    #[test]
    #[ignore = "le garde-fou d'I30 : l'étape « La sentinelle d'I30 » du job windows le lance"]
    fn i30_le_dossier_que_tauri_resout_tombe_dans_la_sentinelle() {
        let sentinelle = std::env::var("PUPITRE_SENTINELLE").expect(
            "PUPITRE_SENTINELLE nomme la sentinelle : le garde-fou ne tourne que sous elle",
        );
        let identifiant = identifiant_de_tauri_conf();
        assert_eq!(identifiant, "app.pupitre.etabli");
        let mut contexte = tauri::test::mock_context(tauri::test::noop_assets());
        contexte.config_mut().identifier = identifiant.clone();
        let app = tauri::test::mock_builder().build(contexte).unwrap();
        let dossier = dossier_de_l_application(app.handle()).unwrap();
        assert!(
            dossier.starts_with(&sentinelle),
            "le dossier résolu n'est pas dans la sentinelle"
        );
        assert!(
            dossier.ends_with(&identifiant),
            "le dossier résolu n'est pas celui de l'application"
        );
    }
}
