//! Contrôle d'I31 (§ 70 · 3), les droits : l'interface n'appelle que les commandes que les droits
//! nomment. Le test relit `app/src-tauri/capabilities/` et `build.rs` sur le disque : seul
//! `windows.json` porte des droits, ils nomment chaque commande de `commandes::NOMS`, une à une,
//! et aucun droit du noyau de Tauri ni d'un greffon ; `build.rs` tire le manifeste de l'application
//! de `commandes::NOMS`, sans droit par défaut. Chaque contrôle est une fonction pure, éprouvée sur
//! des entrées plantées.

use serde_json::{Value, json};
use std::fs;
use std::path::Path;

/// Le seul fichier de droits : Windows est la seule plateforme qui appelle des commandes.
const FICHIER_DES_DROITS: &str = "windows.json";

/// Les fichiers de `capabilities/` : seul `windows.json`.
fn controler_les_fichiers_de_droits(noms: &[String]) -> Vec<String> {
    let mut fautes = Vec::new();
    for nom in noms {
        if nom != FICHIER_DES_DROITS {
            fautes.push(format!(
                "capabilities/{nom} : seul {FICHIER_DES_DROITS} porte des droits"
            ));
        }
    }
    if !noms.iter().any(|nom| nom == FICHIER_DES_DROITS) {
        fautes.push(format!("capabilities/{FICHIER_DES_DROITS} est absent"));
    }
    fautes
}

/// Les droits de `windows.json` : `platforms` vaut `["windows"]`, `windows` vaut `["main"]`, et
/// `permissions` égale, comme ensemble, `allow-` suivi de chaque nom de `noms`, chaque `_` devenu
/// `-` ; aucun droit qui porte `:` (un droit du noyau de Tauri ou d'un greffon).
fn controler_les_droits(droits: &Value, noms: &[&str]) -> Vec<String> {
    let mut fautes = Vec::new();
    if droits.get("platforms") != Some(&json!(["windows"])) {
        fautes.push("platforms ne vaut pas [\"windows\"]".to_string());
    }
    if droits.get("windows") != Some(&json!(["main"])) {
        fautes.push("windows ne vaut pas [\"main\"]".to_string());
    }
    let donnes: Vec<&str> = match droits.get("permissions").and_then(Value::as_array) {
        Some(liste) => liste.iter().filter_map(Value::as_str).collect(),
        None => {
            fautes.push("permissions est absent ou n'est pas une liste".to_string());
            return fautes;
        }
    };
    for droit in &donnes {
        if droit.contains(':') {
            fautes.push(format!(
                "{droit} : un droit du noyau de Tauri ou d'un greffon"
            ));
        }
    }
    let attendus: Vec<String> = noms
        .iter()
        .map(|nom| format!("allow-{}", nom.replace('_', "-")))
        .collect();
    for attendu in &attendus {
        if !donnes.contains(&attendu.as_str()) {
            fautes.push(format!("{attendu} manque"));
        }
    }
    for droit in &donnes {
        if !droit.contains(':') && !attendus.iter().any(|attendu| attendu == droit) {
            fautes.push(format!("{droit} ne nomme aucune commande"));
        }
    }
    fautes
}

/// `build.rs` : `AppManifest::new().commands(commandes::NOMS)`, blancs ignorés, et aucun
/// `default_permission`, qui permettrait tout.
fn controler_le_manifeste(build_rs: &str) -> Vec<String> {
    let mut fautes = Vec::new();
    let sans_blancs: String = build_rs.chars().filter(|c| !c.is_whitespace()).collect();
    if !sans_blancs.contains("AppManifest::new().commands(commandes::NOMS)") {
        fautes.push(
            "build.rs ne tire pas le manifeste de l'application de commandes::NOMS".to_string(),
        );
    }
    if sans_blancs.contains("default_permission") {
        fautes.push("build.rs pose un default_permission, qui permettrait tout".to_string());
    }
    fautes
}

