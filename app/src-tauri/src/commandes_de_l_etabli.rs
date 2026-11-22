//! Les commandes de la tranche 2, les lectures de la boîte (tranche 3) et les commandes de
//! l'import (tranche 5), servies par l'établi : chacune envoie au fil du fonds
//! (`Ouverture::demander`) l'appel de la méthode de même nom des services, et rend sa `Reponse`,
//! aux types de `commandes`. Chacune prend l'état, l'ouverture du fonds, et au plus un argument,
//! nommé `commande` : un seul mot, que le camelCase des arguments de Tauri ne change pas. Les
//! commandes de l'import prennent aussi, de l'état, le dossier choisi (`DossierChoisi`), et celles
//! qui comparent la place libre, le dossier de l'application (`DossierDeLApplication`).

use std::path::Path;
use std::sync::{Arc, PoisonError};

use commandes::{
    Boite, Captation, ChoisirLeType, CloreLImport, Completer, DeclarerUneOrigine, Enregistrer,
    EtatDuFonds, Etiqueter, Faite, Fenetre, FichierVerse, ImportClos, ImportOuvert, Inspection,
    LireUnRapport, Notee, Noter, OuvrirLImport, Raison, RapportDImport, Refus, Regrouper, Relier,
    Renommer, Reponse, RequeteEnregistree, Supprimer, TexteLu, VerserUnFichier,
    VerserUnePriseEnStems, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees, VueDuJournal,
};
use stockage::source::SourceSurFichiers;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

use crate::DossierChoisi;
use crate::fil::Ouverture;
use crate::systeme::{self, DossierDeLApplication};

/// Noter une idée (`Services::noter`).
#[tauri::command]
pub async fn noter(ouverture: tauri::State<'_, Ouverture>, commande: Noter) -> Reponse<Notee> {
    ouverture
        .demander(move |services, contexte| services.noter(&commande, contexte))
        .await
}

/// Compléter le texte d'une idée (`Services::completer`).
#[tauri::command]
pub async fn completer(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Completer,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.completer(&commande, contexte))
        .await
}

/// Poser une étiquette sur une idée (`Services::poser_une_etiquette`).
#[tauri::command]
pub async fn poser_une_etiquette(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Etiqueter,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.poser_une_etiquette(&commande, contexte))
        .await
}

/// Ôter une étiquette d'une idée (`Services::oter_une_etiquette`).
#[tauri::command]
pub async fn oter_une_etiquette(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Etiqueter,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.oter_une_etiquette(&commande, contexte))
        .await
}

/// Relier deux idées (`Services::relier`).
#[tauri::command]
pub async fn relier(ouverture: tauri::State<'_, Ouverture>, commande: Relier) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.relier(&commande, contexte))
        .await
}

/// Délier deux idées (`Services::delier`).
#[tauri::command]
pub async fn delier(ouverture: tauri::State<'_, Ouverture>, commande: Relier) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.delier(&commande, contexte))
        .await
}

/// Regrouper une captation dans une idée (`Services::regrouper`).
#[tauri::command]
pub async fn regrouper(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Regrouper,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.regrouper(&commande, contexte))
        .await
}

/// Retirer une idée (`Services::retirer`).
#[tauri::command]
pub async fn retirer(ouverture: tauri::State<'_, Ouverture>, commande: Viser) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.retirer(&commande, contexte))
        .await
}

/// Rétablir une idée retirée (`Services::retablir`).
#[tauri::command]
pub async fn retablir(ouverture: tauri::State<'_, Ouverture>, commande: Viser) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.retablir(&commande, contexte))
        .await
}

/// Annuler le dernier geste de tri (`Services::annuler`).
#[tauri::command]
pub async fn annuler(ouverture: tauri::State<'_, Ouverture>) -> Reponse<Faite> {
    ouverture
        .demander(|services, contexte| services.annuler(contexte))
        .await
}

/// La vue d'une idée, `commande` son identifiant (`Services::vue_d_une_idee`).
#[tauri::command]
pub async fn vue_d_une_idee(
    ouverture: tauri::State<'_, Ouverture>,
    commande: String,
) -> Reponse<VueDUneIdee> {
    ouverture
        .demander(move |services, _| services.vue_d_une_idee(&commande))
        .await
}

