//! Le verso de la table et la fiche d'un accord : ils tiennent pour les 24 tonalités et les
//! trois instruments, gardent les minimums physiques, et dessinent ce que disent REN-04,
//! REN-08 et MIC-13.

mod commun;

use noyau::accord::{Accord, Qualite};
use noyau::clavier::hors_contexte;
use noyau::guitare::{Doigte, PAS_DE_DOIGTE};
use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::palette::palette;
use noyau::rendu::Instrument;
use noyau::rendu::clavier::diagramme_de_clavier;
use noyau::rendu::manche::diagramme_de_manche;
use noyau::rendu::pages::{HAUTEUR, LARGEUR, Rect};
use noyau::rendu::sous_ensemble::verifier;
use noyau::rendu::svg::{Couleur, Element, Page};
use noyau::rendu::table::page_table;
use noyau::rendu::texte::{Corps, chasse, composer};
use noyau::rendu::verso::{colonne_d_accord, fiche_d_accord, page_verso};
use noyau::table::{Case, CoucheSombre, Rangee, Table, table};
use noyau::tonalite::{Mode, Tonalite};

const INSTRUMENTS: [Instrument; 3] = [
    Instrument::Guitare,
    Instrument::Clavier,
    Instrument::LesDeux,
];

/// La précision des nombres écrits dans le SVG, et un peu plus.
const EPS: f32 = 2e-3;

fn note(lettre: Lettre, alteration: i8) -> NomDeNote {
    NomDeNote { lettre, alteration }
}

fn tonalite(lettre: Lettre, alteration: i8, mode: Mode) -> Tonalite {
    Tonalite {
        tonique: note(lettre, alteration),
        mode,
    }
}

fn accord(degres: i8, demi_tons: i8, qualite: Qualite) -> Accord {
    Accord {
        fondamentale: Intervalle::new(degres, demi_tons),
        qualite,
        renversement: 0,
    }
}

fn verso(t: Tonalite, i: Instrument) -> Page {
    page_verso(&table(t, CoucheSombre::Masquee), i)
        .unwrap_or_else(|e| panic!("{}, {} : {e}", t.nom(), i.nom()))
}

