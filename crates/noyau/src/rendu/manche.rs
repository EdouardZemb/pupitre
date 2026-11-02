//! Le diagramme de manche (REN-04) et la palette sur le manche (MIC-13), à la géométrie de la
//! fiche témoin qu'H09 a validée : six cordes à 3,7 mm, soit 18,5 mm de large.
//!
//! Un diagramme part de son coin haut gauche, la corde grave, au haut de la bande des cordes à
//! vide : le sillet ou la première frette est sous cette bande. Le numéro de la frette de
//! départ, s'il y en a un, déborde à gauche de la corde grave. Les points des doigts sont des
//! `point`, le barré un groupe `barre`, les numéros de doigts un groupe `doigts`.

use super::case::{Support, TRAIT_CREUX, croix, marque};
use super::grille::{COLONNE, RETRAIT, interligne, jusqua_la_base, mi_capitale};
use super::svg::{Couleur, Element, TRAIT_MOYEN, nombre};
use super::texte::{Corps, MotTropLong, chasse, composer, couper};
use crate::guitare::{ACCORDAGE, Corde, Doigte, PAS_DE_DOIGTE};
use crate::hauteur::NomDeNote;
use crate::palette::NoteDePalette;

/// L'écart de deux cordes.
const CORDE: f32 = 3.7;
/// La largeur du manche, de la corde grave à l'aiguë, au-delà des 18 mm du § 50 · 6.3.
pub(crate) const LARGEUR_MANCHE: f32 = 5.0 * CORDE;
const _: () = assert!(LARGEUR_MANCHE >= 18.0);
/// La bande des cordes à vide et étouffées, au-dessus du sillet.
const RANG_A_VIDE: f32 = 3.4;
/// La hauteur d'une case de frette, au diagramme.
const CASE_DOIGTE: f32 = 4.0;
/// La hauteur d'une case de frette, à la palette.
const CASE_PALETTE: f32 = 3.3;
/// L'épaisseur du sillet.
const SILLET: f32 = 0.7;
/// Les frettes d'un diagramme : quatre suffisent à une étendue de trois frettes.
const FRETTES: u8 = 4;
/// Le blanc entre la grille et les numéros de doigts.
const SOUS_LA_GRILLE: f32 = 0.6;
/// Le rayon du point d'un doigt : ce qui déborde de la corde extrême.
pub(crate) const POINT: f32 = 1.15;
/// Le rayon du cercle d'une corde à vide.
const A_VIDE: f32 = 0.8;
/// La demi-largeur de la croix d'une corde étouffée.
const ETOUFFEE: f32 = 0.75;
/// La demi-hauteur du trait épais d'un barré.
const BARRE: f32 = 0.75;
/// Le blanc entre le numéro de frette et le point le plus à gauche.
const AVANT_LE_NUMERO: f32 = 0.4;
/// La place d'un texte dans une colonne de la table, ses retraits ôtés : celle du diagramme
/// sans doigté quand nulle colonne ne la donne.
const PLACE_D_UNE_COLONNE: f32 = COLONNE - 2.0 * RETRAIT;

fn n(v: f32) -> String {
    nombre(v, 3)
}

fn x_corde(c: usize) -> f32 {
    c as f32 * CORDE
}

/// Les frettes appuyées d'un doigté.
fn appuyees(d: &Doigte) -> impl Iterator<Item = u8> + '_ {
    d.cordes.iter().filter_map(|c| match c {
        Corde::Frette(f) => Some(*f),
        _ => None,
    })
}

fn frette_haute(d: &Doigte) -> u8 {
    appuyees(d).max().unwrap_or(0)
}

/// Les frettes qu'une grille montre : la première sous le sillet, ou plus haut, et leur
/// nombre.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fenetre {
    premiere: u8,
    frettes: u8,
}

