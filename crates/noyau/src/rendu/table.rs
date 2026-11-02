//! La page de la table de la tonalité (MIC-12, § 50 · 3.7) : une colonne par degré, les
//! rangées dans l'ordre (dessus, centre, couche sombre affichée, dessous), chacune sous son
//! nom, leurs cadres et leurs flèches, puis la légende des quatre cadres.
//!
//! La grille est celle de la fiche témoin qu'H09 a validée ([`super::grille`]) ; les cases
//! sombres, d'un huitième de la largeur, se placent au plus près de leur cible. Le contenu se
//! calcule depuis la [`Table`]. Le pied est réservé au bas de la page, sur deux lignes à
//! 11 pt : le cahier le pose, puisqu'il porte l'empreinte de la page sans pied.

use super::case::{
    Etat, PASTILLE, PICTOGRAMME_H, PICTOGRAMME_L, POINTE, cadre, fleche, pastille, pictogramme,
};
use super::fleches::{self, Bout};
use super::grille::{
    COLONNE, LIGNES_D_EN_TETE, MARGE_H, MARGE_V, RETRAIT, UTILE_H, UTILE_L, au_plus_pres,
    composer_l_en_tete, corps_nom, hauteur_du_pied, interligne, jusqua_la_base, largeur_sombre,
    mi_capitale, milieu_de_colonne, x_colonne,
};
use super::pages::{HAUTEUR, LARGEUR, Rect};
use super::svg::{Element, Page};
use super::texte::{Corps, chasse, composer, couper};
use super::{Contexte, Debordement, Instrument, en_mm};
use crate::table::{Case, CoucheSombre, Famille, Position, Rangee, Table};
use crate::tonalite::{Mode, Tonalite};

/// Au-dessus et au-dessous du nom d'une rangée, entre deux rangées.
const AUTOUR_DU_NOM: f32 = 0.5;
/// Le blanc entre le filet de l'en-tête et le nom de la première rangée.
const SOUS_L_EN_TETE: f32 = 1.5;
/// Sous le nom de la première rangée, que nulle flèche ne traverse.
const SOUS_LE_PREMIER_NOM: f32 = 0.4;
/// Le blanc qui détache « couche sombre masquée » du nom de la rangée qui suit.
const APRES_LA_COUCHE_MASQUEE: f32 = 1.5;
/// Le blanc entre la dernière rangée et la légende.
const AVANT_LA_LEGENDE: f32 = 1.8;
/// L'écart entre le chiffre d'un accord du centre et sa pastille.
const ECART_PASTILLE: f32 = 1.2;

// --- Ce que rend la page -----------------------------------------------------

/// La place de chaque case sur la page : son cadre y tient, pointe comprise.
#[derive(Clone, Debug, PartialEq)]
pub struct Disposition {
    pub cases: Vec<(Position, Rect)>,
}

/// La page, la place de ses cases, et le point d'arrivée de chaque flèche, rangé sous la
/// case d'où elle part, dans l'ordre où les flèches s'écrivent.
#[derive(Clone, Debug, PartialEq)]
pub struct PageTable {
    pub page: Page,
    pub disposition: Disposition,
    pub arrivees: Vec<(Position, (f32, f32))>,
}

// --- Les textes ----------------------------------------------------------------

/// « masquée » ou « affichée ».
fn dite(c: CoucheSombre) -> &'static str {
    match c {
        CoucheSombre::Masquee => "masquée",
        CoucheSombre::Affichee => "affichée",
    }
}

fn entete(table: &Table, instrument: Instrument) -> String {
    format!(
        "{} · table de la tonalité · tonalité choisie · instrument : {} · couche sombre : {}",
        table.tonalite.nom(),
        instrument.nom(),
        dite(table.couche_sombre)
    )
}

fn nom_de_rangee(rangee: Rangee, t: Tonalite) -> String {
    match rangee {
        Rangee::Dessus => "dominantes secondaires".to_string(),
        Rangee::Centre => "diatoniques".to_string(),
        Rangee::Sombre => "couche sombre".to_string(),
        Rangee::Dessous => {
            let parallele = Tonalite {
                mode: match t.mode {
                    Mode::Majeur => Mode::Mineur,
                    Mode::Mineur => Mode::Majeur,
                },
                ..t
            };
            format!("emprunts au mode parallèle · {}", parallele.nom())
        }
    }
}

const COUCHE_MASQUEE: &str = "couche sombre masquée";

const LEGENDE: [(Famille, &str); 4] = [
    (Famille::Diatonique, "diatonique"),
    (Famille::DominanteSecondaire, "dominante secondaire"),
    (Famille::Emprunt, "emprunt"),
    (Famille::Sombre, "harmonie sombre"),
];

// --- Les cases -----------------------------------------------------------------

