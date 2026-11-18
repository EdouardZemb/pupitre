//! Le repli des textes et les mots cherchés : la recherche d'une requête compare des textes repliés
//! (question 1 ; CAP-06). Le repli ne couvre que ce que couvre [`TABLE`] : un grec ou un cyrillique
//! n'est replié qu'en casse.

/// La table de repli, rangée par caractère, chaque caractère une fois : chaque lettre minuscule de
/// Latin-1 (U+00DF à U+00FF, sans U+00F7) et de Latin étendu A (U+0100 à U+017F) vers sa base, ou sa
/// récriture (œ vers oe, æ vers ae, ß vers ss, þ vers th, ĳ vers ij, ŉ vers 'n) ; les ligatures
/// U+FB00 à U+FB04 ; les apostrophes vers `'`, les tirets vers `-` ; les invisibles vers rien.
pub const TABLE: &[(char, &str)] = &[
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

/// Le texte replié : chaque caractère en minuscules (`char::to_lowercase`), puis, de chaque
/// caractère obtenu, rien pour une marque combinante (U+0300 à U+036F), sa récriture s'il est dans
/// [`TABLE`], lui-même sinon.
pub fn replier(texte: &str) -> String {
    let mut replie = String::with_capacity(texte.len());
    for minuscule in texte.chars().flat_map(char::to_lowercase) {
        if ('\u{0300}'..='\u{036f}').contains(&minuscule) {
            continue;
        }
        match TABLE.binary_search_by_key(&minuscule, |(caractere, _)| *caractere) {
            Ok(rang) => replie.push_str(TABLE[rang].1),
            Err(_) => replie.push(minuscule),
        }
    }
    replie
}

/// Les mots cherchés : la requête repliée, coupée aux blancs, chaque morceau rogné de ce qui n'est
/// ni lettre ni chiffre à ses bords, les morceaux vides écartés.
pub fn mots(requete: &str) -> Vec<String> {
    replier(requete)
        .split_whitespace()
        .map(|morceau| morceau.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|morceau| !morceau.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Chaque mot de `mots` paraît, entier ou en fragment, dans `texte_replie` ; vrai d'une liste vide.
pub fn trouve(texte_replie: &str, mots: &[String]) -> bool {
    mots.iter().all(|mot| texte_replie.contains(mot.as_str()))
}
