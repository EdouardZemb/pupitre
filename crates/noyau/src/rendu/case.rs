//! Les cases de la table : les quatre cadres, la pastille de fonction, la flèche de
//! résolution (§ 50 · 3.7).
//!
//! Les quatre cadres sont au même trait fin : droit (diatoniques), en pointe vers la cible
//! (dominantes secondaires), aux coins arrondis (emprunts), aux coins coupés (harmonie
//! sombre). Ils se distinguent par la forme seule, en noir et blanc (I17).
//!
//! Ce trait vaut 0,5 pt, et non les 0,25 pt de la fiche témoin : à 0,25 pt, deux cadres d'une
//! case sombre (23,6 × 35 mm) ne diffèrent que par 1,4 % des pixels de leur boîte, sous les
//! 2 % d'I17 ; à 0,5 pt, la pire paire en diffère de 2,1 %.

use std::f32::consts::SQRT_2;

use super::pages::Rect;
use super::svg::{Couleur, Element, TRAIT_FIN, nombre};
use crate::table::{Famille, Fonction};

/// Une case ordinaire, ou la cible de la case touchée à l'établi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Etat {
    Ordinaire,
    /// En inverse, fond noir et texte blanc, redoublée d'un double cadre fin.
    Cible,
}

/// La profondeur de la pointe d'une dominante secondaire, prise dans le bas de sa case.
pub(crate) const POINTE: f32 = 3.0;
/// Le diamètre d'une pastille de fonction (§ 50 · 6.3).
pub(crate) const PASTILLE: f32 = 3.0;
/// 0,5 pt : le trait des quatre cadres, le même pour tous (R21).
const TRAIT_CADRE: f32 = 2.0 * TRAIT_FIN;
/// 0,5 pt : le trait d'une flèche.
const TRAIT_FLECHE: f32 = 2.0 * TRAIT_FIN;
/// La longueur de la pointe d'une flèche.
const LONGUEUR_POINTE: f32 = 1.8;
/// La demi-largeur de la pointe d'une flèche.
pub(crate) const DEMI_POINTE: f32 = 0.8;
/// L'écart entre le cadre d'une case cible et son second cadre, en dedans.
const DOUBLE: f32 = 0.7;
/// Le trait d'une flèche entre un peu dans sa pointe, pour ne laisser aucun jour.
const RECOUVREMENT: f32 = 0.3;

fn n(v: f32) -> String {
    nombre(v, 3)
}

/// Vrai pour les cadres aux coins arrondis (emprunts) ou coupés (harmonie sombre).
fn a_des_coins(famille: Famille) -> bool {
    matches!(famille, Famille::Emprunt | Famille::Sombre)
}

/// L'arrondi ou la coupe des coins d'une case : le quart de son petit côté (R10). Un cadre
/// droit ou en pointe n'en a pas.
pub(crate) fn coin(famille: Famille, r: Rect) -> f32 {
    if a_des_coins(famille) {
        r.l.min(r.h) / 4.0
    } else {
        0.0
    }
}

/// Le tracé d'un cadre dans `r`. La pointe d'une dominante secondaire descend jusqu'au bas
/// de `r`, `pointe` sous le bas de la boîte.
fn trace(famille: Famille, r: Rect, coin: f32, pointe: f32) -> String {
    let (x, y, x1, y1) = (r.x, r.y, r.x + r.l, r.y + r.h);
    let c = coin;
    match famille {
        Famille::Diatonique => format!("M{} {}H{}V{}H{}Z", n(x), n(y), n(x1), n(y1), n(x)),
        Famille::DominanteSecondaire => {
            let bas = y1 - pointe;
            format!(
                "M{} {}H{}V{}L{} {}L{} {}Z",
                n(x),
                n(y),
                n(x1),
                n(bas),
                n(x + r.l / 2.0),
                n(y1),
                n(x),
                n(bas)
            )
        }
        Famille::Emprunt => {
            let a = format!("A{} {} 0 0 1", n(c), n(c));
            format!(
                "M{} {}H{}{a} {} {}V{}{a} {} {}H{}{a} {} {}V{}{a} {} {}Z",
                n(x + c),
                n(y),
                n(x1 - c),
                n(x1),
                n(y + c),
                n(y1 - c),
                n(x1 - c),
                n(y1),
                n(x + c),
                n(x),
                n(y1 - c),
                n(y + c),
                n(x + c),
                n(y)
            )
        }
        Famille::Sombre => format!(
            "M{} {}H{}L{} {}V{}L{} {}H{}L{} {}V{}Z",
            n(x + c),
            n(y),
            n(x1 - c),
            n(x1),
            n(y + c),
            n(y1 - c),
            n(x1 - c),
            n(y1),
            n(x + c),
            n(x),
            n(y1 - c),
            n(y + c)
        ),
    }
}

