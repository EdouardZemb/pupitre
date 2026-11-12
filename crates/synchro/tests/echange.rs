//! L'échange entre deux copies d'un document (§ 31 · 4.4, § 31 · 2.5) : deux copies d'une même
//! lignée convergent par le protocole d'Automerge ; une copie plus récente reçoit, et ne transmet
//! aucune modification (§ 31 · 2.4) ; deux copies plus récentes ne s'échangent aucune
//! modification, et leur échange finit ; deux copies étrangères ne s'échangent rien ; un message
//! illisible est refusé, et le document ne bouge pas.
//!
//! Les documents vivent en mémoire. Leur racine porte un propriétaire, un document et une lignée,
//! sous les clés du fonds ; la seconde copie naît d'un `fork()` de la première.

use std::collections::BTreeMap;

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{Automerge, ROOT, ReadDoc};
use noyau::donnees::fonds::{IDENTIFIANT, LIGNEE, PROPRIETAIRE};
use synchro::{Copie, Difference, Echange, Erreur};

/// Les identifiants des tests : écrits, jamais tirés.
const PROPRIETAIRE_1: &str = "proprietaire-1";
const PROPRIETAIRE_2: &str = "proprietaire-2";
const DOCUMENT_1: &str = "fonds-1";
const DOCUMENT_2: &str = "fonds-2";
const LIGNEE_1: &str = "lignee-1";
const LIGNEE_2: &str = "lignee-2";

/// L'heure de chaque modification des tests. Une entrée, jamais l'horloge.
const HEURE: i64 = 1_700_000_000;

/// Au plus ce nombre de tours : une réunion qui ne s'achève pas fait échouer le test.
const TOURS_AU_PLUS: usize = 16;

/// La copie que situent `proprietaire`, `document` et `lignee`.
fn copie(proprietaire: &str, document: &str, lignee: &str) -> Copie {
    Copie {
        proprietaire: proprietaire.to_string(),
        document: document.to_string(),
        lignee: lignee.to_string(),
    }
}

/// Un document en mémoire, d'un acteur neuf, dont la racine porte les trois identifiants de
/// `copie`.
fn naitre(copie: &Copie) -> Automerge {
    let mut doc = Automerge::new();
    let mut tx = doc.transaction();
    for (cle, valeur) in [
        (PROPRIETAIRE, &copie.proprietaire),
        (IDENTIFIANT, &copie.document),
        (LIGNEE, &copie.lignee),
    ] {
        tx.put(ROOT, cle, valeur.as_str())
            .expect("écrire la racine du document");
    }
    tx.commit_with(CommitOptions::default().with_time(HEURE));
    doc
}

/// Écrit `valeur` sous `cle`, à la racine de `doc`, en une modification.
fn ecrire(doc: &mut Automerge, cle: &str, valeur: &str) {
    let mut tx = doc.transaction();
    tx.put(ROOT, cle, valeur).expect("écrire à la racine");
    tx.commit_with(CommitOptions::default().with_time(HEURE));
}

/// La copie que situe la racine de `doc`.
fn copie_de(doc: &Automerge) -> Copie {
    Copie {
        proprietaire: lire(doc, PROPRIETAIRE),
        document: lire(doc, IDENTIFIANT),
        lignee: lire(doc, LIGNEE),
    }
}

/// La chaîne que porte `cle`, à la racine de `doc`.
fn lire(doc: &Automerge, cle: &str) -> String {
    let (valeur, _) = doc
        .get(ROOT, cle)
        .expect("lire la racine du document")
        .unwrap_or_else(|| panic!("{cle} manque à la racine du document"));
    valeur
        .into_string()
        .unwrap_or_else(|autre| panic!("{cle} n'est pas une chaîne : {autre}"))
}

/// Toutes les valeurs de `cle`, à la racine de `doc`, conflits compris (`get_all`), dans
/// l'ordre d'Automerge.
fn valeurs(doc: &Automerge, cle: &str) -> Vec<String> {
    doc.get_all(ROOT, cle)
        .expect("lire la racine du document")
        .into_iter()
        .map(|(valeur, _)| {
            valeur
                .into_string()
                .unwrap_or_else(|autre| panic!("{cle} n'est pas une chaîne : {autre}"))
        })
        .collect()
}

