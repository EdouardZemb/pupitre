//! Les commandes de la tranche 2 et les lectures de la boîte (tranche 3), servies par l'établi :
//! chacune envoie au fil du fonds (`Ouverture::demander`) l'appel de la méthode de même nom des
//! services, et rend sa `Reponse`, aux types de `commandes`. Chacune prend l'état, l'ouverture
//! du fonds, et au plus un argument, nommé `commande` : un seul mot, que le camelCase des
//! arguments de Tauri ne change pas.

use commandes::{
    Boite, Completer, EntreeDuJournal, EtatDuFonds, Etiqueter, Faite, Fenetre, Notee, Noter,
    Regrouper, Relier, Reponse, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees,
};

use crate::fil::Ouverture;

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

/// Le journal du fonds (`Services::journal`).
#[tauri::command]
pub async fn journal(ouverture: tauri::State<'_, Ouverture>) -> Reponse<Vec<EntreeDuJournal>> {
    ouverture.demander(|services, _| services.journal()).await
}

/// La boîte d'idées : les non reliées, le filtre, la fenêtre de lignes (`Services::boite`).
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

#[cfg(test)]
mod tests {
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
}
