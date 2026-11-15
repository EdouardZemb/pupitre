//! Le pont natif de l'interface : les commandes de l'établi, par l'IPC de Tauri (décision 11), et
//! le décalage local d'un geste ou d'un instant.

use commandes::{Raison, Refus};
use js_sys::{Function, JSON, Promise, Reflect};
use serde::de::DeserializeOwned;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// Le refus de toute commande hors de l'application, l'interface servie seule dans un navigateur.
const HORS_DE_L_APPLICATION: &str = "Hors de l'application, aucune commande ne passe.";

/// Le constat d'un rejet qui n'est pas un `Refus` (question 31).
const SANS_ISSUE: &str = "Le geste n'a pas abouti.";

/// `window.__TAURI__.core` et sa fonction `invoke`, dans l'application ; ailleurs, rien.
pub(crate) fn invoke_de_tauri() -> Option<(JsValue, Function)> {
    let tauri = Reflect::get(&js_sys::global(), &JsValue::from_str("__TAURI__")).ok()?;
    let core = Reflect::get(&tauri, &JsValue::from_str("core")).ok()?;
    let invoke = Reflect::get(&core, &JsValue::from_str("invoke")).ok()?;
    Some((core, invoke.dyn_into::<Function>().ok()?))
}

/// Vrai dans l'application : `window.__TAURI__.core.invoke` existe, comme le lit
/// `invoke_de_tauri` ; `invoquer` rend hors de l'application le refus « Hors de l'application,
/// aucune commande ne passe. ».
pub(crate) fn dans_l_application() -> bool {
    invoke_de_tauri().is_some()
}

/// Le constat « Le geste n'a pas abouti. », de raison `Raison::Impossible`.
fn sans_issue() -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: SANS_ISSUE.to_string(),
    }
}

/// `valeur` en JSON, par `JSON.stringify` ; `None` si elle ne s'y écrit pas (`undefined`, une
/// fonction, un cycle).
fn en_json(valeur: &JsValue) -> Option<String> {
    JSON::stringify(valeur).ok()?.as_string()
}

/// Invoque la commande `nom` (une constante de `commandes`), avec `{"commande": …}` ou `{}`, par
/// `serde_json` puis `JSON.parse` (décision 11) ; la réponse se relit par `JSON.stringify` puis
/// `serde_json`. Un rejet : `refus_du_rejet`. Hors de l'application : un refus, « Hors de
/// l'application, aucune commande ne passe. », de raison `Raison::Impossible`. Une réponse qui ne
/// se relit pas : le constat « Le geste n'a pas abouti. ».
pub(crate) async fn invoquer<R: DeserializeOwned>(
    nom: &str,
    commande: Option<serde_json::Value>,
) -> Result<R, Refus> {
    let Some((core, invoke)) = invoke_de_tauri() else {
        return Err(Refus {
            raison: Raison::Impossible,
            message: HORS_DE_L_APPLICATION.to_string(),
        });
    };
    let arguments = match commande {
        Some(commande) => serde_json::json!({ "commande": commande }),
        None => serde_json::json!({}),
    };
    let arguments = JSON::parse(&arguments.to_string()).map_err(|_| sans_issue())?;
    let promesse = invoke
        .call2(&core, &JsValue::from_str(nom), &arguments)
        .map_err(|rejet| refus_du_rejet(en_json(&rejet)))?
        .dyn_into::<Promise>()
        .map_err(|_| sans_issue())?;
    let reponse = JsFuture::from(promesse)
        .await
        .map_err(|rejet| refus_du_rejet(en_json(&rejet)))?;
    en_json(&reponse)
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .ok_or_else(sans_issue)
}

/// Le refus que dit un rejet, lu en JSON : un `Refus`, tel quel ; tout autre rejet, ou aucun, le
/// constat `Refus { raison: Raison::Impossible, message: "Le geste n'a pas abouti." }`
/// (question 31).
pub(crate) fn refus_du_rejet(rejet_en_json: Option<String>) -> Refus {
    rejet_en_json
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_else(sans_issue)
}

/// Le décalage local du geste, en minutes : `decalage_depuis_js` de
/// `Date::new_0().get_timezone_offset()`.
#[expect(
    dead_code,
    reason = "la tâche 19 : chaque geste de la boîte porte le décalage local"
)]
pub(crate) fn decalage_du_geste() -> i32 {
    decalage_depuis_js(js_sys::Date::new_0().get_timezone_offset())
}

/// Le décalage local de l'instant `le` (en secondes), rendu en minutes, celles qu'attend
/// `date_de_la_liste` : `decalage_depuis_js` de
/// `Date::new(&JsValue::from_f64(le as f64 * 1000.0)).get_timezone_offset()` (question 24).
#[expect(
    dead_code,
    reason = "la tâche 19 : la date des lignes de la boîte, au décalage de leur instant"
)]
pub(crate) fn decalage_a(le: i64) -> i32 {
    let instant = js_sys::Date::new(&JsValue::from_f64(le as f64 * 1000.0));
    decalage_depuis_js(instant.get_timezone_offset())
}

/// `getTimezoneOffset` vaut UTC moins l'heure locale : `decalage_minutes` en est l'opposé (LE-6,
/// règle 20), arrondi à la minute (une zone ancienne peut rendre une fraction de minute).
pub(crate) fn decalage_depuis_js(minutes_de_javascript: f64) -> i32 {
    (-minutes_de_javascript).round() as i32
}

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
