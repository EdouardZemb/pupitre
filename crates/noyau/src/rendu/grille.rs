//! La grille des pages en paysage, celle qu'H09 a validée : marges de 5 mm à gauche et à droite, de
//! 6 mm en haut et en bas, gouttières de 1,5 mm, sept colonnes de 27,21 mm, et la mesure des lignes
//! de texte. La page de la table, son verso et le cahier la partagent.

use super::pages::{HAUTEUR, LARGEUR};
use super::svg::{Couleur, Element, TRAIT_FIN};
use super::texte::{Corps, composer, couper_aux_points};
use super::{Contexte, Debordement};

pub(crate) const MARGE_V: f32 = 6.0;
pub(crate) const MARGE_H: f32 = 5.0;
pub(crate) const GOUTTIERE: f32 = 1.5;
pub(crate) const UTILE_L: f32 = LARGEUR - 2.0 * MARGE_H;
pub(crate) const UTILE_H: f32 = HAUTEUR - 2.0 * MARGE_V;
/// Une colonne par degré : sept colonnes et six gouttières.
pub(crate) const COLONNE: f32 = (UTILE_L - 6.0 * GOUTTIERE) / 7.0;
/// Une case sombre : un huitième de la largeur, les huit cases sombres du mineur la
/// remplissant.
pub(crate) const CASE_SOMBRE: f32 = (UTILE_L - 7.0 * GOUTTIERE) / 8.0;
/// Le retrait du texte dans sa case, de chaque côté.
pub(crate) const RETRAIT: f32 = 1.0;
/// L'espace entre un filet et le texte qui le suit ou le précède.
pub(crate) const SOUS_FILET: f32 = 1.2;
/// L'en-tête, toujours sur deux lignes, même quand il tiendrait sur une : le haut de la
/// table ne bouge pas d'une page à l'autre du cahier (R3).
pub(crate) const LIGNES_D_EN_TETE: usize = 2;
/// Le pied, que le cahier pose : un filet et deux lignes à 11 pt (R3).
pub(crate) const LIGNES_DU_PIED: usize = 2;

/// Le bord gauche de la colonne `c`.
pub(crate) fn x_colonne(c: usize) -> f32 {
    MARGE_H + c as f32 * (COLONNE + GOUTTIERE)
}

