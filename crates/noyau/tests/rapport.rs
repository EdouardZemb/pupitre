//! Le manifeste d'une prise en stems, le rapport d'import, ses comptes, sa relecture et la place
//! libre (§ 30 · 9 ; questions 2, 8, 19 et 25).

use noyau::donnees::rapport::{
    Compte, FichierDeLImport, ManqueDePlace, OrigineChangee, RapportLu, comptes, go,
    lire_le_rapport, manifeste, place, rapport,
};
use std::collections::BTreeSet;

const VOIX: &str = "1c0f0638f51a94205014847b526474bdd04c8c00ad2a02ac4dbeba88d5a3d425"; // « stem inventé : voix »
const BATTERIE: &str = "29b0ea58a7431fa5865927c2c38540e99b5ac828bcdd06391038e6bdf56c739f"; // « stem invente : batterie »
const BASSE: &str = "c7fabc575b006385e571f7b2857d3239dcf042211a610b60903585fe9cc99e17"; // « stem invente : basse »

#[test]
fn le_manifeste_temoin() {
    let octets = manifeste(&[VOIX, BATTERIE, BASSE, VOIX]);
    assert_eq!(octets.len(), 195);
    assert_eq!(
        octets,
        format!("{VOIX}\n{BATTERIE}\n{BASSE}\n").into_bytes()
    );
    assert_eq!(manifeste(&[VOIX]), format!("{VOIX}\n").into_bytes());
}

fn e(n: u64) -> String {
    format!("{n:064x}")
}

fn lu(rang: u64, octets: u64, contenu: u64) -> FichierDeLImport {
    FichierDeLImport::Lu {
        rang,
        octets,
        empreinte: e(contenu),
        illisible: false,
        sans_nom: false,
    }
}

#[test]
fn les_comptes_de_cap03_1() {
    // 80 fichiers, dont 3 doublons de contenu des rangs 1 à 3.
    let mut fichiers: Vec<_> = (1..=77).map(|r| lu(r, 1_000 + r, r)).collect();
    fichiers.extend((1..=3).map(|r| lu(77 + r, 1_000 + r, r)));
    let c = comptes(&fichiers, &BTreeSet::new());
    assert_eq!(
        c.vus,
        Compte {
            fichiers: 80,
            octets: 83_009
        }
    );
    assert_eq!(
        c.verses,
        Compte {
            fichiers: 77,
            octets: 80_003
        }
    );
    assert_eq!(
        c.dedoublonnes,
        Compte {
            fichiers: 3,
            octets: 3_006
        }
    );
    assert_eq!(
        (c.illisibles, c.ignores, c.sans_nom),
        (Compte::default(), Compte::default(), Compte::default())
    );
    fichiers.reverse();
    assert_eq!(
        comptes(&fichiers, &BTreeSet::new()),
        c,
        "l'ordre ne compte pas"
    );
}

#[test]
fn un_contenu_connu_a_l_ouverture_est_dedoublonne() {
    let connues = BTreeSet::from([e(1)]);
    let c = comptes(&[lu(1, 10, 1), lu(2, 20, 2)], &connues);
    assert_eq!(
        (c.verses, c.dedoublonnes),
        (
            Compte {
                fichiers: 1,
                octets: 20
            },
            Compte {
                fichiers: 1,
                octets: 10
            }
        )
    );
}

#[test]
fn un_illisible_en_deux_copies_compte_une_fois() {
    let illisible = |rang| FichierDeLImport::Lu {
        rang,
        octets: 50,
        empreinte: e(9),
        illisible: true,
        sans_nom: false,
    };
    let c = comptes(&[illisible(1), illisible(2)], &BTreeSet::new());
    assert_eq!(
        c.illisibles,
        Compte {
            fichiers: 1,
            octets: 50
        }
    );
    assert_eq!(
        c.dedoublonnes,
        Compte {
            fichiers: 1,
            octets: 50
        }
    );
}

const IMPORT_TEMOIN: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ";

/// Les fichiers du rapport témoin, partagés par `le_rapport_temoin` et `le_rapport_se_relit`.
fn fichiers_du_temoin() -> Vec<FichierDeLImport> {
    vec![
        lu(1, 650, 1),
        FichierDeLImport::Ignore {
            rang: 2,
            octets: 300,
        },
        FichierDeLImport::Lu {
            rang: 3,
            octets: 50,
            empreinte: e(3),
            illisible: true,
            sans_nom: false,
        },
        FichierDeLImport::Lu {
            rang: 4,
            octets: 200,
            empreinte: e(4),
            illisible: false,
            sans_nom: true,
        },
        lu(5, 100, 5),
    ]
}

/// L'origine changée du rapport témoin.
fn origines_du_temoin() -> [OrigineChangee; 1] {
    [OrigineChangee {
        empreinte: "3ab628c95bde7c17cc00f4931000a16a5425d8deb0e5e91fbc055731ac76cd3b".into(), // « une prise de test »
        element: "generee:Atelier invent\u{e9}".into(),
    }]
}

#[test]
fn le_rapport_temoin() {
    let octets = rapport(
        IMPORT_TEMOIN,
        &fichiers_du_temoin(),
        &BTreeSet::from([e(5)]),
        &origines_du_temoin(),
    );
    assert_eq!(
        String::from_utf8(octets.clone()).unwrap(),
        "pupitre-rapport-d-import 1\n01HF7YAT00ZZZZZZZZZZZZZZZZ\n\
        vus=5 1300\nverses=3 900\ndedoublonnes=1 100\nillisibles=1 50\nignores=1 300\nsans-nom=1 200\n\
        origine=3ab628c95bde7c17cc00f4931000a16a5425d8deb0e5e91fbc055731ac76cd3b generee:Atelier invent\u{e9}\n\
        fichier-sans-nom=4\n"
    );
    assert_eq!(octets.len(), 259);
}