impl Fenetre {
    /// Une grille qui part du sillet a un sillet ; une autre a le numéro de sa première frette.
    fn numero(self) -> Option<u8> {
        (self.premiere > 1).then_some(self.premiere)
    }

    /// Le milieu de la case de la frette `f`, sous le haut de la grille.
    fn y(self, f: u8, case: f32) -> f32 {
        (f32::from(f) - f32::from(self.premiere) + 0.5) * case
    }
}

/// Le diagramme part du sillet si sa frette la plus haute ne dépasse pas la quatrième, ou s'il
/// fait sonner une corde à vide ; sinon, de sa frette appuyée la plus basse.
fn fenetre_du_diagramme(d: &Doigte) -> Fenetre {
    let haute = frette_haute(d);
    if haute <= FRETTES || d.cordes.contains(&Corde::AVide) {
        Fenetre {
            premiere: 1,
            frettes: FRETTES.max(haute),
        }
    } else {
        let basse = appuyees(d).min().unwrap_or(1);
        Fenetre {
            premiere: basse,
            frettes: FRETTES.max(haute - basse + 1),
        }
    }
}

/// La zone de la palette : la zone du doigté (0 s'il fait sonner une corde à vide, sinon sa
/// frette appuyée la plus basse) à deux frettes près, jusqu'à deux frettes au-dessus de sa
/// frette la plus haute. Rend la première et la dernière frette ; 0 est la corde à vide.
fn zone_de_palette(d: &Doigte) -> (u8, u8) {
    (d.zone().saturating_sub(2), frette_haute(d) + 2)
}

/// La grille de la palette : depuis le sillet si la zone commence à la corde à vide ou à la
/// première frette.
fn fenetre_de_palette(d: &Doigte) -> Fenetre {
    let (debut, fin) = zone_de_palette(d);
    let premiere = debut.max(1);
    Fenetre {
        premiere,
        frettes: fin - premiere + 1,
    }
}

/// Six cordes et `frettes` cases de `case` mm, dès `haut` ; le sillet en tête, ou un trait.
fn grille(haut: f32, f: Fenetre, case: f32) -> Vec<Element> {
    let bas = haut + f32::from(f.frettes) * case;
    let ligne = |de, a, e| Element::Ligne {
        de,
        a,
        trait_: (Couleur::Noir, e),
    };
    let tete = if f.numero().is_none() {
        SILLET
    } else {
        TRAIT_MOYEN
    };
    let mut e = vec![ligne((0.0, haut), (LARGEUR_MANCHE, haut), tete)];
    for k in 1..=f.frettes {
        let y = haut + f32::from(k) * case;
        e.push(ligne((0.0, y), (LARGEUR_MANCHE, y), TRAIT_MOYEN));
    }
    for c in 0..6 {
        let x = x_corde(c);
        e.push(ligne((x, haut), (x, bas), TRAIT_MOYEN));
    }
    e
}

/// Le numéro de la première frette, à gauche de la corde grave, centré sur sa case.
fn numero_de_frette(numero: u8, milieu: f32) -> Element {
    let t = Corps::MIN;
    let texte = numero.to_string();
    let x = -(POINT + AVANT_LE_NUMERO) - chasse(&texte, &t);
    composer(&texte, &t, x, milieu + mi_capitale(&t))
}

/// Ce qui déborde à gauche de la corde grave : le numéro de frette, ou un point.
fn debord(f: Fenetre) -> f32 {
    f.numero().map_or(POINT, |k| {
        POINT + AVANT_LE_NUMERO + chasse(&k.to_string(), &Corps::MIN)
    })
}

/// Le milieu de la bande des cordes à vide, au-dessus du sillet.
fn y_a_vide() -> f32 {
    (RANG_A_VIDE - SILLET / 2.0) / 2.0
}

