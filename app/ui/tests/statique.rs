//! Contrôles statiques d'I31 et d'I29 côté interface (§ 70 · 3). I31 : l'interface ne pose ni
//! HTML, ni style en attribut, ni stockage du navigateur ; elle ne lit l'horloge que dans
//! `date_du_jour` et `decalage_du_geste`, la date à l'heure locale que dans `date_du_jour`, et le
//! décalage local que dans `decalage_du_geste` et `decalage_a`. I29 : ni greffon ni API de
//! notification. Les tests lisent sur le disque chaque `.rs` de `src/` et `index.html`, et, pour
//! I29 seule, le `Cargo.toml` de l'interface ; un lieu dont aucune source n'est lue est une faute.

use std::fs;
use std::path::{Path, PathBuf};

/// Le HTML posé.
const HTML_POSE: &[&str] = &[
    "inner_html",
    "innerHTML",
    "outer_html",
    "outerHTML",
    "insert_adjacent_html",
    "insertAdjacentHTML",
];

/// Les seuls fichiers où `inner_html` paraît, une fois chacun : la table et la palette, qui
/// reçoivent le SVG du noyau.
const INNER_HTML_ADMIS: &[&str] = &["app/ui/src/vue_table.rs", "app/ui/src/panneau_palette.rs"];

/// Un style en attribut ; `style:` reste admis.
const STYLE_EN_ATTRIBUT: &[&str] = &["style=", "\"style\"", "cssText", "css_text", "<style"];

/// Le stockage du navigateur, l'appel comme le nom écrit en chaîne.
const STOCKAGE: &[&str] = &[
    "localStorage",
    "local_storage",
    "sessionStorage",
    "session_storage",
    "indexedDB",
    "indexed_db",
    "IdbFactory",
    "CacheStorage",
    "\"caches\"",
    "cookie",
];

/// L'horloge, admise dans les seules fonctions que nomme `admis_dans`.
const HORLOGE: &[&str] = &["Date::new_0", "Date::now", "performance"];

/// Le décalage local : `getTimezoneOffset`, et chaque lecture à l'heure locale (l'année, le mois,
/// le jour, le jour de la semaine, l'heure, les minutes, un texte à la manière du pays, `Intl`, la
/// date ou l'heure en texte, une date construite à l'heure locale par
/// `new_with_year_month_day…`), admis dans les seules fonctions que nomme `admis_dans`. Une
/// lecture en UTC (`getUTCDate`) n'en est pas une.
const DECALAGE_LOCAL: &[&str] = &[
    "get_timezone_offset",
    "getTimezoneOffset",
    "get_full_year",
    "get_month",
    "get_date",
    "get_day",
    "get_hours",
    "get_minutes",
    "getFullYear",
    "getMonth",
    "getDate",
    "getDay",
    "getHours",
    "getMinutes",
    "to_locale",
    "toLocale",
    "Intl",
    "DateTimeFormat",
    "to_date_string",
    "toDateString",
    "to_time_string",
    "toTimeString",
    "new_with_year_month_day",
];

/// Les fonctions dont le contrôle suit le corps : les seules où un mot de `HORLOGE` ou de
/// `DECALAGE_LOCAL` peut être admis.
const FONCTIONS_SUIVIES: &[&str] = &["date_du_jour", "decalage_du_geste", "decalage_a"];

/// Les fonctions où `mot`, de `HORLOGE` ou de `DECALAGE_LOCAL`, est admis : `Date::new_0`, dans
/// `date_du_jour` et `decalage_du_geste` ; le décalage, `get_timezone_offset` et
/// `getTimezoneOffset`, dans `decalage_du_geste` et `decalage_a` ; tout autre mot, l'horloge ou la
/// date à l'heure locale, dans `date_du_jour` seule.
fn admis_dans(mot: &str) -> &'static [&'static str] {
    match mot {
        "Date::new_0" => &["date_du_jour", "decalage_du_geste"],
        "get_timezone_offset" | "getTimezoneOffset" => &["decalage_du_geste", "decalage_a"],
        _ => &["date_du_jour"],
    }
}

/// Le manifeste de l'interface, nommé depuis la racine du dépôt : le seul manifeste que lit le
/// contrôle d'I29.
const MANIFESTE: &str = "app/ui/Cargo.toml";

