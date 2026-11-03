//! Le verso de la table (REN-01, MIC-13) : pour les sept accords du centre, dans leur ordre,
//! une colonne par accord, alignée sur les colonnes de la table. Et la fiche d'un accord : une
//! colonne seule, pour l'établi.
//!
//! Une colonne porte, de haut en bas, dans l'ordre qu'H09 a validé : le nom et le chiffre de
//! l'accord ; avec la guitare, le diagramme de manche ; avec le clavier, le diagramme de clavier ;
//! la bande de la légende de la palette, que la page trace sur toute sa largeur ; la palette sur le
//! manche, puis sur le clavier, puis sur la portée, qui figure toujours. Sur une page, chaque bloc
//! commence à la même hauteur dans toutes les colonnes, et prend la place du plus haut.

use std::slice;

use super::case::{Support, marque};
use super::grille::{
    COLONNE, LIGNES_D_EN_TETE, MARGE_H, MARGE_V, RETRAIT, SOUS_FILET, UTILE_H, UTILE_L,
    composer_l_en_tete, corps_nom, hauteur_du_pied, interligne, jusqua_la_base, mi_capitale,
    x_colonne,
};
use super::manche::{LARGEUR_MANCHE, POINT};
use super::pages::{HAUTEUR, LARGEUR, Rect};
use super::svg::{Couleur, Element, Page, TRAIT_FIN};
use super::texte::{Corps, chasse, composer};
use super::{Contexte, Debordement, Instrument, clavier, en_mm, manche, portee};
use crate::accord::Accord;
use crate::clavier::Voicing;
use crate::guitare::{self, Doigte, NotesVoulues};
use crate::hauteur::NomDeNote;
use crate::palette::{NoteDePalette, Role, palette};
use crate::table::{CoucheSombre, Rangee, Table, table};
use crate::tonalite::Tonalite;

/// Le blanc entre deux blocs d'une colonne. L'essai H09 en laissait 4,5 mm, avec un en-tête et un
/// pied d'une ligne ; l'en-tête et le pied réservent ici deux lignes chacun, et la palette sur le
/// manche prend jusqu'à huit cases hors de la mineur.
const ENTRE_BLOCS: f32 = 2.5;
/// Le blanc entre le nom de l'accord et le premier diagramme.
const APRES_LE_NOM: f32 = 1.0;
/// L'écart entre le nom d'un accord et son chiffre.
const ECART_CHIFFRE: f32 = 1.5;
/// La précision des comparaisons de longueur.
const EPS: f32 = 1e-3;

// --- Ce qu'une colonne dessine -------------------------------------------------

/// Un accord, tel qu'une colonne le dessine : hors contexte, comme dans la table.
struct Contenu {
    nom: String,
    chiffre: Option<String>,
    doigte: Option<Doigte>,
    /// Les notes, écrites comme le nom, la fondamentale d'abord.
    notes: Vec<NomDeNote>,
    voicing: Voicing,
    basse: NomDeNote,
    palette: Vec<NoteDePalette>,
}

impl Contenu {
    fn de(a: &Accord, t: Tonalite, chiffre: Option<String>) -> Contenu {
        let notes = a.noms_des_notes(t);
        Contenu {
            nom: a.nom(t),
            chiffre,
            doigte: guitare::hors_contexte(&NotesVoulues::de(a, t)),
            basse: notes[usize::from(a.renversement)],
            notes,
            voicing: crate::clavier::hors_contexte(a, t),
            palette: palette(a, t),
        }
    }

    /// Le nom à 14 pt et le chiffre à 11 pt, côte à côte : leur largeur.
    fn largeur_du_titre(&self) -> f32 {
        let l = chasse(&self.nom, &corps_nom());
        self.chiffre
            .as_ref()
            .map_or(l, |ch| l + ECART_CHIFFRE + chasse(ch, &Corps::MIN))
    }
}

/// Le chiffre de l'accord dans la table de la tonalité, couche sombre affichée : chaque accord
/// y a une case, une seule.
fn chiffre_de(a: &Accord, t: Tonalite) -> Option<String> {
    table(t, CoucheSombre::Affichee)
        .cases
        .into_iter()
        .find(|c| c.accord == *a)
        .map(|c| c.chiffre)
}

// --- Le gabarit : les hauteurs communes aux colonnes d'une page ------------------

