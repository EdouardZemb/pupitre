//! La fiche témoin d'H09 : trois pages écrites à la main, qui prouvent que la page
//! en paysage tient au corps de 11 pt.
//!
//! Ses textes et ses positions sont des constantes : elle n'emploie que la couche
//! générique de `noyau::rendu`, sans la théorie. Ses trois pages sont le sommaire,
//! la table de la mineur, couche sombre affichée, et son verso.
//!
//! La grille part de celle du plan (marges de 6 mm, gouttières de 1,5 mm), à un
//! écart près : les marges de gauche et de droite sont de 5 mm, pour que la palette
//! sur le clavier (neuf touches blanches de 3 mm, soit 27 mm) tienne dans sa
//! colonne, qui passe de 26,93 à 27,21 mm. La hauteur utile reste de 145 mm.

use noyau::rendu::pages::{Date, HAUTEUR, LARGEUR, Lien, PageDuCahier, Rect};
use noyau::rendu::svg::{Couleur, Element, Page, TRAIT_FIN, Trait};
use noyau::rendu::texte::{Corps, MotTropLong, chasse, composer, couper};

/// Ce qui empêche la fiche de tenir dans sa page.
#[derive(Debug, Clone, PartialEq)]
pub enum ErreurTemoin {
    /// Un mot plus large que sa case.
    MotTropLong(MotTropLong),
    /// La somme des hauteurs d'une page dépasse les 145 mm utiles.
    Debordement { quoi: String, hauteur: f32 },
}

impl From<MotTropLong> for ErreurTemoin {
    fn from(e: MotTropLong) -> Self {
        ErreurTemoin::MotTropLong(e)
    }
}

/// Les trois pages de la fiche, datées du rendu.
pub fn pages_temoin(date: &Date) -> Result<Vec<PageDuCahier>, ErreurTemoin> {
    Ok(fiche(date)?.0)
}

/// Les pages, et chacun des textes qu'elles composent (pour le test des glyphes).
fn fiche(date: &Date) -> Result<(Vec<PageDuCahier>, Vec<String>), ErreurTemoin> {
    let mut pages = Vec::new();
    let mut textes = Vec::new();
    for (page, t) in [sommaire(date)?, table(date)?, verso(date)?] {
        pages.push(page);
        textes.extend(t);
    }
    Ok((pages, textes))
}

// --- La grille ---------------------------------------------------------------

const MARGE_V: f32 = 6.0;
const MARGE_H: f32 = 5.0;
const GOUTTIERE: f32 = 1.5;
const UTILE_L: f32 = LARGEUR - 2.0 * MARGE_H;
const UTILE_H: f32 = HAUTEUR - 2.0 * MARGE_V;
/// Une colonne par degré : sept colonnes et six gouttières.
const COLONNE: f32 = (UTILE_L - 6.0 * GOUTTIERE) / 7.0;
/// Le retrait du texte dans sa case, de chaque côté.
const RETRAIT: f32 = 1.0;
/// Les huit cases sombres du mineur se répartissent sur toute la largeur.
const CASE_SOMBRE: f32 = (UTILE_L - 7.0 * GOUTTIERE) / 8.0;
/// 0,5 pt : les lignes des diagrammes et les flèches.
const TRAIT_MOYEN: f32 = 2.0 * TRAIT_FIN;
/// L'espace entre un filet et le texte qui le suit ou le précède.
const SOUS_FILET: f32 = 1.2;

fn x_colonne(c: usize) -> f32 {
    MARGE_H + c as f32 * (COLONNE + GOUTTIERE)
}

fn x_sombre(i: usize) -> f32 {
    MARGE_H + i as f32 * (CASE_SOMBRE + GOUTTIERE)
}

fn corps_nom() -> Corps {
    Corps::pt(14.0).expect("14 pt dépasse le minimum")
}

fn corps_titre() -> Corps {
    Corps::pt(20.0).expect("20 pt dépasse le minimum")
}

/// L'interligne : 1,2 cadratin.
fn interligne(c: &Corps) -> f32 {
    1.2 * c.mm()
}

/// Du haut d'une ligne à sa ligne de base : la ligne de 1,2 cadratin centre les
/// 0,928 cadratin au-dessus de la ligne de base et les 0,236 au-dessous de DejaVu Sans.
fn jusqua_la_base(c: &Corps) -> f32 {
    0.946 * c.mm()
}

/// De la ligne de base au milieu des capitales (0,729 cadratin de haut) : là se
/// centrent une pastille ou une marque posée devant un texte.
fn mi_capitale(c: &Corps) -> f32 {
    0.365 * c.mm()
}

