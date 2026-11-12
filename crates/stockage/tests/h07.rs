//! L'essai H07 (§ 31 · 4.4, § 40 · 11) : deux répliques qui divergent hors ligne convergent au
//! retour, et une copie de la racine des données, restaurée dans un dossier neuf, garde ses
//! captations intègres. Ces tests ne précèdent aucun code : ils éprouvent le format de
//! l'ADR-023, et un rouge y est un verdict à instruire.
//!
//! Chaque réplique a sa racine des données, dans le dossier jetable du test. B naît d'une copie
//! de l'arbre d'A, faite A fermée, comme le carnet partira de l'établi (§ 31 · 2.3). Se rouvrir,
//! c'est fermer le document puis la racine des données, et les rouvrir, comme au lancement
//! suivant.

mod commun;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use automerge::{Automerge, ChangeHash, ObjId, ROOT, ReadDoc, hydrate};
use commun::{HEURE, NAISSANCE, faite, objet, ouvrir, ouvrir_le_fonds, rangee};
use noyau::donnees::fonds::{ETIQUETTES, IDEES, PRISES, TEXTE};
use proptest::prelude::*;
use stockage::{Document, Erreur};
use synchro::Copie;

/// Le format des prises des tests, dont les captations sont des octets engendrés.
const FORMAT_DES_PRISES: &str = "wav";

/// Les deux idées communes de l'essai tiré au hasard, nées avec le fonds d'A, que B tient de
/// la copie.
const IDEES_COMMUNES: &[&str] = &["idee-1", "idee-2"];

/// Les textes qu'un geste tiré au hasard peut donner à une idée commune : inventés.
const TEXTES_TIRES: &[&str] = &[
    "une basse qui marche",
    "un accord suspendu",
    "un refrain chuchoté",
    "une coda en fondu",
];

/// Les étiquettes qu'un geste tiré au hasard peut poser sur une idée commune, ou en ôter.
const ETIQUETTES_TIREES: &[&str] = &["refrain", "couplet", "pont"];

/// Un geste d'une réplique hors ligne, tiré au hasard.
#[derive(Debug, Clone)]
enum Geste {
    /// Une écriture du fonds.
    Ecrire(Ecriture),
    /// La réplique se ferme, document et racine des données, puis se rouvre.
    SeRouvrir,
}

/// Une écriture du fonds, tirée au hasard.
#[derive(Debug, Clone)]
enum Ecriture {
    /// Changer le texte d'une idée commune.
    ChangerLeTexte {
        idee: &'static str,
        texte: &'static str,
    },
    /// Étiqueter une idée commune.
    Etiqueter {
        idee: &'static str,
        etiquette: &'static str,
    },
    /// Ôter l'étiquette d'une idée commune, qu'elle la porte ou non.
    OterLEtiquette {
        idee: &'static str,
        etiquette: &'static str,
    },
    /// Noter une idée à la réplique, `<réplique>-<rang>`.
    Noter,
}

/// Un geste parmi les cinq sortes, chacune aussi probable.
fn un_geste() -> impl Strategy<Value = Geste> {
    let idee = || prop::sample::select(IDEES_COMMUNES);
    let etiquette = || prop::sample::select(ETIQUETTES_TIREES);
    prop_oneof![
        (idee(), prop::sample::select(TEXTES_TIRES))
            .prop_map(|(idee, texte)| Geste::Ecrire(Ecriture::ChangerLeTexte { idee, texte })),
        (idee(), etiquette())
            .prop_map(|(idee, etiquette)| Geste::Ecrire(Ecriture::Etiqueter { idee, etiquette })),
        (idee(), etiquette()).prop_map(|(idee, etiquette)| Geste::Ecrire(
            Ecriture::OterLEtiquette { idee, etiquette }
        )),
        Just(Geste::Ecrire(Ecriture::Noter)),
        Just(Geste::SeRouvrir),
    ]
}

