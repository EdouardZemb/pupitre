//! La palette sur une portée d'une octave (MIC-13, R11). Sans clé, la portée ne dit pas la
//! hauteur absolue : la fondamentale est toujours dans le premier interligne, et chaque note
//! monte de l'écart de sa lettre à celle de la fondamentale. Deux notes d'une même lettre (do
//! et do♯) se posent côte à côte, à la même hauteur : aucune altération n'est dessinée, le nom
//! de la note, dessous, la porte. Les noms sont à 11 pt, sur des lignes en quinconce : deux,
//! ou plus s'il le faut.
//!
//! Une portée part de son coin haut gauche, sa ligne du haut.

use super::case::{Support, marque};
use super::grille::{COLONNE, RETRAIT, interligne, jusqua_la_base};
use super::svg::{Couleur, Element, TRAIT_FIN};
use super::texte::{Corps, chasse, composer};
use crate::hauteur::NomDeNote;
use crate::palette::NoteDePalette;

/// L'écart de deux lignes de la portée.
const ESPACE: f32 = 1.8;
/// Le blanc entre deux noms voisins d'une même ligne : 0,8 mm, et non le millimètre de l'essai
/// H09, qui demandait une troisième ligne de noms à sol majeur et à mi mineur pour 0,03 mm.
const JOUR: f32 = 0.8;
/// Le blanc entre la ligne du bas et les noms.
const SOUS_LA_PORTEE: f32 = 0.6;
/// La largeur d'une portée dans une colonne de la table, ses retraits ôtés.
pub(crate) const LARGEUR_PORTEE: f32 = COLONNE - 2.0 * RETRAIT;
/// La précision des comparaisons de largeur.
const EPS: f32 = 1e-4;

/// Le pas des notes et l'abscisse de la première, pour `n` notes et `lignes` lignes de noms :
/// deux noms d'une même ligne sont à `lignes` pas, et les noms extrêmes ne débordent pas la
/// portée quand chaque nom prend la place que lui laissent ses voisins de ligne.
fn pas(n: usize, largeur: f32, lignes: usize) -> (f32, f32) {
    let pas = (largeur + JOUR) / (n + lignes - 1) as f32;
    (pas, (lignes as f32 * pas - JOUR) / 2.0)
}

/// Vrai si chaque nom tient sous sa note : dans la largeur, et à un jour de son voisin de ligne.
pub(crate) fn tiennent(p: &[NoteDePalette], largeur: f32, lignes: usize) -> bool {
    let t = Corps::MIN;
    let (pas, x0) = pas(p.len(), largeur, lignes);
    let bords: Vec<(f32, f32)> = p
        .iter()
        .enumerate()
        .map(|(i, note)| {
            let x = x0 + i as f32 * pas;
            let demi = chasse(&note.nom.solfege(), &t) / 2.0;
            (x - demi, x + demi)
        })
        .collect();
    bords.iter().all(|(g, d)| *g >= -EPS && *d <= largeur + EPS)
        && bords
            .iter()
            .zip(bords.iter().skip(lignes))
            .all(|((_, d), (g, _))| d + JOUR <= g + EPS)
}

/// Le moins de lignes de noms, deux au moins, où chaque nom tient ; rien si même une ligne par
/// nom ne suffit pas.
pub(crate) fn lignes_de_noms(p: &[NoteDePalette], largeur: f32) -> Option<usize> {
    (2..=p.len().max(2)).find(|&k| tiennent(p, largeur, k))
}

/// La hauteur d'une portée et de ses `lignes` lignes de noms.
pub(crate) fn hauteur_de_portee(lignes: usize) -> f32 {
    4.0 * ESPACE + SOUS_LA_PORTEE + lignes as f32 * interligne(&Corps::MIN)
}

