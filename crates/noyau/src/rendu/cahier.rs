//! Le cahier des tables (REN-01) : un sommaire, puis les 24 tonalités dans l'ordre du cahier,
//! chacune sur sa page de table suivie de son verso, dans la couche sombre du réglage, masquée
//! par défaut (MIC-12 · 5) ; soit 49 pages. Le cahier pose sur chaque page le pied que la table
//! et le verso réservent : la page, la date du rendu et l'empreinte de la page sans pied.
//!
//! Les pages sont comptées depuis 0 : le sommaire est à l'index 0 (« page 1 / 49 ») ; la table de
//! la k-ième tonalité, à l'index 1 + 2k (« page 2 + 2k »), et son verso juste après.

use std::fmt::Write;

use sha2::{Digest, Sha256};

use super::grille::{
    GOUTTIERE, LIGNES_DU_PIED, MARGE_H, MARGE_V, RETRAIT, SOUS_FILET, UTILE_H, UTILE_L, corps_nom,
    hauteur_du_pied, interligne, jusqua_la_base,
};
use super::pages::{Date, HAUTEUR, LARGEUR, Lien, PageDuCahier, Rect};
use super::svg::{Couleur, Element, Page, TRAIT_FIN};
use super::table::{dite, page_table};
use super::texte::{Corps, chasse, composer, couper_aux_points};
use super::verso::page_verso;
use super::{Contexte, Debordement, Instrument, en_mm};
use crate::table::{CoucheSombre, table};
use crate::tonalite::Tonalite;

/// L'index, depuis 0, de la table de la k-ième tonalité du cahier : le sommaire la précède, et
/// chaque table est suivie de son verso.
fn index_de_la_table(k: usize) -> usize {
    1 + 2 * k
}

// --- L'empreinte et le pied -------------------------------------------------------

/// Les six premiers caractères hexadécimaux du SHA-256 du SVG d'une page sans son pied : ce que
/// le pied appelle l'empreinte.
pub fn empreinte(svg_sans_pied: &str) -> String {
    let condense = Sha256::digest(svg_sans_pied.as_bytes());
    let mut e = String::with_capacity(6);
    for octet in &condense[..3] {
        let _ = write!(e, "{octet:02x}");
    }
    e
}

/// « page 2 / 49 · rendu le 1er nov. 2026 · empreinte 7c1e4a », et, sur une page de table,
/// « · verso : diagrammes et palettes des accords du centre ».
fn texte_du_pied(
    numero: usize,
    total: usize,
    date: &Date,
    empreinte: &str,
    annonce_le_verso: bool,
) -> String {
    let mut texte = format!(
        "page {numero} / {total} · rendu le {} · empreinte {empreinte}",
        date.en_francais()
    );
    if annonce_le_verso {
        texte.push_str(" · verso : diagrammes et palettes des accords du centre");
    }
    texte
}

/// Pose le pied dans la zone que la page lui réserve, au bas de la hauteur utile : un filet de
/// toute la largeur, puis le texte à 11 pt, coupé aux « · » sur deux lignes au plus (R3). Le pied
/// s'ajoute après les éléments de la page : leurs identifiants ne changent pas.
fn poser_le_pied(page: &mut Page, texte: &str, contexte: &Contexte) -> Result<(), Debordement> {
    let t = Corps::MIN;
    let lignes = couper_aux_points(texte, &t, UTILE_L)
        .map_err(|e| contexte.trop_large(&e.mot, e.chasse, e.largeur))?;
    if lignes.len() > LIGNES_DU_PIED {
        return Err(contexte.deborde(format_args!(
            "le pied prend {} lignes, pour {LIGNES_DU_PIED}",
            lignes.len()
        )));
    }
    let haut = MARGE_V + UTILE_H - hauteur_du_pied();
    let y_filet = haut + SOUS_FILET / 2.0;
    page.elements.push(Element::Ligne {
        de: (MARGE_H, y_filet),
        a: (MARGE_H + UTILE_L, y_filet),
        trait_: (Couleur::Noir, TRAIT_FIN),
    });
    for (k, l) in lignes.iter().enumerate() {
        let base = haut + SOUS_FILET + k as f32 * interligne(&t) + jusqua_la_base(&t);
        page.elements.push(composer(l, &t, MARGE_H, base));
    }
    Ok(())
}

// --- Le sommaire --------------------------------------------------------------------

const TITRE: &str = "Cahier des tables";
/// La hauteur d'une entrée, celle de la zone de son lien : au moins 12 mm (§ 50 · 6.3).
const HAUTEUR_D_ENTREE: f32 = 16.0;
/// Le blanc entre deux lignes d'entrées.
const ENTRE_LES_LIGNES: f32 = 3.0;
/// Le blanc entre une majeure et sa relative.
const DANS_LA_PAIRE: f32 = 2.0;
/// Le blanc entre deux paires d'une ligne, plus large, pour que les paires se lisent.
const ENTRE_LES_PAIRES: f32 = 6.0;
/// Deux paires, majeure et relative, par ligne : quatre entrées.
const PAIRES_PAR_LIGNE: usize = 2;
/// Le blanc entre le filet de l'en-tête et la première ligne d'entrées.
const SOUS_L_EN_TETE: f32 = 4.0;