/// Joue l'écriture `ecriture`, le geste de rang `rang` de la réplique `replique`, sur son fonds :
/// elle est faite, sans avertissement.
fn jouer(fonds: &mut Document<'_>, replique: &str, rang: usize, ecriture: &Ecriture) {
    let heure = HEURE + 1 + i64::try_from(rang).expect("un rang de geste tient dans un i64");
    let fait = match *ecriture {
        Ecriture::ChangerLeTexte { idee, texte } => fonds.ecrire("Texte changé", heure, |tx| {
            commun::changer_le_texte(tx, idee, texte)
        }),
        Ecriture::Etiqueter { idee, etiquette } => fonds.ecrire("Étiquette posée", heure, |tx| {
            commun::etiqueter(tx, idee, etiquette)
        }),
        Ecriture::OterLEtiquette { idee, etiquette } => {
            fonds.ecrire("Étiquette ôtée", heure, |tx| {
                commun::oter_l_etiquette(tx, idee, etiquette)
            })
        }
        Ecriture::Noter => {
            let idee = idee_notee(replique, rang);
            fonds.ecrire("Idée notée", heure, |tx| {
                commun::noter(tx, &idee, "une idée notée hors ligne")
            })
        }
    };
    match fait {
        Ok(None) => {}
        autre => panic!(
            "le geste {} de la réplique {replique} n'est pas fait sans avertissement : {autre:?}",
            rang + 1
        ),
    }
}

/// Le fonds `fonds`, né de la réplique `nom`, est né sans avertissement.
fn nee_sans_avertissement(fonds: &Document<'_>, nom: &str) {
    assert!(
        fonds.avertissements().is_empty(),
        "{nom} est née avec des avertissements : {:?}",
        fonds.avertissements()
    );
}

/// L'idée que note le geste de rang `rang` de la réplique `replique` : `<réplique>-<rang + 1>`.
fn idee_notee(replique: &str, rang: usize) -> String {
    format!("{replique}-{}", rang + 1)
}

/// Les clés de l'objet à clés `objet`, dans l'ordre d'Automerge.
fn cles(doc: &Automerge, objet: &ObjId) -> Vec<String> {
    doc.keys(objet).collect()
}

/// Ce que la réunion de `deux_repliques_modifiees_hors_ligne_convergent` laisse dans le fonds
/// `doc` de la réplique `nom` : les deux textes d'`idee-1`, conflit compris (`get_all`) ; les
/// quatre idées ; les deux étiquettes ; les prises `prises`.
fn constater_la_reunion(doc: &Automerge, nom: &str, prises: &[&str]) {
    let idees = objet(doc, &ROOT, IDEES);
    assert_eq!(
        cles(doc, &idees),
        ["idee-1", "idee-2", "idee-3", "idee-4"],
        "les idées de {nom}"
    );
    let mut textes: Vec<String> = doc
        .get_all(objet(doc, &idees, "idee-1"), TEXTE)
        .expect("lire le texte d'idee-1")
        .into_iter()
        .map(|(valeur, _)| {
            valeur
                .into_string()
                .unwrap_or_else(|autre| panic!("le texte d'idee-1 n'est pas une chaîne : {autre}"))
        })
        .collect();
    textes.sort();
    assert_eq!(
        textes,
        [
            "une ligne de basse en arpèges",
            "une ligne de basse qui monte"
        ],
        "les textes d'idee-1 dans {nom}"
    );
    for (idee, etiquette) in [("idee-1", "basse"), ("idee-2", "refrain")] {
        let etiquettes = objet(doc, &objet(doc, &idees, idee), ETIQUETTES);
        assert_eq!(
            cles(doc, &etiquettes),
            [etiquette],
            "les étiquettes de {idee} dans {nom}"
        );
    }
    let mut attendues = prises.to_vec();
    attendues.sort_unstable();
    assert_eq!(
        cles(doc, &objet(doc, &ROOT, PRISES)),
        attendues,
        "les prises de {nom}"
    );
}

/// Ce qu'A relève avant de se fermer : ses captations, dans l'ordre où elle les a versées, avec
/// leurs octets ; puis les heads, la valeur et la copie de son fonds.
struct Releve {
    captations: Vec<(String, Vec<u8>)>,
    heads: Vec<ChangeHash>,
    valeur: hydrate::Value,
    copie: Copie,
}

