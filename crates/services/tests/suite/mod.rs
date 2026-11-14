//! La suite des commandes : des tests écrits une fois, génériques sur une paire d'adaptateurs
//! (`Paire`), que rejouent la paire en mémoire (`crates/services/tests/commandes_en_memoire.rs`)
//! et celle des fichiers, au stockage (`crates/stockage/tests/commandes_sur_fichiers.rs`) ;
//! l'heure et l'aléa des tests, l'identifiant de l'import des fiches qu'ils versent, la prise de
//! test, les commandes des gestes de tri, la migration de test, les migrations qui la portent, et
//! une migration qui attend ses dépendances. Le fichier s'inclut par `mod suite;` dans les tests
//! des services, et par `#[path = "../../services/tests/suite/mod.rs"]` dans ceux du stockage :
//! il ne nomme que ce que les deux voient, les services, les commandes, le noyau et `automerge`.

// Chaque fichier qui inclut la suite n'en emploie qu'une partie.
#![allow(dead_code)]

use std::collections::BTreeSet;

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{ActorId, Automerge, Change, ObjType, ROOT, ReadDoc};
use commandes::{
    Captation, Completer, EntreeDuJournal, Etiqueter, Faite, Notee, Noter, Raison, Refus,
    Regrouper, Relier, Reponse, Viser, VueDUneIdee,
};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, GENESE, IDENTIFIANT, LIGNEE, PROPRIETAIRE, VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes;
use noyau::donnees::idee::cle_du_lien;
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::version::{Migration, message_de_la_migration};
use services::lecture::{teneur, version_de_schema};
use services::{Alea, Captations, Contexte, Documents, Services, Versee};

/// L'heure des tests, en secondes : le 14 novembre 2023, 22 h 13 min 20 s UTC.
pub const HEURE: i64 = 1_700_000_000;

/// Le lendemain de `HEURE`, en secondes : le 15 novembre 2023, 22 h 13 min 20 s UTC.
pub const LENDEMAIN: i64 = HEURE + 86_400;

/// Les octets de la prise de test.
pub const PRISE_DE_TEST: &[u8] = b"une prise de test";

/// L'identifiant de l'import d'où viennent les fiches que versent les tests :
/// `ulid(HEURE, [0xFF; 10])`.
pub const IMPORT: &str = "01HF7YAT00ZZZZZZZZZZZZZZZZ";

// Les identifiants d'une naissance sous `AleaDeTest { graine: 0, appels: 0 }`, à `HEURE`, tirés
// dans cet ordre : l'ULID du document, celui de la lignée, l'identifiant de propriétaire, puis
// l'ULID de l'idée que note la première note.
const ULID_DU_DOCUMENT: &str = "01HF7YAT000000000000000001";
const ULID_DE_LA_LIGNEE: &str = "01HF7YAT000000000000000002";
const IDENTIFIANT_DE_PROPRIETAIRE: &str = "00000000-0000-4000-8000-000000000003";
pub const IDEE: &str = "01HF7YAT000000000000000004";
/// L'idée d'une seconde note, sur le fonds né : le tirage qui suit.
pub const SECONDE_IDEE: &str = "01HF7YAT000000000000000005";

/// L'aléa des tests : à chaque appel, l'octet 0 vaut `graine`, les huit derniers le rang de
/// l'appel, en gros-boutiste, compté de 1 ; le reste, 0. Chaque réplique d'un test a sa graine.
pub struct AleaDeTest {
    /// L'octet 0 de chaque tirage.
    pub graine: u8,
    /// Le nombre des appels déjà faits.
    pub appels: u64,
}

impl Alea for AleaDeTest {
    /// `octets` en a neuf au moins : un ULID en tire dix, un identifiant de propriétaire seize.
    fn remplir(&mut self, octets: &mut [u8]) {
        self.appels += 1;
        let rang = self.appels.to_be_bytes();
        let debut = octets.len() - rang.len();
        octets.fill(0);
        octets[0] = self.graine;
        octets[debut..].copy_from_slice(&rang);
    }
}

/// Ce que la suite demande à une paire d'adaptateurs.
pub trait Paire {
    type D: Documents;
    type C: Captations;
    /// Des services sur une racine des données sans document (fichiers), ou sans fonds (mémoire).
    fn neufs(&mut self) -> Services<Self::D, Self::C>;
    /// Ferme `services`, puis les rouvre selon `migrations`.
    fn rouvrir(
        &mut self,
        services: Services<Self::D, Self::C>,
        migrations: &[Migration<'_>],
    ) -> Services<Self::D, Self::C>;
}

/// Le conteneur de la racine du document que crée la migration de test : un objet à clés, où la
/// donnée nouvelle d'un objet s'écrit en valeurs simples, sous `<objet>/<élément>` (§ 31 · 2.4).
pub const CONTENEUR_DE_LA_MIGRATION: &str = "elements_de_test";

/// La migration de test vers la version de schéma 2, engendrée à l'exécution depuis `GENESE`, de
/// façon déterministe, comme la recette de la genèse (`genese_par_la_recette`, dans
/// `crates/stockage/tests/genese.rs`) : l'acteur `ACTEUR_DE_LA_GENESE`, qui pose
/// `VERSION_DE_SCHEMA` à 2 et crée `CONTENEUR_DE_LA_MIGRATION`, un objet à clés, à la racine du
/// document, sous le message `message_de_la_migration(2)`, à l'heure 0. Rend les octets de sa
/// modification (`raw_bytes()`), qui ne dépend que de la genèse.
pub fn migration_de_test() -> Vec<u8> {
    let mut doc = Automerge::new().with_actor(ActorId::from(ACTEUR_DE_LA_GENESE));
    doc.apply_changes([Change::try_from(GENESE).expect("lire la genèse")])
        .expect("appliquer la genèse");
    let mut tx = doc.transaction();
    tx.put(ROOT, VERSION_DE_SCHEMA, 2u64)
        .expect("poser la version de schéma 2");
    tx.put_object(ROOT, CONTENEUR_DE_LA_MIGRATION, ObjType::Map)
        .expect("créer le conteneur de la migration");
    let message = message_de_la_migration(2);
    let options = CommitOptions::default()
        .with_message(message.as_str())
        .with_time(0);
    let (h, _) = tx.commit_with(options);
    doc.get_change_by_hash(&h.expect("la migration est une modification"))
        .expect("la modification de la migration")
        .raw_bytes()
        .to_vec()
}

/// Les migrations de test, une seule : `octets`, vers la version de schéma 2, apportée par la
/// version de Pupitre `0.0.0-essai`.
pub fn migrations_de_test(octets: &[u8]) -> [Migration<'_>; 1] {
    [Migration {
        vers: 2,
        octets,
        version_de_pupitre: "0.0.0-essai",
    }]
}

/// Une migration vers la version de schéma 2, engendrée sur un document sans `GENESE` : l'acteur
/// `autre-genese-de-test` y écrit d'abord une modification qu'aucun fonds ne porte, puis la
/// migration, qui dépend d'elle. Un fonds qui la reçoit n'a pas cette dépendance : `apply_changes`
/// la garde en attente, sans erreur. Rend les octets de la migration (`raw_bytes()`).
pub fn migration_sur_une_autre_genese() -> Vec<u8> {
    let mut doc = Automerge::new().with_actor(ActorId::from(b"autre-genese-de-test".as_slice()));
    let mut tx = doc.transaction();
    tx.put(ROOT, VERSION_DE_SCHEMA, 1u64)
        .expect("poser la version de schéma 1");
    let options = CommitOptions::default()
        .with_message("Une autre genèse")
        .with_time(0);
    let (autre_genese, _) = tx.commit_with(options);
    autre_genese.expect("l'autre genèse est une modification");
    let mut tx = doc.transaction();
    tx.put(ROOT, VERSION_DE_SCHEMA, 2u64)
        .expect("poser la version de schéma 2");
    let message = message_de_la_migration(2);
    let options = CommitOptions::default()
        .with_message(message.as_str())
        .with_time(0);
    let (h, _) = tx.commit_with(options);
    doc.get_change_by_hash(&h.expect("la migration est une modification"))
        .expect("la modification de la migration")
        .raw_bytes()
        .to_vec()
}

/// Le contexte des tests : `HEURE`, et `alea`.
pub fn contexte_de_test(alea: &mut AleaDeTest) -> Contexte<'_> {
    Contexte { heure: HEURE, alea }
}