fn dossier_de_l_etabli() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn noms_des_fichiers_de_droits() -> Vec<String> {
    let dossier = dossier_de_l_etabli().join("capabilities");
    fs::read_dir(&dossier)
        .unwrap_or_else(|e| panic!("lecture de {} : {e}", dossier.display()))
        .map(|entree| {
            entree
                .expect("entrée")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn droits_de_windows() -> Value {
    let chemin = dossier_de_l_etabli()
        .join("capabilities")
        .join(FICHIER_DES_DROITS);
    let texte = fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("lecture de {} : {e}", chemin.display()));
    serde_json::from_str(&texte).expect("windows.json est du JSON")
}

#[test]
fn i31_seul_windows_json_porte_des_droits() {
    let fautes = controler_les_fichiers_de_droits(&noms_des_fichiers_de_droits());
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_les_droits_nomment_chaque_commande_et_rien_d_autre() {
    let fautes = controler_les_droits(&droits_de_windows(), commandes::NOMS);
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_build_rs_tire_le_manifeste_de_noms() {
    let chemin = dossier_de_l_etabli().join("build.rs");
    let texte = fs::read_to_string(&chemin)
        .unwrap_or_else(|e| panic!("lecture de {} : {e}", chemin.display()));
    let fautes = controler_le_manifeste(&texte);
    assert!(fautes.is_empty(), "{fautes:?}");
}

/// Des droits valides, sur les noms donnés, pour y planter une faute.
fn droits_valides(noms: &[&str]) -> Value {
    let permissions: Vec<String> = noms
        .iter()
        .map(|nom| format!("allow-{}", nom.replace('_', "-")))
        .collect();
    json!({ "platforms": ["windows"], "windows": ["main"], "permissions": permissions })
}

fn avec_permissions(mut droits: Value, permissions: &[&str]) -> Value {
    droits["permissions"] = json!(permissions);
    droits
}

#[test]
fn i31_chaque_controle_des_droits_refuse_ce_qu_on_y_plante() {
    let noms = ["noter", "exporter_cahier"];
    assert!(controler_les_droits(&droits_valides(&noms), &noms).is_empty());

    // Un second fichier de droits, ou l'absence de windows.json.
    let un = |s: &str| vec![s.to_string()];
    assert!(controler_les_fichiers_de_droits(&un("windows.json")).is_empty());
    let deux = vec!["default.json".to_string(), "windows.json".to_string()];
    assert!(!controler_les_fichiers_de_droits(&deux).is_empty());
    assert!(!controler_les_fichiers_de_droits(&un("default.json")).is_empty());
    assert!(!controler_les_fichiers_de_droits(&[]).is_empty());

    // Une commande sans droit.
    let manque = avec_permissions(droits_valides(&noms), &["allow-noter"]);
    assert!(!controler_les_droits(&manque, &noms).is_empty());

    // Un droit qui ne nomme aucune commande.
    let en_trop = avec_permissions(
        droits_valides(&noms),
        &["allow-noter", "allow-exporter-cahier", "allow-supprimer"],
    );
    assert!(!controler_les_droits(&en_trop, &noms).is_empty());

    // Le `_` gardé.
    let souligne = avec_permissions(
        droits_valides(&noms),
        &["allow-noter", "allow-exporter_cahier"],
    );
    assert!(!controler_les_droits(&souligne, &noms).is_empty());

    // Un droit du noyau de Tauri, ou d'un greffon.
    for droit in ["core:default", "dialog:default"] {
        let plante = avec_permissions(
            droits_valides(&noms),
            &["allow-noter", "allow-exporter-cahier", droit],
        );
        assert!(!controler_les_droits(&plante, &noms).is_empty(), "{droit}");
    }

    // Les plateformes, les fenêtres et la liste des droits.
    let mut sans_plateforme = droits_valides(&noms);
    sans_plateforme.as_object_mut().unwrap().remove("platforms");
    assert!(!controler_les_droits(&sans_plateforme, &noms).is_empty());
    let mut autre_plateforme = droits_valides(&noms);
    autre_plateforme["platforms"] = json!(["windows", "android"]);
    assert!(!controler_les_droits(&autre_plateforme, &noms).is_empty());
    let mut autre_fenetre = droits_valides(&noms);
    autre_fenetre["windows"] = json!(["*"]);
    assert!(!controler_les_droits(&autre_fenetre, &noms).is_empty());
    let mut sans_droits = droits_valides(&noms);
    sans_droits.as_object_mut().unwrap().remove("permissions");
    assert!(!controler_les_droits(&sans_droits, &noms).is_empty());
}

#[test]
fn i31_chaque_controle_du_manifeste_refuse_ce_qu_on_y_plante() {
    let bon = "let mut attributs = tauri_build::Attributes::new()\n        .app_manifest(tauri_build::AppManifest::new().commands(commandes::NOMS));";
    assert!(controler_le_manifeste(bon).is_empty());
    // Les blancs et les coupures de ligne ne comptent pas.
    let coupe = "AppManifest::new()\n    .commands(\n        commandes::NOMS\n    )";
    assert!(controler_le_manifeste(coupe).is_empty());
    // Sans manifeste de l'application.
    assert!(!controler_le_manifeste("let attributs = tauri_build::Attributes::new();").is_empty());
    // Une autre liste que `commandes::NOMS`.
    assert!(!controler_le_manifeste("AppManifest::new().commands(&[\"noter\"])").is_empty());
    // Un droit par défaut.
    let permissif = format!("{bon}\n.default_permission(DefaultPermissionRule::AllowAllCommands)");
    assert!(!controler_le_manifeste(&permissif).is_empty());
    // Un appel seulement commenté ne compte pas.
    let commente = "// AppManifest::new().commands(commandes::NOMS)
let attributs = tauri_build::Attributes::new();";
    assert!(!controler_le_manifeste(commente).is_empty());
    // Un commentaire qui cite `default_permission` ne refuse pas un build.rs juste.
    let cite = format!(
        "    // pas de default_permission ici
{bon}"
    );
    assert!(controler_le_manifeste(&cite).is_empty());
}

fn lire_de_l_etabli(chemin: &str) -> String {
    let complet = dossier_de_l_etabli().join(chemin);
    fs::read_to_string(&complet)
        .unwrap_or_else(|e| panic!("lecture de {} : {e}", complet.display()))
}

#[test]
fn i31_la_csp_n_admet_que_ce_que_l_application_apporte() {
    let conf: Value = serde_json::from_str(&lire_de_l_etabli("tauri.conf.json"))
        .expect("tauri.conf.json est du JSON");
    let csp = conf["app"]["security"]["csp"]
        .as_str()
        .expect("app.security.csp est une chaîne");
    let fautes = controler_la_csp(csp);
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_tauri_conf_ne_pose_aucune_cle_interdite() {
    let conf: Value = serde_json::from_str(&lire_de_l_etabli("tauri.conf.json"))
        .expect("tauri.conf.json est du JSON");
    let fautes = controler_la_configuration(&conf);
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_generate_handler_porte_chaque_nom() {
    let fautes = controler_le_gestionnaire(&lire_de_l_etabli("src/lib.rs"), commandes::NOMS);
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_le_cargo_toml_ne_tire_pas_protocol_asset() {
    let fautes = controler_le_cargo_toml(&lire_de_l_etabli("Cargo.toml"));
    assert!(fautes.is_empty(), "{fautes:?}");
}

const CSP_EXACTE: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' http://captation.localhost; media-src 'self' http://captation.localhost; connect-src 'self' http://ipc.localhost; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'";

fn conf_valide() -> Value {
    json!({ "app": {
        "windows": [{ "label": "main" }],
        "security": { "csp": CSP_EXACTE }
    } })
}

#[test]
fn i31_chaque_controle_refuse_ce_qu_on_y_plante() {
    // La CSP exacte passe : `'wasm-unsafe-eval'` est le cas limite admis.
    assert!(controler_la_csp(CSP_EXACTE).is_empty());
    let sans_script = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'";
    assert!(controler_la_csp(sans_script).is_empty());
    for plante in [
        "default-src 'self'; script-src 'self' 'unsafe-inline'",
        "default-src 'self'; script-src 'self' 'unsafe-eval'",
        "default-src 'self'; img-src 'self' https://exemple.org",
        "default-src 'self'; img-src 'self' http://captation.localhost.exemple.org",
        "default-src *",
    ] {
        assert!(!controler_la_csp(plante).is_empty(), "{plante}");
    }

    // La configuration juste passe ; une CSP en objet ou absente refuse.
    assert!(controler_la_configuration(&conf_valide()).is_empty());
    let mut en_objet = conf_valide();
    en_objet["app"]["security"]["csp"] = json!({ "default-src": "'self'" });
    assert!(!controler_la_configuration(&en_objet).is_empty());
    let mut sans_csp = conf_valide();
    sans_csp["app"]["security"]
        .as_object_mut()
        .unwrap()
        .remove("csp");
    assert!(!controler_la_configuration(&sans_csp).is_empty());

    // Chacune des six clés de `security`, sous chacune de ses deux formes.
    for cle in [
        "dangerousDisableAssetCspModification",
        "dangerous-disable-asset-csp-modification",
        "assetProtocol",
        "asset-protocol",
        "devCsp",
        "dev-csp",
    ] {
        let mut plantee = conf_valide();
        plantee["app"]["security"][cle] = json!(true);
        assert!(!controler_la_configuration(&plantee).is_empty(), "{cle}");
    }
    // `useHttpsScheme`, sous une fenêtre, sous chacune de ses deux formes.
    for cle in ["useHttpsScheme", "use-https-scheme"] {
        let mut plantee = conf_valide();
        plantee["app"]["windows"][0][cle] = json!(true);
        assert!(!controler_la_configuration(&plantee).is_empty(), "{cle}");
    }

    // Le gestionnaire : le bon passe, sans `journal` refuse, avec `supprimer` en trop refuse.
    let noms = ["noter", "journal"];
    let bon = "tauri::generate_handler![
    exporter_cahier,
    commandes_de_l_etabli::noter,
    commandes_de_l_etabli::journal,
]";
    assert!(controler_le_gestionnaire(bon, &["noter", "journal", "exporter_cahier"]).is_empty());
    let sans_journal = "tauri::generate_handler![exporter_cahier, commandes_de_l_etabli::noter]";
    assert!(!controler_le_gestionnaire(sans_journal, &noms).is_empty());
    let en_trop = "tauri::generate_handler![commandes_de_l_etabli::noter, commandes_de_l_etabli::journal, commandes_de_l_etabli::supprimer]";
    assert!(!controler_le_gestionnaire(en_trop, &noms).is_empty());
    assert!(!controler_le_gestionnaire("fn sans_gestionnaire() {}", &noms).is_empty());
    // Une ligne commentée ne compte pas.
    let commente = "tauri::generate_handler![
    commandes_de_l_etabli::noter,
    // commandes_de_l_etabli::journal,
]";
    assert!(!controler_le_gestionnaire(commente, &noms).is_empty());

    // Le `Cargo.toml` : `protocol-asset` refuse, même parmi d'autres fonctions.
    assert!(controler_le_cargo_toml("tauri = { version = \"2\", features = [] }").is_empty());
    assert!(
        !controler_le_cargo_toml("tauri = { version = \"2\", features = [\"protocol-asset\"] }")
            .is_empty()
    );
    assert!(
        !controler_le_cargo_toml(
            "tauri = { version = \"2\", features = [\"test\", \"protocol-asset\"] }"
        )
        .is_empty()
    );
}