/// Coupe un texte de la forme « a · b · c » de préférence aux « · », et chaque
/// segment trop long entre deux mots.
fn couper_aux_points(texte: &str, corps: &Corps, largeur: f32) -> Result<Vec<String>, MotTropLong> {
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

/// Un nombre du tracé, à trois décimales au plus.
fn n(v: f32) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

// --- La feuille : les éléments d'une page, et ses textes ---------------------

#[derive(Default)]
struct Feuille {
    elements: Vec<Element>,
    textes: Vec<String>,
}

impl Feuille {
    fn texte(&mut self, s: &str, corps: &Corps, x: f32, base: f32) {
        self.textes.push(s.to_string());
        self.elements.push(composer(s, corps, x, base));
    }

    fn texte_centre(&mut self, s: &str, corps: &Corps, cx: f32, base: f32) {
        self.texte(s, corps, cx - chasse(s, corps) / 2.0, base);
    }

    fn texte_droite(&mut self, s: &str, corps: &Corps, droite: f32, base: f32) {
        self.texte(s, corps, droite - chasse(s, corps), base);
    }

    /// Des lignes à 11 pt, dès `haut` ; rend leur hauteur.
    fn lignes(&mut self, lignes: &[String], x: f32, haut: f32) -> f32 {
        let c = Corps::MIN;
        for (k, l) in lignes.iter().enumerate() {
            self.texte(
                l,
                &c,
                x,
                haut + k as f32 * interligne(&c) + jusqua_la_base(&c),
            );
        }
        lignes.len() as f32 * interligne(&c)
    }

    fn ligne(&mut self, de: (f32, f32), a: (f32, f32), epaisseur: f32) {
        self.elements.push(Element::Ligne {
            de,
            a,
            trait_: (Couleur::Noir, epaisseur),
        });
    }

    fn filet(&mut self, y: f32) {
        self.ligne((MARGE_H, y), (MARGE_H + UTILE_L, y), TRAIT_FIN);
    }

    fn rect(&mut self, r: Rect, id: Option<&'static str>, plein: Couleur, trait_: Option<Trait>) {
        self.elements.push(Element::Rect {
            r,
            id,
            plein: Some(plein),
            trait_,
        });
    }

    fn cercle(&mut self, c: (f32, f32), r: f32, plein: Option<Couleur>, trait_: Option<Trait>) {
        self.elements.push(Element::Cercle {
            cx: c.0,
            cy: c.1,
            r,
            id: None,
            plein,
            trait_,
        });
    }

    fn chemin(&mut self, d: String, plein: Option<Couleur>, trait_: Option<Trait>) {
        self.elements.push(Element::Chemin { d, plein, trait_ });
    }

    /// Un groupe nommé, dessiné dans son propre repère.
    fn groupe(&mut self, id: &'static str, origine: (f32, f32), dessin: impl FnOnce(&mut Feuille)) {
        let mut dedans = Feuille::default();
        dessin(&mut dedans);
        self.textes.extend(dedans.textes);
        self.elements.push(Element::Groupe {
            id: Some(id),
            translation: origine,
            echelle: None,
            enfants: dedans.elements,
        });
    }

    fn page(self, liens: Vec<Lien>) -> (PageDuCahier, Vec<String>) {
        let svg = Page {
            largeur: LARGEUR,
            hauteur: HAUTEUR,
            elements: self.elements,
        }
        .svg();
        (PageDuCahier { svg, liens }, self.textes)
    }
}

/// Le pied, au bas de la page, sous un filet.
fn pied(f: &mut Feuille, lignes: &[String]) {
    let haut = MARGE_V + UTILE_H - lignes.len() as f32 * interligne(&Corps::MIN);
    f.filet(haut - SOUS_FILET);
    f.lignes(lignes, MARGE_H, haut);
}

/// La hauteur du pied, filet compris.
fn hauteur_pied(lignes: &[String]) -> f32 {
    SOUS_FILET + lignes.len() as f32 * interligne(&Corps::MIN)
}

fn pied_de_page(numero: u8, date: &Date, verso: bool) -> String {
    let mut s = format!(
        "page {numero} / 3 · rendu le {} · empreinte 000000",
        date.en_francais()
    );
    if verso {
        s.push_str(" · verso : diagrammes et palettes des accords du centre");
    }
    s
}

fn deborde(quoi: &str, hauteur: f32) -> Result<(), ErreurTemoin> {
    if hauteur > UTILE_H {
        Err(ErreurTemoin::Debordement {
            quoi: quoi.to_string(),
            hauteur,
        })
    } else {
        Ok(())
    }
}

// --- Page 1 : le sommaire ----------------------------------------------------

const TITRE: &str = "Cahier des tables";
const SOUS_TITRE: &str = "fiche témoin · trois pages, au corps de 11 pt";
const LETTRES: &str = "Lettres : à â ç é è ê ë î ï ô ù û ü ÿ œ æ · ♭ ♯ °";
const ENTREE: &str = "la mineur · table de la tonalité";
/// La zone d'un lien : au moins 12 mm de haut (§ 50 · 6.3).
const ZONE_LIEN: f32 = 12.0;

fn sommaire(date: &Date) -> Result<(PageDuCahier, Vec<String>), ErreurTemoin> {
    let (t, nom, titre) = (Corps::MIN, corps_nom(), corps_titre());
    let pied_l = couper_aux_points(&pied_de_page(1, date, false), &t, UTILE_L)?;
    let lettres = couper(LETTRES, &t, UTILE_L)?;

    let mut f = Feuille::default();
    let mut y = MARGE_V;
    f.texte(TITRE, &titre, MARGE_H, y + jusqua_la_base(&titre));
    y += interligne(&titre);
    y += f.lignes(&[SOUS_TITRE.to_string()], MARGE_H, y);
    y += 4.0;
    y += f.lignes(&lettres, MARGE_H, y);
    y += 8.0;
    f.texte("Sommaire", &nom, MARGE_H, y + jusqua_la_base(&nom));
    y += interligne(&nom) + 1.0;

    // L'entrée : sa zone, entre deux filets, est celle du lien.
    let zone = Rect {
        x: MARGE_H,
        y,
        l: UTILE_L,
        h: ZONE_LIEN,
    };
    f.filet(y);
    f.filet(y + ZONE_LIEN);
    let base = y + ZONE_LIEN / 2.0 + mi_capitale(&t);
    f.texte(ENTREE, &t, MARGE_H + RETRAIT, base);
    f.texte_droite("page 2", &t, MARGE_H + UTILE_L - RETRAIT, base);
    y += ZONE_LIEN;

    // Au moins 4 mm de blanc avant le pied.
    deborde("le sommaire", y - MARGE_V + 4.0 + hauteur_pied(&pied_l))?;
    pied(&mut f, &pied_l);
    Ok(f.page(vec![Lien { zone, vers: 1 }]))
}

// --- Page 2 : la table de la mineur ------------------------------------------

const ENTETE_TABLE: &str = "la mineur · table de la tonalité · tonalité choisie · instrument : guitare et clavier · couche sombre : affichée";

/// Une case de la table : son chiffre, son nom, sa ligne d'effet.
struct Case {
    chiffre: &'static str,
    nom: &'static str,
    effet: &'static str,
}

const fn case(chiffre: &'static str, nom: &'static str, effet: &'static str) -> Case {
    Case {
        chiffre,
        nom,
        effet,
    }
}

/// Les dominantes secondaires, chacune au-dessus de sa cible (colonnes 0, 2 à 6).
const DESSUS: [(usize, Case); 6] = [
    (0, case("V7/i", "E7", "tire vers Am")),
    (2, case("V7/III", "G7", "tire vers C")),
    (3, case("V7/iv", "A7", "tire vers Dm")),
    (4, case("V7/v", "B7", "tire vers Em")),
    (5, case("V7/VI", "C7", "tire vers F")),
    (6, case("V7/VII", "D7", "tire vers G")),
];

/// La fonction d'un accord du centre, que redouble sa pastille.
#[derive(Clone, Copy)]
enum Fonction {
    Repos,
    Eloignement,
    Tension,
}

const CENTRE: [(Case, Fonction); 7] = [
    (case("i", "Am", "repos"), Fonction::Repos),
    (case("ii°", "B°", "éloignement"), Fonction::Eloignement),
    (case("III", "C", "repos"), Fonction::Repos),
    (case("iv", "Dm", "éloignement"), Fonction::Eloignement),
    (case("v", "Em", "tension"), Fonction::Tension),
    (case("VI", "F", "éloignement"), Fonction::Eloignement),
    (case("VII", "G", "tension"), Fonction::Tension),
];

/// Où mène la flèche d'une case sombre.
#[derive(Clone, Copy)]
enum Cible {
    Centre(usize),
    Dessous(usize),
}

/// La couche sombre, dans l'ordre des degrés de ses cibles.
const SOMBRE: [(Case, Option<Cible>); 8] = [
    (
        case("vii°7/i", "G♯°7", "glisse vers Am"),
        Some(Cible::Centre(0)),
    ),
    (case("III+", "C+", "suspend"), None),
    (
        case("vii°7/III", "B°7", "glisse vers C"),
        Some(Cible::Centre(2)),
    ),
    (
        case("vii°7/iv", "C♯°7", "glisse vers Dm"),
        Some(Cible::Centre(3)),
    ),
    (
        case("vii°7/v", "D♯°7", "glisse vers Em"),
        Some(Cible::Centre(4)),
    ),
    (
        case("♭II6", "B♭/D", "tension sombre avant la dominante"),
        Some(Cible::Dessous(4)),
    ),
    (
        case("vii°7/VI", "E°7", "glisse vers F"),
        Some(Cible::Centre(5)),
    ),
    (
        case("vii°7/VII", "F♯°7", "glisse vers G"),
        Some(Cible::Centre(6)),
    ),
];

/// Les emprunts au mode parallèle, la majeur, chacun sous son degré.
const DESSOUS: [Case; 7] = [
    case("I", "A", "éclaire"),
    case("ii", "Bm", "éclaire"),
    case("iii", "C♯m", "éclaire"),
    case("IV", "D", "éclaire"),
    case("V", "E", "éclaire"),
    case("vi", "F♯m", "éclaire"),
    case("vii°", "G♯°", "éclaire"),
];

const RANGEE_DESSUS: &str = "dominantes secondaires";
const RANGEE_CENTRE: &str = "diatoniques";
const RANGEE_SOMBRE: &str = "couche sombre";
const RANGEE_DESSOUS: &str = "emprunts au mode parallèle · la majeur";

/// Les quatre cadres, au même trait fin.
#[derive(Clone, Copy)]
enum Cadre {
    Droit,
    EnPointe,
    Arrondi,
    Coupe,
}

/// La profondeur de la pointe d'une dominante secondaire, sous sa case.
const POINTE: f32 = 3.0;
/// Le diamètre d'une pastille de fonction (§ 50 · 6.3).
const PASTILLE: f32 = 3.0;

impl Cadre {
    /// Le blanc au-dessus et au-dessous du texte. Les coins coupés en
    /// demandent plus : le chiffre et la dernière ligne s'écartent de la coupe.
    fn blancs(self) -> (f32, f32) {
        match self {
            Cadre::Droit | Cadre::EnPointe => (0.8, 0.8),
            Cadre::Arrondi => (1.0, 1.2),
            Cadre::Coupe => (1.4, 4.4),
        }
    }

    /// L'arrondi ou la coupe : le quart du petit côté de la case.
    fn coin(l: f32, h: f32) -> f32 {
        l.min(h) / 4.0
    }

    /// Le tracé du cadre ; la pointe d'une dominante secondaire descend de
    /// `pointe` sous la case.
    fn trace(self, x: f32, y: f32, l: f32, h: f32, pointe: f32) -> String {
        let (x1, y1) = (x + l, y + h);
        let c = Self::coin(l, h);
        match self {
            Cadre::Droit => format!("M{} {}H{}V{}H{}Z", n(x), n(y), n(x1), n(y1), n(x)),
            Cadre::EnPointe => format!(
                "M{} {}H{}V{}L{} {}L{} {}Z",
                n(x),
                n(y),
                n(x1),
                n(y1),
                n(x + l / 2.0),
                n(y1 + pointe),
                n(x),
                n(y1)
            ),
            Cadre::Arrondi => {
                let a = format!("A{} {} 0 0 1", n(c), n(c));
                format!(
                    "M{} {}H{}{a} {} {}V{}{a} {} {}H{}{a} {} {}V{}{a} {} {}Z",
                    n(x + c),
                    n(y),
                    n(x1 - c),
                    n(x1),
                    n(y + c),
                    n(y1 - c),
                    n(x1 - c),
                    n(y1),
                    n(x + c),
                    n(x),
                    n(y1 - c),
                    n(y + c),
                    n(x + c),
                    n(y)
                )
            }
            Cadre::Coupe => format!(
                "M{} {}H{}L{} {}V{}L{} {}H{}L{} {}V{}Z",
                n(x + c),
                n(y),
                n(x1 - c),
                n(x1),
                n(y + c),
                n(y1 - c),
                n(x1 - c),
                n(y1),
                n(x + c),
                n(x),
                n(y1 - c),
                n(y + c)
            ),
        }
    }
}

/// Entre deux rangées : la ligne du nom de la rangée qui suit, et les flèches.
fn entre_rangees() -> f32 {
    0.6 + interligne(&Corps::MIN) + 0.6
}

/// La hauteur d'une case à `lignes` lignes d'effet.
fn hauteur_case(cadre: Cadre, lignes: usize) -> f32 {
    let (haut, bas) = cadre.blancs();
    let (t, nom) = (Corps::MIN, corps_nom());
    haut + interligne(&t) + interligne(&nom) + lignes as f32 * interligne(&t) + bas
}

/// Les lignes d'effet de chaque case, et la hauteur de la rangée : celle de sa
/// case la plus haute.
fn rangee<'a>(
    cases: impl Iterator<Item = &'a Case>,
    cadre: Cadre,
    largeur: f32,
) -> Result<(Vec<Vec<String>>, f32), MotTropLong> {
    let mut effets = Vec::new();
    let mut hauteur: f32 = 0.0;
    for c in cases {
        let lignes = couper(c.effet, &Corps::MIN, largeur - 2.0 * RETRAIT)?;
        couper(c.chiffre, &Corps::MIN, largeur - 2.0 * RETRAIT)?;
        couper(c.nom, &corps_nom(), largeur - 2.0 * RETRAIT)?;
        hauteur = hauteur.max(hauteur_case(cadre, lignes.len()));
        effets.push(lignes);
    }
    Ok((effets, hauteur))
}

