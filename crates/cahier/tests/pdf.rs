//! Le PDF du cahier des tables (REN-07, I19, I25) : 49 pages de la taille de leurs SVG, posées
//! sans mise à l'échelle ; les liens du sommaire, à leur place, mènent chacun à sa table ; deux
//! engendrements à la même date et dans la même couche donnent les mêmes octets, métadonnées
//! comprises ; la commande écrit ce qu'`engendrer` rend.

use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

use cahier::engendrer;
use lopdf::content::Content;
use lopdf::{Dictionary, Document, Object, ObjectId};
use noyau::rendu::Instrument;
use noyau::rendu::cahier::page_sommaire;
use noyau::rendu::pages::{Date, HAUTEUR};
use noyau::table::CoucheSombre;

const DATE: Date = Date {
    annee: 2026,
    mois: 11,
    jour: 1,
};
const L_PT: f32 = 209.5 * 72.0 / 25.4; // 593,858
const H_PT: f32 = 157.0 * 72.0 / 25.4; // 445,039
const PT_PAR_MM: f32 = 72.0 / 25.4;
/// La tolérance sur une longueur du PDF, en points.
const TOLERANCE: f32 = 0.01;

const COUCHES: [CoucheSombre; 2] = [CoucheSombre::Masquee, CoucheSombre::Affichee];

/// Le cahier des deux instruments au 1er nov. 2026, dans une couche, engendré une fois pour les
/// tests qui le lisent.
fn pdf(couche: CoucheSombre) -> &'static [u8] {
    static MASQUEE: OnceLock<Vec<u8>> = OnceLock::new();
    static AFFICHEE: OnceLock<Vec<u8>> = OnceLock::new();
    let cache = match couche {
        CoucheSombre::Masquee => &MASQUEE,
        CoucheSombre::Affichee => &AFFICHEE,
    };
    cache.get_or_init(|| {
        engendrer(Instrument::LesDeux, couche, &DATE).unwrap_or_else(|e| panic!("{e}"))
    })
}

/// Le cahier, couche sombre masquée, comme par défaut.
fn document() -> Document {
    Document::load_mem(pdf(CoucheSombre::Masquee)).expect("un PDF lisible")
}

#[test]
fn ren07_1_le_pdf_a_les_pages_des_svg() {
    // I19 : autant de pages que de SVG, chacune de la taille de sa page SVG, dans les deux
    // couches, qui s'engendrent côte à côte.
    std::thread::scope(|s| {
        for couche in COUCHES {
            s.spawn(move || {
                let doc = Document::load_mem(pdf(couche)).expect("un PDF lisible");
                assert_eq!(doc.get_pages().len(), 49, "{couche:?}");
                for (_, id) in doc.get_pages() {
                    assert_media_box(&doc, id, L_PT, H_PT, TOLERANCE);
                }
            });
        }
    });
}

#[test]
fn le_pdf_pose_ses_pages_sans_mise_a_l_echelle() {
    let doc = document();
    for (n, id) in doc.get_pages() {
        let page = doc.get_dictionary(id).unwrap();
        let ressources = dictionnaire(&doc, page.get(b"Resources").unwrap());
        let xobjets = dictionnaire(&doc, ressources.get(b"XObject").unwrap());
        assert_eq!(xobjets.len(), 1, "page {n} : un seul XObject");
        let (nom, reference) = xobjets.iter().next().unwrap();
        let forme = doc
            .get_object(reference.as_reference().unwrap())
            .unwrap()
            .as_stream()
            .unwrap();
        let bbox = nombres(&doc, forme.dict.get(b"BBox").unwrap());
        let matrice = nombres(&doc, forme.dict.get(b"Matrix").unwrap());

        let flux = Content::decode(&doc.get_page_content(id)).unwrap();
        let mut cm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let mut pose = false;
        for op in &flux.operations {
            match op.operator.as_str() {
                "cm" => {
                    let m: Vec<f32> = op.operands.iter().map(nombre).collect();
                    cm = composer([m[0], m[1], m[2], m[3], m[4], m[5]], cm);
                }
                "Do" => {
                    assert_eq!(op.operands[0].as_name().unwrap(), nom.as_slice());
                    pose = true;
                }
                _ => {}
            }
        }
        assert!(pose, "page {n} : le XObject est posé");

        // La BBox du XObject, par sa /Matrix puis par `cm`, couvre exactement la MediaBox.
        let m = composer(
            [
                matrice[0], matrice[1], matrice[2], matrice[3], matrice[4], matrice[5],
            ],
            cm,
        );
        let (x0, y0) = appliquer(m, bbox[0], bbox[1]);
        let (x1, y1) = appliquer(m, bbox[2], bbox[3]);
        let media = nombres(&doc, page.get(b"MediaBox").unwrap());
        for (vu, attendu) in [
            (x0, media[0]),
            (y0, media[1]),
            (x1, media[2]),
            (y1, media[3]),
        ] {
            assert!(
                (vu - attendu).abs() <= TOLERANCE,
                "page {n} : {vu} au lieu de {attendu}"
            );
        }
    }
}