/// Le second cadre d'une case cible, `DOUBLE` en dedans du premier : les côtés sont
/// parallèles, l'arrondi et la coupe suivent.
fn trace_dedans(famille: Famille, r: Rect, coin: f32, pointe: f32) -> String {
    let d = DOUBLE;
    let dedans = Rect {
        x: r.x + d,
        y: r.y + d,
        l: r.l - 2.0 * d,
        h: r.h - 2.0 * d,
    };
    match famille {
        Famille::Diatonique => trace(famille, dedans, 0.0, 0.0),
        Famille::Emprunt => trace(famille, dedans, (coin - d).max(0.0), 0.0),
        // Une coupe à 45° décalée de d : sa trace sur chaque côté recule de d (2 - √2).
        Famille::Sombre => trace(famille, dedans, (coin - d * (2.0 - SQRT_2)).max(0.0), 0.0),
        Famille::DominanteSecondaire => {
            // Chaque côté de la pointe, décalé de d vers l'intérieur.
            let demi = r.l / 2.0;
            let pente = pointe / demi;
            let recul = d * (1.0 + pente * pente).sqrt();
            let (x, x1, y1) = (dedans.x, dedans.x + dedans.l, r.y + r.h);
            let cote = y1 - pente * (demi - d) - recul;
            format!(
                "M{} {}H{}V{}L{} {}L{} {}Z",
                n(x),
                n(dedans.y),
                n(x1),
                n(cote),
                n(r.x + demi),
                n(y1 - recul),
                n(x),
                n(cote)
            )
        }
    }
}

/// Un cadre aux coins donnés : ce qu'emploient la case et le pictogramme de la légende.
fn cadre_aux_coins(famille: Famille, r: Rect, etat: Etat, coin: f32, pointe: f32) -> Element {
    let d = trace(famille, r, coin, pointe);
    let fin = Some((Couleur::Noir, TRAIT_CADRE));
    match etat {
        Etat::Ordinaire => Element::Chemin {
            d,
            plein: Some(Couleur::Blanc),
            trait_: fin,
        },
        Etat::Cible => Element::Groupe {
            id: None,
            translation: (0.0, 0.0),
            echelle: None,
            enfants: vec![
                Element::Chemin {
                    d,
                    plein: Some(Couleur::Noir),
                    trait_: fin,
                },
                Element::Chemin {
                    d: trace_dedans(famille, r, coin, pointe),
                    plein: None,
                    trait_: Some((Couleur::Blanc, TRAIT_CADRE)),
                },
            ],
        },
    }
}

/// Le cadre d'une case de la famille, dans `r` ; la pointe d'une dominante secondaire est
/// en bas, dans `r`. Une cible est en inverse, redoublée d'un second cadre fin, en dedans.
pub fn cadre(famille: Famille, r: Rect, etat: Etat) -> Element {
    cadre_aux_coins(famille, r, etat, coin(famille, r), POINTE)
}

/// La largeur d'un pictogramme de la légende.
pub(crate) const PICTOGRAMME_L: f32 = 10.0;
/// La hauteur d'un pictogramme de la légende : celle d'une ligne à 11 pt.
pub(crate) const PICTOGRAMME_H: f32 = 4.6;
/// L'arrondi et la coupe d'un pictogramme, bien au-delà du quart de sa hauteur : au quart
/// d'un pictogramme de 4,2 mm, les coins de 1,05 mm de la fiche témoin ne distinguaient pas
/// l'emprunt de l'harmonie sombre sans lire leur nom (H09).
const PICTOGRAMME_COIN: f32 = 2.0;
/// La pointe d'un pictogramme de dominante secondaire.
const PICTOGRAMME_POINTE: f32 = 1.8;