/// Le milieu de la colonne `c`.
pub(crate) fn milieu_de_colonne(c: usize) -> f32 {
    x_colonne(c) + COLONNE / 2.0
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

/// L'en-tête d'une page : `texte` coupé aux « · » dans la largeur utile, en `lignes_max` lignes
/// au plus, à 11 pt, puis un filet de toute la largeur. Il réserve au moins `lignes_reservees`
/// lignes. Rend ses éléments et la hauteur du filet ; un mot plus large que la page, ou des
/// lignes de trop, sont un `Debordement`. La page de la table, son verso et la fiche d'un
/// accord le partagent.
pub(crate) fn composer_l_en_tete(
    texte: &str,
    lignes_reservees: usize,
    lignes_max: usize,
    contexte: &Contexte,
) -> Result<(Vec<Element>, f32), Debordement> {
    let t = Corps::MIN;
    let il = interligne(&t);
    let lignes = couper_aux_points(texte, &t, UTILE_L)
        .map_err(|e| contexte.trop_large(&e.mot, e.chasse, e.largeur))?;
    if lignes.len() > lignes_max {
        return Err(contexte.deborde(format_args!(
            "l'en-tête prend {} lignes, pour {lignes_max}",
            lignes.len()
        )));
    }
    let mut elements: Vec<Element> = lignes
        .iter()
        .enumerate()
        .map(|(k, l)| composer(l, &t, MARGE_H, MARGE_V + k as f32 * il + jusqua_la_base(&t)))
        .collect();
    let y_filet = MARGE_V + lignes.len().max(lignes_reservees) as f32 * il + SOUS_FILET / 2.0;
    elements.push(Element::Ligne {
        de: (MARGE_H, y_filet),
        a: (MARGE_H + UTILE_L, y_filet),
        trait_: (Couleur::Noir, TRAIT_FIN),
    });
    Ok((elements, y_filet))
}

/// La hauteur réservée au pied, filet compris.
pub(crate) fn hauteur_du_pied() -> f32 {
    SOUS_FILET + LIGNES_DU_PIED as f32 * interligne(&Corps::MIN)
}

/// La largeur des cases sombres d'une rangée : un huitième de la largeur, ou leur part si
/// elles sont plus de huit.
pub(crate) fn largeur_sombre(n: usize) -> f32 {
    let n = n.max(1) as f32;
    CASE_SOMBRE.min((UTILE_L - (n - 1.0) * GOUTTIERE) / n)
}

/// Les bords gauches de cases de largeur `l`, dans l'ordre, séparées d'une gouttière au
/// moins, dans la largeur utile, chacune au plus près du milieu qu'elle vise (au sens des
/// moindres carrés).
///
/// Retrancher `k` pas au milieu visé par la `k`-ième case change l'ordre et la gouttière en
/// une suite croissante : c'est une régression isotone, que l'on résout en fusionnant les
/// blocs voisins qui se croisent. Huit cases sombres remplissent la largeur et ne bougent
/// pas ; les sept du majeur s'approchent de leur cible.
pub(crate) fn au_plus_pres(milieux: &[f32], l: f32) -> Vec<f32> {
    let pas = l + GOUTTIERE;
    // Des blocs (moyenne, nombre de cases), dont les moyennes croissent.
    let mut blocs: Vec<(f32, usize)> = Vec::new();
    for (k, m) in milieux.iter().enumerate() {
        let mut bloc = (m - k as f32 * pas, 1_usize);
        while let Some(&(moyenne, nombre)) = blocs.last() {
            if moyenne <= bloc.0 {
                break;
            }
            blocs.pop();
            let total = nombre + bloc.1;
            bloc = (
                (moyenne * nombre as f32 + bloc.0 * bloc.1 as f32) / total as f32,
                total,
            );
        }
        blocs.push(bloc);
    }
    let dernier = milieux.len().saturating_sub(1) as f32;
    let (min, max) = (
        MARGE_H + l / 2.0,
        MARGE_H + UTILE_L - l / 2.0 - dernier * pas,
    );
    blocs
        .iter()
        .flat_map(|&(moyenne, nombre)| std::iter::repeat_n(moyenne.max(min).min(max), nombre))
        .enumerate()
        .map(|(k, d)| d + k as f32 * pas - l / 2.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn huit_cases_sombres_remplissent_la_largeur() {
        let milieux: Vec<f32> = (0..8).map(|k| milieu_de_colonne(k.min(6))).collect();
        let x = au_plus_pres(&milieux, largeur_sombre(8));
        for (k, x) in x.iter().enumerate() {
            let attendu = MARGE_H + k as f32 * (CASE_SOMBRE + GOUTTIERE);
            assert!((x - attendu).abs() < 1e-3, "{k} : {x} au lieu de {attendu}");
        }
    }

    #[test]
    fn des_cases_libres_restent_sur_leur_milieu_et_gardent_leur_gouttiere() {
        let l = largeur_sombre(3);
        // Deux cases qui visent le même milieu s'écartent de part et d'autre.
        let x = au_plus_pres(&[milieu_de_colonne(0), 100.0, 100.0], l);
        assert!((x[0] + l / 2.0 - milieu_de_colonne(0)).abs() < 1e-3);
        assert!((x[1] + l / 2.0 + x[2] + l / 2.0 - 200.0).abs() < 1e-3);
        assert!((x[2] - x[1] - (l + GOUTTIERE)).abs() < 1e-3);
        // Une case qui vise hors de la page reste dans la largeur utile.
        let x = au_plus_pres(&[-50.0, 500.0], l);
        assert!(x[0] >= MARGE_H - 1e-3 && x[1] + l <= MARGE_H + UTILE_L + 1e-3);
    }
}
