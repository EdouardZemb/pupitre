//! Les répliques du fonds, conduites par les services sur la paire de fichiers : deux ou trois
//! copies d'une même racine des données, modifiées hors ligne, puis réunies par l'échange de
//! `synchro` (H07). Le fichier inclut la suite des commandes, pour son aléa et ses commandes.

#[path = "../../services/tests/suite/mod.rs"]
mod suite;

mod commun;

use std::path::PathBuf;

use automerge::{Automerge, ObjId, ROOT, ReadDoc, Value};
use commandes::{Captation, Faite, Raison, Refus, Reponse, Sorte};
use noyau::donnees::fonds::{DUREE, ETIQUETTES, FORMAT, IDEE, IDEES, LIENS, PRISES, TEXTE};
use noyau::donnees::gestes;
use noyau::donnees::idee::cle_du_lien;
use proptest::prelude::*;
use services::lecture::teneur;
use services::{Contexte, Services};
use stockage::{Captations, FondsSurFichiers, RacineDesDonnees};
use suite::{
    AleaDeTest, HEURE, PRISE_DE_TEST, completion, contexte_de_test, etiquetage, fiche_wav, lien,
    regroupement, une_note, verser_une_prise, visee,
};

/// Les services de l'établi sur le fonds et les captations d'une racine des données.
type ServicesSurFichiers<'r> = Services<FondsSurFichiers<'r>, Captations<'r>>;

/// Les services de l'établi sur le fonds et les captations de `racine`.
fn services_sur(racine: &RacineDesDonnees) -> ServicesSurFichiers<'_> {
    Services::new(
        FondsSurFichiers::ouvrir(racine).expect("ouvrir le fonds"),
        racine.captations(),
        gestes::Appareil::Etabli,
    )
}

/// Les répliques d'un même fonds (`repliques`) : leurs racines des données, rouvertes, dans
/// l'ordre, ce que l'amorce a rendu, et le nombre des tirages de son aléa.
struct Repliques<const N: usize, R> {
    racines: [RacineDesDonnees; N],
    amorce: R,
    tirages: u64,
}

impl<const N: usize, R> Repliques<N, R> {
    /// Les services de chaque réplique, dans l'ordre (`services_sur`).
    fn services(&self) -> [ServicesSurFichiers<'_>; N] {
        self.racines.each_ref().map(services_sur)
    }

    /// L'aléa de chaque réplique, dans l'ordre : la n-ième a la graine n, et reprend le compte
    /// des tirages de l'amorce, si bien qu'aucune ne tire un identifiant déjà tiré.
    fn aleas(&self) -> [AleaDeTest; N] {
        std::array::from_fn(|rang| AleaDeTest {
            graine: u8::try_from(rang).expect("une graine tient dans un octet"),
            appels: self.tirages,
        })
    }
}

/// `N` répliques d'un même fonds, sur fichiers. Dans `<dossier jetable>/0`, les services de
/// l'établi, neufs, sous `AleaDeTest { graine: 0, appels: 0 }`, à `HEURE`, font naître le fonds
/// par `amorcer`, qui rend ce que le test en garde ; la racine des données fermée, son arbre est
/// copié dans `1`, `2`… ; puis chaque racine des données est rouverte.
fn repliques<const N: usize, R>(
    nom: &str,
    amorcer: impl FnOnce(&mut ServicesSurFichiers<'_>, &mut Contexte<'_>) -> R,
) -> Repliques<N, R> {
    let d = commun::dossier_jetable(nom);
    let chemins: [PathBuf; N] = std::array::from_fn(|rang| d.join(rang.to_string()));
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let amorce = {
        let racine = commun::ouvrir(&chemins[0]);
        let mut services = services_sur(&racine);
        amorcer(&mut services, &mut contexte_de_test(&mut alea))
    };
    for chemin in &chemins[1..] {
        commun::copier_l_arbre(&chemins[0], chemin);
    }
    Repliques {
        racines: chemins.map(|chemin| commun::ouvrir(&chemin)),
        amorce,
        tirages: alea.appels,
    }
}

/// Note l'idée « Un refrain qui monte, en ré », par `services`, sous `contexte`, et rend son
/// identifiant : l'amorce des tests qui n'ont besoin que d'une idée.
fn une_idee_notee(services: &mut ServicesSurFichiers<'_>, contexte: &mut Contexte<'_>) -> String {
    services
        .noter(&une_note("Un refrain qui monte, en ré"), contexte)
        .expect("la note est faite")
        .idee
}

