//! Le cahier des tables (REN-01) : un sommaire, puis la table et le verso de chacune des 24
//! tonalités, dans l'ordre du cahier, soit 49 pages, couche sombre masquée ou affichée ; un lien
//! par table au sommaire ; sur chaque page, un pied qui dit la page, la date du rendu et
//! l'empreinte de la page sans pied. Et la maquette du § 50 · 3.7, qui est la page du noyau.

mod commun;

use std::path::Path;
use std::sync::OnceLock;

use noyau::hauteur::{Lettre, NomDeNote};
use noyau::rendu::Instrument;
use noyau::rendu::cahier::{empreinte, page_sommaire, pages_du_cahier};
use noyau::rendu::pages::{Date, HAUTEUR, LARGEUR, PageDuCahier, Rect};
use noyau::rendu::sous_ensemble::verifier;
use noyau::rendu::svg::Element;
use noyau::rendu::table::page_table;
use noyau::rendu::texte::{Corps, chasse, composer};
use noyau::rendu::verso::page_verso;
use noyau::table::{CoucheSombre, table};
use noyau::tonalite::{Mode, Tonalite};

const DATE: Date = Date {
    annee: 2026,
    mois: 11,
    jour: 1,
};

const INSTRUMENTS: [Instrument; 3] = [
    Instrument::Guitare,
    Instrument::Clavier,
    Instrument::LesDeux,
];

const COUCHES: [CoucheSombre; 2] = [CoucheSombre::Masquee, CoucheSombre::Affichee];

/// La précision des nombres écrits dans le SVG, et un peu plus.
const EPS: f32 = 2e-3;
const MM_PAR_PT: f32 = 25.4 / 72.0;
/// Unités par cadratin de DejaVu Sans.
const UNITES: f32 = 2048.0;
/// Le bas de la hauteur utile : la page a des marges de 6 mm en haut et en bas.
const BAS_UTILE: f32 = HAUTEUR - 6.0;
/// Les marges de gauche et de droite.
const MARGE_H: f32 = 5.0;
/// Le haut de la zone que la page réserve au pied (REN-01) : un filet et deux lignes à 11 pt,
/// à 1,2 cadratin l'une.
const HAUT_DU_PIED: f32 = BAS_UTILE - (1.2 + 2.0 * 1.2 * 11.0 * MM_PAR_PT);
/// Ce que dit le pied d'une page de table, après l'empreinte.
const VERSO: &str = " · verso : diagrammes et palettes des accords du centre";

// --- Le cahier, rendu une fois par couche pour tous les tests ----------------------

/// Le cahier des deux instruments dans une couche, chacune de ses pages telle que la dessine son
/// propre rendu, sans pied (le sommaire, puis `page_table` et `page_verso` de chaque tonalité,
/// dans cette couche), et l'empreinte de chacune.
struct Cahier {
    pages: Vec<PageDuCahier>,
    sans_pied: Vec<String>,
    empreintes: Vec<String>,
}

fn cahier(couche: CoucheSombre) -> &'static Cahier {
    static MASQUEE: OnceLock<Cahier> = OnceLock::new();
    static AFFICHEE: OnceLock<Cahier> = OnceLock::new();
    let cache = match couche {
        CoucheSombre::Masquee => &MASQUEE,
        CoucheSombre::Affichee => &AFFICHEE,
    };
    cache.get_or_init(|| {
        let i = Instrument::LesDeux;
        // Le cahier et les pages sans pied se rendent côte à côte : une page prend un dixième de
        // seconde en debug.
        let (pages, sans_pied) = std::thread::scope(|s| {
            let pages = s.spawn(|| {
                pages_du_cahier(i, couche, &DATE)
                    .unwrap_or_else(|e| panic!("couche {} : {e}", dite(couche)))
            });
            let mut sans_pied = vec![page_sommaire(i, couche).unwrap().page.svg()];
            for t in Tonalite::les_24() {
                let tb = table(t, couche);
                sans_pied.push(page_table(&tb, i, None).unwrap().page.svg());
                sans_pied.push(page_verso(&tb, i).unwrap().svg());
            }
            (pages.join().expect("le cahier se rend"), sans_pied)
        });
        let empreintes = sans_pied.iter().map(|s| empreinte(s)).collect();
        Cahier {
            pages,
            sans_pied,
            empreintes,
        }
    })
}