/// A naît d'une idée, verse `octets(3, 0)`, `octets(4, 1)` et `octets(5, 70000)`, pose leurs
/// trois prises, relève ses heads et sa valeur, puis se ferme ; son arbre se copie dans un
/// dossier neuf, `verrou` compris. Rend le chemin de la copie et ce qu'A a relevé.
fn une_copie_restauree(nom: &str) -> (PathBuf, Releve) {
    let d = commun::dossier_jetable(nom);
    let chemin_a = d.join("a");
    let chemin_de_la_copie = d.join("copie");
    let racine = ouvrir(&chemin_a);
    let mut a = racine
        .faire_naitre_le_fonds(&NAISSANCE, "Idée notée", HEURE, |tx| {
            commun::noter(tx, "idee-1", "un riff de guitare en boucle")
        })
        .expect("faire naître le fonds d'A");
    nee_sans_avertissement(&a, "A");
    let mut captations = Vec::new();
    for (heure, (graine, longueur)) in (HEURE + 1..).zip([(3, 0), (4, 1), (5, 70_000)]) {
        let octets = commun::octets(graine, longueur);
        let e = racine
            .captations()
            .verser(&octets)
            .expect("verser une captation dans A");
        faite(
            a.ecrire("Prise posée", heure, |tx| {
                commun::poser_une_prise(tx, &e, "idee-1", FORMAT_DES_PRISES)
            }),
            "poser une prise dans A",
        );
        captations.push((e, octets));
    }
    let releve = Releve {
        captations,
        heads: a.automerge().get_heads(),
        valeur: a.automerge().hydrate(None),
        copie: a.copie().expect("la copie d'A"),
    };
    // A fermée, document et racine des données : sous Windows, le verrou qu'elle tient peut
    // refuser la lecture du fichier `verrou`, que la copie emporte.
    drop(a);
    drop(racine);
    commun::copier_l_arbre(&chemin_a, &chemin_de_la_copie);
    (chemin_de_la_copie, releve)
}