/// La valeur de `doc` : chaque clé de sa racine, avec toutes ses valeurs.
fn valeur_de(doc: &Automerge) -> BTreeMap<String, Vec<String>> {
    doc.keys(ROOT)
        .map(|cle| {
            let toutes = valeurs(doc, &cle);
            (cle, toutes)
        })
        .collect()
}

/// Échange les messages des deux copies, tour après tour, jusqu'à ce qu'aucune n'envoie plus
/// rien ; au-delà de `TOURS_AU_PLUS` tours, le test échoue.
fn reunir(
    locale: &mut Automerge,
    de_la_locale: &mut Echange,
    distante: &mut Automerge,
    de_la_distante: &mut Echange,
) {
    reunir_selon(
        locale,
        de_la_locale,
        distante,
        de_la_distante,
        Echange::prochain_message,
        Echange::prochain_message,
    );
}

/// La réunion de `reunir`, où la locale prépare chacun de ses messages par
/// `message_de_la_locale`, et la distante par `message_de_la_distante`.
fn reunir_selon(
    locale: &mut Automerge,
    de_la_locale: &mut Echange,
    distante: &mut Automerge,
    de_la_distante: &mut Echange,
    message_de_la_locale: impl Fn(&mut Echange, &Automerge) -> Option<Vec<u8>>,
    message_de_la_distante: impl Fn(&mut Echange, &Automerge) -> Option<Vec<u8>>,
) {
    for _ in 0..TOURS_AU_PLUS {
        let vers_la_distante = message_de_la_locale(de_la_locale, locale);
        let vers_la_locale = message_de_la_distante(de_la_distante, distante);
        if vers_la_distante.is_none() && vers_la_locale.is_none() {
            return;
        }
        if let Some(message) = vers_la_distante {
            de_la_distante
                .recevoir(distante, &message)
                .expect("la distante reçoit le message de la locale");
        }
        if let Some(message) = vers_la_locale {
            de_la_locale
                .recevoir(locale, &message)
                .expect("la locale reçoit le message de la distante");
        }
    }
    panic!("la réunion ne s'achève pas en {TOURS_AU_PLUS} tours");
}

#[test]
fn deux_copies_d_une_meme_lignee_convergent() {
    let mut locale = naitre(&copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_1));
    let mut distante = locale.fork();
    ecrire(&mut locale, "commune", "écrit par la locale");
    ecrire(&mut locale, "de_la_locale", "seule la locale l'écrit");
    ecrire(&mut distante, "commune", "écrit par la distante");
    ecrire(&mut distante, "de_la_distante", "seule la distante l'écrit");
    assert_ne!(locale.get_heads(), distante.get_heads());

    let mut de_la_locale = Echange::ouvrir(&copie_de(&locale), &copie_de(&distante))
        .expect("ouvrir l'échange de la locale");
    let mut de_la_distante = Echange::ouvrir(&copie_de(&distante), &copie_de(&locale))
        .expect("ouvrir l'échange de la distante");
    reunir(
        &mut locale,
        &mut de_la_locale,
        &mut distante,
        &mut de_la_distante,
    );

    assert_eq!(locale.get_heads(), distante.get_heads());
    assert_eq!(valeur_de(&locale), valeur_de(&distante));
    for doc in [&locale, &distante] {
        let mut communes = valeurs(doc, "commune");
        communes.sort();
        assert_eq!(communes, ["écrit par la distante", "écrit par la locale"]);
        assert_eq!(valeurs(doc, "de_la_locale"), ["seule la locale l'écrit"]);
        assert_eq!(
            valeurs(doc, "de_la_distante"),
            ["seule la distante l'écrit"]
        );
    }
}

