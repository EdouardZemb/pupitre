//! Contrôles statiques de l'interface : le mode encre tient (I17), sans couleur hors du noir et
//! du blanc, et rien ne bouge de soi-même (I18). Les tests lisent les sources sur le disque ;
//! aucun ne lance l'application.

use std::fs;
use std::path::{Path, PathBuf};

fn racine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

/// Retire les commentaires `/* … */` d'une feuille de style.
fn sans_commentaires_css(css: &str) -> String {
    let mut sortie = String::new();
    let mut reste = css;
    while let Some(debut) = reste.find("/*") {
        sortie.push_str(&reste[..debut]);
        match reste[debut + 2..].find("*/") {
            Some(fin) => reste = &reste[debut + 2 + fin + 2..],
            None => return sortie,
        }
    }
    sortie.push_str(reste);
    sortie
}

/// Une règle CSS : ses sélecteurs et ses déclarations, chacune sa propriété et sa valeur.
struct Regle {
    selecteurs: Vec<String>,
    declarations: Vec<(String, String)>,
}

impl Regle {
    /// Les propriétés que la règle déclare.
    fn proprietes(&self) -> impl Iterator<Item = &str> {
        self.declarations.iter().map(|(p, _)| p.as_str())
    }
}

/// Lit les règles d'une feuille de style. Analyseur minimal, juste assez pour ce dépôt : il
/// retire les commentaires, coupe sur `{` et `}`, les sélecteurs sur `,` et les déclarations sur
/// `;` (la propriété est avant le premier `:`, la valeur après). Un bloc `@media` (ou tout `@…`
/// qui contient des blocs) n'est pas une règle : on l'ouvre et ses règles imbriquées comptent.
///
/// Ce qu'il ne gère pas, faute d'en avoir besoin ici : le CSS imbriqué (`&:hover { … }` dans une
/// règle), une chaîne qui contient `{`, `}` ou `;`, et un sélecteur d'attribut comme
/// `[class*="sem-"]` (seul le texte `.sem-` d'un sélecteur est reconnu). Si la feuille de style
/// en vient à en employer, il faut étendre l'analyseur avant de s'y fier.
fn regles(css: &str) -> Vec<Regle> {
    let css = sans_commentaires_css(css);
    let mut resultat = Vec::new();
    let mut tete = String::new();
    let mut corps = String::new();
    // Vrai tant qu'on lit les déclarations d'une règle (entre son `{` et son `}`).
    let mut dans_regle = false;
    for c in css.chars() {
        match c {
            '{' if !dans_regle => {
                if tete.trim_start().starts_with('@') {
                    // Bloc `@media` : on l'ouvre, ses règles viennent ensuite.
                    tete.clear();
                } else {
                    dans_regle = true;
                }
            }
            '}' if dans_regle => {
                resultat.push(Regle {
                    selecteurs: tete.split(',').map(|s| s.trim().to_string()).collect(),
                    declarations: corps
                        .split(';')
                        .filter_map(|d| d.split_once(':'))
                        .map(|(p, v)| (p.trim().to_string(), v.trim().to_string()))
                        .collect(),
                });
                tete.clear();
                corps.clear();
                dans_regle = false;
            }
            '}' => tete.clear(), // fin d'un bloc `@media`
            _ if dans_regle => corps.push(c),
            _ => tete.push(c),
        }
    }
    resultat
}

const COULEURS: &[&str] = &[
    "color",
    "background",
    "background-color",
    "fill",
    "stroke",
    "border-color",
    "outline-color",
];

const INDICES: &[&str] = &[
    "border",
    "border-style",
    "border-width",
    "outline",
    "text-decoration",
    "font-style",
    "font-weight",
    "background-image",
    "content",
    "stroke-dasharray",
    "stroke-width",
    "margin-top",
    "top",
    "transform",
];

/// Contrôle I17 : toute règle dont un sélecteur contient `.sem-` et qui colore doit aussi
/// déclarer un indice non colorimétrique. L'erreur nomme chaque sélecteur fautif.
fn controler_classes_sem(css: &str) -> Result<(), String> {
    let mut erreurs = Vec::new();
    for regle in regles(css) {
        let colore = regle.proprietes().any(|p| COULEURS.contains(&p));
        let indice = regle.proprietes().any(|p| INDICES.contains(&p));
        if colore && !indice {
            for sel in regle.selecteurs.iter().filter(|s| s.contains(".sem-")) {
                erreurs.push(format!(
                    "{sel} : une couleur sans indice non colorimétrique"
                ));
            }
        }
    }
    if erreurs.is_empty() {
        Ok(())
    } else {
        Err(erreurs.join("\n"))
    }
}

/// Les propriétés dont la valeur peut porter une couleur, et que le contrôle du noir et du blanc
/// examine ; toute propriété `border-…` s'y ajoute.
const PROPRIETES_COLOREES: &[&str] = &[
    "color",
    "background",
    "background-color",
    "border",
    "border-color",
    "outline",
    "outline-color",
    "fill",
    "stroke",
    "box-shadow",
];