/// Réunit les fonds de deux répliques (`commun::reunir`).
fn reunir(premiere: &mut ServicesSurFichiers<'_>, seconde: &mut ServicesSurFichiers<'_>) {
    commun::reunir(
        premiere
            .documents_mut()
            .document_mut()
            .expect("le fonds de la première réplique"),
        seconde
            .documents_mut()
            .document_mut()
            .expect("le fonds de la seconde réplique"),
    );
}

/// Le fonds de `services`, qui doit être né.
fn fonds_de<'s>(services: &'s ServicesSurFichiers<'_>) -> &'s Automerge {
    services
        .documents()
        .document()
        .expect("le fonds est né")
        .automerge()
}

/// L'objet à clés de l'idée `idee`, sous les idées de `fonds`.
fn objet_de_l_idee(fonds: &Automerge, idee: &str) -> ObjId {
    commun::objet(fonds, &commun::objet(fonds, &ROOT, IDEES), idee)
}

/// Les valeurs que porte `cle` sous `objet`, celles en conflit comprises (`get_all`).
fn valeurs(fonds: &Automerge, objet: &ObjId, cle: &str) -> Vec<Value<'static>> {
    fonds
        .get_all(objet, cle)
        .expect("lire une clé du fonds")
        .into_iter()
        .map(|(valeur, _)| valeur.into_owned())
        .collect()
}

/// Les chaînes que porte `cle` sous `objet`, celles en conflit comprises (`get_all`), dans l'ordre
/// de leurs octets ; une valeur d'un autre type fait échouer le test.
fn chaines(fonds: &Automerge, objet: &ObjId, cle: &str) -> Vec<String> {
    let mut chaines: Vec<String> = valeurs(fonds, objet, cle)
        .into_iter()
        .map(|valeur| valeur.into_string().expect("une chaîne"))
        .collect();
    chaines.sort();
    chaines
}

/// La valeur que présente Automerge sous `cle` de `objet` (`get`), qui doit y être.
fn presentee(fonds: &Automerge, objet: &ObjId, cle: &str) -> Value<'static> {
    let (valeur, _) = fonds
        .get(objet, cle)
        .expect("lire une clé du fonds")
        .expect("la clé porte une valeur");
    valeur.into_owned()
}

/// La prise d'empreinte `empreinte`, telle que la rend la vue d'une idée.
fn prise(empreinte: &str) -> Captation {
    Captation {
        sorte: Sorte::Prise,
        empreinte: empreinte.to_string(),
    }
}

/// A et B notées par les services dans une racine des données, fermée, puis copiée ; chaque
/// copie rouverte avec son aléa ; sur la première, A reliée à B, sur la seconde, B reliée à A,
/// hors ligne ; puis les deux copies réunies. Chacune ne porte sous `liens` qu'une clé, celle du
/// lien de A et B, ses bouts dans l'ordre de leurs octets (§ 30 · 9.3), où chaque copie a posé
/// `true` : deux valeurs, égales.
#[test]
fn cap02_7_deux_liens_poses_en_sens_contraires_n_en_font_qu_un() {
    let r = repliques::<2, _>("repliques-cap02-7", |services, contexte| {
        let a = services
            .noter(&une_note("Un refrain qui monte, en ré"), contexte)
            .expect("la note A est faite")
            .idee;
        let b = services
            .noter(&une_note("Un pont en mi mineur"), contexte)
            .expect("la note B est faite")
            .idee;
        (a, b)
    });
    let (a, b) = &r.amorce;
    let [mut premiere, mut seconde] = r.services();
    let [mut alea_de_la_premiere, mut alea_de_la_seconde] = r.aleas();
    premiere
        .relier(&lien(a, b), &mut contexte_de_test(&mut alea_de_la_premiere))
        .expect("A est reliée à B, sur la première copie");
    seconde
        .relier(&lien(b, a), &mut contexte_de_test(&mut alea_de_la_seconde))
        .expect("B est reliée à A, sur la seconde copie");
    reunir(&mut premiere, &mut seconde);

    let cle = cle_du_lien(a, b);
    for (nom, copie) in [("première", &premiere), ("seconde", &seconde)] {
        let fonds = fonds_de(copie);
        let liens = commun::objet(fonds, &ROOT, LIENS);
        let cles: Vec<String> = fonds.keys(&liens).collect();
        assert_eq!(
            cles,
            vec![cle.clone()],
            "les clés des liens de la {nom} copie"
        );
        let valeurs: Vec<Option<bool>> = fonds
            .get_all(&liens, cle.as_str())
            .expect("lire le lien")
            .into_iter()
            .map(|(valeur, _)| valeur.as_bool())
            .collect();
        assert_eq!(
            valeurs,
            [Some(true), Some(true)],
            "les valeurs du lien de la {nom} copie"
        );
    }
}