/// Une case : son cadre, son chiffre (et sa pastille), son nom, ses lignes d'effet.
fn dessiner_case(
    f: &mut Feuille,
    cadre: Cadre,
    r: Rect,
    c: &Case,
    effet: &[String],
    fonction: Option<Fonction>,
) {
    let (t, nom) = (Corps::MIN, corps_nom());
    f.chemin(
        cadre.trace(r.x, r.y, r.l, r.h, POINTE),
        Some(Couleur::Blanc),
        Some((Couleur::Noir, TRAIT_FIN)),
    );
    let cx = r.x + r.l / 2.0;
    let mut y = r.y + cadre.blancs().0;
    let base = y + jusqua_la_base(&t);
    match fonction {
        None => f.texte_centre(c.chiffre, &t, cx, base),
        Some(fonction) => {
            // Le chiffre et sa pastille, centrés ensemble.
            let ecart = 1.2;
            let l = chasse(c.chiffre, &t) + ecart + PASTILLE;
            let x = cx - l / 2.0;
            f.texte(c.chiffre, &t, x, base);
            pastille(
                f,
                (x + l - PASTILLE / 2.0, base - mi_capitale(&t)),
                fonction,
            );
        }
    }
    y += interligne(&t);
    f.texte_centre(c.nom, &nom, cx, y + jusqua_la_base(&nom));
    y += interligne(&nom);
    for l in effet {
        f.texte_centre(l, &t, cx, y + jusqua_la_base(&t));
        y += interligne(&t);
    }
}