/// La largeur d'une entrée : quatre par ligne, dans la largeur utile.
const LARGEUR_D_ENTREE: f32 = (UTILE_L
    - PAIRES_PAR_LIGNE as f32 * DANS_LA_PAIRE
    - (PAIRES_PAR_LIGNE - 1) as f32 * ENTRE_LES_PAIRES)
    / (2 * PAIRES_PAR_LIGNE) as f32;

const _: () = assert!(
    HAUTEUR_D_ENTREE >= 12.0,
    "une zone de lien de 12 mm au moins"
);
const _: () = assert!(DANS_LA_PAIRE >= GOUTTIERE && ENTRE_LES_PAIRES > DANS_LA_PAIRE);

fn corps_titre() -> Corps {
    Corps::pt(20.0).expect("20 pt dépasse le minimum")
}

/// Le sommaire, sans son pied, et ses liens.
#[derive(Clone, Debug, PartialEq)]
pub struct Sommaire {
    pub page: Page,
    pub liens: Vec<Lien>,
}

/// La zone de la k-ième entrée, sous `haut`. Les entrées se lisent de ligne en ligne et de gauche
/// à droite, dans l'ordre du cahier : chaque ligne porte deux paires, une majeure suivie de sa
/// relative.
fn zone_d_entree(k: usize, haut: f32) -> Rect {
    let paire = k / 2;
    let (ligne, place) = (paire / PAIRES_PAR_LIGNE, paire % PAIRES_PAR_LIGNE);
    let largeur_de_paire = 2.0 * LARGEUR_D_ENTREE + DANS_LA_PAIRE;
    Rect {
        x: MARGE_H
            + place as f32 * (largeur_de_paire + ENTRE_LES_PAIRES)
            + (k % 2) as f32 * (LARGEUR_D_ENTREE + DANS_LA_PAIRE),
        y: haut + ligne as f32 * (HAUTEUR_D_ENTREE + ENTRE_LES_LIGNES),
        l: LARGEUR_D_ENTREE,
        h: HAUTEUR_D_ENTREE,
    }
}

/// Une entrée : son cadre, qui est la zone du lien, puis le nom de la tonalité à 14 pt et sa page
/// à 11 pt, centrés.
fn entree(
    nom: &str,
    page: &str,
    zone: Rect,
    contexte: &Contexte,
) -> Result<Vec<Element>, Debordement> {
    let (t, corps_du_nom) = (Corps::MIN, corps_nom());
    let place = zone.l - 2.0 * RETRAIT;
    for (texte, corps) in [(nom, &corps_du_nom), (page, &t)] {
        if chasse(texte, corps) > place {
            return Err(contexte.trop_large(texte, chasse(texte, corps), place));
        }
    }
    let cx = zone.x + zone.l / 2.0;
    let bloc = interligne(&corps_du_nom) + interligne(&t);
    let haut = zone.y + (zone.h - bloc) / 2.0;
    Ok(vec![
        Element::Rect {
            r: zone,
            id: Some("lien"),
            plein: None,
            trait_: Some((Couleur::Noir, TRAIT_FIN)),
        },
        composer(
            nom,
            &corps_du_nom,
            cx - chasse(nom, &corps_du_nom) / 2.0,
            haut + jusqua_la_base(&corps_du_nom),
        ),
        composer(
            page,
            &t,
            cx - chasse(page, &t) / 2.0,
            haut + interligne(&corps_du_nom) + jusqua_la_base(&t),
        ),
    ])
}

/// La page du sommaire, sans son pied : le titre « Cahier des tables » à 20 pt, une ligne qui dit
/// l'instrument et la couche sombre du cahier, un filet, puis une entrée par tonalité, dans
/// l'ordre du cahier, chacune avec sa page et la zone de son lien vers sa table. Rend
/// `Debordement` si un texte dépasse sa place ou si les entrées descendent dans le pied.
pub fn page_sommaire(i: Instrument, couche: CoucheSombre) -> Result<Sommaire, Debordement> {
    let contexte = Contexte("le sommaire du cahier".to_string());
    let (t, titre) = (Corps::MIN, corps_titre());
    if chasse(TITRE, &titre) > UTILE_L {
        return Err(contexte.trop_large(TITRE, chasse(TITRE, &titre), UTILE_L));
    }
    let mut elements = vec![composer(
        TITRE,
        &titre,
        MARGE_H,
        MARGE_V + jusqua_la_base(&titre),
    )];
    let mut y = MARGE_V + interligne(&titre);
    let reglages = format!(
        "instrument : {} · couche sombre : {}",
        i.nom(),
        dite(couche)
    );
    let lignes = couper_aux_points(&reglages, &t, UTILE_L)
        .map_err(|e| contexte.trop_large(&e.mot, e.chasse, e.largeur))?;
    for l in &lignes {
        elements.push(composer(l, &t, MARGE_H, y + jusqua_la_base(&t)));
        y += interligne(&t);
    }
    let y_filet = y + SOUS_FILET / 2.0;
    elements.push(Element::Ligne {
        de: (MARGE_H, y_filet),
        a: (MARGE_H + UTILE_L, y_filet),
        trait_: (Couleur::Noir, TRAIT_FIN),
    });

    let haut = y_filet + SOUS_L_EN_TETE;
    let bas = MARGE_V + UTILE_H - hauteur_du_pied();
    let mut liens = Vec::new();
    for (k, ton) in Tonalite::les_24().into_iter().enumerate() {
        let zone = zone_d_entree(k, haut);
        if zone.y + zone.h > bas {
            return Err(contexte.deborde(format_args!(
                "les entrées descendent à {}, le pied commence à {}",
                en_mm(zone.y + zone.h),
                en_mm(bas)
            )));
        }
        let vers = index_de_la_table(k);
        let page = format!("page {}", vers + 1);
        elements.extend(entree(&ton.nom(), &page, zone, &contexte)?);
        liens.push(Lien { zone, vers });
    }
    Ok(Sommaire {
        page: Page {
            largeur: LARGEUR,
            hauteur: HAUTEUR,
            elements,
        },
        liens,
    })
}