/// Les seules valeurs de couleur permises, en minuscules : le noir, le blanc, et ce qui n'en
/// ajoute aucune.
const COULEURS_PERMISES: &[&str] = &[
    "#000",
    "#fff",
    "#000000",
    "#ffffff",
    "black",
    "white",
    "transparent",
    "currentcolor",
    "none",
    "inherit",
];

/// Les noms de couleur courants, en minuscules : les couleurs de base du CSS, leurs variantes
/// les plus employées et les couleurs du système.
const NOMS_DE_COULEUR: &[&str] = &[
    "black",
    "white",
    "red",
    "green",
    "blue",
    "yellow",
    "orange",
    "purple",
    "gray",
    "grey",
    "silver",
    "maroon",
    "navy",
    "teal",
    "olive",
    "lime",
    "aqua",
    "fuchsia",
    "pink",
    "brown",
    "cyan",
    "magenta",
    "gold",
    "violet",
    "indigo",
    "beige",
    "crimson",
    "coral",
    "salmon",
    "tomato",
    "khaki",
    "lavender",
    "turquoise",
    "darkgray",
    "darkgrey",
    "lightgray",
    "lightgrey",
    "dimgray",
    "dimgrey",
    "gainsboro",
    "whitesmoke",
    "graytext",
    "highlight",
    "canvas",
    "canvastext",
];

/// Les fonctions qui écrivent une couleur.
const FONCTIONS_DE_COULEUR: &[&str] = &["rgb", "rgba", "hsl", "hsla"];

/// Vrai si la propriété peut porter une couleur.
fn est_coloree(propriete: &str) -> bool {
    // Un nom de propriété ne tient pas compte de la casse.
    let propriete = propriete.to_ascii_lowercase();
    PROPRIETES_COLOREES.contains(&propriete.as_str()) || propriete.starts_with("border-")
}

/// Les couleurs qu'une valeur écrit, en minuscules : chaque `#…`, chaque appel de `rgb`,
/// `rgba`, `hsl` ou `hsla`, et chaque nom de couleur courant. Les autres mots (`solid`, `1px`)
/// ne sont pas des couleurs.
fn couleurs_de(valeur: &str) -> Vec<String> {
    let valeur = valeur.to_lowercase();
    let mut couleurs = Vec::new();
    let mut caracteres = valeur.char_indices().peekable();
    while let Some((debut, c)) = caracteres.next() {
        if c != '#' && !c.is_ascii_alphabetic() {
            continue;
        }
        // Le mot : `#` ou une lettre, puis des lettres, des chiffres et des tirets.
        let mut fin = debut + c.len_utf8();
        while let Some(&(i, d)) = caracteres.peek() {
            if !(d.is_ascii_alphanumeric() || d == '-') {
                break;
            }
            fin = i + d.len_utf8();
            caracteres.next();
        }
        let mot = &valeur[debut..fin];
        let appel = caracteres.peek().is_some_and(|&(_, d)| d == '(');
        if c == '#' || (!appel && NOMS_DE_COULEUR.contains(&mot)) {
            couleurs.push(mot.to_string());
        } else if appel && FONCTIONS_DE_COULEUR.contains(&mot) {
            couleurs.push(format!("{mot}(…)"));
        }
    }
    couleurs
}

/// Contrôle du mode encre : dans chaque règle, chaque propriété qui peut porter une couleur
/// n'en écrit aucune hors du noir et du blanc. La feuille de style est lue sans ses
/// commentaires. Rend le nombre de déclarations examinées, ou une erreur qui nomme chaque
/// déclaration fautive.
fn controler_couleurs(css: &str) -> Result<usize, String> {
    let mut examinees = 0;
    let mut erreurs = Vec::new();
    for regle in regles(css) {
        for (propriete, valeur) in &regle.declarations {
            if !est_coloree(propriete) {
                continue;
            }
            examinees += 1;
            for couleur in couleurs_de(valeur) {
                if !COULEURS_PERMISES.contains(&couleur.as_str()) {
                    erreurs.push(format!(
                        "{} : {propriete}: {valeur} : « {couleur} », hors du noir et du blanc",
                        regle.selecteurs.join(", ")
                    ));
                }
            }
        }
    }
    if erreurs.is_empty() {
        Ok(examinees)
    } else {
        Err(erreurs.join("\n"))
    }
}

