//! La fenêtre d'une liste : les lignes à demander selon son défilement et sa hauteur vue,
//! l'étalon qui mesure ses lignes, et la ligne où entre le focus venu du dehors.

use commandes::{FENETRE_MAX, Fenetre, LigneDeLaBoite};
use leptos::html;
use leptos::prelude::*;

/// La marge de la fenêtre, en lignes, de chaque côté de ce qui se voit.
const MARGE: u32 = 5;

/// Les lignes que mesure chaque étalon de la feuille de style (`Etalon`),
/// `calc(100 * var(--ligne))` ou `calc(100 * var(--ligne-du-journal))` : sa hauteur, en pixels
/// entiers, donne celle d'une ligne à un centième de pixel près.
pub(super) const LIGNES_DE_L_ETALON: f64 = 100.0;

/// Un étalon de la feuille de style, hors du flux et sans place : il mesure cent lignes d'une
/// liste en fenêtre (`LIGNES_DE_L_ETALON`). Celui de la boîte mesure les lignes de la boîte d'idées
/// et des idées retirées, de hauteur `--ligne` ; celui du journal, dans son volet, les lignes du
/// journal du fonds, de hauteur `--ligne-du-journal`, qui tiennent trois lignes de texte. Chaque
/// liste se mesure ainsi sur la hauteur de ses lignes, et sa piste se pose sur la même.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Etalon {
    Boite,
    Journal,
}

impl Etalon {
    /// La propriété de la feuille de style qui porte la hauteur fixe d'une ligne que l'étalon
    /// mesure.
    fn hauteur_d_une_ligne(self) -> &'static str {
        match self {
            Etalon::Boite => "--ligne",
            Etalon::Journal => "--ligne-du-journal",
        }
    }

    /// La hauteur de `lignes` lignes que l'étalon mesure, en CSS : celle de la piste d'une liste,
    /// ou le haut de ses lignes rendues.
    pub(super) fn hauteur(self, lignes: u64) -> String {
        format!("calc({lignes} * var({}))", self.hauteur_d_une_ligne())
    }
}

