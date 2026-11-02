//! La page de la table de la tonalité (MIC-12, § 50 · 3.7) : une colonne par degré, les
//! rangées dans l'ordre (dessus, centre, couche sombre affichée, dessous), chacune sous son
//! nom, leurs cadres et leurs flèches, puis la légende des quatre cadres.
//!
//! La grille est celle de la fiche témoin qu'H09 a validée : marges de 5 mm à gauche et à
//! droite, de 6 mm en haut et en bas, gouttières de 1,5 mm, sept colonnes de 27,21 mm ; les
//! cases sombres se partagent la largeur. Le contenu se calcule depuis la [`Table`]. Le pied
//! est réservé au bas de la page, sur deux lignes à 11 pt : le cahier le pose, puisqu'il
//! porte l'empreinte de la page sans pied.

use super::case::{
    self, DEMI_POINTE, Etat, PASTILLE, POINTE, cadre, fleche, pastille, pictogramme,
};
use super::pages::{HAUTEUR, LARGEUR, Rect};
use super::svg::{Couleur, Element, Page, TRAIT_FIN};
use super::texte::{Corps, MotTropLong, chasse, composer, couper};
use super::{Debordement, Instrument};
use crate::table::{Case, CoucheSombre, Famille, Position, Rangee, Table};
use crate::tonalite::{Mode, Tonalite};

// --- La grille ---------------------------------------------------------------

pub(crate) const MARGE_V: f32 = 6.0;
pub(crate) const MARGE_H: f32 = 5.0;
pub(crate) const GOUTTIERE: f32 = 1.5;
pub(crate) const UTILE_L: f32 = LARGEUR - 2.0 * MARGE_H;
pub(crate) const UTILE_H: f32 = HAUTEUR - 2.0 * MARGE_V;
/// Une colonne par degré : sept colonnes et six gouttières.
pub(crate) const COLONNE: f32 = (UTILE_L - 6.0 * GOUTTIERE) / 7.0;
/// Le retrait du texte dans sa case, de chaque côté.
pub(crate) const RETRAIT: f32 = 1.0;
/// L'espace entre un filet et le texte qui le suit ou le précède.
pub(crate) const SOUS_FILET: f32 = 1.2;
/// Le pied, que le cahier pose : un filet et deux lignes à 11 pt (R3).
pub(crate) const LIGNES_DU_PIED: usize = 2;

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

/// Le jour entre le nom d'une rangée et une flèche qui passe à côté.
const JOUR_NOM: f32 = 2.0;
/// La butée d'une flèche : la plus petite distance de sa pointe au côté de sa cible.
const BUTEE: f32 = 4.0;
/// Au-delà de l'arrondi ou de la coupe d'une cible, le jeu qui garde la pointe sur son bord
/// droit (un défaut relevé par H09 : la butée de 4 mm passait sous l'arrondi de 4,36 mm).
const JEU_DU_COIN: f32 = 0.5;
/// La queue d'une flèche quitte le bord droit de sa case sombre, à ce jeu de la coupe.
const JEU_DE_LA_QUEUE: f32 = 2.0;
/// Sous cet écart, une flèche presque droite se redresse.
const REDRESSE: f32 = 1.5;
/// Deux pointes sur une même cible restent à cette distance l'une de l'autre (vigilance 4).
const ECART_DES_POINTES: f32 = 3.0;

pub(crate) fn x_colonne(c: usize) -> f32 {
    MARGE_H + c as f32 * (COLONNE + GOUTTIERE)
}

/// Le corps du nom d'un accord.
pub(crate) fn corps_nom() -> Corps {
    Corps::pt(14.0).expect("14 pt dépasse le minimum")
}

/// L'interligne : 1,2 cadratin.
pub(crate) fn interligne(c: &Corps) -> f32 {
    1.2 * c.mm()
}

/// Du haut d'une ligne à sa ligne de base : la ligne de 1,2 cadratin centre les
/// 0,928 cadratin au-dessus de la ligne de base et les 0,236 au-dessous de DejaVu Sans.
pub(crate) fn jusqua_la_base(c: &Corps) -> f32 {
    0.946 * c.mm()
}