/// Ce qu'une corde montre au diagramme : un point plein pour un doigt, un cercle pour une
/// corde à vide, une croix pour une corde étouffée.
pub fn marque_de_corde(corde: &Corde, c: (f32, f32)) -> Element {
    let cercle = |id, r: f32, plein: Couleur, trait_: Option<(Couleur, f32)>| Element::Cercle {
        cx: c.0,
        cy: c.1,
        r,
        id,
        plein: Some(plein),
        trait_,
    };
    match corde {
        Corde::Frette(_) => cercle(Some("point"), POINT, Couleur::Noir, None),
        Corde::AVide => cercle(
            None,
            A_VIDE,
            Couleur::Blanc,
            Some((Couleur::Noir, TRAIT_CREUX)),
        ),
        Corde::Etouffee => croix(c, ETOUFFEE, TRAIT_CREUX, Couleur::Noir),
    }
}

/// Le barré : un groupe `barre`, un trait épais aux bouts arrondis, de l'abscisse `de` à `a`, à
/// la hauteur `y`.
pub fn trait_de_barre(de: f32, a: f32, y: f32) -> Element {
    let r = BARRE;
    let arc = format!("A{} {} 0 0 1", n(r), n(r));
    Element::Groupe {
        id: Some("barre"),
        translation: (0.0, 0.0),
        echelle: None,
        enfants: vec![Element::Chemin {
            d: format!(
                "M{} {}H{}{arc} {} {}H{}{arc} {} {}Z",
                n(de),
                n(y - r),
                n(a),
                n(a),
                n(y + r),
                n(de),
                n(de),
                n(y - r)
            ),
            plein: Some(Couleur::Noir),
            trait_: None,
        }],
    }
}

/// Des lignes centrées sur le milieu du manche, dès son haut.
fn lignes_centrees(lignes: &[String]) -> Vec<Element> {
    let t = Corps::MIN;
    let milieu = LARGEUR_MANCHE / 2.0;
    lignes
        .iter()
        .enumerate()
        .map(|(k, l)| {
            let base = k as f32 * interligne(&t) + jusqua_la_base(&t);
            composer(l, &t, milieu - chasse(l, &t) / 2.0, base)
        })
        .collect()
}

/// « pas de doigté jouable », puis les notes en solfège, coupés dans `place` mm ; un mot plus
/// large est une erreur.
fn lignes_sans_doigte(notes: &[NomDeNote], place: f32) -> Result<Vec<String>, MotTropLong> {
    let t = Corps::MIN;
    let noms: Vec<String> = notes.iter().map(|n| n.solfege()).collect();
    let mut lignes = couper(PAS_DE_DOIGTE, &t, place)?;
    lignes.extend(couper(&noms.join(" "), &t, place)?);
    Ok(lignes)
}

/// Le diagramme de manche (REN-04) : six cordes, quatre frettes, plus si un doigté parti du
/// sillet monte plus haut ; le numéro de la frette de départ s'il dépasse 1. Un point plein
/// marque un doigt, un cercle une corde à vide, une croix une corde étouffée, un trait épais le
/// barré ; les numéros de doigts sont dessous. Sans doigté : « pas de doigté jouable » et les
/// notes, sans grille, coupés dans la place d'une colonne de la table, où leurs mots tiennent
/// tous (le plus long, « jouable », fait 14,2 mm pour 25,2).
pub fn diagramme_de_manche(
    d: Option<&Doigte>,
    notes: &[NomDeNote],
    origine: (f32, f32),
) -> Element {
    diagramme(d, notes, origine, PLACE_D_UNE_COLONNE)
        .expect("chaque mot de ces textes tient dans une colonne")
}