/// Le blanc au-dessus et au-dessous du texte d'une case. Les coins coupés en demandent
/// plus : le chiffre et la dernière ligne s'écartent de la coupe.
fn blancs(f: Famille) -> (f32, f32) {
    match f {
        Famille::Diatonique | Famille::DominanteSecondaire => (0.8, 0.8),
        Famille::Emprunt => (1.0, 1.2),
        Famille::Sombre => (1.4, 4.4),
    }
}

/// La hauteur d'une case à `lignes` lignes d'effet, pointe comprise.
fn hauteur_de_case(f: Famille, lignes: usize) -> f32 {
    let (haut, bas) = blancs(f);
    let (t, nom) = (Corps::MIN, corps_nom());
    let pointe = if f == Famille::DominanteSecondaire {
        POINTE
    } else {
        0.0
    };
    haut + interligne(&t) + interligne(&nom) + lignes as f32 * interligne(&t) + bas + pointe
}

/// Une case à sa place, avec ses textes.
struct Placee<'a> {
    case: &'a Case,
    rect: Rect,
    nom: String,
    effet: Vec<String>,
}

impl Placee<'_> {
    fn bout(&self) -> Bout {
        Bout {
            position: self.case.position,
            famille: self.case.famille,
            rect: self.rect,
        }
    }
}

/// Une rangée à sa place : sa hauteur est celle de sa case la plus haute.
struct RangeePlacee<'a> {
    rangee: Rangee,
    nom: String,
    y_nom: f32,
    cases: Vec<Placee<'a>>,
}

/// Le milieu de la colonne que vise une case : celle de sa cible, ou, sans cible, celle de
/// son degré.
fn milieu_vise(c: &Case) -> f32 {
    let colonne = c.cible.map_or_else(
        || usize::try_from(c.accord.fondamentale.reduit().degres).unwrap_or(0),
        |p| usize::from(p.rang),
    );
    milieu_de_colonne(colonne)
}

/// L'abscisse et la largeur de chaque case d'une rangée : sa colonne ; dans la couche
/// sombre, un huitième de la largeur, au plus près de sa cible.
fn abscisses(cases: &[&Case], rangee: Rangee) -> Vec<(f32, f32)> {
    match rangee {
        Rangee::Sombre => {
            let l = largeur_sombre(cases.len());
            let milieux: Vec<f32> = cases.iter().map(|c| milieu_vise(c)).collect();
            au_plus_pres(&milieux, l)
                .into_iter()
                .map(|x| (x, l))
                .collect()
        }
        _ => cases
            .iter()
            .map(|c| (x_colonne(usize::from(c.position.rang)), COLONNE))
            .collect(),
    }
}

/// Les textes des cases d'une rangée, et sa hauteur ; refuse un mot plus large que sa case.
fn textes_de_rangee<'a>(
    table: &'a Table,
    rangee: Rangee,
    contexte: &Contexte,
) -> Result<(Vec<Placee<'a>>, f32), Debordement> {
    let (t, corps_du_nom) = (Corps::MIN, corps_nom());
    let cases = table.rangee(rangee);
    let mut placees = Vec::new();
    let mut hauteur: f32 = 0.0;
    for (c, (x, l)) in cases.iter().zip(abscisses(&cases, rangee)) {
        let place = l - 2.0 * RETRAIT;
        let nom = table.nom(c);
        let mut tete = chasse(&c.chiffre, &t);
        if c.fonction.is_some() {
            tete += ECART_PASTILLE + PASTILLE;
        }
        for (texte, largeur) in [(&c.chiffre, tete), (&nom, chasse(&nom, &corps_du_nom))] {
            if largeur > place {
                return Err(contexte.trop_large(texte, largeur, place));
            }
        }
        let effet = couper(&table.ligne_d_effet(c), &t, place)
            .map_err(|e| contexte.trop_large(&e.mot, e.chasse, e.largeur))?;
        hauteur = hauteur.max(hauteur_de_case(c.famille, effet.len()));
        placees.push(Placee {
            case: c,
            rect: Rect {
                x,
                y: 0.0,
                l,
                h: 0.0,
            },
            nom,
            effet,
        });
    }
    Ok((placees, hauteur))
}

