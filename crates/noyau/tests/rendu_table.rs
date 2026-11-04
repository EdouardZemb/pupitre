//! La page de la table : elle tient pour les 24 tonalités, dit ses réglages, et ses flèches
//! relient chacune deux cases (MIC-12 · 7).

mod commun;

use noyau::hauteur::{Lettre, NomDeNote};
use noyau::rendu::Instrument;
use noyau::rendu::pages::{HAUTEUR, LARGEUR, Rect};
use noyau::rendu::sous_ensemble::verifier;
use noyau::rendu::svg::Element;
use noyau::rendu::table::{PageTable, page_table};
use noyau::rendu::texte::{Corps, composer};
use noyau::table::{CoucheSombre, Famille, Position, Rangee, Table, table};
use noyau::tonalite::{Mode, Tonalite};

/// La précision des nombres écrits dans le SVG, et un peu plus.
const EPS: f32 = 2e-3;

fn tonalite(lettre: Lettre, alteration: i8, mode: Mode) -> Tonalite {
    Tonalite {
        tonique: NomDeNote { lettre, alteration },
        mode,
    }
}

fn do_majeur() -> Tonalite {
    tonalite(Lettre::C, 0, Mode::Majeur)
}

fn la_mineur() -> Tonalite {
    tonalite(Lettre::A, 0, Mode::Mineur)
}

/// Les 48 tables : les 24 tonalités, couche sombre masquée puis affichée.
fn les_48() -> Vec<Table> {
    Tonalite::les_24()
        .into_iter()
        .flat_map(|t| [CoucheSombre::Masquee, CoucheSombre::Affichee].map(|c| table(t, c)))
        .collect()
}

fn decrire(t: &Table) -> String {
    format!("{}, couche sombre {:?}", t.tonalite.nom(), t.couche_sombre)
}

