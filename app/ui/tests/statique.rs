//! Contrôle statique d'I31 côté interface (§ 70 · 3) : l'interface ne pose ni HTML, ni style en
//! attribut, ni stockage du navigateur, ne lit l'horloge que dans `date_du_jour`, et ne lit pas le
//! décalage local. Le test lit sur le disque chaque `.rs` de `src/` et `index.html` ; un lieu dont
//! aucune source n'est lue est une faute.

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

/// L'horloge, admise dans la seule fonction `FONCTION_DE_L_HORLOGE`.
const HORLOGE: &[&str] = &["Date::new_0", "Date::now", "performance"];

/// La seule fonction où l'interface lit l'horloge.
const FONCTION_DE_L_HORLOGE: &str = "date_du_jour";

/// Le décalage local, nulle part jusqu'à la tâche 18.
const DECALAGE_LOCAL: &[&str] = &["get_timezone_offset", "getTimezoneOffset"];

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

/// Où en est `FONCTION_DE_L_HORLOGE` dans la lecture d'un fichier : pas ouverte ; déclarée, son
/// `fn` lu, sa première `{` pas encore ; ou dans son corps, à cette profondeur d'accolades.
#[derive(Clone, Copy)]
enum Horloge {
    Dehors,
    Declaree,
    Corps(usize),
}

/// Avance `horloge` sur la ligne nettoyée `nette`, et dit si la ligne est dans
/// `FONCTION_DE_L_HORLOGE` : elle porte son `fn`, ou elle commence après lui et avant la `}` qui
/// ferme son corps, les accolades comptées depuis la première `{` qui suit ce `fn`. Un `;` lu
/// avant cette `{` (une déclaration sans corps) la referme.
fn suivre(horloge: &mut Horloge, nette: &str) -> bool {
    let mut dedans = !matches!(*horloge, Horloge::Dehors);
    let mut reste = nette;
    loop {
        match *horloge {
            Horloge::Dehors => {
                let Some(apres) = apres_la_fonction(reste, FONCTION_DE_L_HORLOGE) else {
                    break;
                };
                dedans = true;
                *horloge = Horloge::Declaree;
                reste = apres;
            }
            Horloge::Declaree => {
                let Some(position) = reste.find(['{', ';']) else {
                    break;
                };
                *horloge = if reste[position..].starts_with('{') {
                    Horloge::Corps(1)
                } else {
                    Horloge::Dehors
                };
                reste = &reste[position + 1..];
            }
            Horloge::Corps(profondeur) => {
                let Some(position) = reste.find(['{', '}']) else {
                    break;
                };
                *horloge = match (reste[position..].starts_with('{'), profondeur) {
                    (true, p) => Horloge::Corps(p + 1),
                    (false, 1) => Horloge::Dehors,
                    (false, p) => Horloge::Corps(p - 1),
                };
                reste = &reste[position + 1..];
            }
        }
    }
    dedans
}

/// Les fautes d'I31 sur `sources`, des paires (chemin, texte) nommées depuis la racine du dépôt.
/// Chaque ligne se lit deux fois. Nettoyée (`nettoyer`), elle dit où est `date_du_jour`
/// (`suivre`) : un `}` ou un `fn date_du_jour` écrit dans une chaîne ne compte pas. Telle quelle,
/// elle se lit en sous-chaîne, sensible à la casse : un mot écrit dans une chaîne compte. Une ligne
/// qui commence dans le code et dont le début, blancs ôtés, est `//` (commentaire, doc) ne compte
/// pas. Chaque faute nomme le fichier, la ligne et le mot.
fn controler(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    for (chemin, texte) in sources {
        let mut inner_html_vus = 0;
        let mut lecture = Lecture::Code;
        let mut horloge = Horloge::Dehors;
        for (indice, ligne) in texte.lines().enumerate() {
            let dans_le_code = lecture == Lecture::Code;
            let (nette, suite) = nettoyer(ligne, lecture);
            lecture = suite;
            if dans_le_code && ligne.trim_start().starts_with("//") {
                continue;
            }
            let dans_l_horloge = suivre(&mut horloge, &nette);
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
                .chain(DECALAGE_LOCAL)
                .filter(|mot| ligne.contains(**mot))
            {
                faute(mot);
            }
            if !dans_l_horloge {
                for mot in HORLOGE.iter().filter(|mot| ligne.contains(**mot)) {
                    faute(mot);
                }
            }
        }
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
        "un } dans une chaîne ferme date_du_jour"
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
    // Le décalage local : nulle part.
    refuse(
        &[source(
            autre,
            "fn date_du_jour() {\n    let z = d.getTimezoneOffset();\n}\n",
        )],
        "app/ui/src/autre.rs:2 contient « getTimezoneOffset »",
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
