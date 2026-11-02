//! La table à l'établi : la page que le noyau dessine, chaque case recouverte d'un bouton
//! transparent, et le bouton « Palette » au bas de la case touchée.

use leptos::html;
use leptos::prelude::*;
use noyau::rendu::pages::{HAUTEUR, LARGEUR, Rect};
use noyau::rendu::table::{PageTable, page_table};
use noyau::rendu::{Debordement, Instrument};
use noyau::table::{Position, Table};

use crate::app::Etat;

/// Ce qui recouvre une case : sa place sur la page, et son libellé, « A7 · V7/ii ».
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Recouvrement {
    pub position: Position,
    pub rect: Rect,
    pub libelle: String,
}

/// La page de la table, la cible de la case touchée en inverse. Une case sans cible ne met rien
/// en inverse (vigilance 3).
pub(crate) fn rendre(
    table: &Table,
    instrument: Instrument,
    touchee: Option<Position>,
) -> Result<PageTable, Debordement> {
    page_table(table, instrument, touchee.and_then(|p| table.cible_de(p)))
}

/// Un recouvrement par case, dans l'ordre de la disposition : celui où Tab les atteint.
pub(crate) fn recouvrements(table: &Table, page: &PageTable) -> Vec<Recouvrement> {
    page.disposition
        .cases
        .iter()
        .filter_map(|&(position, rect)| {
            table.case(position).map(|c| Recouvrement {
                position,
                rect,
                libelle: format!("{} · {}", table.nom(c), c.chiffre),
            })
        })
        .collect()
}

/// Ce que la case touchée dit de plus aux lecteurs d'écran : « cible : Dm ». La cible ne se
/// voit qu'en inverse ; une case sans cible ne dit rien de plus.
pub(crate) fn dire_la_cible(table: &Table, touchee: Position) -> Option<String> {
    table
        .cible_de(touchee)
        .and_then(|p| table.case(p))
        .map(|c| format!("cible : {}", table.nom(c)))
}

/// L'identifiant du texte qui dit la cible : une seule case est touchée à la fois.
const ID_CIBLE: &str = "cible-de-la-case-touchee";

/// Un pour-cent, à trois décimales.
fn pc(v: f32) -> String {
    format!("{:.3}%", 100.0 * v)
}

/// La place d'un bouton sur une case, en pour-cent de la page : la page suit la fenêtre, le
/// bouton la suit.
pub(crate) fn style_du_recouvrement(r: Rect) -> String {
    format!(
        "left: {}; top: {}; width: {}; height: {}",
        pc(r.x / LARGEUR),
        pc(r.y / HAUTEUR),
        pc(r.l / LARGEUR),
        pc(r.h / HAUTEUR)
    )
}

/// Le bouton « Palette », en mm de la page : sa taille, et son retrait du bord de la case.
const PALETTE_L: f32 = 15.0;
const PALETTE_H: f32 = 4.6;
const PALETTE_RETRAIT: f32 = 0.4;

/// Le rectangle du bouton « Palette », en mm de la page : dans la case touchée, centré, au bas
/// de la case, sans en toucher le bord. Il ne couvre ni une autre case, ni la cible, ni un nom
/// de rangée ; il cache le bas de la case touchée, sa ligne d'effet ou le haut de sa pointe, que
/// la palette redit.
pub(crate) fn rect_de_la_palette(r: Rect) -> Rect {
    let l = PALETTE_L.min(r.l - 2.0 * PALETTE_RETRAIT);
    Rect {
        x: r.x + (r.l - l) / 2.0,
        y: r.y + r.h - PALETTE_RETRAIT - PALETTE_H,
        l,
        h: PALETTE_H,
    }
}

/// La place du bouton « Palette », en pour-cent de la page, comme celle des cases.
pub(crate) fn style_de_la_palette(r: Rect) -> String {
    style_du_recouvrement(rect_de_la_palette(r))
}