/// Les idées non retirées (`Services::idees`).
#[tauri::command]
pub async fn idees(ouverture: tauri::State<'_, Ouverture>) -> Reponse<Vec<String>> {
    ouverture.demander(|services, _| services.idees()).await
}

/// Le journal du fonds en fenêtre : son nombre d'entrées, son empreinte, et les lignes de la
/// fenêtre, la plus récente d'abord (`Services::journal_en_fenetre`).
#[tauri::command]
pub async fn journal(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Fenetre,
) -> Reponse<VueDuJournal> {
    ouverture
        .demander(move |services, _| services.journal_en_fenetre(&commande))
        .await
}

/// La boîte d'idées : les non reliées, la requête, la fenêtre de lignes (`Services::boite`).
#[tauri::command]
pub async fn boite(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Boite,
) -> Reponse<VueDeLaBoite> {
    ouverture
        .demander(move |services, _| services.boite(&commande))
        .await
}

/// Les idées retirées, la plus récemment retirée d'abord (`Services::retirees`).
#[tauri::command]
pub async fn retirees(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Fenetre,
) -> Reponse<VueDesRetirees> {
    ouverture
        .demander(move |services, _| services.retirees(&commande))
        .await
}

/// L'état du fonds : né ou non, lecture seule, avertissements, geste à annuler
/// (`Services::etat_du_fonds`).
#[tauri::command]
pub async fn etat_du_fonds(ouverture: tauri::State<'_, Ouverture>) -> Reponse<EtatDuFonds> {
    ouverture
        .demander(|services, _| services.etat_du_fonds())
        .await
}

/// Les requêtes enregistrées, par nom (`Services::requetes_enregistrees`).
#[tauri::command]
pub async fn requetes_enregistrees(
    ouverture: tauri::State<'_, Ouverture>,
) -> Reponse<Vec<RequeteEnregistree>> {
    ouverture
        .demander(|services, _| services.requetes_enregistrees())
        .await
}

/// Enregistrer une requête sous un nom (`Services::enregistrer_une_requete`).
#[tauri::command]
pub async fn enregistrer_une_requete(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Enregistrer,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.enregistrer_une_requete(&commande, contexte))
        .await
}

/// Renommer une requête enregistrée (`Services::renommer_une_requete`).
#[tauri::command]
pub async fn renommer_une_requete(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Renommer,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.renommer_une_requete(&commande, contexte))
        .await
}

/// Supprimer une requête enregistrée (`Services::supprimer_une_requete`).
#[tauri::command]
pub async fn supprimer_une_requete(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Supprimer,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.supprimer_une_requete(&commande, contexte))
        .await
}

/// Le refus de `Raison::Impossible`, de message `message`.
fn impossible(message: String) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message,
    }
}

/// La source du dossier choisi, que garde `DossierChoisi` ; sans elle, « Aucun dossier n'est
/// choisi. », de `Raison::Impossible`.
fn la_source(choisi: &DossierChoisi) -> Result<Arc<SourceSurFichiers>, Refus> {
    choisi
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
        .ok_or_else(|| impossible("Aucun dossier n'est choisi.".to_string()))
}

/// Oublie le dossier choisi, s'il est encore `source` : un dossier choisi entre-temps reste.
fn oublier(choisi: &DossierChoisi, source: &Arc<SourceSurFichiers>) {
    let mut gardee = choisi.0.lock().unwrap_or_else(PoisonError::into_inner);
    if gardee
        .as_ref()
        .is_some_and(|actuelle| Arc::ptr_eq(actuelle, source))
    {
        *gardee = None;
    }
}

/// La place libre du volume du dossier de l'application, où vit la racine des données
/// (`systeme::place_libre` ; R19) ; un échec, de `Raison::Disque` : « La place libre ne se lit
/// pas : <erreur>. ».
fn lire_la_place_libre(application: &Path) -> Result<u64, Refus> {
    systeme::place_libre(application).map_err(|erreur| Refus {
        raison: Raison::Disque,
        message: format!("La place libre ne se lit pas : {erreur}."),
    })
}