// --- Les 49 pages -------------------------------------------------------------------

/// Une page du cahier avant son pied.
struct SansPied {
    page: Page,
    liens: Vec<Lien>,
    annonce_le_verso: bool,
}

/// Les 49 pages du cahier, pied posé : le sommaire, puis, pour chaque tonalité de l'ordre du
/// cahier, sa page de table et son verso, dans la couche sombre donnée. Le pied de chaque page
/// porte l'empreinte de son SVG sans pied, tel que le rendent [`page_sommaire`], [`page_table`]
/// et [`page_verso`]. Rend le premier `Debordement` d'une page.
pub fn pages_du_cahier(
    i: Instrument,
    couche: CoucheSombre,
    date: &Date,
) -> Result<Vec<PageDuCahier>, Debordement> {
    let sommaire = page_sommaire(i, couche)?;
    let mut pages = vec![SansPied {
        page: sommaire.page,
        liens: sommaire.liens,
        annonce_le_verso: false,
    }];
    for (k, t) in Tonalite::les_24().into_iter().enumerate() {
        let tb = table(t, couche);
        // La table de la k-ième tonalité entre à l'index que vise le k-ième lien du sommaire.
        debug_assert_eq!(pages.len(), index_de_la_table(k));
        pages.push(SansPied {
            page: page_table(&tb, i, None)?.page,
            liens: Vec::new(),
            annonce_le_verso: true,
        });
        pages.push(SansPied {
            page: page_verso(&tb, i)?,
            liens: Vec::new(),
            annonce_le_verso: false,
        });
    }
    let total = pages.len();
    pages
        .into_iter()
        .enumerate()
        .map(|(n, mut p)| {
            let e = empreinte(&p.page.svg());
            let texte = texte_du_pied(n + 1, total, date, &e, p.annonce_le_verso);
            let contexte = Contexte(format!("le pied de la page {}", n + 1));
            poser_le_pied(&mut p.page, &texte, &contexte)?;
            Ok(PageDuCahier {
                svg: p.page.svg(),
                liens: p.liens,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATE: Date = Date {
        annee: 2026,
        mois: 11,
        jour: 1,
    };

    fn page_vide() -> Page {
        Page {
            largeur: LARGEUR,
            hauteur: HAUTEUR,
            elements: Vec::new(),
        }
    }

    #[test]
    fn le_pied_le_plus_long_tient_sur_ses_deux_lignes() {
        let texte = texte_du_pied(
            49,
            49,
            &Date {
                annee: 2026,
                mois: 9,
                jour: 30,
            },
            "7c1e4a",
            true,
        );
        let mut page = page_vide();
        assert_eq!(
            poser_le_pied(&mut page, &texte, &Contexte("essai".to_string())),
            Ok(())
        );
        // Un filet et deux lignes.
        assert_eq!(page.elements.len(), 3);
    }

    #[test]
    fn un_pied_de_trois_lignes_deborde() {
        // Le verso annoncé deux fois : la seconde annonce ne tient plus sur la deuxième ligne.
        let texte = format!(
            "{} · verso : diagrammes et palettes des accords du centre",
            texte_du_pied(2, 49, &DATE, "7c1e4a", true)
        );
        let e = poser_le_pied(&mut page_vide(), &texte, &Contexte("essai".to_string()));
        assert_eq!(
            e.map_err(|d| d.quoi),
            Err("essai : le pied prend 3 lignes, pour 2".to_string())
        );
    }

    #[test]
    fn la_k_ieme_table_est_a_l_index_1_plus_2k() {
        assert_eq!(index_de_la_table(0), 1);
        assert_eq!(index_de_la_table(12), 25);
        assert_eq!(index_de_la_table(23), 47);
    }
}