#[test]
fn deux_repliques_modifiees_hors_ligne_convergent() {
    let d = commun::dossier_jetable("h07-hors-ligne");
    let chemin_a = d.join("a");
    let chemin_b = d.join("b");
    let captation_d_a = commun::octets(1, 4000);
    let captation_de_b = commun::octets(2, 9000);

    let racine_a = ouvrir(&chemin_a);
    let mut a = racine_a
        .faire_naitre_le_fonds(&NAISSANCE, "Idée notée", HEURE, |tx| {
            commun::noter(tx, "idee-1", "une ligne de basse qui descend")
        })
        .expect("faire naître le fonds d'A");
    nee_sans_avertissement(&a, "A");
    faite(
        a.ecrire("Idée notée", HEURE + 1, |tx| {
            commun::noter(tx, "idee-2", "un refrain en question")
        }),
        "noter idee-2 dans A",
    );
    let e_a = racine_a
        .captations()
        .verser(&captation_d_a)
        .expect("verser la captation d'A");
    faite(
        a.ecrire("Prise posée", HEURE + 2, |tx| {
            commun::poser_une_prise(tx, &e_a, "idee-2", FORMAT_DES_PRISES)
        }),
        "poser la prise d'A",
    );
    // A fermée, document et racine des données : sous Windows, le verrou qu'elle tient peut
    // refuser la lecture du fichier `verrou`, que la copie emporte.
    drop(a);
    drop(racine_a);
    commun::copier_l_arbre(&chemin_a, &chemin_b);

    // Hors ligne, A de son côté, puis elle se rouvre.
    let racine_a = ouvrir(&chemin_a);
    let mut a = ouvrir_le_fonds(&racine_a);
    faite(
        a.ecrire("Texte changé", HEURE + 3, |tx| {
            commun::changer_le_texte(tx, "idee-1", "une ligne de basse qui monte")
        }),
        "changer le texte d'idee-1 dans A",
    );
    faite(
        a.ecrire("Étiquette posée", HEURE + 4, |tx| {
            commun::etiqueter(tx, "idee-2", "refrain")
        }),
        "étiqueter idee-2 dans A",
    );
    faite(
        a.ecrire("Idée notée", HEURE + 5, |tx| {
            commun::noter(tx, "idee-3", "un pont plus calme")
        }),
        "noter idee-3 dans A",
    );
    drop(a);
    drop(racine_a);
    let racine_a = ouvrir(&chemin_a);
    let mut a = ouvrir_le_fonds(&racine_a);

    // Hors ligne, B de son côté, puis elle se rouvre.
    let racine_b = ouvrir(&chemin_b);
    let mut b = ouvrir_le_fonds(&racine_b);
    faite(
        b.ecrire("Texte changé", HEURE + 3, |tx| {
            commun::changer_le_texte(tx, "idee-1", "une ligne de basse en arpèges")
        }),
        "changer le texte d'idee-1 dans B",
    );
    faite(
        b.ecrire("Étiquette posée", HEURE + 4, |tx| {
            commun::etiqueter(tx, "idee-1", "basse")
        }),
        "étiqueter idee-1 dans B",
    );
    faite(
        b.ecrire("Idée notée", HEURE + 5, |tx| {
            commun::noter(tx, "idee-4", "une fin en suspens")
        }),
        "noter idee-4 dans B",
    );
    let e_b = racine_b
        .captations()
        .verser(&captation_de_b)
        .expect("verser la captation de B");
    faite(
        b.ecrire("Prise posée", HEURE + 6, |tx| {
            commun::poser_une_prise(tx, &e_b, "idee-4", FORMAT_DES_PRISES)
        }),
        "poser la prise de B",
    );
    drop(b);
    drop(racine_b);
    let racine_b = ouvrir(&chemin_b);
    let mut b = ouvrir_le_fonds(&racine_b);

    commun::reunir(&mut a, &mut b);

    let heads = a.automerge().get_heads();
    let valeur = a.automerge().hydrate(None);
    assert_eq!(
        b.automerge().get_heads(),
        heads,
        "A et B n'ont pas les mêmes heads"
    );
    assert_eq!(
        b.automerge().hydrate(None),
        valeur,
        "A et B n'ont pas la même valeur"
    );
    let prises = [e_a.as_str(), e_b.as_str()];
    for (nom, doc) in [("A", a.automerge()), ("B", b.automerge())] {
        constater_la_reunion(doc, nom, &prises);
    }
    // La synchronisation ne transporte que les documents : la captation d'A est dans les deux,
    // par la copie, et celle de B n'est que dans B.
    for (nom, racine) in [("A", &racine_a), ("B", &racine_b)] {
        let relus = racine
            .captations()
            .relire(&e_a)
            .unwrap_or_else(|erreur| panic!("relire la captation d'A dans {nom} : {erreur:?}"));
        assert!(
            relus == captation_d_a,
            "la captation d'A ne se relit pas dans {nom}"
        );
    }
    let relus = racine_b
        .captations()
        .relire(&e_b)
        .expect("relire la captation de B dans B");
    assert!(
        relus == captation_de_b,
        "la captation de B ne se relit pas dans B"
    );
    match racine_a.captations().relire(&e_b) {
        Err(Erreur::CaptationAbsente { empreinte }) => assert_eq!(empreinte, e_b),
        Err(autre) => panic!("attendu CaptationAbsente, rendu {autre:?}"),
        Ok(_) => panic!("la captation de B est dans A"),
    }

    // Fermées, puis rouvertes, comme au lancement suivant.
    drop(a);
    drop(b);
    drop(racine_a);
    drop(racine_b);
    for (nom, chemin) in [("A", &chemin_a), ("B", &chemin_b)] {
        let racine = ouvrir(chemin);
        let fonds = ouvrir_le_fonds(&racine);
        assert_eq!(
            fonds.automerge().get_heads(),
            heads,
            "{nom} rouverte n'a plus les heads de la réunion"
        );
        assert_eq!(
            fonds.automerge().hydrate(None),
            valeur,
            "{nom} rouverte n'a plus la valeur de la réunion"
        );
    }
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(32))]

    #[test]
    fn la_convergence_tient_sur_des_gestes_tires_au_hasard(
        gestes_a in prop::collection::vec(un_geste(), 0..=12),
        gestes_b in prop::collection::vec(un_geste(), 0..=12),
    ) {
        let d = commun::dossier_jetable("h07-hasard");
        let chemin_a = d.join("a");
        let chemin_b = d.join("b");
        {
            let racine = ouvrir(&chemin_a);
            let a = racine
                .faire_naitre_le_fonds(&NAISSANCE, "Idées notées", HEURE, |tx| {
                    commun::noter(tx, IDEES_COMMUNES[0], "un motif de trois notes")?;
                    commun::noter(tx, IDEES_COMMUNES[1], "une grille en boucle")
                })
                .expect("faire naître le fonds d'A");
            nee_sans_avertissement(&a, "A");
        }
        commun::copier_l_arbre(&chemin_a, &chemin_b);

        // Hors ligne, chacune de son côté. Se rouvrir ferme le document, puis la racine des
        // données, que le document emprunte ; rouverte, la réplique a les heads et la valeur
        // qu'elle avait avant de se fermer.
        let mut racine_a = ouvrir(&chemin_a);
        let mut a = ouvrir_le_fonds(&racine_a);
        for (rang, geste) in gestes_a.iter().enumerate() {
            match geste {
                Geste::Ecrire(ecriture) => jouer(&mut a, "a", rang, ecriture),
                Geste::SeRouvrir => {
                    let heads_d_avant = a.automerge().get_heads();
                    let valeur_d_avant = a.automerge().hydrate(None);
                    drop(a);
                    drop(racine_a);
                    racine_a = ouvrir(&chemin_a);
                    a = ouvrir_le_fonds(&racine_a);
                    let heads_rouverte = a.automerge().get_heads();
                    let valeur_rouverte = a.automerge().hydrate(None);
                    prop_assert_eq!(
                        &heads_rouverte,
                        &heads_d_avant,
                        "A rouverte au geste {} n'a plus ses heads",
                        rang + 1
                    );
                    prop_assert_eq!(
                        &valeur_rouverte,
                        &valeur_d_avant,
                        "A rouverte au geste {} n'a plus sa valeur",
                        rang + 1
                    );
                }
            }
        }
        let mut racine_b = ouvrir(&chemin_b);
        let mut b = ouvrir_le_fonds(&racine_b);
        for (rang, geste) in gestes_b.iter().enumerate() {
            match geste {
                Geste::Ecrire(ecriture) => jouer(&mut b, "b", rang, ecriture),
                Geste::SeRouvrir => {
                    let heads_d_avant = b.automerge().get_heads();
                    let valeur_d_avant = b.automerge().hydrate(None);
                    drop(b);
                    drop(racine_b);
                    racine_b = ouvrir(&chemin_b);
                    b = ouvrir_le_fonds(&racine_b);
                    let heads_rouverte = b.automerge().get_heads();
                    let valeur_rouverte = b.automerge().hydrate(None);
                    prop_assert_eq!(
                        &heads_rouverte,
                        &heads_d_avant,
                        "B rouverte au geste {} n'a plus ses heads",
                        rang + 1
                    );
                    prop_assert_eq!(
                        &valeur_rouverte,
                        &valeur_d_avant,
                        "B rouverte au geste {} n'a plus sa valeur",
                        rang + 1
                    );
                }
            }
        }

        commun::reunir(&mut a, &mut b);

        let heads = a.automerge().get_heads();
        let valeur = a.automerge().hydrate(None);
        let heads_de_b = b.automerge().get_heads();
        let valeur_de_b = b.automerge().hydrate(None);
        prop_assert_eq!(&heads_de_b, &heads, "A et B n'ont pas les mêmes heads");
        prop_assert_eq!(&valeur_de_b, &valeur, "A et B n'ont pas la même valeur");

        // Fermées, puis rouvertes, comme au lancement suivant.
        drop(a);
        drop(b);
        drop(racine_a);
        drop(racine_b);
        let racine_a = ouvrir(&chemin_a);
        let racine_b = ouvrir(&chemin_b);
        let a = ouvrir_le_fonds(&racine_a);
        let b = ouvrir_le_fonds(&racine_b);
        let heads_de_a = a.automerge().get_heads();
        let valeur_de_a = a.automerge().hydrate(None);
        let heads_de_b = b.automerge().get_heads();
        let valeur_de_b = b.automerge().hydrate(None);
        prop_assert_eq!(&heads_de_a, &heads, "A rouverte n'a plus les heads de la réunion");
        prop_assert_eq!(&valeur_de_a, &valeur, "A rouverte n'a plus la valeur de la réunion");
        prop_assert_eq!(&heads_de_b, &heads, "B rouverte n'a plus les heads de la réunion");
        prop_assert_eq!(&valeur_de_b, &valeur, "B rouverte n'a plus la valeur de la réunion");

        // Aucun geste ne retire une idée : A rouverte porte les deux idées communes et chaque
        // idée notée, d'A comme de B, et B aussi, qui a la même valeur.
        let mut attendues: Vec<String> =
            IDEES_COMMUNES.iter().map(|&idee| idee.to_owned()).collect();
        for (replique, gestes) in [("a", &gestes_a), ("b", &gestes_b)] {
            for (rang, geste) in gestes.iter().enumerate() {
                if matches!(geste, Geste::Ecrire(Ecriture::Noter)) {
                    attendues.push(idee_notee(replique, rang));
                }
            }
        }
        attendues.sort_unstable();
        let mut idees_de_a = cles(a.automerge(), &objet(a.automerge(), &ROOT, IDEES));
        idees_de_a.sort_unstable();
        prop_assert_eq!(
            &idees_de_a,
            &attendues,
            "A rouverte ne porte pas les idées communes et les idées notées"
        );
    }
}

