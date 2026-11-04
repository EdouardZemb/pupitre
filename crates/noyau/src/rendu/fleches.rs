//! Les flèches de résolution de la table : d'où part et où arrive chaque flèche, ce qui ne
//! doit jamais leur arriver, et l'abscisse des noms de rangée, qu'aucune flèche ne traverse.

use super::case::{DEMI_POINTE, coin, sommets_de_pointe};
use super::en_mm;
use super::grille::{MARGE_H, RETRAIT, UTILE_L, x_colonne};
use super::pages::Rect;
use crate::table::{Famille, Position};

/// Le jour entre le nom d'une rangée et la pointe d'une flèche qui passe à côté (2,5 mm
/// jusqu'à son trait). À 1,7 mm, aucune flèche du mineur ne glisse : sa page garde les
/// flèches de l'essai H09.
const JOUR_NOM: f32 = 1.7;
/// La butée d'une flèche : la plus petite distance de sa pointe au côté de sa cible.
const BUTEE: f32 = 4.0;
/// Au-delà de l'arrondi ou de la coupe d'une cible, le jeu qui garde la pointe sur son bord
/// droit (un défaut relevé par H09 : la butée de 4 mm passait sous l'arrondi de 4,36 mm).
const JEU_DU_COIN: f32 = 0.5;
/// La queue d'une flèche quitte le bord droit de sa case sombre, à ce jeu de la coupe.
const JEU_DE_LA_QUEUE: f32 = 2.0;
/// Sous cet écart, une flèche presque droite se redresse : une oblique de 2 mm paraîtrait une
/// erreur.
const REDRESSE: f32 = 2.0;
/// Deux pointes sur une même cible restent à cette distance l'une de l'autre.
const ECART_DES_POINTES: f32 = 3.0;
/// La tolérance du calcul : un sommet posé sur un bord n'entre pas dans la case.
const TOLERANCE: f32 = 1e-3;

/// Une flèche de résolution, et la bande qu'elle franchit entre deux rangées.
#[derive(Clone, Debug)]
pub(crate) struct Trait {
    pub(crate) source: Position,
    pub(crate) cible: Position,
    /// La place de la cible : sa pointe se pose sur son bord, et n'y entre pas.
    pub(crate) rect_cible: Rect,
    pub(crate) queue: (f32, f32),
    pub(crate) pointe: (f32, f32),
    /// Une flèche droite peut glisser entre ces abscisses, sans quitter le bord droit de sa
    /// case ni celui de sa cible ; une flèche oblique ou partie d'une pointe ne glisse pas.
    pub(crate) glisse: Option<(f32, f32)>,
    /// La bande `k` est au-dessus de la rangée `k` de la page.
    pub(crate) bande: usize,
}

impl Trait {
    /// L'étendue horizontale de la flèche, pointe comprise.
    fn etendue(&self) -> (f32, f32) {
        (
            self.queue.0.min(self.pointe.0) - DEMI_POINTE,
            self.queue.0.max(self.pointe.0) + DEMI_POINTE,
        )
    }

    /// La même flèche, droite, glissée en `x`.
    fn glissee(&self, x: f32) -> Trait {
        Trait {
            queue: (x, self.queue.1),
            pointe: (x, self.pointe.1),
            ..self.clone()
        }
    }
}

/// Une case et sa place, d'où part ou où arrive une flèche.
#[derive(Clone, Copy)]
pub(crate) struct Bout {
    pub(crate) position: Position,
    pub(crate) famille: Famille,
    pub(crate) rect: Rect,
}

/// La flèche d'une dominante secondaire : de sa pointe, droit sur sa cible.
pub(crate) fn depuis_une_pointe(source: Bout, cible: Bout, bande: usize) -> Trait {
    let (rs, rc) = (source.rect, cible.rect);
    let x = rs.x + rs.l / 2.0;
    Trait {
        source: source.position,
        cible: cible.position,
        rect_cible: rc,
        queue: (x, rs.y + rs.h),
        pointe: (x, rc.y),
        glisse: None,
        bande,
    }
}