struct Gabarit {
    guitare: bool,
    clavier: bool,
    /// Vrai si le titre d'une colonne au moins se coupe, le chiffre sous le nom.
    titre_sur_deux_lignes: bool,
    /// La hauteur du plus haut diagramme de manche.
    manche: f32,
    /// La hauteur de la plus haute palette sur le manche.
    palette_manche: f32,
    /// Les lignes de noms sous la portée : le plus grand nombre qu'une colonne demande.
    lignes_de_noms: usize,
    /// Les lignes de la bande de la légende.
    lignes_de_legende: usize,
}

/// Le haut de chaque bloc, sous le haut de la colonne, et la hauteur de la colonne.
struct Places {
    manche: f32,
    clavier: f32,
    legende: f32,
    palette_manche: f32,
    palette_clavier: f32,
    portee: f32,
    hauteur: f32,
}

impl Gabarit {
    fn de(
        contenus: &[Contenu],
        i: Instrument,
        largeur: f32,
        lignes_de_legende: usize,
        contexte: &Contexte,
    ) -> Result<Gabarit, Debordement> {
        let mut g = Gabarit {
            guitare: matches!(i, Instrument::Guitare | Instrument::LesDeux),
            clavier: matches!(i, Instrument::Clavier | Instrument::LesDeux),
            titre_sur_deux_lignes: false,
            manche: 0.0,
            palette_manche: 0.0,
            lignes_de_noms: 2,
            lignes_de_legende,
        };
        let place = largeur - 2.0 * RETRAIT;
        for c in contenus {
            g.titre_sur_deux_lignes |= c.largeur_du_titre() > place;
            let h = manche::hauteur_du_diagramme(c.doigte.as_ref(), &c.notes, place)
                .map_err(|e| contexte.trop_large(&e.mot, e.chasse, e.largeur))?;
            g.manche = g.manche.max(h);
            if let Some(d) = &c.doigte {
                g.palette_manche = g.palette_manche.max(manche::hauteur_de_la_palette(d));
            }
            let lignes = portee::lignes_de_noms(&c.palette, place).ok_or_else(|| {
                contexte.deborde(format_args!(
                    "les noms de la palette de {} ne tiennent pas sous sa portée",
                    c.nom
                ))
            })?;
            g.lignes_de_noms = g.lignes_de_noms.max(lignes);
        }
        Ok(g)
    }

    fn places(&self) -> Places {
        let il = interligne(&Corps::MIN);
        let mut y = interligne(&corps_nom()) + APRES_LE_NOM;
        if self.titre_sur_deux_lignes {
            y += il;
        }
        let manche = y;
        if self.guitare {
            y += self.manche + ENTRE_BLOCS;
        }
        let clavier = y;
        if self.clavier {
            y += clavier::hauteur_du_diagramme() + ENTRE_BLOCS;
        }
        let legende = y;
        y += SOUS_FILET + self.lignes_de_legende as f32 * il + SOUS_FILET;
        let palette_manche = y;
        if self.guitare {
            y += self.palette_manche + ENTRE_BLOCS;
        }
        let palette_clavier = y;
        if self.clavier {
            y += clavier::PALETTE_H + ENTRE_BLOCS;
        }
        let portee = y;
        y += portee::hauteur_de_portee(self.lignes_de_noms);
        Places {
            manche,
            clavier,
            legende,
            palette_manche,
            palette_clavier,
            portee,
            hauteur: y,
        }
    }
}

// --- La colonne ----------------------------------------------------------------

/// L'abscisse de la corde grave : le manche centré dans sa colonne, poussé vers la droite si le
/// numéro de sa frette de départ ne tient pas à sa gauche.
fn x_du_manche(r: &Rect, debord: f32, nom: &str, contexte: &Contexte) -> Result<f32, Debordement> {
    let x = (r.x + (r.l - LARGEUR_MANCHE) / 2.0).max(r.x + debord);
    let largeur = x - r.x + LARGEUR_MANCHE + POINT;
    if largeur > r.l + EPS {
        return Err(contexte.deborde(format_args!(
            "le manche de {nom} mesure {}, sa place {}",
            en_mm(largeur),
            en_mm(r.l)
        )));
    }
    Ok(x)
}

