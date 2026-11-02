//! H09, partie automatique : la fiche témoin tient à 11 pt en paysage, reste dans
//! le sous-ensemble de SVG, respecte les minimums physiques, et son PDF pose chaque
//! page à sa taille, sans mise à l'échelle.

use cahier::assembler;
use cahier::temoin::pages_temoin;
use lopdf::content::Content;
use lopdf::{Dictionary, Document, Object, ObjectId};
use noyau::rendu::pages::{Date, LARGEUR};
use noyau::rendu::sous_ensemble::verifier;
use svg2pdf::usvg;

const DATE: Date = Date {
    annee: 2026,
    mois: 11,
    jour: 1,
};
const L_PT: f32 = 209.5 * 72.0 / 25.4; // 593,858
const H_PT: f32 = 157.0 * 72.0 / 25.4; // 445,039
const PT_PAR_MM: f32 = 72.0 / 25.4;
/// Unités par cadratin de DejaVu Sans.
const UNITES: f32 = 2048.0;

#[test]
fn la_fiche_temoin_tient_a_11_pt_en_paysage() {
    let pages = pages_temoin(&DATE);
    assert!(pages.is_ok(), "{:?}", pages.err());
}

#[test]
fn les_pages_temoin_restent_dans_le_sous_ensemble() {
    for p in pages_temoin(&DATE).unwrap() {
        assert_eq!(verifier(&p.svg), Ok(()));
    }
}

#[test]
fn la_fiche_temoin_respecte_les_minimums() {
    for p in pages_temoin(&DATE).unwrap() {
        let m = mesurer(&p.svg);
        assert!(m.corps_min_pt >= 11.0, "corps {}", m.corps_min_pt);
        assert!(m.trait_min_pt >= 0.25 - 1e-3, "trait {}", m.trait_min_pt);
        assert!(m.manches.iter().all(|l| *l >= 18.0), "{:?}", m.manches);
        assert!(
            m.touches_blanches.iter().all(|l| *l >= 3.0),
            "{:?}",
            m.touches_blanches
        );
        assert!(m.pastilles.iter().all(|d| *d >= 3.0), "{:?}", m.pastilles);
    }
}

#[test]
fn la_fiche_temoin_dessine_ce_qu_elle_doit_mesurer() {
    // Sans ce garde-fou, une page sans manche ni touche passerait les minimums.
    let pages = pages_temoin(&DATE).unwrap();
    let table = mesurer(&pages[1].svg);
    let verso = mesurer(&pages[2].svg);
    assert_eq!(table.pastilles.len(), 7, "une pastille par case du centre");
    assert_eq!(
        verso.manches.len(),
        2 * 7,
        "par colonne, le diagramme de manche et la palette sur le manche"
    );
    assert_eq!(
        verso.touches_blanches.len(),
        7 * (7 + 9),
        "par colonne, le diagramme de clavier (7 touches) et la palette (9)"
    );
}

#[test]
fn le_pdf_temoin_a_trois_pages_de_la_taille_des_svg() {
    let doc =
        Document::load_mem(&assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap()).unwrap();
    assert_eq!(doc.get_pages().len(), 3);
    for (_, id) in doc.get_pages() {
        assert_media_box(&doc, id, L_PT, H_PT, 0.01);
    }
}

#[test]
fn le_pdf_temoin_pose_les_pages_sans_mise_a_l_echelle() {
    let doc =
        Document::load_mem(&assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap()).unwrap();
    for (n, id) in doc.get_pages() {
        let page = doc.get_dictionary(id).unwrap();
        let ressources = dictionnaire(&doc, page.get(b"Resources").unwrap());
        let xobjets = dictionnaire(&doc, ressources.get(b"XObject").unwrap());
        assert_eq!(xobjets.len(), 1, "page {n} : un seul XObject");
        let (nom, reference) = xobjets.iter().next().unwrap();
        let forme = doc
            .get_object(reference.as_reference().unwrap())
            .unwrap()
            .as_stream()
            .unwrap();
        let bbox = nombres(&doc, forme.dict.get(b"BBox").unwrap());
        let matrice = nombres(&doc, forme.dict.get(b"Matrix").unwrap());

        let flux = Content::decode(&doc.get_page_content(id)).unwrap();
        let mut cm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let mut pose = false;
        for op in &flux.operations {
            match op.operator.as_str() {
                "cm" => {
                    let m: Vec<f32> = op.operands.iter().map(nombre).collect();
                    cm = composer([m[0], m[1], m[2], m[3], m[4], m[5]], cm);
                }
                "Do" => {
                    assert_eq!(op.operands[0].as_name().unwrap(), nom.as_slice());
                    pose = true;
                }
                _ => {}
            }
        }
        assert!(pose, "page {n} : le XObject est posé");

        let m = composer(
            [
                matrice[0], matrice[1], matrice[2], matrice[3], matrice[4], matrice[5],
            ],
            cm,
        );
        let (x0, y0) = appliquer(m, bbox[0], bbox[1]);
        let (x1, y1) = appliquer(m, bbox[2], bbox[3]);
        let media = nombres(&doc, page.get(b"MediaBox").unwrap());
        for (vu, attendu) in [
            (x0, media[0]),
            (y0, media[1]),
            (x1, media[2]),
            (y1, media[3]),
        ] {
            assert!(
                (vu - attendu).abs() <= 0.01,
                "page {n} : {vu} au lieu de {attendu}"
            );
        }
    }
}

