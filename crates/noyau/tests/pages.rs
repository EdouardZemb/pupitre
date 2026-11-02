use noyau::rendu::pages::{Date, HAUTEUR, LARGEUR, Rect};
use noyau::rendu::sous_ensemble::verifier;
use noyau::rendu::svg::{Couleur, Element, Page, TRAIT_FIN};
use noyau::rendu::texte::{Corps, chasse, composer, couper, glyphe_present};

const SVG_AVEC_TEXT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><text x="1" y="5">a</text></svg>"##;
const SVG_ROUGE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><rect x="0" y="0" width="5" height="5" fill="#f00"/></svg>"##;
const SVG_OPACITE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><rect x="0" y="0" width="5" height="5" fill="#000" opacity="0.5"/></svg>"##;
const SVG_STYLE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><style>rect{fill:#000}</style></svg>"##;
const SVG_ANIMATE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><rect x="0" y="0" width="5" height="5" fill="#000"><animate attributeName="x" from="0" to="5" dur="1s"/></rect></svg>"##;
const SVG_DEGRADE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10mm" height="10mm" viewBox="0 0 10 10"><linearGradient id="g"/></svg>"##;

#[test]
fn une_page_vide_est_dans_le_sous_ensemble() {
    let svg = Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements: vec![],
    }
    .svg();
    assert!(svg.contains(r#"width="209.5mm""#) && svg.contains(r#"viewBox="0 0 209.5 157""#));
    assert_eq!(verifier(&svg), Ok(()));
}

#[test]
fn le_sous_ensemble_refuse_ce_qui_sort() {
    for (svg, motif) in [
        (SVG_AVEC_TEXT, "text"),
        (SVG_ROUGE, "#f00"),
        (SVG_OPACITE, "opacity"),
        (SVG_STYLE, "style"),
        (SVG_ANIMATE, "animate"),
        (SVG_DEGRADE, "linearGradient"),
    ] {
        let erreurs = verifier(svg).unwrap_err();
        assert!(erreurs.iter().any(|e| e.contains(motif)), "{motif}");
    }
}

#[test]
fn les_caracteres_exiges_ont_un_glyphe() {
    for c in "àâäçéèêëîïôöùûüÿœæÀÂÄÇÉÈÊËÎÏÔÖÙÛÜŸŒÆ♭♯°IViv0123456789/+·×".chars()
    {
        assert!(glyphe_present(c), "{c}");
    }
}

#[test]
fn un_texte_compose_est_un_groupe_de_traces_a_son_corps() {
    let page = Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements: vec![composer("glisse vers G♯°7", &Corps::MIN, 10.0, 20.0)],
    };
    let svg = page.svg();
    assert_eq!(verifier(&svg), Ok(()));
    assert!(svg.contains(r#"id="t-1""#) && svg.contains("scale(0.001895 -0.001895)"));
}

#[test]
fn la_chasse_d_eloignement_a_11_pt() {
    assert!((chasse("éloignement", &Corps::MIN) - 24.38).abs() < 0.05);
}

#[test]
fn couper_coupe_entre_deux_mots_et_refuse_un_mot_trop_long() {
    let lignes = couper("tension sombre avant la dominante", &Corps::MIN, 30.0).unwrap();
    assert_eq!(lignes.join(" "), "tension sombre avant la dominante");
    assert!(lignes.iter().all(|l| chasse(l, &Corps::MIN) <= 30.0));
    assert_eq!(
        couper("éloignement", &Corps::MIN, 20.0).unwrap_err().mot,
        "éloignement"
    );
}

#[test]
fn le_corps_refuse_moins_de_11_pt() {
    assert!(Corps::pt(10.9).is_none() && Corps::pt(11.0).is_some());
}

#[test]
fn la_date_en_francais() {
    assert_eq!(
        Date {
            annee: 2026,
            mois: 11,
            jour: 1
        }
        .en_francais(),
        "1er nov. 2026"
    );
    assert_eq!(
        Date {
            annee: 2026,
            mois: 10,
            jour: 11
        }
        .en_francais(),
        "11 oct. 2026"
    );
    assert_eq!(
        Date {
            annee: 2027,
            mois: 2,
            jour: 3
        }
        .en_francais(),
        "3 févr. 2027"
    );
    assert_eq!(
        Date::depuis_iso("2026-08-15").unwrap().en_francais(),
        "15 août 2026"
    );
}

#[test]
fn la_date_iso_refuse_ce_qui_n_existe_pas() {
    for s in [
        "2026-02-29",
        "2026-13-01",
        "2026-00-10",
        "2026-1-01",
        "26-01-01",
        "2026-01-01-01",
        "2026-01-3a",
        "",
    ] {
        assert!(Date::depuis_iso(s).is_none(), "{s}");
    }
    assert!(Date::depuis_iso("2028-02-29").is_some());
}

#[test]
fn les_identifiants_se_numerotent_par_prefixe_dans_l_ordre_d_ecriture() {
    let r = Rect {
        x: 1.0,
        y: 2.0,
        l: 3.0,
        h: 4.0,
    };
    let page = Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements: vec![
            composer("a", &Corps::MIN, 0.0, 0.0),
            Element::Rect {
                r,
                id: Some("case"),
                plein: None,
                trait_: Some((Couleur::Noir, TRAIT_FIN)),
            },
            composer("b", &Corps::MIN, 0.0, 0.0),
            Element::Cercle {
                cx: 5.0,
                cy: 5.0,
                r: 1.5,
                id: Some("case"),
                plein: Some(Couleur::Noir),
                trait_: None,
            },
        ],
    };
    let svg = page.svg();
    let ordre: Vec<usize> = ["t-1", "case-1", "t-2", "case-2"]
        .iter()
        .map(|id| {
            svg.find(&format!(r#"id="{id}""#))
                .unwrap_or_else(|| panic!("{id}"))
        })
        .collect();
    assert!(ordre.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(verifier(&svg), Ok(()));
    assert_eq!(svg, page.svg(), "deux écritures donnent le même texte");
}

#[test]
fn un_identifiant_en_double_est_refuse() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="1mm" height="1mm" viewBox="0 0 1 1"><g id="a-1"/><g id="a-1"/></svg>"##;
    assert!(verifier(svg).unwrap_err().iter().any(|e| e.contains("a-1")));
}

#[test]
fn les_nombres_s_ecrivent_sans_zero_final() {
    let page = Page {
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        elements: vec![Element::Ligne {
            de: (1.5, -0.0001),
            a: (2.0, 3.1234),
            trait_: (Couleur::Noir, TRAIT_FIN),
        }],
    };
    let svg = page.svg();
    assert!(
        svg.contains(r#"x1="1.5" y1="0" x2="2" y2="3.123""#),
        "{svg}"
    );
    // Une épaisseur s'arrondit par excès au dix-millième : 0,25 pt s'écrit 0.0882, car 0.088
    // ne ferait que 0,2494 pt, sous le minimum.
    assert!(svg.contains(r#"stroke-width="0.0882""#), "{svg}");
}