/// Le refus d'un dossier choisi qui ne se parcourt pas : de `Raison::Disque`, le texte de
/// l'erreur du stockage, qui nomme le dossier ou l'entrée en cause.
fn refus_du_parcours(erreur: stockage::Erreur) -> Refus {
    Refus {
        raison: Raison::Disque,
        message: erreur.to_string(),
    }
}

/// Inspecter un dossier à verser (`Services::inspecter`). La boîte de dossier, « Importer un
/// dossier », attend sa réponse hors du fil de l'IPC (`spawn_blocking`) ; fermée sans choix :
/// `Ok(None)`, et rien ne change. Le dossier choisi, admis par `systeme::admettre_le_dossier`
/// (son refus, de `Raison::Impossible`), puis parcouru par `SourceSurFichiers::parcourir` (son
/// refus, de `Raison::Disque`), hors du fil du fonds ; la source, gardée dans `DossierChoisi` ;
/// puis, au fil, la place libre, lue sur le volume du dossier de l'application, où vit la racine
/// des données, que la commande ne connaît pas (R19), et l'inspection.
#[tauri::command]
pub async fn inspecter_un_dossier<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    ouverture: tauri::State<'_, Ouverture>,
    choisi: tauri::State<'_, DossierChoisi>,
    application: tauri::State<'_, DossierDeLApplication>,
) -> Reponse<Option<Inspection>> {
    let dossier_de_l_application = application.0.clone();
    let parcouru = tauri::async_runtime::spawn_blocking(
        move || -> Result<Option<SourceSurFichiers>, Refus> {
            let boite = app.dialog().file().set_title("Importer un dossier");
            #[cfg(desktop)]
            let boite = match app.get_webview_window("main") {
                Some(fenetre) => boite.set_parent(&fenetre),
                None => boite,
            };
            let Some(chemin) = boite.blocking_pick_folder() else {
                return Ok(None);
            };
            let chemin = chemin.into_path().map_err(|erreur| {
                impossible(format!("Le dossier choisi ne se lit pas : {erreur}."))
            })?;
            let admis = systeme::admettre_le_dossier(&chemin, &dossier_de_l_application)
                .map_err(impossible)?;
            SourceSurFichiers::parcourir(&admis)
                .map(Some)
                .map_err(refus_du_parcours)
        },
    )
    .await
    .map_err(|_| impossible("Le choix du dossier s'est interrompu.".to_string()))??;
    let Some(source) = parcouru else {
        return Ok(None);
    };
    let source = Arc::new(source);
    *choisi.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(Arc::clone(&source));
    let application = application.0.clone();
    ouverture
        .demander(move |services, _| {
            let place_libre = lire_la_place_libre(&application)?;
            services.inspecter(&*source, place_libre)
        })
        .await
        .map(Some)
}

/// Ouvrir l'import du dossier choisi, ou le reprendre (`Services::ouvrir_l_import`) : la place
/// libre, relue au fil comme à l'inspection, que les services comparent de nouveau.
#[tauri::command]
pub async fn ouvrir_l_import(
    ouverture: tauri::State<'_, Ouverture>,
    choisi: tauri::State<'_, DossierChoisi>,
    application: tauri::State<'_, DossierDeLApplication>,
    commande: OuvrirLImport,
) -> Reponse<ImportOuvert> {
    let source = la_source(&choisi)?;
    let application = application.0.clone();
    ouverture
        .demander(move |services, contexte| {
            let place_libre = lire_la_place_libre(&application)?;
            services.ouvrir_l_import(&*source, &commande, place_libre, contexte)
        })
        .await
}

/// Verser un fichier du dossier choisi (`Services::verser_un_fichier`).
#[tauri::command]
pub async fn verser_un_fichier(
    ouverture: tauri::State<'_, Ouverture>,
    choisi: tauri::State<'_, DossierChoisi>,
    commande: VerserUnFichier,
) -> Reponse<FichierVerse> {
    let source = la_source(&choisi)?;
    ouverture
        .demander(move |services, contexte| {
            services.verser_un_fichier(&*source, &commande, contexte)
        })
        .await
}