/// La commande qui note `texte`, au décalage local nul.
pub fn une_note(texte: &str) -> Noter {
    Noter {
        texte: texte.to_string(),
        decalage_minutes: 0,
    }
}

/// L'entrée du journal de `message`, à `heure`.
fn entree(message: &str, heure: i64) -> EntreeDuJournal {
    EntreeDuJournal {
        message: message.to_string(),
        heure,
    }
}

/// La chaîne que porte `cle` à la racine du document ; absente, ou de tout autre type, `None`.
fn chaine_de_la_racine(doc: &Automerge, cle: &str) -> Option<String> {
    let (valeur, _) = doc.get(ROOT, cle).ok()??;
    valeur.into_string().ok()
}

/// La fiche d'une prise WAV de `duree_ms` millisecondes.
pub fn fiche_wav(duree_ms: u64) -> gestes::Fiche {
    gestes::Fiche {
        format: "wav".to_string(),
        duree_ms: Some(duree_ms),
    }
}

/// La provenance des fiches que versent les tests : importées à l'établi, par l'import `IMPORT`.
/// À INC-1, une prise n'entre sur l'établi que par l'import.
pub fn importee() -> gestes::Provenance {
    gestes::Provenance::Importee {
        par: gestes::Appareil::Etabli,
        depuis: IMPORT.to_string(),
    }
}

/// Verse la prise `octets`, de fiche `fiche`, importée (`importee`), au décalage local nul : son
/// idée naît avec elle, à l'heure du contexte.
pub fn verser_une_prise<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    octets: &[u8],
    fiche: &gestes::Fiche,
    contexte: &mut Contexte<'_>,
) -> Versee {
    services
        .verser_une_fiche(
            gestes::Sorte::Prise,
            octets,
            fiche,
            &importee(),
            0,
            contexte,
        )
        .expect("la prise se verse")
}

/// La commande qui complète le texte de l'idée `idee` par `texte`, au décalage local nul.
pub fn completion(idee: &str, texte: &str) -> Completer {
    Completer {
        idee: idee.to_string(),
        texte: texte.to_string(),
        decalage_minutes: 0,
    }
}

/// La commande qui pose ou ôte l'étiquette `etiquette` de l'idée `idee`, au décalage local nul.
pub fn etiquetage(idee: &str, etiquette: &str) -> Etiqueter {
    Etiqueter {
        idee: idee.to_string(),
        etiquette: etiquette.to_string(),
        decalage_minutes: 0,
    }
}

/// La commande qui relie ou délie les idées `a` et `b`, au décalage local nul.
pub fn lien(a: &str, b: &str) -> Relier {
    Relier {
        a: a.to_string(),
        b: b.to_string(),
        decalage_minutes: 0,
    }
}

/// La commande qui regroupe la prise `empreinte` dans l'idée `cible`, au décalage local nul.
pub fn regroupement(empreinte: &str, cible: &str) -> Regrouper {
    Regrouper {
        captation: Captation {
            sorte: commandes::Sorte::Prise,
            empreinte: empreinte.to_string(),
        },
        cible: cible.to_string(),
        decalage_minutes: 0,
    }
}

/// La commande qui retire ou rétablit l'idée `idee`, au décalage local nul.
pub fn visee(idee: &str) -> Viser {
    Viser {
        idee: idee.to_string(),
        decalage_minutes: 0,
    }
}

/// Verse `octets`, de sorte `sorte` et de fiche `fiche`, importés (`importee`), au décalage local
/// nul, sans avertissement : son idée naît avec eux, à l'heure du contexte. Rend l'empreinte de la
/// captation versée.
fn verser_sans_avertissement<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    sorte: gestes::Sorte,
    octets: &[u8],
    fiche: &gestes::Fiche,
    contexte: &mut Contexte<'_>,
) -> String {
    let versee = services
        .verser_une_fiche(sorte, octets, fiche, &importee(), 0, contexte)
        .expect("la fiche du témoin se verse");
    assert_eq!(versee.avertissement, None, "le versement de {sorte:?}");
    versee.empreinte
}