/// `n` en `u32`, `u32::MAX` s'il le dépasse.
fn en_u32(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Vrai si une hauteur de ligne se lit : un nombre plus grand que zéro, ni nul, ni négatif, ni NaN.
fn lisible(hauteur_ligne_px: f64) -> bool {
    hauteur_ligne_px > 0.0
}

/// La fenêtre à demander : la première ligne vue, `floor(defilement / hauteur_ligne)`, moins
/// `MARGE` ; les lignes vues, `ceil(hauteur_vue / hauteur_ligne)`, plus `MARGE` après ; bornée par
/// `total` et par `FENETRE_MAX`. Un défilement ou une hauteur vue négatifs comptent pour zéro ;
/// sans hauteur de ligne lisible, les `FENETRE_MAX` premières lignes, toutes s'il y en a moins.
fn fenetre(defilement_px: f64, hauteur_vue_px: f64, hauteur_ligne_px: f64, total: u64) -> Fenetre {
    let plafond = u64::from(FENETRE_MAX);
    if !lisible(hauteur_ligne_px) {
        return Fenetre {
            debut: 0,
            nombre: en_u32(total.min(plafond)),
        };
    }
    // `as` sature : un quotient trop grand donne `u64::MAX`.
    let premiere = (defilement_px.max(0.0) / hauteur_ligne_px).floor() as u64;
    let vues = (hauteur_vue_px.max(0.0) / hauteur_ligne_px).ceil() as u64;
    let marge = u64::from(MARGE);
    let debut = premiere.saturating_sub(marge).min(total);
    let fin = premiere
        .saturating_add(vues)
        .saturating_add(marge)
        .min(total);
    Fenetre {
        debut: en_u32(debut),
        nombre: en_u32(fin.saturating_sub(debut).min(plafond)),
    }
}

/// Vrai si la fenêtre `demandee` porte toutes les lignes qui se voient, même en partie, et une
/// ligne au-delà de chaque bord vu, bornées par 0 et par `total` : le défilement n'a pas à en
/// demander une autre. La ligne au-delà fait partir la demande avant que le focus, qui descend de
/// ligne en ligne, n'atteigne la dernière ligne rendue. Sans hauteur de ligne lisible, rien n'est
/// couvert.
fn couvre(
    demandee: &Fenetre,
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
    total: u64,
) -> bool {
    if !lisible(hauteur_ligne_px) {
        return false;
    }
    let haut = defilement_px.max(0.0);
    let premiere = ((haut / hauteur_ligne_px).floor() as u64)
        .saturating_sub(1)
        .min(total);
    let fin = (((haut + hauteur_vue_px.max(0.0)) / hauteur_ligne_px).ceil() as u64)
        .saturating_add(1)
        .min(total);
    let debut = u64::from(demandee.debut);
    debut <= premiere && fin <= debut + u64::from(demandee.nombre)
}

/// La fenêtre que demande une liste défilée de `defilement_px`, vue sur `hauteur_vue_px`, aux
/// lignes de `hauteur_ligne_px` : celle de son défilement si `forcer`, ou si `derniere`, la
/// dernière demandée, ne porte pas toutes les lignes vues parmi les `total` lues (`couvre`) ;
/// sinon aucune. Elle n'est jamais bornée par le total lu, qu'un geste peut faire croître : la
/// relecture d'après le geste, qui reprend la dernière fenêtre demandée, porte ainsi jusqu'à
/// `MARGE` lignes nouvelles au bout de la liste.
pub(super) fn fenetre_a_demander(
    derniere: Option<&Fenetre>,
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
    total: Option<u64>,
    forcer: bool,
) -> Option<Fenetre> {
    let total = total.unwrap_or(u64::MAX);
    let couverte =
        derniere.is_some_and(|d| couvre(d, defilement_px, hauteur_vue_px, hauteur_ligne_px, total));
    (forcer || !couverte)
        .then(|| fenetre(defilement_px, hauteur_vue_px, hauteur_ligne_px, u64::MAX))
}

/// Le bord d'une liste par où le focus y entre, venu du dehors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Bord {
    Haut,
    Bas,
}

/// La ligne qui prend le focus venu du dehors d'une liste sur sa ligne de rang `rang`, la liste
/// défilée de `defilement_px` avant que `focus()` ne la fasse défiler, vue sur `hauteur_vue_px`,
/// aux lignes de `hauteur_ligne_px`. Tab atteint d'abord la première ligne de la fenêtre lue, et
/// Maj+Tab la dernière, chacune cachée dans sa marge, que `focus()` centrerait : une ligne
/// entièrement cachée au-dessus de la vue cède le focus à la première ligne entièrement vue,
/// `(Bord::Haut, rang)`, une ligne entièrement cachée au-dessous, à la dernière,
/// `(Bord::Bas, rang)`. Une ligne vue, même en partie, le garde ; sans ligne entièrement vue, ou
/// sans hauteur de ligne lisible, rien ne change.
pub(super) fn ligne_d_entree(
    rang: u64,
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
) -> Option<(Bord, u64)> {
    if !lisible(hauteur_ligne_px) {
        return None;
    }
    let haut = defilement_px.max(0.0);
    let bas = haut + hauteur_vue_px.max(0.0);
    // Le haut de la ligne de rang `rang`, en pixels.
    let haut_de = |rang: u64| rang as f64 * hauteur_ligne_px;
    let (bord, cible) = if haut_de(rang.saturating_add(1)) <= haut {
        (Bord::Haut, (haut / hauteur_ligne_px).ceil() as u64)
    } else if haut_de(rang) >= bas {
        (
            Bord::Bas,
            ((bas / hauteur_ligne_px).floor() as u64).checked_sub(1)?,
        )
    } else {
        return None;
    };
    (haut_de(cible) >= haut && haut_de(cible.saturating_add(1)) <= bas).then_some((bord, cible))
}