/// Verser une prise en stems du dossier choisi (`Services::verser_une_prise_en_stems`).
#[tauri::command]
pub async fn verser_une_prise_en_stems(
    ouverture: tauri::State<'_, Ouverture>,
    choisi: tauri::State<'_, DossierChoisi>,
    commande: VerserUnePriseEnStems,
) -> Reponse<Faite> {
    let source = la_source(&choisi)?;
    ouverture
        .demander(move |services, contexte| {
            services.verser_une_prise_en_stems(&*source, &commande, contexte)
        })
        .await
}

/// Clore l'import du dossier choisi (`Services::clore_l_import`) ; clos, le dossier choisi
/// s'oublie, s'il est encore celui de l'import. Refusée, l'import reste ouvert, et le dossier
/// choisi reste.
#[tauri::command]
pub async fn clore_l_import(
    ouverture: tauri::State<'_, Ouverture>,
    choisi: tauri::State<'_, DossierChoisi>,
    commande: CloreLImport,
) -> Reponse<ImportClos> {
    let source = la_source(&choisi)?;
    let de_l_import = Arc::clone(&source);
    let clos = ouverture
        .demander(move |services, contexte| services.clore_l_import(&*source, &commande, contexte))
        .await?;
    oublier(&choisi, &de_l_import);
    Ok(clos)
}

/// Déclarer l'origine d'une captation, ou des captations du dernier import clos sous un préfixe
/// (`Services::declarer_une_origine`).
#[tauri::command]
pub async fn declarer_une_origine(
    ouverture: tauri::State<'_, Ouverture>,
    commande: DeclarerUneOrigine,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.declarer_une_origine(&commande, contexte))
        .await
}

/// Choisir le type d'un stem, ou garder celui qui est proposé
/// (`Services::choisir_le_type_d_un_stem`).
#[tauri::command]
pub async fn choisir_le_type_d_un_stem(
    ouverture: tauri::State<'_, Ouverture>,
    commande: ChoisirLeType,
) -> Reponse<Faite> {
    ouverture
        .demander(move |services, contexte| services.choisir_le_type_d_un_stem(&commande, contexte))
        .await
}

/// Lire un texte versé, `commande` sa captation (`Services::lire_un_texte_verse`).
#[tauri::command]
pub async fn lire_un_texte_verse(
    ouverture: tauri::State<'_, Ouverture>,
    commande: Captation,
) -> Reponse<TexteLu> {
    ouverture
        .demander(move |services, _| services.lire_un_texte_verse(&commande))
        .await
}

/// Lire le rapport d'un import (`Services::lire_un_rapport`).
#[tauri::command]
pub async fn lire_un_rapport(
    ouverture: tauri::State<'_, Ouverture>,
    commande: LireUnRapport,
) -> Reponse<RapportDImport> {
    ouverture
        .demander(move |services, _| services.lire_un_rapport(&commande))
        .await
}