/// L'abscisse d'un clavier de `largeur` mm, centré dans sa colonne.
fn x_du_clavier(
    r: &Rect,
    largeur: f32,
    quoi: &str,
    contexte: &Contexte,
) -> Result<f32, Debordement> {
    if largeur > r.l + EPS {
        return Err(contexte.deborde(format_args!(
            "{quoi} mesure {}, sa place {}",
            en_mm(largeur),
            en_mm(r.l)
        )));
    }
    Ok(r.x + (r.l - largeur) / 2.0)
}

/// La colonne d'un accord dans `r`, aux hauteurs du gabarit.
fn colonne(c: &Contenu, r: Rect, g: &Gabarit, contexte: &Contexte) -> Result<Element, Debordement> {
    let (t, corps_du_nom) = (Corps::MIN, corps_nom());
    let places = g.places();
    if places.hauteur > r.h + EPS {
        return Err(contexte.deborde(format_args!(
            "la colonne de {} mesure {} de haut, pour {}",
            c.nom,
            en_mm(places.hauteur),
            en_mm(r.h)
        )));
    }
    let cx = r.x + r.l / 2.0;
    let place = r.l - 2.0 * RETRAIT;
    let mut e = Vec::new();

    // Le nom à 14 pt et le chiffre à 11 pt, centrés ensemble ; le chiffre sous le nom s'ils ne
    // tiennent pas côte à côte.
    let base = r.y + jusqua_la_base(&corps_du_nom);
    let largeur = c.largeur_du_titre();
    if largeur <= place {
        e.push(composer(&c.nom, &corps_du_nom, cx - largeur / 2.0, base));
        if let Some(chiffre) = &c.chiffre {
            let x = cx + largeur / 2.0 - chasse(chiffre, &t);
            e.push(composer(chiffre, &t, x, base));
        }
    } else {
        let base_du_chiffre = r.y + interligne(&corps_du_nom) + jusqua_la_base(&t);
        let lignes = [(&c.nom, &corps_du_nom, base)]
            .into_iter()
            .chain(c.chiffre.iter().map(|ch| (ch, &t, base_du_chiffre)));
        for (texte, corps, base) in lignes {
            let l = chasse(texte, corps);
            if l > place {
                return Err(contexte.trop_large(texte, l, place));
            }
            e.push(composer(texte, corps, cx - l / 2.0, base));
        }
    }

    if g.guitare {
        let debord = manche::debord_du_diagramme(c.doigte.as_ref());
        let x = x_du_manche(&r, debord, &c.nom, contexte)?;
        let origine = (x, r.y + places.manche);
        let diagramme = manche::diagramme(c.doigte.as_ref(), &c.notes, origine, place)
            .map_err(|e| contexte.trop_large(&e.mot, e.chasse, e.largeur))?;
        e.push(diagramme);
        // Sans doigté, la palette n'a pas de zone sur le manche.
        if let Some(d) = &c.doigte {
            let x = x_du_manche(&r, manche::debord_de_la_palette(d), &c.nom, contexte)?;
            let origine = (x, r.y + places.palette_manche);
            e.push(manche::palette_sur_le_manche(d, &c.palette, origine));
        }
    }
    if g.clavier {
        let quoi = format!("le clavier de {}", c.nom);
        let l = clavier::largeur_du_diagramme(&c.voicing);
        let origine = (x_du_clavier(&r, l, &quoi, contexte)?, r.y + places.clavier);
        e.push(clavier::diagramme_de_clavier(&c.voicing, c.basse, origine));
        let quoi = format!("la palette sur le clavier de {}", c.nom);
        let l = clavier::largeur_de_la_palette(&c.palette);
        let origine = (
            x_du_clavier(&r, l, &quoi, contexte)?,
            r.y + places.palette_clavier,
        );
        e.push(clavier::palette_sur_le_clavier(&c.palette, origine));
    }
    if !portee::tiennent(&c.palette, place, g.lignes_de_noms) {
        return Err(contexte.deborde(format_args!(
            "les noms de la palette de {} ne tiennent pas sur {} lignes",
            c.nom, g.lignes_de_noms
        )));
    }
    let origine = (r.x + RETRAIT, r.y + places.portee);
    e.push(portee::portee(&c.palette, origine, place, g.lignes_de_noms));

    Ok(Element::Groupe {
        id: Some("colonne"),
        translation: (0.0, 0.0),
        echelle: None,
        enfants: e,
    })
}