/// Les mots d'I29 : le greffon (`tauri-plugin-notification`) et l'API de notification
/// (`web_sys::Notification`, `window.Notification`), en sous-chaîne, sensible à la casse.
const NOTIFICATION: &[&str] = &["Notification", "notification"];

/// Où en est la lecture d'un fichier à la fin d'une ligne : dans le code, dans une chaîne, ou dans
/// une chaîne brute ouverte par `r` et ce nombre de `#`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lecture {
    Code,
    Chaine,
    Brute(usize),
}

fn identifiant(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Le `r` de rang `i` commence un mot : rien d'un identifiant avant lui, sinon un `b` qui commence
/// lui-même un mot (`br"…"`).
fn commence_un_mot(c: &[char], i: usize) -> bool {
    match i.checked_sub(1).map(|j| c[j]) {
        None => true,
        Some('b') => i < 2 || !identifiant(c[i - 2]),
        Some(avant) => !identifiant(avant),
    }
}

/// `ligne`, lue à partir de `lecture`, sans le contenu de ses chaînes (leurs guillemets restent),
/// brutes comprises, sans ses caractères littéraux (`'{'`, `'\''` ; une durée de vie, `'a`, reste)
/// ni son commentaire de fin de ligne ; et la lecture à sa fin, une chaîne pouvant courir sur
/// plusieurs lignes. Les commentaires `/* … */` restent.
fn nettoyer(ligne: &str, mut lecture: Lecture) -> (String, Lecture) {
    let c: Vec<char> = ligne.chars().collect();
    let mut nette = String::new();
    let mut i = 0;
    while i < c.len() {
        match lecture {
            Lecture::Chaine => {
                if c[i] == '\\' {
                    i += 2;
                    continue;
                }
                if c[i] == '"' {
                    nette.push('"');
                    lecture = Lecture::Code;
                }
                i += 1;
            }
            Lecture::Brute(dieses) => {
                let fermee = c[i] == '"'
                    && c.get(i + 1..i + 1 + dieses)
                        .is_some_and(|suite| suite.iter().all(|x| *x == '#'));
                if fermee {
                    nette.push('"');
                    lecture = Lecture::Code;
                    i += 1 + dieses;
                } else {
                    i += 1;
                }
            }
            Lecture::Code => {
                if c[i] == '/' && c.get(i + 1) == Some(&'/') {
                    break;
                }
                if c[i] == '"' {
                    nette.push('"');
                    lecture = Lecture::Chaine;
                    i += 1;
                    continue;
                }
                if c[i] == 'r' && commence_un_mot(&c, i) {
                    let dieses = c[i + 1..].iter().take_while(|x| **x == '#').count();
                    if c.get(i + 1 + dieses) == Some(&'"') {
                        nette.push('"');
                        lecture = Lecture::Brute(dieses);
                        i += 2 + dieses;
                        continue;
                    }
                }
                if c[i] == '\'' {
                    if c.get(i + 1) == Some(&'\\') {
                        let fin = (i + 3..c.len()).find(|&j| c[j] == '\'').unwrap_or(c.len());
                        nette.push_str("''");
                        i = fin + 1;
                        continue;
                    }
                    if c.get(i + 2) == Some(&'\'') {
                        nette.push_str("''");
                        i += 3;
                        continue;
                    }
                }
                nette.push(c[i]);
                i += 1;
            }
        }
    }
    (nette, lecture)
}

/// Ce qui suit, dans `ligne`, le premier `fn <nom>`, s'il y en a un.
fn apres_la_fonction<'a>(ligne: &'a str, nom: &str) -> Option<&'a str> {
    ligne.match_indices("fn ").find_map(|(debut, _)| {
        if ligne[..debut].chars().next_back().is_some_and(identifiant) {
            return None;
        }
        ligne[debut + 3..]
            .trim_start()
            .strip_prefix(nom)
            .filter(|apres| !apres.starts_with(identifiant))
    })
}

/// Où en est une fonction suivie (`FONCTIONS_SUIVIES`) dans la lecture d'un fichier : pas
/// ouverte ; déclarée, son `fn` lu, sa première `{` pas encore ; ou dans son corps, à cette
/// profondeur d'accolades.
#[derive(Clone, Copy)]
enum Fonction {
    Dehors,
    Declaree,
    Corps(usize),
}

