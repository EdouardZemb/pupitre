//! L'origine déclarée, le premier nom vu et la portée d'une déclaration (§ 30 · 9 ; questions 17,
//! 20, 22 et 23).

use noyau::donnees::origine::{
    Ajout, Declaration, DeclarationDuFichier, Origine, OriginesLues, ajout_d_origine,
    declaration_du_fichier, la_plus_forte, premier_nom_vu, sous_le_prefixe,
};
use std::collections::{BTreeMap, BTreeSet};

const A: &str = "01HF7YAT000000000000000001"; // un import
const B: &str = "01HF7YAT010000000000000001"; // un import d'une milliseconde plus tard
const A_BIS: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ"; // la même heure que A, un autre aléa

fn lues(main: bool, generees: &[&str]) -> OriginesLues {
    OriginesLues {
        main,
        generees: generees.iter().map(|n| n.to_string()).collect(),
    }
}

#[test]
fn la_plus_forte_generee_puis_main_puis_rien() {
    assert_eq!(la_plus_forte(&lues(false, &[])), None);
    assert_eq!(la_plus_forte(&lues(true, &[])), Some(Origine::Main));
    assert_eq!(
        la_plus_forte(&lues(true, &["Atelier"])),
        Some(Origine::Generee(BTreeSet::from(["Atelier".to_string()])))
    );
    assert_eq!(
        la_plus_forte(&lues(false, &["Atelier", "Forge"])).map(|o| match o {
            Origine::Generee(n) => n.len(),
            _ => 0,
        }),
        Some(2)
    );
}

#[test]
fn main_sur_une_generee_ne_change_rien() {
    let gen_ = |n: &str| Ajout::Generee(n.to_string());
    assert_eq!(
        ajout_d_origine(&lues(false, &["Atelier"]), &Ajout::Main),
        None
    );
    assert_eq!(
        ajout_d_origine(&lues(false, &[]), &Ajout::Main),
        Some(Ajout::Main)
    );
    assert_eq!(ajout_d_origine(&lues(true, &[]), &Ajout::Main), None);
    assert_eq!(
        ajout_d_origine(&lues(true, &[]), &gen_("Atelier")),
        Some(gen_("Atelier"))
    );
    assert_eq!(
        ajout_d_origine(&lues(false, &["Atelier"]), &gen_("Atelier")),
        None
    );
    assert_eq!(
        ajout_d_origine(&lues(false, &["Atelier"]), &gen_("Forge")),
        Some(gen_("Forge"))
    );
    assert_eq!(
        (Ajout::Main.element(), gen_("Atelier inventé").element()),
        ("main".to_string(), "generee:Atelier inventé".to_string())
    );
}

#[test]
fn les_valeurs_d_origine_se_lisent_et_s_ecrivent() {
    for valeur in ["main", "generee:Atelier inventé", "generee:a:b"] {
        assert_eq!(
            Ajout::lire(valeur).map(|a| a.element()),
            Some(valeur.to_string())
        );
    }
    for valeur in ["generee:", "stems", "aucune", "Main", "", "main ", "stems "] {
        assert_eq!(Ajout::lire(valeur), None, "{valeur}");
    }
}

/// La déclaration d'un dossier qui ne porte que son origine.
fn origine(ajout: Ajout) -> Declaration {
    Declaration {
        stems: false,
        origine: Some(ajout),
    }
}

/// La déclaration d'un dossier qui ne se déclare que jeu de stems.
fn stems() -> Declaration {
    Declaration {
        stems: true,
        origine: None,
    }
}

#[test]
fn la_declaration_d_un_fichier_vient_de_son_plus_proche_dossier_declare() {
    let declarations = BTreeMap::from([
        (String::new(), origine(Ajout::Generee("Atelier".into()))),
        ("Démos".to_string(), origine(Ajout::Main)),
        ("Démos/Jeu A".to_string(), stems()),
    ]);
    assert_eq!(
        declaration_du_fichier("Démos/Jeu A/voix.wav", &declarations),
        DeclarationDuFichier {
            origine: Some(Ajout::Main),
            jeu: Some("Démos/Jeu A".into())
        }
    );
    assert_eq!(
        declaration_du_fichier("Démos/Jeu A/sous/x.wav", &declarations),
        DeclarationDuFichier {
            origine: Some(Ajout::Main),
            jeu: None
        }
    );
    assert_eq!(
        declaration_du_fichier("Autre/y.wav", &declarations),
        DeclarationDuFichier {
            origine: Some(Ajout::Generee("Atelier".into())),
            jeu: None
        }
    );
    assert_eq!(
        declaration_du_fichier("z.wav", &BTreeMap::new()),
        DeclarationDuFichier {
            origine: None,
            jeu: None
        }
    );
    let jeu_seul = BTreeMap::from([(String::new(), stems())]);
    assert_eq!(
        declaration_du_fichier("voix.wav", &jeu_seul).jeu,
        Some(String::new())
    );
}