#[test]
fn le_rapport_se_relit() {
    // L'inverse de `rapport`, sur le rapport témoin (balayage P14).
    let connues = BTreeSet::from([e(5)]);
    let octets = rapport(
        IMPORT_TEMOIN,
        &fichiers_du_temoin(),
        &connues,
        &origines_du_temoin(),
    );
    let relu = lire_le_rapport(&octets).expect("le rapport témoin se relit");
    assert_eq!(
        relu,
        RapportLu {
            import: IMPORT_TEMOIN.to_string(),
            comptes: comptes(&fichiers_du_temoin(), &connues),
            origines_changees: 1,
            fichiers_sans_nom: vec![4],
        }
    );
    let texte = String::from_utf8(octets).unwrap();
    let autre = texte.replacen(
        "pupitre-rapport-d-import 1",
        "pupitre-rapport-d-import 2",
        1,
    );
    assert_eq!(lire_le_rapport(autre.as_bytes()), None, "une autre version");
    assert_eq!(
        lire_le_rapport(b"pupitre-rapport-d-import 1\n"),
        None,
        "des lignes manquent"
    );
    let mal_formee = texte.replacen("vus=5 1300", "vus=cinq 1300", 1);
    assert_eq!(
        lire_le_rapport(mal_formee.as_bytes()),
        None,
        "une ligne mal formée"
    );
}

#[test]
fn la_place_manque_se_dit_en_go_decimaux() {
    let manque = place(4_000_000_000, 2, 0, 1_000_000_000).unwrap_err();
    assert_eq!(
        manque,
        ManqueDePlace {
            a_verser: 4_000_000_000,
            libre: 1_000_000_000,
            manque: 3_000_008_192
        }
    );
    assert_eq!(
        manque.to_string(),
        "La taille à verser, 4,0 Go, passe la place libre, 1,0 Go : il manque 3,1 Go."
    );
    assert_eq!(place(1_000, 1, 10, 1_000 + 20 + 4_096), Ok(()));
    assert_eq!(
        place(1_000, 1, 10, 1_000 + 20 + 4_095).unwrap_err().manque,
        1
    );
    assert_eq!(place(u64::MAX, 1, 0, u64::MAX).unwrap_err().manque, 4_096);
}

#[test]
fn les_go_decimaux_a_une_decimale() {
    for (octets, haut, bas) in [
        (4_000_000_000, "4,0 Go", "4,0 Go"),
        (1_234_567_890, "1,3 Go", "1,2 Go"),
        (99_999_999, "0,1 Go", "0,0 Go"),
        (100_000_000, "0,1 Go", "0,1 Go"),
        (100_000_001, "0,2 Go", "0,1 Go"),
        (0, "0,0 Go", "0,0 Go"),
    ] {
        assert_eq!(
            (go(octets, true).as_str(), go(octets, false).as_str()),
            (haut, bas),
            "{octets}"
        );
    }
}

#[test]
fn le_premier_rang_d_un_contenu_est_verse_quel_que_soit_l_ordre() {
    // Deux copies de même contenu, d'octets différents : celle de rang 1 est versée.
    for fichiers in [[lu(1, 10, 7), lu(2, 20, 7)], [lu(2, 20, 7), lu(1, 10, 7)]] {
        let c = comptes(&fichiers, &BTreeSet::new());
        assert_eq!(
            (c.verses.octets, c.dedoublonnes.octets),
            (10, 20),
            "{fichiers:?}"
        );
    }
}

#[test]
fn la_relecture_refuse_ce_qu_elle_ne_lit_pas() {
    let connues = BTreeSet::from([e(5)]);
    let texte = String::from_utf8(rapport(
        IMPORT_TEMOIN,
        &fichiers_du_temoin(),
        &connues,
        &origines_du_temoin(),
    ))
    .unwrap();
    let origine = format!(
        "origine={} generee:Atelier invent\u{e9}\n",
        origines_du_temoin()[0].empreinte
    );
    assert!(texte.contains(&origine));
    for (remplace, par, raison) in [
        ("vus=5 1300", "vus=+5 1300", "un signe n'est pas un chiffre"),
        ("vus=5 1300", "vus=5 ", "des octets vides"),
        (origine.as_str(), "origine= generee:x\n", "empreinte vide"),
        (origine.as_str(), "origine=abc \n", "élément vide"),
        (origine.as_str(), "origine=abc\n", "sans élément"),
        (
            "fichier-sans-nom=4",
            "fichier-sans-nom=x",
            "un rang illisible",
        ),
    ] {
        let autre = texte.replacen(remplace, par, 1);
        assert_eq!(lire_le_rapport(autre.as_bytes()), None, "{raison}");
    }
    let sans_lf = texte.trim_end_matches('\n');
    assert_eq!(lire_le_rapport(sans_lf.as_bytes()), None, "sans LF final");
    let origine_apres = texte.replacen(&origine, "", 1) + &origine;
    assert_eq!(
        lire_le_rapport(origine_apres.as_bytes()),
        None,
        "une origine après un fichier sans nom"
    );
}