/// Les gestes du témoin du test 11 (§ 31 · 10), écrits une seule fois, à l'heure et sous l'aléa
/// de `contexte`, au décalage local nul, chacun fait sans avertissement : noter « Un refrain qui
/// monte, en ré », A, qui fait naître le fonds s'il n'est pas né ; poser `refrain` sur A ; noter
/// « Un pont en mi mineur », B ; relier A et B ; verser une prise WAV de 30 000 ms, un MIDI
/// `smf1`, puis un texte `brut`, chacun importé (`importee`), sous l'idée qui naît avec lui ;
/// regrouper le MIDI dans A, ce qui retire l'idée née avec lui.
pub fn temoin<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
) {
    let fait = Ok(Faite {
        avertissement: None,
    });
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), contexte)
        .expect("le témoin note A");
    assert_eq!(a.avertissement, None, "la note A du témoin");
    assert_eq!(
        services.poser_une_etiquette(&etiquetage(&a.idee, "refrain"), contexte),
        fait,
        "l'étiquette du témoin"
    );
    let b = services
        .noter(&une_note("Un pont en mi mineur"), contexte)
        .expect("le témoin note B");
    assert_eq!(b.avertissement, None, "la note B du témoin");
    assert_eq!(
        services.relier(&lien(&a.idee, &b.idee), contexte),
        fait,
        "le lien du témoin"
    );
    verser_sans_avertissement(
        services,
        gestes::Sorte::Prise,
        b"une prise du temoin",
        &fiche_wav(30_000),
        contexte,
    );
    let midi = verser_sans_avertissement(
        services,
        gestes::Sorte::MidiVerseSeul,
        b"un midi du temoin",
        &gestes::Fiche {
            format: "smf1".to_string(),
            duree_ms: None,
        },
        contexte,
    );
    verser_sans_avertissement(
        services,
        gestes::Sorte::TexteVerse,
        b"un texte du temoin",
        &gestes::Fiche {
            format: "brut".to_string(),
            duree_ms: None,
        },
        contexte,
    );
    let regroupement_du_midi = Regrouper {
        captation: Captation {
            sorte: commandes::Sorte::MidiVerseSeul,
            empreinte: midi,
        },
        cible: a.idee,
        decalage_minutes: 0,
    };
    assert_eq!(
        services.regrouper(&regroupement_du_midi, contexte),
        fait,
        "le regroupement du MIDI dans A"
    );
}

/// Le refus d'une commande qui ne se fait pas sur le fonds tel qu'il est : `Raison::Impossible`,
/// et `message`.
fn impossible(message: String) -> Refus {
    Refus {
        raison: Raison::Impossible,
        message,
    }
}

/// Les prises `empreintes`, telles que les rend la vue d'une idée : dans l'ordre de leurs
/// empreintes.
fn prises(empreintes: &[&str]) -> Vec<Captation> {
    let mut rangees = empreintes.to_vec();
    rangees.sort();
    rangees
        .into_iter()
        .map(|empreinte| Captation {
            sorte: commandes::Sorte::Prise,
            empreinte: empreinte.to_string(),
        })
        .collect()
}

/// Le geste de tri `rendu` est fait, et le journal porte une entrée de plus : `message`, au
/// lendemain (`LENDEMAIN`), l'heure du contexte des gestes.
fn parait_au_journal<D: Documents, C: Captations>(
    services: &Services<D, C>,
    journal: &mut Vec<EntreeDuJournal>,
    rendu: Reponse<Faite>,
    message: &str,
) {
    rendu.unwrap_or_else(|refus| panic!("le geste « {message} » est refusé : {refus:?}"));
    journal.push(entree(message, LENDEMAIN));
    assert_eq!(
        services.journal(),
        Ok(journal.clone()),
        "après « {message} »"
    );
}

/// La teneur du fonds (§ 31 · 2.6), qui doit être né.
fn teneur_du_fonds<D: Documents, C: Captations>(services: &Services<D, C>) -> String {
    teneur(services.documents().fonds().expect("le fonds est né"))
        .expect("la teneur du fonds s'écrit")
}

/// Le refus d'« annuler » sans dernier geste de tri : `Raison::RienAAnnuler`.
fn rien_a_annuler() -> Reponse<Faite> {
    Err(Refus {
        raison: Raison::RienAAnnuler,
        message: "Aucun geste de tri à annuler depuis l'ouverture.".to_string(),
    })
}

/// « Annuler » est fait : le journal porte, en dernier, « Annulé : » suivi de `message`, à l'heure
/// du contexte.
fn annule<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
    message: &str,
) {
    assert_eq!(
        services.annuler(contexte),
        Ok(Faite {
            avertissement: None
        }),
        "l'annulation de « {message} »"
    );
    let journal = services.journal().expect("le journal se lit");
    assert_eq!(
        journal.last(),
        Some(&entree(&format!("Annulé : {message}"), contexte.heure)),
        "{journal:?}"
    );
}

/// « Annuler » est refusé, rien à annuler, et le journal ne change pas.
fn rien_n_est_annule<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
) {
    let journal = services.journal();
    assert_eq!(services.annuler(contexte), rien_a_annuler());
    assert_eq!(services.journal(), journal);
}

/// Le geste de tri `rendu`, fait sous `message`, a changé le fonds ; annulé, le fonds revient à
/// la teneur `avant`, lue avant le geste, et les deux dernières entrées du journal sont le geste,
/// puis « Annulé : » suivi de son message, à l'heure du contexte.
fn fait_puis_annule<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
    avant: &str,
    rendu: Reponse<Faite>,
    message: &str,
) {
    rendu.unwrap_or_else(|refus| panic!("le geste « {message} » est refusé : {refus:?}"));
    assert_ne!(
        teneur_du_fonds(services),
        avant,
        "« {message} » laisse le fonds tel qu'il était"
    );
    annule(services, contexte, message);
    assert_eq!(
        teneur_du_fonds(services),
        avant,
        "l'annulation de « {message} » ne rend pas le fonds d'avant le geste"
    );
    let journal = services.journal().expect("le journal se lit");
    assert_eq!(
        journal[journal.len() - 2],
        entree(message, contexte.heure),
        "{journal:?}"
    );
}

/// Une note, sur des services neufs, à l'établi : l'idée née est datée de `HEURE`, `captee`, née
/// sur l'établi, avec son texte, sans captation, ni durée, ni retrait ; rouverte, sa vue est la
/// même.
pub fn une_idee_notee_est_captee_datee_sans_prise_ni_duree<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    assert_eq!(
        services.noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea)
        ),
        Ok(Notee {
            idee: IDEE.to_string(),
            avertissement: None,
        })
    );
    let vue = VueDUneIdee {
        idee: IDEE.to_string(),
        provenance: commandes::Provenance::Captee {
            par: commandes::Appareil::Etabli,
        },
        le: HEURE,
        texte: Some("Un refrain qui monte, en ré".to_string()),
        etiquettes: Vec::new(),
        liens: Vec::new(),
        captations: Vec::new(),
        reliee: false,
        duree_ms: None,
        retiree: None,
    };
    assert_eq!(services.vue_d_une_idee(IDEE), Ok(vue.clone()));

    let services = paire.rouvrir(services, &[]);
    assert_eq!(services.vue_d_une_idee(IDEE), Ok(vue));
}

