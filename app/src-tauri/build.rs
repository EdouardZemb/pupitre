//! Le script de construction de l'établi : `tauri-build`, à une chose près. Sous Windows (MSVC), le
//! manifeste Windows (l'« application manifest » de Microsoft, qui n'est pas le manifeste de
//! l'application où Tauri déclare ses commandes) n'entre plus dans la ressource de `tauri-build`,
//! que celui-ci ne lie qu'aux exécutables (`cargo:rustc-link-arg-bins`) : l'éditeur de liens
//! l'embarque lui-même dans chaque artefact lié du paquet, l'exécutable des tests unitaires compris.
//! Sans lui, cet exécutable lie la version 5 de `comctl32`, où manque `TaskDialogIndirect`,
//! qu'importe le greffon de dialogue (`rfd`, fonction `common-controls-v6`) : Windows refuse de le
//! charger (`STATUS_ENTRYPOINT_NOT_FOUND`), avant tout test. C'est le contournement que donne
//! l'issue tauri-apps/tauri#13419.

use std::env;
use std::path::PathBuf;

/// Le manifeste Windows par défaut de `tauri-build` 2.7.4 (`windows-app-manifest.xml`), recopié
/// tel quel : la seule dépendance aux Common Controls v6.
const MANIFESTE_WINDOWS: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
</assembly>
"#;

fn main() {
    let mut attributs = tauri_build::Attributes::new();
    let cible_msvc = env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    if cible_msvc {
        // Un seul manifeste Windows par exécutable : celui de l'éditeur de liens, et non plus celui
        // de la ressource de `tauri-build`.
        attributs = attributs
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let fichier = PathBuf::from(env::var("OUT_DIR").expect("cargo donne OUT_DIR"))
            .join("manifeste-windows.xml");
        std::fs::write(&fichier, MANIFESTE_WINDOWS)
            .unwrap_or_else(|e| panic!("écrire {} : {e}", fichier.display()));
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", fichier.display());
        // Sans les droits demandés que l'éditeur de liens ajouterait de lui-même : le manifeste
        // Windows embarqué reste celui de `tauri-build`, rien de plus.
        println!("cargo:rustc-link-arg=/MANIFESTUAC:NO");
    }
    if let Err(erreur) = tauri_build::try_build(attributs) {
        println!("{erreur:#}");
        std::process::exit(1);
    }
}
