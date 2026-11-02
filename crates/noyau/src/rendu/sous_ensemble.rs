//! Le contrôle du sous-ensemble de SVG, sans dépendance.

use std::collections::BTreeSet;

const ELEMENTS: [&str; 6] = ["svg", "g", "path", "rect", "circle", "line"];
const ATTRIBUTS: [&str; 22] = [
    "xmlns",
    "width",
    "height",
    "viewBox",
    "x",
    "y",
    "cx",
    "cy",
    "r",
    "x1",
    "y1",
    "x2",
    "y2",
    "d",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "fill",
    "fill-rule",
    "transform",
    "id",
];
const COULEURS: [&str; 3] = ["#000", "#fff", "none"];

/// Relève les éléments, attributs et couleurs hors du sous-ensemble, et les
/// identifiants en double. Chaque écart est nommé.
pub fn verifier(svg: &str) -> Result<(), Vec<String>> {
    let mut erreurs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut reste = svg;
    while let Some(debut) = reste.find('<') {
        let apres = &reste[debut + 1..];
        let Some(fin) = fin_de_balise(apres) else {
            erreurs.push("balise non fermée".to_string());
            break;
        };
        let balise = &apres[..fin];
        reste = &apres[fin + 1..];
        if balise.starts_with('/') {
            continue;
        }
        if balise.starts_with(['!', '?']) {
            erreurs.push(format!("balise non admise : <{balise}>"));
            continue;
        }
        verifier_balise(balise.trim_end_matches('/'), &mut ids, &mut erreurs);
    }
    if erreurs.is_empty() {
        Ok(())
    } else {
        Err(erreurs)
    }
}

/// La position du `>` qui ferme la balise, hors des guillemets.
fn fin_de_balise(s: &str) -> Option<usize> {
    let mut guillemet = None;
    for (i, c) in s.char_indices() {
        match (guillemet, c) {
            (None, '"' | '\'') => guillemet = Some(c),
            (Some(g), c) if c == g => guillemet = None,
            (None, '>') => return Some(i),
            _ => {}
        }
    }
    None
}

fn verifier_balise(balise: &str, ids: &mut BTreeSet<String>, erreurs: &mut Vec<String>) {
    let nom_fin = balise.find(char::is_whitespace).unwrap_or(balise.len());
    let nom = &balise[..nom_fin];
    if !ELEMENTS.contains(&nom) {
        erreurs.push(format!("élément non admis : {nom}"));
    }
    let mut reste = balise[nom_fin..].trim_start();
    while !reste.is_empty() {
        let Some(egal) = reste.find('=') else {
            erreurs.push(format!("attribut mal formé : {reste}"));
            return;
        };
        let attribut = reste[..egal].trim();
        let apres = reste[egal + 1..].trim_start();
        let Some(guillemet) = apres.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
            erreurs.push(format!("valeur sans guillemets : {attribut}"));
            return;
        };
        let Some(longueur) = apres[1..].find(guillemet) else {
            erreurs.push(format!("valeur non fermée : {attribut}"));
            return;
        };
        let valeur = &apres[1..=longueur];
        reste = apres[longueur + 2..].trim_start();
        if !ATTRIBUTS.contains(&attribut) {
            erreurs.push(format!("attribut non admis : {attribut} (dans {nom})"));
        } else if matches!(attribut, "fill" | "stroke") && !COULEURS.contains(&valeur) {
            erreurs.push(format!("couleur non admise : {valeur}"));
        } else if attribut == "id" && !ids.insert(valeur.to_string()) {
            erreurs.push(format!("identifiant en double : {valeur}"));
        }
    }
}
