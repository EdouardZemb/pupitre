//! Les aides des tests de rendu : la mesure d'une page, et la différence de deux rendus
//! couleur supprimée (I17).

// Chaque fichier de test n'emploie qu'une partie des aides.
#![allow(dead_code)]

use resvg::{tiny_skia, usvg};

const PT_PAR_MM: f32 = 72.0 / 25.4;
/// Les pixels d'usvg par mm : 96 ppp. La largeur d'une page n'y entre pas, une fiche étant
/// plus étroite qu'une table.
const PX_PAR_MM: f32 = 96.0 / 25.4;
/// Unités par cadratin de DejaVu Sans.
const UNITES: f32 = 2048.0;
/// La résolution du test de différence (§ 50 · 6.5).
const PPP: f32 = 300.0;

/// Ce que les tests mesurent d'une page, en mm et en points.
pub struct Mesures {
    pub corps_min_pt: f32,
    pub trait_min_pt: f32,
    pub manches: Vec<f32>,
    pub touches_blanches: Vec<f32>,
    pub pastilles: Vec<f32>,
}

/// Lit la page par usvg et la mesure par ses identifiants : l'échelle des groupes
/// `t-*` donne le corps, le plus petit `stroke-width` le trait, et les boîtes des
/// `manche-*`, `touche-blanche-*` et `pastille-*` leurs largeurs. Les longueurs sont
/// arrondies au micromètre, la précision des nombres écrits dans le SVG.
///
/// Une page sans texte ou sans trait ne se mesure pas : le minimum partirait de
/// l'infini et passerait tout seuil.
pub fn mesurer(svg: &str) -> Mesures {
    let arbre = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("SVG lisible");
    let px_par_mm = PX_PAR_MM;
    let mut m = Mesures {
        corps_min_pt: f32::INFINITY,
        trait_min_pt: f32::INFINITY,
        manches: Vec::new(),
        touches_blanches: Vec::new(),
        pastilles: Vec::new(),
    };
    parcourir(arbre.root(), px_par_mm, &mut m);
    assert!(
        m.corps_min_pt.is_finite(),
        "la page ne compose aucun texte : son corps ne se mesure pas"
    );
    assert!(
        m.trait_min_pt.is_finite(),
        "la page ne trace aucun trait : son épaisseur ne se mesure pas"
    );
    m
}

fn parcourir(groupe: &usvg::Group, px_par_mm: f32, m: &mut Mesures) {
    let mm = |px: f32| (px / px_par_mm * 1000.0).round() / 1000.0;
    for noeud in groupe.children() {
        match noeud {
            usvg::Node::Group(g) => {
                if g.id().starts_with("t-") {
                    let corps_mm = echelle(g.abs_transform()) / px_par_mm * UNITES;
                    m.corps_min_pt = m.corps_min_pt.min(corps_mm * PT_PAR_MM);
                }
                if g.id().starts_with("manche-") {
                    m.manches.push(mm(g.abs_bounding_box().width()));
                }
                parcourir(g, px_par_mm, m);
            }
            usvg::Node::Path(p) => {
                if let Some(s) = p.stroke() {
                    let e = s.width().get() * echelle(p.abs_transform()) / px_par_mm;
                    m.trait_min_pt = m.trait_min_pt.min(e * PT_PAR_MM);
                }
                let largeur = mm(p.abs_bounding_box().width());
                if p.id().starts_with("touche-blanche-") {
                    m.touches_blanches.push(largeur);
                }
                if p.id().starts_with("pastille-") {
                    m.pastilles.push(largeur);
                }
            }
            _ => {}
        }
    }
}

/// Une boîte englobante, en mm depuis le coin haut gauche de la page.
#[derive(Debug, Clone, Copy)]
pub struct Boite {
    pub gauche: f32,
    pub haut: f32,
    pub droite: f32,
    pub bas: f32,
}

