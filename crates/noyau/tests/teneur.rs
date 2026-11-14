//! La teneur (§ 31 · 2.6) : le texte déterministe de l'arbre d'un document, que `services` tire
//! du document et que le noyau écrit : les clés par octets, chaque type sous sa lettre, le
//! flottant sous sa forme la plus courte, NaN et un type inconnu refusés en se nommant.

use std::collections::BTreeMap;

use noyau::donnees::fonds::{LIGNEE, LIGNEES_REMPLACEES};
use noyau::donnees::teneur::{ErreurDeTeneur, Valeur, ecrire_la_teneur};

fn t(paires: &[(&str, Valeur)]) -> Result<String, ErreurDeTeneur> {
    let racine: BTreeMap<String, Valeur> = paires
        .iter()
        .map(|(cle, valeur)| (cle.to_string(), valeur.clone()))
        .collect();
    ecrire_la_teneur(&racine)
}

#[test]
fn les_cles_se_rangent_par_octets() {
    let racine = ["a", "Z", "é", "z", "_"].map(|k| (k, Valeur::Null));
    assert_eq!(t(&racine).unwrap(), r#"{"Z":n,"_":n,"a":n,"z":n,"é":n}"#);
}

#[test]
fn chaque_valeur_s_ecrit_avec_son_type_exact() {
    let racine = [
        ("b", Valeur::Boolean(true)),
        ("b0", Valeur::Boolean(false)),
        ("c", Valeur::Counter(3)),
        ("d", Valeur::Timestamp(-5)),
        ("e", Valeur::Liste(vec![])),
        ("f", Valeur::F64(0.5)),
        ("i", Valeur::Int(-7)),
        (
            "l",
            Valeur::Liste(vec![Valeur::Uint(1), Valeur::Str("a".into())]),
        ),
        ("n", Valeur::Null),
        ("o", Valeur::ObjetACles(BTreeMap::new())),
        ("s", Valeur::Str("é\"\\".into())),
        ("t", Valeur::Texte("x".into())),
        ("u", Valeur::Uint(7)),
        ("x", Valeur::Bytes(vec![0x00, 0xab])),
    ];
    assert_eq!(
        t(&racine).unwrap(),
        r#"{"b":b1,"b0":b0,"c":c3,"d":d-5,"e":[],"f":f0.5,"i":i-7,"l":[u1,s"a"],"n":n,"o":{},"s":s"é\"\\","t":t"x","u":u7,"x":x00ab}"#
    );
}

#[test]
fn un_f64_s_ecrit_sous_sa_forme_la_plus_courte() {
    for (x, forme) in [
        (0.1, "0.1"),
        (1e300, "1e300"),
        (-0.0, "-0"),
        (0.0, "0"),
        (100.0, "100"),
        (0.01, "0.01"),
        (1e15, "1e15"),
        (1e-7, "1e-7"),
        (1.5, "1.5"),
        (f64::INFINITY, "inf"),
        (f64::NEG_INFINITY, "-inf"),
    ] {
        assert_eq!(
            t(&[("v", Valeur::F64(x))]).unwrap(),
            format!(r#"{{"v":f{forme}}}"#)
        );
    }
}

#[test]
fn nan_et_un_type_inconnu_se_refusent_en_se_nommant() {
    let nan = Valeur::ObjetACles(BTreeMap::from([("b".to_string(), Valeur::F64(f64::NAN))]));
    assert_eq!(
        t(&[("a", nan)]),
        Err(ErreurDeTeneur::NaN {
            chemin: vec!["a".into(), "b".into()]
        })
    );
    let liste = Valeur::Liste(vec![Valeur::Null, Valeur::Inconnu { type_code: 9 }]);
    assert_eq!(
        t(&[("k", liste)]),
        Err(ErreurDeTeneur::Inconnu {
            chemin: vec!["k".into(), "1".into()],
            type_code: 9
        })
    );
}

#[test]
fn les_textes_se_prennent_octet_pour_octet() {
    assert_ne!(
        t(&[("s", Valeur::Str("\u{e9}".into()))]),
        t(&[("s", Valeur::Str("e\u{301}".into()))])
    );
    assert_ne!(
        t(&[("s", Valeur::Str("x".into()))]),
        t(&[("s", Valeur::Texte("x".into()))])
    );
}

#[test]
fn la_teneur_laisse_les_lignees_de_cote() {
    let racine = [
        (LIGNEE, Valeur::Str("lignee-1".into())),
        (
            LIGNEES_REMPLACEES,
            Valeur::Liste(vec![Valeur::Str("lignee-0".into())]),
        ),
        (
            "o",
            Valeur::ObjetACles(BTreeMap::from([(LIGNEE.to_string(), Valeur::Null)])),
        ),
    ];
    assert_eq!(t(&racine).unwrap(), r#"{"o":{"lignee":n}}"#);
}
