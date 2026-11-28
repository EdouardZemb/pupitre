//! Contrôle d'I31 (§ 70 · 3), les droits : l'interface n'appelle que les commandes que les droits
//! nomment. Le test relit sur le disque `app/src-tauri/capabilities/`, `build.rs`,
//! `tauri.conf.json`, le `Cargo.toml` et les sources de `src/` : seul `windows.json` porte des
//! droits, ils nomment chaque commande de `commandes::NOMS`, une à une, chacun par son seul nom, et
//! aucun droit du noyau de Tauri ni d'un greffon ; aucun droit ne s'écrit ailleurs, ni en ligne
//! dans `tauri.conf.json`, ni par `build.rs`, ni par les sources de l'établi ; `build.rs` tire le
//! manifeste de l'application de `commandes::NOMS`, sans droit par défaut ; le protocole des
//! captations s'enregistre en asynchrone, jamais en synchrone. Chaque contrôle est une fonction
//! pure, éprouvée sur des entrées plantées.

use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// Le seul fichier de droits : Windows est la seule plateforme qui appelle des commandes.
const FICHIER_DES_DROITS: &str = "windows.json";

/// Les clés que les droits peuvent porter. Toute autre change à qui ils s'appliquent : `remote`
/// les donne à une origine distante, `webviews` à une vue nommée, et `local` décide s'ils valent
/// pour les pages de l'application.
const CLES_DES_DROITS: [&str; 6] = [
    "$schema",
    "identifier",
    "description",
    "platforms",
    "windows",
    "permissions",
];

/// Le fichier de l'établi qui porte le gestionnaire des commandes, nommé depuis son dossier.
const LIB_DE_L_ETABLI: &str = "src/lib.rs";

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

