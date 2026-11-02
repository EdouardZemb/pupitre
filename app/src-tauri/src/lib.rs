//! L'établi : l'enveloppe Tauri de l'interface, et l'export du cahier des tables.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use noyau::rendu::Instrument;
use noyau::rendu::pages::Date;
use noyau::table::CoucheSombre;
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

/// La couche sombre que l'interface nomme : « masquee » ou « affichee ».
fn lire_couche(s: &str) -> Option<CoucheSombre> {
    match s {
        "masquee" => Some(CoucheSombre::Masquee),
        "affichee" => Some(CoucheSombre::Affichee),
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
/// a été fermée : rien ne s'engendre, rien ne s'écrit, et rien n'est à dire (vigilance 5).
///
/// Le cahier s'écrit d'abord dans un fichier voisin, `<nom>.<processus>.partiel`, qui prend
/// ensuite la place du fichier choisi : un cahier qui existait reste entier jusqu'au dernier
/// moment, et le reste si l'export échoue. Le voisin se crée avant que le cahier ne s'engendre :
/// un dossier où l'on ne peut écrire se dit tout de suite. En cas d'échec, le voisin est retiré,
/// et l'erreur se dit en français, en nommant le fichier.
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
    let ne_s_ecrit_pas = |e: std::io::Error| format!("{nom} ne s'écrit pas : {e}");
    let voisin = chemin.with_file_name(format!("{nom}.{}.partiel", std::process::id()));
    let mut fichier = File::create(&voisin).map_err(ne_s_ecrit_pas)?;
    let ecrit = engendrer()
        .map_err(|e| format!("{nom} ne s'engendre pas : {e}"))
        .and_then(|pdf| {
            fichier
                .write_all(&pdf)
                .and_then(|()| fichier.sync_all())
                .map_err(ne_s_ecrit_pas)
        });
    // Fermé avant de prendre la place : Windows ne renomme pas un fichier ouvert.
    drop(fichier);
    let place = ecrit.and_then(|()| std::fs::rename(&voisin, chemin).map_err(ne_s_ecrit_pas));
    if place.is_err() {
        let _ = std::fs::remove_file(&voisin);
    }
    place.map(|()| Some(nom))
}

/// Exporte le cahier des tables : la boîte d'enregistrement propose
/// `cahier-des-tables-AAAA-MM-JJ.pdf`, puis le cahier s'engendre pour cet instrument et dans la
/// couche sombre de l'en-tête, daté du jour que l'interface donne (I25), et s'écrit.
/// `Ok(None)` : la boîte a été fermée, rien n'est écrit ; `Ok(Some(nom du fichier))` ;
/// `Err(message en français)`.
#[tauri::command]
async fn exporter_cahier(
    app: tauri::AppHandle,
    date: String,
    instrument: String,
    couche: String,
) -> Result<Option<String>, String> {
    let i = lire_instrument(&instrument).ok_or_else(|| {
        format!("instrument inconnu : « {instrument} », attendu guitare, clavier ou les-deux")
    })?;
    let c = lire_couche(&couche)
        .ok_or_else(|| format!("couche inconnue : « {couche} », attendue masquee ou affichee"))?;
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
            cahier::engendrer(i, c, &d).map_err(|e| e.to_string())
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
    fn lire_les_couches() {
        assert_eq!(lire_couche("masquee"), Some(CoucheSombre::Masquee));
        assert_eq!(lire_couche("affichee"), Some(CoucheSombre::Affichee));
        // Les noms sont ceux de la commande `cahier`, sans accent.
        assert_eq!(lire_couche("affichée"), None);
        assert_eq!(lire_couche("sombre"), None);
        assert_eq!(lire_couche(""), None);
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

    /// Les noms de ce que contient un dossier, dans l'ordre.
    fn entrees(d: &Path) -> Vec<String> {
        let mut noms: Vec<String> = std::fs::read_dir(d)
            .expect("le dossier se lit")
            .map(|e| {
                e.expect("une entrée")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        noms.sort();
        noms
    }

    const NOM: &str = "cahier-des-tables-2026-11-01.pdf";

    #[test]
    fn la_boite_fermee_n_engendre_ni_n_ecrit_rien() {
        // Vigilance 5 : rien d'écrit, et rien à dire.
        let r = ecrire_le_cahier(None, || panic!("rien ne s'engendre"));
        assert_eq!(r, Ok(None));
    }

    #[test]
    fn le_cahier_s_ecrit_et_l_export_rend_le_nom_du_fichier() {
        let d = dossier("ecrit");
        let chemin = d.join(NOM);
        let r = ecrire_le_cahier(Some(&chemin), || Ok(b"%PDF-1.7".to_vec()));
        assert_eq!(r, Ok(Some(NOM.to_string())));
        assert_eq!(
            std::fs::read(&chemin).expect("le fichier est écrit"),
            b"%PDF-1.7"
        );
        assert_eq!(entrees(&d), [NOM], "rien de temporaire ne reste");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn le_cahier_s_ecrit_d_abord_a_cote_puis_prend_la_place_du_cahier_choisi() {
        let d = dossier("remplace");
        let chemin = d.join(NOM);
        std::fs::write(&chemin, b"ancien").expect("l'ancien cahier s'écrit");
        let r = ecrire_le_cahier(Some(&chemin), || {
            // Pendant que le cahier s'engendre, l'ancien est intact, et un fichier voisin attend.
            assert_eq!(std::fs::read(&chemin).expect("l'ancien se lit"), b"ancien");
            assert_eq!(entrees(&d).len(), 2, "{:?}", entrees(&d));
            Ok(b"%PDF-1.7".to_vec())
        });
        assert_eq!(r, Ok(Some(NOM.to_string())));
        assert_eq!(
            std::fs::read(&chemin).expect("le cahier se lit"),
            b"%PDF-1.7"
        );
        assert_eq!(entrees(&d), [NOM], "rien de temporaire ne reste");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn un_dossier_ou_l_on_ne_peut_ecrire_se_dit_avant_que_le_cahier_ne_s_engendre() {
        // Vigilance 5 : dans un dossier qui n'existe pas, le fichier ne s'écrit pas, et le dire
        // n'attend pas les secondes du cahier.
        let d = dossier("refus");
        let chemin = d.join("absent").join(NOM);
        let r = ecrire_le_cahier(Some(&chemin), || panic!("rien ne s'engendre"));
        let e = r.expect_err("l'écriture échoue");
        assert!(e.starts_with(&format!("{NOM} ne s'écrit pas : ")), "{e}");
        assert!(entrees(&d).is_empty(), "{:?}", entrees(&d));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn un_cahier_qui_ne_s_engendre_pas_laisse_l_ancien_intact_et_rien_de_temporaire() {
        let d = dossier("engendre");
        let chemin = d.join("cahier.pdf");
        std::fs::write(&chemin, b"ancien").expect("l'ancien cahier s'écrit");
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
        assert_eq!(std::fs::read(&chemin).expect("l'ancien se lit"), b"ancien");
        assert_eq!(entrees(&d), ["cahier.pdf"]);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn un_remplacement_qui_echoue_se_dit_et_ne_laisse_rien_de_temporaire() {
        // Le nom choisi est celui d'un dossier : le cahier, écrit à côté, ne peut en prendre la
        // place, et ce que le dossier contient ne bouge pas.
        let d = dossier("place-prise");
        let chemin = d.join("cahier.pdf");
        std::fs::create_dir(&chemin).expect("le dossier se crée");
        std::fs::write(chemin.join("garde"), b"ancien").expect("son fichier s'écrit");
        let r = ecrire_le_cahier(Some(&chemin), || Ok(b"%PDF-1.7".to_vec()));
        let e = r.expect_err("le remplacement échoue");
        assert!(e.starts_with("cahier.pdf ne s'écrit pas : "), "{e}");
        assert_eq!(entrees(&d), ["cahier.pdf"], "rien de temporaire ne reste");
        assert_eq!(
            std::fs::read(chemin.join("garde")).expect("le fichier se lit"),
            b"ancien"
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
