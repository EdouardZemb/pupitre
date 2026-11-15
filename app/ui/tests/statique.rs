//! Contrôle statique d'I31 côté interface (§ 70 · 3) : l'interface ne pose ni HTML, ni style en
//! attribut, ni stockage du navigateur, ne lit l'horloge que dans `date_du_jour`, et ne lit pas le
//! décalage local. Le test lit sur le disque chaque `.rs` de `src/` et `index.html` ; un lieu dont
//! aucune source n'est lue est une faute.

use std::fs;
use std::path::{Path, PathBuf};

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
    let page = "app/ui/src/vue_table.rs";
    let fiche = "app/ui/src/panneau_palette.rs";
    let autre = "app/ui/src/autre.rs";
    let index = "app/ui/index.html";
    // Du HTML posé : un `inner_html` par fichier admis, un second est refusé sur sa ligne.
    assert!(
        controler(&[
            source(page, "let a = view! { <div inner_html=page/> };\n"),
            source(fiche, "let b = view! { <div inner_html=fiche/> };\n"),
        ])
        .is_empty(),
        "un inner_html admis est refusé"
    );
    refuse(
        &[source(
            page,
            "let a = view! { <div inner_html=page/> };\nlet b = view! { <p inner_html=texte/> };\n",
        )],
        "app/ui/src/vue_table.rs:2 contient « inner_html »",
    );
    refuse(
        &[source(autre, "let a = view! { <div inner_html=page/> };\n")],
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