/// Le tracé d'un texte composé, tel que la page l'écrit : `d="…"`, quelle que soit sa place.
fn trace(texte: &str, corps: &Corps) -> String {
    let Element::Groupe { enfants, .. } = composer(texte, corps, 0.0, 0.0) else {
        unreachable!("un texte est un groupe")
    };
    let Element::Chemin { d, .. } = &enfants[0] else {
        unreachable!("un texte est un tracé")
    };
    format!(r#"d="{d}""#)
}

/// Vrai si la page compose le texte sur des lignes coupées à l'un des séparateurs, quelle que
/// soit la coupe.
fn compose_en_lignes(svg: &str, texte: &str, separateur: &str) -> bool {
    let segments: Vec<&str> = texte.split(separateur).collect();
    let coupes = segments.len() - 1;
    (0..1_u32 << coupes).any(|masque| {
        let mut lignes = vec![segments[0].to_string()];
        for (k, s) in segments[1..].iter().enumerate() {
            if masque & (1 << k) != 0 {
                lignes.push(s.to_string());
            } else {
                let derniere = lignes.last_mut().expect("une ligne au moins");
                derniere.push_str(separateur);
                derniere.push_str(s);
            }
        }
        lignes.iter().all(|l| svg.contains(&trace(l, &Corps::MIN)))
    })
}

/// Un point de la page, en mm.
type Point = (f32, f32);

/// Un cercle relevé : le groupe nommé qui l'entoure, son identifiant, son centre, son
/// remplissage et la couleur de son trait.
struct Cercle {
    dans: Option<&'static str>,
    id: Option<&'static str>,
    centre: Point,
    plein: Option<Couleur>,
    trait_: Option<Couleur>,
}

/// Ce qu'on lit d'un dessin, ses groupes traversés, en coordonnées de la page. Chaque élément
/// garde le groupe nommé le plus proche qui l'entoure (`dans`), un texte excepté.
#[derive(Default)]
struct Releve {
    /// Les rectangles, avec leur identifiant et leur remplissage.
    rects: Vec<(Option<&'static str>, Rect, Option<Couleur>)>,
    cercles: Vec<Cercle>,
    /// Les textes : le groupe nommé qui les entoure, le tracé, l'origine de la ligne de base.
    textes: Vec<(Option<&'static str>, String, Point)>,
    /// Les tracés hors des textes : le groupe nommé qui les entoure, les nombres de leur `d`, dans
    /// le repère de ce groupe, et l'origine de ce repère.
    chemins: Vec<(Option<&'static str>, Vec<f32>, Point)>,
    /// Les traits : leurs deux bouts.
    lignes: Vec<(Point, Point)>,
    /// Les groupes nommés : leur identifiant et leur origine.
    groupes: Vec<(&'static str, Point)>,
}

/// Les nombres d'un tracé, dans l'ordre : `M0 1H2A…` donne 0, 1, 2…
fn nombres(d: &str) -> Vec<f32> {
    d.split(|c: char| c.is_ascii_alphabetic() || c == ' ')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().expect("un nombre"))
        .collect()
}

fn relever(e: &Element, origine: (f32, f32), dans: Option<&'static str>, r: &mut Releve) {
    let place = |(x, y): (f32, f32)| (x + origine.0, y + origine.1);
    match e {
        Element::Groupe {
            id,
            translation,
            enfants,
            ..
        } => {
            let o = place(*translation);
            if *id == Some("t") {
                if let Element::Chemin { d, .. } = &enfants[0] {
                    r.textes.push((dans, format!(r#"d="{d}""#), o));
                }
                return;
            }
            if let Some(id) = id {
                r.groupes.push((id, o));
            }
            for enfant in enfants {
                relever(enfant, o, id.or(dans), r);
            }
        }
        Element::Rect {
            r: rect, id, plein, ..
        } => r.rects.push((
            *id,
            Rect {
                x: rect.x + origine.0,
                y: rect.y + origine.1,
                ..*rect
            },
            *plein,
        )),
        Element::Cercle {
            cx,
            cy,
            id,
            plein,
            trait_,
            ..
        } => r.cercles.push(Cercle {
            dans,
            id: *id,
            centre: place((*cx, *cy)),
            plein: *plein,
            trait_: trait_.map(|(c, _)| c),
        }),
        Element::Chemin { d, .. } => r.chemins.push((dans, nombres(d), origine)),
        Element::Ligne { de, a, .. } => r.lignes.push((place(*de), place(*a))),
    }
}

fn releve(e: &Element) -> Releve {
    let mut r = Releve::default();
    relever(e, (0.0, 0.0), None, &mut r);
    r
}

fn contient(r: &Rect, (x, y): (f32, f32)) -> bool {
    x > r.x && x < r.x + r.l && y > r.y && y < r.y + r.h
}

#[test]
fn tous_les_versos_tiennent() {
    for t in Tonalite::les_24() {
        for i in INSTRUMENTS {
            let p = verso(t, i);
            let svg = p.svg();
            assert_eq!(verifier(&svg), Ok(()), "{}, {}", t.nom(), i.nom());
            assert_eq!((p.largeur, p.hauteur), (LARGEUR, HAUTEUR));
        }
    }
}

#[test]
fn les_minimums_du_verso() {
    for t in Tonalite::les_24() {
        for i in INSTRUMENTS {
            let nom = format!("{}, {}", t.nom(), i.nom());
            let m = commun::mesurer(&verso(t, i).svg());
            assert!(m.corps_min_pt >= 11.0, "{nom} : corps {}", m.corps_min_pt);
            assert!(m.trait_min_pt >= 0.25, "{nom} : trait {}", m.trait_min_pt);
            // Un diagramme de manche et une palette sur le manche par accord du centre.
            let manches = match i {
                Instrument::Clavier => 0,
                Instrument::Guitare | Instrument::LesDeux => 14,
            };
            assert_eq!(m.manches.len(), manches, "{nom} : {:?}", m.manches);
            assert!(
                m.manches.iter().all(|l| *l >= 18.0),
                "{nom} : {:?}",
                m.manches
            );
            assert_eq!(
                m.touches_blanches.is_empty(),
                i == Instrument::Guitare,
                "{nom} : les claviers suivent l'instrument"
            );
            assert!(
                m.touches_blanches.iter().all(|l| *l >= 3.0),
                "{nom} : {:?}",
                m.touches_blanches
            );
        }
    }
}

#[test]
fn l_en_tete_et_les_colonnes_du_verso() {
    let la_mineur = tonalite(Lettre::A, 0, Mode::Mineur);
    let svg = verso(la_mineur, Instrument::LesDeux).svg();
    assert!(
        compose_en_lignes(
            &svg,
            "la mineur · verso de la table · diagrammes et palettes des accords du centre",
            " · "
        ),
        "l'en-tête du verso"
    );
    // Chaque colonne porte le nom et le chiffre de son accord, dans l'ordre du centre.
    let nom = Corps::pt(14.0).unwrap();
    for (accord, chiffre) in [
        ("Am", "i"),
        ("B°", "ii°"),
        ("C", "III"),
        ("Dm", "iv"),
        ("Em", "v"),
        ("F", "VI"),
        ("G", "VII"),
    ] {
        assert!(svg.contains(&trace(accord, &nom)), "{accord}");
        assert!(svg.contains(&trace(chiffre, &Corps::MIN)), "{chiffre}");
    }
}

#[test]
fn les_colonnes_du_verso_s_alignent_sur_la_table() {
    // La palette sur le clavier (neuf touches blanches) se centre dans sa colonne, et chaque
    // colonne est celle de la case de son accord sur la page de la table.
    for t in [
        tonalite(Lettre::A, 0, Mode::Mineur),
        tonalite(Lettre::F, 1, Mode::Majeur),
    ] {
        let tb = table(t, CoucheSombre::Masquee);
        let p = page_table(&tb, Instrument::LesDeux, None).unwrap();
        let verso = page_verso(&tb, Instrument::LesDeux).unwrap();
        let colonnes: Vec<&Element> = verso
            .elements
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Element::Groupe {
                        id: Some("colonne"),
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(colonnes.len(), 7, "{}", t.nom());
        for (case, colonne) in tb.rangee(Rangee::Centre).iter().zip(colonnes) {
            let r = p
                .disposition
                .cases
                .iter()
                .find(|(q, _)| *q == case.position)
                .unwrap()
                .1;
            let releve = releve(colonne);
            let palette = releve
                .groupes
                .iter()
                .filter(|(id, _)| *id == "clavier")
                .nth(1)
                .expect("le diagramme de clavier, puis la palette")
                .1;
            let touches: Vec<&Rect> = releve
                .rects
                .iter()
                .filter(|(id, r, _)| *id == Some("touche-blanche") && r.y >= palette.1 - EPS)
                .map(|(_, r, _)| r)
                .collect();
            assert_eq!(touches.len(), 9, "{} : {}", t.nom(), tb.nom(case));
            let gauche = touches.iter().map(|r| r.x).fold(f32::INFINITY, f32::min);
            let droite = touches.iter().map(|r| r.x + r.l).fold(0.0, f32::max);
            let milieu = (gauche + droite) / 2.0;
            assert!(
                (milieu - (r.x + r.l / 2.0)).abs() <= EPS,
                "{} : {} centré en {milieu}, sa case en {}",
                t.nom(),
                tb.nom(case),
                r.x + r.l / 2.0
            );
        }
    }
}

#[test]
fn ren08_1_do_marque_do_mi_sol() {
    // `C` dans la table, hors contexte : son voicing et sa basse, tels que la colonne les prend.
    let do_majeur = tonalite(Lettre::C, 0, Mode::Majeur);
    let c = accord(0, 0, Qualite::Majeur);
    let basse = c.noms_des_notes(do_majeur)[usize::from(c.renversement)];
    let e = diagramme_de_clavier(&hors_contexte(&c, do_majeur), basse, (0.0, 0.0));
    let r = releve(&e);
    let mut blanches: Vec<Rect> = r
        .rects
        .iter()
        .filter(|(id, ..)| *id == Some("touche-blanche"))
        .map(|(_, r, _)| *r)
        .collect();
    let mut noires: Vec<Rect> = r
        .rects
        .iter()
        .filter(|(id, ..)| *id == Some("touche-noire"))
        .map(|(_, r, _)| *r)
        .collect();
    blanches.sort_by(|a, b| a.x.total_cmp(&b.x));
    noires.sort_by(|a, b| a.x.total_cmp(&b.x));
    // De la touche blanche qui précède do 3 (si 2, MIDI 59) à celle qui suit sol 3 (la 3, 69),
    // les noires entre les deux.
    let midi_blanches = [59, 60, 62, 64, 65, 67, 69];
    let midi_noires = [61, 63, 66, 68];
    assert_eq!(blanches.len(), midi_blanches.len(), "{blanches:?}");
    assert_eq!(noires.len(), midi_noires.len(), "{noires:?}");
    let mut marquees: Vec<u8> = r
        .cercles
        .iter()
        .map(|cercle| {
            let centre = cercle.centre;
            let noire = noires.iter().position(|t| contient(t, centre));
            let blanche = blanches.iter().position(|t| contient(t, centre));
            match (noire, blanche) {
                (Some(k), _) => midi_noires[k],
                (None, Some(k)) => midi_blanches[k],
                (None, None) => panic!("un point hors des touches : {centre:?}"),
            }
        })
        .collect();
    marquees.sort_unstable();
    assert_eq!(marquees, [60, 64, 67]);
    // La basse, « C », sous sa touche.
    let do_3 = blanches[1];
    let (_, _, base) = r
        .textes
        .iter()
        .find(|(_, d, _)| *d == trace("C", &Corps::MIN))
        .expect("la basse est écrite");
    let milieu = base.0 + chasse("C", &Corps::MIN) / 2.0;
    assert!((milieu - (do_3.x + do_3.l / 2.0)).abs() <= EPS, "{milieu}");
    assert!(base.1 > do_3.y + do_3.h, "la lettre est sous le clavier");
}

#[test]
fn ren04_le_diagramme_de_fa_barre() {
    // F (133211) : l'index barre la première frette de la corde grave à l'aiguë ; les doigts 3,
    // 4 et 2 tiennent les frettes 3, 3 et 2 ; les numéros 1 3 4 2 1 1 sont sous leurs cordes.
    let d = Doigte::depuis("133211").unwrap();
    let notes = [note(Lettre::F, 0), note(Lettre::A, 0), note(Lettre::C, 0)];
    let r = releve(&diagramme_de_manche(Some(&d), &notes, (0.0, 0.0)));
    // La grille, lue dans le dessin : les cordes sont les traits verticaux, les frettes les
    // horizontaux, le sillet en tête.
    let mut cordes: Vec<f32> = r
        .lignes
        .iter()
        .filter(|(de, a)| (de.0 - a.0).abs() < EPS)
        .map(|(de, _)| de.0)
        .collect();
    let mut frettes: Vec<f32> = r
        .lignes
        .iter()
        .filter(|(de, a)| (de.1 - a.1).abs() < EPS)
        .map(|(de, _)| de.1)
        .collect();
    cordes.sort_by(f32::total_cmp);
    frettes.sort_by(f32::total_cmp);
    assert_eq!(
        (cordes.len(), frettes.len()),
        (6, 5),
        "six cordes, quatre frettes"
    );
    let milieu = |f: usize| (frettes[f - 1] + frettes[f]) / 2.0;
    // Le barré : un seul tracé, de la corde 0 à la corde 5, au milieu de la première frette.
    let barres: Vec<&Vec<f32>> = r
        .chemins
        .iter()
        .filter(|(dans, ..)| *dans == Some("barre"))
        .map(|(_, n, _)| n)
        .collect();
    assert_eq!(barres.len(), 1, "un barré");
    // M x0 haut H x1 A r r 0 0 1 x1 bas H x0 …
    let b = barres[0];
    let (de, a, y) = (b[0], b[2], (b[1] + b[9]) / 2.0);
    assert!(
        (de - cordes[0]).abs() < EPS && (a - cordes[5]).abs() < EPS,
        "{de} à {a}"
    );
    assert!(
        (y - milieu(1)).abs() < EPS,
        "le barré en {y}, la frette 1 en {}",
        milieu(1)
    );
    // Les doigts hors du barré, et aucun point sous lui.
    let mut points: Vec<(usize, usize)> = r
        .cercles
        .iter()
        .filter(|c| c.id == Some("point"))
        .map(|c| {
            let corde = cordes.iter().position(|x| (x - c.centre.0).abs() < EPS);
            let frette = (1..frettes.len()).find(|f| (milieu(*f) - c.centre.1).abs() < EPS);
            (
                corde.expect("sur une corde"),
                frette.expect("dans une case"),
            )
        })
        .collect();
    points.sort_unstable();
    assert_eq!(points, [(1, 3), (2, 3), (3, 2)]);
    // Les numéros de doigts, chacun centré sous sa corde, sous la grille.
    let mut doigts: Vec<(f32, &String, f32)> = r
        .textes
        .iter()
        .filter(|(dans, ..)| *dans == Some("doigts"))
        .map(|(_, d, base)| (base.0, d, base.1))
        .collect();
    doigts.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(doigts.len(), 6);
    let attendus = ["1", "3", "4", "2", "1", "1"];
    for ((x, d, base), (chiffre, corde)) in doigts.iter().zip(attendus.iter().zip(&cordes)) {
        assert_eq!(**d, trace(chiffre, &Corps::MIN), "le doigt {chiffre}");
        let centre = x + chasse(chiffre, &Corps::MIN) / 2.0;
        assert!(
            (centre - corde).abs() < EPS,
            "{chiffre} en {centre}, sa corde en {corde}"
        );
        assert!(*base > frettes[4], "sous la grille");
    }
}

#[test]
fn r11_la_fondamentale_est_a_la_meme_place_dans_chaque_colonne() {
    // Sans clé, la portée se lit depuis sa note la plus grave : dans un verso, la fondamentale
    // (le premier rond de chaque portée) est au même point de chaque colonne, en abscisse comme
    // en hauteur. Dans une fiche à neuf notes, elle est au même interligne ; son abscisse suit le
    // pas de ses neuf notes, et une fiche n'a qu'une colonne.
    let premier_rond = |colonne: &Element| {
        let r = releve(colonne);
        let (_, portee) = *r
            .groupes
            .iter()
            .find(|(id, _)| *id == "portee")
            .expect("une portée par colonne");
        let rond = r
            .cercles
            .iter()
            .find(|c| c.dans == Some("portee"))
            .expect("une note au moins")
            .centre;
        (rond, (rond.0 - portee.0, rond.1 - portee.1))
    };
    let colonnes = |p: &Page| -> Vec<Element> {
        p.elements
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Element::Groupe {
                        id: Some("colonne"),
                        ..
                    }
                )
            })
            .cloned()
            .collect()
    };
    let mut sous_la_portee = Vec::new();
    for t in [
        tonalite(Lettre::A, 0, Mode::Mineur),
        tonalite(Lettre::F, 1, Mode::Majeur),
        tonalite(Lettre::E, -1, Mode::Mineur),
    ] {
        let tb = table(t, CoucheSombre::Masquee);
        let cases = page_table(&tb, Instrument::LesDeux, None)
            .unwrap()
            .disposition
            .cases;
        let p = page_verso(&tb, Instrument::LesDeux).unwrap();
        let places: Vec<(f32, f32)> = tb
            .rangee(Rangee::Centre)
            .iter()
            .zip(colonnes(&p))
            .map(|(case, colonne)| {
                let r = cases.iter().find(|(q, _)| *q == case.position).unwrap().1;
                let ((x, y), relatif) = premier_rond(&colonne);
                sous_la_portee.push(relatif.1);
                (x - r.x, y)
            })
            .collect();
        assert_eq!(places.len(), 7, "{}", t.nom());
        for (x, y) in &places {
            assert!(
                (x - places[0].0).abs() < EPS && (y - places[0].1).abs() < EPS,
                "{} : {places:?}",
                t.nom()
            );
        }
    }
    let do_majeur = tonalite(Lettre::C, 0, Mode::Majeur);
    let tb = table(do_majeur, CoucheSombre::Affichee);
    let c = tb.cases.iter().find(|c| tb.nom(c) == "C♯°7").unwrap();
    let fiche = fiche_d_accord(&c.accord, do_majeur, Instrument::LesDeux).unwrap();
    let (_, (_, y)) = premier_rond(&colonnes(&fiche)[0]);
    sous_la_portee.push(y);
    for y in &sous_la_portee {
        assert!((y - sous_la_portee[0]).abs() < EPS, "{sous_la_portee:?}");
    }
}

#[test]
fn un_appui_sur_une_touche_noire_est_un_disque_blanc() {
    // A7 en do majeur : sa palette sur le clavier part de sol, la blanche qui précède la ; ses
    // noires sont sol♯, la♯, do♯… Do♯, la tierce, est un appui : un disque blanc, sans trait.
    let do_majeur = tonalite(Lettre::C, 0, Mode::Majeur);
    let la_7 = accord(5, 9, Qualite::Septieme);
    let fiche = fiche_d_accord(&la_7, do_majeur, Instrument::Clavier).unwrap();
    let colonne = fiche
        .elements
        .iter()
        .find(|e| {
            matches!(
                e,
                Element::Groupe {
                    id: Some("colonne"),
                    ..
                }
            )
        })
        .unwrap();
    let r = releve(colonne);
    let palette = r
        .groupes
        .iter()
        .filter(|(id, _)| *id == "clavier")
        .nth(1)
        .expect("le diagramme, puis la palette")
        .1;
    let mut noires: Vec<Rect> = r
        .rects
        .iter()
        .filter(|(id, t, _)| *id == Some("touche-noire") && t.y >= palette.1 - EPS)
        .map(|(_, t, _)| *t)
        .collect();
    noires.sort_by(|a, b| a.x.total_cmp(&b.x));
    let do_diese = noires[2];
    let marques: Vec<&Cercle> = r
        .cercles
        .iter()
        .filter(|c| contient(&do_diese, c.centre))
        .collect();
    assert_eq!(marques.len(), 1, "une marque sur do♯");
    assert_eq!(
        (marques[0].plein, marques[0].trait_),
        (Some(Couleur::Blanc), None),
        "un disque blanc"
    );
    let croix = r
        .lignes
        .iter()
        .any(|(de, a)| contient(&do_diese, *de) || contient(&do_diese, *a));
    assert!(!croix, "sans croix");
}

#[test]
fn un_accord_injouable_dit_pas_de_doigte() {
    // Tous les accords des tables ont un doigté : on appelle le diagramme sans doigté.
    let notes = [note(Lettre::A, 0), note(Lettre::C, 1), note(Lettre::E, 0)];
    let e = diagramme_de_manche(None, &notes, (5.0, 5.0));
    let svg = Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements: vec![e],
    }
    .svg();
    assert_eq!(verifier(&svg), Ok(()));
    assert!(
        compose_en_lignes(&svg, PAS_DE_DOIGTE, " "),
        "{PAS_DE_DOIGTE}"
    );
    assert!(compose_en_lignes(&svg, "la do♯ mi", " "), "les notes");
    assert!(!svg.contains("<line"), "ni corde ni frette");
    assert!(!svg.contains(r#"id="manche-"#), "aucune grille de manche");
}

/// Les fiches dont on vérifie le SVG : dans une tonalité majeure sans altération et une mineure
/// à bémols, la première case de chaque famille et chaque case à neuf notes.
fn a_verifier(t: Tonalite, tb: &Table, c: &Case) -> bool {
    let echantillon = [
        tonalite(Lettre::C, 0, Mode::Majeur),
        tonalite(Lettre::E, -1, Mode::Mineur),
    ];
    let premiere = tb
        .cases
        .iter()
        .find(|d| d.famille == c.famille)
        .map(|d| d.position);
    echantillon.contains(&t) && (premiere == Some(c.position) || palette(&c.accord, t).len() == 9)
}

#[test]
fn toute_case_a_sa_fiche() {
    // La table affichée porte toutes les cases de la table masquée, et la couche sombre. Chaque
    // fiche est rendue ; le SVG d'un échantillon, qui passe par chaque famille, chaque
    // instrument et les palettes de neuf notes, est vérifié. Les 1 944 fiches demandent une
    // minute en debug sur un seul fil : une tonalité par fil.
    let verifiees = std::thread::scope(|fils| {
        let fils: Vec<_> = Tonalite::les_24()
            .into_iter()
            .map(|t| {
                fils.spawn(move || {
                    let tb = table(t, CoucheSombre::Affichee);
                    let mut verifiees = 0;
                    for c in &tb.cases {
                        for i in INSTRUMENTS {
                            let nom = format!("{} en {}, {}", tb.nom(c), t.nom(), i.nom());
                            let p = fiche_d_accord(&c.accord, t, i)
                                .unwrap_or_else(|e| panic!("{nom} : {e}"));
                            if a_verifier(t, &tb, c) {
                                assert_eq!(verifier(&p.svg()), Ok(()), "{nom}");
                                verifiees += 1;
                            }
                        }
                    }
                    verifiees
                })
            })
            .collect();
        fils.into_iter().map(|f| f.join().unwrap()).sum::<usize>()
    });
    // Quatre familles et les cases à neuf notes, dans deux tonalités, par trois instruments.
    assert!(verifiees >= 2 * 4 * 3, "{verifiees} fiches vérifiées");
}

#[test]
fn la_fiche_dit_l_accord_son_chiffre_et_l_instrument() {
    let do_majeur = tonalite(Lettre::C, 0, Mode::Majeur);
    let la_7 = Accord {
        fondamentale: Intervalle::new(5, 9),
        qualite: Qualite::Septieme,
        renversement: 0,
    };
    let p = fiche_d_accord(&la_7, do_majeur, Instrument::LesDeux).unwrap();
    let svg = p.svg();
    assert!(
        compose_en_lignes(
            &svg,
            "A7 · V7/ii en do majeur · instrument : guitare et clavier",
            " · "
        ),
        "l'en-tête de la fiche"
    );
    // Une colonne seule, ses minimums gardés ; sa palette a huit notes (do et do♯).
    let m = commun::mesurer(&svg);
    assert_eq!(m.manches.len(), 2);
    assert!(m.manches.iter().all(|l| *l >= 18.0), "{:?}", m.manches);
    assert!(m.touches_blanches.iter().all(|l| *l >= 3.0));
    assert!(m.corps_min_pt >= 11.0 && m.trait_min_pt >= 0.25);
    for nom in ["do", "do♯"] {
        assert!(svg.contains(&trace(nom, &Corps::MIN)), "{nom}");
    }
}

#[test]
fn une_colonne_qui_ne_tient_pas_le_dit() {
    let do_majeur = tonalite(Lettre::C, 0, Mode::Majeur);
    let c = Accord {
        fondamentale: Intervalle::new(0, 0),
        qualite: Qualite::Majeur,
        renversement: 0,
    };
    let colonne = |i: Instrument, l: f32, h: f32| {
        colonne_d_accord(
            &c,
            do_majeur,
            i,
            Rect {
                x: 0.0,
                y: 0.0,
                l,
                h,
            },
        )
    };
    // Une colonne de la table, sur la hauteur d'une page : elle tient.
    assert!(colonne(Instrument::LesDeux, 27.2, 130.0).is_ok());
    // Trop basse, ou trop étroite pour le manche : la colonne le dit, jamais un diagramme
    // rogné.
    let basse = colonne(Instrument::LesDeux, 27.2, 50.0).unwrap_err();
    assert!(
        basse
            .quoi
            .starts_with("C en do majeur : la colonne de C mesure "),
        "{}",
        basse.quoi
    );
    let etroite = colonne(Instrument::Guitare, 15.0, 130.0).unwrap_err();
    assert!(
        etroite
            .quoi
            .starts_with("C en do majeur : le manche de C mesure "),
        "{}",
        etroite.quoi
    );
}
