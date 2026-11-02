//! L'interface de pupitre. Pour l'essai H10 : une seule page, dessinée par le noyau.

use leptos::prelude::*;
use noyau::rendu::pages::{HAUTEUR, LARGEUR, Rect};
use noyau::rendu::svg::{Couleur, Element, Page, TRAIT_FIN};
use noyau::rendu::texte::{Corps, composer};

/// La page d'essai d'H10 (retirée par la PR 6) : un cadre et un texte en tracés.
fn page_d_essai() -> Page {
    let cadre = Element::Rect {
        r: Rect {
            x: 10.0,
            y: 10.0,
            l: LARGEUR - 20.0,
            h: HAUTEUR - 20.0,
        },
        id: Some("cadre"),
        plein: None,
        trait_: Some((Couleur::Noir, 4.0 * TRAIT_FIN)),
    };
    Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements: vec![
            cadre,
            composer("pupitre · essai H10", &Corps::MIN, 20.0, 40.0),
        ],
    }
}

#[component]
fn Ecran() -> impl IntoView {
    view! { <div class="page" inner_html=page_d_essai().svg()></div> }
}

fn main() {
    mount_to_body(Ecran);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_page_d_essai_est_dans_le_sous_ensemble() {
        assert_eq!(
            noyau::rendu::sous_ensemble::verifier(&page_d_essai().svg()),
            Ok(())
        );
    }
}
