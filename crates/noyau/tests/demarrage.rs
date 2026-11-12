//! Les règles du démarrage : le nom d'un voisin, le sort de chaque `.partiel` et les noms
//! provisoires à ranger (§ 31 · 2.1, § 31 · 6.1).

use std::collections::BTreeSet;

use noyau::donnees::demarrage::{
    Partiel, Provisoire, Sort, a_ranger, fichier_du_voisin, pupitre_neuf, sort_d_un_partiel,
    voisin_libre,
};

fn e(n: u8) -> String {
    format!("{n:064x}")
}

#[test]
fn le_voisin_prend_le_premier_nom_libre() {
    let p = |s: &[&str]| s.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    let c = "complet.automerge";
    assert_eq!(voisin_libre(c, 4242, &[]), "complet.automerge.4242.partiel");
    assert_eq!(
        voisin_libre(c, 4242, &p(&["complet.automerge.4242.partiel"])),
        "complet.automerge.4242-2.partiel"
    );
    assert_eq!(
        voisin_libre(
            c,
            4242,
            &p(&[
                "complet.automerge.4242.partiel",
                "complet.automerge.4242-2.partiel"
            ])
        ),
        "complet.automerge.4242-3.partiel"
    );
    assert_eq!(
        voisin_libre(
            c,
            4242,
            &p(&[
                "complet.automerge.4242.partiel",
                "complet.automerge.4242-3.partiel"
            ])
        ),
        "complet.automerge.4242-2.partiel"
    );
    assert_eq!(
        voisin_libre(
            c,
            4242,
            &p(&["complet.automerge.7.partiel", "complet.automerge"])
        ),
        "complet.automerge.4242.partiel"
    );
}

#[test]
fn le_nom_d_un_voisin_se_reconnait_avec_son_suffixe() {
    assert_eq!(
        fichier_du_voisin("complet.automerge.4242.partiel"),
        Some("complet.automerge")
    );
    assert_eq!(
        fichier_du_voisin("complet.automerge.4242-2.partiel"),
        Some("complet.automerge")
    );
    assert_eq!(
        fichier_du_voisin("complet.automerge.4242-12.partiel"),
        Some("complet.automerge")
    );
    for nom in [
        "complet.automerge",
        "complet.automerge.partiel",
        "complet.automerge.42a.partiel",
        "complet.automerge.4242-.partiel",
        "complet.automerge.4242-x.partiel",
        "complet.automerge.-2.partiel",
        "complet.automerge.4242-2-3.partiel",
        "complet.automerge4242.partiel",
        ".4242.partiel",
    ] {
        assert_eq!(fichier_du_voisin(nom), None, "{nom}");
    }
}

#[test]
fn chaque_partiel_a_son_sort() {
    let a = format!("{}.automerge.4242.partiel", e(1));
    let a2 = format!("{}.automerge.4242-2.partiel", e(1));
    assert_eq!(
        sort_d_un_partiel(Partiel::DansAjouts {
            nom: &a,
            empreinte: &e(1)
        }),
        Sort::MettreEnPlace
    );
    assert_eq!(
        sort_d_un_partiel(Partiel::DansAjouts {
            nom: &a2,
            empreinte: &e(1)
        }),
        Sort::MettreEnPlace
    );
    assert_eq!(
        sort_d_un_partiel(Partiel::DansAjouts {
            nom: &a,
            empreinte: &e(2)
        }),
        Sort::Retirer
    );
    assert_eq!(
        sort_d_un_partiel(Partiel::DansLeDossier {
            nom: "complet.automerge.4242.partiel",
            lisible: true
        }),
        Sort::Lire
    );
    assert_eq!(
        sort_d_un_partiel(Partiel::DansLeDossier {
            nom: "complet.automerge.4242-3.partiel",
            lisible: true
        }),
        Sort::Lire
    );
    assert_eq!(
        sort_d_un_partiel(Partiel::DansLeDossier {
            nom: "complet.automerge.4242.partiel",
            lisible: false
        }),
        Sort::Retirer
    );
    assert_eq!(
        sort_d_un_partiel(Partiel::DansLeDossier {
            nom: "autre.automerge.4242.partiel",
            lisible: true
        }),
        Sort::Retirer
    );
    assert_eq!(sort_d_un_partiel(Partiel::Ailleurs), Sort::Retirer);
}

#[test]
fn pupitre_est_neuf_sans_complet_ni_voisin_lisible() {
    assert!(pupitre_neuf(false, 0));
    assert!(!pupitre_neuf(true, 0) && !pupitre_neuf(false, 1) && !pupitre_neuf(true, 2));
}

// § 31 · 10, test 4 : la captation nommée par un document prend son nom au démarrage
#[test]
fn les_noms_provisoires_a_ranger() {
    let pr = |nom: String, n: u8, deja_rangee| Provisoire {
        nom,
        empreinte: e(n),
        deja_rangee,
    };
    let nommees = BTreeSet::from([e(1), e(2), e(4)]);
    let provisoires = [
        pr(format!("{}.4242.partiel", e(1)), 1, false), // nommée, absente : rangée
        pr(format!("{}.4242.partiel", e(2)), 2, true),  // déjà rangée : reste
        pr(format!("{}.4242.partiel", e(3)), 3, false), // non nommée : reste
        pr(format!("{}.4242.partiel", e(4)), 5, false), // coupée, contenu non nommé : reste
        pr(format!("{}.4242.partiel", e(5)), 4, false), // le contenu nomme e(4) : rangée sous e(4)
        pr(format!("{}.4242-2.partiel", e(1)), 1, false), // même contenu qu'une rangée plus haut : reste
    ];
    assert_eq!(
        a_ranger(&provisoires, &nommees),
        [
            (format!("{}.4242.partiel", e(1)), e(1)),
            (format!("{}.4242.partiel", e(5)), e(4))
        ]
    );
}

// § 31 · 10, test 4 : une empreinte déjà rangée ne se range pas une seconde fois, sous un autre nom provisoire
#[test]
fn un_contenu_deja_range_ne_se_range_pas_sous_un_autre_nom() {
    let nommees = BTreeSet::from([e(1)]);
    let provisoires = [
        Provisoire {
            nom: format!("{}.4242.partiel", e(1)),
            empreinte: e(1),
            deja_rangee: true,
        },
        Provisoire {
            nom: format!("{}.4242-2.partiel", e(1)),
            empreinte: e(1),
            deja_rangee: false,
        },
    ];
    assert!(a_ranger(&provisoires, &nommees).is_empty());
}