// § 31 · 10, test 4 : l'empreinte de chaque objet égale son nom
#[test]
fn une_copie_restauree_relit_chaque_captation_contre_son_empreinte() {
    let (chemin, releve) = une_copie_restauree("h07-copie-restauree");
    assert!(
        chemin.join("verrou").is_file(),
        "la copie n'emporte pas le fichier verrou"
    );

    // La seule présence du fichier `verrou` ne dit rien : la copie s'ouvre.
    let racine = ouvrir(&chemin);
    let captations = racine.captations();
    assert_eq!(
        captations
            .verifier()
            .expect("vérifier les captations de la copie"),
        Vec::<PathBuf>::new(),
        "la copie restaurée nomme une captation"
    );
    let mut fonds = ouvrir_le_fonds(&racine);
    let versees: BTreeMap<&str, &[u8]> = releve
        .captations
        .iter()
        .map(|(e, octets)| (e.as_str(), octets.as_slice()))
        .collect();
    let prises = cles(fonds.automerge(), &objet(fonds.automerge(), &ROOT, PRISES));
    assert_eq!(
        prises,
        versees.keys().copied().collect::<Vec<&str>>(),
        "les prises de la copie ne sont pas celles d'A"
    );
    for e in &prises {
        let relus = captations
            .relire(e)
            .unwrap_or_else(|erreur| panic!("relire la captation {e} : {erreur:?}"));
        assert!(
            relus == versees[e.as_str()],
            "la captation {e} ne se relit pas telle qu'A l'a versée"
        );
    }
    assert_eq!(
        fonds.automerge().get_heads(),
        releve.heads,
        "le fonds de la copie n'a pas les heads d'A"
    );
    assert_eq!(
        fonds.automerge().hydrate(None),
        releve.valeur,
        "le fonds de la copie n'a pas la valeur d'A"
    );
    faite(
        fonds.ecrire("Étiquette posée", HEURE + 10, |tx| {
            commun::etiqueter(tx, "idee-1", "riff")
        }),
        "écrire dans la copie restaurée",
    );
    assert_eq!(
        fonds
            .copie()
            .expect("la copie que situe la copie restaurée"),
        releve.copie,
        "la copie restaurée ne se situe pas comme A"
    );
}