fn rendre(t: &Table) -> PageTable {
    page_table(t, Instrument::default(), None).unwrap_or_else(|e| panic!("{} : {e}", decrire(t)))
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

/// Vrai si la page compose le texte sur des lignes coupées aux « · », quelle que soit la coupe.
fn compose_en_lignes(svg: &str, texte: &str) -> bool {
    let segments: Vec<&str> = texte.split(" · ").collect();
    let coupes = segments.len() - 1;
    (0..1_u32 << coupes).any(|masque| {
        let mut lignes = vec![segments[0].to_string()];
        for (k, s) in segments[1..].iter().enumerate() {
            if masque & (1 << k) != 0 {
                lignes.push(s.to_string());
            } else {
                let derniere = lignes.last_mut().expect("une ligne au moins");
                derniere.push_str(" · ");
                derniere.push_str(s);
            }
        }
        lignes.iter().all(|l| svg.contains(&trace(l, &Corps::MIN)))
    })
}

fn rect_de(p: &PageTable, pos: Position) -> Rect {
    p.disposition
        .cases
        .iter()
        .find(|(q, _)| *q == pos)
        .unwrap_or_else(|| panic!("{pos:?} n'a pas de place"))
        .1
}

/// Le point est dans le rectangle, bords compris.
fn dans(r: &Rect, (x, y): (f32, f32)) -> bool {
    x >= r.x - EPS && x <= r.x + r.l + EPS && y >= r.y - EPS && y <= r.y + r.h + EPS
}

/// Le point est franchement à l'intérieur du rectangle.
fn dedans(r: &Rect, (x, y): (f32, f32)) -> bool {
    x > r.x + EPS && x < r.x + r.l - EPS && y > r.y + EPS && y < r.y + r.h - EPS
}

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// La valeur d'un attribut numérique d'une balise.
fn attribut(balise: &str, nom: &str) -> f32 {
    let motif = format!(r#" {nom}=""#);
    let debut = balise
        .find(&motif)
        .unwrap_or_else(|| panic!("{nom} absent"))
        + motif.len();
    let fin = debut + balise[debut..].find('"').expect("valeur fermée");
    balise[debut..fin].parse().expect("un nombre")
}

/// Une flèche lue dans le SVG : sa queue, et les trois sommets de sa pointe.
type Fleche = ((f32, f32), [(f32, f32); 3]);

/// Les flèches de la page, dans l'ordre d'écriture : la queue, et les trois sommets de la
/// pointe, lus dans le SVG. Une flèche est un groupe `fleche-n` : son trait (`line`), puis sa
/// pointe (`path`), dont le premier sommet est posé sur la cible.
fn fleches(svg: &str) -> Vec<Fleche> {
    let mut resultat = Vec::new();
    for (n, morceau) in svg.split(r#"<g id="fleche-"#).enumerate().skip(1) {
        let numero: usize = morceau[..morceau.find('"').expect("id fermé")]
            .parse()
            .expect("un numéro");
        assert_eq!(numero, n, "les flèches sont numérotées dans l'ordre");
        let ligne = &morceau[morceau.find("<line").expect("le trait")..];
        let ligne = &ligne[..ligne.find("/>").expect("balise fermée")];
        let queue = (attribut(ligne, "x1"), attribut(ligne, "y1"));
        let d = &morceau[morceau.find(r#"<path d="M"#).expect("la pointe") + 10..];
        let d = &d[..d.find('Z').expect("une pointe fermée")];
        let mut nombres = d
            .split(['L', ' '])
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<f32>().expect("un nombre"));
        let mut sommet = || (nombres.next().unwrap(), nombres.next().unwrap());
        let sommets = [sommet(), sommet(), sommet()];
        resultat.push((queue, sommets));
    }
    resultat
}

#[test]
fn toutes_les_pages_de_table_tiennent() {
    for t in les_48() {
        let p = page_table(&t, Instrument::default(), None);
        let p = p.unwrap_or_else(|e| panic!("{} : {e}", decrire(&t)));
        assert_eq!(verifier(&p.page.svg()), Ok(()), "{}", decrire(&t));
        assert_eq!((p.page.largeur, p.page.hauteur), (LARGEUR, HAUTEUR));
    }
}

#[test]
fn les_minimums_de_la_page_de_table() {
    let mut pages: Vec<(String, PageTable)> =
        les_48().iter().map(|t| (decrire(t), rendre(t))).collect();
    // La case cible, en inverse, garde ses minimums : son double cadre, sa pastille.
    let t = table(la_mineur(), CoucheSombre::Affichee);
    for cible in [
        Position {
            rangee: Rangee::Centre,
            rang: 4,
        },
        Position {
            rangee: Rangee::Dessous,
            rang: 4,
        },
    ] {
        let p = page_table(&t, Instrument::default(), Some(cible)).unwrap();
        pages.push((format!("{} avec la cible {cible:?}", decrire(&t)), p));
    }
    for (nom, p) in pages {
        let svg = p.page.svg();
        assert_eq!(verifier(&svg), Ok(()), "{nom}");
        let m = commun::mesurer(&svg);
        assert!(m.corps_min_pt >= 11.0, "{nom} : corps {}", m.corps_min_pt);
        assert!(m.trait_min_pt >= 0.25, "{nom} : trait {}", m.trait_min_pt);
        assert_eq!(
            m.pastilles.len(),
            7,
            "{nom} : une pastille par case du centre"
        );
        assert!(
            m.pastilles.iter().all(|d| *d >= 3.0),
            "{nom} : {:?}",
            m.pastilles
        );
    }
}

#[test]
fn la_couche_masquee_le_dit() {
    let masquee = trace("couche sombre masquée", &Corps::MIN);
    for t in les_48() {
        let svg = rendre(&t).page.svg();
        let dit = svg.contains(&masquee);
        match t.couche_sombre {
            CoucheSombre::Masquee => assert!(dit, "{}", decrire(&t)),
            CoucheSombre::Affichee => assert!(!dit, "{}", decrire(&t)),
        }
    }
}

#[test]
fn l_en_tete_dit_les_reglages() {
    let cas = [
        (
            la_mineur(),
            CoucheSombre::Affichee,
            Instrument::LesDeux,
            "la mineur · table de la tonalité · tonalité choisie · instrument : guitare et clavier · couche sombre : affichée",
        ),
        (
            do_majeur(),
            CoucheSombre::Masquee,
            Instrument::Guitare,
            "do majeur · table de la tonalité · tonalité choisie · instrument : guitare · couche sombre : masquée",
        ),
        (
            tonalite(Lettre::E, -1, Mode::Mineur),
            CoucheSombre::Masquee,
            Instrument::Clavier,
            "mi♭ mineur · table de la tonalité · tonalité choisie · instrument : clavier · couche sombre : masquée",
        ),
    ];
    for (t, couche, instrument, attendu) in cas {
        let svg = page_table(&table(t, couche), instrument, None)
            .unwrap()
            .page
            .svg();
        assert!(compose_en_lignes(&svg, attendu), "{attendu}");
    }
}

#[test]
fn deux_fleches_sur_la_meme_cible_arrivent_a_part() {
    let t = table(do_majeur(), CoucheSombre::Affichee);
    let p = rendre(&t);
    let arrivee = |nom: &str| {
        let c = t.cases.iter().find(|c| t.nom(c) == nom).expect(nom);
        let sol = t
            .cases
            .iter()
            .find(|c| t.nom(c) == "G" && c.position.rangee == Rangee::Centre);
        assert_eq!(c.cible, sol.map(|s| s.position), "{nom} mène à G");
        p.arrivees
            .iter()
            .find(|(source, _)| *source == c.position)
            .unwrap_or_else(|| panic!("la flèche de {nom}"))
            .1
    };
    let d = distance(arrivee("F♯°7"), arrivee("D♭/F"));
    assert!(d >= 3.0, "{d} mm");

    // Et partout : deux pointes sur une même case restent à 3 mm l'une de l'autre.
    for t in les_48() {
        let p = rendre(&t);
        for (i, (a, pa)) in p.arrivees.iter().enumerate() {
            for (b, pb) in &p.arrivees[i + 1..] {
                if t.cible_de(*a) == t.cible_de(*b) {
                    let d = distance(*pa, *pb);
                    assert!(d >= 3.0, "{} : {a:?} et {b:?} à {d} mm", decrire(&t));
                }
            }
        }
    }
}

#[test]
fn chaque_fleche_relie_exactement_deux_cases() {
    for t in les_48() {
        let nom = decrire(&t);
        let p = rendre(&t);
        let svg = p.page.svg();
        let avec_cible = t.cases.iter().filter(|c| c.cible.is_some()).count();
        assert_eq!(
            p.arrivees.len(),
            avec_cible,
            "{nom} : une flèche par case qui a une cible"
        );
        let traits = fleches(&svg);
        assert_eq!(
            traits.len(),
            avec_cible,
            "{nom} : autant de flèches dessinées"
        );
        for ((source, arrivee), (queue, sommets)) in p.arrivees.iter().zip(&traits) {
            let pointe = &sommets[0];
            let cible = t.cible_de(*source).expect("une flèche a une cible");
            let (rs, rc) = (rect_de(&p, *source), rect_de(&p, cible));
            assert!(
                distance(*pointe, *arrivee) <= EPS,
                "{nom} : {source:?}, {pointe:?} au lieu de {arrivee:?}"
            );
            assert!(
                dans(&rs, *queue),
                "{nom} : la queue de {source:?} hors de sa case"
            );
            assert!(
                dans(&rc, *pointe),
                "{nom} : la pointe de {source:?} hors de {cible:?}"
            );
            // La pointe se pose sur un bord de la cible, horizontal…
            let sur_un_bord =
                (pointe.1 - rc.y).abs() <= EPS || (pointe.1 - rc.y - rc.h).abs() <= EPS;
            assert!(
                sur_un_bord,
                "{nom} : la pointe de {source:?} n'est pas sur un bord"
            );
            // … et droit : hors de l'arrondi ou de la coupe des coins (le quart du petit côté).
            if t.case(cible).unwrap().famille != Famille::Diatonique {
                let coin = rc.l.min(rc.h) / 4.0;
                assert!(
                    pointe.0 >= rc.x + coin && pointe.0 <= rc.x + rc.l - coin,
                    "{nom} : la pointe de {source:?} touche un coin de {cible:?}"
                );
            }
            // Aucune partie de la pointe n'entre dans le cadre de la cible.
            for s in sommets {
                assert!(
                    !dedans(&rc, *s),
                    "{nom} : la pointe de {source:?} entre dans {cible:?} en {s:?}"
                );
            }
            // Entre les deux bouts, le trait ne traverse aucune autre case.
            for (autre, r) in &p.disposition.cases {
                if *autre == *source || *autre == cible {
                    continue;
                }
                for k in 0..=100 {
                    let s = k as f32 / 100.0;
                    let point = (
                        queue.0 + s * (pointe.0 - queue.0),
                        queue.1 + s * (pointe.1 - queue.1),
                    );
                    assert!(
                        !dedans(r, point),
                        "{nom} : la flèche de {source:?} traverse {autre:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn la_disposition_couvre_chaque_case() {
    for t in les_48() {
        let nom = decrire(&t);
        let p = rendre(&t);
        let cases = &p.disposition.cases;
        assert_eq!(cases.len(), t.cases.len(), "{nom} : une place par case");
        for c in &t.cases {
            let n = cases.iter().filter(|(q, _)| *q == c.position).count();
            assert_eq!(n, 1, "{nom} : {:?} a une place, une seule", c.position);
        }
        for (i, (a, ra)) in cases.iter().enumerate() {
            assert!(
                ra.x >= 0.0 && ra.y >= 0.0 && ra.x + ra.l <= LARGEUR && ra.y + ra.h <= HAUTEUR,
                "{nom} : {a:?} hors de la page"
            );
            for (b, rb) in &cases[i + 1..] {
                let l = (ra.x + ra.l).min(rb.x + rb.l) - ra.x.max(rb.x);
                let h = (ra.y + ra.h).min(rb.y + rb.h) - ra.y.max(rb.y);
                assert!(
                    l <= EPS || h <= EPS,
                    "{nom} : {a:?} et {b:?} se chevauchent"
                );
            }
        }
    }
}

#[test]
fn l_en_tete_reserve_toujours_deux_lignes() {
    // Avec un seul instrument, l'en-tête tiendrait parfois sur une ligne : la table ne doit
    // pas bouger d'une page à l'autre du cahier.
    let cas = [
        (do_majeur(), CoucheSombre::Masquee),
        (la_mineur(), CoucheSombre::Affichee),
        (
            tonalite(Lettre::E, -1, Mode::Mineur),
            CoucheSombre::Affichee,
        ),
    ];
    for (t, couche) in cas {
        let tb = table(t, couche);
        let origines: Vec<f32> = [
            Instrument::Guitare,
            Instrument::Clavier,
            Instrument::LesDeux,
        ]
        .into_iter()
        .map(|i| {
            let p = page_table(&tb, i, None).unwrap();
            rect_de(&p, tb.rangee(Rangee::Dessus)[0].position).y
        })
        .collect();
        assert!(
            origines.iter().all(|y| *y == origines[0]),
            "{} : {origines:?}",
            decrire(&tb)
        );
    }
}

#[test]
fn un_debordement_se_dit() {
    // Un chiffre plus large que sa case : la page le dit, jamais un mot coupé.
    let mut t = table(do_majeur(), CoucheSombre::Masquee);
    let chiffre = "V7/ii-bien-plus-large-que-sa-case";
    let c = t
        .cases
        .iter_mut()
        .find(|c| c.position.rangee == Rangee::Dessus)
        .unwrap();
    c.chiffre = chiffre.to_string();
    let e = page_table(&t, Instrument::default(), None).unwrap_err();
    assert!(
        e.quoi
            .starts_with("la table de do majeur, couche sombre masquée : "),
        "{}",
        e.quoi
    );
    assert!(
        e.quoi.contains(&format!("« {chiffre} » mesure ")),
        "{}",
        e.quoi
    );
    assert!(
        e.to_string()
            .starts_with("la page ne tient pas : la table de do majeur"),
        "{e}"
    );
}