/// ○ repos, ◐ éloignement, ● tension : 3 mm de diamètre.
fn pastille(f: &mut Feuille, c: (f32, f32), fonction: Fonction) {
    let r = PASTILLE / 2.0;
    let plein = match fonction {
        Fonction::Tension => Couleur::Noir,
        Fonction::Repos | Fonction::Eloignement => Couleur::Blanc,
    };
    f.elements.push(Element::Cercle {
        cx: c.0,
        cy: c.1,
        r,
        id: Some("pastille"),
        plein: Some(plein),
        trait_: Some((Couleur::Noir, TRAIT_FIN)),
    });
    if let Fonction::Eloignement = fonction {
        // La moitié gauche, pleine.
        f.chemin(
            format!(
                "M{} {}A{} {} 0 0 0 {} {}Z",
                n(c.0),
                n(c.1 - r),
                n(r),
                n(r),
                n(c.0),
                n(c.1 + r)
            ),
            Some(Couleur::Noir),
            None,
        );
    }
}

/// Une flèche de résolution : un trait plein, et sa pointe pleine posée sur la cible.
fn fleche(f: &mut Feuille, de: (f32, f32), a: (f32, f32)) {
    let (longueur, demi) = (1.8, 0.8);
    let (dx, dy) = (a.0 - de.0, a.1 - de.1);
    let d = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = (dx / d, dy / d);
    let base = (a.0 - ux * longueur, a.1 - uy * longueur);
    // Le trait entre un peu dans la pointe, pour ne laisser aucun jour.
    f.ligne(de, (base.0 + ux * 0.3, base.1 + uy * 0.3), TRAIT_MOYEN);
    f.chemin(
        format!(
            "M{} {}L{} {}L{} {}Z",
            n(a.0),
            n(a.1),
            n(base.0 - uy * demi),
            n(base.1 + ux * demi),
            n(base.0 + uy * demi),
            n(base.1 - ux * demi)
        ),
        Some(Couleur::Noir),
        None,
    );
}

fn table(date: &Date) -> Result<(PageDuCahier, Vec<String>), ErreurTemoin> {
    let t = Corps::MIN;
    let il = interligne(&t);
    let entete = couper_aux_points(ENTETE_TABLE, &t, UTILE_L)?;
    let pied_l = couper_aux_points(&pied_de_page(2, date, true), &t, UTILE_L)?;
    let (effets_dessus, h_dessus) =
        rangee(DESSUS.iter().map(|(_, c)| c), Cadre::EnPointe, COLONNE)?;
    let (effets_centre, h_centre) = rangee(CENTRE.iter().map(|(c, _)| c), Cadre::Droit, COLONNE)?;
    let (effets_sombre, h_sombre) =
        rangee(SOMBRE.iter().map(|(c, _)| c), Cadre::Coupe, CASE_SOMBRE)?;
    let (effets_dessous, h_dessous) = rangee(DESSOUS.iter(), Cadre::Arrondi, COLONNE)?;

    // La mise en page, de haut en bas : chaque rangée prend la hauteur de sa
    // case la plus haute, et le nom de la rangée qui suit se loge entre deux.
    let y_filet = MARGE_V + entete.len() as f32 * il + SOUS_FILET / 2.0;
    let y_nom_dessus = y_filet + 1.5 * SOUS_FILET;
    let y_dessus = y_nom_dessus + il + 0.4;
    let y_centre = y_dessus + h_dessus + POINTE + entre_rangees();
    let y_sombre = y_centre + h_centre + entre_rangees();
    let y_dessous = y_sombre + h_sombre + entre_rangees();
    let y_legende = y_dessous + h_dessous + 3.0;
    let hauteur = y_legende + il - MARGE_V + hauteur_pied(&pied_l);
    deborde("la table de la mineur", hauteur)?;

    let mut f = Feuille::default();
    f.lignes(&entete, MARGE_H, MARGE_V);
    f.filet(y_filet);

    // Le nom de chaque rangée s'aligne sur la colonne 1, que nulle flèche ne
    // traverse dans la table de la mineur.
    let x_nom = x_colonne(1) + RETRAIT;
    let nom_de_rangee = |f: &mut Feuille, nom: &str, haut: f32| {
        f.texte(nom, &t, x_nom, haut + jusqua_la_base(&t));
    };

    nom_de_rangee(&mut f, RANGEE_DESSUS, y_nom_dessus);
    for ((col, c), effet) in DESSUS.iter().zip(&effets_dessus) {
        let r = Rect {
            x: x_colonne(*col),
            y: y_dessus,
            l: COLONNE,
            h: h_dessus,
        };
        dessiner_case(&mut f, Cadre::EnPointe, r, c, effet, None);
    }

    nom_de_rangee(&mut f, RANGEE_CENTRE, y_centre - entre_rangees() + 0.6);
    for (col, ((c, fonction), effet)) in CENTRE.iter().zip(&effets_centre).enumerate() {
        let r = Rect {
            x: x_colonne(col),
            y: y_centre,
            l: COLONNE,
            h: h_centre,
        };
        dessiner_case(&mut f, Cadre::Droit, r, c, effet, Some(*fonction));
    }

    nom_de_rangee(&mut f, RANGEE_SOMBRE, y_sombre - entre_rangees() + 0.6);
    for (i, ((c, _), effet)) in SOMBRE.iter().zip(&effets_sombre).enumerate() {
        let r = Rect {
            x: x_sombre(i),
            y: y_sombre,
            l: CASE_SOMBRE,
            h: h_sombre,
        };
        dessiner_case(&mut f, Cadre::Coupe, r, c, effet, None);
    }

    nom_de_rangee(&mut f, RANGEE_DESSOUS, y_dessous - entre_rangees() + 0.6);
    for (col, (c, effet)) in DESSOUS.iter().zip(&effets_dessous).enumerate() {
        let r = Rect {
            x: x_colonne(col),
            y: y_dessous,
            l: COLONNE,
            h: h_dessous,
        };
        dessiner_case(&mut f, Cadre::Arrondi, r, c, effet, None);
    }

    // Les flèches, d'une rangée à sa voisine. Une case sombre vise sa cible au
    // plus près : droit quand elle est en face, en oblique sinon.
    for (col, _) in &DESSUS {
        let x = x_colonne(*col) + COLONNE / 2.0;
        fleche(&mut f, (x, y_dessus + h_dessus + POINTE), (x, y_centre));
    }
    let au_plus_pres =
        |x: f32, col: usize| x.clamp(x_colonne(col) + 4.0, x_colonne(col) + COLONNE - 4.0);
    for (i, (_, cible)) in SOMBRE.iter().enumerate() {
        let x = x_sombre(i) + CASE_SOMBRE / 2.0;
        match cible {
            Some(Cible::Centre(col)) => {
                let a = (au_plus_pres(x, *col), y_centre + h_centre);
                fleche(&mut f, (x, y_sombre), a);
            }
            Some(Cible::Dessous(col)) => {
                let a = (au_plus_pres(x, *col), y_dessous);
                fleche(&mut f, (x, y_sombre + h_sombre), a);
            }
            None => {}
        }
    }

    legende(&mut f, y_legende);
    pied(&mut f, &pied_l);
    Ok(f.page(Vec::new()))
}