/// La boîte de ce que la page dessine, lue par usvg : les contours des glyphes et les traits,
/// épaisseur comprise. Le fond blanc de toute la page, son premier `rect`, n'en est pas.
pub fn boite_du_dessin(svg: &str) -> Boite {
    let debut = svg.find("<rect").expect("le fond de la page");
    let fin = debut + svg[debut..].find("/>").expect("un rect fermé") + 2;
    let fond = &svg[debut..fin];
    assert!(
        fond.contains(r#"x="0" y="0""#) && fond.contains(r##"fill="#fff""##),
        "le premier rect n'est pas le fond : {fond}"
    );
    let sans_fond = format!("{}{}", &svg[..debut], &svg[fin..]);
    let arbre = usvg::Tree::from_str(&sans_fond, &usvg::Options::default()).expect("SVG lisible");
    let px_par_mm = PX_PAR_MM;
    let b = arbre.root().abs_stroke_bounding_box();
    assert!(
        b.width() > 0.0 && b.height() > 0.0,
        "la page ne dessine rien"
    );
    Boite {
        gauche: b.left() / px_par_mm,
        haut: b.top() / px_par_mm,
        droite: b.right() / px_par_mm,
        bas: b.bottom() / px_par_mm,
    }
}

/// Le facteur d'échelle d'une transformation sans cisaillement.
fn echelle(t: usvg::Transform) -> f32 {
    (t.sx * t.sy - t.kx * t.ky).abs().sqrt()
}

/// I17, en pixels : rastérise les deux SVG par resvg à 300 ppp, force toute couleur non
/// blanche au noir, et rend la part des pixels qui diffèrent dans l'union des boîtes
/// englobantes de leurs pixels noirs.
pub fn difference(svg_a: &str, svg_b: &str) -> f32 {
    let a = noircir(svg_a);
    let b = noircir(svg_b);
    assert_eq!(
        (a.largeur, a.hauteur),
        (b.largeur, b.hauteur),
        "deux pages de même taille"
    );
    let boite = match (a.boite(), b.boite()) {
        (Some(p), Some(q)) => (p.0.min(q.0), p.1.min(q.1), p.2.max(q.2), p.3.max(q.3)),
        (Some(p), None) | (None, Some(p)) => p,
        (None, None) => panic!("aucun pixel noir dans les deux rendus"),
    };
    let (x0, y0, x1, y1) = boite;
    let mut differents = 0_usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = y * a.largeur + x;
            if a.noirs[i] != b.noirs[i] {
                differents += 1;
            }
        }
    }
    differents as f32 / ((x1 - x0 + 1) * (y1 - y0 + 1)) as f32
}

/// Une image où chaque pixel est noir ou blanc.
struct Noircie {
    noirs: Vec<bool>,
    largeur: usize,
    hauteur: usize,
}

impl Noircie {
    /// La boîte englobante des pixels noirs : (x0, y0, x1, y1), bornes comprises.
    fn boite(&self) -> Option<(usize, usize, usize, usize)> {
        let mut boite: Option<(usize, usize, usize, usize)> = None;
        for (i, _) in self.noirs.iter().enumerate().filter(|(_, n)| **n) {
            let (x, y) = (i % self.largeur, i / self.largeur);
            boite = Some(match boite {
                None => (x, y, x, y),
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            });
        }
        boite
    }
}

fn noircir(svg: &str) -> Noircie {
    let options = usvg::Options::default();
    let arbre = usvg::Tree::from_str(svg, &options).expect("SVG lisible");
    let k = PPP / options.dpi;
    let taille = arbre
        .size()
        .to_int_size()
        .scale_by(k)
        .expect("une page de taille non nulle");
    let mut image =
        tiny_skia::Pixmap::new(taille.width(), taille.height()).expect("une image allouable");
    image.fill(tiny_skia::Color::WHITE);
    resvg::render(
        &arbre,
        tiny_skia::Transform::from_scale(k, k),
        &mut image.as_mut(),
    );
    let noirs = image
        .pixels()
        .iter()
        .map(|p| p.red() < 255 || p.green() < 255 || p.blue() < 255)
        .collect();
    Noircie {
        noirs,
        largeur: taille.width() as usize,
        hauteur: taille.height() as usize,
    }
}