/// Avance `fonction`, la fonction suivie `nom`, sur la ligne nettoyée `nette`, et dit si la
/// ligne est dans `nom` : elle porte son `fn`, ou elle commence après lui et avant la `}` qui
/// ferme son corps, les accolades comptées depuis la première `{` qui suit ce `fn`. Un `;` lu
/// avant cette `{` (une déclaration sans corps) la referme.
fn suivre(fonction: &mut Fonction, nom: &str, nette: &str) -> bool {
    let mut dedans = !matches!(*fonction, Fonction::Dehors);
    let mut reste = nette;
    loop {
        match *fonction {
            Fonction::Dehors => {
                let Some(apres) = apres_la_fonction(reste, nom) else {
                    break;
                };
                dedans = true;
                *fonction = Fonction::Declaree;
                reste = apres;
            }
            Fonction::Declaree => {
                let Some(position) = reste.find(['{', ';']) else {
                    break;
                };
                *fonction = if reste[position..].starts_with('{') {
                    Fonction::Corps(1)
                } else {
                    Fonction::Dehors
                };
                reste = &reste[position + 1..];
            }
            Fonction::Corps(profondeur) => {
                let Some(position) = reste.find(['{', '}']) else {
                    break;
                };
                *fonction = match (reste[position..].starts_with('{'), profondeur) {
                    (true, p) => Fonction::Corps(p + 1),
                    (false, 1) => Fonction::Dehors,
                    (false, p) => Fonction::Corps(p - 1),
                };
                reste = &reste[position + 1..];
            }
        }
    }
    dedans
}

/// Les fautes d'I31 sur `sources`, des paires (chemin, texte) nommées depuis la racine du dépôt.
/// Chaque ligne se lit deux fois. Nettoyée (`nettoyer`), elle dit dans quelles fonctions suivies
/// elle est (`suivre`, pour chacune de `FONCTIONS_SUIVIES`) : un `}` ou un `fn date_du_jour`
/// écrit dans une chaîne ne compte pas. Telle quelle, elle se lit en sous-chaîne, sensible à la
/// casse : un mot écrit dans une chaîne compte ; un mot de `HORLOGE` ou de `DECALAGE_LOCAL` est
/// une faute hors des fonctions où `admis_dans` l'admet. Une ligne qui commence dans le code et
/// dont le début, blancs ôtés, est `//` (commentaire, doc) ne compte pas. Chaque faute nomme le
/// fichier, la ligne et le mot.
fn controler(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    for (chemin, texte) in sources {
        let mut inner_html_vus = 0;
        let mut lecture = Lecture::Code;
        let mut suivies = [Fonction::Dehors; FONCTIONS_SUIVIES.len()];
        for (indice, ligne) in texte.lines().enumerate() {
            let dans_le_code = lecture == Lecture::Code;
            let (nette, suite) = nettoyer(ligne, lecture);
            lecture = suite;
            if dans_le_code && ligne.trim_start().starts_with("//") {
                continue;
            }
            let dedans: Vec<&str> = FONCTIONS_SUIVIES
                .iter()
                .zip(suivies.iter_mut())
                .filter_map(|(nom, fonction)| suivre(fonction, nom, &nette).then_some(*nom))
                .collect();
            let mut faute = |mot: &str| {
                fautes.push(format!("{chemin}:{} contient « {mot} »", indice + 1));
            };
            for mot in HTML_POSE.iter().filter(|mot| ligne.contains(**mot)) {
                if *mot == "inner_html" && INNER_HTML_ADMIS.contains(&chemin.as_str()) {
                    inner_html_vus += ligne.matches(mot).count();
                    if inner_html_vus <= 1 {
                        continue;
                    }
                }
                faute(mot);
            }
            for mot in STYLE_EN_ATTRIBUT
                .iter()
                .chain(STOCKAGE)
                .filter(|mot| ligne.contains(**mot))
            {
                faute(mot);
            }
            for mot in HORLOGE
                .iter()
                .chain(DECALAGE_LOCAL)
                .filter(|mot| ligne.contains(**mot))
            {
                if !admis_dans(mot).iter().any(|admise| dedans.contains(admise)) {
                    faute(mot);
                }
            }
        }
    }
    fautes
}