/// La légende des quatre cadres, sur une ligne.
fn legende(f: &mut Feuille, haut: f32) {
    let t = Corps::MIN;
    let base = haut + jusqua_la_base(&t);
    let (l, h) = (7.0, 4.2);
    let y = base - 3.4;
    let mut x = MARGE_H;
    let titre = "cadres :";
    f.texte(titre, &t, x, base);
    x += chasse(titre, &t) + 3.0;
    for (cadre, nom) in [
        (Cadre::Droit, "diatonique"),
        (Cadre::EnPointe, "dominante secondaire"),
        (Cadre::Arrondi, "emprunt"),
        (Cadre::Coupe, "harmonie sombre"),
    ] {
        let pointe = 1.4;
        let trace = match cadre {
            Cadre::EnPointe => cadre.trace(x, y, l, h - pointe, pointe),
            _ => cadre.trace(x, y, l, h, 0.0),
        };
        f.chemin(
            trace,
            Some(Couleur::Blanc),
            Some((Couleur::Noir, TRAIT_FIN)),
        );
        x += l + 1.5;
        f.texte(nom, &t, x, base);
        x += chasse(nom, &t) + 5.0;
    }
}

// --- Page 3 : le verso de la mineur ------------------------------------------

const ENTETE_VERSO: &str =
    "la mineur · verso de la table · diagrammes et palettes des accords du centre";

/// Un accord du centre, écrit à la main : son doigté, ses notes au clavier, et
/// les rôles de sa palette (en classes de hauteur, do = 0), ceux de l'oracle.
struct Accord {
    nom: &'static str,
    chiffre: &'static str,
    /// Les frettes, de la corde grave à l'aiguë : `x` pour une corde étouffée.
    frettes: &'static str,
    /// Le doigt de chaque corde appuyée (0 : aucun), de la grave à l'aiguë.
    doigts: [u8; 6],
    /// Le barré : sa frette, et ses cordes extrêmes.
    barre: Option<(u8, usize, usize)>,
    /// Les notes au clavier, en position fondamentale (MIDI, do 3 = 60).
    clavier: [u8; 3],
    /// La lettre de la basse.
    basse: &'static str,
    appuis: &'static [u8],
    tensions: &'static [u8],
    frottent: &'static [u8],
}

const ACCORDS: [Accord; 7] = [
    Accord {
        nom: "Am",
        chiffre: "i",
        frettes: "x02210",
        doigts: [0, 0, 2, 3, 1, 0],
        barre: None,
        clavier: [57, 60, 64],
        basse: "A",
        appuis: &[9, 0, 4],
        tensions: &[11, 2],
        frottent: &[5, 7],
    },
    Accord {
        nom: "B°",
        chiffre: "ii°",
        frettes: "x2343x",
        doigts: [0, 1, 2, 4, 3, 0],
        barre: None,
        clavier: [59, 62, 65],
        basse: "B",
        appuis: &[11, 2, 5],
        tensions: &[4],
        frottent: &[0, 7, 9],
    },
    Accord {
        nom: "C",
        chiffre: "III",
        frettes: "x32010",
        doigts: [0, 3, 2, 0, 1, 0],
        barre: None,
        clavier: [60, 64, 67],
        basse: "C",
        appuis: &[0, 4, 7],
        tensions: &[2, 9],
        frottent: &[5, 11],
    },
    Accord {
        nom: "Dm",
        chiffre: "iv",
        frettes: "xx0231",
        doigts: [0, 0, 0, 2, 3, 1],
        barre: None,
        clavier: [62, 65, 69],
        basse: "D",
        appuis: &[2, 5, 9],
        tensions: &[4, 7, 11],
        frottent: &[0],
    },
    Accord {
        nom: "Em",
        chiffre: "v",
        frettes: "022000",
        doigts: [0, 1, 2, 0, 0, 0],
        barre: None,
        clavier: [64, 67, 71],
        basse: "E",
        appuis: &[4, 7, 11],
        tensions: &[9],
        frottent: &[5, 0, 2],
    },
    Accord {
        nom: "F",
        chiffre: "VI",
        frettes: "133211",
        doigts: [1, 3, 4, 2, 1, 1],
        barre: Some((1, 0, 5)),
        clavier: [65, 69, 72],
        basse: "F",
        appuis: &[5, 9, 0],
        tensions: &[7, 11, 2],
        frottent: &[4],
    },
    Accord {
        nom: "G",
        chiffre: "VII",
        frettes: "320003",
        doigts: [2, 1, 0, 0, 0, 3],
        barre: None,
        clavier: [55, 59, 62],
        basse: "G",
        appuis: &[7, 11, 2],
        tensions: &[9, 4],
        frottent: &[0, 5],
    },
];

/// Les cordes à vide, de la grave à l'aiguë (MIDI : mi 1 … mi 3).
const CORDES: [u8; 6] = [40, 45, 50, 55, 59, 64];
const SOLFEGE: [&str; 12] = [
    "do", "do♯", "ré", "mi♭", "mi", "fa", "fa♯", "sol", "sol♯", "la", "si♭", "si",
];
/// Les classes de hauteur des notes naturelles, de do à si.
const NATURELLES: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// Le rôle d'une note dans la palette d'un accord (MIC-13).
#[derive(Clone, Copy, PartialEq)]
enum Role {
    Appui,
    Tension,
    Frotte,
}

impl Accord {
    fn role(&self, midi: u8) -> Option<Role> {
        let classe = midi % 12;
        if self.appuis.contains(&classe) {
            Some(Role::Appui)
        } else if self.tensions.contains(&classe) {
            Some(Role::Tension)
        } else if self.frottent.contains(&classe) {
            Some(Role::Frotte)
        } else {
            None
        }
    }

