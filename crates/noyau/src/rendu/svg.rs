//! Les éléments d'une page et leur écriture en SVG.

use std::collections::BTreeMap;
use std::fmt::Write;

use super::pages::Rect;

/// Les pages n'ont que du noir et du blanc (mode encre).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Couleur {
    Noir,
    Blanc,
}

impl Couleur {
    fn svg(self) -> &'static str {
        match self {
            Couleur::Noir => "#000",
            Couleur::Blanc => "#fff",
        }
    }

    /// Le noir pour le blanc, le blanc pour le noir.
    pub(crate) fn inverse(self) -> Couleur {
        match self {
            Couleur::Noir => Couleur::Blanc,
            Couleur::Blanc => Couleur::Noir,
        }
    }
}

/// Un trait : couleur et épaisseur en mm.
pub type Trait = (Couleur, f32);

/// Un élément du sous-ensemble de SVG.
///
/// Un `id` est un préfixe statique : [`Page::svg`] y ajoute `-n`, un compteur
/// par préfixe qui part de 1, dans l'ordre d'écriture.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    Groupe {
        id: Option<&'static str>,
        translation: (f32, f32),
        echelle: Option<(f32, f32)>,
        enfants: Vec<Element>,
    },
    Chemin {
        d: String,
        plein: Option<Couleur>,
        trait_: Option<Trait>,
    },
    Rect {
        r: Rect,
        id: Option<&'static str>,
        plein: Option<Couleur>,
        trait_: Option<Trait>,
    },
    Cercle {
        cx: f32,
        cy: f32,
        r: f32,
        id: Option<&'static str>,
        plein: Option<Couleur>,
        trait_: Option<Trait>,
    },
    Ligne {
        de: (f32, f32),
        a: (f32, f32),
        trait_: Trait,
    },
}

/// 0,25 pt en mm : le trait fin.
pub const TRAIT_FIN: f32 = 0.25 * 25.4 / 72.0;
/// 0,5 pt, deux fois le trait fin : le trait des cadres, des flèches et des diagrammes.
pub(crate) const TRAIT_MOYEN: f32 = 2.0 * TRAIT_FIN;

/// Un nombre à `decimales` décimales au plus, sans zéro final.
pub(crate) fn nombre(v: f32, decimales: usize) -> String {
    let mut s = format!("{v:.decimales$}");
    if s.contains('.') {
        s.truncate(s.trim_end_matches('0').trim_end_matches('.').len());
    }
    if s == "-0" { "0".to_string() } else { s }
}

fn n(v: f32) -> String {
    nombre(v, 3)
}

/// Une épaisseur de trait, arrondie par excès au dix-millième de mm : un trait écrit ne
/// passe jamais sous celui qu'on a voulu. Le trait fin de 0,25 pt s'écrit `0.0882`, et non
/// `0.088`, qui ne ferait que 0,2494 pt.
fn epaisseur(v: f32) -> String {
    nombre(((f64::from(v) * 1e4).ceil() / 1e4) as f32, 4)
}

