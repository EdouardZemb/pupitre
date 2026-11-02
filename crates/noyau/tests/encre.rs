//! I17 en pixels : chaque paire de valeurs d'une même information se distingue couleur
//! supprimée, rastérisée à 300 ppp, par au moins 2 % des pixels de sa boîte englobante
//! (§ 50 · 6.5).

mod commun;

use commun::difference;
use noyau::clavier::Voicing;
use noyau::guitare::Corde;
use noyau::hauteur::{Lettre, NomDeNote};
use noyau::palette::Role;
use noyau::rendu::Instrument;
use noyau::rendu::case::{Etat, Support, cadre, marque, pastille, pictogramme};
use noyau::rendu::clavier::diagramme_de_clavier;
use noyau::rendu::manche::{marque_de_corde, trait_de_barre};
use noyau::rendu::pages::Rect;
use noyau::rendu::svg::{Couleur, Element, Page};
use noyau::rendu::table::page_table;
use noyau::rendu::texte::{Corps, chasse, composer};
use noyau::table::{CoucheSombre, Famille, Fonction, Rangee, table};
use noyau::tonalite::{Mode, Tonalite};

const SEUIL: f32 = 0.02;
const FAMILLES: [Famille; 4] = [
    Famille::Diatonique,
    Famille::DominanteSecondaire,
    Famille::Emprunt,
    Famille::Sombre,
];
/// Le blanc autour d'un élément, sur sa petite page.
const MARGE: f32 = 5.0;

/// Une petite page, juste assez grande pour l'élément : la différence ne se mesure que dans
/// la boîte de ses pixels noirs.
fn svg(largeur: f32, hauteur: f32, elements: Vec<Element>) -> String {
    Page {
        largeur,
        hauteur,
        elements,
    }
    .svg()
}

/// Chaque taille de case que prend une famille sur les 48 pages de table.
fn tailles() -> Vec<(Famille, f32, f32)> {
    let mut tailles: Vec<(Famille, f32, f32)> = Vec::new();
    for t in Tonalite::les_24() {
        for couche in [CoucheSombre::Masquee, CoucheSombre::Affichee] {
            let tb = table(t, couche);
            let p = page_table(&tb, Instrument::default(), None).unwrap();
            for (pos, r) in &p.disposition.cases {
                let f = tb.case(*pos).unwrap().famille;
                let deja = tailles
                    .iter()
                    .any(|(g, l, h)| *g == f && (l - r.l).abs() < 1e-3 && (h - r.h).abs() < 1e-3);
                if !deja {
                    tailles.push((f, r.l, r.h));
                }
            }
        }
    }
    tailles
}

/// Le même contenu dans chaque cadre : un chiffre, un nom, une ligne d'effet, centrés.
fn contenu(r: &Rect) -> Vec<Element> {
    let (t, nom) = (Corps::MIN, Corps::pt(14.0).unwrap());
    let cx = r.x + r.l / 2.0;
    let centre = |texte: &str, corps: &Corps, base: f32| {
        composer(texte, corps, cx - chasse(texte, corps) / 2.0, base)
    };
    vec![
        centre("V7/V", &t, r.y + 5.0),
        centre("D7", &nom, r.y + 10.5),
        centre("tire vers G", &t, r.y + 15.0),
    ]
}

/// Une case de la famille, à la taille donnée, avec son contenu.
fn case(famille: Famille, l: f32, h: f32, etat: Etat) -> String {
    let r = Rect {
        x: MARGE,
        y: MARGE,
        l,
        h,
    };
    let mut elements = vec![cadre(famille, r, etat)];
    elements.extend(contenu(&r));
    svg(l + 2.0 * MARGE, h + 2.0 * MARGE, elements)
}

#[test]
fn i17_les_quatre_familles() {
    // Chaque paire se compare aux tailles que prennent les cases de ses deux familles.
    let tailles = tailles();
    for (i, a) in FAMILLES.iter().enumerate() {
        for b in &FAMILLES[i + 1..] {
            let mut comparees = 0;
            for (_, l, h) in tailles.iter().filter(|(f, ..)| f == a || f == b) {
                let d = difference(
                    &case(*a, *l, *h, Etat::Ordinaire),
                    &case(*b, *l, *h, Etat::Ordinaire),
                );
                assert!(d >= SEUIL, "{a:?} et {b:?}, case de {l} × {h} mm : {d}");
                comparees += 1;
            }
            assert!(
                comparees >= 2,
                "{a:?} et {b:?} : une taille de chacune au moins"
            );
        }
    }
}

