//! Le cahier des tables : ses 49 pages, que le noyau dessine, assemblées en un PDF.
//!
//! Chaque page SVG devient, par `svg2pdf`, un XObject de formulaire, posé sur une
//! page de la même taille : 1 mm du SVG fait 1 mm de la page. Le PDF ne porte ni
//! horloge ni identifiant : deux assemblages à la même date sont identiques à
//! l'octet (I25).

use std::collections::BTreeMap;
use std::fmt;

use noyau::rendu::cahier::pages_du_cahier;
use noyau::rendu::pages::{Date, HAUTEUR, LARGEUR, PageDuCahier, Rect};
use noyau::rendu::{Debordement, Instrument};
use pdf_writer::types::AnnotationType;
use pdf_writer::writers::Destination;
use pdf_writer::{Content, Finish, Name, Pdf, Ref, TextStr};
use svg2pdf::usvg;

const PT_PAR_MM: f32 = 72.0 / 25.4;
/// Le nom du XObject de la page dans ses ressources.
const XOBJET: Name<'static> = Name(b"Page");

/// Une page que le PDF ne peut pas prendre. `page` compte depuis 0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurPdf {
    /// Le SVG ne se lit pas.
    Svg { page: usize, message: String },
    /// Le SVG se lit, mais ne se convertit pas, ou un lien mène hors du cahier.
    Conversion { page: usize, message: String },
}

impl fmt::Display for ErreurPdf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurPdf::Svg { page, message } => {
                write!(f, "la page {} ne se lit pas : {message}", page + 1)
            }
            ErreurPdf::Conversion { page, message } => {
                write!(f, "la page {} ne se convertit pas : {message}", page + 1)
            }
        }
    }
}

impl std::error::Error for ErreurPdf {}

/// Ce qui empêche d'engendrer le cahier : une page qui ne tient pas, ou un PDF qui ne
/// s'assemble pas.
#[derive(Debug, Clone, PartialEq)]
pub enum ErreurCahier {
    Debordement(Debordement),
    Pdf(ErreurPdf),
}

impl fmt::Display for ErreurCahier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErreurCahier::Debordement(d) => d.fmt(f),
            ErreurCahier::Pdf(e) => write!(f, "le PDF ne s'assemble pas : {e}"),
        }
    }
}

impl std::error::Error for ErreurCahier {}

impl From<Debordement> for ErreurCahier {
    fn from(d: Debordement) -> Self {
        ErreurCahier::Debordement(d)
    }
}

impl From<ErreurPdf> for ErreurCahier {
    fn from(e: ErreurPdf) -> Self {
        ErreurCahier::Pdf(e)
    }
}

/// Le PDF du cahier des tables pour cet instrument, daté du rendu : les 49 pages de
/// `pages_du_cahier`, assemblées. La date du rendu est une entrée, jamais l'horloge : deux
/// engendrements des mêmes entrées sont identiques à l'octet (I25).
pub fn engendrer(i: Instrument, date: &Date) -> Result<Vec<u8>, ErreurCahier> {
    let pages = pages_du_cahier(i, date)?;
    Ok(assembler(&pages, date)?)
}

/// Assemble les pages en un PDF : une page par SVG, de MediaBox
/// `[0 0 209,5 mm 157 mm]`, et une annotation `/Link` par lien, à destination
/// `/Fit` de sa page. `/Info` porte le producteur et la date du rendu, à minuit
/// UTC ; il n'y a pas d'`/ID`.
pub fn assembler(pages: &[PageDuCahier], date: &Date) -> Result<Vec<u8>, ErreurPdf> {
    let (l, h) = (LARGEUR * PT_PAR_MM, HAUTEUR * PT_PAR_MM);
    let mut suivant = Ref::new(1);
    let catalogue = suivant.bump();
    let arbre = suivant.bump();
    let infos = suivant.bump();
    let refs: Vec<Ref> = pages.iter().map(|_| suivant.bump()).collect();

    let mut pdf = Pdf::new();
    pdf.catalog(catalogue).pages(arbre);
    pdf.pages(arbre)
        .kids(refs.iter().copied())
        .count(i32::try_from(pages.len()).unwrap_or(i32::MAX));

    for (n, page) in pages.iter().enumerate() {
        let svg = usvg::Tree::from_str(&page.svg, &usvg::Options::default()).map_err(|e| {
            ErreurPdf::Svg {
                page: n,
                message: e.to_string(),
            }
        })?;
        let (morceau, xobjet) = svg2pdf::to_chunk(&svg, svg2pdf::ConversionOptions::default())
            .map_err(|e| ErreurPdf::Conversion {
                page: n,
                message: e.to_string(),
            })?;
        let mut correspondance = BTreeMap::new();
        let morceau = morceau.renumber(|ancien| {
            *correspondance
                .entry(ancien)
                .or_insert_with(|| suivant.bump())
        });
        let xobjet = correspondance[&xobjet];
        pdf.extend(&morceau);

        // Le XObject est un carré unité (sa /Matrix ramène sa BBox à 1 × 1) :
        // `cm` l'étire à la page.
        let flux = suivant.bump();
        let mut contenu = Content::new();
        contenu.save_state();
        contenu.transform([l, 0.0, 0.0, h, 0.0, 0.0]);
        contenu.x_object(XOBJET);
        contenu.restore_state();
        pdf.stream(flux, &contenu.finish());

        let annotations: Vec<Ref> = page.liens.iter().map(|_| suivant.bump()).collect();
        let mut ecrite = pdf.page(refs[n]);
        ecrite
            .parent(arbre)
            .media_box(pdf_writer::Rect::new(0.0, 0.0, l, h))
            .contents(flux);
        ecrite.resources().x_objects().pair(XOBJET, xobjet);
        if !annotations.is_empty() {
            ecrite.annotations(annotations.iter().copied());
        }
        ecrite.finish();

        for (lien, id) in page.liens.iter().zip(annotations) {
            let vers = *refs.get(lien.vers).ok_or_else(|| ErreurPdf::Conversion {
                page: n,
                message: format!("lien vers la page {}, hors du cahier", lien.vers),
            })?;
            let mut annotation = pdf.annotation(id);
            annotation
                .subtype(AnnotationType::Link)
                .rect(en_points(&lien.zone))
                .border(0.0, 0.0, 0.0, None);
            annotation
                .insert(Name(b"Dest"))
                .start::<Destination>()
                .page(vers)
                .fit();
        }
    }

    pdf.document_info(infos)
        .producer(TextStr("pupitre"))
        .creation_date(
            pdf_writer::Date::new(date.annee)
                .month(date.mois)
                .day(date.jour)
                .hour(0)
                .minute(0)
                .second(0)
                .utc_offset_hour(0),
        );
    Ok(pdf.finish())
}

/// Une zone en mm depuis le haut de la page, en points depuis le bas.
fn en_points(r: &Rect) -> pdf_writer::Rect {
    pdf_writer::Rect::new(
        r.x * PT_PAR_MM,
        (HAUTEUR - r.y - r.h) * PT_PAR_MM,
        (r.x + r.l) * PT_PAR_MM,
        (HAUTEUR - r.y) * PT_PAR_MM,
    )
}
