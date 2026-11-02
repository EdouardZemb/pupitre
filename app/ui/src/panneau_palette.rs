//! Le panneau de la palette : la fiche de l'accord touché, sa colonne comme au verso, selon
//! l'instrument, à droite de la table et à l'échelle de ses colonnes.

use leptos::html;
use leptos::prelude::*;
use noyau::rendu::verso::fiche_d_accord;

use crate::app::Etat;

/// La place de la palette, toujours réservée : la table ne bouge pas quand la palette s'ouvre.
/// Ouverte, la palette porte « Fermer », puis la fiche, qui défile si elle est plus haute que la
/// table.
#[component]
pub fn PanneauPalette(etat: Etat, bouton_palette: NodeRef<html::Button>) -> impl IntoView {
    let ouverte = Memo::new(move |_| etat.palette.get() && etat.touchee.get().is_some());
    // La fiche : sa largeur en mm et son SVG, ou le débordement qui l'empêche.
    let fiche = Memo::new(move |_| {
        let p = etat.touchee.get()?;
        etat.table.with(|t| {
            t.case(p).map(|c| {
                let rendu = fiche_d_accord(&c.accord, t.tonalite, etat.instrument.get())
                    .map(|page| (page.largeur, page.svg()));
                (t.nom(c), rendu)
            })
        })
    });
    let fermer = NodeRef::<html::Button>::new();
    // La palette ouverte prend le focus sur « Fermer » ; fermée, elle le rend à « Palette ».
    Effect::new(move |_| {
        if let Some(b) = fermer.get() {
            let _ = b.focus();
        }
    });
    let fermer_la_palette = move |_| {
        etat.fermer_palette();
        if let Some(b) = bouton_palette.get_untracked() {
            let _ = b.focus();
        }
    };
    let contenu = move || {
        fiche.get().map(|(nom, rendu)| match rendu {
            Ok((largeur, svg)) => view! {
                <div
                    class="fiche"
                    role="img"
                    aria-label=format!("palette de {nom}")
                    style=format!("width: calc({largeur} * var(--mm))")
                    inner_html=svg
                ></div>
            }
            .into_any(),
            Err(e) => view! { <p class="erreur" role="alert">{e.to_string()}</p> }.into_any(),
        })
    };
    view! {
        <div class="place-palette">
            <Show when=move || ouverte.get()>
                <aside id="palette" class="panneau" aria-label="palette">
                    <div class="panneau-haut">
                        <button type="button" node_ref=fermer on:click=fermer_la_palette>
                            "Fermer"
                        </button>
                    </div>
                    <div class="defile">{contenu}</div>
                </aside>
            </Show>
        </div>
    }
}