#[test]
fn i17_les_trois_fonctions() {
    let fonctions = [Fonction::Repos, Fonction::Eloignement, Fonction::Tension];
    let page = |f: Fonction| svg(10.0, 10.0, vec![pastille(f, 5.0, 5.0)]);
    for (i, a) in fonctions.iter().enumerate() {
        for b in &fonctions[i + 1..] {
            let d = difference(&page(*a), &page(*b));
            assert!(d >= SEUIL, "{a:?} et {b:?} : {d}");
        }
    }
}

#[test]
fn i17_les_pictogrammes_de_la_legende() {
    // Un défaut d'H09 : aux coins de 1,05 mm, l'emprunt ne se distinguait de l'harmonie
    // sombre qu'en lisant son nom.
    let page = |f: Famille| svg(20.0, 15.0, vec![pictogramme(f, MARGE, MARGE)]);
    for (i, a) in FAMILLES.iter().enumerate() {
        for b in &FAMILLES[i + 1..] {
            let d = difference(&page(*a), &page(*b));
            assert!(d >= SEUIL, "{a:?} et {b:?} : {d}");
        }
    }
}

/// La boîte de vue d'une page de table, que `recadrer` remplace.
const VUE: &str = r#"width="209.5mm" height="157mm" viewBox="0 0 209.5 157""#;