/// La colonne d'un accord dans `r` : son nom et son chiffre dans la tonalité (le chiffre sous
/// le nom s'ils ne tiennent pas côte à côte), ses diagrammes selon l'instrument, et ses
/// palettes. La bande de la légende y est réservée, sous les diagrammes ; la page qui accueille
/// la colonne la trace. Rend `Debordement` si la colonne ne tient pas dans `r`.
pub fn colonne_d_accord(
    a: &Accord,
    t: Tonalite,
    i: Instrument,
    r: Rect,
) -> Result<Element, Debordement> {
    let contexte = Contexte(format!("{} en {}", a.nom(t), t.nom()));
    let c = Contenu::de(a, t, chiffre_de(a, t));
    let g = Gabarit::de(slice::from_ref(&c), i, r.l, 1, &contexte)?;
    colonne(&c, r, &g, &contexte)
}

// --- La légende et l'en-tête ------------------------------------------------------

const TITRE_DE_LEGENDE: &str = "palette :";
const LEGENDE: [(Role, &str); 3] = [
    (Role::Appui, "appui"),
    (Role::Tension, "tension"),
    (Role::Frotte, "frotte"),
];
const APRES_LE_TITRE: f32 = 3.0;
/// La place d'une marque devant son mot.
const DEVANT_LE_MOT: f32 = 3.2;
const ENTRE_LES_ROLES: f32 = 5.0;

fn largeur_de_legende() -> f32 {
    let t = Corps::MIN;
    let mots: f32 = LEGENDE.iter().map(|(_, mot)| chasse(mot, &t)).sum();
    chasse(TITRE_DE_LEGENDE, &t)
        + APRES_LE_TITRE
        + LEGENDE.len() as f32 * DEVANT_LE_MOT
        + mots
        + (LEGENDE.len() - 1) as f32 * ENTRE_LES_ROLES
}

/// Les lignes de la légende dans `largeur` mm : une seule si elle tient, sinon le titre, puis
/// une ligne par rôle.
fn lignes_de_legende(largeur: f32) -> usize {
    if largeur_de_legende() <= largeur {
        1
    } else {
        1 + LEGENDE.len()
    }
}

/// La légende de la palette, sous un filet de `largeur` mm, dès `haut` : ce qui sépare les
/// diagrammes des palettes, et dit leurs formes. Sur une ligne si elle tient ; sinon le titre,
/// puis une ligne par rôle, sa marque devant son mot.
fn legende(haut: f32, largeur: f32, contexte: &Contexte) -> Result<Vec<Element>, Debordement> {
    let t = Corps::MIN;
    let en_une_ligne = lignes_de_legende(largeur) == 1;
    let plus_large = if en_une_ligne {
        largeur_de_legende()
    } else {
        LEGENDE
            .iter()
            .map(|(_, mot)| RETRAIT + DEVANT_LE_MOT + chasse(mot, &t))
            .fold(chasse(TITRE_DE_LEGENDE, &t), f32::max)
    };
    if plus_large > largeur + EPS {
        return Err(contexte.deborde(format_args!(
            "la légende de la palette mesure {}, pour {} utiles",
            en_mm(plus_large),
            en_mm(largeur)
        )));
    }
    let base = haut + SOUS_FILET + jusqua_la_base(&t);
    let mut e = vec![Element::Ligne {
        de: (MARGE_H, haut),
        a: (MARGE_H + largeur, haut),
        trait_: (Couleur::Noir, TRAIT_FIN),
    }];
    let mut x = MARGE_H;
    e.push(composer(TITRE_DE_LEGENDE, &t, x, base));
    if en_une_ligne {
        x += chasse(TITRE_DE_LEGENDE, &t) + APRES_LE_TITRE;
    }
    for (k, (role, mot)) in LEGENDE.into_iter().enumerate() {
        let base = if en_une_ligne {
            base
        } else {
            x = MARGE_H + RETRAIT;
            base + (k + 1) as f32 * interligne(&t)
        };
        e.push(marque(
            role,
            (x + 1.0, base - mi_capitale(&t)),
            Support::Manche,
        ));
        x += DEVANT_LE_MOT;
        e.push(composer(mot, &t, x, base));
        x += chasse(mot, &t) + ENTRE_LES_ROLES;
    }
    Ok(e)
}

// --- Les pages -------------------------------------------------------------------

