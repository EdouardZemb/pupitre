//! Le repli des textes cherchés : la recherche ne distingue ni la casse, ni les accents, ni les
//! apostrophes et les tirets de formes voisines (question 1 ; CAP-06).

use noyau::donnees::repli::{TABLE, mots, replier, trouve};

const ATTENDUES: &[(char, &str)] = &[
    ('\u{0060}', "'"),
    ('\u{00ad}', ""),
    ('\u{00b4}', "'"),
    ('ß', "ss"),
    ('à', "a"),
    ('á', "a"),
    ('â', "a"),
    ('ã', "a"),
    ('ä', "a"),
    ('å', "a"),
    ('æ', "ae"),
    ('ç', "c"),
    ('è', "e"),
    ('é', "e"),
    ('ê', "e"),
    ('ë', "e"),
    ('ì', "i"),
    ('í', "i"),
    ('î', "i"),
    ('ï', "i"),
    ('ð', "d"),
    ('ñ', "n"),
    ('ò', "o"),
    ('ó', "o"),
    ('ô', "o"),
    ('õ', "o"),
    ('ö', "o"),
    ('ø', "o"),
    ('ù', "u"),
    ('ú', "u"),
    ('û', "u"),
    ('ü', "u"),
    ('ý', "y"),
    ('þ', "th"),
    ('ÿ', "y"),
    ('ā', "a"),
    ('ă', "a"),
    ('ą', "a"),
    ('ć', "c"),
    ('ĉ', "c"),
    ('ċ', "c"),
    ('č', "c"),
    ('ď', "d"),
    ('đ', "d"),
    ('ē', "e"),
    ('ĕ', "e"),
    ('ė', "e"),
    ('ę', "e"),
    ('ě', "e"),
    ('ĝ', "g"),
    ('ğ', "g"),
    ('ġ', "g"),
    ('ģ', "g"),
    ('ĥ', "h"),
    ('ħ', "h"),
    ('ĩ', "i"),
    ('ī', "i"),
    ('ĭ', "i"),
    ('į', "i"),
    ('ı', "i"),
    ('ĳ', "ij"),
    ('ĵ', "j"),
    ('ķ', "k"),
    ('ĸ', "k"),
    ('ĺ', "l"),
    ('ļ', "l"),
    ('ľ', "l"),
    ('ŀ', "l"),
    ('ł', "l"),
    ('ń', "n"),
    ('ņ', "n"),
    ('ň', "n"),
    ('ŉ', "'n"),
    ('ŋ', "n"),
    ('ō', "o"),
    ('ŏ', "o"),
    ('ő', "o"),
    ('œ', "oe"),
    ('ŕ', "r"),
    ('ŗ', "r"),
    ('ř', "r"),
    ('ś', "s"),
    ('ŝ', "s"),
    ('ş', "s"),
    ('š', "s"),
    ('ţ', "t"),
    ('ť', "t"),
    ('ŧ', "t"),
    ('ũ', "u"),
    ('ū', "u"),
    ('ŭ', "u"),
    ('ů', "u"),
    ('ű', "u"),
    ('ų', "u"),
    ('ŵ', "w"),
    ('ŷ', "y"),
    ('ź', "z"),
    ('ż', "z"),
    ('ž', "z"),
    ('ſ', "s"),
    ('\u{02bc}', "'"),
    ('\u{200b}', ""),
    ('\u{200c}', ""),
    ('\u{200d}', ""),
    ('\u{2010}', "-"),
    ('\u{2011}', "-"),
    ('\u{2012}', "-"),
    ('\u{2013}', "-"),
    ('\u{2014}', "-"),
    ('\u{2018}', "'"),
    ('\u{2019}', "'"),
    ('\u{2032}', "'"),
    ('\u{2060}', ""),
    ('\u{2212}', "-"),
    ('\u{fb00}', "ff"),
    ('\u{fb01}', "fi"),
    ('\u{fb02}', "fl"),
    ('\u{fb03}', "ffi"),
    ('\u{fb04}', "ffl"),
    ('\u{feff}', ""),
];