/// Le chiffre (et la pastille), le nom, les lignes d'effet d'une case, centrés.
fn contenu_de_case(p: &Placee) -> Vec<Element> {
    let (t, corps_du_nom) = (Corps::MIN, corps_nom());
    let r = p.rect;
    let cx = r.x + r.l / 2.0;
    let centre = |texte: &str, corps: &Corps, base: f32| {
        composer(texte, corps, cx - chasse(texte, corps) / 2.0, base)
    };
    let mut elements = Vec::new();
    let mut y = r.y + blancs(p.case.famille).0;
    let base = y + jusqua_la_base(&t);
    match p.case.fonction {
        None => elements.push(centre(&p.case.chiffre, &t, base)),
        Some(f) => {
            // Le chiffre et sa pastille, centrés ensemble.
            let l = chasse(&p.case.chiffre, &t) + ECART_PASTILLE + PASTILLE;
            let x = cx - l / 2.0;
            elements.push(composer(&p.case.chiffre, &t, x, base));
            elements.push(pastille(f, x + l - PASTILLE / 2.0, base - mi_capitale(&t)));
        }
    }
    y += interligne(&t);
    elements.push(centre(
        &p.nom,
        &corps_du_nom,
        y + jusqua_la_base(&corps_du_nom),
    ));
    y += interligne(&corps_du_nom);
    for ligne in &p.effet {
        elements.push(centre(ligne, &t, y + jusqua_la_base(&t)));
        y += interligne(&t);
    }
    elements
}

// --- La page -------------------------------------------------------------------

/// La page de la table, en paysage : l'en-tête dit la tonalité et les réglages ; la case
/// `cible`, s'il y en a une, est en inverse. Rend `Debordement` si un mot dépasse sa place,
/// si la page ne tient pas dans ses 145 mm utiles, pied réservé, ou si ses flèches ne
/// trouvent pas leur place.
pub fn page_table(
    table: &Table,
    instrument: Instrument,
    cible: Option<Position>,
) -> Result<PageTable, Debordement> {
    let t = Corps::MIN;
    let il = interligne(&t);
    let contexte = Contexte(format!(
        "la table de {}, couche sombre {}",
        table.tonalite.nom(),
        dite(table.couche_sombre)
    ));
    // L'en-tête réserve toujours ses deux lignes.
    let (en_tete, y_filet) = composer_l_en_tete(
        &entete(table, instrument),
        LIGNES_D_EN_TETE,
        LIGNES_D_EN_TETE,
        &contexte,
    )?;

    // La mise en page, de haut en bas : chaque rangée prend la hauteur de sa case la plus
    // haute, et le nom de la rangée qui suit se loge entre deux.
    let masquee = table.couche_sombre == CoucheSombre::Masquee;
    let mut ordre = vec![Rangee::Dessus, Rangee::Centre];
    if !masquee {
        ordre.push(Rangee::Sombre);
    }
    ordre.push(Rangee::Dessous);

    let mut rangees: Vec<RangeePlacee> = Vec::new();
    let mut y_masquee = None;
    let mut bas = y_filet + SOUS_L_EN_TETE - AUTOUR_DU_NOM;
    for rangee in ordre {
        if rangee == Rangee::Dessous && masquee {
            // La couche sombre se dit à l'endroit où elle s'insère.
            y_masquee = Some(bas + AUTOUR_DU_NOM);
            bas += AUTOUR_DU_NOM + il + APRES_LA_COUCHE_MASQUEE;
        }
        let y_nom = bas + AUTOUR_DU_NOM;
        let apres_le_nom = if rangee == Rangee::Dessus {
            SOUS_LE_PREMIER_NOM
        } else {
            AUTOUR_DU_NOM
        };
        let y = y_nom + il + apres_le_nom;
        let (mut cases, h) = textes_de_rangee(table, rangee, &contexte)?;
        for c in &mut cases {
            c.rect.y = y;
            c.rect.h = h;
        }
        bas = y + h;
        rangees.push(RangeePlacee {
            rangee,
            nom: nom_de_rangee(rangee, table.tonalite),
            y_nom,
            cases,
        });
    }
    let y_legende = bas + AVANT_LA_LEGENDE;
    let (legende, h_legende) = legende(y_legende);
    let hauteur = y_legende + h_legende - MARGE_V + hauteur_du_pied();
    if hauteur > UTILE_H {
        return Err(contexte.deborde(format_args!(
            "la page mesure {} de haut, pour {} utiles",
            en_mm(hauteur),
            en_mm(UTILE_H)
        )));
    }
    if largeur_de_legende() > UTILE_L {
        return Err(contexte.deborde(format_args!(
            "la légende mesure {}, pour {} utiles",
            en_mm(largeur_de_legende()),
            en_mm(UTILE_L)
        )));
    }

    // Les flèches, d'une rangée à sa voisine : chacune franchit la bande au-dessus de la plus
    // basse de ses deux rangées.
    let placee = |p: Position| {
        rangees
            .iter()
            .flat_map(|r| &r.cases)
            .find(|c| c.case.position == p)
    };
    let rang_sur_la_page = |r: Rangee| rangees.iter().position(|x| x.rangee == r);
    let mut traits = Vec::new();
    for source in rangees.iter().flat_map(|r| &r.cases) {
        let Some(vers) = source.case.cible.and_then(placee) else {
            continue;
        };
        let bande = rang_sur_la_page(source.case.position.rangee)
            .max(rang_sur_la_page(vers.case.position.rangee))
            .expect("les deux rangées sont sur la page");
        traits.push(match source.case.position.rangee {
            Rangee::Dessus => fleches::depuis_une_pointe(source.bout(), vers.bout(), bande),
            _ => fleches::depuis_une_case_sombre(source.bout(), vers.bout(), bande),
        });
    }
    let nom_de = |p: Position| {
        table
            .case(p)
            .map_or_else(|| format!("{p:?}"), |c| table.nom(c))
    };
    if let Some(d) = fleches::defaut(&traits) {
        return Err(contexte.deborde(d.decrire(nom_de)));
    }

    // Les noms de rangée, sur une même abscisse, là où nulle flèche ne passe.
    let mut largeurs: Vec<f32> = rangees.iter().map(|r| chasse(&r.nom, &t)).collect();
    if masquee && let Some(k) = rang_sur_la_page(Rangee::Dessous) {
        // « couche sombre masquée » se loge dans la bande des emprunts, au-dessus de leur nom.
        largeurs[k] = largeurs[k].max(chasse(COUCHE_MASQUEE, &t));
    }
    let x_nom = fleches::placer_les_noms(&largeurs, &mut traits).ok_or_else(|| {
        contexte.deborde("les noms de rangée ne trouvent pas de place entre les flèches")
    })?;
    debug_assert!(
        fleches::defaut(&traits).is_none(),
        "un glissement ne fait naître aucun défaut"
    );

    // Le dessin.
    let mut elements = en_tete;
    for r in &rangees {
        if r.rangee == Rangee::Dessous
            && let Some(y) = y_masquee
        {
            elements.push(composer(COUCHE_MASQUEE, &t, x_nom, y + jusqua_la_base(&t)));
        }
        elements.push(composer(&r.nom, &t, x_nom, r.y_nom + jusqua_la_base(&t)));
        for c in &r.cases {
            let etat = if Some(c.case.position) == cible {
                Etat::Cible
            } else {
                Etat::Ordinaire
            };
            elements.push(cadre(c.case.famille, c.rect, etat));
            let contenu = contenu_de_case(c);
            match etat {
                Etat::Ordinaire => elements.extend(contenu),
                Etat::Cible => elements.extend(contenu.into_iter().map(Element::inverse)),
            }
        }
    }
    for tr in &traits {
        elements.push(fleche(tr.queue, tr.pointe));
    }
    elements.extend(legende);

    let disposition = Disposition {
        cases: table
            .cases
            .iter()
            .filter_map(|c| placee(c.position).map(|p| (c.position, p.rect)))
            .collect(),
    };
    Ok(PageTable {
        page: Page {
            largeur: LARGEUR,
            hauteur: HAUTEUR,
            elements,
        },
        disposition,
        arrivees: traits.iter().map(|tr| (tr.source, tr.pointe)).collect(),
    })
}

