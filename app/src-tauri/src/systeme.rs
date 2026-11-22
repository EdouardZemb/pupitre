//! Le système : l'heure, l'aléa, le dossier de l'application et la place libre du disque, les
//! seuls que l'établi lit ; et le dossier choisi, admis s'il n'est pas celui de l'application.

use std::fs;
use std::io;
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

/// Le dossier de l'application, que `run()` garde dans l'état de Tauri.
pub struct DossierDeLApplication(pub PathBuf);

/// La place offerte à l'appelant sur le volume de `dossier` (question 8 ; P10) : sous Windows,
/// `GetDiskFreeSpaceExW`, déclaré dans un bloc `unsafe extern "system"`, le seul du dépôt, sur le
/// chemin canonique de `dossier` (`fs::canonicalize`, qui rend un `\\?\`), en UTF-16 fini par un
/// zéro, `lpFreeBytesAvailableToCaller` seul lu ; un échec rend l'erreur du système. Hors de
/// Windows : `io::ErrorKind::Unsupported`, une branche qu'aucun job ne compile, l'établi n'ayant
/// que Windows pour cible (balayage Q12).
#[cfg(windows)]
pub fn place_libre(dossier: &Path) -> io::Result<u64> {
    use std::os::windows::ffi::OsStrExt;
    let canonique = fs::canonicalize(dossier)?;
    let nom: Vec<u16> = canonique
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let (mut offerte, mut totale, mut libre) = (0u64, 0u64, 0u64);
    // SAFETY : `nom` est un chemin en UTF-16 fini par un zéro, qui vit jusqu'au retour de l'appel ;
    // les trois sorties sont des `u64` à nous, chacun de la taille et de l'alignement d'un
    // `ULARGE_INTEGER`, que l'appel écrit sans en garder l'adresse ; il ne lit rien d'autre.
    let fait = unsafe { GetDiskFreeSpaceExW(nom.as_ptr(), &mut offerte, &mut totale, &mut libre) };
    if fait == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(offerte)
}

/// Hors de Windows, la place libre ne se lit pas : `io::ErrorKind::Unsupported`. Aucun job ne
/// compile cette branche, l'établi n'ayant que Windows pour cible (balayage Q12).
#[cfg(not(windows))]
pub fn place_libre(_dossier: &Path) -> io::Result<u64> {
    Err(io::ErrorKind::Unsupported.into())
}

/// L'appel de `kernel32` que l'établi déclare à la main, sans dépendance.
#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    /// `GetDiskFreeSpaceExW` : sur le volume du dossier `dossier`, la place offerte à l'appelant
    /// (`lpFreeBytesAvailableToCaller`, quotas compris), la taille offerte à l'appelant
    /// (`lpTotalNumberOfBytes`) et la place libre du volume (`lpTotalNumberOfFreeBytes`) ; 0 en
    /// cas d'échec, l'erreur dans `GetLastError`.
    fn GetDiskFreeSpaceExW(
        dossier: *const u16,
        offerte: *mut u64,
        totale: *mut u64,
        libre: *mut u64,
    ) -> i32;
}

/// Le refus d'un dossier qui est celui de l'application, le contient ou s'y trouve.
const DANS_PUPITRE: &str =
    "Ce dossier est celui de Pupitre, le contient ou s'y trouve : il ne se verse pas.";

/// Admet le dossier choisi (question 14) : refusé s'il est le dossier de l'application, le contient
/// ou s'y trouve, leurs chemins canoniques comparés par segments (`Path::starts_with`), « Ce
/// dossier est celui de Pupitre, le contient ou s'y trouve : il ne se verse pas. » ; rend son
/// chemin, tel que choisi. Un chemin qui ne se rend pas canonique (`fs::canonicalize`) : « Le
/// dossier choisi ne se lit pas : <erreur>. », ou « Le dossier de Pupitre ne se lit pas :
/// <erreur>. ».
pub fn admettre_le_dossier(choisi: &Path, application: &Path) -> Result<PathBuf, String> {
    let canonique = fs::canonicalize(choisi)
        .map_err(|erreur| format!("Le dossier choisi ne se lit pas : {erreur}."))?;
    let de_l_application = fs::canonicalize(application)
        .map_err(|erreur| format!("Le dossier de Pupitre ne se lit pas : {erreur}."))?;
    if canonique.starts_with(&de_l_application) || de_l_application.starts_with(&canonique) {
        return Err(DANS_PUPITRE.to_string());
    }
    Ok(choisi.to_path_buf())
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