/// Le diagramme de manche ; sans doigté, ses textes coupés dans `place` mm, centrés sur le
/// manche.
pub(crate) fn diagramme(
    d: Option<&Doigte>,
    notes: &[NomDeNote],
    origine: (f32, f32),
    place: f32,
) -> Result<Element, MotTropLong> {
    let Some(d) = d else {
        return Ok(Element::Groupe {
            id: None,
            translation: origine,
            echelle: None,
            enfants: lignes_centrees(&lignes_sans_doigte(notes, place)?),
        });
    };
    let t = Corps::MIN;
    let f = fenetre_du_diagramme(d);
    let haut = RANG_A_VIDE;
    let mut e = grille(haut, f, CASE_DOIGTE);
    if let Some(k) = f.numero() {
        e.push(numero_de_frette(k, haut + CASE_DOIGTE / 2.0));
    }
    let base_des_doigts =
        haut + f32::from(f.frettes) * CASE_DOIGTE + SOUS_LA_GRILLE + jusqua_la_base(&t);
    let mut doigts = Vec::new();
    for (c, corde) in d.cordes.iter().enumerate() {
        let x = x_corde(c);
        match corde {
            Corde::Frette(fr) if d.barre.as_ref().is_some_and(|b| b.frette == *fr) => {}
            Corde::Frette(fr) => e.push(marque_de_corde(corde, (x, haut + f.y(*fr, CASE_DOIGTE)))),
            Corde::AVide | Corde::Etouffee => e.push(marque_de_corde(corde, (x, y_a_vide()))),
        }
        if let Some(doigt) = d.doigts[c] {
            let texte = doigt.to_string();
            doigts.push(composer(
                &texte,
                &t,
                x - chasse(&texte, &t) / 2.0,
                base_des_doigts,
            ));
        }
    }
    if let Some(b) = &d.barre {
        e.push(trait_de_barre(
            x_corde(b.de),
            x_corde(b.a),
            haut + f.y(b.frette, CASE_DOIGTE),
        ));
    }
    e.push(Element::Groupe {
        id: Some("doigts"),
        translation: (0.0, 0.0),
        echelle: None,
        enfants: doigts,
    });
    Ok(Element::Groupe {
        id: Some("manche"),
        translation: origine,
        echelle: None,
        enfants: e,
    })
}

/// La hauteur du diagramme : la bande des cordes à vide, la grille et les numéros de doigts ;
/// sans doigté, ses lignes de texte, coupées dans `place` mm.
pub(crate) fn hauteur_du_diagramme(
    d: Option<&Doigte>,
    notes: &[NomDeNote],
    place: f32,
) -> Result<f32, MotTropLong> {
    let il = interligne(&Corps::MIN);
    Ok(match d {
        Some(d) => {
            let f = fenetre_du_diagramme(d);
            RANG_A_VIDE + f32::from(f.frettes) * CASE_DOIGTE + SOUS_LA_GRILLE + il
        }
        None => lignes_sans_doigte(notes, place)?.len() as f32 * il,
    })
}

/// Ce que le diagramme déborde à gauche de la corde grave ; sans doigté, rien de plus que ses
/// lignes, centrées sur le manche.
pub(crate) fn debord_du_diagramme(d: Option<&Doigte>) -> f32 {
    d.map_or(POINT, |d| debord(fenetre_du_diagramme(d)))
}

/// La palette sur le manche (MIC-13), dans la zone du doigté à ± 2 frettes : chaque note de la
/// palette, sur chaque corde où elle sonne, par la forme de son rôle. La bande des cordes à vide
/// n'est là que si la zone part de la corde à vide.
pub fn palette_sur_le_manche(d: &Doigte, p: &[NoteDePalette], origine: (f32, f32)) -> Element {
    let (debut, fin) = zone_de_palette(d);
    let f = fenetre_de_palette(d);
    let haut = if debut == 0 { RANG_A_VIDE } else { 0.0 };
    let mut e = grille(haut, f, CASE_PALETTE);
    if let Some(k) = f.numero() {
        e.push(numero_de_frette(k, haut + CASE_PALETTE / 2.0));
    }
    for (c, a_vide) in ACCORDAGE.iter().enumerate() {
        for fr in debut..=fin {
            let classe = (a_vide + fr) % 12;
            let Some(note) = p.iter().find(|n| n.nom.classe() == classe) else {
                continue;
            };
            let y = if fr == 0 {
                y_a_vide()
            } else {
                haut + f.y(fr, CASE_PALETTE)
            };
            e.push(marque(note.role, (x_corde(c), y), Support::Manche));
        }
    }
    Element::Groupe {
        id: Some("manche"),
        translation: origine,
        echelle: None,
        enfants: e,
    }
}