// --- La légende ----------------------------------------------------------------

const APRES_LE_TITRE: f32 = 3.0;
const APRES_LE_PICTOGRAMME: f32 = 1.5;
const ENTRE_LES_CADRES: f32 = 4.0;
const TITRE_DE_LEGENDE: &str = "cadres :";

/// La largeur de la légende, sur sa ligne.
fn largeur_de_legende() -> f32 {
    let t = Corps::MIN;
    let mut l = chasse(TITRE_DE_LEGENDE, &t) + APRES_LE_TITRE;
    for (k, (_, nom)) in LEGENDE.iter().enumerate() {
        if k > 0 {
            l += ENTRE_LES_CADRES;
        }
        l += PICTOGRAMME_L + APRES_LE_PICTOGRAMME + chasse(nom, &t);
    }
    l
}

/// La légende des quatre cadres, sur une ligne, dès `haut` ; rend ses éléments et sa
/// hauteur. Chaque pictogramme se centre sur le milieu des capitales de son nom.
fn legende(haut: f32) -> (Vec<Element>, f32) {
    let t = Corps::MIN;
    let hauteur = PICTOGRAMME_H.max(interligne(&t));
    let milieu = haut + hauteur / 2.0;
    let base = milieu + mi_capitale(&t);
    let mut elements = Vec::new();
    let mut x = MARGE_H;
    elements.push(composer(TITRE_DE_LEGENDE, &t, x, base));
    x += chasse(TITRE_DE_LEGENDE, &t) + APRES_LE_TITRE;
    for (famille, nom) in LEGENDE {
        elements.push(pictogramme(famille, x, milieu - PICTOGRAMME_H / 2.0));
        x += PICTOGRAMME_L + APRES_LE_PICTOGRAMME;
        elements.push(composer(nom, &t, x, base));
        x += chasse(nom, &t) + ENTRE_LES_CADRES;
    }
    (elements, hauteur)
}