/// La page, recadrée sur un rectangle et ses abords : le rendu réel d'une case.
fn recadrer(svg: &str, r: &Rect, marge: f32) -> String {
    assert!(svg.contains(VUE), "une page de table entière");
    let (x, y, l, h) = (
        r.x - marge,
        r.y - marge,
        r.l + 2.0 * marge,
        r.h + 2.0 * marge,
    );
    svg.replacen(
        VUE,
        &format!(r#"width="{l}mm" height="{h}mm" viewBox="{x} {y} {l} {h}""#),
        1,
    )
}

#[test]
fn i17_la_case_cible() {
    // La case cible, telle que la page la rend (cadre plein, texte et pastille en blanc),
    // contre la même case ordinaire : une case de chaque rangée, en la mineur, en do majeur,
    // et en ré majeur (dont les dominantes secondaires ont deux lignes d'effet).
    for (lettre, mode) in [
        (Lettre::A, Mode::Mineur),
        (Lettre::C, Mode::Majeur),
        (Lettre::D, Mode::Majeur),
    ] {
        let t = Tonalite {
            tonique: NomDeNote {
                lettre,
                alteration: 0,
            },
            mode,
        };
        let tb = table(t, CoucheSombre::Affichee);
        let ordinaire = page_table(&tb, Instrument::default(), None).unwrap();
        let svg_ordinaire = ordinaire.page.svg();
        for rangee in [
            Rangee::Dessus,
            Rangee::Centre,
            Rangee::Sombre,
            Rangee::Dessous,
        ] {
            let pos = tb.rangee(rangee)[0].position;
            let r = ordinaire
                .disposition
                .cases
                .iter()
                .find(|(q, _)| *q == pos)
                .unwrap()
                .1;
            let cible = page_table(&tb, Instrument::default(), Some(pos)).unwrap();
            assert_eq!(
                cible.disposition, ordinaire.disposition,
                "la cible ne bouge rien"
            );
            let d = difference(
                &recadrer(&cible.page.svg(), &r, 0.5),
                &recadrer(&svg_ordinaire, &r, 0.5),
            );
            assert!(d >= SEUIL, "{} {pos:?} : {d}", t.nom());
        }
    }
}

#[test]
fn i17_les_marques_du_manche() {
    // Chaque valeur, posée sur deux cordes voisines du diagramme (3,7 mm) : deux doigts, deux
    // cordes à vide ou étouffées, contre l'index qui barre les deux cordes.
    let ecart = 3.7;
    let page = |valeur: &Option<Corde>| {
        let (a, b, y) = (MARGE, MARGE + ecart, MARGE);
        let elements = match valeur {
            Some(c) => vec![marque_de_corde(c, (a, y)), marque_de_corde(c, (b, y))],
            None => vec![trait_de_barre(a, b, y)],
        };
        svg(ecart + 2.0 * MARGE, 2.0 * MARGE, elements)
    };
    let valeurs = [
        ("doigt", Some(Corde::Frette(1))),
        ("corde à vide", Some(Corde::AVide)),
        ("corde étouffée", Some(Corde::Etouffee)),
        ("barré", None),
    ];
    for (i, (a, va)) in valeurs.iter().enumerate() {
        for (b, vb) in &valeurs[i + 1..] {
            let d = difference(&page(va), &page(vb));
            assert!(d >= SEUIL, "{a} et {b} : {d}");
        }
    }
}

/// Les touches d'un clavier dessiné, de gauche à droite, en coordonnées de la page.
fn touches(clavier: &Element, id: &str) -> Vec<Rect> {
    let Element::Groupe {
        translation: (ox, oy),
        enfants,
        ..
    } = clavier
    else {
        unreachable!("un clavier est un groupe")
    };
    let mut touches: Vec<Rect> = enfants
        .iter()
        .filter_map(|e| match e {
            Element::Rect { r, id: Some(i), .. } if *i == id => Some(Rect {
                x: r.x + ox,
                y: r.y + oy,
                ..*r
            }),
            _ => None,
        })
        .collect();
    touches.sort_by(|a, b| a.x.total_cmp(&b.x));
    touches
}

/// Le dessin vu par une seule touche : la page réduite à son rectangle.
fn sur_la_touche(clavier: &Element, r: &Rect) -> String {
    svg(
        r.l,
        r.h,
        vec![Element::Groupe {
            id: None,
            translation: (-r.x, -r.y),
            echelle: None,
            enfants: vec![clavier.clone()],
        }],
    )
}

#[test]
fn i17_le_clavier() {
    // Do majeur et do mineur couvrent les mêmes touches, de si 2 à la 3 : mi 3 est jouée dans
    // l'un et libre dans l'autre, mi♭ 3 l'inverse.
    let do_ = NomDeNote {
        lettre: Lettre::C,
        alteration: 0,
    };
    let dessin = |notes: &[u8]| {
        diagramme_de_clavier(
            &Voicing {
                notes: notes.to_vec(),
            },
            do_,
            (MARGE, MARGE),
        )
    };
    let (majeur, mineur) = (dessin(&[60, 64, 67]), dessin(&[60, 63, 67]));
    assert_eq!(
        touches(&majeur, "touche-blanche"),
        touches(&mineur, "touche-blanche")
    );
    // Les blanches : si, do, ré, mi… ; les noires : do♯, ré♯…
    let mi = touches(&majeur, "touche-blanche")[3];
    let mi_bemol = touches(&majeur, "touche-noire")[1];
    for (touche, r) in [("blanche", mi), ("noire", mi_bemol)] {
        let d = difference(&sur_la_touche(&majeur, &r), &sur_la_touche(&mineur, &r));
        assert!(d >= SEUIL, "touche {touche} jouée et libre : {d}");
    }
}

#[test]
fn i17_les_roles_de_la_palette() {
    // Appui, tension et frotte, sur chacun de leurs supports : sur une touche noire, en blanc.
    let roles = [Role::Appui, Role::Tension, Role::Frotte];
    let supports = [
        Support::Portee,
        Support::Manche,
        Support::ToucheBlanche,
        Support::ToucheNoire,
    ];
    for s in supports {
        let page = |r: Role| {
            let mut elements = Vec::new();
            if s == Support::ToucheNoire {
                // Une touche noire de 1,8 mm, sous la marque.
                elements.push(Element::Rect {
                    r: Rect {
                        x: 2.1,
                        y: 2.1,
                        l: 1.8,
                        h: 1.8,
                    },
                    id: None,
                    plein: Some(Couleur::Noir),
                    trait_: None,
                });
            }
            elements.push(marque(r, (3.0, 3.0), s));
            svg(6.0, 6.0, elements)
        };
        for (i, a) in roles.iter().enumerate() {
            for b in &roles[i + 1..] {
                let d = difference(&page(*a), &page(*b));
                assert!(d >= SEUIL, "{s:?}, {a:?} et {b:?} : {d}");
            }
        }
    }
}