/// La plus petite distance de la pointe au côté de la cible : la butée, et toujours au-delà
/// de l'arrondi ou de la coupe.
fn butee(cible: Bout) -> f32 {
    (coin(cible.famille, cible.rect) + DEMI_POINTE + JEU_DU_COIN).max(BUTEE)
}

/// La flèche d'une case sombre : elle part du milieu de son bord et vise sa cible au plus
/// près, droite quand la cible est en face, oblique sinon.
pub(crate) fn depuis_une_case_sombre(source: Bout, cible: Bout, bande: usize) -> Trait {
    let (rs, rc) = (source.rect, cible.rect);
    let vers_le_bas = rc.y > rs.y;
    let (y_queue, y_pointe) = if vers_le_bas {
        (rs.y + rs.h, rc.y)
    } else {
        (rs.y, rc.y + rc.h)
    };
    let jeu = coin(source.famille, rs) + JEU_DE_LA_QUEUE;
    let (s_lo, s_hi) = (rs.x + jeu, rs.x + rs.l - jeu);
    let b = butee(cible);
    let (c_lo, c_hi) = (rc.x + b, rc.x + rc.l - b);
    let xs = rs.x + rs.l / 2.0;
    let xc = xs.clamp(c_lo, c_hi);
    let droite = (xc - xs).abs() <= REDRESSE && (s_lo..=s_hi).contains(&xc);
    Trait {
        source: source.position,
        cible: cible.position,
        rect_cible: rc,
        queue: (if droite { xc } else { xs }, y_queue),
        pointe: (xc, y_pointe),
        glisse: droite.then_some((s_lo.max(c_lo), s_hi.min(c_hi))),
        bande,
    }
}

/// Ce qui ne doit jamais arriver à un jeu de flèches.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Defaut {
    /// Deux pointes sur une même cible, à moins de 3 mm l'une de l'autre.
    PointesProches {
        a: Position,
        b: Position,
        cible: Position,
        ecart: f32,
    },
    /// Un sommet de la pointe entre dans le cadre de la cible.
    PointeDansLaCible { source: Position, cible: Position },
}

impl Defaut {
    /// Le défaut, en français, chaque case dite par son nom.
    pub(crate) fn decrire(&self, nom: impl Fn(Position) -> String) -> String {
        match self {
            Defaut::PointesProches { a, b, cible, ecart } => format!(
                "les flèches de {} et de {} arrivent sur {} à {} l'une de l'autre, à moins de 3 mm",
                nom(*a),
                nom(*b),
                nom(*cible),
                en_mm(*ecart)
            ),
            Defaut::PointeDansLaCible { source, cible } => format!(
                "la pointe de la flèche de {} entre dans le cadre de {}",
                nom(*source),
                nom(*cible)
            ),
        }
    }
}

/// Vrai si le point est franchement à l'intérieur du rectangle.
fn dedans(r: &Rect, (x, y): (f32, f32)) -> bool {
    x > r.x + TOLERANCE
        && x < r.x + r.l - TOLERANCE
        && y > r.y + TOLERANCE
        && y < r.y + r.h - TOLERANCE
}

/// Le premier défaut du jeu de flèches, s'il y en a un.
pub(crate) fn defaut(traits: &[Trait]) -> Option<Defaut> {
    for t in traits {
        if sommets_de_pointe(t.queue, t.pointe)
            .iter()
            .any(|s| dedans(&t.rect_cible, *s))
        {
            return Some(Defaut::PointeDansLaCible {
                source: t.source,
                cible: t.cible,
            });
        }
    }
    for (i, a) in traits.iter().enumerate() {
        for b in traits[i + 1..].iter().filter(|b| b.cible == a.cible) {
            let ecart =
                ((a.pointe.0 - b.pointe.0).powi(2) + (a.pointe.1 - b.pointe.1).powi(2)).sqrt();
            if ecart < ECART_DES_POINTES {
                return Some(Defaut::PointesProches {
                    a: a.source,
                    b: b.source,
                    cible: a.cible,
                    ecart,
                });
            }
        }
    }
    None
}