    fn frette(&self, corde: usize) -> Option<u8> {
        let c = self.frettes.as_bytes()[corde];
        c.is_ascii_digit().then(|| c - b'0')
    }

    /// La dernière frette de la zone du doigté, à ± 2 frettes ; la zone part
    /// toujours de la corde à vide pour les sept accords de la fiche.
    fn fin_de_zone(&self) -> u8 {
        (0..6).filter_map(|c| self.frette(c)).max().unwrap_or(0) + 2
    }
}

// Le diagramme de manche et la palette sur le manche.
const CORDE: f32 = 3.7;
const LARGEUR_MANCHE: f32 = 5.0 * CORDE;
const RANG_A_VIDE: f32 = 3.4;
const CASE_DOIGTE: f32 = 4.0;
const FRETTES_DOIGTE: u8 = 4;
const CASE_PALETTE: f32 = 3.3;
const SILLET: f32 = 0.7;
// Les claviers.
const TOUCHE: f32 = 3.0;
const NOIRE: f32 = 1.8;
const CLAVIER_H: f32 = 11.0;
const CLAVIER_NOIRE_H: f32 = 7.0;
const PALETTE_CLAVIER_H: f32 = 9.5;
const PALETTE_NOIRE_H: f32 = 6.0;
// La palette sur le clavier, neuf touches blanches, tient dans sa colonne.
const _: () = assert!(9.0 * TOUCHE <= COLONNE);
// La portée, sans clé, sur la largeur utile de la colonne.
const ESPACE: f32 = 1.8;
const LARGEUR_PORTEE: f32 = COLONNE - 2.0 * RETRAIT;
/// Le jour entre deux noms voisins d'une même ligne, sous la portée.
const JOUR_NOMS: f32 = 1.0;
/// Le pas des sept notes : deux noms d'une même ligne sont à deux pas, et ceux
/// des notes extrêmes ne débordent pas la portée (2 × (pas − jour / 2) + 6 pas).
const PAS_NOTES: f32 = (LARGEUR_PORTEE + JOUR_NOMS) / 8.0;
/// La chasse la plus large qu'un nom de note peut prendre.
const NOM_DE_NOTE: f32 = 2.0 * PAS_NOTES - JOUR_NOMS;
// Les blancs du verso, entre deux diagrammes et après une ligne de titre.
const ENTRE_BLOCS: f32 = 4.5;
const APRES_NOM: f32 = 1.5;
/// La place d'une marque devant le texte qu'elle précède.
const DEVANT_TEXTE: f32 = 3.2;

/// Le manche : la grille, ses cordes à vide ou étouffées, et sa zone de frettes.
fn grille(f: &mut Feuille, frettes: u8, case_frette: f32) {
    let bas = f32::from(frettes) * case_frette;
    f.ligne((0.0, 0.0), (LARGEUR_MANCHE, 0.0), SILLET);
    for k in 1..=frettes {
        let y = f32::from(k) * case_frette;
        f.ligne((0.0, y), (LARGEUR_MANCHE, y), TRAIT_MOYEN);
    }
    for c in 0..6 {
        let x = c as f32 * CORDE;
        f.ligne((x, 0.0), (x, bas), TRAIT_MOYEN);
    }
}

/// Le diagramme de manche : un point plein par doigt, un cercle par corde à vide,
/// une croix par corde étouffée, un trait épais pour le barré, les doigts dessous.
fn manche(f: &mut Feuille, a: &Accord) {
    let t = Corps::MIN;
    let bas = f32::from(FRETTES_DOIGTE) * CASE_DOIGTE;
    grille(f, FRETTES_DOIGTE, CASE_DOIGTE);
    let y_vide = -RANG_A_VIDE / 2.0 - 0.1;
    for c in 0..6 {
        let x = c as f32 * CORDE;
        match a.frette(c) {
            None => croix(f, (x, y_vide), 0.75, 0.22, Couleur::Noir),
            Some(0) => f.cercle(
                (x, y_vide),
                0.8,
                Some(Couleur::Blanc),
                Some((Couleur::Noir, 0.22)),
            ),
            Some(fr) if a.barre.is_some_and(|(b, _, _)| b == fr) => {}
            Some(fr) => f.cercle(
                (x, (f32::from(fr) - 0.5) * CASE_DOIGTE),
                1.15,
                Some(Couleur::Noir),
                None,
            ),
        }
        if a.doigts[c] > 0 {
            f.texte_centre(
                &a.doigts[c].to_string(),
                &t,
                x,
                bas + 0.6 + jusqua_la_base(&t),
            );
        }
    }
    if let Some((fr, de, jusqua)) = a.barre {
        let y = (f32::from(fr) - 0.5) * CASE_DOIGTE;
        let r = 0.75;
        let (x0, x1) = (de as f32 * CORDE, jusqua as f32 * CORDE);
        f.chemin(
            format!(
                "M{} {}H{}A{} {} 0 0 1 {} {}H{}A{} {} 0 0 1 {} {}Z",
                n(x0),
                n(y - r),
                n(x1),
                n(r),
                n(r),
                n(x1),
                n(y + r),
                n(x0),
                n(r),
                n(r),
                n(x0),
                n(y - r)
            ),
            Some(Couleur::Noir),
            None,
        );
    }
}

fn hauteur_manche() -> f32 {
    RANG_A_VIDE + f32::from(FRETTES_DOIGTE) * CASE_DOIGTE + 0.6 + interligne(&Corps::MIN)
}

/// La palette sur le manche, dans la zone du doigté : chaque note de la palette
/// à sa place, par sa forme.
fn palette_manche(f: &mut Feuille, a: &Accord) {
    let fin = a.fin_de_zone();
    grille(f, fin, CASE_PALETTE);
    for (c, a_vide) in CORDES.iter().enumerate() {
        let x = c as f32 * CORDE;
        for fr in 0..=fin {
            if let Some(role) = a.role(a_vide + fr) {
                let y = if fr == 0 {
                    -RANG_A_VIDE / 2.0 - 0.1
                } else {
                    (f32::from(fr) - 0.5) * CASE_PALETTE
                };
                marque(f, role, (x, y), 1.0);
            }
        }
    }
}

/// Appui plein, tension creuse, frotte en croix.
fn marque(f: &mut Feuille, role: Role, c: (f32, f32), r: f32) {
    match role {
        Role::Appui => f.cercle(c, r, Some(Couleur::Noir), None),
        Role::Tension => f.cercle(
            c,
            r - 0.11,
            Some(Couleur::Blanc),
            Some((Couleur::Noir, 0.22)),
        ),
        Role::Frotte => {
            f.cercle(c, r, Some(Couleur::Blanc), None);
            croix(f, c, r * 0.8, 0.3, Couleur::Noir);
        }
    }
}