/// La première note fait naître le fonds : la genèse, la naissance, puis la note, trois
/// modifications au journal ; la racine du document porte l'ULID du document, celui de la lignée
/// et l'identifiant de propriétaire, tirés dans cet ordre, avant l'idée. Une seconde note ajoute
/// une entrée, et son idée prend le tirage suivant.
pub fn noter_fait_naitre_le_fonds_en_trois_modifications<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note est faite");
    let mut journal = vec![
        entree("Genèse du fonds", 0),
        entree("Fonds créé", HEURE),
        entree("Idée du 14 novembre notée", HEURE),
    ];
    assert_eq!(services.journal(), Ok(journal.clone()));
    let fonds = services.documents().fonds().expect("le fonds est né");
    assert_eq!(
        chaine_de_la_racine(fonds, IDENTIFIANT).as_deref(),
        Some(ULID_DU_DOCUMENT)
    );
    assert_eq!(
        chaine_de_la_racine(fonds, LIGNEE).as_deref(),
        Some(ULID_DE_LA_LIGNEE)
    );
    assert_eq!(
        chaine_de_la_racine(fonds, PROPRIETAIRE).as_deref(),
        Some(IDENTIFIANT_DE_PROPRIETAIRE)
    );

    assert_eq!(
        services.noter(&une_note("Un pont en mi mineur"), &mut contexte),
        Ok(Notee {
            idee: SECONDE_IDEE.to_string(),
            avertissement: None,
        })
    );
    journal.push(entree("Idée du 14 novembre notée", HEURE));
    assert_eq!(services.journal(), Ok(journal));
}

/// Une note vide, une fois ses blancs ôtés, est refusée, et rien ne naît : ni fonds, ni journal,
/// ni idée.
pub fn une_note_vide_refusee_ne_fait_rien_naitre<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    assert_eq!(
        services.noter(&une_note(" \t\n "), &mut contexte_de_test(&mut alea)),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Le texte de la note est vide.".to_string(),
        })
    );
    assert!(services.documents().fonds().is_none());
    assert_eq!(services.journal(), Ok(Vec::new()));
    assert_eq!(
        services.vue_d_une_idee(IDEE),
        Err(Refus {
            raison: Raison::Impossible,
            message: format!("Aucune idée ne porte l'identifiant {IDEE}."),
        })
    );
}

/// Une note au décalage local de deux heures, à `HEURE` (22 h 13 UTC, le 14 novembre) : les
/// services passent le décalage au noyau, et le message dit le jour décalé, le 15 novembre ;
/// l'idée née reste datée de `HEURE`.
pub fn une_note_au_decalage_local_dit_le_jour_decale<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let commande = Noter {
        texte: "Un pont en mi mineur".to_string(),
        decalage_minutes: 120,
    };
    services
        .noter(&commande, &mut contexte_de_test(&mut alea))
        .expect("la note est faite");
    let journal = services.journal().expect("le journal se lit");
    assert_eq!(
        journal.last(),
        Some(&entree("Idée du 15 novembre notée", HEURE)),
        "{journal:?}"
    );
    let vue = services
        .vue_d_une_idee(IDEE)
        .expect("la vue de l'idée se lit");
    assert_eq!(vue.le, HEURE);
}

/// Une empreinte qu'aucune captation ne porte ne se relit pas : `Raison::Impossible`, sur des
/// services neufs comme après qu'une autre captation est versée.
pub fn une_empreinte_qu_aucune_captation_ne_porte_ne_se_relit_pas<P: Paire>(paire: &mut P) {
    let services = paire.neufs();
    let inconnue = empreinte("une captation jamais versée".as_bytes());
    let relire = |services: &Services<P::D, P::C>| {
        services
            .captations()
            .relire(&inconnue)
            .map_err(|refus| refus.raison)
    };
    assert_eq!(relire(&services), Err(Raison::Impossible));
    services
        .captations()
        .verser("une autre captation".as_bytes())
        .expect("la captation se verse");
    assert_eq!(relire(&services), Err(Raison::Impossible));
}

// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que la version de schéma N
/// Le fonds migré vers la version de schéma 2, rouvert par une application qui ne connaît que la
/// version de schéma 1 : il se lit, son journal, la liste des idées et la vue de l'idée notée
/// montrés, cette vue entière la même qu'avant la migration ; une note, « annuler » et la fiche
/// d'une prise sont refusés du refus que rend `lecture_seule()`, qui nomme la version de schéma 2,
/// sans rien tirer, et le journal ne change pas ; la captation de la fiche refusée est versée, et
/// une captation se verse encore, et se relit.
pub fn une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    services
        .noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la note est faite");
    let avant = services
        .vue_d_une_idee(IDEE)
        .expect("la vue de l'idée se lit");
    let migration = migration_de_test();
    let services = paire.rouvrir(services, &migrations_de_test(&migration));
    let mut services = paire.rouvrir(services, &[]);

    let journal = services.journal().expect("le journal se lit");
    // La migration, concurrente de la naissance et de la note, est au journal, à côté d'elles.
    for montree in [
        entree("Genèse du fonds", 0),
        entree("Fonds créé", HEURE),
        entree("Idée du 14 novembre notée", HEURE),
        entree(&message_de_la_migration(2), 0),
    ] {
        assert!(
            journal.contains(&montree),
            "{montree:?} manque au journal : {journal:?}"
        );
    }
    assert_eq!(journal.len(), 4, "{journal:?}");
    assert_eq!(services.vue_d_une_idee(IDEE), Ok(avant));
    assert_eq!(services.idees(), Ok(vec![IDEE.to_string()]));

    let refus = services
        .documents()
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(refus.raison, Raison::VersionPlusRecente { version: 2 });
    assert_eq!(
        services.noter(
            &une_note("Un pont en mi mineur"),
            &mut contexte_de_test(&mut alea)
        ),
        Err(refus.clone())
    );
    // Sans dernier geste de tri, « annuler » rend lui aussi le refus de la lecture seule, et non
    // « rien à annuler » : la lecture seule se dit d'abord.
    assert_eq!(
        services.annuler(&mut contexte_de_test(&mut alea)),
        Err(refus.clone())
    );
    let prise: &[u8] = b"une prise devant un fonds plus recent";
    assert_eq!(
        services.verser_une_fiche(
            gestes::Sorte::Prise,
            prise,
            &fiche_wav(30_000),
            &importee(),
            0,
            &mut contexte_de_test(&mut alea),
        ),
        Err(refus)
    );
    // Les quatre tirages de la naissance, et aucun de plus : ni la note ni la fiche refusées
    // n'ont rien tiré.
    assert_eq!(alea.appels, 4);
    assert_eq!(services.journal(), Ok(journal));
    // La fiche est refusée, sa captation versée : « Une captation n'est jamais refusée pour
    // autant » (§ 31 · 2.4).
    assert_eq!(
        services.captations().relire(&empreinte(prise)),
        Ok(prise.to_vec())
    );

    let octets = b"une captation devant un fonds plus recent";
    let empreinte = services
        .captations()
        .verser(octets)
        .expect("la captation se verse");
    assert_eq!(
        services.captations().relire(&empreinte),
        Ok(octets.to_vec())
    );
}