#[test]
fn le_lien_du_sommaire_mene_a_la_table() {
    let doc =
        Document::load_mem(&assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap()).unwrap();
    let pages = doc.get_pages();
    let (sommaire, table) = (pages[&1], pages[&2]);
    let annotations = match doc.get_dictionary(sommaire).unwrap().get(b"Annots") {
        Ok(o) => tableau(&doc, o),
        Err(_) => Vec::new(),
    };
    assert_eq!(annotations.len(), 1, "une seule annotation au sommaire");
    let lien = dictionnaire(&doc, &annotations[0]);
    assert_eq!(lien.get(b"Subtype").unwrap().as_name().unwrap(), b"Link");
    let dest = tableau(&doc, lien.get(b"Dest").unwrap());
    assert_eq!(
        dest[0].as_reference().unwrap(),
        table,
        "la destination est la page 2"
    );
    assert_eq!(dest[1].as_name().unwrap(), b"Fit");
    let zone = nombres(&doc, lien.get(b"Rect").unwrap());
    assert!(
        (zone[3] - zone[1]).abs() >= 12.0 * PT_PAR_MM - 1e-3,
        "hauteur {}",
        (zone[3] - zone[1]).abs()
    );
    for (_, id) in pages.iter().skip(1) {
        assert!(doc.get_dictionary(*id).unwrap().get(b"Annots").is_err());
    }
}

#[test]
fn deux_engendrements_du_temoin_sont_identiques() {
    let a = assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap();
    let b = assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap();
    assert_eq!(a, b);
}

#[test]
fn le_pdf_temoin_porte_ses_metadonnees_sans_identifiant() {
    let pdf = assembler(&pages_temoin(&DATE).unwrap(), &DATE).unwrap();
    let doc = Document::load_mem(&pdf).unwrap();
    let info = dictionnaire(&doc, doc.trailer.get(b"Info").unwrap());
    assert_eq!(info.get(b"Producer").unwrap().as_str().unwrap(), b"pupitre");
    assert_eq!(
        info.get(b"CreationDate").unwrap().as_str().unwrap(),
        b"D:20261101000000Z"
    );
    assert!(doc.trailer.get(b"ID").is_err(), "pas d'/ID");
}

// --- Les aides ---------------------------------------------------------------

/// Ce que les tests mesurent d'une page, en mm et en points.
struct Mesures {
    corps_min_pt: f32,
    trait_min_pt: f32,
    manches: Vec<f32>,
    touches_blanches: Vec<f32>,
    pastilles: Vec<f32>,
}

/// Lit la page par usvg et la mesure par ses identifiants : l'échelle des groupes
/// `t-*` donne le corps, le plus petit `stroke-width` le trait, et les boîtes des
/// `manche-*`, `touche-blanche-*` et `pastille-*` leurs largeurs. Les longueurs sont
/// arrondies au micromètre, la précision des nombres écrits dans le SVG.
fn mesurer(svg: &str) -> Mesures {
    let arbre = usvg::Tree::from_str(svg, &usvg::Options::default()).expect("SVG lisible");
    let px_par_mm = arbre.size().width() / LARGEUR;
    let mut m = Mesures {
        corps_min_pt: f32::INFINITY,
        trait_min_pt: f32::INFINITY,
        manches: Vec::new(),
        touches_blanches: Vec::new(),
        pastilles: Vec::new(),
    };
    parcourir(arbre.root(), px_par_mm, &mut m);
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

/// Le facteur d'échelle d'une transformation sans cisaillement.
fn echelle(t: usvg::Transform) -> f32 {
    (t.sx * t.sy - t.kx * t.ky).abs().sqrt()
}

fn assert_media_box(doc: &Document, page: ObjectId, l: f32, h: f32, tolerance: f32) {
    let media = nombres(
        doc,
        doc.get_dictionary(page).unwrap().get(b"MediaBox").unwrap(),
    );
    assert_eq!(media.len(), 4);
    for (vu, attendu) in media.iter().zip([0.0, 0.0, l, h]) {
        assert!(
            (vu - attendu).abs() <= tolerance,
            "{media:?} au lieu de [0 0 {l} {h}]"
        );
    }
}

fn resoudre<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    match o {
        Object::Reference(id) => doc.get_object(*id).unwrap(),
        _ => o,
    }
}

fn dictionnaire<'a>(doc: &'a Document, o: &'a Object) -> &'a Dictionary {
    resoudre(doc, o).as_dict().unwrap()
}

fn tableau(doc: &Document, o: &Object) -> Vec<Object> {
    resoudre(doc, o).as_array().unwrap().clone()
}

fn nombre(o: &Object) -> f32 {
    match o {
        Object::Integer(i) => *i as f32,
        Object::Real(r) => *r,
        _ => panic!("pas un nombre : {o:?}"),
    }
}

fn nombres(doc: &Document, o: &Object) -> Vec<f32> {
    tableau(doc, o).iter().map(nombre).collect()
}

/// `a` puis `b`, en matrices PDF `[a b c d e f]`.
fn composer(a: [f32; 6], b: [f32; 6]) -> [f32; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn appliquer(m: [f32; 6], x: f32, y: f32) -> (f32, f32) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}