/// De la ligne de base au milieu des capitales (0,729 cadratin de haut) : là se centrent
/// une pastille ou un pictogramme posé devant un texte.
pub(crate) fn mi_capitale(c: &Corps) -> f32 {
    0.365 * c.mm()
}

/// La hauteur réservée au pied, filet compris.
pub(crate) fn hauteur_du_pied() -> f32 {
    SOUS_FILET + LIGNES_DU_PIED as f32 * interligne(&Corps::MIN)
}

/// Coupe un texte de la forme « a · b · c » de préférence aux « · », et chaque segment
/// trop long entre deux mots.
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

fn entete(table: &Table, instrument: Instrument) -> String {
    format!(
        "{} · table de la tonalité · tonalité choisie · instrument : {} · couche sombre : {}",
        table.tonalite.nom(),
        instrument.nom(),
        match table.couche_sombre {
            CoucheSombre::Masquee => "masquée",
            CoucheSombre::Affichee => "affichée",
        }
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

/// Un nombre de millimètres, à la française.
fn mm(v: f32) -> String {
    format!("{v:.1} mm").replace('.', ",")
}

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

/// Une rangée à sa place : sa hauteur est celle de sa case la plus haute.
struct RangeePlacee<'a> {
    rangee: Rangee,
    nom: String,
    y_nom: f32,
    cases: Vec<Placee<'a>>,
}

/// L'abscisse et la largeur d'une case : sa colonne, ou, dans la couche sombre, sa part de
/// la largeur.
fn colonne_de(p: Position, n_sombres: usize) -> (f32, f32) {
    match p.rangee {
        Rangee::Sombre => {
            let n = n_sombres as f32;
            let l = (UTILE_L - (n - 1.0) * GOUTTIERE) / n;
            (MARGE_H + f32::from(p.rang) * (l + GOUTTIERE), l)
        }
        _ => (x_colonne(usize::from(p.rang)), COLONNE),
    }
}

/// Les textes des cases d'une rangée, et sa hauteur ; refuse un mot plus large que sa case.
fn textes_de_rangee<'a>(
    table: &'a Table,
    rangee: Rangee,
    contexte: &str,
) -> Result<(Vec<Placee<'a>>, f32), Debordement> {
    let (t, corps_du_nom) = (Corps::MIN, corps_nom());
    let n_sombres = table.rangee(Rangee::Sombre).len();
    let mut cases = Vec::new();
    let mut hauteur: f32 = 0.0;
    for c in table.rangee(rangee) {
        let (x, l) = colonne_de(c.position, n_sombres);
        let place = l - 2.0 * RETRAIT;
        let nom = table.nom(c);
        let mut tete = chasse(&c.chiffre, &t);
        if c.fonction.is_some() {
            tete += ECART_PASTILLE + PASTILLE;
        }
        for (texte, largeur) in [(&c.chiffre, tete), (&nom, chasse(&nom, &corps_du_nom))] {
            if largeur > place {
                return Err(Debordement {
                    quoi: format!(
                        "{contexte} : « {texte} » mesure {}, sa case {}",
                        mm(largeur),
                        mm(place)
                    ),
                });
            }
        }
        let effet = couper(&table.ligne_d_effet(c), &t, place).map_err(|e| Debordement {
            quoi: format!(
                "{contexte} : « {} » mesure {}, sa case {}",
                e.mot,
                mm(e.chasse),
                mm(e.largeur)
            ),
        })?;
        hauteur = hauteur.max(hauteur_de_case(c.famille, effet.len()));
        cases.push(Placee {
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
    Ok((cases, hauteur))
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

/// Le même élément, le noir et le blanc échangés : le contenu d'une case cible.
fn inverser(e: Element) -> Element {
    let inverse = |c: Couleur| match c {
        Couleur::Noir => Couleur::Blanc,
        Couleur::Blanc => Couleur::Noir,
    };
    match e {
        Element::Groupe {
            id,
            translation,
            echelle,
            enfants,
        } => Element::Groupe {
            id,
            translation,
            echelle,
            enfants: enfants.into_iter().map(inverser).collect(),
        },
        Element::Chemin { d, plein, trait_ } => Element::Chemin {
            d,
            plein: plein.map(inverse),
            trait_: trait_.map(|(c, e)| (inverse(c), e)),
        },
        Element::Rect {
            r,
            id,
            plein,
            trait_,
        } => Element::Rect {
            r,
            id,
            plein: plein.map(inverse),
            trait_: trait_.map(|(c, e)| (inverse(c), e)),
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
            plein: plein.map(inverse),
            trait_: trait_.map(|(c, e)| (inverse(c), e)),
        },
        Element::Ligne { de, a, trait_ } => Element::Ligne {
            de,
            a,
            trait_: (inverse(trait_.0), trait_.1),
        },
    }
}

// --- Les flèches ---------------------------------------------------------------

/// Une flèche de résolution, et la bande qu'elle franchit entre deux rangées.
struct Trait {
    source: Position,
    cible: Position,
    queue: (f32, f32),
    pointe: (f32, f32),
    /// Une flèche droite peut glisser entre ces abscisses, sans quitter le bord droit de sa
    /// case ni celui de sa cible ; une flèche oblique ou partie d'une pointe ne glisse pas.
    glisse: Option<(f32, f32)>,
    bande: usize,
}

impl Trait {
    /// L'étendue horizontale de la flèche, pointe comprise.
    fn etendue(&self) -> (f32, f32) {
        (
            self.queue.0.min(self.pointe.0) - DEMI_POINTE,
            self.queue.0.max(self.pointe.0) + DEMI_POINTE,
        )
    }
}

/// La plus petite distance de la pointe au côté de la cible : la butée, et toujours
/// au-delà de l'arrondi ou de la coupe.
fn butee(famille: Famille, r: Rect) -> f32 {
    (case::coin(famille, r) + DEMI_POINTE + JEU_DU_COIN).max(BUTEE)
}

/// La flèche d'une case sombre : elle part du milieu de sa case et vise sa cible au plus
/// près, droite quand la cible est en face, oblique sinon.
fn trait_sombre(source: &Placee, cible: &Placee, bande: usize) -> Trait {
    let (rs, rc) = (source.rect, cible.rect);
    let vers_le_bas = cible.case.position.rangee == Rangee::Dessous;
    let (y_queue, y_pointe) = if vers_le_bas {
        (rs.y + rs.h, rc.y)
    } else {
        (rs.y, rc.y + rc.h)
    };
    let jeu_s = case::coin(Famille::Sombre, rs) + JEU_DE_LA_QUEUE;
    let (s_lo, s_hi) = (rs.x + jeu_s, rs.x + rs.l - jeu_s);
    let b = butee(cible.case.famille, rc);
    let (c_lo, c_hi) = (rc.x + b, rc.x + rc.l - b);
    let xs = rs.x + rs.l / 2.0;
    let xc = xs.clamp(c_lo, c_hi);
    let droite = (xc - xs).abs() <= REDRESSE && (s_lo..=s_hi).contains(&xc);
    Trait {
        source: source.case.position,
        cible: cible.case.position,
        queue: (if droite { xc } else { xs }, y_queue),
        pointe: (xc, y_pointe),
        glisse: droite.then_some((s_lo.max(c_lo), s_hi.min(c_hi))),
        bande,
    }
}

/// Les glissements qui libèrent le nom de chaque bande posé en `x`, ou rien si une flèche
/// ne peut s'écarter. Deux pointes sur une même cible doivent rester à part.
fn essayer(x: f32, largeurs: &[f32], traits: &[Trait]) -> Option<Vec<(usize, f32)>> {
    let mut glissements = Vec::new();
    for (bande, largeur) in largeurs.iter().enumerate() {
        if x + largeur > MARGE_H + UTILE_L {
            return None;
        }
        let (g, d) = (x - JOUR_NOM, x + largeur + JOUR_NOM);
        for (i, t) in traits.iter().enumerate().filter(|(_, t)| t.bande == bande) {
            let (a0, a1) = t.etendue();
            if a1 <= g || a0 >= d {
                continue;
            }
            let (lo, hi) = t.glisse?;
            let a_gauche = (g - DEMI_POINTE).min(hi);
            let a_droite = (d + DEMI_POINTE).max(lo);
            let possibles = [
                (a_gauche >= lo).then_some(a_gauche),
                (a_droite <= hi).then_some(a_droite),
            ];
            let nouveau = possibles
                .into_iter()
                .flatten()
                .min_by(|p, q| (p - t.queue.0).abs().total_cmp(&(q - t.queue.0).abs()))?;
            glissements.push((i, nouveau));
        }
    }
    let pointe = |i: usize| {
        let p = traits[i].pointe;
        glissements
            .iter()
            .find(|(j, _)| *j == i)
            .map_or(p, |(_, x)| (*x, p.1))
    };
    for i in 0..traits.len() {
        for j in i + 1..traits.len() {
            let (p, q) = (pointe(i), pointe(j));
            let d = ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2)).sqrt();
            if traits[i].cible == traits[j].cible && d < ECART_DES_POINTES {
                return None;
            }
        }
    }
    Some(glissements)
}

/// L'abscisse commune des noms de rangée : celle, de gauche à droite, qui demande le moins
/// de flèches à pousser. Une colonne d'abord, comme la fiche témoin ; sinon, juste à droite
/// d'une flèche droite poussée au bord de sa case.
fn placer_les_noms(largeurs: &[f32], traits: &mut [Trait]) -> Option<f32> {
    let mut candidats: Vec<f32> = (0..7).map(|k| x_colonne(k) + RETRAIT).collect();
    candidats.extend(
        traits
            .iter()
            .filter_map(|t| t.glisse.map(|(lo, _)| lo + DEMI_POINTE + JOUR_NOM)),
    );
    candidats.sort_by(f32::total_cmp);
    let mut retenu: Option<(f32, Vec<(usize, f32)>)> = None;
    for x in candidats {
        if let Some(g) = essayer(x, largeurs, traits)
            && retenu.as_ref().is_none_or(|(_, r)| g.len() < r.len())
        {
            retenu = Some((x, g));
        }
    }
    let (x, glissements) = retenu?;
    for (i, nouveau) in glissements {
        traits[i].queue.0 = nouveau;
        traits[i].pointe.0 = nouveau;
    }
    Some(x)
}

// --- La page -------------------------------------------------------------------

/// La page de la table, en paysage : l'en-tête dit la tonalité et les réglages ; la case
/// `cible`, s'il y en a une, est en inverse. Rend `Debordement` si un mot dépasse sa case ou
/// si la page ne tient pas dans ses 145 mm utiles, pied réservé.
pub fn page_table(
    table: &Table,
    instrument: Instrument,
    cible: Option<Position>,
) -> Result<PageTable, Debordement> {
    let t = Corps::MIN;
    let il = interligne(&t);
    let contexte = format!(
        "la table de {}, couche sombre {}",
        table.tonalite.nom(),
        match table.couche_sombre {
            CoucheSombre::Masquee => "masquée",
            CoucheSombre::Affichee => "affichée",
        }
    );
    let trop_long = |e: MotTropLong| Debordement {
        quoi: format!(
            "{contexte} : « {} » mesure {}, sa place {}",
            e.mot,
            mm(e.chasse),
            mm(e.largeur)
        ),
    };
    let lignes_d_entete =
        couper_aux_points(&entete(table, instrument), &t, UTILE_L).map_err(trop_long)?;

    // La mise en page, de haut en bas : chaque rangée prend la hauteur de sa case la plus
    // haute, et le nom de la rangée qui suit se loge entre deux.
    let y_filet = MARGE_V + lignes_d_entete.len() as f32 * il + SOUS_FILET / 2.0;
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
        return Err(Debordement {
            quoi: format!(
                "{contexte} : la page mesure {} de haut, pour {} utiles",
                mm(hauteur),
                mm(UTILE_H)
            ),
        });
    }
    let fin_legende = legende_largeur();
    if fin_legende > UTILE_L {
        return Err(Debordement {
            quoi: format!(
                "{contexte} : la légende mesure {}, pour {} utiles",
                mm(fin_legende),
                mm(UTILE_L)
            ),
        });
    }

    // Les flèches, d'une rangée à sa voisine. La bande `k` est au-dessus de la rangée `k`.
    let placee = |p: Position| {
        rangees
            .iter()
            .flat_map(|r| &r.cases)
            .find(|c| c.case.position == p)
    };
    let bande_de = |r: Rangee| {
        rangees
            .iter()
            .position(|x| x.rangee == r)
            .expect("la rangée est sur la page")
    };
    let mut traits = Vec::new();
    for source in rangees.iter().flat_map(|r| &r.cases) {
        let Some(vers) = source.case.cible.and_then(placee) else {
            continue;
        };
        let (rs, rc) = (source.rect, vers.rect);
        traits.push(match source.case.position.rangee {
            Rangee::Dessus => {
                // De la pointe de la dominante secondaire, droit sur sa cible.
                let x = rs.x + rs.l / 2.0;
                Trait {
                    source: source.case.position,
                    cible: vers.case.position,
                    queue: (x, rs.y + rs.h),
                    pointe: (x, rc.y),
                    glisse: None,
                    bande: bande_de(Rangee::Centre),
                }
            }
            _ => {
                // La bande sous la couche sombre si la cible est un emprunt, dessus sinon.
                let bande = if vers.case.position.rangee == Rangee::Dessous {
                    bande_de(Rangee::Dessous)
                } else {
                    bande_de(Rangee::Sombre)
                };
                trait_sombre(source, vers, bande)
            }
        });
    }

    // Les noms de rangée, sur une même abscisse, là où nulle flèche ne passe.
    let mut largeurs: Vec<f32> = rangees.iter().map(|r| chasse(&r.nom, &t)).collect();
    if masquee {
        // « couche sombre masquée » se loge dans la bande des emprunts, au-dessus de leur nom.
        let k = bande_de(Rangee::Dessous);
        largeurs[k] = largeurs[k].max(chasse(COUCHE_MASQUEE, &t));
    }
    let Some(x_nom) = placer_les_noms(&largeurs, &mut traits) else {
        return Err(Debordement {
            quoi: format!(
                "{contexte} : les noms de rangée ne trouvent pas de place entre les flèches"
            ),
        });
    };

    // Le dessin.
    let mut elements = Vec::new();
    for (k, ligne) in lignes_d_entete.iter().enumerate() {
        elements.push(composer(
            ligne,
            &t,
            MARGE_H,
            MARGE_V + k as f32 * il + jusqua_la_base(&t),
        ));
    }
    elements.push(Element::Ligne {
        de: (MARGE_H, y_filet),
        a: (MARGE_H + UTILE_L, y_filet),
        trait_: (Couleur::Noir, TRAIT_FIN),
    });
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
                Etat::Cible => elements.extend(contenu.into_iter().map(inverser)),
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
fn legende_largeur() -> f32 {
    let t = Corps::MIN;
    let mut l = chasse(TITRE_DE_LEGENDE, &t) + APRES_LE_TITRE;
    for (k, (_, nom)) in LEGENDE.iter().enumerate() {
        if k > 0 {
            l += ENTRE_LES_CADRES;
        }
        l += case::PICTOGRAMME_L + APRES_LE_PICTOGRAMME + chasse(nom, &t);
    }
    l
}

/// La légende des quatre cadres, sur une ligne, dès `haut` ; rend ses éléments et sa
/// hauteur. Chaque pictogramme se centre sur le milieu des capitales de son nom.
fn legende(haut: f32) -> (Vec<Element>, f32) {
    let t = Corps::MIN;
    let hauteur = case::PICTOGRAMME_H.max(interligne(&t));
    let milieu = haut + hauteur / 2.0;
    let base = milieu + mi_capitale(&t);
    let mut elements = Vec::new();
    let mut x = MARGE_H;
    elements.push(composer(TITRE_DE_LEGENDE, &t, x, base));
    x += chasse(TITRE_DE_LEGENDE, &t) + APRES_LE_TITRE;
    for (famille, nom) in LEGENDE {
        elements.push(pictogramme(famille, x, milieu - case::PICTOGRAMME_H / 2.0));
        x += case::PICTOGRAMME_L + APRES_LE_PICTOGRAMME;
        elements.push(composer(nom, &t, x, base));
        x += chasse(nom, &t) + ENTRE_LES_CADRES;
    }
    (elements, hauteur)
}