/// Le fonds né d'une note, rouvert selon une migration qui attend ses dépendances
/// (`migration_sur_une_autre_genese`) : il n'a pas migré, et se lit sans écrire, du refus que
/// rend `lecture_seule()`, `Raison::Illisible` ; sa version de schéma reste 1, et son journal et
/// la vue de l'idée notée se lisent. Une note est refusée de ce refus, sans rien tirer : rien ne
/// s'écrit à la version de schéma 1 devant une application qui en connaît une autre, et le journal
/// ne change pas. Rend les services, que chaque paire examine selon ses adaptateurs : la raison de
/// la lecture seule du document sur fichiers, le message du refus en mémoire.
pub fn une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule<P: Paire>(
    paire: &mut P,
) -> Services<P::D, P::C> {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    services
        .noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la note est faite");
    let journal = vec![
        entree("Genèse du fonds", 0),
        entree("Fonds créé", HEURE),
        entree("Idée du 14 novembre notée", HEURE),
    ];
    let migration = migration_sur_une_autre_genese();
    let mut services = paire.rouvrir(services, &migrations_de_test(&migration));

    let refus = services
        .documents()
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(refus.raison, Raison::Illisible);
    let fonds = services.documents().fonds().expect("le fonds se lit");
    assert_eq!(version_de_schema(fonds), Some(1));
    assert_eq!(services.journal(), Ok(journal.clone()));
    let vue = services
        .vue_d_une_idee(IDEE)
        .expect("la vue de l'idée se lit");
    assert_eq!(vue.texte.as_deref(), Some("Un refrain qui monte, en ré"));

    assert_eq!(
        services.noter(
            &une_note("Un pont en mi mineur"),
            &mut contexte_de_test(&mut alea)
        ),
        Err(refus)
    );
    // Les quatre tirages de la naissance, et aucun de plus : la note refusée n'a rien tiré.
    assert_eq!(alea.appels, 4);
    assert_eq!(services.journal(), Ok(journal));
    services
}

/// Sur deux notes A et B et une prise P versée (son idée V), chacun des huit gestes de tri :
/// compléter A, poser `#pont` sur A, l'ôter, relier A et B, les délier, regrouper P dans A, qui
/// retire V, retirer B, rétablir B. Après chacun, le journal porte une entrée de plus, au message
/// du geste, à l'heure du contexte : le lendemain de la naissance des idées, dont le message dit
/// le jour.
pub fn chaque_geste_de_tri_parait_au_journal<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    let b = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite")
        .idee;
    let p = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    )
    .empreinte;
    let mut journal = services.journal().expect("le journal se lit");
    contexte.heure = LENDEMAIN;

    let rendu = services.completer(
        &completion(&a, "Un refrain qui monte, puis retombe"),
        &mut contexte,
    );
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre : texte complété",
    );
    let rendu = services.poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre : étiquette #pont posée",
    );
    let rendu = services.oter_une_etiquette(&etiquetage(&a, "pont"), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre : étiquette #pont ôtée",
    );
    let rendu = services.relier(&lien(&a, &b), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre reliée à l'idée du 14 novembre",
    );
    let rendu = services.delier(&lien(&a, &b), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre déliée de l'idée du 14 novembre",
    );
    let rendu = services.regrouper(&regroupement(&p, &a), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée",
    );
    let rendu = services.retirer(&visee(&b), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre retirée",
    );
    let rendu = services.retablir(&visee(&b), &mut contexte);
    parait_au_journal(
        &services,
        &mut journal,
        rendu,
        "Idée du 14 novembre rétablie",
    );
}

