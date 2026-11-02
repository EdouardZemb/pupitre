//! Le diagramme de clavier (REN-08) et la palette sur le clavier (MIC-13), à la géométrie
//! qu'H09 a validée : des touches blanches de 3 mm, les noires par-dessus.
//!
//! Un clavier part de son coin haut gauche. Ses touches blanches sont des `touche-blanche`, ses
//! noires des `touche-noire`, de gauche à droite.

use super::case::{Support, marque};
use super::grille::{interligne, jusqua_la_base};
use super::pages::Rect;
use super::svg::{Couleur, Element, TRAIT_MOYEN};
use super::texte::{Corps, chasse, composer};
use crate::clavier::Voicing;
use crate::hauteur::NomDeNote;
use crate::palette::NoteDePalette;

/// La largeur d'une touche blanche, au minimum du § 50 · 6.3.
const TOUCHE: f32 = 3.0;
/// La largeur d'une touche noire.
const NOIRE: f32 = 1.8;
/// La hauteur du diagramme, et celle de ses touches noires.
const DIAGRAMME_H: f32 = 11.0;
const DIAGRAMME_NOIRE_H: f32 = 7.0;
/// La hauteur de la palette, et celle de ses touches noires.
pub(crate) const PALETTE_H: f32 = 9.5;
const PALETTE_NOIRE_H: f32 = 6.0;
/// Le rayon du point d'une note jouée, sur une touche blanche et sur une noire.
const POINT_BLANCHE: f32 = 0.85;
const POINT_NOIRE: f32 = 0.65;
/// La hauteur d'un point, au-dessus du bas de sa touche.
const POINT_SUR_BLANCHE: f32 = 2.1;
const POINT_SUR_NOIRE: f32 = 1.5;
/// La hauteur d'une marque de palette, au-dessus du bas de sa touche.
const MARQUE_SUR_BLANCHE: f32 = 1.8;
const MARQUE_SUR_NOIRE: f32 = 1.3;
/// Le blanc entre le clavier et la lettre de la basse.
const SOUS_LE_CLAVIER: f32 = 0.5;
/// Le do 3 (MIDI 60) : la palette se dessine dans l'octave qui part de sa fondamentale, de do 3
/// à si 3 ; seule compte la place des touches.
const DO_3: u8 = 60;