/// Les fautes d'I29 (§ 70 · 3) : dans l'interface, ni greffon ni API de notification. `manifeste`
/// est le `Cargo.toml` de l'interface, `sources` ses sources, nommées depuis la racine du dépôt.
/// Une ligne de source dont le début, blancs ôtés, est `//`, et une ligne du manifeste dont le
/// début est `#`, ne comptent pas. Chaque mot de `NOTIFICATION` se lit en sous-chaîne, sensible à
/// la casse ; chaque faute nomme le fichier, la ligne et le mot.
fn controler_i29(manifeste: &str, sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    let mut examiner = |chemin: &str, texte: &str, commentaire: &str| {
        for (indice, ligne) in texte.lines().enumerate() {
            if ligne.trim_start().starts_with(commentaire) {
                continue;
            }
            for mot in NOTIFICATION.iter().filter(|mot| ligne.contains(**mot)) {
                fautes.push(format!("{chemin}:{} contient « {mot} »", indice + 1));
            }
        }
    };
    examiner(MANIFESTE, manifeste, "#");
    for (chemin, texte) in sources {
        examiner(chemin, texte, "//");
    }
    fautes
}

/// Une source inventée, nommée depuis la racine du dépôt.
fn source(chemin: &str, texte: &str) -> (String, String) {
    (chemin.to_string(), texte.to_string())
}

fn lire(chemin: &Path) -> String {
    fs::read_to_string(chemin).unwrap_or_else(|e| panic!("lecture de {} : {e}", chemin.display()))
}

/// Tous les fichiers `*.rs` sous `dossier`, en descendant dans les sous-dossiers.
fn fichiers_rs(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    for entree in fs::read_dir(dossier).expect("lecture du dossier") {
        let chemin = entree.expect("entrée").path();
        if chemin.is_dir() {
            fichiers_rs(&chemin, sortie);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            sortie.push(chemin);
        }
    }
}