fn croix(f: &mut Feuille, c: (f32, f32), demi: f32, epaisseur: f32, couleur: Couleur) {
    for s in [-1.0, 1.0] {
        f.elements.push(Element::Ligne {
            de: (c.0 - demi, c.1 - s * demi),
            a: (c.0 + demi, c.1 + s * demi),
            trait_: (couleur, epaisseur),
        });
    }
}

fn est_blanche(midi: u8) -> bool {
    NATURELLES.contains(&(midi % 12))
}

/// Un clavier, de la touche blanche `de` à la touche blanche `a` : les touches
/// blanches de 3 mm, les noires par-dessus. Rend chaque note dessinée, avec le
/// centre de sa touche et sa couleur (blanche ou non), pour les marques.
fn clavier(f: &mut Feuille, de: u8, a: u8, hauteur: f32, noire_h: f32) -> Vec<(u8, f32, bool)> {
    let mut centres = Vec::new();
    let mut x = 0.0;
    for midi in de..=a {
        if est_blanche(midi) {
            let r = Rect {
                x,
                y: 0.0,
                l: TOUCHE,
                h: hauteur,
            };
            f.rect(
                r,
                Some("touche-blanche"),
                Couleur::Blanc,
                Some((Couleur::Noir, TRAIT_MOYEN)),
            );
            centres.push((midi, x + TOUCHE / 2.0, true));
            x += TOUCHE;
        } else if midi > de && midi < a {
            let r = Rect {
                x: x - NOIRE / 2.0,
                y: 0.0,
                l: NOIRE,
                h: noire_h,
            };
            f.rect(r, None, Couleur::Noir, None);
            centres.push((midi, x, false));
        }
    }
    centres
}

fn largeur_clavier(de: u8, a: u8) -> f32 {
    (de..=a).filter(|m| est_blanche(*m)).count() as f32 * TOUCHE
}

/// La touche blanche qui précède `midi`, et celle qui suit `midi`.
fn blanche_avant(midi: u8) -> u8 {
    (0..midi).rev().find(|m| est_blanche(*m)).unwrap_or(0)
}

fn blanche_apres(midi: u8) -> u8 {
    (midi + 1..).find(|m| est_blanche(*m)).unwrap_or(midi)
}

/// Le diagramme de clavier : de la touche blanche qui précède la note la plus
/// grave à celle qui suit la plus aiguë ; un point plein par note jouée (blanc
/// sur une touche noire), la basse en lettre dessous.
fn diagramme_clavier(f: &mut Feuille, a: &Accord) {
    let t = Corps::MIN;
    let (de, fin) = (blanche_avant(a.clavier[0]), blanche_apres(a.clavier[2]));
    let centres = clavier(f, de, fin, CLAVIER_H, CLAVIER_NOIRE_H);
    for (midi, x, blanche) in centres {
        if !a.clavier.contains(&midi) {
            continue;
        }
        if blanche {
            f.cercle((x, CLAVIER_H - 2.1), 0.85, Some(Couleur::Noir), None);
        } else {
            f.cercle((x, CLAVIER_NOIRE_H - 1.5), 0.65, Some(Couleur::Blanc), None);
        }
        if midi == a.clavier[0] {
            f.texte_centre(a.basse, &t, x, CLAVIER_H + 0.5 + jusqua_la_base(&t));
        }
    }
}

fn hauteur_clavier() -> f32 {
    CLAVIER_H + 0.5 + interligne(&Corps::MIN)
}

/// La palette sur le clavier : une octave à partir de la fondamentale, et une
/// touche blanche de marge de chaque côté.
fn palette_clavier(f: &mut Feuille, a: &Accord) {
    let racine = a.clavier[0];
    let (de, fin) = (blanche_avant(racine), blanche_apres(racine + 11));
    for (midi, x, blanche) in clavier(f, de, fin, PALETTE_CLAVIER_H, PALETTE_NOIRE_H) {
        let Some(role) = (midi >= racine && midi <= racine + 11)
            .then(|| a.role(midi))
            .flatten()
        else {
            continue;
        };
        // Les sept accords du centre n'ont que des notes naturelles : leurs
        // marques sont toutes sur des touches blanches.
        debug_assert!(blanche);
        marque(f, role, (x, PALETTE_CLAVIER_H - 1.8), 0.95);
    }
}

/// Les sept notes de la palette, de la fondamentale vers l'aigu, de note
/// naturelle en note naturelle (les sept accords du centre n'en ont pas d'autre).
fn notes_de_la_portee(a: &Accord) -> [u8; 7] {
    let racine = NATURELLES
        .iter()
        .position(|c| *c == a.clavier[0] % 12)
        .expect("la fondamentale est une note naturelle");
    std::array::from_fn(|i| NATURELLES[(racine + i) % 7])
}

/// La palette sur une portée d'une octave, sans clé : sans clé, la portée ne dit
/// pas la hauteur absolue. La fondamentale est donc toujours dans le premier
/// interligne, et les six autres notes montent de degré en degré, chacune par la
/// forme de son rôle. Le nom de chaque note est dessous, à 11 pt, sur deux lignes
/// alternées : sept noms ne tiennent pas sur une seule.
fn portee(f: &mut Feuille, a: &Accord) {
    let t = Corps::MIN;
    for k in 0..5 {
        let y = k as f32 * ESPACE;
        f.ligne((0.0, y), (LARGEUR_PORTEE, y), TRAIT_FIN);
    }
    for (i, classe) in notes_de_la_portee(a).into_iter().enumerate() {
        let x = PAS_NOTES - JOUR_NOMS / 2.0 + i as f32 * PAS_NOTES;
        // Depuis la ligne du bas : le premier interligne, puis un degré par note.
        let degre = 1 + i;
        let y = 4.0 * ESPACE - degre as f32 * ESPACE / 2.0;
        if let Some(role) = a.role(classe) {
            marque(f, role, (x, y), 0.85);
        }
        let haut = 4.0 * ESPACE + 0.6 + (i % 2) as f32 * interligne(&t);
        f.texte_centre(
            SOLFEGE[usize::from(classe)],
            &t,
            x,
            haut + jusqua_la_base(&t),
        );
    }
}

/// La portée et ses deux lignes de noms.
fn hauteur_portee() -> f32 {
    4.0 * ESPACE + 0.6 + 2.0 * interligne(&Corps::MIN)
}