/// L'idée de la ligne de rang `rang` dans sa liste, si la fenêtre lue `fenetre` la porte parmi ses
/// lignes `lignes`.
pub(super) fn idee_de_rang(
    fenetre: &Fenetre,
    lignes: &[LigneDeLaBoite],
    rang: u64,
) -> Option<String> {
    let place = usize::try_from(rang.checked_sub(u64::from(fenetre.debut))?).ok()?;
    lignes.get(place).map(|ligne| ligne.idee.clone())
}

/// Le défilement de la liste montée `liste`, sa hauteur vue, et la hauteur d'une ligne, que donne
/// l'étalon monté `etalon`, celui qui mesure les lignes de cette liste, en pixels ; rien si l'une
/// ou l'autre n'est pas montée.
pub(super) fn mesures(
    liste: NodeRef<html::Div>,
    etalon: NodeRef<html::Div>,
) -> Option<(f64, f64, f64)> {
    let liste = liste.get_untracked()?;
    let etalon = etalon.get_untracked()?;
    Some((
        f64::from(liste.scroll_top()),
        f64::from(liste.client_height()),
        f64::from(etalon.client_height()) / LIGNES_DE_L_ETALON,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boite::tests::ligne;

    #[test]
    fn la_fenetre_borne_les_lignes_rendues() {
        // Deux mille idées, des lignes de 40 px, une vue de 800 px : vingt lignes vues.
        let a = |defilement_px| fenetre(defilement_px, 800.0, 40.0, 2_000);
        assert_eq!(
            a(0.0),
            Fenetre {
                debut: 0,
                nombre: 25
            }
        );
        assert_eq!(
            a(40_000.0),
            Fenetre {
                debut: 995,
                nombre: 30
            }
        );
        assert_eq!(
            a(79_200.0),
            Fenetre {
                debut: 1_975,
                nombre: 25
            }
        );
        assert_eq!(a(200_000.0).nombre, 0);
        // Des quotients qui ne tombent pas juste : la première ligne vue s'arrondit vers le bas, et
        // le nombre de lignes vues vers le haut.
        assert_eq!(
            fenetre(40_020.0, 790.0, 40.0, 2_000),
            Fenetre {
                debut: 995,
                nombre: 30
            }
        );
        // Jamais plus de trente lignes : les vingt vues, plus deux marges.
        for defilement_px in (0..=80_000).step_by(7) {
            let rendues = a(f64::from(defilement_px)).nombre;
            assert!(
                rendues <= 20 + 2 * MARGE,
                "{rendues} lignes à {defilement_px} px"
            );
        }
        // Une vue plus haute que `FENETRE_MAX` lignes : la fenêtre s'y borne.
        assert_eq!(
            fenetre(0.0, 1_000_000.0, 40.0, 1_000_000).nombre,
            FENETRE_MAX
        );
        // Sans hauteur de ligne lisible : les `FENETRE_MAX` premières lignes, toutes s'il y en a
        // moins.
        assert_eq!(
            fenetre(40_000.0, 800.0, 0.0, 2_000),
            Fenetre {
                debut: 0,
                nombre: FENETRE_MAX
            }
        );
        assert_eq!(
            fenetre(0.0, 800.0, f64::NAN, 12),
            Fenetre {
                debut: 0,
                nombre: 12
            }
        );
    }

    #[test]
    fn la_fenetre_demandee_couvre_les_lignes_vues() {
        let demandee = fenetre(40_000.0, 800.0, 40.0, 2_000);
        let couverte = |defilement_px| couvre(&demandee, defilement_px, 800.0, 40.0, 2_000);
        // La fenêtre de 995 à 1 025 porte les lignes vues, de 1 000 à 1 019, et une ligne au-delà
        // de chaque bord, 999 et 1 020.
        assert!(couverte(40_000.0));
        // Quatre lignes plus bas, de 1 004 à 1 023 : la ligne au-delà, 1 024, est dans la fenêtre.
        assert!(couverte(40_160.0));
        // Cinq lignes plus bas, de 1 005 à 1 024 : la ligne au-delà, 1 025, n'y est pas, et le
        // défilement demande une autre fenêtre avant que Tab n'atteigne la dernière ligne rendue.
        assert!(!couverte(40_200.0));
        assert!(!couverte(40_220.0));
        // Quatre lignes plus haut, de 996 à 1 015, la ligne 995 au-delà ; cinq lignes plus haut, la
        // ligne au-delà, 994, manque.
        assert!(couverte(39_840.0));
        assert!(!couverte(39_800.0));
        assert!(!couverte(39_760.0));
        // Au bout de la liste, les lignes vues s'arrêtent à la dernière.
        let au_bout = fenetre(79_200.0, 800.0, 40.0, 2_000);
        assert!(couvre(&au_bout, 79_200.0, 800.0, 40.0, 2_000));
        // Sans hauteur de ligne lisible, rien n'est couvert.
        assert!(!couvre(&demandee, 40_000.0, 800.0, 0.0, 2_000));
    }

    #[test]
    fn la_fenetre_demandee_au_bout_porte_les_idees_qu_un_geste_ajoute() {
        // Deux mille idées lues, la liste défilée jusqu'au bout : la fenêtre demandée va cinq
        // lignes au-delà de la dernière, que la relecture d'après une note reprend et montre.
        let au_bout = Fenetre {
            debut: 1_975,
            nombre: 30,
        };
        assert_eq!(
            fenetre_a_demander(None, 79_200.0, 800.0, 40.0, Some(2_000), false),
            Some(au_bout.clone())
        );
        // Une idée notée, la même fenêtre porte encore les lignes vues : rien ne se redemande.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 79_240.0, 800.0, 40.0, Some(2_001), false),
            None
        );
        // Forcée, elle se redemande.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 79_200.0, 800.0, 40.0, Some(2_000), true),
            Some(au_bout.clone())
        );
        // Revenue au début, la liste ne voit plus ses lignes dans la dernière fenêtre : une autre
        // se demande.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 0.0, 800.0, 40.0, Some(2_000), false),
            Some(Fenetre {
                debut: 0,
                nombre: 25
            })
        );
        // Sans total lu, rien ne borne les lignes vues.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 79_200.0, 800.0, 40.0, None, false),
            None
        );
    }

    #[test]
    fn l_etalon_de_la_feuille_de_style_mesure_cent_lignes() {
        let css = include_str!("../../style.css");
        assert!(
            css.contains("--ligne: 2.5rem;"),
            "la hauteur d'une ligne manque à la feuille de style"
        );
        let etalon = format!("height: calc({LIGNES_DE_L_ETALON} * var(--ligne));");
        assert!(
            css.contains(&etalon),
            "{etalon} manque à la feuille de style"
        );
    }

    /// La valeur que déclare pour `propriete` une règle de la feuille de style `css` dont le
    /// sélecteur, seul ou dernier d'une liste, ouvre une ligne et vaut `selecteur` : la première
    /// règle qui la déclare ; rien si aucune ne la déclare.
    fn declaree<'a>(css: &'a str, selecteur: &str, propriete: &str) -> Option<&'a str> {
        let tete = format!("\n{selecteur} {{");
        let declaration = format!("\n  {propriete}: ");
        css.match_indices(&tete).find_map(|(debut, _)| {
            let regle = &css[debut + tete.len()..];
            let regle = &regle[..regle.find('}').unwrap_or(regle.len())];
            let valeur = &regle[regle.find(&declaration)? + declaration.len()..];
            Some(valeur[..valeur.find(';').unwrap_or(valeur.len())].trim())
        })
    }

    /// Une longueur de la feuille de style en rem, sans son unité.
    fn en_rem(valeur: &str) -> f64 {
        valeur
            .strip_suffix("rem")
            .and_then(|nombre| nombre.parse().ok())
            .unwrap_or_else(|| panic!("{valeur} n'est pas une longueur en rem"))
    }

    #[test]
    fn une_ligne_du_journal_tient_trois_lignes_et_son_etalon_en_mesure_cent() {
        // Coupé sur une ligne, un message long du journal perdait son verbe (« Requête « Une
        // requête au nom très long… »). Chaque ligne du journal a une hauteur fixe qui tient trois
        // lignes de texte, où le message passe à la ligne ; il ne se coupe qu'au-delà de la
        // troisième. La hauteur et l'interligne s'écrivent en rem : ils suivent la police, à 100 %
        // comme à 200 %.
        let css = include_str!("../../style.css");
        let lue = |selecteur: &str, propriete: &str| {
            declaree(css, selecteur, propriete)
                .unwrap_or_else(|| panic!("{selecteur} ne déclare pas {propriete}"))
        };
        let message = ".cellules.journal .cellule";
        assert_eq!(lue(message, "white-space"), "normal");
        assert_eq!(lue(message, "display"), "-webkit-box");
        assert_eq!(lue(message, "-webkit-box-orient"), "vertical");
        let lignes_de_texte: u32 = lue(message, "-webkit-line-clamp")
            .parse()
            .expect("un nombre de lignes");
        assert_eq!(lignes_de_texte, 3);
        let interligne = en_rem(lue(message, "line-height"));
        let hauteur = en_rem(lue(".volet-du-journal", "--ligne-du-journal"));
        // Les trois lignes de texte et une marge d'un demi-rem au moins, le filet du bas compris,
        // tiennent dans la ligne : rien ne déborde sur la suivante.
        assert!(
            hauteur >= f64::from(lignes_de_texte) * interligne + 0.5,
            "une ligne du journal de {hauteur} rem ne tient pas trois lignes de {interligne} rem"
        );
        assert_eq!(
            lue(".volet-du-journal .ligne", "height"),
            "var(--ligne-du-journal)"
        );
        // Le second étalon, dans le volet du journal, mesure cent lignes du journal, comme le
        // premier mesure cent lignes de la boîte.
        assert_eq!(
            lue(".volet-du-journal .etalon > div", "height"),
            format!("calc({LIGNES_DE_L_ETALON} * var(--ligne-du-journal))")
        );
        // Les lignes de la boîte et des idées retirées gardent leur hauteur et leur coupe sur une
        // ligne.
        assert_eq!(lue(".ligne", "height"), "var(--ligne)");
        assert_eq!(lue(".cellule", "white-space"), "nowrap");
        assert_eq!(lue(".cellule", "text-overflow"), "ellipsis");
    }

    #[test]
    fn chaque_liste_se_mesure_sur_la_hauteur_de_ses_lignes() {
        // Les lignes de la boîte et des idées retirées ont la hauteur `--ligne`, que mesure
        // l'étalon de la boîte ; celles du journal, la hauteur `--ligne-du-journal`, que mesure
        // l'étalon de son volet. La piste et le haut des lignes rendues se posent sur la hauteur
        // que mesure l'étalon de la fenêtre demandée : mesuré sur les lignes de la boîte, plus
        // basses, le journal demanderait des lignes plus loin que celles qu'il montre, et des
        // blancs paraîtraient au défilement.
        assert_eq!(Etalon::Boite.hauteur_d_une_ligne(), "--ligne");
        assert_eq!(Etalon::Journal.hauteur_d_une_ligne(), "--ligne-du-journal");
        assert_eq!(Etalon::Boite.hauteur(12), "calc(12 * var(--ligne))");
        assert_eq!(
            Etalon::Journal.hauteur(12),
            "calc(12 * var(--ligne-du-journal))"
        );
        assert_eq!(
            Etalon::Journal.hauteur(0),
            "calc(0 * var(--ligne-du-journal))"
        );
    }

    #[test]
    fn la_liste_et_le_volet_ne_grandissent_pas_avec_leur_contenu() {
        // `.boite-corps` passe à la ligne (`flex-wrap: wrap`) : chaque rangée y prend au moins la
        // hauteur de son contenu. Sans `contain: size`, la liste grandit avec sa piste, la page
        // défile à sa place, et la liste, qui ne défile plus, ne demande plus de fenêtre de lignes
        // à son défilement ; le volet ne défile pas non plus.
        let css = include_str!("../../style.css");
        for selecteur in [".colonne-liste", ".volet"] {
            let tete = format!("\n{selecteur} {{");
            let debut = css
                .find(&tete)
                .unwrap_or_else(|| panic!("la règle {selecteur} manque à la feuille de style"));
            let regle = &css[debut..];
            let regle = &regle[..regle.find('}').unwrap_or(regle.len())];
            assert!(
                regle.contains("contain: size;"),
                "la règle {selecteur} ne porte pas `contain: size;`"
            );
        }
    }

    #[test]
    fn le_focus_venu_du_dehors_entre_sur_une_ligne_entierement_vue() {
        // Des lignes de 40 px, une vue de 320 px défilée de 1 296 px : les lignes 33 à 39 sont
        // entièrement vues, la 32 et la 40 en partie.
        let a = |rang| ligne_d_entree(rang, 1_296.0, 320.0, 40.0);
        // Tab atteint d'abord la première ligne de la fenêtre, cachée dans la marge du haut.
        assert_eq!(a(27), Some((Bord::Haut, 33)));
        assert_eq!(a(31), Some((Bord::Haut, 33)));
        // Une ligne vue, même en partie, garde le focus.
        assert_eq!(a(32), None);
        assert_eq!(a(33), None);
        assert_eq!(a(39), None);
        assert_eq!(a(40), None);
        // Maj+Tab, venu d'en dessous, atteint d'abord une ligne cachée dans la marge du bas.
        assert_eq!(a(41), Some((Bord::Bas, 39)));
        assert_eq!(a(44), Some((Bord::Bas, 39)));
        // Aux bords exacts, une vue de 1 280 à 1 600 px : la ligne 31 finit au haut de la vue, la
        // ligne 40 commence à son bas ; les lignes 32 et 39 sont entièrement vues.
        let b = |rang| ligne_d_entree(rang, 1_280.0, 320.0, 40.0);
        assert_eq!(b(31), Some((Bord::Haut, 32)));
        assert_eq!(b(32), None);
        assert_eq!(b(39), None);
        assert_eq!(b(40), Some((Bord::Bas, 39)));
        // La liste en haut : la première ligne est vue, rien ne change.
        assert_eq!(ligne_d_entree(0, 0.0, 320.0, 40.0), None);
        assert_eq!(ligne_d_entree(12, 0.0, 320.0, 40.0), Some((Bord::Bas, 7)));
        // Une vue plus basse qu'une ligne n'en voit aucune entière : rien ne change.
        assert_eq!(ligne_d_entree(0, 50.0, 30.0, 40.0), None);
        assert_eq!(ligne_d_entree(5, 50.0, 30.0, 40.0), None);
        // Sans hauteur de ligne lisible, rien ne change.
        assert_eq!(ligne_d_entree(0, 1_296.0, 320.0, 0.0), None);
        assert_eq!(ligne_d_entree(0, 1_296.0, 320.0, f64::NAN), None);
    }

    #[test]
    fn l_idee_d_un_rang_se_lit_dans_la_fenetre() {
        let lignes: Vec<LigneDeLaBoite> = ["a", "b", "c"]
            .iter()
            .map(|id| LigneDeLaBoite {
                idee: id.to_string(),
                ..ligne(None, &[])
            })
            .collect();
        let fenetre = Fenetre {
            debut: 10,
            nombre: 3,
        };
        assert_eq!(idee_de_rang(&fenetre, &lignes, 10).as_deref(), Some("a"));
        assert_eq!(idee_de_rang(&fenetre, &lignes, 12).as_deref(), Some("c"));
        assert_eq!(idee_de_rang(&fenetre, &lignes, 9), None);
        assert_eq!(idee_de_rang(&fenetre, &lignes, 13), None);
    }
}