/// Le degré d'une note au-dessus de la fondamentale, compté en lettres : 0 pour la
/// fondamentale et pour une note de même lettre au-dessus d'elle (do♯ sur do) ; 7, l'octave,
/// pour une note de même lettre au-dessous (sol sur sol♯), qui sonne une octave plus haut dans
/// la palette.
fn degre(fondamentale: NomDeNote, note: NomDeNote) -> i8 {
    let i = fondamentale.intervalle_vers(note);
    if i.demi_tons < 0 {
        i.degres + 7
    } else {
        i.degres
    }
}

/// La palette sur la portée, sur la largeur et le nombre de lignes de noms donnés.
pub(crate) fn portee(
    p: &[NoteDePalette],
    origine: (f32, f32),
    largeur: f32,
    lignes: usize,
) -> Element {
    let t = Corps::MIN;
    let mut e: Vec<Element> = (0..5)
        .map(|k| {
            let y = k as f32 * ESPACE;
            Element::Ligne {
                de: (0.0, y),
                a: (largeur, y),
                trait_: (Couleur::Noir, TRAIT_FIN),
            }
        })
        .collect();
    let bas = 4.0 * ESPACE;
    if let Some(premiere) = p.first() {
        let (pas, x0) = pas(p.len(), largeur, lignes);
        for (i, note) in p.iter().enumerate() {
            let x = x0 + i as f32 * pas;
            // Le premier interligne, puis un demi-espace par lettre.
            let y = bas - f32::from(1 + degre(premiere.nom, note.nom)) * ESPACE / 2.0;
            e.push(marque(note.role, (x, y), Support::Portee));
            let nom = note.nom.solfege();
            let haut = bas + SOUS_LA_PORTEE + (i % lignes) as f32 * interligne(&t);
            e.push(composer(
                &nom,
                &t,
                x - chasse(&nom, &t) / 2.0,
                haut + jusqua_la_base(&t),
            ));
        }
    }
    Element::Groupe {
        id: Some("portee"),
        translation: origine,
        echelle: None,
        enfants: e,
    }
}

/// La palette sur une portée d'une octave, sans clé, les noms dessous, sur la largeur d'une
/// colonne de la table : la fondamentale dans le premier interligne, les autres notes plus haut
/// de l'écart de leur lettre, chacune par la forme de son rôle.
///
/// Les noms prennent le moins de lignes où chacun tient sous sa note. Si un nom ne tenait pas
/// même seul sur sa ligne, la portée prendrait une ligne par nom, et ce nom déborderait sans le
/// dire : cette fonction rend un dessin, pas un `Debordement`. Aucun nom de note n'en approche
/// (le plus large, « sol♯ », fait 7,4 mm pour une portée de 25,2 mm), et une colonne, elle,
/// vérifie ses noms et rend `Debordement`.
pub fn palette_sur_la_portee(p: &[NoteDePalette], origine: (f32, f32)) -> Element {
    let lignes = lignes_de_noms(p, LARGEUR_PORTEE);
    debug_assert!(
        lignes.is_some(),
        "un nom de note tient toujours seul sur une ligne de la portée"
    );
    portee(p, origine, LARGEUR_PORTEE, lignes.unwrap_or(p.len().max(2)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hauteur::Lettre;

    fn note(lettre: Lettre, alteration: i8) -> NomDeNote {
        NomDeNote { lettre, alteration }
    }

    #[test]
    fn le_degre_se_compte_en_lettres_depuis_la_fondamentale() {
        let (do_, do_diese) = (note(Lettre::C, 0), note(Lettre::C, 1));
        assert_eq!(degre(do_, do_), 0);
        assert_eq!(degre(do_, do_diese), 0, "côte à côte, à la même hauteur");
        assert_eq!(degre(do_, note(Lettre::B, 0)), 6);
        assert_eq!(degre(note(Lettre::A, 0), note(Lettre::C, 1)), 2);
        // Sol sous sol♯ sonne une octave plus haut dans la palette de sol♯.
        assert_eq!(degre(note(Lettre::G, 1), note(Lettre::G, 0)), 7);
    }
}