fn verso(date: &Date) -> Result<(PageDuCahier, Vec<String>), ErreurTemoin> {
    let (t, nom) = (Corps::MIN, corps_nom());
    let il = interligne(&t);
    let entete = couper_aux_points(ENTETE_VERSO, &t, UTILE_L)?;
    let pied_l = couper_aux_points(&pied_de_page(3, date, false), &t, UTILE_L)?;
    let titre_palette = "palette :";
    let fin_max = ACCORDS.iter().map(Accord::fin_de_zone).max().unwrap_or(0);
    let h_palette_manche = RANG_A_VIDE + f32::from(fin_max) * CASE_PALETTE;
    // Chaque nom de note tient dans sa place sous la portée, ou c'est un mot trop long.
    for a in &ACCORDS {
        for classe in notes_de_la_portee(a) {
            let mot = SOLFEGE[usize::from(classe)];
            let c = chasse(mot, &t);
            if c > NOM_DE_NOTE {
                return Err(ErreurTemoin::MotTropLong(MotTropLong {
                    mot: mot.to_string(),
                    chasse: c,
                    largeur: NOM_DE_NOTE,
                }));
            }
        }
    }

    // La mise en page, de haut en bas, commune aux sept colonnes.
    let y_filet = MARGE_V + entete.len() as f32 * il + SOUS_FILET / 2.0;
    let y_nom = y_filet + 1.5 * SOUS_FILET;
    let y_manche = y_nom + interligne(&nom) + APRES_NOM + RANG_A_VIDE;
    let y_clavier = y_manche - RANG_A_VIDE + hauteur_manche() + ENTRE_BLOCS;
    let y_legende = y_clavier + hauteur_clavier() + ENTRE_BLOCS;
    let y_palette = y_legende + 2.0 * SOUS_FILET + il + APRES_NOM + RANG_A_VIDE;
    let y_palette_clavier = y_palette - RANG_A_VIDE + h_palette_manche + ENTRE_BLOCS;
    let y_portee = y_palette_clavier + PALETTE_CLAVIER_H + ENTRE_BLOCS;
    let hauteur = y_portee + hauteur_portee() - MARGE_V + hauteur_pied(&pied_l);
    deborde("le verso de la mineur", hauteur)?;

    let mut f = Feuille::default();
    f.lignes(&entete, MARGE_H, MARGE_V);
    f.filet(y_filet);

    for (col, a) in ACCORDS.iter().enumerate() {
        let x = x_colonne(col);
        let cx = x + COLONNE / 2.0;
        // Le nom à 14 pt et le chiffre à 11 pt, centrés ensemble.
        let ecart = 1.5;
        let l = chasse(a.nom, &nom) + ecart + chasse(a.chiffre, &t);
        let base = y_nom + jusqua_la_base(&nom);
        f.texte(a.nom, &nom, cx - l / 2.0, base);
        f.texte_droite(a.chiffre, &t, cx + l / 2.0, base);

        let x_manche = cx - LARGEUR_MANCHE / 2.0;
        f.groupe("manche", (x_manche, y_manche), |g| manche(g, a));
        let de = blanche_avant(a.clavier[0]);
        let l_clavier = largeur_clavier(de, blanche_apres(a.clavier[2]));
        f.groupe("clavier", (cx - l_clavier / 2.0, y_clavier), |g| {
            diagramme_clavier(g, a);
        });
        f.groupe("manche", (x_manche, y_palette), |g| palette_manche(g, a));
        let racine = a.clavier[0];
        let l_palette = largeur_clavier(blanche_avant(racine), blanche_apres(racine + 11));
        f.groupe("clavier", (cx - l_palette / 2.0, y_palette_clavier), |g| {
            palette_clavier(g, a);
        });
        f.groupe("portee", (x + RETRAIT, y_portee), |g| portee(g, a));
    }

    // La ligne qui sépare le doigté de la palette, et dit ses formes.
    f.filet(y_legende);
    let base = y_legende + SOUS_FILET + jusqua_la_base(&t);
    let mut x = MARGE_H;
    f.texte(titre_palette, &t, x, base);
    x += chasse(titre_palette, &t) + 3.0;
    for (role, mot) in [
        (Role::Appui, "appui"),
        (Role::Tension, "tension"),
        (Role::Frotte, "frotte"),
    ] {
        marque(&mut f, role, (x + 1.0, base - mi_capitale(&t)), 1.0);
        x += DEVANT_TEXTE;
        f.texte(mot, &t, x, base);
        x += chasse(mot, &t) + 5.0;
    }

    pied(&mut f, &pied_l);
    Ok(f.page(Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use noyau::rendu::texte::glyphe_present;

    const DATE: Date = Date {
        annee: 2026,
        mois: 11,
        jour: 1,
    };

    #[test]
    fn chaque_texte_de_la_fiche_a_ses_glyphes() {
        let (_, textes) = fiche(&DATE).unwrap();
        assert!(textes.len() > 100, "{} textes", textes.len());
        for t in &textes {
            assert!(!t.is_empty(), "un texte vide écrirait un tracé vide");
            for c in t.chars() {
                assert!(glyphe_present(c), "{c:?} manque à DejaVu Sans, dans {t:?}");
            }
        }
    }

    #[test]
    fn la_portee_part_de_la_fondamentale_et_monte_de_note_en_note() {
        // la mineur : la si do ré mi fa sol ; si diminué : si do ré mi fa sol la.
        assert_eq!(notes_de_la_portee(&ACCORDS[0]), [9, 11, 0, 2, 4, 5, 7]);
        assert_eq!(notes_de_la_portee(&ACCORDS[1]), [11, 0, 2, 4, 5, 7, 9]);
        assert!(chasse("sol", &Corps::MIN) <= NOM_DE_NOTE);
    }

    #[test]
    fn la_grille_tient_dans_la_largeur_utile() {
        assert!((7.0 * COLONNE + 6.0 * GOUTTIERE - UTILE_L).abs() < 1e-3);
        assert!((8.0 * CASE_SOMBRE + 7.0 * GOUTTIERE - UTILE_L).abs() < 1e-3);
    }

    #[test]
    fn l_en_tete_et_le_pied_de_la_table_tiennent_en_deux_lignes() {
        let t = Corps::MIN;
        let entete = couper_aux_points(ENTETE_TABLE, &t, UTILE_L).unwrap();
        assert_eq!(
            entete,
            [
                "la mineur · table de la tonalité · tonalité choisie · instrument : guitare et clavier",
                "couche sombre : affichée"
            ]
        );
        let pied = couper_aux_points(&pied_de_page(2, &DATE, true), &t, UTILE_L).unwrap();
        assert_eq!(
            pied,
            [
                "page 2 / 3 · rendu le 1er nov. 2026 · empreinte 000000",
                "verso : diagrammes et palettes des accords du centre"
            ]
        );
    }

    #[test]
    fn un_texte_trop_long_pour_la_page_deborde() {
        assert_eq!(deborde("x", 145.0), Ok(()));
        assert!(matches!(
            deborde("x", 145.1),
            Err(ErreurTemoin::Debordement { .. })
        ));
    }
}
