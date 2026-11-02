//! L'établi : l'enveloppe Tauri de l'interface, et l'export du cahier des tables.

use std::path::Path;

use noyau::rendu::Instrument;
use noyau::rendu::pages::Date;
#[cfg(desktop)]
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

/// L'instrument que l'interface nomme : « guitare », « clavier » ou « les-deux ».
fn lire_instrument(s: &str) -> Option<Instrument> {
    match s {
        "guitare" => Some(Instrument::Guitare),
        "clavier" => Some(Instrument::Clavier),
        "les-deux" => Some(Instrument::LesDeux),
        _ => None,
    }
}

/// Le nom que la boîte d'enregistrement propose : `cahier-des-tables-AAAA-MM-JJ.pdf`.
fn nom_propose(d: &Date) -> String {
    format!(
        "cahier-des-tables-{:04}-{:02}-{:02}.pdf",
        d.annee, d.mois, d.jour
    )
}

/// Écrit le cahier à l'endroit choisi, et rend le nom du fichier écrit. Sans endroit, la boîte
/// a été fermée : rien ne s'engendre, rien ne s'écrit, et rien n'est à dire (vigilance 5). Un
/// cahier qui ne s'engendre pas, ou un fichier qui ne s'écrit pas, se disent en français, en
/// nommant le fichier.
fn ecrire_le_cahier(
    chemin: Option<&Path>,
    engendrer: impl FnOnce() -> Result<Vec<u8>, String>,
) -> Result<Option<String>, String> {
    let Some(chemin) = chemin else {
        return Ok(None);
    };
    let nom = chemin.file_name().map_or_else(
        || chemin.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    let pdf = engendrer().map_err(|e| format!("{nom} ne s'engendre pas : {e}"))?;
    std::fs::write(chemin, pdf).map_err(|e| format!("{nom} ne s'écrit pas : {e}"))?;
    Ok(Some(nom))
}

/// Exporte le cahier des tables : la boîte d'enregistrement propose
/// `cahier-des-tables-AAAA-MM-JJ.pdf`, puis le cahier s'engendre pour cet instrument, daté du
/// jour que l'interface donne (I25), et s'écrit. `Ok(None)` : la boîte a été fermée, rien n'est
/// écrit ; `Ok(Some(nom du fichier))` ; `Err(message en français)`.
#[tauri::command]
async fn exporter_cahier(
    app: tauri::AppHandle,
    date: String,
    instrument: String,
) -> Result<Option<String>, String> {
    let i = lire_instrument(&instrument).ok_or_else(|| {
        format!("instrument inconnu : « {instrument} », attendu guitare, clavier ou les-deux")
    })?;
    let d = Date::depuis_iso(&date)
        .ok_or_else(|| format!("date invalide : « {date} », attendue sous la forme AAAA-MM-JJ"))?;
    // La boîte attend sa réponse, et le cahier prend quelques secondes : l'un et l'autre hors du
    // fil principal, sur un fil qui peut attendre.
    tauri::async_runtime::spawn_blocking(move || {
        let boite = app
            .dialog()
            .file()
            .set_title("Exporter le cahier des tables")
            .set_file_name(nom_propose(&d))
            .add_filter("PDF", &["pdf"]);
        #[cfg(desktop)]
        let boite = match app.get_webview_window("main") {
            Some(fenetre) => boite.set_parent(&fenetre),
            None => boite,
        };
        let chemin = boite
            .blocking_save_file()
            .map(|c| c.into_path())
            .transpose()
            .map_err(|e| format!("l'emplacement choisi ne se lit pas : {e}"))?;
        ecrire_le_cahier(chemin.as_deref(), || {
            cahier::engendrer(i, &d).map_err(|e| e.to_string())
        })
    })
    .await
    .map_err(|e| format!("l'export s'est interrompu : {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![exporter_cahier])
        .run(tauri::generate_context!())
        .expect("l'établi ne démarre pas");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn lire_les_instruments() {
        assert_eq!(lire_instrument("guitare"), Some(Instrument::Guitare));
        assert_eq!(lire_instrument("clavier"), Some(Instrument::Clavier));
        assert_eq!(lire_instrument("les-deux"), Some(Instrument::LesDeux));
        assert_eq!(lire_instrument("piano"), None);
    }

    #[test]
    fn le_nom_propose_du_fichier() {
        let date = Date::depuis_iso("2026-11-01").expect("une date valide");
        assert_eq!(nom_propose(&date), "cahier-des-tables-2026-11-01.pdf");
    }

    /// Un dossier à soi dans le dossier temporaire, vide.
    fn dossier(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("etabli-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("le dossier temporaire se crée");
        d
    }

    #[test]
    fn la_boite_fermee_n_engendre_ni_n_ecrit_rien() {
        // Vigilance 5 : rien d'écrit, et rien à dire.
        let r = ecrire_le_cahier(None, || panic!("rien ne s'engendre"));
        assert_eq!(r, Ok(None));
    }

    #[test]
    fn le_cahier_s_ecrit_et_l_export_rend_le_nom_du_fichier() {
        let d = dossier("ecrit");
        let chemin = d.join("cahier-des-tables-2026-11-01.pdf");
        let r = ecrire_le_cahier(Some(&chemin), || Ok(b"%PDF-1.7".to_vec()));
        assert_eq!(r, Ok(Some("cahier-des-tables-2026-11-01.pdf".to_string())));
        assert_eq!(
            std::fs::read(&chemin).expect("le fichier est écrit"),
            b"%PDF-1.7"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn une_ecriture_qui_echoue_se_dit_en_francais_en_nommant_le_fichier() {
        // Vigilance 5 : dans un dossier qui n'existe pas, le fichier ne s'écrit pas.
        let d = dossier("refus");
        let chemin = d.join("absent").join("cahier-des-tables-2026-11-01.pdf");
        let r = ecrire_le_cahier(Some(&chemin), || Ok(b"%PDF-1.7".to_vec()));
        let e = r.expect_err("l'écriture échoue");
        assert!(
            e.starts_with("cahier-des-tables-2026-11-01.pdf ne s'écrit pas : "),
            "{e}"
        );
        assert!(!chemin.exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn un_cahier_qui_ne_s_engendre_pas_se_dit_et_rien_ne_s_ecrit() {
        let d = dossier("engendre");
        let chemin = d.join("cahier.pdf");
        let r = ecrire_le_cahier(Some(&chemin), || {
            Err("la page ne tient pas : la table de mi♭ mineur".to_string())
        });
        assert_eq!(
            r,
            Err(
                "cahier.pdf ne s'engendre pas : la page ne tient pas : la table de mi♭ mineur"
                    .to_string()
            )
        );
        assert!(!chemin.exists());
        let _ = std::fs::remove_dir_all(&d);
    }
}