/// Les textes que deux répliques donnent hors ligne à une même idée, dans l'ordre de leurs
/// octets : la première réplique le premier, la seconde le second.
const TEXTES_CONCURRENTS: [&str; 2] = [
    "Un refrain qui monte, puis retombe",
    "Un refrain qui monte, puis s'éteint",
];

/// Sur chacune des deux répliques, l'idée `idee` complétée hors ligne de son texte
/// (`TEXTES_CONCURRENTS`), sous son aléa ; puis les deux répliques réunies.
fn completer_des_deux_cotes(
    services: &mut [ServicesSurFichiers<'_>; 2],
    aleas: &mut [AleaDeTest; 2],
    idee: &str,
) {
    for ((replique, alea), texte) in services.iter_mut().zip(aleas).zip(TEXTES_CONCURRENTS) {
        replique
            .completer(&completion(idee, texte), &mut contexte_de_test(alea))
            .expect("le texte de l'idée est complété");
    }
    let [premiere, seconde] = services;
    reunir(premiere, seconde);
}

/// X notée, puis copiée ; sur chaque copie, hors ligne, son texte complété d'un texte différent ;
/// puis les deux copies réunies. Aussitôt, chacune porte sous `idees/X/texte` les deux textes,
/// entiers : Automerge garde l'autre valeur dans ses conflits jusqu'à la prochaine écriture, et
/// ne mélange jamais deux textes caractère par caractère (§ 31 · 5).
#[test]
fn un_texte_change_des_deux_cotes_garde_ses_deux_valeurs() {
    let r = repliques::<2, _>("repliques-texte", une_idee_notee);
    let x = &r.amorce;
    let mut services = r.services();
    let mut aleas = r.aleas();
    completer_des_deux_cotes(&mut services, &mut aleas, x);

    for (nom, copie) in [("première", &services[0]), ("seconde", &services[1])] {
        let fonds = fonds_de(copie);
        assert_eq!(
            chaines(fonds, &objet_de_l_idee(fonds, x), TEXTE),
            TEXTES_CONCURRENTS.map(str::to_string),
            "les textes de X sur la {nom} copie"
        );
    }
}

/// X notée, puis copiée ; sur chaque copie, hors ligne, l'étiquette `pont` posée sur X ; puis
/// les deux copies réunies. Chacune ne porte sous les étiquettes de X qu'une clé, `pont`, à
/// `true` (§ 31 · 2), que chaque copie a posé : deux valeurs, égales.
#[test]
fn une_etiquette_posee_des_deux_cotes_n_en_fait_qu_une() {
    let r = repliques::<2, _>("repliques-etiquette", une_idee_notee);
    let x = &r.amorce;
    let [mut premiere, mut seconde] = r.services();
    let [mut alea_de_la_premiere, mut alea_de_la_seconde] = r.aleas();
    premiere
        .poser_une_etiquette(
            &etiquetage(x, "pont"),
            &mut contexte_de_test(&mut alea_de_la_premiere),
        )
        .expect("l'étiquette est posée sur X, sur la première copie");
    seconde
        .poser_une_etiquette(
            &etiquetage(x, "pont"),
            &mut contexte_de_test(&mut alea_de_la_seconde),
        )
        .expect("l'étiquette est posée sur X, sur la seconde copie");
    reunir(&mut premiere, &mut seconde);

    for (nom, copie) in [("première", &premiere), ("seconde", &seconde)] {
        let fonds = fonds_de(copie);
        let etiquettes = commun::objet(fonds, &objet_de_l_idee(fonds, x), ETIQUETTES);
        let cles: Vec<String> = fonds.keys(&etiquettes).collect();
        assert_eq!(
            cles,
            vec!["pont".to_string()],
            "les étiquettes de X sur la {nom} copie"
        );
        assert_eq!(
            valeurs(fonds, &etiquettes, "pont"),
            vec![Value::from(true); 2],
            "les valeurs de l'étiquette sur la {nom} copie"
        );
    }
}

/// Y et Z notées, chacune avec son texte, et une prise versée, sous l'idée V qui naît avec elle ;
/// puis copiées. Sur la première copie, hors ligne, la prise regroupée dans Y ; sur la seconde,
/// dans Z ; chacune retire V, laissée sans rien. Réunies, chaque copie porte sous
/// `<prise>/idee` les deux cibles, en conflit ; Y et Z restent visibles toutes deux, et la prise
/// ne paraît que dans la vue d'une seule, la même sur les deux copies : une prise appartient à
/// une seule idée (§ 30 · 9.3).
#[test]
fn deux_regroupements_d_une_meme_prise_ne_la_montrent_que_dans_une_idee() {
    let r = repliques::<2, _>("repliques-regroupements", |services, contexte| {
        let y = services
            .noter(&une_note("Un refrain qui monte, en ré"), contexte)
            .expect("la note Y est faite")
            .idee;
        let z = services
            .noter(&une_note("Un pont en mi mineur"), contexte)
            .expect("la note Z est faite")
            .idee;
        let e1 = verser_une_prise(services, PRISE_DE_TEST, &fiche_wav(30_000), contexte).empreinte;
        (y, z, e1)
    });
    let (y, z, e1) = &r.amorce;
    let [mut premiere, mut seconde] = r.services();
    let [mut alea_de_la_premiere, mut alea_de_la_seconde] = r.aleas();
    premiere
        .regrouper(
            &regroupement(e1, y),
            &mut contexte_de_test(&mut alea_de_la_premiere),
        )
        .expect("la prise se regroupe dans Y, sur la première copie");
    seconde
        .regrouper(
            &regroupement(e1, z),
            &mut contexte_de_test(&mut alea_de_la_seconde),
        )
        .expect("la prise se regroupe dans Z, sur la seconde copie");
    reunir(&mut premiere, &mut seconde);

    let mut cibles = vec![y.clone(), z.clone()];
    cibles.sort();
    let mut montrees = Vec::new();
    for (nom, copie) in [("première", &premiere), ("seconde", &seconde)] {
        let fonds = fonds_de(copie);
        let prises = commun::objet(fonds, &ROOT, PRISES);
        assert_eq!(
            chaines(fonds, &prises, &format!("{e1}/{IDEE}")),
            cibles,
            "les idées de la prise sur la {nom} copie"
        );
        assert_eq!(
            copie.idees(),
            Ok(cibles.clone()),
            "les idées visibles de la {nom} copie"
        );
        let montrent: Vec<String> = [y, z]
            .into_iter()
            .filter(|cible| {
                copie
                    .vue_d_une_idee(cible)
                    .expect("la vue d'une cible se lit")
                    .captations
                    .contains(&prise(e1))
            })
            .cloned()
            .collect();
        assert_eq!(
            montrent.len(),
            1,
            "les idées qui montrent la prise sur la {nom} copie : {montrent:?}"
        );
        montrees.push(montrent);
    }
    assert_eq!(
        montrees[1], montrees[0],
        "les deux copies ne montrent pas la prise dans la même idée"
    );
}

/// X, née d'une prise P, et Y, née d'une prise Q, sans rien d'autre ; puis copiées. Sur la
/// première copie, hors ligne, P regroupée dans Y, qui retire X, laissée sans rien ; sur la
/// seconde, Q regroupée dans X, qui retire Y (§ 30 · 9.3). Réunies, chaque copie porte X et Y
/// retirées toutes deux, chacune avec la prise de l'autre, et la liste des idées est vide : le
/// retrait s'est décidé sur ce que voyait chaque copie, un cas que le lot 3 traitera.
#[test]
fn deux_regroupements_croises_retirent_les_deux_idees() {
    let r = repliques::<2, _>("repliques-croises", |services, contexte| {
        let p = verser_une_prise(services, PRISE_DE_TEST, &fiche_wav(30_000), contexte);
        let q = verser_une_prise(
            services,
            b"une seconde prise de test",
            &fiche_wav(64_000),
            contexte,
        );
        (p, q)
    });
    let (p, q) = &r.amorce;
    let (x, y) = (&p.idee, &q.idee);
    let [mut premiere, mut seconde] = r.services();
    let [mut alea_de_la_premiere, mut alea_de_la_seconde] = r.aleas();
    premiere
        .regrouper(
            &regroupement(&p.empreinte, y),
            &mut contexte_de_test(&mut alea_de_la_premiere),
        )
        .expect("P se regroupe dans Y, sur la première copie");
    seconde
        .regrouper(
            &regroupement(&q.empreinte, x),
            &mut contexte_de_test(&mut alea_de_la_seconde),
        )
        .expect("Q se regroupe dans X, sur la seconde copie");
    reunir(&mut premiere, &mut seconde);

    for (nom, copie) in [("première", &premiere), ("seconde", &seconde)] {
        assert_eq!(
            copie
                .vue_d_une_idee(x)
                .map(|vue| (vue.retiree, vue.captations)),
            Ok((Some(HEURE), vec![prise(&q.empreinte)])),
            "X sur la {nom} copie"
        );
        assert_eq!(
            copie
                .vue_d_une_idee(y)
                .map(|vue| (vue.retiree, vue.captations)),
            Ok((Some(HEURE), vec![prise(&p.empreinte)])),
            "Y sur la {nom} copie"
        );
        assert_eq!(
            copie.idees(),
            Ok(Vec::new()),
            "les idées visibles de la {nom} copie"
        );
    }
}

/// A notée, puis copiée ; la même prise, importée par l'établi sur chaque copie, hors ligne, à
/// `HEURE` sur la première, une minute plus tard sur la seconde, sous l'idée qui naît avec elle,
/// V1 puis V2, que l'heure date, jamais la prise. Réunies, chaque copie ne porte sous les prises
/// qu'une clé par fait, `<prise>/duree`, `<prise>/format` et `<prise>/idee`, ni `<prise>/le` ni
/// `<prise>/par` : la date et l'appareil d'une prise n'ont pas de clé (§ 31 · 2). Chaque copie
/// a posé chaque fait : sous chacun, deux valeurs, V1 et V2 sous `idee`, deux formats et deux
/// durées égaux, dont Automerge présente la valeur. L'idée que la réunion laisse sans la prise
/// reste visible, vide : aucun geste ne l'a retirée.
#[test]
fn une_prise_nee_sur_deux_repliques_ne_cache_aucun_de_ses_faits() {
    let r = repliques::<2, _>("repliques-meme-prise", une_idee_notee);
    let a = &r.amorce;
    let [mut premiere, mut seconde] = r.services();
    let [mut alea_de_la_premiere, mut alea_de_la_seconde] = r.aleas();
    let v1 = verser_une_prise(
        &mut premiere,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte_de_test(&mut alea_de_la_premiere),
    );
    let mut contexte_de_la_seconde = contexte_de_test(&mut alea_de_la_seconde);
    contexte_de_la_seconde.heure = HEURE + 60;
    let v2 = verser_une_prise(
        &mut seconde,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte_de_la_seconde,
    );
    assert_eq!(
        v1.empreinte, v2.empreinte,
        "les deux copies versent la même prise"
    );
    reunir(&mut premiere, &mut seconde);

    let e = &v1.empreinte;
    let (cle_de_l_idee, cle_du_format, cle_de_la_duree) = (
        format!("{e}/{IDEE}"),
        format!("{e}/{FORMAT}"),
        format!("{e}/{DUREE}"),
    );
    let mut idees_de_la_prise = vec![v1.idee.clone(), v2.idee.clone()];
    idees_de_la_prise.sort();
    let mut visibles = vec![a.clone(), v1.idee.clone(), v2.idee.clone()];
    visibles.sort();
    for (nom, copie) in [("première", &premiere), ("seconde", &seconde)] {
        let fonds = fonds_de(copie);
        let prises = commun::objet(fonds, &ROOT, PRISES);
        let cles: Vec<String> = fonds.keys(&prises).collect();
        assert_eq!(
            cles,
            vec![
                cle_de_la_duree.clone(),
                cle_du_format.clone(),
                cle_de_l_idee.clone()
            ],
            "les clés des prises de la {nom} copie"
        );
        assert_eq!(
            chaines(fonds, &prises, &cle_de_l_idee),
            idees_de_la_prise,
            "les idées de la prise sur la {nom} copie"
        );
        assert_eq!(
            valeurs(fonds, &prises, &cle_du_format),
            vec![Value::from("wav"); 2],
            "les formats de la prise sur la {nom} copie"
        );
        assert_eq!(
            presentee(fonds, &prises, &cle_du_format),
            Value::from("wav"),
            "le format présenté sur la {nom} copie"
        );
        assert_eq!(
            valeurs(fonds, &prises, &cle_de_la_duree),
            vec![Value::from(30_000u64); 2],
            "les durées de la prise sur la {nom} copie"
        );
        assert_eq!(
            presentee(fonds, &prises, &cle_de_la_duree),
            Value::from(30_000u64),
            "la durée présentée sur la {nom} copie"
        );
        assert_eq!(
            copie.vue_d_une_idee(&v1.idee).map(|vue| vue.le),
            Ok(HEURE),
            "la date de V1 sur la {nom} copie"
        );
        assert_eq!(
            copie.vue_d_une_idee(&v2.idee).map(|vue| vue.le),
            Ok(HEURE + 60),
            "la date de V2 sur la {nom} copie"
        );

        let porteuse = presentee(fonds, &prises, &cle_de_l_idee)
            .into_string()
            .expect("l'idée présentée de la prise est une chaîne");
        let laissee = if porteuse == v1.idee {
            &v2.idee
        } else {
            &v1.idee
        };
        assert_eq!(
            copie.vue_d_une_idee(&porteuse).map(|vue| vue.captations),
            Ok(vec![prise(e)]),
            "l'idée présentée de la prise sur la {nom} copie"
        );
        assert_eq!(
            copie.vue_d_une_idee(laissee).map(|vue| vue.captations),
            Ok(Vec::new()),
            "l'idée laissée sans la prise sur la {nom} copie"
        );
        assert_eq!(
            copie.idees(),
            Ok(visibles.clone()),
            "les idées visibles de la {nom} copie"
        );
    }
}

/// X notée, puis copiée ; son texte complété hors ligne sur chaque copie, d'un texte différent ;
/// puis les deux copies réunies : le texte de X est en conflit. Sur la première copie, X
/// complétée d'un troisième texte, puis ce complément annulé : l'inverse repose le texte tel que
/// le fonds le présentait avant le complément, sa seule valeur présentée (§ 30 · 11). Le fonds a
/// la teneur d'avant le complément, et le texte de X n'a plus qu'une valeur, celle-ci.
#[test]
fn l_inverse_d_un_texte_en_conflit_rend_la_valeur_presentee() {
    let r = repliques::<2, _>("repliques-annulation", une_idee_notee);
    let x = &r.amorce;
    let mut services = r.services();
    let mut aleas = r.aleas();
    completer_des_deux_cotes(&mut services, &mut aleas, x);
    let [premiere, _] = &mut services;

    let fonds = fonds_de(premiere);
    let idee_x = objet_de_l_idee(fonds, x);
    assert_eq!(
        chaines(fonds, &idee_x, TEXTE).len(),
        2,
        "le texte de X est en conflit"
    );
    let texte_presente = presentee(fonds, &idee_x, TEXTE);
    let avant = teneur(fonds).expect("la teneur du fonds s'écrit");
    premiere
        .completer(
            &completion(x, "Un refrain qui hésite, puis repart"),
            &mut contexte_de_test(&mut aleas[0]),
        )
        .expect("le texte de X est complété");
    premiere
        .annuler(&mut contexte_de_test(&mut aleas[0]))
        .expect("le complément est annulé");

    let fonds = fonds_de(premiere);
    assert_eq!(
        teneur(fonds).expect("la teneur du fonds s'écrit"),
        avant,
        "l'annulation ne rend pas la teneur d'avant le complément"
    );
    assert_eq!(
        valeurs(fonds, &idee_x, TEXTE),
        vec![texte_presente],
        "le texte de X après l'annulation"
    );
}

/// Les textes des deux idées notées sur la première réplique avant les copies, au test des trois
/// répliques.
const NOTES_DES_TROIS: [&str; 2] = ["Un refrain qui monte, en ré", "Un pont en mi mineur"];

/// Les octets des deux prises versées sur la première réplique avant les copies, au test des
/// trois répliques, chacune sous l'idée qui naît avec elle.
const PRISES_DES_TROIS: [&[u8]; 2] = [
    b"une premiere prise des trois",
    b"une seconde prise des trois",
];

/// Le nombre des idées nées avant les copies, au test des trois répliques : les notées, puis
/// celles des prises.
const IDEES_DES_TROIS: usize = NOTES_DES_TROIS.len() + PRISES_DES_TROIS.len();

/// Le nombre des gestes de tri tirés, au test des trois répliques.
const GESTES_TIRES: usize = 30;

/// Les textes qu'un geste tiré peut donner à une idée : inventés.
const TEXTES_TIRES: &[&str] = &[
    "Une basse qui marche",
    "Un accord suspendu",
    "Une coda en fondu",
];

/// Les étiquettes qu'un geste tiré peut poser sur une idée, ou en ôter.
const ETIQUETTES_TIREES: &[&str] = &["pont", "refrain"];

/// Les paires de répliques qu'une réunion tirée peut réunir, par leurs rangs.
const PAIRES: &[(usize, usize)] = &[(0, 1), (1, 2), (0, 2)];

/// Un geste de tri tiré : une idée s'y désigne par son rang parmi celles nées avant les copies,
/// une prise par son rang dans `PRISES_DES_TROIS`.
#[derive(Clone, Debug)]
enum GesteTire {
    /// Compléter le texte d'une idée.
    Completer { idee: usize, texte: &'static str },
    /// Poser une étiquette sur une idée.
    PoserUneEtiquette {
        idee: usize,
        etiquette: &'static str,
    },
    /// Ôter une étiquette d'une idée.
    OterUneEtiquette {
        idee: usize,
        etiquette: &'static str,
    },
    /// Relier deux idées.
    Relier { a: usize, b: usize },
    /// Délier deux idées.
    Delier { a: usize, b: usize },
    /// Regrouper une prise dans une idée.
    Regrouper { prise: usize, cible: usize },
    /// Retirer une idée.
    Retirer { idee: usize },
    /// Rétablir une idée.
    Retablir { idee: usize },
}

/// Un pas du test des trois répliques : un geste de tri sur la réplique de rang `replique`, puis,
/// s'il y en a une, la réunion de deux répliques.
#[derive(Clone, Debug)]
struct Pas {
    replique: usize,
    geste: GesteTire,
    reunion: Option<(usize, usize)>,
}

/// Un geste de tri parmi les huit, chacun aussi probable, ses arguments tirés, refusés compris :
/// une idée reliée à elle-même, une prise regroupée dans son idée, une idée retirée touchée.
fn un_geste_tire() -> impl Strategy<Value = GesteTire> {
    let idee = || 0..IDEES_DES_TROIS;
    let etiquette = || prop::sample::select(ETIQUETTES_TIREES);
    prop_oneof![
        (idee(), prop::sample::select(TEXTES_TIRES))
            .prop_map(|(idee, texte)| GesteTire::Completer { idee, texte }),
        (idee(), etiquette())
            .prop_map(|(idee, etiquette)| GesteTire::PoserUneEtiquette { idee, etiquette }),
        (idee(), etiquette())
            .prop_map(|(idee, etiquette)| GesteTire::OterUneEtiquette { idee, etiquette }),
        (idee(), idee()).prop_map(|(a, b)| GesteTire::Relier { a, b }),
        (idee(), idee()).prop_map(|(a, b)| GesteTire::Delier { a, b }),
        (0..PRISES_DES_TROIS.len(), idee())
            .prop_map(|(prise, cible)| GesteTire::Regrouper { prise, cible }),
        idee().prop_map(|idee| GesteTire::Retirer { idee }),
        idee().prop_map(|idee| GesteTire::Retablir { idee }),
    ]
}

/// Un pas : la réplique, chacune aussi probable, le geste (`un_geste_tire`), puis, une fois sur
/// trois, la réunion d'une paire de répliques (`PAIRES`), chacune aussi probable.
fn un_pas() -> impl Strategy<Value = Pas> {
    (
        0..3usize,
        un_geste_tire(),
        prop::option::weighted(1.0 / 3.0, prop::sample::select(PAIRES)),
    )
        .prop_map(|(replique, geste, reunion)| Pas {
            replique,
            geste,
            reunion,
        })
}

/// Joue `geste` par `services`, sous `contexte` : `idees` sont les idées nées avant les copies,
/// `prises` les empreintes des prises versées, chacune à son rang. Rend ce que rend la commande.
fn jouer(
    services: &mut ServicesSurFichiers<'_>,
    geste: &GesteTire,
    idees: &[String],
    prises: &[String],
    contexte: &mut Contexte<'_>,
) -> Reponse<Faite> {
    match *geste {
        GesteTire::Completer { idee, texte } => {
            services.completer(&completion(&idees[idee], texte), contexte)
        }
        GesteTire::PoserUneEtiquette { idee, etiquette } => {
            services.poser_une_etiquette(&etiquetage(&idees[idee], etiquette), contexte)
        }
        GesteTire::OterUneEtiquette { idee, etiquette } => {
            services.oter_une_etiquette(&etiquetage(&idees[idee], etiquette), contexte)
        }
        GesteTire::Relier { a, b } => services.relier(&lien(&idees[a], &idees[b]), contexte),
        GesteTire::Delier { a, b } => services.delier(&lien(&idees[a], &idees[b]), contexte),
        GesteTire::Regrouper { prise, cible } => {
            services.regrouper(&regroupement(&prises[prise], &idees[cible]), contexte)
        }
        GesteTire::Retirer { idee } => services.retirer(&visee(&idees[idee]), contexte),
        GesteTire::Retablir { idee } => services.retablir(&visee(&idees[idee]), contexte),
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 16,
        max_shrink_iters: 64,
        max_shrink_time: 60_000,
        ..ProptestConfig::default()
    })]

    // § 31 · 10, test 3 : trois répliques, opérations concurrentes aléatoires, synchronisation dans un ordre aléatoire
    /// Quatre idées nées sur la première réplique, deux notées, deux nées d'une prise versée, puis
    /// copiées dans la deuxième et la troisième ; trente gestes de tri tirés (`un_pas`), chacun
    /// sur une réplique tirée, ses arguments tirés, refusés compris, et, en chemin, des réunions
    /// de deux répliques tirées ; puis les réunions de la première et de la deuxième, de la
    /// deuxième et de la troisième, et de nouveau de la première et de la deuxième. Les trois
    /// répliques ont alors les mêmes heads et la même teneur (§ 31 · 2.6). Un geste fait l'est
    /// sans avertissement ; un geste refusé l'est d'un constat (`Raison::Impossible`).
    ///
    /// Seize cas, et une réduction bornée, à 64 essais et 60 secondes : un échec se lit comme un
    /// échec, sous les délais des jobs `verifier` et `windows`, et non comme un délai dépassé.
    /// Chaque ouverture d'une racine des données tire l'acteur de son document
    /// (`ActorId::random()`) : la valeur que présente Automerge parmi des valeurs en conflit, et
    /// donc les gestes refusés, changent d'un passage à l'autre, et un cas qui échoue se rejoue
    /// sans garantie.
    #[test]
    fn trois_repliques_convergent(pas in prop::collection::vec(un_pas(), GESTES_TIRES)) {
        let r = repliques::<3, _>("repliques-trois", |services, contexte| {
            let mut idees = Vec::new();
            for texte in NOTES_DES_TROIS {
                let notee = services
                    .noter(&une_note(texte), contexte)
                    .expect("la note est faite");
                idees.push(notee.idee);
            }
            let mut prises = Vec::new();
            for octets in PRISES_DES_TROIS {
                let versee = verser_une_prise(services, octets, &fiche_wav(30_000), contexte);
                idees.push(versee.idee);
                prises.push(versee.empreinte);
            }
            (idees, prises)
        });
        let (idees, prises) = &r.amorce;
        let mut services = r.services();
        let mut aleas = r.aleas();
        for (rang, un) in pas.iter().enumerate() {
            let rendu = jouer(
                &mut services[un.replique],
                &un.geste,
                idees,
                prises,
                &mut contexte_de_test(&mut aleas[un.replique]),
            );
            prop_assert!(
                matches!(
                    rendu,
                    Ok(Faite { avertissement: None })
                        | Err(Refus { raison: Raison::Impossible, .. })
                ),
                "le geste {} ({:?}) rend {:?}",
                rang + 1,
                un.geste,
                rendu
            );
            if let Some((i, j)) = un.reunion {
                let [premiere, seconde] = services
                    .get_disjoint_mut([i, j])
                    .expect("deux répliques distinctes");
                reunir(premiere, seconde);
            }
        }
        for (i, j) in [(0, 1), (1, 2), (0, 1)] {
            let [premiere, seconde] = services
                .get_disjoint_mut([i, j])
                .expect("deux répliques distinctes");
            reunir(premiere, seconde);
        }

        let heads = services.each_ref().map(|replique| {
            let mut heads = fonds_de(replique).get_heads();
            heads.sort();
            heads
        });
        let teneurs = services
            .each_ref()
            .map(|replique| teneur(fonds_de(replique)).expect("la teneur du fonds s'écrit"));
        for (rang, (heads_de_la_replique, teneur_de_la_replique)) in
            heads.iter().zip(&teneurs).enumerate().skip(1)
        {
            prop_assert_eq!(
                heads_de_la_replique,
                &heads[0],
                "la réplique {} n'a pas les heads de la première",
                rang + 1
            );
            prop_assert_eq!(
                teneur_de_la_replique,
                &teneurs[0],
                "la réplique {} n'a pas la teneur de la première",
                rang + 1
            );
        }
    }
}
