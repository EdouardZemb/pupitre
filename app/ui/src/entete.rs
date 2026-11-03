//! L'en-tête de l'établi : la tonalité, les 24 d'un geste ; la couche sombre, affichée ou
//! masquée ; l'instrument ; et l'export du cahier des tables.

use leptos::prelude::*;
use noyau::rendu::Instrument;
use noyau::table::CoucheSombre;
use noyau::tonalite::Tonalite;

use crate::app::Etat;
use crate::export::BoutonExport;

/// Un choix parmi d'autres, d'un geste : un bouton radio et son libellé.
fn choix<T>(
    groupe: &'static str,
    valeur: T,
    libelle: String,
    nom_complet: Option<String>,
    actuel: Signal<T>,
    choisir: impl Fn(T) + Copy + 'static,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    view! {
        <label class="choix">
            <input
                type="radio"
                name=groupe
                aria-label=nom_complet
                prop:checked=move || actuel.get() == valeur
                on:change=move |_| choisir(valeur)
            />
            {libelle}
        </label>
    }
}

#[component]
pub fn Entete(etat: Etat) -> impl IntoView {
    // Les 24 tonalités en deux rangs, chaque majeure au-dessus de sa relative, dans l'ordre du
    // cahier : la grille se remplit colonne par colonne, et les flèches du clavier suivent cet
    // ordre. Chaque bouton montre la tonique, et se nomme en entier.
    let tonalites = Tonalite::les_24().map(|t| {
        choix(
            "tonalite",
            t,
            t.tonique.solfege(),
            Some(t.nom()),
            etat.tonalite.into(),
            move |t| etat.choisir_tonalite(t),
        )
    });
    let couches = [
        (CoucheSombre::Masquee, "masquée"),
        (CoucheSombre::Affichee, "affichée"),
    ]
    .map(|(c, libelle)| {
        choix(
            "couche",
            c,
            libelle.to_string(),
            None,
            etat.couche.into(),
            move |c| etat.choisir_couche(c),
        )
    });
    let instruments = [
        (Instrument::Guitare, "guitare"),
        (Instrument::Clavier, "clavier"),
        (Instrument::LesDeux, "les deux"),
    ]
    .map(|(i, libelle)| {
        choix(
            "instrument",
            i,
            libelle.to_string(),
            None,
            etat.instrument.into(),
            move |i| etat.choisir_instrument(i),
        )
    });
    view! {
        <header class="entete">
            <fieldset>
                <legend>"Tonalité"</legend>
                <div class="tonalites">
                    <span class="mode" aria-hidden="true">"majeur"</span>
                    <span class="mode" aria-hidden="true">"mineur"</span>
                    {tonalites.into_iter().collect_view()}
                </div>
            </fieldset>
            <fieldset>
                <legend>"Couche sombre"</legend>
                <div class="choix-en-ligne">{couches.into_iter().collect_view()}</div>
            </fieldset>
            <fieldset>
                <legend>"Instrument"</legend>
                <div class="choix-en-ligne">{instruments.into_iter().collect_view()}</div>
            </fieldset>
            <BoutonExport instrument=etat.instrument couche=etat.couche />
        </header>
    }
}