// § 31 · 10, test 10 : la copie plus récente reçoit, et ne transmet aucune modification
#[test]
fn une_copie_n_plus_1_recoit_sans_transmettre_aucune_modification() {
    // A, la copie plus récente, envoie ses messages sans aucune modification ; B, les siens par
    // le protocole. Chacune a une modification que l'autre n'a pas : A reçoit celle de B, et B
    // ne reçoit pas celle d'A.
    let mut a = naitre(&copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_1));
    let mut b = a.fork();
    ecrire(&mut a, "a", "x");
    ecrire(&mut b, "b", "y");
    let mut de_a = Echange::ouvrir(&copie_de(&a), &copie_de(&b)).expect("ouvrir l'échange d'A");
    let mut de_b = Echange::ouvrir(&copie_de(&b), &copie_de(&a)).expect("ouvrir l'échange de B");

    // La réunion finit avant `TOURS_AU_PLUS` tours, quand ni A ni B n'envoie plus rien.
    reunir_selon(
        &mut a,
        &mut de_a,
        &mut b,
        &mut de_b,
        Echange::prochain_message_sans_modification,
        Echange::prochain_message,
    );

    assert_eq!(valeur_de(&b).get("a"), None, "B a reçu la modification d'A");
    assert_eq!(
        valeur_de(&a).get("b"),
        Some(&vec!["y".to_string()]),
        "A n'a pas reçu la modification de B"
    );
}

// § 31 · 10, test 10 : deux copies plus récentes ne s'échangent aucune modification, et leur échange finit
#[test]
fn deux_copies_plus_recentes_ne_s_echangent_aucune_modification() {
    // A et B, deux copies plus récentes, envoient chacune ses messages sans aucune modification.
    // Chacune a une modification que l'autre n'a pas : aucune ne la reçoit.
    let mut a = naitre(&copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_1));
    let mut b = a.fork();
    ecrire(&mut a, "a", "x");
    ecrire(&mut b, "b", "y");
    let (heads_d_a, heads_de_b) = (a.get_heads(), b.get_heads());
    let mut de_a = Echange::ouvrir(&copie_de(&a), &copie_de(&b)).expect("ouvrir l'échange d'A");
    let mut de_b = Echange::ouvrir(&copie_de(&b), &copie_de(&a)).expect("ouvrir l'échange de B");

    // La réunion finit avant `TOURS_AU_PLUS` tours, quand ni A ni B n'envoie plus rien.
    reunir_selon(
        &mut a,
        &mut de_a,
        &mut b,
        &mut de_b,
        Echange::prochain_message_sans_modification,
        Echange::prochain_message_sans_modification,
    );

    assert_eq!(a.get_heads(), heads_d_a, "A a reçu une modification de B");
    assert_eq!(b.get_heads(), heads_de_b, "B a reçu une modification d'A");
}

#[test]
fn deux_lignees_differentes_n_ouvrent_aucun_echange() {
    let locale = copie_de(&naitre(&copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_1)));
    for (distante, difference) in [
        (
            copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_2),
            Difference::AutreLignee,
        ),
        (
            copie(PROPRIETAIRE_2, DOCUMENT_1, LIGNEE_1),
            Difference::AutreProprietaire,
        ),
        (
            copie(PROPRIETAIRE_1, DOCUMENT_2, LIGNEE_1),
            Difference::AutreDocument,
        ),
    ] {
        let distante = copie_de(&naitre(&distante));
        assert_eq!(
            Echange::ouvrir(&locale, &distante).err(),
            Some(difference),
            "{distante:?}"
        );
    }
}

#[test]
fn un_message_illisible_est_refuse_et_le_document_ne_bouge_pas() {
    let mut doc = naitre(&copie(PROPRIETAIRE_1, DOCUMENT_1, LIGNEE_1));
    let situee = copie_de(&doc);
    let mut echange = Echange::ouvrir(&situee, &situee).expect("ouvrir l'échange");
    let avant = doc.get_heads();
    let illisibles: [&[u8]; 3] = [b"pas un message", &[0xff, 0x00, 0x13, 0x37], &[]];
    for octets in illisibles {
        let refus = echange.recevoir(&mut doc, octets);
        assert!(
            matches!(refus, Err(Erreur::MessageIllisible(_))),
            "{octets:?} : {refus:?}"
        );
        assert_eq!(doc.get_heads(), avant, "{octets:?}");
    }
}