#[test]
fn chaque_lien_du_pdf_mene_a_sa_table() {
    // Chaque annotation du sommaire couvre la zone de son lien (en mm depuis le haut de la page,
    // en points depuis le bas dans le PDF), et mène à la table de sa tonalité. Les liens sont
    // ceux du sommaire, que le cahier reprend tels quels (le test du noyau le vérifie).
    let zones: Vec<_> = page_sommaire(Instrument::LesDeux, CoucheSombre::Masquee)
        .unwrap()
        .liens
        .iter()
        .map(|l| (l.zone, l.vers))
        .collect();
    assert_eq!(zones.len(), 24);
    let doc = document();
    let pages = doc.get_pages();
    let annotations = match doc.get_dictionary(pages[&1]).unwrap().get(b"Annots") {
        Ok(o) => tableau(&doc, o),
        Err(_) => Vec::new(),
    };
    assert_eq!(annotations.len(), 24, "une annotation par table");
    for (k, (annotation, (zone, vers))) in annotations.iter().zip(&zones).enumerate() {
        let lien = dictionnaire(&doc, annotation);
        assert_eq!(lien.get(b"Subtype").unwrap().as_name().unwrap(), b"Link");
        let rect = nombres(&doc, lien.get(b"Rect").unwrap());
        let attendu = [
            zone.x * PT_PAR_MM,
            (HAUTEUR - zone.y - zone.h) * PT_PAR_MM,
            (zone.x + zone.l) * PT_PAR_MM,
            (HAUTEUR - zone.y) * PT_PAR_MM,
        ];
        for (vu, attendu) in rect.iter().zip(attendu) {
            assert!(
                (vu - attendu).abs() <= TOLERANCE,
                "lien {k} : {rect:?} au lieu de {attendu:?}"
            );
        }
        assert!(
            rect[3] - rect[1] >= 12.0 * PT_PAR_MM - TOLERANCE,
            "lien {k}"
        );
        // L'index 1 + 2k est la page 2 + 2k du PDF, que lopdf compte depuis 1.
        assert_eq!(*vers, 1 + 2 * k);
        let dest = tableau(&doc, lien.get(b"Dest").unwrap());
        let page_visee = u32::try_from(vers + 1).unwrap();
        assert_eq!(
            dest[0].as_reference().unwrap(),
            pages[&page_visee],
            "lien {k} : la destination est la page {page_visee}"
        );
        assert_eq!(dest[1].as_name().unwrap(), b"Fit");
    }
    for (n, id) in pages.iter().skip(1) {
        assert!(
            doc.get_dictionary(*id).unwrap().get(b"Annots").is_err(),
            "page {n} : seul le sommaire porte des liens"
        );
    }
}

#[test]
fn ren07_2_deux_engendrements_identiques() {
    // I25 : la date est une entrée, et rien d'autre ne varie, ni horloge ni identifiant, dans
    // l'une et l'autre couche, qui s'engendrent côte à côte. La couche, elle, change le cahier.
    std::thread::scope(|s| {
        for couche in COUCHES {
            s.spawn(move || {
                let encore = engendrer(Instrument::LesDeux, couche, &DATE).unwrap();
                assert!(
                    encore == pdf(couche),
                    "{couche:?} : deux engendrements diffèrent"
                );
            });
        }
    });
    assert!(
        pdf(CoucheSombre::Masquee) != pdf(CoucheSombre::Affichee),
        "la couche ne change rien"
    );
    let doc = document();
    let info = dictionnaire(&doc, doc.trailer.get(b"Info").unwrap());
    assert_eq!(info.get(b"Producer").unwrap().as_str().unwrap(), b"pupitre");
    assert_eq!(
        info.get(b"CreationDate").unwrap().as_str().unwrap(),
        b"D:20261101000000Z"
    );
    assert!(doc.trailer.get(b"ID").is_err(), "pas d'/ID");
}

// --- La commande -----------------------------------------------------------------------

/// Un fichier du dossier temporaire, au nom propre à ce test et à ce processus.
fn fichier_temporaire(nom: &str) -> PathBuf {
    std::env::temp_dir().join(format!("pupitre-{nom}-{}.pdf", std::process::id()))
}