/// La table : la page du noyau, ses boutons par-dessus.
#[component]
pub fn VueTable(etat: Etat, bouton_palette: NodeRef<html::Button>) -> impl IntoView {
    let rendu = Memo::new(move |_| {
        etat.table
            .with(|t| rendre(t, etat.instrument.get(), etat.touchee.get()))
    });
    // La case touchée ne déplace aucune case : les recouvrements restent égaux, ne se redessinent
    // pas, et le bouton qui a le focus le garde.
    let cases = Memo::new(move |_| {
        rendu.with(|r| match r {
            Ok(page) => etat.table.with(|t| recouvrements(t, page)),
            Err(_) => Vec::new(),
        })
    });
    let svg = move || rendu.with(|r| r.as_ref().map(|page| page.page.svg()).unwrap_or_default());
    let erreur = move || {
        rendu.with(|r| {
            r.as_ref()
                .err()
                .map(|e| view! { <p class="erreur" role="alert">{e.to_string()}</p> })
        })
    };
    view! {
        <div class="table">
            <div class="page" inner_html=svg></div>
            {erreur}
            {move || {
                cases
                    .get()
                    .into_iter()
                    .map(|c| view! { <BoutonsDeCase recouvrement=c etat=etat bouton_palette=bouton_palette /> })
                    .collect_view()
            }}
        </div>
    }
}