#[test]
fn seuls_des_dossiers_se_declarent() {
    // Le chemin du fichier lui-même n'est jamais une clé lue (balayage P22).
    let declarations = BTreeMap::from([
        ("Démos/a.wav".to_string(), origine(Ajout::Main)),
        ("a.wav".to_string(), stems()),
    ]);
    let rien = DeclarationDuFichier {
        origine: None,
        jeu: None,
    };
    assert_eq!(declaration_du_fichier("Démos/a.wav", &declarations), rien);
    assert_eq!(declaration_du_fichier("a.wav", &declarations), rien);
}

#[test]
fn une_declaration_stems_n_arrete_pas_l_origine_d_un_dossier_plus_haut() {
    let declarations = BTreeMap::from([
        ("Haut".to_string(), origine(Ajout::Generee("Forge".into()))),
        ("Haut/Bas".to_string(), stems()),
        ("Haut/Bas/Fond".to_string(), stems()),
    ]);
    assert_eq!(
        declaration_du_fichier("Haut/Bas/Fond/x.wav", &declarations),
        DeclarationDuFichier {
            origine: Some(Ajout::Generee("Forge".into())),
            jeu: Some("Haut/Bas/Fond".into())
        }
    );
    assert_eq!(
        declaration_du_fichier("Haut/Bas/x.wav", &declarations),
        DeclarationDuFichier {
            origine: Some(Ajout::Generee("Forge".into())),
            jeu: Some("Haut/Bas".into())
        }
    );
}

#[test]
fn un_dossier_se_declare_jeu_de_stems_et_porte_son_origine() {
    // Un jeu de stems tiré d'un générateur, le cas type de l'ADR-024 (CAP-03).
    let atelier = Ajout::Generee("Atelier".into());
    let les_deux = Declaration {
        stems: true,
        origine: Some(atelier.clone()),
    };
    let declarations = BTreeMap::from([("Démos/Jeu A".to_string(), les_deux.clone())]);
    assert_eq!(
        declaration_du_fichier("Démos/Jeu A/voix.wav", &declarations),
        DeclarationDuFichier {
            origine: Some(atelier.clone()),
            jeu: Some("Démos/Jeu A".into())
        }
    );
    // Le dossier versé lui-même, déclaré jeu de stems et origine à la fois.
    let racine = BTreeMap::from([(String::new(), les_deux)]);
    assert_eq!(
        declaration_du_fichier("voix.wav", &racine),
        DeclarationDuFichier {
            origine: Some(atelier),
            jeu: Some(String::new())
        }
    );
}

#[test]
fn le_premier_nom_vu_est_le_plus_petit_couple() {
    assert_eq!(
        premier_nom_vu([("Démos/b.wav", A), ("Démos/a.wav", A), ("Autre/z.wav", B)]),
        Some("Démos/a.wav")
    );
    assert_eq!(premier_nom_vu([("A.wav", B), ("Z.wav", A)]), Some("Z.wav"));
    assert_eq!(
        premier_nom_vu([("b.wav", A), ("a.wav", A_BIS)]),
        Some("a.wav"),
        "même heure : le chemin décide"
    );
    assert_eq!(premier_nom_vu([]), None);
}

#[test]
fn le_premier_nom_vu_ne_lit_que_l_heure_de_l_ulid() {
    // Même heure, aléas inversés : seul le chemin décide.
    assert_eq!(
        premier_nom_vu([("a.wav", A_BIS), ("b.wav", A)]),
        Some("a.wav")
    );
    // Un identifiant plus court que dix caractères se compare tel quel, sans panique.
    assert_eq!(
        premier_nom_vu([("b.wav", "01"), ("a.wav", "02")]),
        Some("b.wav")
    );
}

#[test]
fn la_portee_par_segments_entiers() {
    assert!(sous_le_prefixe("Démos/a.wav", "Démos"));
    assert!(sous_le_prefixe("Démos/sous/a.wav", "Démos"));
    assert!(sous_le_prefixe("Démos", "Démos"));
    assert!(!sous_le_prefixe("Démos 2015/a.wav", "Démos"));
    assert!(!sous_le_prefixe("démos/a.wav", "Démos"));
    assert!(
        !sous_le_prefixe("De\u{301}mos/a.wav", "D\u{e9}mos"),
        "octet pour octet, sans normaliser"
    );
    assert!(sous_le_prefixe("x.wav", ""));
    assert!(
        !sous_le_prefixe("Dém", "Démos"),
        "un préfixe plus long que le chemin"
    );
    assert!(!sous_le_prefixe("Autre/Démos", "Démos"));
}

#[test]
fn une_barre_finale_du_prefixe_s_ote() {
    // L'écran montre « Démos/ » : une seule barre finale s'ôte avant la comparaison.
    assert!(sous_le_prefixe("Démos/a.wav", "Démos/"));
    assert!(sous_le_prefixe("Démos", "Démos/"));
    assert!(!sous_le_prefixe("Démos 2/a.wav", "Démos/"));
    assert!(sous_le_prefixe("x.wav", "/"), "« / » vaut le préfixe vide");
}