/// X notée, et P1, versée sous son idée W, regroupée dans X ; puis V, née du versement de P2
/// seule. Regrouper P2 dans X retire V par le même geste : le journal gagne une seule entrée, qui
/// le dit ; V, retirée à l'heure du contexte, sans captation, quitte la liste des idées, comme W,
/// vidée par le premier regroupement ; X porte P1 et P2. Annuler ce regroupement, au lendemain,
/// rend P2 à V, rétablie : V revient dans la liste des idées, W non, X ne porte plus que P1, et le
/// journal porte le regroupement puis son annulation, à l'heure du contexte de celle-ci, non à
/// celle du geste ; un second « annuler » est refusé, rien à annuler.
pub fn l_idee_videe_par_un_regroupement_est_retiree<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let x = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note X est faite")
        .idee;
    let p1 = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    )
    .empreinte;
    services
        .regrouper(&regroupement(&p1, &x), &mut contexte)
        .expect("P1 se regroupe dans X");
    let v = verser_une_prise(
        &mut services,
        b"une seconde prise de test",
        &fiche_wav(64_000),
        &mut contexte,
    );
    let mut journal = services.journal().expect("le journal se lit");

    assert_eq!(
        services.regrouper(&regroupement(&v.empreinte, &x), &mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    journal.push(entree(
        "Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée",
        HEURE,
    ));
    assert_eq!(services.journal(), Ok(journal.clone()));
    let vue_de_v = services
        .vue_d_une_idee(&v.idee)
        .expect("la vue de V se lit");
    assert_eq!(vue_de_v.retiree, Some(HEURE));
    assert_eq!(vue_de_v.captations, Vec::new());
    assert_eq!(services.idees(), Ok(vec![x.clone()]));
    assert_eq!(
        services.vue_d_une_idee(&x).map(|vue| vue.captations),
        Ok(prises(&[&p1, &v.empreinte]))
    );

    contexte.heure = LENDEMAIN;
    assert_eq!(
        services.annuler(&mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    journal.push(entree(
        "Annulé : Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée",
        LENDEMAIN,
    ));
    assert_eq!(services.journal(), Ok(journal));
    let vue_de_v = services
        .vue_d_une_idee(&v.idee)
        .expect("la vue de V se lit");
    assert_eq!(vue_de_v.retiree, None);
    assert_eq!(vue_de_v.captations, prises(&[&v.empreinte]));
    assert_eq!(services.idees(), Ok(vec![x.clone(), v.idee.clone()]));
    assert_eq!(
        services.vue_d_une_idee(&x).map(|vue| vue.captations),
        Ok(prises(&[&p1]))
    );
    rien_n_est_annule(&mut services, &mut contexte);
}

/// Deux prises de deux idées, P1 de V1 et P2 de V2, regroupées dans X : X porte deux captations
/// distinctes, d'empreintes celles que le versement a rendues, celles de leurs octets, que les
/// captations relisent.
pub fn deux_prises_regroupees_restent_deux_captations<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let x = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note X est faite")
        .idee;
    let premiere: &[u8] = PRISE_DE_TEST;
    let seconde: &[u8] = b"une seconde prise de test";
    let v1 = verser_une_prise(&mut services, premiere, &fiche_wav(30_000), &mut contexte);
    let v2 = verser_une_prise(&mut services, seconde, &fiche_wav(64_000), &mut contexte);
    assert_eq!(v1.empreinte, empreinte(premiere));
    assert_eq!(v2.empreinte, empreinte(seconde));
    assert_ne!(v1.empreinte, v2.empreinte);
    services
        .regrouper(&regroupement(&v1.empreinte, &x), &mut contexte)
        .expect("P1 se regroupe dans X");
    services
        .regrouper(&regroupement(&v2.empreinte, &x), &mut contexte)
        .expect("P2 se regroupe dans X");

    assert_eq!(
        services.vue_d_une_idee(&x).map(|vue| vue.captations),
        Ok(prises(&[&v1.empreinte, &v2.empreinte]))
    );
    assert_eq!(
        services.captations().relire(&v1.empreinte),
        Ok(premiere.to_vec())
    );
    assert_eq!(
        services.captations().relire(&v2.empreinte),
        Ok(seconde.to_vec())
    );
}

/// V, née d'une prise, complétée d'un texte, étiquetée et reliée à deux idées : retirée, elle
/// quitte la liste des idées, et sa vue, qui garde tout, porte la date de son retrait ;
/// rétablie, elle revient dans la liste, et sa vue est celle d'avant le retrait, entière : texte,
/// étiquettes, liens, dans leur ordre, et captations, d'empreintes inchangées, que les captations
/// relisent.
pub fn une_idee_retiree_quitte_la_liste_et_se_retablit_telle_qu_elle_etait<P: Paire>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    // Deux ULID, d'une même longueur, rangent les clés des liens d'une idée comme leurs autres
    // bouts : une clé est `apparente:<p>:<q>`, ses bouts dans l'ordre de leurs octets, et ne se
    // départage que sur l'autre bout. Seul un identifiant d'une autre longueur sépare les deux
    // ordres : A0, A suivi d'un « 0 », qu'aucun appareil ne forme, et que le test écrit donc par
    // une opération du noyau, sans les services. V, tirée après A, vient après A et A0 : ses clés
    // se rangent A0 puis A (« 0 » passe avant « : »), quand ses autres bouts se rangent A puis
    // A0 (A est un préfixe de A0). Une vue qui ne rangerait pas les autres bouts rendrait l'ordre
    // des clés.
    let a0 = format!("{a}0");
    let note_a0 = gestes::noter(
        &a0,
        "Un pont en mi mineur",
        gestes::Appareil::Etabli,
        HEURE,
        0,
    )
    .expect("la note A0 se calcule");
    services
        .documents_mut()
        .appliquer(&note_a0)
        .expect("la note A0 s'écrit");
    let v = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    );
    let cles: BTreeSet<String> = [cle_du_lien(&a, &v.idee), cle_du_lien(&a0, &v.idee)].into();
    assert_eq!(
        cles.into_iter().collect::<Vec<_>>(),
        vec![cle_du_lien(&a0, &v.idee), cle_du_lien(&a, &v.idee)]
    );
    services
        .completer(
            &completion(&v.idee, "Une basse qui descend sous la prise"),
            &mut contexte,
        )
        .expect("le texte de V est complété");
    services
        .poser_une_etiquette(&etiquetage(&v.idee, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur V");
    services
        .relier(&lien(&v.idee, &a), &mut contexte)
        .expect("V est reliée à A");
    services
        .relier(&lien(&v.idee, &a0), &mut contexte)
        .expect("V est reliée à A0");

    let telle_qu_elle_etait = VueDUneIdee {
        idee: v.idee.clone(),
        provenance: commandes::Provenance::Importee {
            par: commandes::Appareil::Etabli,
            depuis: IMPORT.to_string(),
        },
        le: HEURE,
        texte: Some("Une basse qui descend sous la prise".to_string()),
        etiquettes: vec!["pont".to_string()],
        liens: vec![a.clone(), a0.clone()],
        captations: prises(&[&v.empreinte]),
        reliee: true,
        duree_ms: Some(30_000),
        retiree: None,
    };
    assert_eq!(
        services.vue_d_une_idee(&v.idee),
        Ok(telle_qu_elle_etait.clone())
    );
    assert_eq!(
        services.idees(),
        Ok(vec![a.clone(), a0.clone(), v.idee.clone()])
    );

    services
        .retirer(&visee(&v.idee), &mut contexte)
        .expect("V est retirée");
    assert_eq!(services.idees(), Ok(vec![a.clone(), a0.clone()]));
    assert_eq!(
        services.vue_d_une_idee(&v.idee),
        Ok(VueDUneIdee {
            retiree: Some(HEURE),
            ..telle_qu_elle_etait.clone()
        })
    );

    services
        .retablir(&visee(&v.idee), &mut contexte)
        .expect("V est rétablie");
    assert_eq!(services.idees(), Ok(vec![a, a0, v.idee.clone()]));
    assert_eq!(services.vue_d_une_idee(&v.idee), Ok(telle_qu_elle_etait));
    assert_eq!(
        services.captations().relire(&v.empreinte),
        Ok(PRISE_DE_TEST.to_vec())
    );
}

/// Sur des services neufs, sans fonds, chacun des huit gestes de tri est refusé : aucune idée ne
/// porte l'identifiant que vise la commande, le premier qu'elle nomme, ou la cible d'un
/// regroupement. Rien ne naît, ni fonds, ni journal, ni idée, et rien n'est tiré : seuls les
/// gestes qui créent une idée, noter et verser une fiche, font naître le fonds.
pub fn un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let absente = Err(impossible(format!(
        "Aucune idée ne porte l'identifiant {IDEE}."
    )));
    assert_eq!(
        services.completer(
            &completion(IDEE, "Un refrain qui monte, en ré"),
            &mut contexte
        ),
        absente
    );
    assert_eq!(
        services.poser_une_etiquette(&etiquetage(IDEE, "pont"), &mut contexte),
        absente
    );
    assert_eq!(
        services.oter_une_etiquette(&etiquetage(IDEE, "pont"), &mut contexte),
        absente
    );
    assert_eq!(
        services.relier(&lien(IDEE, SECONDE_IDEE), &mut contexte),
        absente
    );
    assert_eq!(
        services.delier(&lien(IDEE, SECONDE_IDEE), &mut contexte),
        absente
    );
    assert_eq!(
        services.regrouper(
            &regroupement(&empreinte(PRISE_DE_TEST), IDEE),
            &mut contexte
        ),
        absente
    );
    assert_eq!(services.retirer(&visee(IDEE), &mut contexte), absente);
    assert_eq!(services.retablir(&visee(IDEE), &mut contexte), absente);

    assert!(services.documents().fonds().is_none());
    assert_eq!(services.journal(), Ok(Vec::new()));
    assert_eq!(services.idees(), Ok(Vec::new()));
    assert_eq!(alea.appels, 0);
}

/// Une fiche versée deux fois : la seconde est refusée, la captation déjà versée, et ni le
/// journal ni la liste des idées ne changent. La captation, elle, se verse de nouveau sans
/// erreur, sous la même empreinte, et se relit : rien ne change une captation, ni sa fiche (I14 :
/// les captations sont immuables).
pub fn une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let versee = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    );
    let journal = services.journal().expect("le journal se lit");
    let idees = services.idees().expect("la liste des idées se lit");

    assert_eq!(
        services.verser_une_fiche(
            gestes::Sorte::Prise,
            PRISE_DE_TEST,
            &fiche_wav(30_000),
            &importee(),
            0,
            &mut contexte,
        ),
        Err(impossible(format!(
            "La captation {} est déjà versée.",
            versee.empreinte
        )))
    );
    assert_eq!(services.journal(), Ok(journal));
    assert_eq!(services.idees(), Ok(idees));
    assert_eq!(
        services.captations().verser(PRISE_DE_TEST),
        Ok(versee.empreinte.clone())
    );
    assert_eq!(
        services.captations().relire(&versee.empreinte),
        Ok(PRISE_DE_TEST.to_vec())
    );
}