// --- Les aides ---------------------------------------------------------------------

/// Rejoue `controle` sur le cahier de chaque couche, côte à côte, chacune sur un fil qui porte
/// son nom : un échec dit sa couche.
fn dans_les_deux_couches(controle: impl Fn(&'static Cahier) + Sync) {
    let controle = &controle;
    std::thread::scope(|s| {
        let fils: Vec<_> = COUCHES
            .into_iter()
            .map(|couche| {
                std::thread::Builder::new()
                    .name(format!("couche {}", dite(couche)))
                    .spawn_scoped(s, move || controle(cahier(couche)))
                    .expect("le fil se lance")
            })
            .collect();
        for f in fils {
            if let Err(e) = f.join() {
                std::panic::resume_unwind(e);
            }
        }
    });
}

/// La couche, comme le sommaire la dit.
fn dite(c: CoucheSombre) -> &'static str {
    match c {
        CoucheSombre::Masquee => "masquée",
        CoucheSombre::Affichee => "affichée",
    }
}

fn do_majeur() -> Tonalite {
    Tonalite {
        tonique: NomDeNote {
            lettre: Lettre::C,
            alteration: 0,
        },
        mode: Mode::Majeur,
    }
}

/// Le SVG sans sa balise fermante : le début de toute page qui y ajoute des éléments.
fn ouvert(svg: &str) -> &str {
    svg.strip_suffix("</svg>").expect("un SVG fermé")
}

/// Le tracé d'un texte composé, tel que la page l'écrit : `d="…"`, quelle que soit sa place et
/// quel que soit son corps.
fn trace(texte: &str) -> String {
    let Element::Groupe { enfants, .. } = composer(texte, &Corps::MIN, 0.0, 0.0) else {
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
        lignes.iter().all(|l| svg.contains(&trace(l)))
    })
}

/// La translation et l'échelle d'un groupe : `translate(x y) scale(k -k)`.
fn transformation(balise: &str) -> ((f32, f32), f32) {
    let debut = balise.find("translate(").expect("une translation") + "translate(".len();
    let fin = debut + balise[debut..].find(')').expect("translation fermée");
    let mut xy = balise[debut..fin]
        .split(' ')
        .map(|s| s.parse::<f32>().expect("un nombre"));
    let origine = (xy.next().unwrap(), xy.next().unwrap());
    let echelle = balise.find("scale(").map_or(1.0, |d| {
        let d = d + "scale(".len();
        balise[d..d + balise[d..].find(' ').expect("deux facteurs")]
            .parse()
            .expect("un nombre")
    });
    (origine, echelle)
}