impl Element {
    /// Le même élément, le noir et le blanc échangés partout : le contenu d'une case en
    /// inverse.
    pub(crate) fn inverse(self) -> Element {
        let trait_inverse = |(c, e): Trait| (c.inverse(), e);
        match self {
            Element::Groupe {
                id,
                translation,
                echelle,
                enfants,
            } => Element::Groupe {
                id,
                translation,
                echelle,
                enfants: enfants.into_iter().map(Element::inverse).collect(),
            },
            Element::Chemin { d, plein, trait_ } => Element::Chemin {
                d,
                plein: plein.map(Couleur::inverse),
                trait_: trait_.map(trait_inverse),
            },
            Element::Rect {
                r,
                id,
                plein,
                trait_,
            } => Element::Rect {
                r,
                id,
                plein: plein.map(Couleur::inverse),
                trait_: trait_.map(trait_inverse),
            },
            Element::Cercle {
                cx,
                cy,
                r,
                id,
                plein,
                trait_,
            } => Element::Cercle {
                cx,
                cy,
                r,
                id,
                plein: plein.map(Couleur::inverse),
                trait_: trait_.map(trait_inverse),
            },
            Element::Ligne { de, a, trait_ } => Element::Ligne {
                de,
                a,
                trait_: trait_inverse(trait_),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub largeur: f32,
    pub hauteur: f32,
    pub elements: Vec<Element>,
}

struct Plume {
    sortie: String,
    compteurs: BTreeMap<&'static str, u32>,
}

impl Plume {
    fn id(&mut self, prefixe: Option<&'static str>) -> String {
        let Some(prefixe) = prefixe else {
            return String::new();
        };
        let c = self.compteurs.entry(prefixe).or_insert(0);
        *c += 1;
        format!(r#" id="{prefixe}-{c}""#)
    }

    fn peinture(&mut self, plein: Option<Couleur>, trait_: Option<Trait>) {
        let fill = plein.map_or("none", Couleur::svg);
        let _ = write!(self.sortie, r#" fill="{fill}""#);
        if let Some((c, e)) = trait_ {
            let _ = write!(
                self.sortie,
                r#" stroke="{}" stroke-width="{}""#,
                c.svg(),
                epaisseur(e)
            );
        }
    }

    fn ecrire(&mut self, e: &Element) {
        match e {
            Element::Groupe {
                id,
                translation,
                echelle,
                enfants,
            } => {
                let id = self.id(*id);
                let _ = write!(
                    self.sortie,
                    r#"<g{id} transform="translate({} {})"#,
                    n(translation.0),
                    n(translation.1)
                );
                if let Some((a, b)) = echelle {
                    let _ = write!(self.sortie, " scale({} {})", nombre(*a, 6), nombre(*b, 6));
                }
                self.sortie.push_str(r#"">"#);
                for enfant in enfants {
                    self.ecrire(enfant);
                }
                self.sortie.push_str("</g>");
            }
            Element::Chemin { d, plein, trait_ } => {
                let _ = write!(self.sortie, r#"<path d="{d}""#);
                self.peinture(*plein, *trait_);
                self.sortie.push_str("/>");
            }
            Element::Rect {
                r,
                id,
                plein,
                trait_,
            } => {
                let id = self.id(*id);
                let _ = write!(
                    self.sortie,
                    r#"<rect{id} x="{}" y="{}" width="{}" height="{}""#,
                    n(r.x),
                    n(r.y),
                    n(r.l),
                    n(r.h)
                );
                self.peinture(*plein, *trait_);
                self.sortie.push_str("/>");
            }
            Element::Cercle {
                cx,
                cy,
                r,
                id,
                plein,
                trait_,
            } => {
                let id = self.id(*id);
                let _ = write!(
                    self.sortie,
                    r#"<circle{id} cx="{}" cy="{}" r="{}""#,
                    n(*cx),
                    n(*cy),
                    n(*r)
                );
                self.peinture(*plein, *trait_);
                self.sortie.push_str("/>");
            }
            Element::Ligne { de, a, trait_ } => {
                let _ = write!(
                    self.sortie,
                    r#"<line x1="{}" y1="{}" x2="{}" y2="{}""#,
                    n(de.0),
                    n(de.1),
                    n(a.0),
                    n(a.1)
                );
                self.peinture(None, Some(*trait_));
                self.sortie.push_str("/>");
            }
        }
    }
}

impl Page {
    /// Le SVG de la page : un `rect` blanc de toute la page, puis les éléments.
    /// Deux appels donnent le même texte, à l'octet.
    pub fn svg(&self) -> String {
        let (l, h) = (n(self.largeur), n(self.hauteur));
        let mut plume = Plume {
            sortie: String::new(),
            compteurs: BTreeMap::new(),
        };
        let _ = write!(
            plume.sortie,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{l}mm" height="{h}mm" viewBox="0 0 {l} {h}"><rect x="0" y="0" width="{l}" height="{h}" fill="#fff"/>"##
        );
        for e in &self.elements {
            plume.ecrire(e);
        }
        plume.sortie.push_str("</svg>");
        plume.sortie
    }
}