/// Les glissements qui libèrent le nom de chaque bande posé en `x`, ou rien si une flèche ne
/// peut s'écarter, ou si un glissement ferait naître un défaut.
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
            // Une abscisse candidate vient d'une butée : la tolérance absorbe l'arrondi.
            let possibles = [
                (a_gauche >= lo - TOLERANCE).then_some(a_gauche.max(lo)),
                (a_droite <= hi + TOLERANCE).then_some(a_droite.min(hi)),
            ];
            let nouveau = possibles
                .into_iter()
                .flatten()
                .min_by(|p, q| (p - t.queue.0).abs().total_cmp(&(q - t.queue.0).abs()))?;
            glissements.push((i, nouveau));
        }
    }
    let mut glisses = traits.to_vec();
    for (i, x) in &glissements {
        glisses[*i] = glisses[*i].glissee(*x);
    }
    defaut(&glisses).is_none().then_some(glissements)
}

/// L'abscisse commune des noms de rangée : celle, de gauche à droite, qui demande le moins
/// de flèches à pousser. Une colonne d'abord, comme dans l'essai H09 ; sinon, juste à droite
/// d'une flèche droite poussée au bord de sa case. Les flèches poussées glissent.
pub(crate) fn placer_les_noms(largeurs: &[f32], traits: &mut [Trait]) -> Option<f32> {
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
        traits[i] = traits[i].glissee(nouveau);
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Rangee;

    fn pos(rangee: Rangee, rang: u8) -> Position {
        Position { rangee, rang }
    }

    const CIBLE: Rect = Rect {
        x: 10.0,
        y: 0.0,
        l: 20.0,
        h: 10.0,
    };

    fn trait_vers_la_cible(rang: u8, queue: (f32, f32), x_pointe: f32) -> Trait {
        Trait {
            source: pos(Rangee::Sombre, rang),
            cible: pos(Rangee::Centre, 0),
            rect_cible: CIBLE,
            queue,
            pointe: (x_pointe, CIBLE.y + CIBLE.h),
            glisse: None,
            bande: 1,
        }
    }

    fn nom(p: Position) -> String {
        format!("{:?} {}", p.rangee, p.rang)
    }

    #[test]
    fn deux_pointes_trop_proches_se_disent() {
        let traits = [
            trait_vers_la_cible(0, (18.0, 16.0), 18.0),
            trait_vers_la_cible(1, (20.0, 16.0), 20.0),
        ];
        let d = defaut(&traits).expect("deux pointes à 2 mm");
        assert_eq!(
            d.decrire(nom),
            "les flèches de Sombre 0 et de Sombre 1 arrivent sur Centre 0 à 2,0 mm l'une de \
             l'autre, à moins de 3 mm"
        );
        let traits = [
            trait_vers_la_cible(0, (16.0, 16.0), 16.0),
            trait_vers_la_cible(1, (20.0, 16.0), 20.0),
        ];
        assert_eq!(defaut(&traits), None, "à 4 mm, elles restent à part");
    }

    #[test]
    fn une_pointe_qui_entre_dans_sa_cible_se_dit() {
        // Une oblique plate : un coin de la base de sa pointe passe le bord.
        let plate = [trait_vers_la_cible(0, (45.0, 16.0), 26.0)];
        let d = defaut(&plate).expect("une pointe dans sa cible");
        assert_eq!(
            d.decrire(nom),
            "la pointe de la flèche de Sombre 0 entre dans le cadre de Centre 0"
        );
        let droite = [trait_vers_la_cible(0, (26.0, 16.0), 26.0)];
        assert_eq!(defaut(&droite), None);
    }
}