/// R, retirée : poser une étiquette sur elle, la relier depuis l'autre bout, y regrouper une
/// prise, chacun est refusé, l'idée retirée, et le journal ne change pas.
pub fn un_geste_qui_touche_une_idee_retiree_est_refuse<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    let r = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note R est faite")
        .idee;
    let v = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    );
    services
        .retirer(&visee(&r), &mut contexte)
        .expect("R est retirée");
    let journal = services.journal().expect("le journal se lit");

    let retiree = Err(impossible(format!("L'idée {r} est retirée.")));
    assert_eq!(
        services.poser_une_etiquette(&etiquetage(&r, "pont"), &mut contexte),
        retiree
    );
    assert_eq!(services.relier(&lien(&a, &r), &mut contexte), retiree);
    assert_eq!(
        services.regrouper(&regroupement(&v.empreinte, &r), &mut contexte),
        retiree
    );
    assert_eq!(services.journal(), Ok(journal));
}

/// A et B reliées : A est reliée ; B retirée, A ne l'est plus ; B rétablie, elle l'est de
/// nouveau. V1, née d'une prise de 30 000 ms, dure 30 000 ms ; une prise de 64 000 ms regroupée
/// dans V1 la porte à 64 000 ms ; une prise sans durée lue, en FLAC, regroupée dans V1, lui ôte sa
/// durée (CAP-02, Règles : `—` dès qu'une de ses prises n'a pas la sienne) : son versement n'a
/// écrit aucune durée.
pub fn la_vue_tire_reliee_et_la_duree_des_regles_du_noyau<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    let b = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite")
        .idee;
    let reliee =
        |services: &Services<P::D, P::C>| services.vue_d_une_idee(&a).map(|vue| vue.reliee);
    services
        .relier(&lien(&a, &b), &mut contexte)
        .expect("A est reliée à B");
    assert_eq!(reliee(&services), Ok(true));
    services
        .retirer(&visee(&b), &mut contexte)
        .expect("B est retirée");
    assert_eq!(reliee(&services), Ok(false));
    services
        .retablir(&visee(&b), &mut contexte)
        .expect("B est rétablie");
    assert_eq!(reliee(&services), Ok(true));

    let v1 = verser_une_prise(
        &mut services,
        b"une prise de trente secondes",
        &fiche_wav(30_000),
        &mut contexte,
    );
    let duree =
        |services: &Services<P::D, P::C>| services.vue_d_une_idee(&v1.idee).map(|vue| vue.duree_ms);
    assert_eq!(duree(&services), Ok(Some(30_000)));
    let v2 = verser_une_prise(
        &mut services,
        b"une prise de soixante-quatre secondes",
        &fiche_wav(64_000),
        &mut contexte,
    );
    services
        .regrouper(&regroupement(&v2.empreinte, &v1.idee), &mut contexte)
        .expect("la prise de V2 se regroupe dans V1");
    assert_eq!(duree(&services), Ok(Some(64_000)));
    let sans_duree_lue = gestes::Fiche {
        format: "flac".to_string(),
        duree_ms: None,
    };
    let v3 = verser_une_prise(
        &mut services,
        b"une prise sans duree lue",
        &sans_duree_lue,
        &mut contexte,
    );
    services
        .regrouper(&regroupement(&v3.empreinte, &v1.idee), &mut contexte)
        .expect("la prise de V3 se regroupe dans V1");
    assert_eq!(duree(&services), Ok(None));
}