/// Le bouton d'une case, et, quand la case est touchée, le texte qui dit sa cible et le bouton
/// « Palette », juste après lui : Tab va de l'un à l'autre.
#[component]
fn BoutonsDeCase(
    recouvrement: Recouvrement,
    etat: Etat,
    bouton_palette: NodeRef<html::Button>,
) -> impl IntoView {
    let Recouvrement {
        position,
        rect,
        libelle,
    } = recouvrement;
    let touchee = move || etat.touchee.get() == Some(position);
    let cible = move || {
        touchee()
            .then(|| etat.table.with(|t| dire_la_cible(t, position)))
            .flatten()
    };
    view! {
        <button
            type="button"
            class="case"
            style=style_du_recouvrement(rect)
            aria-label=libelle
            aria-current=move || touchee().then_some("true")
            aria-describedby=move || cible().map(|_| ID_CIBLE)
            on:click=move |_| etat.toucher(position)
        ></button>
        {move || cible().map(|texte| view! { <span id=ID_CIBLE hidden=true>{texte}</span> })}
        {move || {
            touchee()
                .then(|| {
                    view! {
                        <button
                            type="button"
                            class="bouton-palette"
                            style=style_de_la_palette(rect)
                            node_ref=bouton_palette
                            aria-controls=move || etat.palette.get().then_some("palette")
                            aria-expanded=move || etat.palette.get().to_string()
                            on:click=move |_| etat.ouvrir_palette()
                        >
                            "Palette"
                        </button>
                    }
                })
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noyau::table::{CoucheSombre, Rangee, table};
    use noyau::tonalite::Tonalite;

    fn do_majeur() -> Tonalite {
        Tonalite::les_24()[0]
    }

    fn la_mineur() -> Tonalite {
        Tonalite::les_24()[1]
    }

    fn position_de(t: &Table, nom: &str) -> Position {
        t.cases
            .iter()
            .find(|c| t.nom(c) == nom)
            .unwrap_or_else(|| panic!("{nom} n'est pas dans la table"))
            .position
    }

    fn svg(t: &Table, touchee: Option<Position>) -> String {
        rendre(t, Instrument::LesDeux, touchee)
            .expect("la page tient")
            .page
            .svg()
    }

    #[test]
    fn toucher_a7_en_do_majeur_met_dm_en_inverse() {
        let t = table(do_majeur(), CoucheSombre::Masquee);
        let a7 = position_de(&t, "A7");
        let dm = position_de(&t, "Dm");
        assert_eq!(t.cible_de(a7), Some(dm));
        let attendue = page_table(&t, Instrument::LesDeux, Some(dm)).expect("la page tient");
        assert_eq!(svg(&t, Some(a7)), attendue.page.svg());
        assert_ne!(svg(&t, Some(a7)), svg(&t, None));
    }

    #[test]
    fn toucher_une_case_sans_cible_ne_met_rien_en_inverse() {
        // Vigilance 3 : C+, le III+ de la mineur, n'a pas de cible ; un diatonique non plus.
        let t = table(la_mineur(), CoucheSombre::Affichee);
        for nom in ["C+", "Am"] {
            let p = position_de(&t, nom);
            assert_eq!(t.cible_de(p), None, "{nom}");
            assert_eq!(svg(&t, Some(p)), svg(&t, None), "{nom}");
        }
    }

    #[test]
    fn la_case_touchee_dit_sa_cible_et_une_case_sans_cible_rien_de_plus() {
        let t = table(do_majeur(), CoucheSombre::Masquee);
        assert_eq!(
            dire_la_cible(&t, position_de(&t, "A7")),
            Some("cible : Dm".to_string())
        );
        let t = table(la_mineur(), CoucheSombre::Affichee);
        for nom in ["C+", "Am"] {
            assert_eq!(dire_la_cible(&t, position_de(&t, nom)), None, "{nom}");
        }
    }

    #[test]
    fn chaque_case_est_recouverte_et_dit_son_nom_et_son_chiffre() {
        for t in Tonalite::les_24() {
            for couche in [CoucheSombre::Masquee, CoucheSombre::Affichee] {
                let tb = table(t, couche);
                let page = rendre(&tb, Instrument::LesDeux, None).expect("la page tient");
                let r = recouvrements(&tb, &page);
                // Un bouton par case de la page, et une case de la page par case de la table.
                assert_eq!(
                    r.len(),
                    page.disposition.cases.len(),
                    "{} {couche:?}",
                    t.nom()
                );
                assert_eq!(r.len(), tb.cases.len(), "{} {couche:?}", t.nom());
                for (rec, (position, _)) in r.iter().zip(&page.disposition.cases) {
                    assert_eq!(rec.position, *position, "dans l'ordre de la disposition");
                }
            }
        }
        // A7, le V7/ii de do majeur, au-dessus de ii.
        let tb = table(do_majeur(), CoucheSombre::Masquee);
        let page = rendre(&tb, Instrument::LesDeux, None).expect("la page tient");
        let a7 = recouvrements(&tb, &page)
            .into_iter()
            .find(|r| r.position.rangee == Rangee::Dessus && r.position.rang == 1)
            .expect("A7 est recouvert");
        assert_eq!(a7.libelle, "A7 · V7/ii");
    }

    /// Deux rectangles qui ont une surface en commun ; se toucher par un bord ne compte pas.
    fn recoupe(a: Rect, b: Rect) -> bool {
        a.x < b.x + b.l && b.x < a.x + a.l && a.y < b.y + b.h && b.y < a.y + a.h
    }

    #[test]
    fn palette_ne_couvre_aucune_autre_case_et_reste_dans_la_page() {
        for t in Tonalite::les_24() {
            for couche in [CoucheSombre::Masquee, CoucheSombre::Affichee] {
                let tb = table(t, couche);
                let page = rendre(&tb, Instrument::LesDeux, None).expect("la page tient");
                let cases = &page.disposition.cases;
                for &(touchee, r) in cases {
                    let p = rect_de_la_palette(r);
                    let ou = || format!("{} {couche:?}, {touchee:?} : {p:?}", t.nom());
                    assert!(
                        p.x >= 0.0 && p.y >= 0.0 && p.x + p.l <= LARGEUR && p.y + p.h <= HAUTEUR,
                        "hors de la page : {}",
                        ou()
                    );
                    assert!(
                        p.x >= r.x
                            && p.y >= r.y
                            && p.x + p.l <= r.x + r.l
                            && p.y + p.h <= r.y + r.h,
                        "hors de la case touchée : {}",
                        ou()
                    );
                    for &(autre, q) in cases.iter().filter(|(pos, _)| *pos != touchee) {
                        assert!(!recoupe(p, q), "{} couvre {autre:?}", ou());
                    }
                }
            }
        }
    }

    #[test]
    fn le_recouvrement_se_place_en_pour_cent_de_la_page() {
        let r = Rect {
            x: 20.95,
            y: 15.7,
            l: 41.9,
            h: 31.4,
        };
        assert_eq!(
            style_du_recouvrement(r),
            "left: 10.000%; top: 10.000%; width: 20.000%; height: 20.000%"
        );
    }

    #[test]
    fn palette_se_pose_au_bas_de_la_case_touchee() {
        let r = Rect {
            x: 20.0,
            y: 30.0,
            l: 27.0,
            h: 20.0,
        };
        // Centré, 0,4 mm au-dessus du bas de la case.
        let p = rect_de_la_palette(r);
        for (vu, attendu) in [(p.x, 26.0), (p.y, 45.0), (p.l, 15.0), (p.h, 4.6)] {
            assert!((vu - attendu).abs() < 1e-4, "{p:?}");
        }
        assert_eq!(
            style_de_la_palette(r),
            style_du_recouvrement(rect_de_la_palette(r))
        );
    }
}