/// Le verso de la table : pour les sept accords du centre, dans leur ordre, une colonne
/// alignée sur celle de leur case. L'en-tête réserve ses deux lignes et le pied est réservé au
/// bas de la page, comme sur la page de la table ; le cahier le pose. Rend `Debordement` si une
/// colonne ne tient pas dans les 145 mm utiles, pied réservé.
pub fn page_verso(table: &Table, i: Instrument) -> Result<Page, Debordement> {
    let ton = table.tonalite;
    let contexte = Contexte(format!("le verso de {}", ton.nom()));
    let texte = format!(
        "{} · verso de la table · diagrammes et palettes des accords du centre",
        ton.nom()
    );
    let (mut elements, y_filet) = composer_l_en_tete(
        &texte,
        UTILE_L,
        LIGNES_D_EN_TETE,
        LIGNES_D_EN_TETE,
        &contexte,
    )?;
    let haut = y_filet + SOUS_FILET;
    let bas = MARGE_V + UTILE_H - hauteur_du_pied();
    let cases = table.rangee(Rangee::Centre);
    let contenus: Vec<Contenu> = cases
        .iter()
        .map(|c| Contenu::de(&c.accord, ton, Some(c.chiffre.clone())))
        .collect();
    let g = Gabarit::de(&contenus, i, COLONNE, 1, &contexte)?;
    let places = g.places();
    if haut + places.hauteur > bas + EPS {
        return Err(contexte.deborde(format_args!(
            "le verso mesure {} de haut, pour {} utiles",
            en_mm(haut + places.hauteur - MARGE_V + hauteur_du_pied()),
            en_mm(UTILE_H)
        )));
    }
    for (case, c) in cases.iter().zip(&contenus) {
        let r = Rect {
            x: x_colonne(usize::from(case.position.rang)),
            y: haut,
            l: COLONNE,
            h: bas - haut,
        };
        elements.push(colonne(c, r, &g, &contexte)?);
    }
    elements.extend(legende(haut + places.legende, UTILE_L, &contexte)?);
    Ok(Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements,
    })
}

/// La fiche d'un accord, pour l'établi : « V7/ii en do majeur · instrument : guitare et
/// clavier », le chiffre en tonalité puis l'instrument, puis la colonne de l'accord, dont le
/// titre dit le nom et le chiffre, « A7 V7/ii », et la légende de sa palette. L'en-tête ne
/// répète pas le nom. La page a la largeur d'une colonne de la table et de ses marges, pour
/// s'afficher à l'échelle des colonnes ; elle s'arrête au bas de la colonne. L'en-tête se coupe
/// aux « · », puis aux espaces, dans la largeur de la colonne. La fiche n'a pas de pied. Un
/// accord hors de la table n'a pas de chiffre : l'en-tête dit « en do majeur · instrument : … ».
pub fn fiche_d_accord(a: &Accord, t: Tonalite, i: Instrument) -> Result<Page, Debordement> {
    let nom = a.nom(t);
    let chiffre = chiffre_de(a, t);
    let contexte = Contexte(format!("la fiche de {nom} en {}", t.nom()));
    let texte = match &chiffre {
        Some(ch) => format!("{ch} en {} · instrument : {}", t.nom(), i.nom()),
        None => format!("en {} · instrument : {}", t.nom(), i.nom()),
    };
    let (mut elements, y_filet) = composer_l_en_tete(&texte, COLONNE, 1, usize::MAX, &contexte)?;
    let haut = y_filet + SOUS_FILET;
    let c = Contenu::de(a, t, chiffre);
    let g = Gabarit::de(
        slice::from_ref(&c),
        i,
        COLONNE,
        lignes_de_legende(COLONNE),
        &contexte,
    )?;
    let places = g.places();
    let r = Rect {
        x: x_colonne(0),
        y: haut,
        l: COLONNE,
        h: places.hauteur,
    };
    elements.push(colonne(&c, r, &g, &contexte)?);
    elements.extend(legende(haut + places.legende, COLONNE, &contexte)?);
    Ok(Page {
        largeur: 2.0 * MARGE_H + COLONNE,
        hauteur: haut + places.hauteur + MARGE_V,
        elements,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_accord_a_une_seule_case_dans_sa_table() {
        // `chiffre_de` cherche la case d'un accord : il n'y en a jamais deux.
        for t in Tonalite::les_24() {
            let tb = table(t, CoucheSombre::Affichee);
            for (k, a) in tb.cases.iter().enumerate() {
                for b in &tb.cases[k + 1..] {
                    assert_ne!(a.accord, b.accord, "{} : {}", t.nom(), tb.nom(a));
                }
            }
        }
    }
}