/// Sur deux notes A et B et une prise P versée (son idée V), chacun des huit gestes de tri, fait
/// puis annulé, au lendemain : compléter A, poser `#pont` sur A, l'ôter, relier A et B, les
/// délier, regrouper P dans A, qui retire V, retirer B, rétablir B. Chaque annulation rend au
/// fonds la teneur d'avant son geste, et le journal porte le geste, puis « Annulé : » suivi de
/// son message. Ôter, délier et rétablir demandent l'étiquette, le lien ou le retrait : un geste
/// de tri les pose d'abord, et reste.
pub fn annuler_rend_le_fonds_d_avant<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    let b = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite")
        .idee;
    let p = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    )
    .empreinte;
    contexte.heure = LENDEMAIN;

    let avant = teneur_du_fonds(&services);
    let rendu = services.completer(
        &completion(&a, "Un refrain qui monte, puis retombe"),
        &mut contexte,
    );
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre : texte complété",
    );

    let avant = teneur_du_fonds(&services);
    let rendu = services.poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre : étiquette #pont posée",
    );

    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    let avant = teneur_du_fonds(&services);
    let rendu = services.oter_une_etiquette(&etiquetage(&a, "pont"), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre : étiquette #pont ôtée",
    );

    let avant = teneur_du_fonds(&services);
    let rendu = services.relier(&lien(&a, &b), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre reliée à l'idée du 14 novembre",
    );

    services
        .relier(&lien(&a, &b), &mut contexte)
        .expect("A est reliée à B");
    let avant = teneur_du_fonds(&services);
    let rendu = services.delier(&lien(&a, &b), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre déliée de l'idée du 14 novembre",
    );

    let avant = teneur_du_fonds(&services);
    let rendu = services.regrouper(&regroupement(&p, &a), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre : prise regroupée, idée du 14 novembre retirée",
    );

    let avant = teneur_du_fonds(&services);
    let rendu = services.retirer(&visee(&b), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre retirée",
    );

    services
        .retirer(&visee(&b), &mut contexte)
        .expect("B est retirée");
    let avant = teneur_du_fonds(&services);
    let rendu = services.retablir(&visee(&b), &mut contexte);
    fait_puis_annule(
        &mut services,
        &mut contexte,
        &avant,
        rendu,
        "Idée du 14 novembre rétablie",
    );
}

/// Annuler ne vise que le dernier geste de tri fait depuis l'ouverture (§ 30 · 11) : une note ne
/// le vise ni ne le remplace, un geste refusé ne le change pas, et une annulation ne s'annule
/// pas, un second « annuler » est refusé. Pas à pas : noter A ; annuler, refusé ; poser `#pont`
/// sur A ; noter B ; annuler, qui ôte `#pont` ; annuler, refusé ; relier A et B ; retirer B ;
/// poser `#pont` sur A ; le reposer, refusé ; annuler, qui ôte `#pont` ; annuler, refusé. Restent
/// le lien de A et B, B retirée, et A sans étiquette.
pub fn annuler_ne_vise_que_le_dernier_geste_de_tri_depuis_l_ouverture<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    rien_n_est_annule(&mut services, &mut contexte);
    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    let b = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite")
        .idee;
    annule(
        &mut services,
        &mut contexte,
        "Idée du 14 novembre : étiquette #pont posée",
    );
    rien_n_est_annule(&mut services, &mut contexte);
    services
        .relier(&lien(&a, &b), &mut contexte)
        .expect("A est reliée à B");
    services
        .retirer(&visee(&b), &mut contexte)
        .expect("B est retirée");
    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    assert_eq!(
        services.poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte),
        Err(impossible("L'étiquette #pont est déjà posée.".to_string()))
    );
    annule(
        &mut services,
        &mut contexte,
        "Idée du 14 novembre : étiquette #pont posée",
    );
    rien_n_est_annule(&mut services, &mut contexte);

    let vue_de_a = services.vue_d_une_idee(&a).expect("la vue de A se lit");
    assert_eq!(vue_de_a.liens, vec![b.clone()]);
    assert_eq!(vue_de_a.etiquettes, Vec::<String>::new());
    assert_eq!(
        services.vue_d_une_idee(&b).map(|vue| vue.retiree),
        Ok(Some(HEURE))
    );
}

/// Un geste de tri refusé ne devient pas ce qu'annuler vise, et ne change pas ce qu'il vise ; un
/// geste fait le devient, à la place du précédent. Sur A notée : relier A à elle-même, refusé,
/// puis annuler, refusé ; poser `#pont` sur A, puis compléter A ; reposer `#pont`, refusé ;
/// annuler défait le seul complément : A garde `#pont` et retrouve son texte ; un second
/// « annuler » est refusé.
pub fn un_geste_refuse_ne_change_pas_ce_qu_annuler_vise<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    assert_eq!(
        services.relier(&lien(&a, &a), &mut contexte),
        Err(impossible(
            "Une idée ne se relie pas à elle-même.".to_string()
        ))
    );
    rien_n_est_annule(&mut services, &mut contexte);

    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    services
        .completer(
            &completion(&a, "Un refrain qui monte, puis retombe"),
            &mut contexte,
        )
        .expect("le texte de A est complété");
    assert_eq!(
        services.poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte),
        Err(impossible("L'étiquette #pont est déjà posée.".to_string()))
    );
    annule(
        &mut services,
        &mut contexte,
        "Idée du 14 novembre : texte complété",
    );
    let vue_de_a = services.vue_d_une_idee(&a).expect("la vue de A se lit");
    assert_eq!(
        vue_de_a.texte.as_deref(),
        Some("Un refrain qui monte, en ré")
    );
    assert_eq!(vue_de_a.etiquettes, vec!["pont".to_string()]);
    rien_n_est_annule(&mut services, &mut contexte);
}

/// Annuler ne vise que ce qui est fait depuis l'ouverture : `#pont` posée sur A, puis les
/// services rouverts, « annuler » est refusé, rien à annuler ; A garde `#pont`.
pub fn la_reouverture_vide_ce_qu_annuler_vise<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");

    let mut services = paire.rouvrir(services, &[]);
    rien_n_est_annule(&mut services, &mut contexte);
    assert_eq!(
        services.vue_d_une_idee(&a).map(|vue| vue.etiquettes),
        Ok(vec!["pont".to_string()])
    );
}

/// Une annulation ne s'annule pas : `#pont` posée sur A, puis ôtée par « annuler » ; un second
/// « annuler » est refusé, rien à annuler, et A reste sans étiquette.
pub fn une_annulation_ne_s_annule_pas<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    annule(
        &mut services,
        &mut contexte,
        "Idée du 14 novembre : étiquette #pont posée",
    );
    rien_n_est_annule(&mut services, &mut contexte);
    assert_eq!(
        services.vue_d_une_idee(&a).map(|vue| vue.etiquettes),
        Ok(Vec::new())
    );
}
