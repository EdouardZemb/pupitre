//! Le verso de la table et la fiche d'un accord : ils tiennent pour les 24 tonalités et les
//! trois instruments, gardent les minimums physiques, et dessinent ce que disent REN-04,
//! REN-08 et MIC-13.

mod commun;

use noyau::accord::{Accord, Qualite};
use noyau::clavier::Voicing;
use noyau::guitare::PAS_DE_DOIGTE;
use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use noyau::rendu::Instrument;
use noyau::rendu::clavier::diagramme_de_clavier;
use noyau::rendu::manche::diagramme_de_manche;
use noyau::rendu::pages::{HAUTEUR, LARGEUR, Rect};
use noyau::rendu::sous_ensemble::verifier;
use noyau::rendu::svg::{Couleur, Element, Page};
use noyau::rendu::table::page_table;
use noyau::rendu::texte::{Corps, chasse, composer};
use noyau::rendu::verso::{colonne_d_accord, fiche_d_accord, page_verso};
use noyau::table::{CoucheSombre, Rangee, table};
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

/// Ce qu'on lit d'un dessin, ses groupes traversés, en coordonnées de la page.
#[derive(Default)]
struct Releve {
    /// Les rectangles, avec leur identifiant et leur remplissage.
    rects: Vec<(Option<&'static str>, Rect, Option<Couleur>)>,
    /// Les cercles : centre, rayon, remplissage.
    cercles: Vec<((f32, f32), f32, Option<Couleur>)>,
    /// Les textes : le tracé, et l'origine de leur ligne de base.
    textes: Vec<(String, (f32, f32))>,
    /// Les groupes nommés : leur identifiant et leur origine.
    groupes: Vec<(&'static str, (f32, f32))>,
}

fn relever(e: &Element, origine: (f32, f32), r: &mut Releve) {
    match e {
        Element::Groupe {
            id,
            translation,
            enfants,
            ..
        } => {
            let o = (origine.0 + translation.0, origine.1 + translation.1);
            if *id == Some("t") {
                if let Element::Chemin { d, .. } = &enfants[0] {
                    r.textes.push((format!(r#"d="{d}""#), o));
                }
                return;
            }
            if let Some(id) = id {
                r.groupes.push((id, o));
            }
            for enfant in enfants {
                relever(enfant, o, r);
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
            r: rayon,
            plein,
            ..
        } => r
            .cercles
            .push(((cx + origine.0, cy + origine.1), *rayon, *plein)),
        Element::Chemin { .. } | Element::Ligne { .. } => {}
    }
}

fn releve(e: &Element) -> Releve {
    let mut r = Releve::default();
    relever(e, (0.0, 0.0), &mut r);
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
    let e = diagramme_de_clavier(
        &Voicing {
            notes: vec![60, 64, 67],
        },
        note(Lettre::C, 0),
        (0.0, 0.0),
    );
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
        .map(|(centre, ..)| {
            let noire = noires.iter().position(|t| contient(t, *centre));
            let blanche = blanches.iter().position(|t| contient(t, *centre));
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
    let (_, base) = r
        .textes
        .iter()
        .find(|(d, _)| *d == trace("C", &Corps::MIN))
        .expect("la basse est écrite");
    let milieu = base.0 + chasse("C", &Corps::MIN) / 2.0;
    assert!((milieu - (do_3.x + do_3.l / 2.0)).abs() <= EPS, "{milieu}");
    assert!(base.1 > do_3.y + do_3.h, "la lettre est sous le clavier");
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

#[test]
fn toute_case_a_sa_fiche() {
    // La table affichée porte toutes les cases de la table masquée, et la couche sombre. Les
    // 1 944 fiches demandent une minute en debug sur un seul fil : une tonalité par fil.
    std::thread::scope(|fils| {
        for t in Tonalite::les_24() {
            fils.spawn(move || {
                let tb = table(t, CoucheSombre::Affichee);
                for c in &tb.cases {
                    for i in INSTRUMENTS {
                        let nom = format!("{} en {}, {}", tb.nom(c), t.nom(), i.nom());
                        let p = fiche_d_accord(&c.accord, t, i)
                            .unwrap_or_else(|e| panic!("{nom} : {e}"));
                        assert_eq!(verifier(&p.svg()), Ok(()), "{nom}");
                    }
                }
            });
        }
    });
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