/// Le pictogramme d'un cadre, pour la légende, dont le coin haut gauche est en (x, y).
pub fn pictogramme(famille: Famille, x: f32, y: f32) -> Element {
    let r = Rect {
        x,
        y,
        l: PICTOGRAMME_L,
        h: PICTOGRAMME_H,
    };
    let coin = if a_des_coins(famille) {
        PICTOGRAMME_COIN
    } else {
        0.0
    };
    cadre_aux_coins(famille, r, Etat::Ordinaire, coin, PICTOGRAMME_POINTE)
}

/// ○ repos, ◐ éloignement, ● tension : 3 mm de diamètre, au même trait fin.
pub fn pastille(f: Fonction, cx: f32, cy: f32) -> Element {
    let r = PASTILLE / 2.0;
    let plein = match f {
        Fonction::Tension => Couleur::Noir,
        Fonction::Repos | Fonction::Eloignement => Couleur::Blanc,
    };
    let mut enfants = vec![Element::Cercle {
        cx,
        cy,
        r,
        id: Some("pastille"),
        plein: Some(plein),
        trait_: Some((Couleur::Noir, TRAIT_FIN)),
    }];
    if f == Fonction::Eloignement {
        // La moitié gauche, pleine.
        enfants.push(Element::Chemin {
            d: format!(
                "M{} {}A{} {} 0 0 0 {} {}Z",
                n(cx),
                n(cy - r),
                n(r),
                n(r),
                n(cx),
                n(cy + r)
            ),
            plein: Some(Couleur::Noir),
            trait_: None,
        });
    }
    Element::Groupe {
        id: None,
        translation: (0.0, 0.0),
        echelle: None,
        enfants,
    }
}

/// La direction de `de` vers `a`, et le milieu de la base de la pointe posée en `a`.
fn axe(de: (f32, f32), a: (f32, f32)) -> ((f32, f32), (f32, f32)) {
    let (dx, dy) = (a.0 - de.0, a.1 - de.1);
    let longueur = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = (dx / longueur, dy / longueur);
    (
        (ux, uy),
        (a.0 - ux * LONGUEUR_POINTE, a.1 - uy * LONGUEUR_POINTE),
    )
}

/// Les trois sommets de la pointe d'une flèche de `de` à `a` : le sommet posé en `a`, puis
/// les deux coins de sa base.
pub(crate) fn sommets_de_pointe(de: (f32, f32), a: (f32, f32)) -> [(f32, f32); 3] {
    let ((ux, uy), base) = axe(de, a);
    [
        a,
        (base.0 - uy * DEMI_POINTE, base.1 + ux * DEMI_POINTE),
        (base.0 + uy * DEMI_POINTE, base.1 - ux * DEMI_POINTE),
    ]
}

/// Une flèche de résolution, de `de` à `a` : un groupe `fleche`, son trait plein, puis sa
/// pointe pleine, dont le sommet se pose en `a`, sur la cible.
pub fn fleche(de: (f32, f32), a: (f32, f32)) -> Element {
    debug_assert!(de != a, "une flèche relie deux points distincts");
    let ((ux, uy), base) = axe(de, a);
    let fin = (base.0 + ux * RECOUVREMENT, base.1 + uy * RECOUVREMENT);
    let [s, g, d] = sommets_de_pointe(de, a);
    Element::Groupe {
        id: Some("fleche"),
        translation: (0.0, 0.0),
        echelle: None,
        enfants: vec![
            Element::Ligne {
                de,
                a: fin,
                trait_: (Couleur::Noir, TRAIT_FLECHE),
            },
            Element::Chemin {
                d: format!(
                    "M{} {}L{} {}L{} {}Z",
                    n(s.0),
                    n(s.1),
                    n(g.0),
                    n(g.1),
                    n(d.0),
                    n(d.1)
                ),
                plein: Some(Couleur::Noir),
                trait_: None,
            },
        ],
    }
}
