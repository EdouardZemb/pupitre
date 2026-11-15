//! Les commandes de la tranche 2, servies par l'établi : chacune envoie au fil du fonds
//! (`Ouverture::demander`) l'appel de la méthode de même nom des services, et rend sa `Reponse`,
//! aux types de `commandes`. Chacune prend l'état, l'ouverture du fonds, et au plus un argument,
//! nommé `commande` : un seul mot, que le camelCase des arguments de Tauri ne change pas.

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
    /// `http://tauri.localhost` sous Windows et Android, `tauri://localhost` ailleurs : une
    /// commande de l'application, sans manifeste, n'y passe par aucune ACL.
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
