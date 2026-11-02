//! Les glyphes en tracés : chasse, composition, coupe des lignes.
//!
//! Ni crénage, ni ligature, ni césure. La police entre par `include_bytes!`.

use std::fmt::Write;
use std::sync::OnceLock;

use ttf_parser::{Face, GlyphId, OutlineBuilder};

use super::svg::{Couleur, Element, nombre};

static POLICE: &[u8] = include_bytes!("../../polices/DejaVuSans.ttf");

/// Unités par cadratin de DejaVu Sans.
const UNITES: f32 = 2048.0;
const MM_PAR_PT: f32 = 25.4 / 72.0;

fn face() -> &'static Face<'static> {
    static FACE: OnceLock<Face<'static>> = OnceLock::new();
    FACE.get_or_init(|| Face::parse(POLICE, 0).expect("la police embarquée est valide"))
}

/// Un corps de texte, d'au moins 11 pt (§ 50 · 6.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corps(f32);

impl Corps {
    pub const MIN: Corps = Corps(11.0);

    pub fn pt(pt: f32) -> Option<Corps> {
        (pt >= Self::MIN.0).then_some(Corps(pt))
    }

    pub fn mm(&self) -> f32 {
        self.0 * MM_PAR_PT
    }
}

/// Un mot plus large que sa place : une erreur, jamais un mot coupé.
#[derive(Debug, Clone, PartialEq)]
pub struct MotTropLong {
    pub mot: String,
    pub chasse: f32,
    pub largeur: f32,
}

fn glyphe(c: char) -> GlyphId {
    face().glyph_index(c).unwrap_or(GlyphId(0))
}

pub fn glyphe_present(c: char) -> bool {
    face().glyph_index(c).is_some()
}

fn avance(c: char) -> f32 {
    f32::from(face().glyph_hor_advance(glyphe(c)).unwrap_or(0))
}

/// La chasse du texte, en mm : la somme des avances, sans crénage.
pub fn chasse(texte: &str, corps: &Corps) -> f32 {
    texte.chars().map(avance).sum::<f32>() * corps.mm() / UNITES
}

struct Trace {
    d: String,
    decalage: f32,
}

impl Trace {
    fn point(&mut self, x: f32, y: f32) {
        let _ = write!(self.d, "{} {}", nombre(x + self.decalage, 3), nombre(y, 3));
    }
}

impl OutlineBuilder for Trace {
    fn move_to(&mut self, x: f32, y: f32) {
        self.d.push('M');
        self.point(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.d.push('L');
        self.point(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.d.push('Q');
        self.point(x1, y1);
        self.d.push(' ');
        self.point(x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.d.push('C');
        self.point(x1, y1);
        self.d.push(' ');
        self.point(x2, y2);
        self.d.push(' ');
        self.point(x, y);
    }
    fn close(&mut self) {
        self.d.push('Z');
    }
}

/// Un texte : un groupe `t`, dont le tracé est en unités de police et que
/// `scale(k -k)` ramène en mm (`k` = corps en mm / 2048).
///
/// Un caractère absent de la police se dessinerait par le glyphe `.notdef`, un rectangle, sans
/// rien dire : en debug, et donc dans les tests des pages, `composer` le nomme et s'arrête.
pub fn composer(texte: &str, corps: &Corps, x: f32, ligne_de_base: f32) -> Element {
    let mut trace = Trace {
        d: String::new(),
        decalage: 0.0,
    };
    for c in texte.chars() {
        debug_assert!(
            glyphe_present(c),
            "« {c} » (U+{:04X}) n'a pas de glyphe dans la police, dans « {texte} »",
            u32::from(c)
        );
        // Un glyphe sans contour (l'espace) n'écrit rien, mais avance.
        face().outline_glyph(glyphe(c), &mut trace);
        trace.decalage += avance(c);
    }
    let k = corps.mm() / UNITES;
    Element::Groupe {
        id: Some("t"),
        translation: (x, ligne_de_base),
        echelle: Some((k, -k)),
        enfants: vec![Element::Chemin {
            d: trace.d,
            plein: Some(Couleur::Noir),
            trait_: None,
        }],
    }
}

/// Coupe le texte entre deux mots, chaque ligne remplie tant que sa chasse
/// tient dans `largeur`.
pub fn couper(texte: &str, corps: &Corps, largeur: f32) -> Result<Vec<String>, MotTropLong> {
    let mut lignes: Vec<String> = Vec::new();
    for mot in texte.split_whitespace() {
        if let Some(ligne) = lignes.last_mut() {
            let essai = format!("{ligne} {mot}");
            if chasse(&essai, corps) <= largeur {
                *ligne = essai;
                continue;
            }
        }
        let c = chasse(mot, corps);
        if c > largeur {
            return Err(MotTropLong {
                mot: mot.to_string(),
                chasse: c,
                largeur,
            });
        }
        lignes.push(mot.to_string());
    }
    Ok(lignes)
}

/// Coupe un texte de la forme « a · b · c » de préférence aux « · », et chaque segment trop
/// long entre deux mots : l'en-tête et le pied d'une page.
pub(crate) fn couper_aux_points(
    texte: &str,
    corps: &Corps,
    largeur: f32,
) -> Result<Vec<String>, MotTropLong> {
    let mut lignes: Vec<String> = Vec::new();
    for segment in texte.split(" · ") {
        if let Some(ligne) = lignes.last_mut() {
            let essai = format!("{ligne} · {segment}");
            if chasse(&essai, corps) <= largeur {
                *ligne = essai;
                continue;
            }
        }
        lignes.extend(couper(segment, corps, largeur)?);
    }
    Ok(lignes)
}