#[test]
fn i18_aucune_animation_ni_minuterie() {
    const MOTS: &[&str] = &[
        "animation",
        "transition",
        "@keyframes",
        "set_timeout",
        "set_interval",
        "setTimeout",
        "setInterval",
        "request_animation_frame",
        "requestAnimationFrame",
        "<animate",
        "scroll-behavior: smooth",
    ];
    // La recherche se fait en sous-chaîne, sensible à la casse, sur la liste exacte ci-dessus : un
    // identifiant comme `transitions_de_page` la fait échouer, et c'est voulu.
    // La feuille de style est lue sans ses commentaires (le sien dit « rien ne bouge » et peut
    // citer ces mots) ; les sources Rust et la page HTML sont lues telles quelles, commentaires
    // compris : aucun n'a de raison de les nommer.
    let mut textes = vec![
        (
            "style.css".to_string(),
            sans_commentaires_css(&lire(&racine().join("style.css"))),
        ),
        ("index.html".to_string(), lire(&racine().join("index.html"))),
    ];
    let mut sources = Vec::new();
    fichiers_rs(&racine().join("src"), &mut sources);
    assert!(!sources.is_empty(), "aucune source Rust trouvée");
    for s in sources {
        textes.push((s.display().to_string(), lire(&s)));
    }
    let mut fautes = Vec::new();
    for (nom, texte) in &textes {
        for mot in MOTS {
            if texte.contains(mot) {
                fautes.push(format!("{nom} contient « {mot} »"));
            }
        }
    }
    assert!(fautes.is_empty(), "{}", fautes.join("\n"));
}

#[test]
fn i17_toute_classe_sem_qui_colore_a_un_indice_non_colorimetrique() {
    let css = lire(&racine().join("style.css"));
    // Garde-fou : un analyseur cassé ne doit pas laisser le test vert à vide.
    assert!(
        regles(&css)
            .iter()
            .any(|r| r.selecteurs.iter().any(|s| s == ".page svg")),
        "l'analyseur ne retrouve pas la règle `.page svg` de style.css"
    );
    if let Err(e) = controler_classes_sem(&css) {
        panic!("{e}");
    }
}

#[test]
fn la_feuille_de_style_n_a_aucune_couleur_hors_du_noir_et_du_blanc() {
    let css = lire(&racine().join("style.css"));
    match controler_couleurs(&css) {
        // Garde-fou : un analyseur cassé ne doit pas laisser le test vert à vide. La feuille de
        // style colore au moins `html, body`, `button` et `:focus-visible`.
        Ok(examinees) => assert!(examinees >= 5, "{examinees} déclarations examinées"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn le_controle_du_noir_et_du_blanc_refuse_une_couleur() {
    let rouge = controler_couleurs(".x { color: #c00 }").unwrap_err();
    assert!(
        rouge.contains(".x") && rouge.contains("« #c00 »"),
        "{rouge}"
    );
    let cadre = controler_couleurs(".y { border: 1px solid red }").unwrap_err();
    assert!(cadre.contains(".y") && cadre.contains("« red »"), "{cadre}");
    // Les fonctions de couleur, une propriété `border-…`, une règle dans un `@media` : la casse
    // ne compte pas, ni celle des valeurs, ni celle des propriétés.
    for (css, couleur) in [
        (".a { background-color: rgb(0 0 0) }", "rgb(…)"),
        (".b { box-shadow: 0 0 2px HSLA(0, 0%, 0%, 0.5) }", "hsla(…)"),
        (".c { border-left-color: Grey }", "grey"),
        ("@media print { .d { fill: #808080 } }", "#808080"),
        (
            ".e { stroke: hsl(0 0% 0%); outline: 1px solid rgba(0, 0, 0, 1) }",
            "rgba(…)",
        ),
        (".f { Border-Top: 1px solid Navy }", "navy"),
    ] {
        let e = controler_couleurs(css).unwrap_err();
        assert!(e.contains(&format!("« {couleur} »")), "{css} : {e}");
    }
    // Il accepte le noir, le blanc, et ce qui n'ajoute aucune couleur, quelle que soit la casse ;
    // les mots qui ne sont pas des couleurs (`solid`, `dashed`) ne comptent pas. Un commentaire
    // ne compte pas non plus.
    let permis = concat!(
        ".z { color: #000; background: #FFF; background-color: #ffffff; ",
        "border: 1px solid black; border-bottom: 2px dashed White; outline: none; ",
        "outline-color: #000000; fill: currentColor; stroke: transparent; ",
        "box-shadow: none; border-color: inherit /* et non red */ }"
    );
    assert_eq!(controler_couleurs(permis), Ok(11));
    // Une propriété qui ne porte pas de couleur n'est pas examinée.
    assert_eq!(controler_couleurs(".w { font-family: red }"), Ok(0));
}

#[test]
fn i17_le_controle_refuse_une_classe_coloree_seule() {
    let erreur = controler_classes_sem(".sem-x { color: #c00 }").unwrap_err();
    assert!(erreur.contains(".sem-x"), "{erreur}");
    // Et il accepte la même couleur accompagnée d'un indice, y compris dans un `@media`.
    assert!(controler_classes_sem(".sem-x { color: #c00; font-weight: bold }").is_ok());
    assert!(
        controler_classes_sem("@media print { .sem-y, .autre { fill: #c00 } }")
            .unwrap_err()
            .contains(".sem-y")
    );
}