/// Les droits de `windows.json` : un objet, sans autre clé que `CLES_DES_DROITS` ; `platforms`
/// vaut `["windows"]`, `windows` vaut `["main"]`, et `permissions` est une liste de chaînes, qui
/// égale, comme ensemble, `allow-` suivi de chaque nom de `noms`, chaque `_` devenu `-` ; aucun
/// droit qui porte `:` (un droit du noyau de Tauri ou d'un greffon). Un droit écrit en objet, que
/// Tauri admet (`{"identifier": …}`, avec une portée), est une faute.
fn controler_les_droits(droits: &Value, noms: &[&str]) -> Vec<String> {
    let Some(objet) = droits.as_object() else {
        return vec![
            "les droits ne sont pas un objet : un fichier de droits n'en porte qu'un".to_string(),
        ];
    };
    let mut fautes = Vec::new();
    for cle in objet.keys() {
        if !CLES_DES_DROITS.contains(&cle.as_str()) {
            fautes.push(format!("la clé {cle} n'est pas admise dans les droits"));
        }
    }
    if droits.get("platforms") != Some(&json!(["windows"])) {
        fautes.push("platforms ne vaut pas [\"windows\"]".to_string());
    }
    if droits.get("windows") != Some(&json!(["main"])) {
        fautes.push("windows ne vaut pas [\"main\"]".to_string());
    }
    let Some(liste) = droits.get("permissions").and_then(Value::as_array) else {
        fautes.push("permissions est absent ou n'est pas une liste".to_string());
        return fautes;
    };
    let mut donnes: Vec<&str> = Vec::new();
    for (rang, droit) in liste.iter().enumerate() {
        match droit.as_str() {
            Some(nom) => donnes.push(nom),
            None => fautes.push(format!(
                "permissions[{rang}] n'est pas une chaîne : un droit se nomme, sans objet ni portée"
            )),
        }
    }
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

/// `texte` sans les lignes dont le début, blancs ôtés, est `marque` (`//` en Rust, `#` en TOML) :
/// un appel ou un mot cité en commentaire ne compte ni pour ni contre.
fn sans_lignes_qui_commencent_par(texte: &str, marque: &str) -> String {
    texte
        .lines()
        .filter(|ligne| !ligne.trim_start().starts_with(marque))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `texte` sans aucun blanc.
fn sans_blancs(texte: &str) -> String {
    texte.chars().filter(|c| !c.is_whitespace()).collect()
}

/// `build.rs` : `AppManifest::new().commands(commandes::NOMS)`, blancs ignorés, lignes de
/// commentaire écartées ; aucun `default_permission`, qui permettrait tout, ni
/// `capabilities_path_pattern`, qui lirait des droits hors de `capabilities/`.
fn controler_le_manifeste(build_rs: &str) -> Vec<String> {
    let mut fautes = Vec::new();
    let code = sans_blancs(&sans_lignes_qui_commencent_par(build_rs, "//"));
    if !code.contains("AppManifest::new().commands(commandes::NOMS)") {
        fautes.push(
            "build.rs ne tire pas le manifeste de l'application de commandes::NOMS".to_string(),
        );
    }
    if code.contains("default_permission") {
        fautes.push("build.rs pose un default_permission, qui permettrait tout".to_string());
    }
    if code.contains("capabilities_path_pattern") {
        fautes.push(
            "build.rs pose un capabilities_path_pattern, qui lirait des droits hors de capabilities/"
                .to_string(),
        );
    }
    fautes
}

/// Les sources que la CSP admet, comparées entières.
const SOURCES_ADMISES: [&str; 5] = [
    "'self'",
    "'none'",
    "'wasm-unsafe-eval'",
    "http://captation.localhost",
    "http://ipc.localhost",
];

/// La CSP : au moins une directive, dont `default-src`, qui borne ce qu'aucune autre ne nomme ;
/// chaque directive (`;`) a pour premier mot son nom, hors de la comparaison ; chacun des suivants
/// est une source, et seules sont admises `SOURCES_ADMISES`, comparées entières.
fn controler_la_csp(csp: &str) -> Vec<String> {
    let directives: Vec<Vec<&str>> = csp
        .split(';')
        .map(|directive| directive.split_whitespace().collect::<Vec<_>>())
        .filter(|mots| !mots.is_empty())
        .collect();
    if directives.is_empty() {
        return vec!["la CSP est vide".to_string()];
    }
    let mut fautes = Vec::new();
    if !directives.iter().any(|mots| mots[0] == "default-src") {
        fautes.push("la CSP n'a pas de directive default-src".to_string());
    }
    for mots in &directives {
        let nom = mots[0];
        for source in &mots[1..] {
            if !SOURCES_ADMISES.contains(source) {
                fautes.push(format!("CSP, {nom} : la source {source} n'est pas admise"));
            }
        }
    }
    fautes
}

/// La configuration : `app.security.csp` est une chaîne ; aucune clé interdite sous
/// `app.security`, ni `useHttpsScheme` sous une fenêtre, sous l'une ou l'autre de leurs formes ;
/// aucun `app.security.capabilities`, où des droits s'écriraient en ligne.
fn controler_la_configuration(conf: &Value) -> Vec<String> {
    let mut fautes = Vec::new();
    let securite = &conf["app"]["security"];
    if !securite["csp"].is_string() {
        fautes.push("app.security.csp n'est pas une chaîne".to_string());
    }
    for cle in [
        "dangerousDisableAssetCspModification",
        "dangerous-disable-asset-csp-modification",
        "assetProtocol",
        "asset-protocol",
        "devCsp",
        "dev-csp",
    ] {
        if securite.get(cle).is_some() {
            fautes.push(format!("app.security.{cle} est posée"));
        }
    }
    if securite.get("capabilities").is_some() {
        fautes.push(
            "app.security.capabilities est posée : des droits s'y écriraient en ligne, hors de capabilities/"
                .to_string(),
        );
    }
    if let Some(fenetres) = conf["app"]["windows"].as_array() {
        for (rang, fenetre) in fenetres.iter().enumerate() {
            for cle in ["useHttpsScheme", "use-https-scheme"] {
                if fenetre.get(cle).is_some() {
                    fautes.push(format!("app.windows[{rang}].{cle} est posée"));
                }
            }
        }
    }
    fautes
}

/// `generate_handler![…]` de `src/lib.rs`, lignes de commentaire écartées : il paraît une fois, et
/// le dernier segment de chaque chemin, jusqu'au premier `]`, comme ensemble, égale `noms`.
fn controler_le_gestionnaire(source_de_lib: &str, noms: &[&str]) -> Vec<String> {
    let mut fautes = Vec::new();
    let source = sans_lignes_qui_commencent_par(source_de_lib, "//");
    let occurrences = source.matches("generate_handler!").count();
    if occurrences > 1 {
        fautes.push(format!(
            "generate_handler! paraît {occurrences} fois dans {LIB_DE_L_ETABLI}, et non une"
        ));
    }
    let Some(debut) = source.find("generate_handler![") else {
        fautes.push(format!("generate_handler! est absent de {LIB_DE_L_ETABLI}"));
        return fautes;
    };
    let reste = &source[debut + "generate_handler![".len()..];
    let Some(fin) = reste.find(']') else {
        fautes.push("generate_handler! n'est pas fermé".to_string());
        return fautes;
    };
    let portes: Vec<&str> = reste[..fin]
        .split(',')
        .map(str::trim)
        .filter(|chemin| !chemin.is_empty())
        .map(|chemin| chemin.rsplit("::").next().unwrap_or(chemin))
        .collect();
    for nom in noms {
        if !portes.contains(nom) {
            fautes.push(format!("{nom} manque à generate_handler!"));
        }
    }
    for porte in &portes {
        if !noms.contains(porte) {
            fautes.push(format!(
                "{porte} : generate_handler! nomme ce que NOMS n'a pas"
            ));
        }
    }
    fautes
}

/// Les sources de l'établi, des paires (chemin, texte), les chemins nommés depuis son dossier,
/// lignes de commentaire écartées : aucun `add_capability`, qui ajouterait des droits à
/// l'exécution ; aucun `generate_context!` qui reçoit des arguments (des fichiers de droits de
/// plus, ou une autre configuration), blancs ignorés, `generate_context!()` seul admis ; aucun
/// `generate_handler!` hors de `src/lib.rs`, où `controler_le_gestionnaire` le lit.
fn controler_les_sources(sources: &[(String, String)]) -> Vec<String> {
    let mut fautes = Vec::new();
    let contexte = "generate_context!";
    for (chemin, texte) in sources {
        let code = sans_lignes_qui_commencent_par(texte, "//");
        if code.contains("add_capability") {
            fautes.push(format!(
                "{chemin} : add_capability ajouterait des droits à l'exécution"
            ));
        }
        let serre = sans_blancs(&code);
        for (debut, _) in serre.match_indices(contexte) {
            if !serre[debut + contexte.len()..].starts_with("()") {
                fautes.push(format!(
                    "{chemin} : generate_context! reçoit des arguments, seul generate_context!() est admis"
                ));
            }
        }
        if chemin != LIB_DE_L_ETABLI && code.contains("generate_handler!") {
            fautes.push(format!(
                "{chemin} : generate_handler! hors de {LIB_DE_L_ETABLI}"
            ));
        }
    }
    fautes
}

/// Le protocole des captations dans `src/lib.rs`, lignes de commentaire écartées, blancs ignorés :
/// il s'enregistre en asynchrone, `register_asynchronous_uri_scheme_protocol("captation"`, dont
/// le gestionnaire répond une fois, hors du fil de la WebView ; aucun protocole ne s'enregistre en
/// synchrone (`register_uri_scheme_protocol(`), dont le gestionnaire tiendrait ce fil pendant la
/// demande au fil du fonds.
fn controler_le_protocole(source_de_lib: &str) -> Vec<String> {
    let mut fautes = Vec::new();
    let code = sans_blancs(&sans_lignes_qui_commencent_par(source_de_lib, "//"));
    if !code.contains("register_asynchronous_uri_scheme_protocol(\"captation\"") {
        fautes.push(format!(
            "{LIB_DE_L_ETABLI} n'enregistre pas le protocole captation en asynchrone"
        ));
    }
    if code.contains("register_uri_scheme_protocol(") {
        fautes.push(format!(
            "{LIB_DE_L_ETABLI} enregistre un protocole en synchrone"
        ));
    }
    fautes
}

/// Le `Cargo.toml` d'`etabli` : aucune fonction `protocol-asset`, lignes de commentaire écartées.
fn controler_le_cargo_toml(cargo_toml: &str) -> Vec<String> {
    let mut fautes = Vec::new();
    if sans_lignes_qui_commencent_par(cargo_toml, "#").contains("protocol-asset") {
        fautes.push("Cargo.toml tire la fonction protocol-asset".to_string());
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

/// `controler_…` refuse ce qu'on y plante d'une seule faute, `attendue`, telle qu'il la dit.
fn refuse_seulement(fautes: Vec<String>, attendue: &str) {
    assert_eq!(fautes, vec![attendue.to_string()]);
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

    // Un droit écrit en objet, que Tauri admet : il passerait le contrôle des chaînes.
    let mut en_objet = droits_valides(&noms);
    en_objet["permissions"] = json!([
        "allow-noter",
        "allow-exporter-cahier",
        { "identifier": "core:default" }
    ]);
    refuse_seulement(
        controler_les_droits(&en_objet, &noms),
        "permissions[2] n'est pas une chaîne : un droit se nomme, sans objet ni portée",
    );

    // Une clé qui change à qui les droits s'appliquent : une origine distante, une vue, les pages
    // de l'application.
    for (cle, valeur) in [
        ("remote", json!({ "urls": ["https://*"] })),
        ("webviews", json!(["main"])),
        ("local", json!(false)),
    ] {
        let mut plantee = droits_valides(&noms);
        plantee[cle] = valeur;
        refuse_seulement(
            controler_les_droits(&plantee, &noms),
            &format!("la clé {cle} n'est pas admise dans les droits"),
        );
    }

    // Un fichier de droits écrit en liste, que Tauri admet aussi.
    refuse_seulement(
        controler_les_droits(&json!([droits_valides(&noms)]), &noms),
        "les droits ne sont pas un objet : un fichier de droits n'en porte qu'un",
    );

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
    // Des droits lus hors de `capabilities/`.
    let ailleurs =
        format!("{bon}\nattributs = attributs.capabilities_path_pattern(\"./droits/**/*\");");
    refuse_seulement(
        controler_le_manifeste(&ailleurs),
        "build.rs pose un capabilities_path_pattern, qui lirait des droits hors de capabilities/",
    );
    // Un appel seulement commenté ne compte pas.
    let commente = "// AppManifest::new().commands(commandes::NOMS)\nlet attributs = tauri_build::Attributes::new();";
    assert!(!controler_le_manifeste(commente).is_empty());
    // Un commentaire qui cite `default_permission` ne refuse pas un build.rs juste.
    let cite = format!("    // pas de default_permission ici\n{bon}");
    assert!(controler_le_manifeste(&cite).is_empty());
}

fn lire_de_l_etabli(chemin: &str) -> String {
    let complet = dossier_de_l_etabli().join(chemin);
    fs::read_to_string(&complet)
        .unwrap_or_else(|e| panic!("lecture de {} : {e}", complet.display()))
}

/// Tous les fichiers `*.rs` sous `dossier`, en descendant dans les sous-dossiers.
fn fichiers_rs(dossier: &Path, sortie: &mut Vec<PathBuf>) {
    for entree in
        fs::read_dir(dossier).unwrap_or_else(|e| panic!("lecture de {} : {e}", dossier.display()))
    {
        let chemin = entree.expect("entrée").path();
        if chemin.is_dir() {
            fichiers_rs(&chemin, sortie);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            sortie.push(chemin);
        }
    }
}

/// Chaque `.rs` sous `src/`, nommé depuis le dossier de l'établi, ses composants joints par `/`,
/// sous Windows comme sous Linux.
fn sources_de_l_etabli() -> Vec<(String, String)> {
    let dossier = dossier_de_l_etabli();
    let mut fichiers = Vec::new();
    fichiers_rs(&dossier.join("src"), &mut fichiers);
    fichiers.sort();
    fichiers
        .iter()
        .map(|fichier| {
            let nom = fichier
                .strip_prefix(dossier)
                .expect("un fichier sous l'établi")
                .components()
                .map(|composant| composant.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let texte = fs::read_to_string(fichier)
                .unwrap_or_else(|e| panic!("lecture de {} : {e}", fichier.display()));
            (nom, texte)
        })
        .collect()
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
    let fautes = controler_le_gestionnaire(&lire_de_l_etabli(LIB_DE_L_ETABLI), commandes::NOMS);
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_les_sources_de_l_etabli_n_ajoutent_aucun_droit() {
    let sources = sources_de_l_etabli();
    assert!(
        sources.iter().any(|(chemin, _)| chemin == LIB_DE_L_ETABLI),
        "{LIB_DE_L_ETABLI} n'est pas lu"
    );
    let fautes = controler_les_sources(&sources);
    assert!(fautes.is_empty(), "{fautes:?}");
}

#[test]
fn i31_le_protocole_s_enregistre_en_asynchrone() {
    let fautes = controler_le_protocole(&lire_de_l_etabli(LIB_DE_L_ETABLI));
    assert!(fautes.is_empty(), "{fautes:?}");

    let asynchrone = "        .register_asynchronous_uri_scheme_protocol(\"captation\", |contexte, requete, repondeur| {\n            servir(contexte, requete, repondeur)\n        })\n";
    assert!(controler_le_protocole(asynchrone).is_empty());
    // rustfmt peut couper l'appel : les blancs ne comptent pas.
    let coupe = "        .register_asynchronous_uri_scheme_protocol(\n            \"captation\",\n            |contexte, requete, repondeur| servir(contexte, requete, repondeur),\n        )\n";
    assert!(controler_le_protocole(coupe).is_empty());
    let absent = format!("{LIB_DE_L_ETABLI} n'enregistre pas le protocole captation en asynchrone");
    let synchrone_dit = format!("{LIB_DE_L_ETABLI} enregistre un protocole en synchrone");
    // L'enregistrement synchrone, seul, puis à côté de l'asynchrone.
    let synchrone = "        .register_uri_scheme_protocol(\"captation\", |_contexte, requete| {\n            refuser(&requete)\n        })\n";
    assert_eq!(
        controler_le_protocole(synchrone),
        vec![absent.clone(), synchrone_dit.clone()]
    );
    refuse_seulement(
        controler_le_protocole(&format!("{asynchrone}{synchrone}")),
        &synchrone_dit,
    );
    // L'absence d'enregistrement, ou un enregistrement seulement commenté.
    refuse_seulement(
        controler_le_protocole("        .invoke_handler(gestionnaire())\n"),
        &absent,
    );
    let commente = asynchrone
        .lines()
        .map(|ligne| format!("// {ligne}"))
        .collect::<Vec<_>>()
        .join("\n");
    refuse_seulement(controler_le_protocole(&commente), &absent);
    // L'asynchrone d'un autre protocole ne vaut pas celui des captations.
    refuse_seulement(
        controler_le_protocole(&asynchrone.replace("\"captation\"", "\"photos\"")),
        &absent,
    );
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

/// Une source inventée de l'établi, nommée depuis son dossier.
fn source(chemin: &str, texte: &str) -> (String, String) {
    (chemin.to_string(), texte.to_string())
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
    // Une CSP vide, ou sans `default-src`.
    refuse_seulement(controler_la_csp(""), "la CSP est vide");
    refuse_seulement(
        controler_la_csp("script-src 'self'; object-src 'none'"),
        "la CSP n'a pas de directive default-src",
    );

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
    // Des droits écrits en ligne dans `tauri.conf.json`.
    let mut en_ligne = conf_valide();
    en_ligne["app"]["security"]["capabilities"] = json!([{
        "identifier": "en-ligne",
        "windows": ["main"],
        "permissions": ["core:default"]
    }]);
    refuse_seulement(
        controler_la_configuration(&en_ligne),
        "app.security.capabilities est posée : des droits s'y écriraient en ligne, hors de capabilities/",
    );

    // Le gestionnaire : le bon passe ; sans `journal`, avec `supprimer` en trop, ou un second
    // `generate_handler!`, il refuse.
    let noms = ["noter", "journal"];
    let tous = ["noter", "journal", "exporter_cahier"];
    let bon = "tauri::generate_handler![\n    exporter_cahier,\n    commandes_de_l_etabli::noter,\n    commandes_de_l_etabli::journal,\n]";
    assert!(controler_le_gestionnaire(bon, &tous).is_empty());
    let sans_journal = "tauri::generate_handler![exporter_cahier, commandes_de_l_etabli::noter]";
    refuse_seulement(
        controler_le_gestionnaire(sans_journal, &tous),
        "journal manque à generate_handler!",
    );
    let en_trop = "tauri::generate_handler![commandes_de_l_etabli::noter, commandes_de_l_etabli::journal, commandes_de_l_etabli::supprimer]";
    assert!(!controler_le_gestionnaire(en_trop, &noms).is_empty());
    assert!(!controler_le_gestionnaire("fn sans_gestionnaire() {}", &noms).is_empty());
    let second = format!(
        "{bon}\nfn autre<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool {{\n    tauri::generate_handler![commandes_de_l_etabli::noter]\n}}"
    );
    refuse_seulement(
        controler_le_gestionnaire(&second, &tous),
        "generate_handler! paraît 2 fois dans src/lib.rs, et non une",
    );
    // Une ligne commentée ne compte pas.
    let commente = "tauri::generate_handler![\n    commandes_de_l_etabli::noter,\n    // commandes_de_l_etabli::journal,\n]";
    assert!(!controler_le_gestionnaire(commente, &noms).is_empty());

    // Les sources de l'établi : `generate_context!()` seul passe, et une ligne commentée ne compte
    // pas ; `add_capability`, un `generate_context!` qui reçoit des arguments, ou un
    // `generate_handler!` hors de `src/lib.rs` refusent.
    let lancement = "        .invoke_handler(gestionnaire())\n        .run(tauri::generate_context! ())\n        // app.add_capability(droits)?;\n";
    assert!(controler_les_sources(&[source("src/lib.rs", lancement)]).is_empty());
    refuse_seulement(
        controler_les_sources(&[source(
            "src/lib.rs",
            "        .setup(|app| {\n            app.add_capability(droits)?;\n            Ok(())\n        })\n",
        )]),
        "src/lib.rs : add_capability ajouterait des droits à l'exécution",
    );
    refuse_seulement(
        controler_les_sources(&[source(
            "src/lib.rs",
            "        .run(tauri::generate_context!(capabilities = [\"./ailleurs/droits.json\"]))\n",
        )]),
        "src/lib.rs : generate_context! reçoit des arguments, seul generate_context!() est admis",
    );
    refuse_seulement(
        controler_les_sources(&[source(
            "src/fil.rs",
            "    let gestionnaire = tauri::generate_handler![noter];\n",
        )]),
        "src/fil.rs : generate_handler! hors de src/lib.rs",
    );

    // Le `Cargo.toml` : `protocol-asset` refuse, même parmi d'autres fonctions ; une ligne
    // commentée ne compte pas.
    assert!(controler_le_cargo_toml("tauri = { version = \"2\", features = [] }").is_empty());
    assert!(
        controler_le_cargo_toml(
            "# features = [\"protocol-asset\"]\ntauri = { version = \"2\", features = [] }"
        )
        .is_empty()
    );
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