/// Chaque endroit où la page compose `texte` : l'origine de sa ligne de base, en mm, et son
/// corps, en mm.
fn ou_est_compose(svg: &str, texte: &str) -> Vec<((f32, f32), f32)> {
    let motif = trace(texte);
    svg.match_indices(&motif)
        .map(|(i, _)| {
            let debut = svg[..i]
                .rfind(r#"<g id="t-"#)
                .expect("un texte est un groupe");
            let (origine, k) = transformation(&svg[debut..i]);
            (origine, k * UNITES)
        })
        .collect()
}

/// Vrai si la page compose `texte` tout entier dans la zone : de sa ligne de base, chasse et
/// jambages compris.
fn compose_dans(svg: &str, texte: &str, zone: &Rect) -> bool {
    ou_est_compose(svg, texte)
        .iter()
        .any(|&((x, base), corps_mm)| {
            let corps = Corps::pt(corps_mm / MM_PAR_PT - 1e-3).unwrap_or(Corps::MIN);
            x >= zone.x - EPS
                && x + chasse(texte, &corps) <= zone.x + zone.l + EPS
                && base - 0.928 * corps_mm >= zone.y - EPS
                && base + 0.236 * corps_mm <= zone.y + zone.h + EPS
        })
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

fn se_chevauchent(a: &Rect, b: &Rect) -> bool {
    a.x < b.x + b.l && b.x < a.x + a.l && a.y < b.y + b.h && b.y < a.y + a.h
}

// --- Les 49 pages --------------------------------------------------------------------

#[test]
fn le_cahier_compte_49_pages_dans_les_deux_couches() {
    // Le sommaire, puis la table et le verso de chaque tonalité, dans l'ordre du cahier : chaque
    // page commence par la page que dessine son propre rendu dans la couche du cahier, et le pied
    // s'y ajoute.
    for couche in COUCHES {
        let c = cahier(couche);
        let quoi = format!("couche {}", dite(couche));
        assert_eq!(c.pages.len(), 49, "{quoi}");
        assert_eq!(c.sans_pied.len(), 49, "{quoi}");
        for (n, (page, sans_pied)) in c.pages.iter().zip(&c.sans_pied).enumerate() {
            assert!(
                page.svg.starts_with(ouvert(sans_pied)),
                "{quoi}, page {} : elle ne commence pas par son rendu sans pied",
                n + 1
            );
            assert!(
                page.svg.len() > sans_pied.len(),
                "{quoi}, page {} : sans pied",
                n + 1
            );
            assert!(page.svg.ends_with("</svg>"));
        }
    }
    // La couche change les tables, et non les versos : ils portent les accords du centre.
    let (masquee, affichee) = (
        cahier(CoucheSombre::Masquee),
        cahier(CoucheSombre::Affichee),
    );
    for n in 1..49 {
        let meme = masquee.sans_pied[n] == affichee.sans_pied[n];
        assert_eq!(meme, n % 2 == 0, "page {}", n + 1);
    }
    // Les deux autres instruments donnent aussi 49 pages dans chaque couche, et leurs tables le
    // disent ; ils se rendent côte à côte.
    std::thread::scope(|s| {
        for couche in COUCHES {
            for i in [Instrument::Guitare, Instrument::Clavier] {
                s.spawn(move || {
                    let quoi = format!("{}, couche {}", i.nom(), dite(couche));
                    let pages = pages_du_cahier(i, couche, &DATE)
                        .unwrap_or_else(|e| panic!("{quoi} : {e}"));
                    assert_eq!(pages.len(), 49, "{quoi}");
                    let tb = table(do_majeur(), couche);
                    let attendue = page_table(&tb, i, None).unwrap().page.svg();
                    assert!(pages[1].svg.starts_with(ouvert(&attendue)), "{quoi}");
                });
            }
        }
    });
}

#[test]
fn le_sommaire_mene_a_chaque_table() {
    let c = cahier(CoucheSombre::Masquee);
    let sommaire = &c.pages[0];
    let tonalites = Tonalite::les_24();
    assert_eq!(sommaire.liens.len(), 24, "un lien par table");
    assert_eq!(
        sommaire.liens,
        page_sommaire(Instrument::LesDeux, CoucheSombre::Masquee)
            .unwrap()
            .liens,
        "le cahier reprend les liens du sommaire"
    );
    for (k, (lien, t)) in sommaire.liens.iter().zip(tonalites).enumerate() {
        let z = &lien.zone;
        // Le k-ième mène à la table de la k-ième tonalité, à l'index 1 + 2k (la page 2 + 2k).
        assert_eq!(lien.vers, 1 + 2 * k, "{}", t.nom());
        assert!(
            c.pages[lien.vers]
                .svg
                .starts_with(ouvert(&c.sans_pied[1 + 2 * k])),
            "{} : le lien ne mène pas à sa table",
            t.nom()
        );
        assert!(z.h >= 12.0, "{} : zone de {} mm de haut", t.nom(), z.h);
        assert!(
            z.x >= 0.0 && z.x + z.l <= LARGEUR && z.y >= 0.0 && z.y + z.h <= HAUT_DU_PIED + EPS,
            "{} : la zone {z:?} sort de la page ou entre dans le pied",
            t.nom()
        );
        // L'entrée dit sa tonalité et sa page, dans sa zone.
        assert!(
            compose_dans(&sommaire.svg, &t.nom(), z),
            "{} n'est pas dans sa zone",
            t.nom()
        );
        let page = format!("page {}", 2 + 2 * k);
        assert!(
            compose_dans(&sommaire.svg, &page, z),
            "{page} hors de sa zone"
        );
    }
    // Les zones ne se chevauchent pas, et l'ordre de lecture (de ligne en ligne, de gauche à
    // droite) est l'ordre du cahier.
    let zones: Vec<Rect> = sommaire.liens.iter().map(|l| l.zone).collect();
    for (k, a) in zones.iter().enumerate() {
        for b in &zones[k + 1..] {
            assert!(!se_chevauchent(a, b), "{a:?} et {b:?} se chevauchent");
        }
    }
    let mut lecture: Vec<usize> = (0..24).collect();
    lecture.sort_by(|&a, &b| {
        let (za, zb) = (&zones[a], &zones[b]);
        let meme_ligne = (za.y - zb.y).abs() < EPS;
        if meme_ligne {
            za.x.total_cmp(&zb.x)
        } else {
            za.y.total_cmp(&zb.y)
        }
    });
    assert_eq!(lecture, (0..24).collect::<Vec<_>>(), "l'ordre de lecture");
    // Vigilance 2 : la relative de fa♯ majeur s'écrit mi♭ mineur, et vient juste après lui, sur
    // la même ligne.
    assert_eq!(tonalites[12].nom(), "fa♯ majeur");
    assert_eq!(tonalites[13].nom(), "mi♭ mineur");
    let (fa_diese, mi_bemol) = (&zones[12], &zones[13]);
    assert!((fa_diese.y - mi_bemol.y).abs() < EPS, "sur la même ligne");
    assert!(
        mi_bemol.x > fa_diese.x + fa_diese.l,
        "à droite de fa♯ majeur"
    );
    assert!(compose_dans(&sommaire.svg, "mi♭ mineur", mi_bemol));
    assert!(!sommaire.svg.contains(&trace("ré♯ mineur")));
    // Seul le sommaire porte des liens.
    assert!(c.pages[1..].iter().all(|p| p.liens.is_empty()));
    // Couche sombre affichée, le sommaire mène aux mêmes pages, par les mêmes zones : chacune
    // à la table de sa tonalité, dans cette couche.
    let affichee = cahier(CoucheSombre::Affichee);
    assert_eq!(affichee.pages[0].liens, sommaire.liens);
    for lien in &affichee.pages[0].liens {
        assert!(
            affichee.pages[lien.vers]
                .svg
                .starts_with(ouvert(&affichee.sans_pied[lien.vers])),
            "couche affichée : le lien vers la page {} ne mène pas à sa table",
            lien.vers + 1
        );
    }
}

#[test]
fn le_sommaire_dit_son_titre_l_instrument_et_la_couche() {
    for couche in COUCHES {
        for i in INSTRUMENTS {
            let svg = page_sommaire(i, couche).unwrap().page.svg();
            let quoi = format!("{}, couche {}", i.nom(), dite(couche));
            assert_eq!(verifier(&svg), Ok(()), "{quoi}");
            assert!(svg.contains(&trace("Cahier des tables")), "{quoi}");
            let ligne = format!(
                "instrument : {} · couche sombre : {}",
                i.nom(),
                dite(couche)
            );
            assert!(compose_en_lignes(&svg, &ligne), "{ligne}");
            // Il ne dit que sa couche.
            let autre = COUCHES.into_iter().find(|&c| c != couche).unwrap();
            let fausse = format!("instrument : {} · couche sombre : {}", i.nom(), dite(autre));
            assert!(!compose_en_lignes(&svg, &fausse), "{quoi} : {fausse}");
        }
    }
}

#[test]
fn le_pied_porte_page_date_et_empreinte() {
    dans_les_deux_couches(|c| {
        // La page 2, la table de do majeur, mot pour mot.
        let attendu = format!(
            "page 2 / 49 · rendu le 1er nov. 2026 · empreinte {} · verso : diagrammes et palettes des accords du centre",
            c.empreintes[1]
        );
        assert!(compose_en_lignes(&c.pages[1].svg, &attendu), "{attendu}");
        for (n, (page, sans_pied)) in c.pages.iter().zip(&c.sans_pied).enumerate() {
            // Les pages de table sont aux index impairs ; le sommaire et les versos n'annoncent pas
            // de verso.
            let suite = if n % 2 == 1 { VERSO } else { "" };
            let pied = format!(
                "page {} / 49 · rendu le 1er nov. 2026 · empreinte {}{suite}",
                n + 1,
                c.empreintes[n]
            );
            assert!(
                compose_en_lignes(&page.svg, &pied),
                "page {} : {pied}",
                n + 1
            );
            if n % 2 == 0 {
                assert!(
                    !page.svg.contains(&trace(
                        "verso : diagrammes et palettes des accords du centre"
                    )),
                    "page {} : seule une page de table annonce son verso",
                    n + 1
                );
            }
            // Le pied est dessiné dans la zone réservée, jamais sur le contenu : un filet, puis ses
            // lignes, glyphes compris.
            let pied = &page.svg[ouvert(sans_pied).len()..page.svg.len() - "</svg>".len()];
            assert_eq!(
                pied.matches("<line").count(),
                1,
                "page {} : un filet",
                n + 1
            );
            let filet = &pied[pied.find("<line").unwrap()..];
            for y in [attribut(filet, "y1"), attribut(filet, "y2")] {
                assert!(
                    (HAUT_DU_PIED..=BAS_UTILE).contains(&y),
                    "page {} : filet à {y}",
                    n + 1
                );
            }
            let lignes: Vec<&str> = pied.split(r#"<g id="t-"#).skip(1).collect();
            assert!(
                (1..=2).contains(&lignes.len()),
                "page {} : {} lignes",
                n + 1,
                lignes.len()
            );
            for l in lignes {
                let ((x, base), k) = transformation(l);
                let corps = k * UNITES;
                assert!(x >= MARGE_H - EPS, "page {} : ligne à {x}", n + 1);
                assert!(
                    base - 0.928 * corps >= HAUT_DU_PIED - EPS
                        && base + 0.236 * corps <= BAS_UTILE + EPS,
                    "page {} : ligne de base à {base}",
                    n + 1
                );
            }
            assert!(
                !pied.contains("<rect") && !pied.contains("<circle"),
                "page {} : le pied n'est que du texte et un filet",
                n + 1
            );
            // Mesuré par usvg, glyphes et traits compris : le contenu, la page sans pied, s'arrête
            // au-dessus de la zone du pied ; le pied seul tient entre le haut de cette zone et le bas
            // de la hauteur utile ; chacune de ses lignes, entre les deux marges.
            let contenu = commun::boite_du_dessin(sans_pied);
            assert!(
                contenu.bas <= HAUT_DU_PIED + EPS,
                "page {} : le contenu descend à {} mm, le pied commence à {HAUT_DU_PIED} mm",
                n + 1,
                contenu.bas
            );
            // Le début du SVG jusqu'au fond de la page compris : de quoi dessiner une partie seule.
            let fond = &page.svg[..page.svg.find("/>").expect("le fond de la page") + 2];
            let seul = commun::boite_du_dessin(&format!("{fond}{pied}</svg>"));
            assert!(
                seul.haut >= HAUT_DU_PIED - EPS && seul.bas <= BAS_UTILE + EPS,
                "page {} : le pied va de {} à {} mm",
                n + 1,
                seul.haut,
                seul.bas
            );
            for l in pied.split(r#"<g id="t-"#).skip(1) {
                let ligne = commun::boite_du_dessin(&format!(r#"{fond}<g id="t-{l}</svg>"#));
                assert!(
                    ligne.gauche >= MARGE_H - EPS && ligne.droite <= LARGEUR - MARGE_H + EPS,
                    "page {} : une ligne du pied va de {} à {} mm en largeur",
                    n + 1,
                    ligne.gauche,
                    ligne.droite
                );
            }
        }
    });
}

#[test]
fn l_empreinte_est_celle_de_la_page_sans_pied() {
    // Les six premiers caractères hexadécimaux du SHA-256 : les vecteurs de la FIPS 180-2.
    assert_eq!(empreinte(""), "e3b0c4");
    assert_eq!(empreinte("abc"), "ba7816");
    dans_les_deux_couches(|c| {
        let mut vues = std::collections::BTreeSet::new();
        for (n, ((page, sans_pied), e)) in c
            .pages
            .iter()
            .zip(&c.sans_pied)
            .zip(&c.empreintes)
            .enumerate()
        {
            assert_eq!(e.len(), 6);
            assert!(
                e.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "{e}"
            );
            assert!(vues.insert(e), "page {} : une empreinte déjà vue", n + 1);
            // La page, son pied ôté, est son rendu sans pied, à l'octet : c'est lui que l'empreinte
            // résume (le pied la porte : `le_pied_porte_page_date_et_empreinte`).
            let sans_son_pied = format!("{}</svg>", &page.svg[..ouvert(sans_pied).len()]);
            assert!(sans_son_pied == *sans_pied, "page {}", n + 1);
        }
    });
}

#[test]
fn les_minimums_sur_les_49_pages() {
    dans_les_deux_couches(|c| {
        for (n, page) in c.pages.iter().enumerate() {
            let quoi = format!("page {}", n + 1);
            assert_eq!(verifier(&page.svg), Ok(()), "{quoi}");
            let m = commun::mesurer(&page.svg);
            assert!(m.corps_min_pt >= 11.0, "{quoi} : corps {}", m.corps_min_pt);
            assert!(m.trait_min_pt >= 0.25, "{quoi} : trait {}", m.trait_min_pt);
            assert!(
                m.manches.iter().all(|l| *l >= 18.0),
                "{quoi} : {:?}",
                m.manches
            );
            assert!(
                m.touches_blanches.iter().all(|l| *l >= 3.0),
                "{quoi} : {:?}",
                m.touches_blanches
            );
            assert!(
                m.pastilles.iter().all(|d| *d >= 3.0),
                "{quoi} : {:?}",
                m.pastilles
            );
            assert!(page.liens.iter().all(|l| l.zone.h >= 12.0), "{quoi}");
            // Ce que la page doit mesurer, elle le dessine.
            match n {
                0 => assert_eq!(page.liens.len(), 24, "{quoi}"),
                _ if n % 2 == 1 => assert_eq!(m.pastilles.len(), 7, "{quoi} : les pastilles"),
                _ => assert!(
                    !m.manches.is_empty() && !m.touches_blanches.is_empty(),
                    "{quoi} : les diagrammes du verso"
                ),
            }
        }
    });
}

// --- La maquette du § 50 · 3.7 ---------------------------------------------------------

/// La maquette de la table, `docs/assets/maquettes/table.svg`, est la page que le noyau dessine :
/// do majeur, instrument guitare, couche sombre masquée, sans pied. Le test compare le fichier au
/// rendu ; avec `PUPITRE_ECRIRE_MAQUETTE=1`, il réécrit d'abord le fichier :
/// `PUPITRE_ECRIRE_MAQUETTE=1 cargo test -p noyau --test cahier maquette`.
#[test]
fn la_maquette_de_la_table_est_la_page_du_noyau() {
    let chemin =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/assets/maquettes/table.svg");
    let tb = table(do_majeur(), CoucheSombre::Masquee);
    let page = page_table(&tb, Instrument::Guitare, None)
        .unwrap()
        .page
        .svg()
        + "\n";
    if std::env::var_os("PUPITRE_ECRIRE_MAQUETTE").is_some_and(|v| v == "1") {
        std::fs::write(&chemin, &page).expect("la maquette s'écrit");
    }
    let lue = std::fs::read_to_string(&chemin)
        .expect("la maquette se lit")
        .replace("\r\n", "\n");
    assert!(
        lue == page,
        "la maquette n'est plus la page du noyau : PUPITRE_ECRIRE_MAQUETTE=1 cargo test -p noyau --test cahier maquette"
    );
}