/// La hauteur de la palette sur le manche : la bande des cordes à vide si la zone y part, et
/// ses cases.
pub(crate) fn hauteur_de_la_palette(d: &Doigte) -> f32 {
    let a_vide = if zone_de_palette(d).0 == 0 {
        RANG_A_VIDE
    } else {
        0.0
    };
    a_vide + f32::from(fenetre_de_palette(d).frettes) * CASE_PALETTE
}

/// Ce que la palette déborde à gauche de la corde grave.
pub(crate) fn debord_de_la_palette(d: &Doigte) -> f32 {
    debord(fenetre_de_palette(d))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hauteur::Lettre;

    #[test]
    fn sans_doigte_les_textes_se_coupent_dans_la_place_donnee() {
        let notes = [Lettre::A, Lettre::C, Lettre::E].map(|lettre| NomDeNote {
            lettre,
            alteration: 0,
        });
        let il = interligne(&Corps::MIN);
        // Dans une colonne de la table : « pas de », « doigté », « jouable », puis les notes.
        assert_eq!(
            hauteur_du_diagramme(None, &notes, PLACE_D_UNE_COLONNE),
            Ok(4.0 * il)
        );
        // Dans une place plus large, « pas de doigté jouable » tient sur une ligne.
        assert_eq!(hauteur_du_diagramme(None, &notes, 60.0), Ok(2.0 * il));
        // Plus étroite que « doigté » : une erreur, jamais un mot coupé.
        let e = diagramme(None, &notes, (0.0, 0.0), 10.0).unwrap_err();
        assert_eq!(e.mot, "doigté");
    }

    fn d(notation: &str) -> Doigte {
        Doigte::depuis(notation).expect(notation)
    }

    #[test]
    fn la_fenetre_part_du_sillet_tant_qu_elle_le_peut() {
        // Am, B°, F barré : du sillet, quatre frettes.
        for notation in ["x02210", "x2343x", "133211"] {
            assert_eq!(
                fenetre_du_diagramme(&d(notation)),
                Fenetre {
                    premiere: 1,
                    frettes: 4
                },
                "{notation}"
            );
        }
        // Plus haut, sans corde à vide : dès la frette appuyée la plus basse, numérotée.
        let haut = fenetre_du_diagramme(&d("xx4575"));
        assert_eq!(
            haut,
            Fenetre {
                premiere: 4,
                frettes: 4
            }
        );
        assert_eq!(haut.numero(), Some(4));
    }

    #[test]
    fn la_zone_de_la_palette_est_celle_du_doigte_a_deux_frettes_pres() {
        // Une corde à vide : la zone part de la corde à vide, jusqu'à deux frettes au-dessus
        // de la plus haute.
        assert_eq!(zone_de_palette(&d("x02210")), (0, 4));
        // B° (x2343x) : de la frette 0 (2 − 2) à la sixième.
        assert_eq!(zone_de_palette(&d("x2343x")), (0, 6));
        // Plus haut : de deux frettes sous la plus basse.
        assert_eq!(zone_de_palette(&d("x46654")), (2, 8));
        assert_eq!(fenetre_de_palette(&d("x46654")).numero(), Some(2));
        // Dès la première frette : le sillet, sans corde à vide.
        assert_eq!(zone_de_palette(&d("x35543")), (1, 7));
        assert_eq!(fenetre_de_palette(&d("x35543")).numero(), None);
        assert_eq!(hauteur_de_la_palette(&d("x35543")), 7.0 * CASE_PALETTE);
    }
}