// § 31 · 10, test 4 : l'empreinte de chaque objet égale son nom
#[test]
fn une_captation_alteree_ou_absente_de_la_copie_se_nomme() {
    let (chemin, releve) = une_copie_restauree("h07-copie-alteree");
    // La deuxième captation, `octets(4, 1)`, altérée d'un octet ; la troisième retirée.
    let alteree = releve.captations[1].0.as_str();
    let absente = releve.captations[2].0.as_str();
    let chemin_de_l_alteree = rangee(&chemin, alteree);
    let mut octets = fs::read(&chemin_de_l_alteree).expect("lire la deuxième captation");
    octets[0] ^= 1;
    fs::write(&chemin_de_l_alteree, octets).expect("altérer la deuxième captation");
    fs::remove_file(rangee(&chemin, absente)).expect("retirer la troisième captation");

    let racine = ouvrir(&chemin);
    let captations = racine.captations();
    assert_eq!(
        captations
            .verifier()
            .expect("vérifier les captations de la copie"),
        [chemin_de_l_alteree],
        "vérifier ne nomme pas la deuxième captation, et elle seule"
    );
    match captations.relire(alteree) {
        Err(Erreur::CaptationAlteree { empreinte }) => assert_eq!(empreinte, alteree),
        Err(autre) => panic!("attendu CaptationAlteree, rendu {autre:?}"),
        Ok(_) => panic!("la deuxième captation, altérée, se relit"),
    }
    match captations.relire(absente) {
        Err(Erreur::CaptationAbsente { empreinte }) => assert_eq!(empreinte, absente),
        Err(autre) => panic!("attendu CaptationAbsente, rendu {autre:?}"),
        Ok(_) => panic!("la troisième captation, retirée, se relit"),
    }
}