#[test]
fn la_table_est_rangee_et_chaque_entree_se_replie() {
    assert!(TABLE.windows(2).all(|paire| paire[0].0 < paire[1].0));
    assert_eq!(TABLE, ATTENDUES);
    for (caractere, attendu) in ATTENDUES {
        assert_eq!(replier(&caractere.to_string()), *attendu, "{caractere:?}");
    }
}

#[test]
fn eglise_trouve_eglise_composee_ou_decomposee() {
    assert_eq!(replier("Église du port"), "eglise du port"); // É, U+00C9
    assert_eq!(replier("E\u{301}glise du port"), "eglise du port"); // E, puis U+0301
    assert!(trouve(&replier("EGLISE"), &mots("église")));
    assert!(trouve(&replier("église"), &mots("ÉGLISE")));
    assert!(trouve(&replier("Eglise"), &mots("église")));
}

#[test]
fn oe_trouve_oe_lie_et_l_inverse() {
    assert!(trouve(&replier("cœur"), &mots("oe")));
    assert!(trouve(&replier("CŒUR"), &mots("coeur")));
    assert!(trouve(&replier("coeur"), &mots("œ")));
    assert_eq!(replier("Straẞe"), "strasse");
    assert_eq!(replier("İzmir"), "izmir");
    assert_eq!(replier("Łódź"), "lodz");
    assert_eq!(replier("\u{fb01}n"), "fin");
}

#[test]
fn les_apostrophes_et_les_tirets_se_replient_dans_un_mot() {
    assert_eq!(mots("l'eglise"), ["l'eglise"]);
    assert!(trouve(&replier("l’Église"), &mots("l'eglise")));
    assert!(!trouve(&replier("la vieille église"), &mots("l'eglise")));
    assert!(trouve(&replier("aujourd’hui"), &mots("aujourd'hui")));
    assert!(trouve(
        &replier("arc\u{2011}en\u{2011}ciel"),
        &mots("arc-en-ciel")
    ));
    assert!(trouve(&replier("arc-en-ciel"), &mots("arc en ciel")));
    assert!(!trouve(&replier("arc en ciel"), &mots("arc-en-ciel")));
    assert!(trouve(&replier("IV\u{2013}iv\u{2013}I"), &mots("IV-iv-I")));
}

#[test]
fn les_mots_se_coupent_aux_blancs_et_perdent_la_ponctuation_des_bords() {
    assert_eq!(mots("mi\u{202f}mineur"), ["mi", "mineur"]);
    assert_eq!(mots("mi\u{a0}mineur"), ["mi", "mineur"]);
    assert_eq!(mots("port."), ["port"]);
    assert_eq!(mots("«eglise»"), ["eglise"]);
    assert_eq!(mots("1/2 IV/V"), ["1/2", "iv/v"]);
    assert_eq!(mots(""), Vec::<String>::new());
    assert_eq!(mots("  \u{a0} "), Vec::<String>::new());
    assert_eq!(mots("' - « »"), Vec::<String>::new());
}

#[test]
fn les_invisibles_s_otent_et_les_formes_de_compatibilite_restent() {
    assert!(trouve(&replier("égl\u{ad}ise"), &mots("eglise")));
    assert!(trouve(&replier("égl\u{200b}ise"), &mots("eglise")));
    assert!(!trouve(&replier("1ᵉʳ couplet"), &mots("1er")));
    assert!(!trouve(&replier("x²"), &mots("x2")));
}

#[test]
fn un_fragment_suffit_sans_longueur_minimale() {
    assert!(trouve(&replier("à 120 bpm"), &mots("12")));
    assert!(trouve(&replier("Noël"), &mots("ë")));
    assert!(trouve(&replier("cœur"), &mots("coe")));
    assert!(trouve(&replier("un texte"), &[]));
    assert!(!trouve(&replier("un texte"), &mots("pont")));
}