/// Les noms des générateurs déclarés du fonds (`Services::generateurs`).
#[tauri::command]
pub async fn generateurs(ouverture: tauri::State<'_, Ouverture>) -> Reponse<Vec<String>> {
    ouverture
        .demander(|services, _| services.generateurs())
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use commandes::{Notee, Raison, Refus, VueDUneIdee};
    use serde_json::json;
    use tauri::ipc::{CallbackFn, InvokeBody, InvokeResponseBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::webview::InvokeRequest;

    use crate::fil;
    use crate::tests::dossier_jetable;

    /// Invoque la commande `nom` par l'IPC simulée de `tauri::test`, depuis `fenetre`, avec le
    /// corps `corps` ; rend sa réponse, ou son rejet. L'adresse est l'origine locale de l'établi,
    /// `http://tauri.localhost` sous Windows et Android, `tauri://localhost` ailleurs : le
    /// contexte simulé, sans manifeste de l'application, n'y passe par aucune ACL.
    fn invoquer(
        fenetre: &tauri::WebviewWindow<MockRuntime>,
        nom: &str,
        corps: serde_json::Value,
    ) -> Result<InvokeResponseBody, serde_json::Value> {
        get_ipc_response(
            fenetre,
            InvokeRequest {
                cmd: nom.to_string(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: if cfg!(any(windows, target_os = "android")) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .expect("une adresse"),
                body: InvokeBody::Json(corps),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
    }

    #[test]
    fn les_commandes_passent_par_l_ipc_de_tauri() {
        let racine = dossier_jetable("ipc").join("donnees");
        let app = mock_builder()
            .invoke_handler(crate::gestionnaire())
            .manage(fil::ouvrir(racine))
            .build(mock_context(noop_assets()))
            .expect("l'application simulée se construit");
        let fenetre = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("la fenêtre main se construit");

        // noter : l'argument se lit sous `commande`, et la réponse est une `Notee`.
        let notee: Notee = invoquer(
            &fenetre,
            "noter",
            json!({"commande": {"texte": "Une idée de pont", "decalage_minutes": 60}}),
        )
        .expect("noter répond")
        .deserialize()
        .expect("une Notee");

        // vue_d_une_idee : l'idée née se relit, son texte compris.
        let vue: VueDUneIdee =
            invoquer(&fenetre, "vue_d_une_idee", json!({"commande": notee.idee}))
                .expect("vue_d_une_idee répond")
                .deserialize()
                .expect("une VueDUneIdee");
        assert_eq!(vue.texte.as_deref(), Some("Une idée de pont"));

        // annuler, sans geste de tri depuis l'ouverture : la promesse est rejetée, et le rejet
        // se lit en `Refus`.
        let rejet = invoquer(&fenetre, "annuler", json!({})).expect_err("annuler est rejetée");
        let refus: Refus = serde_json::from_value(rejet).expect("un Refus");
        assert_eq!(refus.raison, Raison::RienAAnnuler);
    }

    #[test]
    fn les_commandes_de_l_import_passent_par_l_ipc_de_tauri() {
        let dossier = dossier_jetable("ipc-import");
        let app = mock_builder()
            .invoke_handler(crate::gestionnaire())
            .manage(fil::ouvrir(dossier.join("donnees")))
            .manage(crate::DossierChoisi(Mutex::new(None)))
            .manage(crate::systeme::DossierDeLApplication(dossier))
            .build(mock_context(noop_assets()))
            .expect("l'application simulée se construit");
        let fenetre = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("la fenêtre main se construit");

        // Sans dossier choisi, l'import ne s'ouvre, ne verse ni ne se clôt ; chaque argument se
        // lit sous `commande`.
        let aucun = Refus {
            raison: Raison::Impossible,
            message: "Aucun dossier n'est choisi.".to_string(),
        };
        let import = "01JD0000000000000000000000";
        for (nom, corps) in [
            (
                "ouvrir_l_import",
                json!({"commande": {"declarations": [], "decalage_minutes": 60}}),
            ),
            (
                "verser_un_fichier",
                json!({"commande": {"import": import, "rang": 1, "decalage_minutes": 60}}),
            ),
            (
                "verser_une_prise_en_stems",
                json!({"commande": {"import": import, "dossier": "Démos", "decalage_minutes": 60}}),
            ),
            (
                "clore_l_import",
                json!({"commande": {"import": import, "decalage_minutes": 60}}),
            ),
        ] {
            let rejet = invoquer(&fenetre, nom, corps).expect_err(nom);
            let refus: Refus = serde_json::from_value(rejet).expect("un Refus");
            assert_eq!(refus, aucun, "{nom}");
        }

        // Les lectures passent au fil : aucun générateur, et aucun rapport, dans un fonds qui
        // n'est pas né.
        let generateurs: Vec<String> = invoquer(&fenetre, "generateurs", json!({}))
            .expect("generateurs répond")
            .deserialize()
            .expect("des noms");
        assert!(generateurs.is_empty(), "{generateurs:?}");
        let rejet = invoquer(
            &fenetre,
            "lire_un_rapport",
            json!({"commande": {"import": import}}),
        )
        .expect_err("lire_un_rapport est rejetée");
        let refus: Refus = serde_json::from_value(rejet).expect("un Refus");
        assert_eq!(refus.message, "Cet import n'a pas de rapport.");
    }
}
