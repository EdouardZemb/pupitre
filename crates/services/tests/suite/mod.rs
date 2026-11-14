//! La suite des commandes : des tests écrits une fois, génériques sur une paire d'adaptateurs
//! (`Paire`), que rejouent la paire en mémoire (`crates/services/tests/commandes_en_memoire.rs`)
//! et celle des fichiers, au stockage (`crates/stockage/tests/commandes_sur_fichiers.rs`) ;
//! l'heure et l'aléa des tests, l'identifiant de l'import des fiches qu'ils versent, la migration
//! de test, les migrations qui la portent, et une migration qui attend ses dépendances. Le
//! fichier s'inclut par `mod suite;` dans les tests des services, et par
//! `#[path = "../../services/tests/suite/mod.rs"]` dans ceux du stockage : il ne nomme que ce que
//! les deux voient, les services, les commandes, le noyau et `automerge`.

// Chaque fichier qui inclut la suite n'en emploie qu'une partie.
#![allow(dead_code)]

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{ActorId, Automerge, Change, ObjType, ROOT, ReadDoc};
use commandes::{EntreeDuJournal, Notee, Noter, Raison, Refus, VueDUneIdee};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, GENESE, IDENTIFIANT, LIGNEE, PROPRIETAIRE, VERSION_DE_SCHEMA,
};
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::version::{Migration, message_de_la_migration};
use services::lecture::version_de_schema;
use services::{Alea, Captations, Contexte, Documents, Services};

/// L'heure des tests, en secondes : le 14 novembre 2023, 22 h 13 min 20 s UTC.
pub const HEURE: i64 = 1_700_000_000;

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
/// version de schéma 1 : il se lit, son journal et la vue de l'idée notée montrés, cette vue
/// entière la même qu'avant la migration ; une note est refusée du refus que rend
/// `lecture_seule()`, qui nomme la version de schéma 2, sans rien tirer, et le journal ne change
/// pas ; une captation se verse encore, et se relit.
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
        Err(refus)
    );
    // Les quatre tirages de la naissance, et aucun de plus : la note refusée n'a rien tiré.
    assert_eq!(alea.appels, 4);
    assert_eq!(services.journal(), Ok(journal));

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
/// ne change pas.
pub fn une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule<P: Paire>(
    paire: &mut P,
) {
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
}