fn est_blanche(midi: u8) -> bool {
    matches!(midi % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
}

/// La touche blanche qui précède `midi`.
fn blanche_avant(midi: u8) -> u8 {
    (0..midi).rev().find(|m| est_blanche(*m)).unwrap_or(0)
}

/// La touche blanche qui suit `midi`.
fn blanche_apres(midi: u8) -> u8 {
    (midi.saturating_add(1)..=u8::MAX)
        .find(|m| est_blanche(*m))
        .unwrap_or(midi)
}

/// Une touche dessinée : sa note, l'abscisse de son milieu, et sa couleur.
struct Touche {
    midi: u8,
    x: f32,
    blanche: bool,
}

/// Les touches de la blanche `de` à la blanche `a` : les blanches côte à côte, puis les noires
/// par-dessus, à cheval sur deux blanches.
fn touches(de: u8, a: u8, hauteur: f32, noire_h: f32) -> (Vec<Element>, Vec<Touche>) {
    let mut blanches = Vec::new();
    let mut noires = Vec::new();
    let mut centres = Vec::new();
    let mut x = 0.0;
    for midi in de..=a {
        if est_blanche(midi) {
            blanches.push(Element::Rect {
                r: Rect {
                    x,
                    y: 0.0,
                    l: TOUCHE,
                    h: hauteur,
                },
                id: Some("touche-blanche"),
                plein: Some(Couleur::Blanc),
                trait_: Some((Couleur::Noir, TRAIT_MOYEN)),
            });
            centres.push(Touche {
                midi,
                x: x + TOUCHE / 2.0,
                blanche: true,
            });
            x += TOUCHE;
        } else {
            noires.push(Element::Rect {
                r: Rect {
                    x: x - NOIRE / 2.0,
                    y: 0.0,
                    l: NOIRE,
                    h: noire_h,
                },
                id: Some("touche-noire"),
                plein: Some(Couleur::Noir),
                trait_: None,
            });
            centres.push(Touche {
                midi,
                x,
                blanche: false,
            });
        }
    }
    blanches.extend(noires);
    (blanches, centres)
}

fn largeur(de: u8, a: u8) -> f32 {
    (de..=a).filter(|m| est_blanche(*m)).count() as f32 * TOUCHE
}

/// Les bornes du diagramme : la touche blanche qui précède la note la plus grave, celle qui suit
/// la plus aiguë.
fn bornes_du_diagramme(v: &Voicing) -> Option<(u8, u8)> {
    Some((
        blanche_avant(*v.notes.first()?),
        blanche_apres(*v.notes.last()?),
    ))
}

/// Les bornes de la palette : une octave dès la fondamentale, une touche blanche de marge de
/// chaque côté.
fn bornes_de_la_palette(p: &[NoteDePalette]) -> Option<(u8, u8, u8)> {
    let racine = DO_3 + p.first()?.nom.classe();
    Some((blanche_avant(racine), racine, blanche_apres(racine + 11)))
}

/// Le diagramme de clavier (REN-08) : de la touche blanche qui précède la note la plus grave à
/// celle qui suit la plus aiguë. Une note jouée est un point plein, blanc sur une touche noire ;
/// la basse est une lettre sous sa touche.
pub fn diagramme_de_clavier(v: &Voicing, basse: NomDeNote, origine: (f32, f32)) -> Element {
    let mut e = Vec::new();
    if let Some((de, a)) = bornes_du_diagramme(v) {
        let t = Corps::MIN;
        let (dessin, centres) = touches(de, a, DIAGRAMME_H, DIAGRAMME_NOIRE_H);
        e = dessin;
        for touche in centres.iter().filter(|k| v.notes.contains(&k.midi)) {
            let (y, r, plein) = if touche.blanche {
                (
                    DIAGRAMME_H - POINT_SUR_BLANCHE,
                    POINT_BLANCHE,
                    Couleur::Noir,
                )
            } else {
                (
                    DIAGRAMME_NOIRE_H - POINT_SUR_NOIRE,
                    POINT_NOIRE,
                    Couleur::Blanc,
                )
            };
            e.push(Element::Cercle {
                cx: touche.x,
                cy: y,
                r,
                id: None,
                plein: Some(plein),
                trait_: None,
            });
            if Some(&touche.midi) == v.notes.first() {
                let lettre = basse.anglais();
                e.push(composer(
                    &lettre,
                    &t,
                    touche.x - chasse(&lettre, &t) / 2.0,
                    DIAGRAMME_H + SOUS_LE_CLAVIER + jusqua_la_base(&t),
                ));
            }
        }
    }
    Element::Groupe {
        id: Some("clavier"),
        translation: origine,
        echelle: None,
        enfants: e,
    }
}

/// La largeur du diagramme de clavier.
pub(crate) fn largeur_du_diagramme(v: &Voicing) -> f32 {
    bornes_du_diagramme(v).map_or(0.0, |(de, a)| largeur(de, a))
}

/// La hauteur du diagramme de clavier, la lettre de la basse comprise.
pub(crate) fn hauteur_du_diagramme() -> f32 {
    DIAGRAMME_H + SOUS_LE_CLAVIER + interligne(&Corps::MIN)
}

/// La palette sur le clavier (MIC-13) : une octave à partir de la fondamentale, avec une touche
/// blanche de marge de chaque côté ; chaque note par la forme de son rôle, en blanc sur une
/// touche noire.
pub fn palette_sur_le_clavier(p: &[NoteDePalette], origine: (f32, f32)) -> Element {
    let mut e = Vec::new();
    if let Some((de, racine, a)) = bornes_de_la_palette(p) {
        let (dessin, centres) = touches(de, a, PALETTE_H, PALETTE_NOIRE_H);
        e = dessin;
        for touche in centres
            .iter()
            .filter(|k| (racine..=racine + 11).contains(&k.midi))
        {
            let Some(note) = p.iter().find(|n| n.nom.classe() == touche.midi % 12) else {
                continue;
            };
            let (y, support) = if touche.blanche {
                (PALETTE_H - MARQUE_SUR_BLANCHE, Support::ToucheBlanche)
            } else {
                (PALETTE_NOIRE_H - MARQUE_SUR_NOIRE, Support::ToucheNoire)
            };
            e.push(marque(note.role, (touche.x, y), support));
        }
    }
    Element::Groupe {
        id: Some("clavier"),
        translation: origine,
        echelle: None,
        enfants: e,
    }
}

/// La largeur de la palette sur le clavier : neuf touches blanches, quelle que soit la
/// fondamentale.
pub(crate) fn largeur_de_la_palette(p: &[NoteDePalette]) -> f32 {
    bornes_de_la_palette(p).map_or(0.0, |(de, _, a)| largeur(de, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_octave_et_ses_marges_font_toujours_neuf_touches_blanches() {
        for racine in DO_3..DO_3 + 12 {
            let (de, a) = (blanche_avant(racine), blanche_apres(racine + 11));
            assert_eq!(largeur(de, a), 9.0 * TOUCHE, "{racine}");
        }
    }
}
