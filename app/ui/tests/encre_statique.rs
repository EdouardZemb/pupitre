//! Contrôles statiques de l'interface : le mode encre tient (I17) et rien ne bouge de soi-même
//! (I18). Les tests lisent les sources sur le disque ; aucun ne lance l'application.

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

/// Une règle CSS : ses sélecteurs et les propriétés qu'elle déclare.
struct Regle {
    selecteurs: Vec<String>,
    proprietes: Vec<String>,
}

/// Lit les règles d'une feuille de style. Analyseur minimal, juste assez pour ce dépôt : il
/// retire les commentaires, coupe sur `{` et `}`, les sélecteurs sur `,` et les déclarations sur
/// `;` (la propriété est avant le premier `:`). Un bloc `@media` (ou tout `@…` qui contient
/// des blocs) n'est pas une règle : on l'ouvre et ses règles imbriquées comptent.
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
                    proprietes: corps
                        .split(';')
                        .filter_map(|d| d.split_once(':'))
                        .map(|(p, _)| p.trim().to_string())
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
        let colore = regle
            .proprietes
            .iter()
            .any(|p| COULEURS.contains(&p.as_str()));
        let indice = regle
            .proprietes
            .iter()
            .any(|p| INDICES.contains(&p.as_str()));
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
