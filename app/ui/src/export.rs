//! L'export du cahier des tables : l'interface demande à l'établi de l'écrire, par la commande
//! `exporter_cahier` de Tauri, avec la date du jour, l'instrument et la couche sombre choisis.
//! Hors de l'application, le bouton le dit, et ne fait rien.

use js_sys::{Function, Object, Promise, Reflect};
use leptos::prelude::*;
use noyau::rendu::Instrument;
use noyau::table::CoucheSombre;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// L'instrument tel que la commande le lit : « guitare », « clavier » ou « les-deux ».
pub(crate) fn nom_d_instrument(i: Instrument) -> &'static str {
    match i {
        Instrument::Guitare => "guitare",
        Instrument::Clavier => "clavier",
        Instrument::LesDeux => "les-deux",
    }
}

/// La couche sombre telle que l'établi la lit : « masquee » ou « affichee ».
pub(crate) fn nom_de_couche(c: CoucheSombre) -> &'static str {
    match c {
        CoucheSombre::Masquee => "masquee",
        CoucheSombre::Affichee => "affichee",
    }
}

/// Une date au format AAAA-MM-JJ ; le mois compte de 1 à 12.
pub(crate) fn date_iso(annee: u32, mois: u32, jour: u32) -> String {
    format!("{annee:04}-{mois:02}-{jour:02}")
}

/// Ce que l'interface dit de l'export : le nom du fichier écrit, ou l'erreur ; rien si la boîte
/// d'enregistrement a été fermée.
pub(crate) fn message(resultat: Result<Option<String>, String>) -> Option<String> {
    match resultat {
        Ok(None) => None,
        Ok(Some(nom)) => Some(format!("Cahier exporté : {nom}")),
        Err(e) => Some(e),
    }
}

/// La date du jour, à l'heure de l'ordinateur. `getMonth` compte de 0 à 11.
fn date_du_jour() -> String {
    let d = js_sys::Date::new_0();
    date_iso(d.get_full_year(), d.get_month() + 1, d.get_date())
}

/// `window.__TAURI__.core` et sa fonction `invoke`, dans l'application ; ailleurs, rien.
fn invoke_de_tauri() -> Option<(JsValue, Function)> {
    let tauri = Reflect::get(&js_sys::global(), &JsValue::from_str("__TAURI__")).ok()?;
    let core = Reflect::get(&tauri, &JsValue::from_str("core")).ok()?;
    let invoke = Reflect::get(&core, &JsValue::from_str("invoke")).ok()?;
    Some((core, invoke.dyn_into::<Function>().ok()?))
}

/// Le texte d'un refus : celui de la commande, en français, ou celui d'une erreur JavaScript.
fn texte_du_refus(e: &JsValue) -> String {
    e.as_string()
        .or_else(|| e.dyn_ref::<js_sys::Error>().map(|e| e.message().into()))
        .unwrap_or_else(|| "l'export a échoué, sans dire pourquoi".to_string())
}

/// Appelle `exporter_cahier`. `Ok(None)` : la boîte d'enregistrement a été fermée.
async fn exporter(
    core: JsValue,
    invoke: Function,
    instrument: Instrument,
    couche: CoucheSombre,
) -> Result<Option<String>, String> {
    let arguments = Object::new();
    for (cle, valeur) in [
        ("date", date_du_jour()),
        ("instrument", nom_d_instrument(instrument).to_string()),
        ("couche", nom_de_couche(couche).to_string()),
    ] {
        Reflect::set(
            &arguments,
            &JsValue::from_str(cle),
            &JsValue::from_str(&valeur),
        )
        .map_err(|e| texte_du_refus(&e))?;
    }
    let promesse = invoke
        .call2(&core, &JsValue::from_str("exporter_cahier"), &arguments)
        .map_err(|e| texte_du_refus(&e))?
        .dyn_into::<Promise>()
        .map_err(|_| "l'établi n'a pas lancé l'export".to_string())?;
    let reponse = JsFuture::from(promesse)
        .await
        .map_err(|e| texte_du_refus(&e))?;
    if reponse.is_null() || reponse.is_undefined() {
        return Ok(None);
    }
    reponse
        .as_string()
        .map(Some)
        .ok_or_else(|| "l'établi a répondu sans nom de fichier".to_string())
}

/// « Exporter le cahier », pour l'instrument et dans la couche sombre de l'en-tête. Pendant
/// l'export, le bouton est désactivé, et un texte fixe dit « Export en cours… ».
#[component]
pub fn BoutonExport(
    #[prop(into)] instrument: Signal<Instrument>,
    #[prop(into)] couche: Signal<CoucheSombre>,
) -> impl IntoView {
    let disponible = invoke_de_tauri().is_some();
    let en_cours = RwSignal::new(false);
    let dit = RwSignal::new(None::<String>);
    let lancer = move |_| {
        if en_cours.get_untracked() {
            return;
        }
        let Some((core, invoke)) = invoke_de_tauri() else {
            return;
        };
        en_cours.set(true);
        dit.set(None);
        let (i, c) = (instrument.get_untracked(), couche.get_untracked());
        leptos::task::spawn_local(async move {
            dit.set(message(exporter(core, invoke, i, c).await));
            en_cours.set(false);
        });
    };
    let libelle = if disponible {
        "Exporter le cahier"
    } else {
        "Export indisponible hors de l'application"
    };
    view! {
        <div class="export">
            <button type="button" disabled=move || !disponible || en_cours.get() on:click=lancer>
                {libelle}
            </button>
            <p class="etat-export" role="status">
                {move || {
                    if en_cours.get() { Some("Export en cours…".to_string()) } else { dit.get() }
                }}
            </p>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_instrument_se_nomme_comme_la_commande_le_lit() {
        assert_eq!(nom_d_instrument(Instrument::Guitare), "guitare");
        assert_eq!(nom_d_instrument(Instrument::Clavier), "clavier");
        assert_eq!(nom_d_instrument(Instrument::LesDeux), "les-deux");
    }

    #[test]
    fn la_couche_se_nomme_comme_l_etabli_la_lit() {
        assert_eq!(nom_de_couche(CoucheSombre::Masquee), "masquee");
        assert_eq!(nom_de_couche(CoucheSombre::Affichee), "affichee");
    }

    #[test]
    fn la_date_s_ecrit_aaaa_mm_jj() {
        assert_eq!(date_iso(2026, 11, 1), "2026-11-01");
        assert_eq!(date_iso(2027, 1, 31), "2027-01-31");
    }

    #[test]
    fn une_boite_fermee_ne_dit_rien_et_une_erreur_se_dit() {
        // Vigilance 5 : la boîte fermée, rien ne s'affiche ; une écriture qui échoue, son message.
        assert_eq!(message(Ok(None)), None);
        assert_eq!(
            message(Ok(Some("cahier-des-tables-2026-11-01.pdf".to_string()))),
            Some("Cahier exporté : cahier-des-tables-2026-11-01.pdf".to_string())
        );
        let refus = "cahier-des-tables-2026-11-01.pdf ne s'écrit pas : accès refusé";
        assert_eq!(message(Err(refus.to_string())), Some(refus.to_string()));
    }
}