#[test]
fn la_commande_ecrit_le_cahier() {
    // Sans `--couche`, la couche sombre est masquée ; avec `--couche affichee`, elle s'affiche.
    // Les deux commandes se lancent côte à côte.
    let cas: [(&[&str], CoucheSombre); 2] = [
        (&[], CoucheSombre::Masquee),
        (&["--couche", "affichee"], CoucheSombre::Affichee),
    ];
    std::thread::scope(|s| {
        for (couche, attendue) in cas {
            s.spawn(move || {
                let sortie =
                    fichier_temporaire(&format!("la-commande-ecrit-le-cahier-{attendue:?}"));
                let resultat = Command::new(env!("CARGO_BIN_EXE_cahier"))
                    .args(["--date", "2026-11-01"])
                    .args(couche)
                    .arg("--sortie")
                    .arg(&sortie)
                    .output()
                    .expect("la commande se lance");
                let ecrit = std::fs::read(&sortie);
                let _ = std::fs::remove_file(&sortie);
                assert!(
                    resultat.status.success(),
                    "{couche:?} : {:?} : {}",
                    resultat.status,
                    String::from_utf8_lossy(&resultat.stderr)
                );
                let ecrit = ecrit.expect("le fichier existe");
                assert!(
                    ecrit == pdf(attendue),
                    "{couche:?} : le fichier n'est pas ce qu'engendrer rend"
                );
            });
        }
    });
}

#[test]
fn la_commande_refuse_un_usage_faux() {
    // Une erreur d'usage sort en 2, avec un message en français et l'usage, sans rien écrire.
    let sortie = fichier_temporaire("la-commande-refuse");
    let sortie = sortie.to_str().expect("un chemin lisible");
    let cas: [&[&str]; 5] = [
        &[],
        &["--sortie", sortie],
        &["--date", "2026-02-30", "--sortie", sortie],
        &[
            "--date",
            "2026-11-01",
            "--instrument",
            "violon",
            "--sortie",
            sortie,
        ],
        &["--date", "2026-11-01", "--sortie", sortie, "--format", "a4"],
    ];
    for args in cas {
        let resultat = Command::new(env!("CARGO_BIN_EXE_cahier"))
            .args(args)
            .output()
            .expect("la commande se lance");
        let erreur = String::from_utf8_lossy(&resultat.stderr);
        assert_eq!(resultat.status.code(), Some(2), "{args:?} : {erreur}");
        assert!(
            erreur.contains("usage : cahier --date AAAA-MM-JJ"),
            "{args:?} : {erreur}"
        );
        assert!(
            !std::path::Path::new(sortie).exists(),
            "{args:?} : un fichier écrit"
        );
    }
}

#[test]
fn la_commande_refuse_une_couche_inconnue() {
    // Comme un instrument inconnu : une erreur d'usage, qui sort en 2, dit ce qu'elle attend, en
    // français, avec l'usage, et n'écrit rien.
    let sortie = fichier_temporaire("la-commande-refuse-une-couche");
    let resultat = Command::new(env!("CARGO_BIN_EXE_cahier"))
        .args(["--date", "2026-11-01", "--couche", "grise", "--sortie"])
        .arg(&sortie)
        .output()
        .expect("la commande se lance");
    let erreur = String::from_utf8_lossy(&resultat.stderr);
    assert_eq!(resultat.status.code(), Some(2), "{erreur}");
    assert!(
        erreur.contains("couche inconnue : « grise », attendue masquee ou affichee"),
        "{erreur}"
    );
    assert!(
        erreur.contains("[--couche masquee|affichee]"),
        "l'usage dit l'option : {erreur}"
    );
    assert!(!sortie.exists(), "un fichier écrit");
}

// --- Les aides ---------------------------------------------------------------------------

fn assert_media_box(doc: &Document, page: ObjectId, l: f32, h: f32, tolerance: f32) {
    let media = nombres(
        doc,
        doc.get_dictionary(page).unwrap().get(b"MediaBox").unwrap(),
    );
    assert_eq!(media.len(), 4);
    for (vu, attendu) in media.iter().zip([0.0, 0.0, l, h]) {
        assert!(
            (vu - attendu).abs() <= tolerance,
            "{media:?} au lieu de [0 0 {l} {h}]"
        );
    }
}

fn resoudre<'a>(doc: &'a Document, o: &'a Object) -> &'a Object {
    match o {
        Object::Reference(id) => doc.get_object(*id).unwrap(),
        _ => o,
    }
}

fn dictionnaire<'a>(doc: &'a Document, o: &'a Object) -> &'a Dictionary {
    resoudre(doc, o).as_dict().unwrap()
}

fn tableau(doc: &Document, o: &Object) -> Vec<Object> {
    resoudre(doc, o).as_array().unwrap().clone()
}

fn nombre(o: &Object) -> f32 {
    match o {
        Object::Integer(i) => *i as f32,
        Object::Real(r) => *r,
        _ => panic!("pas un nombre : {o:?}"),
    }
}

fn nombres(doc: &Document, o: &Object) -> Vec<f32> {
    tableau(doc, o).iter().map(nombre).collect()
}

/// `a` puis `b`, en matrices PDF `[a b c d e f]`.
fn composer(a: [f32; 6], b: [f32; 6]) -> [f32; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn appliquer(m: [f32; 6], x: f32, y: f32) -> (f32, f32) {
    (m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5])
}