/// `chemin`, relatif à `racine`, ses composants joints par `/`, sous Windows comme sous Linux.
fn relatif(racine: &Path, chemin: &Path) -> String {
    let reste = chemin
        .strip_prefix(racine)
        .expect("un chemin sous la racine");
    reste
        .components()
        .map(|composant| composant.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// La racine du dépôt, deux dossiers au-dessus de l'interface.
fn racine_du_depot() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("la racine du dépôt")
}

/// Les sources lues : chaque `.rs` de `app/ui/src/` et `app/ui/index.html`, nommés depuis la
/// racine du dépôt.
fn sources_du_depot() -> Vec<(String, String)> {
    let racine = racine_du_depot();
    let mut fichiers = Vec::new();
    fichiers_rs(&racine.join("app/ui/src"), &mut fichiers);
    fichiers.push(racine.join("app/ui/index.html"));
    fichiers.sort();
    fichiers
        .iter()
        .map(|f| (relatif(&racine, f), lire(f)))
        .collect()
}

/// `controler` refuse `sources` d'une seule faute, `attendue`, telle qu'il la dit : le fichier, la
/// ligne et le mot.
fn refuse(sources: &[(String, String)], attendue: &str) {
    assert_eq!(controler(sources), vec![attendue.to_string()]);
}

#[test]
fn i31_l_interface_ne_pose_ni_html_ni_style_ni_stockage() {
    let fautes = controler(&sources_du_depot());
    assert!(fautes.is_empty(), "{fautes:#?}");
}

#[test]
fn i31_chaque_lieu_lu_porte_des_sources() {
    let sources = sources_du_depot();
    for lieu in ["app/ui/src/", "app/ui/index.html"] {
        assert!(
            sources.iter().any(|(chemin, _)| chemin.starts_with(lieu)),
            "aucune source lue sous {lieu}"
        );
    }
}

#[test]
fn i31_chaque_controle_refuse_ce_qu_on_y_plante() {
    let table = "app/ui/src/vue_table.rs";
    let palette = "app/ui/src/panneau_palette.rs";
    let autre = "app/ui/src/autre.rs";
    let index = "app/ui/index.html";
    // Du HTML posé : un `inner_html` par fichier admis, un second est refusé sur sa ligne.
    assert!(
        controler(&[
            source(table, "let a = view! { <div inner_html=table/> };\n"),
            source(palette, "let b = view! { <div inner_html=palette/> };\n"),
        ])
        .is_empty(),
        "un inner_html admis est refusé"
    );
    refuse(
        &[source(
            table,
            "let a = view! { <div inner_html=table/> };\nlet b = view! { <p inner_html=texte/> };\n",
        )],
        "app/ui/src/vue_table.rs:2 contient « inner_html »",
    );
    refuse(
        &[source(
            autre,
            "let a = view! { <div inner_html=table/> };\n",
        )],
        "app/ui/src/autre.rs:1 contient « inner_html »",
    );
    refuse(
        &[source(autre, "<div prop:innerHTML=texte/>\n")],
        "app/ui/src/autre.rs:1 contient « innerHTML »",
    );
    refuse(
        &[source(autre, "element.set_outer_html(texte);\n")],
        "app/ui/src/autre.rs:1 contient « outer_html »",
    );
    refuse(
        &[source(
            autre,
            "element.insert_adjacent_html(\"beforeend\", t);\n",
        )],
        "app/ui/src/autre.rs:1 contient « insert_adjacent_html »",
    );
    // Un style en attribut : `style:` reste admis.
    refuse(
        &[source(index, "<div style=\"left: 0\"></div>\n")],
        "app/ui/index.html:1 contient « style= »",
    );
    refuse(
        &[source(autre, "<div style=format!(\"left: {x}\")/>\n")],
        "app/ui/src/autre.rs:1 contient « style= »",
    );
    refuse(
        &[source(autre, "element.set_attribute(\"style\", texte);\n")],
        "app/ui/src/autre.rs:1 contient « \"style\" »",
    );
    refuse(
        &[source(autre, "element.style().set_css_text(texte);\n")],
        "app/ui/src/autre.rs:1 contient « css_text »",
    );
    refuse(
        &[source(autre, "let t = style.cssText;\n")],
        "app/ui/src/autre.rs:1 contient « cssText »",
    );
    refuse(
        &[source(index, "<style>body { margin: 0 }</style>\n")],
        "app/ui/index.html:1 contient « <style »",
    );
    assert!(
        controler(&[source(
            autre,
            "<div style:left=gauche style:width=largeur/>\n"
        )])
        .is_empty(),
        "style: est refusé"
    );
    // Le stockage du navigateur, l'appel comme le nom écrit en chaîne.
    refuse(
        &[source(
            autre,
            "let s = Reflect::get(&fenetre, &\"localStorage\".into());\n",
        )],
        "app/ui/src/autre.rs:1 contient « localStorage »",
    );
    refuse(
        &[source(autre, "let s = window.local_storage();\n")],
        "app/ui/src/autre.rs:1 contient « local_storage »",
    );
    refuse(
        &[source(
            autre,
            "let s = Reflect::get(&f, &\"indexedDB\".into());\n",
        )],
        "app/ui/src/autre.rs:1 contient « indexedDB »",
    );
    refuse(
        &[source(autre, "let c = document.cookie;\n")],
        "app/ui/src/autre.rs:1 contient « cookie »",
    );
    // L'horloge : refusée hors de `date_du_jour`, admise dedans.
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let d = js_sys::Date::new_0();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « Date::new_0 »",
    );
    assert!(
        controler(&[source(
            autre,
            "fn date_du_jour() -> f64 {\n    let d = js_sys::Date::new_0();\n    d.get_time()\n}\n",
        )])
        .is_empty(),
        "date_du_jour est refusée"
    );
    // La fin de `date_du_jour` est la `}` qui ferme son corps : une lecture de l'horloge qui la
    // suit, sans autre `fn` avant elle, est refusée.
    refuse(
        &[source(
            autre,
            "fn date_du_jour() -> f64 {\n    js_sys::Date::new_0().get_time()\n}\nconst T: f64 = js_sys::Date::now();\n",
        )],
        "app/ui/src/autre.rs:4 contient « Date::now »",
    );
    // Un `}` écrit dans une chaîne ne ferme pas `date_du_jour`.
    assert!(
        controler(&[source(
            autre,
            "fn date_du_jour() -> f64 {\n    let s = \"}\";\n    js_sys::Date::new_0().get_time()\n}\n",
        )])
        .is_empty(),
        "une accolade dans une chaîne ferme date_du_jour"
    );
    // Un `fn date_du_jour` écrit dans une chaîne n'ouvre pas `date_du_jour`.
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let s = \"fn date_du_jour\";\n    let d = js_sys::Date::new_0();\n}\n",
        )],
        "app/ui/src/autre.rs:3 contient « Date::new_0 »",
    );
    // Ni un caractère, ni une chaîne brute, ni une chaîne sur plusieurs lignes, ni un commentaire
    // de fin de ligne ne ferment `date_du_jour` ; une durée de vie reste du code.
    assert!(
        controler(&[source(
            autre,
            "fn date_du_jour<'a>(x: &'a str) -> f64 {\n    let o = '}';\n    let e = '\\'';\n    let r = r#\"}\"#;\n    let m = \"\n}\n\";\n    let n = 1; // }\n    js_sys::Date::now()\n}\n",
        )])
        .is_empty(),
        "un caractère, une chaîne ou un commentaire ferme date_du_jour"
    );
    // Une ligne qui commence dans une chaîne n'est pas un commentaire, même si elle commence par
    // `//`.
    refuse(
        &[source(autre, "let s = \"\n// localStorage\n\";\n")],
        "app/ui/src/autre.rs:2 contient « localStorage »",
    );
    // Le décalage local : refusé dans une autre fonction, comme dans `date_du_jour`, qui ne lit
    // que la date.
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let z = d.getTimezoneOffset();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « getTimezoneOffset »",
    );
    refuse(
        &[source(
            autre,
            "fn date_du_jour() {\n    let z = d.getTimezoneOffset();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « getTimezoneOffset »",
    );
    // Admis : l'horloge et le décalage dans `decalage_du_geste`, le décalage dans `decalage_a`, la
    // date à l'heure locale dans `date_du_jour`.
    assert!(
        controler(&[source(
            autre,
            "fn decalage_du_geste() -> i32 {\n    decalage_depuis_js(js_sys::Date::new_0().get_timezone_offset())\n}\nfn decalage_a(le: i64) -> i32 {\n    let d = js_sys::Date::new(&JsValue::from_f64(le as f64 * 1000.0));\n    decalage_depuis_js(d.get_timezone_offset())\n}\nfn date_du_jour() -> String {\n    let d = js_sys::Date::new_0();\n    date_iso(d.get_full_year(), d.get_month() + 1, d.get_date())\n}\n",
        )])
        .is_empty(),
        "decalage_du_geste, decalage_a ou date_du_jour est refusée"
    );
    // Chaque mot n'est admis que dans ses fonctions : l'horloge, pas dans `decalage_a` ; la date à
    // l'heure locale, pas dans `decalage_du_geste`.
    refuse(
        &[source(
            autre,
            "fn decalage_a(le: i64) -> i32 {\n    let d = js_sys::Date::new_0();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « Date::new_0 »",
    );
    refuse(
        &[source(
            autre,
            "fn decalage_du_geste() -> i32 {\n    let h = d.get_hours();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « get_hours »",
    );
    // La fin de `decalage_du_geste` et celle de `decalage_a` sont la `}` qui ferme leur corps.
    refuse(
        &[source(
            autre,
            "fn decalage_du_geste() -> i32 {\n    1\n}\nfn autre() {\n    let d = js_sys::Date::new_0();\n}\n",
        )],
        "app/ui/src/autre.rs:5 contient « Date::new_0 »",
    );
    refuse(
        &[source(
            autre,
            "fn decalage_a(le: i64) -> i32 {\n    d.get_timezone_offset()\n}\nfn autre() {\n    let z = d.getTimezoneOffset();\n}\n",
        )],
        "app/ui/src/autre.rs:5 contient « getTimezoneOffset »",
    );
    // La date à l'heure locale, hors de `date_du_jour` : l'heure, un texte à la manière du pays.
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let h = d.get_hours();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « get_hours »",
    );
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let t = d.to_locale_date_string(\"fr\", &o);\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « to_locale »",
    );
    // L'heure en texte, et une date construite à l'heure locale, hors de `date_du_jour`.
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let t = d.to_time_string();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « to_time_string »",
    );
    refuse(
        &[source(
            autre,
            "fn autre() {\n    let d = js_sys::Date::new_with_year_month_day(2026, 10, 1);\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « new_with_year_month_day »",
    );
    // Dans `date_du_jour`, elles sont admises comme les autres lectures à l'heure locale.
    assert!(
        controler(&[source(
            autre,
            "fn date_du_jour() -> String {\n    let d = js_sys::Date::new_0();\n    String::from(d.to_date_string())\n}\n",
        )])
        .is_empty(),
        "to_date_string est refusée dans date_du_jour"
    );
    // Une lecture en UTC n'est pas une lecture à l'heure locale.
    assert!(
        controler(&[source(
            autre,
            "fn autre() {\n    let j = d.getUTCDate();\n    let k = d.get_utc_date();\n}\n",
        )])
        .is_empty(),
        "une lecture en UTC est refusée"
    );
    // Un mot dans un commentaire ou une doc ne compte pas.
    assert!(
        controler(&[source(
            autre,
            "/// localStorage\n    // inner_html\nfn f() {}\n"
        )])
        .is_empty(),
        "un commentaire est refusé"
    );
}

#[test]
fn i29_ni_greffon_ni_api_de_notification_dans_l_interface() {
    let manifeste = lire(&racine_du_depot().join(MANIFESTE));
    let fautes = controler_i29(&manifeste, &sources_du_depot());
    assert!(fautes.is_empty(), "{fautes:#?}");

    let autre = "app/ui/src/autre.rs";
    let sain = "[dependencies]\nleptos = \"0.8.23\"\n";
    let aucune = Vec::<(String, String)>::new();
    assert!(controler_i29(sain, &aucune).is_empty());
    // L'API de notification, dans une source et dans la page.
    assert_eq!(
        controler_i29(
            sain,
            &[source(
                autre,
                "let n = web_sys::Notification::new(\"Bonjour\");\n"
            )]
        ),
        vec!["app/ui/src/autre.rs:1 contient « Notification »".to_string()]
    );
    assert_eq!(
        controler_i29(
            sain,
            &[source(
                "app/ui/index.html",
                "<script>Notification.requestPermission();</script>\n"
            )]
        ),
        vec!["app/ui/index.html:1 contient « Notification »".to_string()]
    );
    // Le greffon, ou la fonction de `web-sys` qui porte l'API, dans le manifeste.
    assert_eq!(
        controler_i29(
            "[dependencies]\ntauri-plugin-notification = \"2\"\n",
            &aucune
        ),
        vec!["app/ui/Cargo.toml:2 contient « notification »".to_string()]
    );
    assert_eq!(
        controler_i29(
            "[dependencies]\nweb-sys = { version = \"0.3\", features = [\"Notification\"] }\n",
            &aucune
        ),
        vec!["app/ui/Cargo.toml:2 contient « Notification »".to_string()]
    );
    // Une boîte de message de la WebView, chacune refusée par son mot ; le texte piégé des tests,
    // `alert(1)`, n'en appelle aucune.
    for (texte, mot) in [
        (
            "fenetre.alert_with_message(\"Refusé\").ok();\n",
            "alert_with_message",
        ),
        (
            "let oui = fenetre.confirm_with_message(\"Retirer ?\");\n",
            "confirm_with_message",
        ),
        (
            "let texte = fenetre.prompt_with_message(\"Texte\");\n",
            "prompt_with_message",
        ),
    ] {
        assert_eq!(
            controler_i29(sain, &[source(autre, texte)]),
            vec![format!("{autre}:1 contient « {mot} »")],
            "{texte}"
        );
    }
    assert!(
        controler_i29(
            sain,
            &[source(
                autre,
                "let piege = \"<img src=x onerror=alert(1)>\";\n"
            )]
        )
        .is_empty()
    );
    // Les lignes `//` des sources, et `#` du manifeste, ne comptent pas.
    assert!(
        controler_i29(
            "# notification\n[dependencies]\n",
            &[source(
                autre,
                "/// aucune notification (I29)\n    // web_sys::Notification\nfn f() {}\n"
            )]
        )
        .is_empty()
    );
}
