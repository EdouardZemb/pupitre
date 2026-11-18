//! La suite des commandes : des tests écrits une fois, génériques sur une paire d'adaptateurs
//! (`Paire`), que rejouent la paire en mémoire (`crates/services/tests/commandes_en_memoire.rs`)
//! et celle des fichiers, au stockage (`crates/stockage/tests/commandes_sur_fichiers.rs`) ;
//! l'heure et l'aléa des tests, l'identifiant de l'import des fiches qu'ils versent, la prise de
//! test, les commandes des gestes de tri, les gestes du témoin du test 11 (§ 31 · 10), que
//! joue le test des empreintes de référence (`crates/stockage/tests/references.rs`), la
//! migration de test, les migrations qui la portent, une migration qui attend ses
//! dépendances, le fonds de chaque geste, et les aides qui refusent ce qu'I28, I01 et les types
//! interdisent (`une_modification`, `message_connu`, `parcourir`, `parcourir_les_types`) ; la
//! boîte entière, l'assiette du noyau qui répond à celle des commandes, le texte du compte, celui
//! de la réponse d'une requête, le jeu de requêtes d'I09 et d'I13, un document dont une idée est
//! illisible, les commandes des requêtes enregistrées et la requête entière. Le fichier s'inclut par `mod suite;` dans les tests des
//! services, et par
//! `#[path = "../../services/tests/suite/mod.rs"]` dans ceux du stockage : il ne nomme que ce
//! que les deux voient, les services, les commandes, le noyau et `automerge`.

// Chaque fichier qui inclut la suite n'en emploie qu'une partie.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{
    ActorId, Automerge, Change, ChangeHash, ObjId, ObjType, Prop, ROOT, ReadDoc, ScalarValue, Value,
};
use commandes::{
    Assiette, Boite, Bornes, Captation, Completer, Contenu, Enregistrer, EntreeDuJournal, Etat,
    EtatDuFonds, Etiqueter, FENETRE_MAX, Faite, Fenetre, Jour, LigneDeLaBoite, Notee, Noter, Ordre,
    ProvenanceDeLIdee, Raison, Refus, Regrouper, Relier, Renommer, Reponse, Requete,
    RequeteEnregistree, Sans, Supprimer, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees,
};
use noyau::donnees::boite::texte_du_compte;
use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, APPARENTE, AU, CONTENEURS_DE_LA_GENESE, CONTENU, DEPUIS, DU, DUREE, ETAT,
    ETIQUETTE, ETIQUETTES, FORMAT, GENESE, IDEE as IDEE_D_UNE_CAPTATION, IDEES, IDENTIFIANT, LE,
    LIENS, LIGNEE, MESSAGE_DE_LA_GENESE, MESSAGE_DE_LA_NAISSANCE, MIDI_VERSES, NOM, PAR, PHOTOS,
    PLUS_DE_SECONDES, PRISES, PROPRIETAIRE, PROVENANCE, REQUETES_ENREGISTREES, RETIREE, TEXTE,
    TEXTES_VERSES, VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes;
use noyau::donnees::idee::cle_du_lien;
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::requete::texte_de_la_reponse;
use noyau::donnees::version::{Migration, message_de_la_migration};
use services::application;
use services::lecture::{lire_l_idee, teneur, version_de_schema};
use services::{Alea, Captations, Contexte, Documents, Services, Versee};

/// L'heure des tests, en secondes : le 14 novembre 2023, 22 h 13 min 20 s UTC.
pub const HEURE: i64 = 1_700_000_000;

/// Le lendemain de `HEURE`, en secondes : le 15 novembre 2023, 22 h 13 min 20 s UTC.
pub const LENDEMAIN: i64 = HEURE + 86_400;

/// Les octets de la prise de test.
pub const PRISE_DE_TEST: &[u8] = b"une prise de test";

/// Les octets de la prise que verse le témoin (`temoin`).
pub const PRISE_DU_TEMOIN: &[u8] = b"une prise du temoin";

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

/// La commande qui enregistre `requete` sous le nom `nom`.
pub fn enregistrement(nom: &str, requete: Requete) -> Enregistrer {
    Enregistrer {
        nom: nom.to_string(),
        requete,
    }
}

/// La commande qui renomme `nom` la requête enregistrée `requete`, un ULID.
pub fn renommage(requete: &str, nom: &str) -> Renommer {
    Renommer {
        requete: requete.to_string(),
        nom: nom.to_string(),
    }
}

/// La commande qui supprime la requête enregistrée `requete`, un ULID.
pub fn suppression(requete: &str) -> Supprimer {
    Supprimer {
        requete: requete.to_string(),
    }
}

/// La requête dont chaque critère est posé : le texte « église », du 1er au 30 novembre 2023,
/// l'étiquette `pont`, plus d'une minute, les reliées, les captées, le texte de l'idée.
pub fn requete_entiere() -> Requete {
    Requete {
        texte: "église".to_string(),
        du: Some(Jour {
            annee: 2023,
            mois: 11,
            jour: 1,
        }),
        au: Some(Jour {
            annee: 2023,
            mois: 11,
            jour: 30,
        }),
        etiquette: Some("pont".to_string()),
        plus_de_secondes: Some(60),
        etat: Some(Etat::Reliee),
        provenance: Some(ProvenanceDeLIdee::Captee),
        contenu: Some(Contenu::TexteDeLIdee),
    }
}

/// Les requêtes enregistrées des services, qui se lisent en lecture seule comme ailleurs.
fn les_requetes<D: Documents, C: Captations>(services: &Services<D, C>) -> Vec<RequeteEnregistree> {
    services
        .requetes_enregistrees()
        .expect("les requêtes enregistrées se lisent")
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

/// Les gestes du témoin du test 11 (§ 31 · 10), écrits une seule fois (`temoin_suivi`, que le
/// témoin joue ici sans suivi), à l'heure et sous l'aléa de `contexte`, au décalage local nul,
/// chacun fait sans avertissement : noter « Un refrain qui monte, en ré », A, qui fait naître le
/// fonds s'il n'est pas né ; poser `refrain` sur A ; noter « Un pont en mi mineur », B ; relier A
/// et B ; verser une prise WAV de 30 000 ms (`PRISE_DU_TEMOIN`), un MIDI `smf1`, puis un texte
/// `brut`, chacun importé (`importee`), sous l'idée qui naît avec lui ; regrouper le MIDI dans A,
/// ce qui retire l'idée née avec lui.
pub fn temoin<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
) {
    temoin_suivi(services, contexte, &mut |_, _, _| {});
}

/// Ce que reçoit le suivi des commandes, après chacune (`suivre`) : le fonds, ses heads d'avant
/// la commande, aucune avant celle qui le fait naître, et le message attendu, celui que doit rendre
/// le noyau.
pub type Suivi<'s> = dyn FnMut(&Automerge, &[ChangeHash], &str) + 's;

/// Fait `commande` sur `services`, puis montre à `suivi` le fonds, qui doit être né, ses heads
/// d'avant la commande, et `message`, le message attendu, celui que doit rendre le noyau. Rend ce
/// que rend la commande.
fn suivre<D: Documents, C: Captations, R>(
    services: &mut Services<D, C>,
    suivi: &mut Suivi<'_>,
    message: &str,
    commande: impl FnOnce(&mut Services<D, C>) -> R,
) -> R {
    let avant = services
        .documents()
        .fonds()
        .map(Automerge::get_heads)
        .unwrap_or_default();
    let rendu = commande(services);
    suivi(
        services.documents().fonds().expect("le fonds est né"),
        avant.as_slice(),
        message,
    );
    rendu
}

/// `suivre`, pour un geste de tri ou une annulation, qui doit être fait sans avertissement.
fn faire_sans_avertissement<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    suivi: &mut Suivi<'_>,
    message: &str,
    commande: impl FnOnce(&mut Services<D, C>) -> Reponse<Faite>,
) {
    let rendu = suivre(services, suivi, message, commande);
    assert_eq!(
        rendu,
        Ok(Faite {
            avertissement: None
        }),
        "« {message} »"
    );
}

/// Les gestes du témoin (`temoin`), chacun montré à `suivi` (`suivre`), avec le message que rend
/// le noyau : chaque idée du témoin naît à l'heure du contexte, dont ce message dit le jour.
fn temoin_suivi<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
    suivi: &mut Suivi<'_>,
) {
    let jour = gestes::date_du_message(contexte.heure, 0);
    let a = suivre(
        services,
        suivi,
        &format!("Idée du {jour} notée"),
        |services| services.noter(&une_note("Un refrain qui monte, en ré"), contexte),
    )
    .expect("le témoin note A");
    assert_eq!(a.avertissement, None, "la note A du témoin");
    faire_sans_avertissement(
        services,
        suivi,
        &format!("Idée du {jour} : étiquette #refrain posée"),
        |services| services.poser_une_etiquette(&etiquetage(&a.idee, "refrain"), contexte),
    );
    let b = suivre(
        services,
        suivi,
        &format!("Idée du {jour} notée"),
        |services| services.noter(&une_note("Un pont en mi mineur"), contexte),
    )
    .expect("le témoin note B");
    assert_eq!(b.avertissement, None, "la note B du témoin");
    faire_sans_avertissement(
        services,
        suivi,
        &format!("Idée du {jour} reliée à l'idée du {jour}"),
        |services| services.relier(&lien(&a.idee, &b.idee), contexte),
    );
    suivre(
        services,
        suivi,
        &format!("Idée du {jour} : prise versée"),
        |services| {
            verser_sans_avertissement(
                services,
                gestes::Sorte::Prise,
                PRISE_DU_TEMOIN,
                &fiche_wav(30_000),
                contexte,
            )
        },
    );
    let midi = suivre(
        services,
        suivi,
        &format!("Idée du {jour} : MIDI versé"),
        |services| {
            verser_sans_avertissement(
                services,
                gestes::Sorte::MidiVerseSeul,
                b"un midi du temoin",
                &gestes::Fiche {
                    format: "smf1".to_string(),
                    duree_ms: None,
                },
                contexte,
            )
        },
    );
    suivre(
        services,
        suivi,
        &format!("Idée du {jour} : texte versé"),
        |services| {
            verser_sans_avertissement(
                services,
                gestes::Sorte::TexteVerse,
                b"un texte du temoin",
                &gestes::Fiche {
                    format: "brut".to_string(),
                    duree_ms: None,
                },
                contexte,
            )
        },
    );
    let regroupement_du_midi = Regrouper {
        captation: Captation {
            sorte: commandes::Sorte::MidiVerseSeul,
            empreinte: midi,
        },
        cible: a.idee,
        decalage_minutes: 0,
    };
    faire_sans_avertissement(
        services,
        suivi,
        &format!("Idée du {jour} : MIDI versé regroupé, idée du {jour} retirée"),
        |services| services.regrouper(&regroupement_du_midi, contexte),
    );
}

/// Le fonds où chaque geste est fait une fois, sur des services neufs de `paire`, sous
/// `AleaDeTest { graine: 0, appels: 0 }` : les gestes du témoin (`temoin`), à `HEURE` ; puis, au
/// lendemain (`LENDEMAIN`), sur les idées A et B qu'il note (`IDEE` et `SECONDE_IDEE`) et sa
/// prise (`PRISE_DU_TEMOIN`) : compléter A, poser `#pont` sur A, l'ôter, délier A et B, que le
/// témoin a reliées, les relier de nouveau, regrouper la prise dans A, ce qui retire l'idée née
/// avec elle, retirer B, rétablir B ; puis annuler ce rétablissement. Chaque commande, faite
/// sans avertissement, se montre à `suivi` (`suivre`). Rend les services.
pub fn le_fonds_de_chaque_geste<P: Paire>(
    paire: &mut P,
    suivi: &mut Suivi<'_>,
) -> Services<P::D, P::C> {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    temoin_suivi(&mut services, &mut contexte, suivi);
    contexte.heure = LENDEMAIN;
    let jour = gestes::date_du_message(HEURE, 0);
    let prise = empreinte(PRISE_DU_TEMOIN);
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} : texte complété"),
        |services| {
            services.completer(
                &completion(IDEE, "Un refrain qui monte, puis retombe"),
                &mut contexte,
            )
        },
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} : étiquette #pont posée"),
        |services| services.poser_une_etiquette(&etiquetage(IDEE, "pont"), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} : étiquette #pont ôtée"),
        |services| services.oter_une_etiquette(&etiquetage(IDEE, "pont"), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} déliée de l'idée du {jour}"),
        |services| services.delier(&lien(IDEE, SECONDE_IDEE), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} reliée à l'idée du {jour}"),
        |services| services.relier(&lien(IDEE, SECONDE_IDEE), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} : prise regroupée, idée du {jour} retirée"),
        |services| services.regrouper(&regroupement(&prise, IDEE), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} retirée"),
        |services| services.retirer(&visee(SECONDE_IDEE), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Idée du {jour} rétablie"),
        |services| services.retablir(&visee(SECONDE_IDEE), &mut contexte),
    );
    faire_sans_avertissement(
        &mut services,
        suivi,
        &format!("Annulé : Idée du {jour} rétablie"),
        |services| services.annuler(&mut contexte),
    );
    services
}

/// Un document de test, né par `application::faire_naitre` : la naissance du document, à
/// `HEURE`, puis la note de l'idée `IDEE`. Les plantations des tests s'y posent à la main.
fn un_document_de_test() -> Automerge {
    let naissance = gestes::naissance(
        ULID_DU_DOCUMENT,
        ULID_DE_LA_LIGNEE,
        IDENTIFIANT_DE_PROPRIETAIRE,
        HEURE,
    );
    let note = gestes::noter(
        IDEE,
        "Un refrain qui monte, en ré",
        gestes::Appareil::Etabli,
        HEURE,
        0,
    )
    .expect("la note du document de test se calcule");
    application::faire_naitre(&naissance, &note).expect("le document de test naît")
}

/// L'objet à clés au bout de `chemin`, depuis la racine du document, chaque pas un objet à clés.
fn objet_au_bout(doc: &Automerge, chemin: &[&str]) -> ObjId {
    let mut objet = ROOT;
    for cle in chemin {
        objet = match doc.get(&objet, *cle) {
            Ok(Some((Value::Object(ObjType::Map), id))) => id,
            autre => panic!("{cle} n'est pas un objet à clés : {autre:?}"),
        };
    }
    objet
}

/// Les messages des modifications que `doc` porte depuis `heads`, dans l'ordre de
/// `get_changes` ; une modification sans message en a un vide.
fn messages_depuis(doc: &Automerge, heads: &[ChangeHash]) -> Vec<String> {
    doc.get_changes(heads)
        .iter()
        .map(|modification| modification.message().unwrap_or_default().to_string())
        .collect()
}

/// I28 (§ 70 · 3) : depuis `heads_d_avant`, `doc` porte une seule modification, de message
/// `message` ; sinon, le refus nomme le message de chaque modification qu'il porte depuis.
pub fn une_modification(
    doc: &Automerge,
    heads_d_avant: &[ChangeHash],
    message: &str,
) -> Result<(), String> {
    let messages = messages_depuis(doc, heads_d_avant);
    if matches!(messages.as_slice(), [seul] if seul == message) {
        Ok(())
    } else {
        Err(format!(
            "attendu une seule modification, « {message} », depuis les heads d'avant ; \
             écrites : {messages:?}"
        ))
    }
}

/// Les douze mois que nomme une date de `gestes::date_du_message`.
const MOIS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// Ce qui suit la date qui ouvre `texte`, une date de `gestes::date_du_message` : « 1er », ou un
/// jour de 2 à 31, écrit sans zéro ni signe devant, puis une espace et l'un des douze mois ;
/// `None` si `texte` ne s'ouvre pas sur une date.
fn apres_une_date(texte: &str) -> Option<&str> {
    let (jour, reste) = texte.split_once(' ')?;
    let jour_du_mois = jour
        .parse::<u8>()
        .is_ok_and(|n| (2..=31).contains(&n) && n.to_string() == jour);
    if jour != "1er" && !jour_du_mois {
        return None;
    }
    MOIS.into_iter().find_map(|mois| reste.strip_prefix(mois))
}

/// `message` est celui d'une migration, `message_de_la_migration(v)`, vers une version de schéma
/// `v` de 2 au moins : la genèse porte la version de schéma 1.
fn est_une_migration(message: &str) -> bool {
    message
        .rsplit_once(' ')
        .and_then(|(_, version)| version.parse::<u64>().ok())
        .is_some_and(|version| version >= 2 && message_de_la_migration(version) == message)
}

/// `message` est celui d'une note ou d'un versement : `Idée du <date> notée`, ou
/// `Idée du <date> : ` suivi de `prise versée`, `MIDI versé` ou `texte versé`.
fn est_une_note_ou_un_versement(message: &str) -> bool {
    message
        .strip_prefix("Idée du ")
        .and_then(apres_une_date)
        .is_some_and(|reste| {
            matches!(
                reste,
                " notée" | " : prise versée" | " : MIDI versé" | " : texte versé"
            )
        })
}

/// `message` est celui d'un geste de tri, `Idée du <date>` suivi de l'une de ses formes
/// fermées : ` : texte complété` ; ` : étiquette #<e> posée` ou ` : étiquette #<e> ôtée`,
/// l'étiquette non vide ; ` reliée à l'idée du <date>` ou ` déliée de l'idée du <date>` ;
/// ` : prise regroupée`, ` : MIDI versé regroupé` ou ` : texte versé regroupé`, suivi au besoin
/// de `, idée du <date> retirée` ; ` retirée` ou ` rétablie`. Aucune ne finit par une suite
/// libre.
fn est_un_geste_de_tri(message: &str) -> bool {
    let Some(reste) = message.strip_prefix("Idée du ").and_then(apres_une_date) else {
        return false;
    };
    if matches!(reste, " : texte complété" | " retirée" | " rétablie") {
        return true;
    }
    if let Some(etiquette) = reste.strip_prefix(" : étiquette #") {
        return etiquette
            .strip_suffix(" posée")
            .or_else(|| etiquette.strip_suffix(" ôtée"))
            .is_some_and(|nom| !nom.is_empty());
    }
    if let Some(autre) = reste
        .strip_prefix(" reliée à l'idée du ")
        .or_else(|| reste.strip_prefix(" déliée de l'idée du "))
    {
        return apres_une_date(autre).is_some_and(str::is_empty);
    }
    [
        " : prise regroupée",
        " : MIDI versé regroupé",
        " : texte versé regroupé",
    ]
    .into_iter()
    .any(|regroupee| {
        reste.strip_prefix(regroupee).is_some_and(|fin| {
            fin.is_empty()
                || fin.strip_prefix(", idée du ").and_then(apres_une_date) == Some(" retirée")
        })
    })
}

/// `message` est celui d'une opération sur une requête enregistrée : « Requête « », un nom non
/// vide, puis « » enregistrée » ou « » supprimée » ; ou « Requête « », un nom non vide,
/// « » renommée « », un nom non vide, puis « » ».
fn est_une_operation_sur_une_requete(message: &str) -> bool {
    let Some(reste) = message.strip_prefix("Requête « ") else {
        return false;
    };
    if let Some(nom) = reste
        .strip_suffix(" » enregistrée")
        .or_else(|| reste.strip_suffix(" » supprimée"))
    {
        return !nom.is_empty();
    }
    reste
        .strip_suffix(" »")
        .and_then(|noms| noms.split_once(" » renommée « "))
        .is_some_and(|(ancien, nouveau)| !ancien.is_empty() && !nouveau.is_empty())
}

/// I28 (§ 70 · 3) : `message` est celui d'une opération connue : la genèse, la naissance, une
/// migration, ou l'une des formes fermées d'une note, d'un versement, d'un geste de tri ou d'une
/// opération sur une requête enregistrée, ou « Annulé : » suivi d'une forme d'un geste de tri.
pub fn message_connu(message: &str) -> bool {
    message == MESSAGE_DE_LA_GENESE
        || message == MESSAGE_DE_LA_NAISSANCE
        || est_une_migration(message)
        || est_une_note_ou_un_versement(message)
        || est_un_geste_de_tri(message)
        || est_une_operation_sur_une_requete(message)
        || message
            .strip_prefix("Annulé : ")
            .is_some_and(est_un_geste_de_tri)
}

/// Chaque valeur du document, avec les clés qui y mènent depuis la racine du document : chaque
/// valeur de chaque clé d'un objet à clés et de chaque indice d'une liste, les valeurs en conflit
/// comprises (`get_all`), et, d'une valeur qui est un objet, les siennes ; un objet texte n'en a
/// pas.
fn chaque_valeur(doc: &Automerge) -> Vec<(Vec<String>, Value<'_>)> {
    let mut valeurs = Vec::new();
    descendre(doc, &ROOT, ObjType::Map, &[], &mut valeurs);
    valeurs
}

/// Ajoute à `valeurs` chaque valeur de `objet`, de sorte `sorte`, au bout de `chemin`, puis celles
/// de chaque objet qu'il porte (`chaque_valeur`).
fn descendre<'d>(
    doc: &'d Automerge,
    objet: &ObjId,
    sorte: ObjType,
    chemin: &[String],
    valeurs: &mut Vec<(Vec<String>, Value<'d>)>,
) {
    let cles: Vec<(String, Prop)> = match sorte {
        ObjType::Map | ObjType::Table => doc
            .keys(objet)
            .map(|cle| (cle.clone(), Prop::Map(cle)))
            .collect(),
        ObjType::List => (0..doc.length(objet))
            .map(|indice| (indice.to_string(), Prop::Seq(indice)))
            .collect(),
        ObjType::Text => Vec::new(),
    };
    for (segment, cle) in cles {
        let mut sous = chemin.to_vec();
        sous.push(segment);
        let valeurs_de_la_cle = doc
            .get_all(objet, cle)
            .expect("lire une clé que le document vient de nommer");
        for (valeur, id) in valeurs_de_la_cle {
            if let Value::Object(sorte) = valeur {
                descendre(doc, &id, sorte, &sous, valeurs);
            }
            valeurs.push((sous.clone(), valeur));
        }
    }
}

/// Les clés de la racine du document hors de ses conteneurs : la version de schéma,
/// l'identifiant, la lignée et l'identifiant de propriétaire.
const CHAMPS_DE_LA_RACINE: [&str; 4] = [VERSION_DE_SCHEMA, IDENTIFIANT, LIGNEE, PROPRIETAIRE];
/// Les clés d'une idée.
const CHAMPS_D_UNE_IDEE: [&str; 7] = [TEXTE, ETIQUETTES, PROVENANCE, PAR, LE, DEPUIS, RETIREE];
/// Les faits d'une prise, sous `<empreinte>/<fait>`.
const FAITS_D_UNE_PRISE: [&str; 3] = [IDEE_D_UNE_CAPTATION, FORMAT, DUREE];
/// Les faits d'un MIDI versé ou d'un texte versé, sous `<empreinte>/<fait>`.
const FAITS_D_UN_MIDI_OU_D_UN_TEXTE: [&str; 2] = [IDEE_D_UNE_CAPTATION, FORMAT];
/// Les noms qu'une idée ne porte jamais (§ 30 · 13) : sa durée, son état « reliée », ses prises,
/// ses photos, ses MIDI versés et ses textes versés, tous engendrés.
const INTERDITS_SOUS_UNE_IDEE: [&str; 6] =
    [DUREE, "reliee", PRISES, PHOTOS, MIDI_VERSES, TEXTES_VERSES];
/// Les noms qu'aucune clé ne porte (§ 30 · 13) : un compte, un total, l'état « à envoyer ».
const INTERDITS_PARTOUT: [&str; 3] = ["compte", "total", "a_envoyer"];
/// Les clés d'une requête enregistrée : son nom, et une par sorte de critère, celles qu'écrit
/// `noyau::donnees::requete::enregistrer_une_requete`.
const CHAMPS_D_UNE_REQUETE: [&str; 9] = [
    NOM,
    TEXTE,
    DU,
    AU,
    ETIQUETTE,
    PLUS_DE_SECONDES,
    ETAT,
    PROVENANCE,
    CONTENU,
];
/// Les noms qu'une requête enregistrée ne porte jamais (I01) : ses résultats et leur nombre, que
/// chaque lecture engendre du fonds.
const INTERDITS_SOUS_UNE_REQUETE: [&str; 2] = ["resultats", "nombre"];

/// `cle` est `<empreinte>/<fait>`, coupée au premier `/`, l'empreinte non vide et le fait l'un de
/// `faits`.
fn fait_d_une_captation(cle: &str, faits: &[&str]) -> bool {
    cle.split_once('/')
        .is_some_and(|(nom, fait)| !nom.is_empty() && faits.contains(&fait))
}

/// `cle` est celle d'un lien « apparenté », `apparente:<a>:<b>`, ses deux bouts non vides.
fn cle_d_un_lien(cle: &str) -> bool {
    cle.strip_prefix(APPARENTE)
        .and_then(|reste| reste.strip_prefix(':'))
        .and_then(|bouts| bouts.split_once(':'))
        .is_some_and(|(a, b)| !a.is_empty() && !b.is_empty())
}

/// La liste blanche, tirée des clés du schéma : à la racine du document, ses champs et ses neuf
/// conteneurs ; sous `idees`, chaque idée, ses champs, et chaque étiquette sous `etiquettes` ;
/// sous `prises`, `<empreinte>/` suivi de `idee`, `format` ou `duree` ; sous `midi_verses` et
/// `textes_verses`, `<empreinte>/` suivi de `idee` ou `format` ; sous `liens`, la clé d'un lien ;
/// sous `requetes_enregistrees`, chaque requête enregistrée et ses champs
/// (`CHAMPS_D_UNE_REQUETE`), rien d'autre dessous.
fn permis(chemin: &[String]) -> bool {
    match chemin {
        [cle] => {
            CHAMPS_DE_LA_RACINE.contains(&cle.as_str())
                || CONTENEURS_DE_LA_GENESE.contains(&cle.as_str())
        }
        [conteneur, _] if conteneur == IDEES => true,
        [conteneur, _, champ] if conteneur == IDEES => CHAMPS_D_UNE_IDEE.contains(&champ.as_str()),
        [conteneur, _, sous, _] if conteneur == IDEES && sous == ETIQUETTES => true,
        [conteneur, cle] if conteneur == PRISES => fait_d_une_captation(cle, &FAITS_D_UNE_PRISE),
        [conteneur, cle] if conteneur == MIDI_VERSES || conteneur == TEXTES_VERSES => {
            fait_d_une_captation(cle, &FAITS_D_UN_MIDI_OU_D_UN_TEXTE)
        }
        [conteneur, cle] if conteneur == LIENS => cle_d_un_lien(cle),
        [conteneur, _] if conteneur == REQUETES_ENREGISTREES => true,
        [conteneur, _, champ] if conteneur == REQUETES_ENREGISTREES => {
            CHAMPS_D_UNE_REQUETE.contains(&champ.as_str())
        }
        _ => false,
    }
}

/// Un nom interdit (§ 30 · 13) : `etiquettes` à la racine du document ; sous une idée, l'un de
/// `INTERDITS_SOUS_UNE_IDEE` ; sous une requête enregistrée, l'un de `INTERDITS_SOUS_UNE_REQUETE` ;
/// partout, chaque clé coupée à chaque `/`, l'un de `INTERDITS_PARTOUT`, une étiquette ou un
/// identifiant de ce nom compris.
fn interdit(chemin: &[String]) -> bool {
    let sous_son_parent = match chemin {
        [cle] => cle == ETIQUETTES,
        [conteneur, _, champ] if conteneur == IDEES => {
            INTERDITS_SOUS_UNE_IDEE.contains(&champ.as_str())
        }
        [conteneur, _, champ] if conteneur == REQUETES_ENREGISTREES => {
            INTERDITS_SOUS_UNE_REQUETE.contains(&champ.as_str())
        }
        _ => false,
    };
    sous_son_parent
        || chemin
            .iter()
            .flat_map(|cle| cle.split('/'))
            .any(|segment| INTERDITS_PARTOUT.contains(&segment))
}

/// I01 (§ 70 · 3) : les chemins du document dont la clé sort de la liste blanche que tirent les
/// clés du schéma (`noyau::donnees::fonds`), ou porte un nom interdit (§ 30 · 13), chacun une
/// fois, ses clés jointes par `/`, dans l'ordre de leurs octets.
pub fn parcourir(doc: &Automerge) -> Vec<String> {
    let fautifs: BTreeSet<String> = chaque_valeur(doc)
        .into_iter()
        .filter(|(chemin, _)| !permis(chemin) || interdit(chemin))
        .map(|(chemin, _)| chemin.join("/"))
        .collect();
    fautifs.into_iter().collect()
}

/// Les chemins du document dont la valeur est un compteur (`ScalarValue::Counter`) ou un objet
/// texte (`ObjType::Text`), que Pupitre n'écrit pas (§ 31 · 2.6), ou une table ou une liste
/// (`ObjType::Table`, `ObjType::List`), que le schéma de la `v0.3.0` n'a pas (§ 31 · 2), chacun
/// une fois, ses clés jointes par `/`, dans l'ordre de leurs octets.
pub fn parcourir_les_types(doc: &Automerge) -> Vec<String> {
    let fautifs: BTreeSet<String> = chaque_valeur(doc)
        .into_iter()
        .filter(|(_, valeur)| match valeur {
            Value::Object(sorte) => {
                matches!(sorte, ObjType::Text | ObjType::Table | ObjType::List)
            }
            Value::Scalar(scalaire) => matches!(**scalaire, ScalarValue::Counter(_)),
        })
        .map(|(chemin, _)| chemin.join("/"))
        .collect();
    fautifs.into_iter().collect()
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
            message: "Le texte est vide.".to_string(),
        })
    );
    assert!(services.documents().fonds().is_none());
    assert_eq!(services.journal(), Ok(Vec::new()));
    assert_eq!(
        services.vue_d_une_idee(IDEE),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Aucune idée ne porte cet identifiant.".to_string(),
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
/// montrés, cette vue entière la même qu'avant la migration ; une note, deux gestes de tri,
/// « annuler » et la fiche d'une prise sont refusés du refus que rend `lecture_seule()`, qui nomme
/// la version de schéma 2 et dit que ce fonds vient d'une version plus récente de Pupitre, que
/// cette version-ci le lit sans y écrire, avant tout constat du noyau, sans rien tirer, et le
/// journal ne change pas ; la captation de la fiche refusée est versée, et une captation se verse
/// encore, et se relit.
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
    // Un constat, jamais un ordre (§ 50 · 5), le même sur les deux paires.
    assert_eq!(
        refus.message,
        "Ce fonds vient d'une version plus récente de Pupitre ; cette version-ci le lit sans y \
         écrire."
    );
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
    // Un geste de tri rend aussi le refus de la lecture seule, avant tout constat du noyau :
    // compléter l'idée d'un identifiant qu'aucune idée du fonds ne porte rendrait sinon « Aucune
    // idée ne porte cet identifiant. », et rétablir l'idée notée, qui n'est pas retirée, « Cette
    // idée n'est pas retirée. ».
    assert_eq!(
        services.completer(
            &completion(SECONDE_IDEE, "Un pont en mi mineur"),
            &mut contexte_de_test(&mut alea)
        ),
        Err(refus.clone())
    );
    assert_eq!(
        services.retablir(&visee(IDEE), &mut contexte_de_test(&mut alea)),
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
    let absente = Err(impossible(
        "Aucune idée ne porte cet identifiant.".to_string(),
    ));
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

    let retiree = Err(impossible("Cette idée est retirée.".to_string()));
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

/// A notée, une prise versée : regrouper dans A une empreinte qu'aucune captation du fonds ne
/// porte est refusé, du constat `Impossible::CaptationAbsente`, que seuls les services rendent,
/// et ni le journal ni la liste des idées ne changent.
pub fn regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse<P: Paire>(paire: &mut P) {
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
    verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    );
    let journal = services.journal().expect("le journal se lit");
    let idees = services.idees().expect("la liste des idées se lit");

    let jamais_versee = empreinte(b"une prise jamais versee");
    assert_eq!(
        services.regrouper(&regroupement(&jamais_versee, &a), &mut contexte),
        Err(impossible(format!(
            "Aucune captation ne porte l'empreinte {jamais_versee}."
        )))
    );
    assert_eq!(services.journal(), Ok(journal));
    assert_eq!(services.idees(), Ok(idees));
}

/// A et B notées : sous `liens`, une clé qui n'a pas la forme de celle d'un lien « apparenté »,
/// `autre:<A>:<B>`, qu'aucun geste n'écrit, et que le test pose donc par une opération, au type
/// du noyau, sans les services, n'est un lien ni de A ni de B. L'idée lue ne compte pour ses liens
/// que les clés que la règle du noyau reconnaît (`idee::bouts_du_lien`), comme la vue.
pub fn une_cle_d_une_autre_forme_sous_les_liens_n_est_pas_un_lien<P: Paire>(paire: &mut P) {
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
    let autre_forme = Operation {
        changements: vec![Changement::Poser {
            chemin: vec![LIENS.to_string(), format!("autre:{a}:{b}")],
            valeur: Simple::Vrai,
        }],
        message: "Une clé d'une autre forme sous les liens".to_string(),
        heure: HEURE,
    };
    services
        .documents_mut()
        .appliquer(&autre_forme)
        .expect("la clé d'une autre forme s'écrit");

    let fonds = services.documents().fonds().expect("le fonds est né");
    for id in [&a, &b] {
        assert_eq!(
            lire_l_idee(fonds, id).map(|idee| idee.liens),
            Some(BTreeSet::new()),
            "les liens de l'idée {id}"
        );
    }
    assert_eq!(
        services
            .vue_d_une_idee(&a)
            .map(|vue| (vue.liens, vue.reliee)),
        Ok((Vec::new(), false))
    );
}

/// A et B reliées : A est reliée ; B retirée, A ne l'est plus ; B rétablie, elle l'est de
/// nouveau. V1, née d'une prise de 30 000 ms, dure 30 000 ms ; une prise de 64 000 ms regroupée
/// dans V1 la porte à 64 000 ms ; un MIDI versé, sans durée, regroupé dans V1, la laisse à
/// 64 000 ms : la vue ne tire la durée que des prises (CAP-02, Règles : « celle de sa prise la
/// plus longue ») ; une prise sans durée lue, en FLAC, regroupée dans V1, lui ôte sa durée
/// (CAP-02, Règles : `—` dès qu'une de ses prises n'a pas la sienne) : son versement n'a écrit
/// aucune durée.
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
    let midi = services
        .verser_une_fiche(
            gestes::Sorte::MidiVerseSeul,
            b"un midi de la prise de trente secondes",
            &gestes::Fiche {
                format: "smf1".to_string(),
                duree_ms: None,
            },
            &importee(),
            0,
            &mut contexte,
        )
        .expect("le MIDI se verse");
    let regroupement_du_midi = Regrouper {
        captation: Captation {
            sorte: commandes::Sorte::MidiVerseSeul,
            empreinte: midi.empreinte,
        },
        cible: v1.idee.clone(),
        decalage_minutes: 0,
    };
    services
        .regrouper(&regroupement_du_midi, &mut contexte)
        .expect("le MIDI se regroupe dans V1");
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

/// Annuler ne vise que le dernier geste de tri fait depuis l'ouverture (§ 30 · 11) : une note ou
/// une fiche versée ne le vise ni ne le remplace, un geste refusé ne le change pas, et une
/// annulation ne s'annule pas, un second « annuler » est refusé. Pas à pas : noter A ; annuler,
/// refusé ; poser `#pont` sur A ; noter B ; verser une prise ; annuler, qui ôte `#pont` ; annuler,
/// refusé ; relier A et B ; retirer B ; poser `#pont` sur A ; le reposer, refusé ; annuler, qui
/// ôte `#pont` ; annuler, refusé. Restent le lien de A et B, B retirée, et A sans étiquette.
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
    verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    );
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

/// I28 sur l'historique (§ 70 · 3) : sur le fonds de chaque geste (`le_fonds_de_chaque_geste`),
/// la naissance compte à part, la première note écrivant la genèse, la naissance, puis son
/// message ; chaque autre commande faite ajoute une seule modification à `get_changes`, au message
/// que rend le noyau (`une_modification`). Sur un document de test, une seule modification, au
/// message d'une autre commande que celle attendue, est refusée ; une commande plantée en deux
/// modifications, deux `application::appliquer` de suite, est refusée, et le refus nomme les deux
/// messages.
pub fn chaque_operation_laisse_une_seule_modification_avec_son_message<P: Paire>(paire: &mut P) {
    let mut naissances = 0;
    let mut autres = 0;
    let services = le_fonds_de_chaque_geste(paire, &mut |fonds, avant, message| {
        if avant.is_empty() {
            naissances += 1;
            assert_eq!(
                messages_depuis(fonds, avant),
                vec![
                    MESSAGE_DE_LA_GENESE.to_string(),
                    MESSAGE_DE_LA_NAISSANCE.to_string(),
                    message.to_string(),
                ]
            );
        } else {
            autres += 1;
            assert_eq!(une_modification(fonds, avant, message), Ok(()));
        }
    });
    // La note qui fait naître le fonds ; puis les sept autres commandes du témoin, les huit gestes
    // de tri et l'annulation.
    assert_eq!((naissances, autres), (1, 16));
    let fonds = services.documents().fonds().expect("le fonds est né");
    assert_eq!(fonds.get_changes(&[]).len(), 19);

    let mut doc = un_document_de_test();
    let lue = lire_l_idee(&doc, IDEE).expect("l'idée du document de test se lit");
    let complement = gestes::completer(&lue, "Un refrain qui monte, puis retombe", HEURE, 0)
        .expect("le complément se calcule")
        .operation;
    let pose = gestes::poser_une_etiquette(&lue, "pont", HEURE, 0)
        .expect("la pose se calcule")
        .operation;
    let avant = doc.get_heads();
    application::appliquer(&mut doc, &complement).expect("le complément s'applique");
    // Une seule modification, celle du complément : attendue au message de la pose, elle est
    // refusée, et le refus nomme le message écrit.
    let refus = une_modification(&doc, &avant, &pose.message)
        .expect_err("une modification au message d'une autre commande est refusée");
    assert!(refus.contains(complement.message.as_str()), "{refus}");
    application::appliquer(&mut doc, &pose).expect("la pose s'applique");
    let refus = une_modification(&doc, &avant, &complement.message)
        .expect_err("une commande en deux modifications est refusée");
    for message in [&complement.message, &pose.message] {
        assert!(refus.contains(message.as_str()), "{refus}");
    }
}

/// Sur `services`, dont le fonds est né, à `LENDEMAIN`, sous un aléa de graine 1 : enregistrer
/// « Ponts lents » (la requête entière, `requete_entiere`), la renommer « Ponts du soir », puis la
/// supprimer, chacune faite sans avertissement.
fn les_operations_sur_une_requete<D: Documents, C: Captations>(services: &mut Services<D, C>) {
    let mut alea = AleaDeTest {
        graine: 1,
        appels: 0,
    };
    let mut contexte = Contexte {
        heure: LENDEMAIN,
        alea: &mut alea,
    };
    let fait = || -> Reponse<Faite> {
        Ok(Faite {
            avertissement: None,
        })
    };
    assert_eq!(
        services.enregistrer_une_requete(
            &enregistrement("Ponts lents", requete_entiere()),
            &mut contexte
        ),
        fait()
    );
    let id = les_requetes(services)
        .pop()
        .expect("la requête enregistrée se lit")
        .id;
    assert_eq!(
        services.renommer_une_requete(&renommage(&id, "Ponts du soir"), &mut contexte),
        fait()
    );
    assert_eq!(
        services.supprimer_une_requete(&suppression(&id), &mut contexte),
        fait()
    );
}

/// I28 sur l'historique (§ 70 · 3) : sur le fonds de chaque geste, où une requête est aussi
/// enregistrée, renommée et supprimée (`les_operations_sur_une_requete`), chaque message du
/// journal est celui d'une opération connue (`message_connu`). `message_connu` refuse « Idée du
/// 3 novembre : autre chose », une forme qu'aucune opération n'écrit, « Annulé : Idée du
/// 3 novembre notée », une note, qu'annuler ne vise pas, des messages bien formés hors d'une date
/// que le noyau n'écrit pas (un jour 0, 1, 01, 03 ou 32, un mois inconnu) ou d'une étiquette vide,
/// et, d'une requête enregistrée, un nom vide, une autre forme, ou « Annulé : » suivi d'une
/// opération sur elle, qu'annuler ne vise pas ; il reconnaît la genèse, la naissance, une
/// migration, un exemple de chaque forme d'une note, d'un versement, d'un geste de tri et d'une
/// opération sur une requête enregistrée, et « Annulé : » suivi de chaque forme d'un geste de tri.
pub fn l_historique_ne_porte_que_des_operations_connues<P: Paire>(paire: &mut P) {
    let mut services = le_fonds_de_chaque_geste(paire, &mut |_, _, _| {});
    les_operations_sur_une_requete(&mut services);
    let journal = services.journal().expect("le journal se lit");
    // Les dix-neuf modifications du fonds de chaque geste, puis les trois de la requête.
    assert_eq!(journal.len(), 19 + 3, "{journal:?}");
    for entree_du_journal in &journal {
        assert!(
            message_connu(&entree_du_journal.message),
            "« {} » n'est le message d'aucune opération connue",
            entree_du_journal.message
        );
    }

    // Hors des deux premiers, chacun est bien formé, hors de sa date ou de son étiquette : un
    // jour 0, 1 (le noyau écrit « 1er »), 01 ou 03 (aucun zéro devant), ou 32 ; un mois inconnu ;
    // une étiquette vide.
    for inconnu in [
        "Idée du 3 novembre : autre chose",
        "Annulé : Idée du 3 novembre notée",
        "Idée du 0 novembre notée",
        "Idée du 1 novembre notée",
        "Idée du 01 novembre notée",
        "Idée du 03 novembre notée",
        "Idée du 32 novembre notée",
        "Idée du 3 brumaire notée",
        "Idée du 3 novembre : étiquette # posée",
        "Requête «  » enregistrée",
        "Requête «  » supprimée",
        "Requête «  » renommée « Ponts du soir »",
        "Requête « Ponts lents » renommée «  »",
        "Requête « Ponts lents » renommée",
        "Requête « Ponts lents » oubliée",
        "Requête « Ponts lents » enregistrée hier",
        "Requête Ponts lents enregistrée",
        "Annulé : Requête « Ponts lents » enregistrée",
    ] {
        assert!(!message_connu(inconnu), "« {inconnu} » est reconnu");
    }
    let notes_et_versements = [
        "Idée du 3 novembre notée",
        "Idée du 3 novembre : prise versée",
        "Idée du 3 novembre : MIDI versé",
        "Idée du 3 novembre : texte versé",
    ];
    let gestes_de_tri = [
        "Idée du 3 novembre : texte complété",
        "Idée du 1er janvier : étiquette #pont posée",
        "Idée du 31 décembre : étiquette #refrain ôtée",
        "Idée du 2 février reliée à l'idée du 29 février",
        "Idée du 15 mars déliée de l'idée du 1er avril",
        "Idée du 9 mai : prise regroupée",
        "Idée du 10 juin : MIDI versé regroupé",
        "Idée du 11 juillet : texte versé regroupé",
        "Idée du 12 août : prise regroupée, idée du 13 septembre retirée",
        "Idée du 14 octobre : MIDI versé regroupé, idée du 1er novembre retirée",
        "Idée du 3 novembre : texte versé regroupé, idée du 4 novembre retirée",
        "Idée du 5 novembre retirée",
        "Idée du 6 novembre rétablie",
    ];
    let operations_sur_une_requete = [
        "Requête « Ponts lents » enregistrée",
        "Requête « Ponts lents » renommée « Ponts du soir »",
        "Requête « Ponts du soir » supprimée",
    ];
    let migration = message_de_la_migration(2);
    let connus = [
        MESSAGE_DE_LA_GENESE,
        MESSAGE_DE_LA_NAISSANCE,
        migration.as_str(),
    ];
    for connu in connus
        .into_iter()
        .chain(notes_et_versements)
        .chain(gestes_de_tri)
        .chain(operations_sur_une_requete)
    {
        assert!(message_connu(connu), "« {connu} » n'est pas reconnu");
    }
    for geste in gestes_de_tri {
        let annulation = format!("Annulé : {geste}");
        assert!(
            message_connu(&annulation),
            "« {annulation} » n'est pas reconnu"
        );
    }
}

// § 31 · 10, test 7 : chercher les champs interdits du § 30 · 13
/// I01 (§ 70 · 3) : sur le fonds de chaque geste, où une requête entière (`requete_entiere`) est
/// enregistrée de plus, chacune de ses neuf clés écrite, chaque clé est dans la liste blanche que
/// tirent les clés du schéma (`noyau::donnees::fonds`), et aucune ne porte un nom interdit
/// (§ 30 · 13) : `parcourir` n'y trouve rien. Sur un document de test, une durée posée à la main
/// sous l'idée, `idees/<i>/duree`, la durée d'une idée étant engendrée de ses prises, puis une
/// date et un appareil sous une prise, `prises/<empreinte>/le` et `prises/<empreinte>/par`,
/// qu'une prise ne porte pas, des faits de sa captation que rien ne lit encore sur elle
/// (§ 30 · 9.2), puis, sous une requête enregistrée plantée avec son nom, ses résultats et leur
/// nombre, `requetes_enregistrees/<r>/resultats` et `requetes_enregistrees/<r>/nombre`, et une clé
/// sous son contenu, posé en objet, `requetes_enregistrees/<r>/contenu/prise`, sont les six
/// chemins qu'il rend.
pub fn rien_d_engendre_n_est_stocke<P: Paire>(paire: &mut P) {
    let mut services = le_fonds_de_chaque_geste(paire, &mut |_, _, _| {});
    let mut alea = AleaDeTest {
        graine: 1,
        appels: 0,
    };
    services
        .enregistrer_une_requete(
            &enregistrement("Ponts lents", requete_entiere()),
            &mut Contexte {
                heure: LENDEMAIN,
                alea: &mut alea,
            },
        )
        .expect("la requête entière est enregistrée");
    let fonds = services.documents().fonds().expect("le fonds est né");
    assert_eq!(parcourir(fonds), Vec::<String>::new());

    let mut doc = un_document_de_test();
    let idee = objet_au_bout(&doc, &[IDEES, IDEE]);
    let prises_du_document = objet_au_bout(&doc, &[PRISES]);
    let requetes = objet_au_bout(&doc, &[REQUETES_ENREGISTREES]);
    let prise = empreinte(PRISE_DE_TEST);
    let plantee = "01HF7YAT000000000000000009";
    let mut tx = doc.transaction();
    tx.put(&idee, DUREE, 30_000u64)
        .expect("poser une durée sous l'idée");
    tx.put(&prises_du_document, format!("{prise}/{LE}"), HEURE)
        .expect("poser une date sous la prise");
    tx.put(&prises_du_document, format!("{prise}/{PAR}"), "etabli")
        .expect("poser un appareil sous la prise");
    let requete = tx
        .put_object(&requetes, plantee, ObjType::Map)
        .expect("créer une requête enregistrée");
    tx.put(&requete, NOM, "Ponts lents")
        .expect("poser le nom de la requête");
    tx.put(&requete, "resultats", 3u64)
        .expect("poser des résultats sous la requête");
    tx.put(&requete, "nombre", 3u64)
        .expect("poser un nombre sous la requête");
    let contenu = tx
        .put_object(&requete, CONTENU, ObjType::Map)
        .expect("créer un objet sous le contenu de la requête");
    tx.put(&contenu, "prise", true)
        .expect("poser une clé sous le contenu de la requête");
    let (plantation, _) = tx.commit();
    plantation.expect("la plantation est une modification");
    assert_eq!(
        parcourir(&doc),
        vec![
            format!("{IDEES}/{IDEE}/{DUREE}"),
            format!("{PRISES}/{prise}/{LE}"),
            format!("{PRISES}/{prise}/{PAR}"),
            format!("{REQUETES_ENREGISTREES}/{plantee}/{CONTENU}/prise"),
            format!("{REQUETES_ENREGISTREES}/{plantee}/nombre"),
            format!("{REQUETES_ENREGISTREES}/{plantee}/resultats"),
        ]
    );
}

/// Pupitre n'écrit ni compteur ni objet texte (§ 31 · 2.6), ni table ni liste : sur le fonds de
/// chaque geste, `parcourir_les_types` ne rencontre ni `ScalarValue::Counter`, ni
/// `ObjType::Text`, ni `ObjType::Table`, ni `ObjType::List`. Sur un document de test, un compteur
/// et un objet texte posés à la main sous l'idée, `idees/<i>/compteur_de_test` et
/// `idees/<i>/texte_de_test`, sont les deux chemins qu'il rend.
pub fn aucun_counter_ni_objet_text_dans_un_document_des_operations<P: Paire>(paire: &mut P) {
    let services = le_fonds_de_chaque_geste(paire, &mut |_, _, _| {});
    let fonds = services.documents().fonds().expect("le fonds est né");
    assert_eq!(parcourir_les_types(fonds), Vec::<String>::new());

    let mut doc = un_document_de_test();
    let idee = objet_au_bout(&doc, &[IDEES, IDEE]);
    let mut tx = doc.transaction();
    tx.put(&idee, "compteur_de_test", ScalarValue::counter(0))
        .expect("poser un compteur sous l'idée");
    tx.put_object(&idee, "texte_de_test", ObjType::Text)
        .expect("créer un objet texte sous l'idée");
    let (plantation, _) = tx.commit();
    plantation.expect("la plantation est une modification");
    assert_eq!(
        parcourir_les_types(&doc),
        vec![
            format!("{IDEES}/{IDEE}/compteur_de_test"),
            format!("{IDEES}/{IDEE}/texte_de_test"),
        ]
    );
}

/// L'heure des cinquante idées de CAP-02 · 1, en secondes : le 13 septembre 2020, 12 h 26 min
/// 40 s UTC, plus de trois ans avant `HEURE`.
const EN_2020: i64 = 1_600_000_000;

/// La boîte entière, autant qu'une fenêtre en rend : par date, sans critère ni borne, de la
/// première ligne, `FENETRE_MAX` lignes.
pub fn toute_la_boite() -> Boite {
    Boite {
        ordre: Ordre::Date,
        requete: Requete::default(),
        bornes: Bornes::default(),
        debut: 0,
        nombre: FENETRE_MAX,
    }
}

/// Les idées retirées, autant qu'une fenêtre en rend : de la première ligne, `FENETRE_MAX` lignes.
fn toutes_les_retirees() -> Fenetre {
    Fenetre {
        debut: 0,
        nombre: FENETRE_MAX,
    }
}

/// La vue de la boîte entière (`toute_la_boite`).
fn la_boite<D: Documents, C: Captations>(services: &Services<D, C>) -> VueDeLaBoite {
    services.boite(&toute_la_boite()).expect("la boîte se lit")
}

/// L'idée de chaque ligne, dans l'ordre des lignes.
fn idees_des_lignes(lignes: &[LigneDeLaBoite]) -> Vec<String> {
    lignes.iter().map(|ligne| ligne.idee.clone()).collect()
}

/// L'assiette dont chaque compte vaut `n`, sans illisible.
fn assiette_de(n: u64) -> Assiette {
    Assiette {
        considerees: n,
        portant: n,
        decidees: n,
        illisibles: 0,
    }
}

/// L'assiette du noyau qui répond à celle des commandes, sans ses illisibles, que le noyau ne
/// porte pas dans l'assiette, chaque compte par `usize::try_from`, `usize::MAX` au-delà : la seule
/// conversion de la suite d'une assiette des commandes.
pub fn assiette_du_noyau(assiette: &Assiette) -> noyau::donnees::idee::Assiette {
    let compte = |n: u64| usize::try_from(n).unwrap_or(usize::MAX);
    noyau::donnees::idee::Assiette {
        considerees: compte(assiette.considerees),
        portant: compte(assiette.portant),
        decidees: compte(assiette.decidees),
    }
}

/// Le texte du compte de la boîte, tel que le dit le noyau (`texte_du_compte`), de
/// `non_reliees` idées non reliées sur `assiette`.
pub fn texte_du_compte_de(non_reliees: u64, assiette: &Assiette) -> String {
    texte_du_compte(
        usize::try_from(non_reliees).unwrap_or(usize::MAX),
        assiette_du_noyau(assiette),
    )
}

/// Un document de test (`un_document_de_test`), puis la note de l'idée `SECONDE_IDEE`, « Un pont
/// en mi mineur », à `HEURE`.
fn un_document_de_deux_idees() -> Automerge {
    let mut doc = un_document_de_test();
    let note = gestes::noter(
        SECONDE_IDEE,
        "Un pont en mi mineur",
        gestes::Appareil::Etabli,
        HEURE,
        0,
    )
    .expect("la seconde note du document de test se calcule");
    application::appliquer(&mut doc, &note).expect("la seconde note s'écrit");
    doc
}

/// Un document de deux idées (`un_document_de_deux_idees`), dont le test pose la date de
/// `SECONDE_IDEE`, `le`, en texte : elle n'a plus le type du schéma, et ne se lit plus. Rend les
/// octets du document (`save()`).
pub fn un_document_dont_une_idee_est_illisible() -> Vec<u8> {
    let mut doc = un_document_de_deux_idees();
    let idee = objet_au_bout(&doc, &[IDEES, SECONDE_IDEE]);
    let mut tx = doc.transaction();
    tx.put(&idee, LE, "le lendemain")
        .expect("poser la date de l'idée en texte");
    let (plantation, _) = tx.commit();
    plantation.expect("la plantation est une modification");
    doc.save()
}

/// Un document de deux idées (`un_document_de_deux_idees`), où le test pose, dans le conteneur
/// `photos`, la clé `<empreinte>/idee` d'une photo inventée, qui nomme `SECONDE_IDEE` : aucun geste
/// n'écrit de photo au lot 1. Rend les octets du document (`save()`).
pub fn un_document_dont_une_idee_porte_une_photo() -> Vec<u8> {
    let mut doc = un_document_de_deux_idees();
    let photos = objet_au_bout(&doc, &[PHOTOS]);
    let photo = empreinte(b"une photo de test");
    let mut tx = doc.transaction();
    tx.put(
        &photos,
        format!("{photo}/{IDEE_D_UNE_CAPTATION}"),
        SECONDE_IDEE,
    )
    .expect("poser l'idée de la photo");
    let (plantation, _) = tx.commit();
    plantation.expect("la plantation est une modification");
    doc.save()
}

/// La requête enregistrée « Ponts lents » que plante
/// `un_document_aux_requetes_d_une_version_plus_recente` : une sorte de critère inconnue.
pub const REQUETE_PLUS_RECENTE: &str = "01HF7YAT000000000000000011";
/// La requête enregistrée « Église » que plante le même document : son texte est un objet.
pub const REQUETE_AU_TEXTE_EN_OBJET: &str = "01HF7YAT000000000000000012";

/// Un document de test (`un_document_de_test`), où le test plante, sous `requetes_enregistrees`,
/// ce qu'aucune opération de cette version n'écrit : « Ponts lents » (`REQUETE_PLUS_RECENTE`), le
/// texte « pont » et une sorte de critère inconnue, `tempo`, comme d'une version plus récente de
/// Pupitre ; « Église » (`REQUETE_AU_TEXTE_EN_OBJET`), dont le texte est un objet ; une requête sans
/// nom, son texte seul ; et une valeur simple, qui n'est pas un objet. Rend les octets du document
/// (`save()`).
pub fn un_document_aux_requetes_d_une_version_plus_recente() -> Vec<u8> {
    let mut doc = un_document_de_test();
    let requetes = objet_au_bout(&doc, &[REQUETES_ENREGISTREES]);
    let mut tx = doc.transaction();
    let plus_recente = tx
        .put_object(&requetes, REQUETE_PLUS_RECENTE, ObjType::Map)
        .expect("créer la requête d'une version plus récente");
    tx.put(&plus_recente, NOM, "Ponts lents")
        .expect("poser son nom");
    tx.put(&plus_recente, TEXTE, "pont")
        .expect("poser son texte");
    tx.put(&plus_recente, "tempo", "lent")
        .expect("poser une sorte de critère inconnue");
    let texte_en_objet = tx
        .put_object(&requetes, REQUETE_AU_TEXTE_EN_OBJET, ObjType::Map)
        .expect("créer la requête au texte en objet");
    tx.put(&texte_en_objet, NOM, "Église")
        .expect("poser son nom");
    tx.put_object(&texte_en_objet, TEXTE, ObjType::Map)
        .expect("poser son texte en objet");
    let sans_nom = tx
        .put_object(&requetes, "01HF7YAT000000000000000013", ObjType::Map)
        .expect("créer la requête sans nom");
    tx.put(&sans_nom, TEXTE, "pont").expect("poser son texte");
    tx.put(&requetes, "01HF7YAT000000000000000014", "une valeur simple")
        .expect("poser une valeur simple sous les requêtes enregistrées");
    let (plantation, _) = tx.commit();
    plantation.expect("la plantation est une modification");
    doc.save()
}

/// CAP-02 · 1 : cinquante idées non reliées, notées en 2020, plus de trois ans avant `HEURE`. La
/// boîte en compte cinquante, sur une assiette de cinquante, « 50 idées non reliées · 50
/// interrogées », et ses lignes ne portent que ce que montre une ligne, sans ancienneté : la vue,
/// puis chaque ligne, se défont par un motif exhaustif, si bien qu'un champ ajouté, un âge, un
/// badge ou une alerte, casse la compilation. Ni les parts « sans … » de la requête, aucune sans
/// critère, ni les étiquettes du fonds, aucune ici, ne datent une idée.
pub fn aucun_compteur_d_anciennete<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    contexte.heure = EN_2020;
    let mut notees = BTreeSet::new();
    for n in 1..=50 {
        let notee = services
            .noter(
                &une_note(&format!("Une idée de septembre, n° {n}")),
                &mut contexte,
            )
            .expect("la note est faite");
        notees.insert(notee.idee);
    }

    let VueDeLaBoite {
        non_reliees,
        assiette_du_compte,
        retenues,
        assiette_de_la_requete,
        sans,
        etiquettes_du_fonds,
        lignes,
    } = la_boite(&services);
    assert_eq!(non_reliees, 50);
    assert_eq!(assiette_du_compte, assiette_de(50));
    assert_eq!(retenues, 50);
    assert_eq!(assiette_de_la_requete, assiette_de(50));
    assert_eq!(sans, Sans::default());
    assert_eq!(etiquettes_du_fonds, Vec::<String>::new());
    assert_eq!(
        texte_du_compte_de(non_reliees, &assiette_du_compte),
        "50 idées non reliées · 50 interrogées"
    );
    assert_eq!(lignes.len(), 50);
    let mut montrees = BTreeSet::new();
    for ligne in lignes {
        let LigneDeLaBoite {
            idee,
            le,
            texte,
            etiquettes,
            duree_ms,
            reliee,
            provenance,
            retiree,
        } = ligne;
        assert_eq!(le, EN_2020, "la date de l'idée {idee}");
        assert!(
            texte
                .as_deref()
                .is_some_and(|texte| texte.starts_with("Une idée de septembre, n° ")),
            "le texte de l'idée {idee} : {texte:?}"
        );
        assert_eq!(etiquettes, Vec::<String>::new());
        assert_eq!(duree_ms, None);
        assert!(!reliee, "l'idée {idee} est reliée");
        assert_eq!(
            provenance,
            commandes::Provenance::Captee {
                par: commandes::Appareil::Etabli,
            }
        );
        assert_eq!(retiree, None);
        montrees.insert(idee);
    }
    assert_eq!(montrees, notees);
}

/// CAP-02 · 3 : P et Q, nées du versement de deux prises, de 12 000 et de 47 000 ms : la boîte
/// compte deux idées non reliées, en deux lignes. La prise de Q regroupée dans P, la boîte les
/// montre ensemble : une seule ligne, P, de la durée de sa prise la plus longue (`idee::duree`) ;
/// P porte les deux captations, chacune distincte, d'empreinte inchangée, que les captations
/// relisent (LE-6, règle 4), et Q, vidée, est retirée par le même geste : la boîte ne compte plus
/// qu'une idée non reliée.
pub fn deux_prises_regroupees_paraissent_ensemble_dans_la_boite<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let octets_de_p: &[u8] = b"une prise de douze secondes";
    let octets_de_q: &[u8] = b"une prise de quarante-sept secondes";
    let Versee {
        empreinte: p_emp,
        idee: p,
        ..
    } = verser_une_prise(
        &mut services,
        octets_de_p,
        &fiche_wav(12_000),
        &mut contexte,
    );
    let Versee {
        empreinte: q_emp,
        idee: q,
        ..
    } = verser_une_prise(
        &mut services,
        octets_de_q,
        &fiche_wav(47_000),
        &mut contexte,
    );
    assert_eq!(p_emp, empreinte(octets_de_p));
    assert_eq!(q_emp, empreinte(octets_de_q));
    let vue = la_boite(&services);
    assert_eq!(vue.non_reliees, 2);
    assert_eq!(vue.lignes.len(), 2);

    services
        .regrouper(&regroupement(&q_emp, &p), &mut contexte)
        .expect("la prise de Q se regroupe dans P");
    let vue = la_boite(&services);
    assert_eq!(idees_des_lignes(&vue.lignes), vec![p.clone()]);
    assert_eq!(vue.lignes[0].duree_ms, Some(47_000));
    assert_eq!(vue.non_reliees, 1);
    assert_eq!(
        services.vue_d_une_idee(&p).map(|vue| vue.captations),
        Ok(prises(&[&p_emp, &q_emp]))
    );
    assert_eq!(
        services.vue_d_une_idee(&q).map(|vue| vue.retiree),
        Ok(Some(HEURE))
    );
    assert_eq!(
        services.captations().relire(&p_emp),
        Ok(octets_de_p.to_vec())
    );
    assert_eq!(
        services.captations().relire(&q_emp),
        Ok(octets_de_q.to_vec())
    );
}

/// CAP-02 · 5, une part (les requêtes viennent en tranche 4, les photos au lot 2) : X, née d'une
/// prise, porte un texte, une étiquette, sa prise et un lien vers Y, seule autre idée. Avant :
/// deux lignes, X et Y, aucune idée non reliée, l'assiette du compte `{2, 2, 2}`. X retirée :
/// elle quitte la boîte, Y ne relie plus rien de non retiré et devient non reliée, l'assiette
/// `{1, 1, 1}`, et la liste des idées retirées porte X, seule. X rétablie : sa ligne est celle
/// d'avant, aucune idée non reliée, l'assiette `{2, 2, 2}`, et aucune idée retirée (règle 5).
pub fn retirer_ote_l_idee_de_la_boite_et_retablir_l_y_rend<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let y = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note Y est faite")
        .idee;
    let x = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(30_000),
        &mut contexte,
    )
    .idee;
    services
        .completer(
            &completion(&x, "Une basse qui descend sous la prise"),
            &mut contexte,
        )
        .expect("le texte de X est complété");
    services
        .poser_une_etiquette(&etiquetage(&x, "refrain"), &mut contexte)
        .expect("l'étiquette est posée sur X");
    services
        .relier(&lien(&x, &y), &mut contexte)
        .expect("X est reliée à Y");

    let avant = la_boite(&services);
    // Deux idées de la même date se rangent par leur identifiant : Y, tirée avant X, d'abord.
    assert_eq!(idees_des_lignes(&avant.lignes), vec![y.clone(), x.clone()]);
    assert_eq!(avant.non_reliees, 0);
    assert_eq!(avant.assiette_du_compte, assiette_de(2));
    let ligne_de_x = LigneDeLaBoite {
        idee: x.clone(),
        le: HEURE,
        texte: Some("Une basse qui descend sous la prise".to_string()),
        etiquettes: vec!["refrain".to_string()],
        duree_ms: Some(30_000),
        reliee: true,
        provenance: commandes::Provenance::Importee {
            par: commandes::Appareil::Etabli,
            depuis: IMPORT.to_string(),
        },
        retiree: None,
    };
    assert_eq!(avant.lignes[1], ligne_de_x);

    services
        .retirer(&visee(&x), &mut contexte)
        .expect("X est retirée");
    let pendant = la_boite(&services);
    assert_eq!(idees_des_lignes(&pendant.lignes), vec![y.clone()]);
    assert_eq!(pendant.non_reliees, 1);
    assert_eq!(pendant.assiette_du_compte, assiette_de(1));
    assert_eq!(
        services.retirees(&toutes_les_retirees()),
        Ok(VueDesRetirees {
            total: 1,
            lignes: vec![LigneDeLaBoite {
                retiree: Some(HEURE),
                ..ligne_de_x.clone()
            }],
        })
    );

    services
        .retablir(&visee(&x), &mut contexte)
        .expect("X est rétablie");
    let apres = la_boite(&services);
    assert_eq!(idees_des_lignes(&apres.lignes), vec![y, x]);
    assert_eq!(apres.lignes[1], ligne_de_x);
    assert_eq!(apres.non_reliees, 0);
    assert_eq!(apres.assiette_du_compte, assiette_de(2));
    assert_eq!(
        services.retirees(&toutes_les_retirees()),
        Ok(VueDesRetirees {
            total: 0,
            lignes: Vec::new(),
        })
    );
}

/// CAP-07 · 4 : le nombre d'idées non reliées, après chaque geste qui le change. A et B notées,
/// 2 ; reliées, 0, le nombre baisse de deux ; C notée, 1 ; B retirée, 2 (A redevient non reliée,
/// B sort) ; B rétablie, 1 ; A et B déliées, 3 ; A reliée à B et à C, 0 ; B retirée, 0 (A reste
/// reliée à C). Lire la boîte, les idées retirées et l'état du fonds n'écrit rien : le journal est
/// le même avant et après, sans qu'aucun compteur ne soit écrit (règle 1).
pub fn le_nombre_d_idees_non_reliees_suit_chaque_geste<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let non_reliees = |services: &Services<P::D, P::C>| la_boite(services).non_reliees;
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    let b = services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite")
        .idee;
    assert_eq!(non_reliees(&services), 2);
    services
        .relier(&lien(&a, &b), &mut contexte)
        .expect("A est reliée à B");
    assert_eq!(non_reliees(&services), 0);
    let c = services
        .noter(&une_note("Une coda en suspens"), &mut contexte)
        .expect("la note C est faite")
        .idee;
    assert_eq!(non_reliees(&services), 1);
    services
        .retirer(&visee(&b), &mut contexte)
        .expect("B est retirée");
    assert_eq!(non_reliees(&services), 2);
    services
        .retablir(&visee(&b), &mut contexte)
        .expect("B est rétablie");
    assert_eq!(non_reliees(&services), 1);
    services
        .delier(&lien(&a, &b), &mut contexte)
        .expect("A et B sont déliées");
    assert_eq!(non_reliees(&services), 3);
    services
        .relier(&lien(&a, &b), &mut contexte)
        .expect("A est reliée à B");
    services
        .relier(&lien(&a, &c), &mut contexte)
        .expect("A est reliée à C");
    assert_eq!(non_reliees(&services), 0);
    services
        .retirer(&visee(&b), &mut contexte)
        .expect("B est retirée");
    assert_eq!(non_reliees(&services), 0);

    let journal = services.journal().expect("le journal se lit");
    la_boite(&services);
    services
        .retirees(&toutes_les_retirees())
        .expect("les idées retirées se lisent");
    services.etat_du_fonds().expect("l'état du fonds se lit");
    assert_eq!(services.journal(), Ok(journal));
}

/// I31 : deux textes piégés, `<script>alert(1)</script>` et `<img src=x onerror=alert(1)>`,
/// notés ; le second posé aussi en étiquette sur la première idée, et le premier donné pour nom à
/// une requête enregistrée : la vue de l'idée et la ligne de la boîte rendent chaque texte, la
/// vue de la première idée et les étiquettes du fonds l'étiquette, et la liste des requêtes
/// enregistrées le nom, octet pour octet, rien n'en étant échappé ni retiré.
pub fn un_texte_piege_revient_tel_quel<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let script = "<script>alert(1)</script>";
    let image = "<img src=x onerror=alert(1)>";
    let mut notees = Vec::new();
    for piege in [script, image] {
        let idee = services
            .noter(&une_note(piege), &mut contexte)
            .expect("le texte piégé est noté")
            .idee;
        notees.push((idee, piege));
    }
    let premiere = notees[0].0.clone();
    services
        .poser_une_etiquette(&etiquetage(&premiere, image), &mut contexte)
        .expect("l'étiquette piégée est posée");
    services
        .enregistrer_une_requete(&enregistrement(script, cherche("alert")), &mut contexte)
        .expect("la requête au nom piégé est enregistrée");

    let vue = la_boite(&services);
    assert_eq!(
        services.vue_d_une_idee(&premiere).map(|vue| {
            vue.etiquettes
                .into_iter()
                .map(String::into_bytes)
                .collect::<Vec<_>>()
        }),
        Ok(vec![image.as_bytes().to_vec()]),
        "les étiquettes de l'idée {premiere}"
    );
    assert_eq!(
        vue.etiquettes_du_fonds
            .iter()
            .map(String::as_bytes)
            .collect::<Vec<_>>(),
        vec![image.as_bytes()]
    );
    assert_eq!(
        les_requetes(&services)
            .iter()
            .map(|requete| requete.nom.as_bytes())
            .collect::<Vec<_>>(),
        vec![script.as_bytes()]
    );
    for (idee, piege) in notees {
        assert_eq!(
            services
                .vue_d_une_idee(&idee)
                .map(|vue| vue.texte.map(String::into_bytes)),
            Ok(Some(piege.as_bytes().to_vec())),
            "la vue de l'idée {idee}"
        );
        let ligne = vue
            .lignes
            .iter()
            .find(|ligne| ligne.idee == idee)
            .expect("l'idée a sa ligne");
        assert_eq!(
            ligne.texte.as_deref().map(str::as_bytes),
            Some(piege.as_bytes()),
            "la ligne de l'idée {idee}"
        );
    }
}

/// Note une idée par numéro de `numeros`, « Une idée de la fenêtre, n° N ».
fn noter_les_idees_de_la_fenetre<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    contexte: &mut Contexte<'_>,
    numeros: impl IntoIterator<Item = u32>,
) {
    for n in numeros {
        services
            .noter(
                &une_note(&format!("Une idée de la fenêtre, n° {n}")),
                contexte,
            )
            .expect("la note est faite");
    }
}

/// Trois idées : une fenêtre qui commence après la dernière ligne n'en rend aucune, et la boîte
/// compte pourtant ses trois idées, non reliées et retenues ; une fenêtre de zéro ligne n'en rend
/// aucune. Puis 205 idées : une fenêtre de 1 000 lignes en rend `FENETRE_MAX`, 200, sur 205
/// retenues, et la fenêtre suivante les cinq dernières, si bien que les deux en rendent 205,
/// chacune une fois.
pub fn une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let fenetre = |debut: u32, nombre: u32| Boite {
        ordre: Ordre::Date,
        requete: Requete::default(),
        bornes: Bornes::default(),
        debut,
        nombre,
    };
    noter_les_idees_de_la_fenetre(&mut services, &mut contexte, 1..=3);
    let vue = services
        .boite(&fenetre(5, FENETRE_MAX))
        .expect("la boîte se lit");
    assert_eq!(vue.lignes, Vec::new());
    assert_eq!(vue.non_reliees, 3);
    assert_eq!(vue.retenues, 3);
    let vue = services.boite(&fenetre(0, 0)).expect("la boîte se lit");
    assert_eq!(vue.lignes, Vec::new());
    assert_eq!(vue.retenues, 3);

    noter_les_idees_de_la_fenetre(&mut services, &mut contexte, 4..=205);
    let premieres = services.boite(&fenetre(0, 1_000)).expect("la boîte se lit");
    assert_eq!(premieres.lignes.len(), 200);
    assert_eq!(premieres.retenues, 205);
    let dernieres = services
        .boite(&fenetre(200, 1_000))
        .expect("la boîte se lit");
    assert_eq!(dernieres.lignes.len(), 5);
    let montrees: BTreeSet<String> = premieres
        .lignes
        .iter()
        .chain(&dernieres.lignes)
        .map(|ligne| ligne.idee.clone())
        .collect();
    assert_eq!(montrees.len(), 205);
}

/// Sans fonds, la boîte n'a aucune ligne, ses comptes à 0, ses assiettes à `{0, 0, 0, 0}`, aucune
/// part « sans … » ni étiquette du fonds, aucune idée n'est retirée, et l'état du fonds le dit non
/// né, sans lecture seule, ni avertissement, ni geste à annuler. Une note fait naître le fonds :
/// l'état le dit né, et rien d'autre ne change.
pub fn la_boite_d_un_pupitre_neuf<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    assert_eq!(
        services.boite(&toute_la_boite()),
        Ok(VueDeLaBoite {
            non_reliees: 0,
            assiette_du_compte: assiette_de(0),
            retenues: 0,
            assiette_de_la_requete: assiette_de(0),
            sans: Sans::default(),
            etiquettes_du_fonds: Vec::new(),
            lignes: Vec::new(),
        })
    );
    assert_eq!(
        services.retirees(&toutes_les_retirees()),
        Ok(VueDesRetirees {
            total: 0,
            lignes: Vec::new(),
        })
    );
    let neuf = EtatDuFonds {
        ne: false,
        lecture_seule: None,
        avertissements: Vec::new(),
        a_annuler: None,
    };
    assert_eq!(services.etat_du_fonds(), Ok(neuf.clone()));

    services
        .noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la note est faite");
    assert_eq!(
        services.etat_du_fonds(),
        Ok(EtatDuFonds { ne: true, ..neuf })
    );
}

/// Un texte de 250 `é` : la ligne de la boîte en porte les 200 premiers caractères, et la vue de
/// l'idée les 250.
pub fn une_ligne_borne_son_texte<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let long = "é".repeat(250);
    let idee = services
        .noter(&une_note(&long), &mut contexte_de_test(&mut alea))
        .expect("la note est faite")
        .idee;

    let vue = la_boite(&services);
    let texte = vue.lignes[0]
        .texte
        .clone()
        .expect("la ligne porte un texte");
    assert_eq!(texte.chars().count(), 200);
    assert_eq!(texte, "é".repeat(200));
    assert_eq!(
        services.vue_d_une_idee(&idee).map(|vue| vue.texte),
        Ok(Some(long))
    );
}

/// A notée : aucun geste à annuler. Une étiquette posée sur A : l'état du fonds dit le message de
/// la dernière entrée du journal, celui du geste. B notée, une note ne visant pas « annuler » :
/// l'état ne change pas. Le geste annulé : aucun geste à annuler.
pub fn l_etat_du_fonds_dit_le_geste_qu_annuler_vise<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a_annuler = |services: &Services<P::D, P::C>| {
        services
            .etat_du_fonds()
            .expect("l'état du fonds se lit")
            .a_annuler
    };
    let a = services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite")
        .idee;
    assert_eq!(a_annuler(&services), None);

    services
        .poser_une_etiquette(&etiquetage(&a, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    let journal = services.journal().expect("le journal se lit");
    let geste = journal
        .last()
        .expect("le journal porte le geste")
        .message
        .clone();
    assert_eq!(geste, "Idée du 14 novembre : étiquette #pont posée");
    assert_eq!(a_annuler(&services), Some(geste.clone()));

    services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite");
    assert_eq!(a_annuler(&services), Some(geste));

    services
        .annuler(&mut contexte)
        .expect("le geste est annulé");
    assert_eq!(a_annuler(&services), None);
}

/// A, B et C notées ; A retirée à `HEURE + 10`, B à `HEURE + 20` : les idées retirées se rangent
/// par la date du retrait, la plus récente d'abord, B puis A, quand leurs identifiants les
/// rangeraient A puis B ; elles sont deux, et la boîte ne montre que C.
pub fn les_retirees_se_rangent_par_la_date_du_retrait<P: Paire>(paire: &mut P) {
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
    let c = services
        .noter(&une_note("Une coda en suspens"), &mut contexte)
        .expect("la note C est faite")
        .idee;
    assert!(a < b, "A, tirée avant B, se range avant elle");
    contexte.heure = HEURE + 10;
    services
        .retirer(&visee(&a), &mut contexte)
        .expect("A est retirée");
    contexte.heure = HEURE + 20;
    services
        .retirer(&visee(&b), &mut contexte)
        .expect("B est retirée");

    let retirees = services
        .retirees(&toutes_les_retirees())
        .expect("les idées retirées se lisent");
    assert_eq!(retirees.total, 2);
    assert_eq!(idees_des_lignes(&retirees.lignes), vec![b, a]);
    assert_eq!(
        retirees
            .lignes
            .iter()
            .map(|ligne| ligne.retiree)
            .collect::<Vec<_>>(),
        vec![Some(HEURE + 20), Some(HEURE + 10)]
    );
    assert_eq!(idees_des_lignes(&la_boite(&services).lignes), vec![c]);
}

/// La requête et l'ordre de la commande passent au noyau, et `retenues` compte les idées que la
/// requête trouve. A, notée à `HEURE`, puis, de minute en minute, B, une prise de 12 000 ms, C,
/// une prise de 47 000 ms, D, un MIDI versé, et E, un texte versé, chacune importée et sans texte ;
/// A reliée à C ; `#couplet` sur D, `#refrain` sur E. Sans critère, cinq idées retenues, dont
/// trois non reliées, B, D et E, et les étiquettes du fonds, `couplet` puis `refrain` ; la requête
/// d'un contenu ne trouve que ses idées, par date, et laisse les trois non reliées : le texte de
/// l'idée, A ; une prise, C puis B ; une photo, aucune ; un MIDI versé, D ; un texte versé, E.
/// La requête d'un état ou d'une provenance ne trouve, par date, que ses idées : les non reliées,
/// E, D, B ; les reliées, C, A ; la captée, A ; les importées, E, D, C, B : un bras inversé de
/// l'état ou de la provenance de la requête, dans sa conversion vers le noyau, se voit.
/// Chaque ordre range les cinq selon `noyau::donnees::boite::ordonner`, à égalité par la date, la
/// plus récente d'abord, puis par l'identifiant : par date, E, D, C, B, A ; par étiquette, D
/// (`couplet`), E (`refrain`), puis les sans étiquette, C, B, A ; par durée, C, B, puis les sans
/// durée, E, D, A ; par état, les non reliées E, D, B, puis les reliées C, A ; par provenance, la
/// captée A, puis les importées E, D, C, B. Les cinq suites, comme les cinq jeux que trouvent les
/// requêtes d'un contenu, sont deux à deux distinctes : un bras d'ordre ou de contenu inversé, un
/// ordre ou un contenu perdu, se voient. Puis, sur le fonds de `paire_des_notes`, trois idées
/// notées, « Un refrain qui monte, en ré », « Un pont en mi mineur » et « Une coda qui
/// ralentit » : la requête de texte « pont » ne trouve que la deuxième ; aucune n'est sans texte
/// (`sans.texte` à `Some(0)`, les deux autres parts à `None`, leur critère n'étant pas posé), et
/// ce fonds n'a aucune étiquette.
pub fn la_requete_passe_au_noyau<P: Paire>(paire: &mut P, paire_des_notes: &mut P) {
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
    contexte.heure = HEURE + 60;
    let b = verser_une_prise(
        &mut services,
        b"une prise de douze secondes",
        &fiche_wav(12_000),
        &mut contexte,
    )
    .idee;
    contexte.heure = HEURE + 120;
    let c = verser_une_prise(
        &mut services,
        b"une prise de quarante-sept secondes",
        &fiche_wav(47_000),
        &mut contexte,
    )
    .idee;
    contexte.heure = HEURE + 180;
    let d = services
        .verser_une_fiche(
            gestes::Sorte::MidiVerseSeul,
            b"un midi de la boite",
            &gestes::Fiche {
                format: "smf1".to_string(),
                duree_ms: None,
            },
            &importee(),
            0,
            &mut contexte,
        )
        .expect("le MIDI se verse")
        .idee;
    contexte.heure = HEURE + 240;
    let e = services
        .verser_une_fiche(
            gestes::Sorte::TexteVerse,
            b"un texte de la boite",
            &gestes::Fiche {
                format: "brut".to_string(),
                duree_ms: None,
            },
            &importee(),
            0,
            &mut contexte,
        )
        .expect("le texte se verse")
        .idee;
    services
        .relier(&lien(&a, &c), &mut contexte)
        .expect("A est reliée à C");
    services
        .poser_une_etiquette(&etiquetage(&d, "couplet"), &mut contexte)
        .expect("l'étiquette est posée sur D");
    services
        .poser_une_etiquette(&etiquetage(&e, "refrain"), &mut contexte)
        .expect("l'étiquette est posée sur E");
    let boite_de = |ordre: Ordre, contenu: Option<Contenu>| Boite {
        ordre,
        requete: Requete {
            contenu,
            ..Requete::default()
        },
        bornes: Bornes::default(),
        debut: 0,
        nombre: FENETRE_MAX,
    };

    let toutes = services
        .boite(&boite_de(Ordre::Date, None))
        .expect("la boîte se lit");
    assert_eq!(toutes.retenues, 5);
    assert_eq!(toutes.non_reliees, 3);
    assert_eq!(toutes.etiquettes_du_fonds, ["couplet", "refrain"]);

    for (contenu, attendues, retenues) in [
        (Contenu::TexteDeLIdee, vec![a.clone()], 1),
        (Contenu::Prise, vec![c.clone(), b.clone()], 2),
        (Contenu::Photo, Vec::new(), 0),
        (Contenu::MidiVerse, vec![d.clone()], 1),
        (Contenu::TexteVerse, vec![e.clone()], 1),
    ] {
        let vue = services
            .boite(&boite_de(Ordre::Date, Some(contenu.clone())))
            .expect("la boîte se lit");
        assert_eq!(
            idees_des_lignes(&vue.lignes),
            attendues,
            "le contenu {contenu:?}"
        );
        assert_eq!(vue.retenues, retenues, "le contenu {contenu:?}");
        assert_eq!(vue.non_reliees, 3, "le contenu {contenu:?}");
    }

    for (requete, attendues) in [
        (
            Requete {
                etat: Some(Etat::NonReliee),
                ..Requete::default()
            },
            vec![e.clone(), d.clone(), b.clone()],
        ),
        (
            Requete {
                etat: Some(Etat::Reliee),
                ..Requete::default()
            },
            vec![c.clone(), a.clone()],
        ),
        (
            Requete {
                provenance: Some(ProvenanceDeLIdee::Captee),
                ..Requete::default()
            },
            vec![a.clone()],
        ),
        (
            Requete {
                provenance: Some(ProvenanceDeLIdee::Importee),
                ..Requete::default()
            },
            vec![e.clone(), d.clone(), c.clone(), b.clone()],
        ),
    ] {
        let vue = services
            .boite(&Boite {
                requete: requete.clone(),
                ..toute_la_boite()
            })
            .expect("la boîte se lit");
        assert_eq!(
            idees_des_lignes(&vue.lignes),
            attendues,
            "la requête {requete:?}"
        );
        assert_eq!(
            vue.retenues,
            u64::try_from(attendues.len()).expect("un compte"),
            "la requête {requete:?}"
        );
        assert_eq!(vue.non_reliees, 3, "la requête {requete:?}");
    }

    let mut suites = BTreeSet::new();
    for (ordre, attendue) in [
        (Ordre::Date, [&e, &d, &c, &b, &a]),
        (Ordre::Etiquette, [&d, &e, &c, &b, &a]),
        (Ordre::Duree, [&c, &b, &e, &d, &a]),
        (Ordre::Reliee, [&e, &d, &b, &c, &a]),
        (Ordre::Provenance, [&a, &e, &d, &c, &b]),
    ] {
        let vue = services
            .boite(&boite_de(ordre.clone(), None))
            .expect("la boîte se lit");
        let rangees = idees_des_lignes(&vue.lignes);
        assert_eq!(rangees, attendue.map(String::clone), "l'ordre {ordre:?}");
        suites.insert(rangees);
    }
    assert_eq!(suites.len(), 5, "deux ordres rendent la même suite");

    let mut services = paire_des_notes.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let notees: Vec<String> = [
        "Un refrain qui monte, en ré",
        "Un pont en mi mineur",
        "Une coda qui ralentit",
    ]
    .into_iter()
    .map(|texte| {
        services
            .noter(&une_note(texte), &mut contexte)
            .expect("la note est faite")
            .idee
    })
    .collect();
    let pont = Boite {
        requete: Requete {
            texte: "pont".to_string(),
            ..Requete::default()
        },
        ..toute_la_boite()
    };
    let vue = services.boite(&pont).expect("la boîte se lit");
    assert_eq!(idees_des_lignes(&vue.lignes), vec![notees[1].clone()]);
    assert_eq!(vue.retenues, 1);
    assert_eq!(
        vue.sans,
        Sans {
            etiquette: None,
            duree: None,
            texte: Some(0),
        }
    );
    assert_eq!(vue.etiquettes_du_fonds, Vec::<String>::new());
}

/// Le texte de la réponse d'une vue de la boîte, tel que le dit le noyau (`texte_de_la_reponse`) :
/// ses retenues, l'assiette de la requête (`assiette_du_noyau`), ses illisibles et ses parts
/// « sans … », chaque compte par `usize::try_from`, `usize::MAX` au-delà.
pub fn texte_de_la_reponse_de(vue: &VueDeLaBoite) -> String {
    let compte = |n: u64| usize::try_from(n).unwrap_or(usize::MAX);
    texte_de_la_reponse(
        compte(vue.retenues),
        assiette_du_noyau(&vue.assiette_de_la_requete),
        compte(vue.assiette_de_la_requete.illisibles),
        noyau::donnees::requete::Sans {
            etiquette: vue.sans.etiquette.map(compte),
            duree: vue.sans.duree.map(compte),
            texte: vue.sans.texte.map(compte),
        },
    )
}

/// La boîte sous `requete`, autant qu'une fenêtre en rend : `toute_la_boite`, la requête posée.
fn boite_de_la_requete(requete: Requete) -> Boite {
    Boite {
        requete,
        ..toute_la_boite()
    }
}

/// La requête du texte `texte`, sans autre critère.
fn cherche(texte: &str) -> Requete {
    Requete {
        texte: texte.to_string(),
        ..Requete::default()
    }
}

/// La requête « plus d'une minute » : une durée affichée de plus de 60 secondes.
fn plus_d_une_minute() -> Requete {
    Requete {
        plus_de_secondes: Some(60),
        ..Requete::default()
    }
}

/// Le jeu de requêtes d'I09, d'I13 et de « chercher n'écrit rien » : la requête vide ; « eglise » ;
/// l'étiquette `pont` ; plus d'une minute ; les non reliées ; les importées ; le contenu `Prise` ;
/// « pont » et les non reliées ensemble.
fn jeu_de_requetes() -> Vec<Requete> {
    vec![
        Requete::default(),
        cherche("eglise"),
        Requete {
            etiquette: Some("pont".to_string()),
            ..Requete::default()
        },
        plus_d_une_minute(),
        Requete {
            etat: Some(Etat::NonReliee),
            ..Requete::default()
        },
        Requete {
            provenance: Some(ProvenanceDeLIdee::Importee),
            ..Requete::default()
        },
        Requete {
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        },
        Requete {
            etat: Some(Etat::NonReliee),
            ..cherche("pont")
        },
    ]
}

/// Note `texte` (`une_note`) ; rend l'idée notée.
fn noter_l_idee<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    texte: &str,
    contexte: &mut Contexte<'_>,
) -> String {
    services
        .noter(&une_note(texte), contexte)
        .expect("la note est faite")
        .idee
}

/// Verse une prise WAV de `duree_ms` millisecondes (`fiche_wav`), d'octets propres à sa durée
/// (« une prise de N ms »), par `verser_une_prise`.
fn verser_la_prise_de<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    duree_ms: u64,
    contexte: &mut Contexte<'_>,
) -> Versee {
    verser_une_prise(
        services,
        format!("une prise de {duree_ms} ms").as_bytes(),
        &fiche_wav(duree_ms),
        contexte,
    )
}

/// CAP-06 · 7, l'exemple de la tâche 3 (`plus_d_une_minute_se_juge_a_la_seconde_affichee`, dans
/// `crates/noyau/tests/index.rs`) rejoué par les opérations nommées : trois idées notées, sans
/// prise ; neuf nées du versement d'une prise, de 61 000, 90 000, 600 000, 75 000, 60 999,
/// 59 999, 30 000, 20 000 et 999 ms ; une prise de 12 000 ms regroupée dans l'idée de celle de
/// 90 000, une de 5 000 dans celle de 20 000, chacune retirant l'idée née avec elle. La requête
/// « plus d'une minute » retient quatre idées, et les lignes les listent : celles des prises de
/// 61 000, 90 000, 600 000 et 75 000 ms, 61 000 comprise, 60 999 (« 1:00 ») non ; elle
/// interroge douze idées, dont neuf portent une durée, décidée, sans illisible ; les trois notées
/// sont sans durée : « 4 idées · 3 sans durée · 12 interrogées ».
pub fn plus_d_une_minute_declare_son_assiette<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    for texte in [
        "Un refrain qui monte, en ré",
        "Un pont en mi mineur",
        "Une coda en suspens",
    ] {
        noter_l_idee(&mut services, texte, &mut contexte);
    }
    let idees: BTreeMap<u64, String> = [
        61_000, 90_000, 600_000, 75_000, 60_999, 59_999, 30_000, 20_000, 999,
    ]
    .into_iter()
    .map(|duree_ms| {
        let idee = verser_la_prise_de(&mut services, duree_ms, &mut contexte).idee;
        (duree_ms, idee)
    })
    .collect();
    for (duree_ms, cible) in [(12_000, 90_000_u64), (5_000, 20_000)] {
        let versee = verser_la_prise_de(&mut services, duree_ms, &mut contexte);
        services
            .regrouper(
                &regroupement(&versee.empreinte, &idees[&cible]),
                &mut contexte,
            )
            .expect("la prise se regroupe");
    }

    let vue = services
        .boite(&boite_de_la_requete(plus_d_une_minute()))
        .expect("la boîte se lit");
    assert_eq!(vue.retenues, 4);
    assert_eq!(
        idees_des_lignes(&vue.lignes)
            .into_iter()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(
            [61_000_u64, 90_000, 600_000, 75_000].map(|duree_ms| idees[&duree_ms].clone())
        )
    );
    assert_eq!(
        vue.assiette_de_la_requete,
        Assiette {
            considerees: 12,
            portant: 9,
            decidees: 9,
            illisibles: 0,
        }
    );
    assert_eq!(vue.sans.duree, Some(3));
    assert_eq!(
        texte_de_la_reponse_de(&vue),
        "4 idées · 3 sans durée · 12 interrogées"
    );
}

/// CAP-06 · 8, l'exemple de la tâche 3 (`une_idee_retiree_n_est_ni_trouvee_ni_comptee`, dans
/// `crates/noyau/tests/index.rs`) rejoué par les opérations nommées : cinq idées notées,
/// « Église du port », « une église retirée », que le test retire, « le phare et la jetée »,
/// « … », un texte sans mot, et « L’EGLISE, encore » ; deux nées du versement d'une prise, sans
/// texte. « eglise » trouve deux idées, compte les deux prises sans texte, et interroge les six
/// idées non retirées : « 2 idées · 2 sans texte · 6 interrogées » ; l'idée retirée n'est pas dans
/// les lignes.
pub fn une_idee_retiree_n_est_ni_trouvee_ni_comptee<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let mut notees = Vec::new();
    for texte in [
        "Église du port",
        "une église retirée",
        "le phare et la jetée",
        "\u{2026}",
        "L\u{2019}EGLISE, encore",
    ] {
        notees.push(noter_l_idee(&mut services, texte, &mut contexte));
    }
    verser_la_prise_de(&mut services, 30_000, &mut contexte);
    verser_la_prise_de(&mut services, 47_000, &mut contexte);
    let retiree = &notees[1];
    services
        .retirer(&visee(retiree), &mut contexte)
        .expect("l'idée est retirée");

    let vue = services
        .boite(&boite_de_la_requete(cherche("eglise")))
        .expect("la boîte se lit");
    assert_eq!(
        texte_de_la_reponse_de(&vue),
        "2 idées · 2 sans texte · 6 interrogées"
    );
    assert!(
        !idees_des_lignes(&vue.lignes).contains(retiree),
        "l'idée retirée est dans les lignes"
    );
}

/// CAP-02 · 5, la part des requêtes (les photos viennent au lot 2) : X, notée « Un pont en mi
/// mineur », porte `pont` ; Y, notée « Un refrain qui monte, en ré », n'a aucune étiquette. La
/// requête de l'étiquette `pont` trouve X ; X retirée, elle quitte les lignes, et la requête
/// interroge une idée de moins.
pub fn une_idee_retiree_quitte_les_requetes<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let x = noter_l_idee(&mut services, "Un pont en mi mineur", &mut contexte);
    noter_l_idee(&mut services, "Un refrain qui monte, en ré", &mut contexte);
    services
        .poser_une_etiquette(&etiquetage(&x, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur X");
    let pont = boite_de_la_requete(Requete {
        etiquette: Some("pont".to_string()),
        ..Requete::default()
    });

    let avant = services.boite(&pont).expect("la boîte se lit");
    assert_eq!(idees_des_lignes(&avant.lignes), vec![x.clone()]);
    services
        .retirer(&visee(&x), &mut contexte)
        .expect("X est retirée");
    let apres = services.boite(&pont).expect("la boîte se lit");
    assert_eq!(apres.lignes, Vec::new());
    assert_eq!(
        apres.assiette_de_la_requete.considerees + 1,
        avant.assiette_de_la_requete.considerees
    );
}

/// CAP-06 · 9 : P, née du versement d'une prise de 30 000 ms, où se regroupe une prise versée
/// sans durée (`Fiche { duree_ms: None, … }`) : la durée de P ne se lit pas (`idee::duree`), et
/// l'idée née avec la prise sans durée est retirée par le regroupement. La requête « plus d'une
/// minute » ne trouve pas P, la compte parmi les sans durée, et interroge une idée, P, qui ne porte
/// pas de durée : l'assiette `{1, 0, 0}`, sans illisible.
pub fn une_prise_sans_duree_compte_parmi_les_sans_duree<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let p = verser_la_prise_de(&mut services, 30_000, &mut contexte).idee;
    let sans_duree = verser_une_prise(
        &mut services,
        b"une prise sans duree",
        &gestes::Fiche {
            format: "wav".to_string(),
            duree_ms: None,
        },
        &mut contexte,
    );
    services
        .regrouper(&regroupement(&sans_duree.empreinte, &p), &mut contexte)
        .expect("la prise sans durée se regroupe dans P");

    let vue = services
        .boite(&boite_de_la_requete(plus_d_une_minute()))
        .expect("la boîte se lit");
    assert!(
        !idees_des_lignes(&vue.lignes).contains(&p),
        "P est dans les lignes"
    );
    assert_eq!(
        vue.assiette_de_la_requete,
        Assiette {
            considerees: 1,
            portant: 0,
            decidees: 0,
            illisibles: 0,
        }
    );
    assert_eq!(vue.sans.duree, Some(1));
}

/// CAP-06 · 10 : « Église du port », son É d'un seul point de code (U+00C9), et
/// « E\u{301}glise du port », un E suivi de l'accent aigu combinant (U+0301), notées : « eglise »
/// les trouve toutes deux.
pub fn eglise_trouve_eglise_du_port<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let composee = noter_l_idee(&mut services, "\u{c9}glise du port", &mut contexte);
    let decomposee = noter_l_idee(&mut services, "E\u{301}glise du port", &mut contexte);

    let vue = services
        .boite(&boite_de_la_requete(cherche("eglise")))
        .expect("la boîte se lit");
    assert_eq!(
        idees_des_lignes(&vue.lignes)
            .into_iter()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([composee, decomposee])
    );
}

/// I09 : sur le fonds de chaque geste (`le_fonds_de_chaque_geste`), chaque requête du jeu
/// (`jeu_de_requetes`) déclare son assiette : elle considère les idées lisibles non retirées
/// (`Services::idees`) et les illisibles ; celles dont la donnée est décidée sont celles qui la
/// portent ; les idées retenues ne dépassent pas celles qui la portent, ni aucune part « sans … »
/// les considérées ; et son texte (`texte_de_la_reponse`) finit par « interrogée » ou
/// « interrogées ».
pub fn chaque_reponse_declare_son_assiette<P: Paire>(paire: &mut P) {
    let services = le_fonds_de_chaque_geste(paire, &mut |_, _, _| {});
    let lisibles = u64::try_from(services.idees().expect("les idées se lisent").len())
        .expect("un compte d'idées");
    for requete in jeu_de_requetes() {
        let vue = services
            .boite(&boite_de_la_requete(requete.clone()))
            .expect("la boîte se lit");
        let assiette = &vue.assiette_de_la_requete;
        assert_eq!(
            assiette.considerees,
            lisibles + assiette.illisibles,
            "la requête {requete:?}"
        );
        assert_eq!(
            assiette.decidees, assiette.portant,
            "la requête {requete:?}"
        );
        assert!(
            vue.retenues <= assiette.portant,
            "la requête {requete:?} : {vue:?}"
        );
        for part in [vue.sans.etiquette, vue.sans.duree, vue.sans.texte]
            .into_iter()
            .flatten()
        {
            assert!(
                part <= assiette.considerees,
                "la requête {requete:?} : {vue:?}"
            );
        }
        let texte = texte_de_la_reponse_de(&vue);
        assert!(
            texte.ends_with(" interrogée") || texte.ends_with(" interrogées"),
            "la requête {requete:?} : {texte}"
        );
    }
}

/// L'empreinte d'une vue de la boîte (I13) : `identifiants::empreinte` de ses octets en JSON
/// (`serde_json::to_vec`).
fn empreinte_de_la_vue(vue: &VueDeLaBoite) -> String {
    empreinte(&serde_json::to_vec(vue).expect("la vue s'écrit en JSON"))
}

/// Pour chaque requête du jeu (`jeu_de_requetes`), dans son ordre, la fenêtre de `FENETRE_MAX`
/// lignes : l'empreinte de la vue que rend l'index tenu (`Services::boite`), puis celle que rend un
/// index neuf (`Services::boite_sans_index`), chacune par `empreinte_de_la_vue`.
fn empreintes_du_jeu<D: Documents, C: Captations>(
    services: &Services<D, C>,
) -> Vec<(String, String)> {
    jeu_de_requetes()
        .into_iter()
        .map(|requete| {
            let boite = boite_de_la_requete(requete);
            let tenu = services.boite(&boite).expect("la boîte se lit");
            let neuf = services
                .boite_sans_index(&boite)
                .expect("la boîte se lit sans index");
            (empreinte_de_la_vue(&tenu), empreinte_de_la_vue(&neuf))
        })
        .collect()
}

/// I13 : l'index est un dérivé jetable. A, « Église du port », et B, « Un pont en mi mineur »,
/// notées ; P, née du versement d'une prise de 61 000 ms, puis Q, d'une prise de 30 000 ms ; `pont`
/// posée sur B ; A reliée à P ; P retirée, si bien que A redevient non reliée ; le retrait
/// annulé ; puis une étiquette de blancs sur B, refusée. Après chaque geste, chaque requête du jeu
/// (`jeu_de_requetes`) rend, par l'index tenu, la vue de même empreinte que par un index neuf
/// (`empreintes_du_jeu`) ; puis le fonds rouvert rend, par les deux, les empreintes d'avant la
/// réouverture.
pub fn l_index_reconstruit_rend_les_memes_reponses<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let memes_reponses = |services: &Services<P::D, P::C>, geste: &str| {
        let empreintes = empreintes_du_jeu(services);
        for (requete, (tenu, neuf)) in jeu_de_requetes().iter().zip(&empreintes) {
            assert_eq!(tenu, neuf, "après {geste}, la requête {requete:?}");
        }
        empreintes
    };

    let a = noter_l_idee(&mut services, "Église du port", &mut contexte);
    memes_reponses(&services, "la note A");
    let b = noter_l_idee(&mut services, "Un pont en mi mineur", &mut contexte);
    memes_reponses(&services, "la note B");
    let p = verser_la_prise_de(&mut services, 61_000, &mut contexte).idee;
    memes_reponses(&services, "le versement de P");
    verser_la_prise_de(&mut services, 30_000, &mut contexte);
    memes_reponses(&services, "le versement de Q");
    services
        .poser_une_etiquette(&etiquetage(&b, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur B");
    memes_reponses(&services, "l'étiquette posée sur B");
    services
        .relier(&lien(&a, &p), &mut contexte)
        .expect("A est reliée à P");
    memes_reponses(&services, "le lien de A et de P");
    services
        .retirer(&visee(&p), &mut contexte)
        .expect("P est retirée");
    assert_eq!(services.vue_d_une_idee(&a).map(|vue| vue.reliee), Ok(false));
    memes_reponses(&services, "le retrait de P");
    services
        .annuler(&mut contexte)
        .expect("le retrait est annulé");
    memes_reponses(&services, "l'annulation");
    assert_eq!(
        services.poser_une_etiquette(&etiquetage(&b, "   "), &mut contexte),
        Err(impossible("L'étiquette est vide.".to_string()))
    );
    let avant = memes_reponses(&services, "l'étiquette de blancs refusée");

    let services = paire.rouvrir(services, &[]);
    assert_eq!(memes_reponses(&services, "la réouverture"), avant);
}

/// Chercher n'écrit rien (question 14) : sur le fonds de chaque geste
/// (`le_fonds_de_chaque_geste`), chaque requête du jeu (`jeu_de_requetes`) laisse les têtes du
/// fonds (`Automerge::get_heads`) et le journal tels qu'ils étaient avant elle.
pub fn chercher_n_ecrit_rien<P: Paire>(paire: &mut P) {
    let services = le_fonds_de_chaque_geste(paire, &mut |_, _, _| {});
    let tetes = |services: &Services<P::D, P::C>| {
        services
            .documents()
            .fonds()
            .expect("le fonds est né")
            .get_heads()
    };
    for requete in jeu_de_requetes() {
        let tetes_d_avant = tetes(&services);
        let journal = services.journal().expect("le journal se lit");
        services
            .boite(&boite_de_la_requete(requete.clone()))
            .expect("la boîte se lit");
        assert_eq!(tetes(&services), tetes_d_avant, "la requête {requete:?}");
        assert_eq!(services.journal(), Ok(journal), "la requête {requete:?}");
    }
}

/// Le point de revue 1 : une note faite sous une requête en cours la laisse en place. « Église du
/// port » et « le phare et la jetée » notées, sous la requête « eglise » : noter « pont du soir »,
/// que la requête ne trouve pas ; la vue suivante, sous la même requête, ne la montre pas, garde
/// les lignes et les retenues de la vue d'avant, et compte une idée non reliée de plus. Puis
/// noter « l’église du soir » : la vue suivante la montre, retient une idée de plus, et compte
/// encore une idée non reliée de plus.
pub fn noter_pendant_une_requete_la_laisse_en_place<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    noter_l_idee(&mut services, "Église du port", &mut contexte);
    noter_l_idee(&mut services, "le phare et la jetée", &mut contexte);
    let eglise = boite_de_la_requete(cherche("eglise"));
    let avant = services.boite(&eglise).expect("la boîte se lit");

    let pont = noter_l_idee(&mut services, "pont du soir", &mut contexte);
    let pendant = services.boite(&eglise).expect("la boîte se lit");
    assert!(
        !idees_des_lignes(&pendant.lignes).contains(&pont),
        "« pont du soir » est dans les lignes"
    );
    assert_eq!(
        idees_des_lignes(&pendant.lignes),
        idees_des_lignes(&avant.lignes)
    );
    assert_eq!(pendant.retenues, avant.retenues);
    assert_eq!(pendant.non_reliees, avant.non_reliees + 1);

    let soir = noter_l_idee(&mut services, "l\u{2019}église du soir", &mut contexte);
    let apres = services.boite(&eglise).expect("la boîte se lit");
    assert!(
        idees_des_lignes(&apres.lignes).contains(&soir),
        "« l’église du soir » n'est pas dans les lignes"
    );
    assert_eq!(apres.retenues, pendant.retenues + 1);
    assert_eq!(apres.non_reliees, pendant.non_reliees + 1);
}

/// Une note, puis une étiquette posée sur elle, le dernier geste de tri. Enregistrer « Ponts
/// lents » (la requête « pont ») : sous l'ULID tiré à l'heure du contexte, le cinquième tirage,
/// après les quatre de la naissance et de la note ; la renommer « Ponts du soir » ; enregistrer
/// « Église » (la requête entière, `requete_entiere`), sous le sixième : la liste les range par
/// nom replié, « Église » d'abord, quand leurs ULID les rangeraient « Ponts du soir » d'abord ;
/// supprimer « Ponts du soir » : la liste rend « Église » seule. Le journal finit par les trois
/// messages, puis « Requête « Ponts du soir » supprimée », à l'heure du contexte. Renommer, même
/// sous un nom vide, ou supprimer la requête supprimée est refusé (`Impossible::RequeteAbsente`) ;
/// enregistrer sous un nom vide (`NomVide`) ou renommer « Église » « Église » (`MemeNom`) aussi, du
/// constat du noyau ; le journal ne change pas. Aucune de ces opérations ne touche ce qu'annuler
/// vise : l'état du fonds dit toujours l'étiquette posée, et l'annulation la défait.
pub fn enregistrer_renommer_et_supprimer_une_requete<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let fait = || -> Reponse<Faite> {
        Ok(Faite {
            avertissement: None,
        })
    };
    let a = noter_l_idee(&mut services, "Un pont en mi mineur", &mut contexte);
    services
        .poser_une_etiquette(&etiquetage(&a, "lent"), &mut contexte)
        .expect("l'étiquette est posée");
    let geste = "Idée du 14 novembre : étiquette #lent posée";
    let ponts = "01HF7YAT000000000000000005";
    let eglise = "01HF7YAT000000000000000006";

    assert_eq!(
        services.enregistrer_une_requete(
            &enregistrement("Ponts lents", cherche("pont")),
            &mut contexte
        ),
        fait()
    );
    assert_eq!(
        les_requetes(&services),
        vec![RequeteEnregistree {
            id: ponts.to_string(),
            nom: "Ponts lents".to_string(),
            requete: Some(cherche("pont")),
        }]
    );
    assert_eq!(
        services.renommer_une_requete(&renommage(ponts, "Ponts du soir"), &mut contexte),
        fait()
    );
    assert_eq!(
        services
            .enregistrer_une_requete(&enregistrement("Église", requete_entiere()), &mut contexte),
        fait()
    );
    let l_eglise = RequeteEnregistree {
        id: eglise.to_string(),
        nom: "Église".to_string(),
        requete: Some(requete_entiere()),
    };
    let ponts_du_soir = RequeteEnregistree {
        id: ponts.to_string(),
        nom: "Ponts du soir".to_string(),
        requete: Some(cherche("pont")),
    };
    assert_eq!(
        les_requetes(&services),
        vec![l_eglise.clone(), ponts_du_soir]
    );
    assert_eq!(
        services.supprimer_une_requete(&suppression(ponts), &mut contexte),
        fait()
    );
    assert_eq!(les_requetes(&services), vec![l_eglise]);
    let journal = services.journal().expect("le journal se lit");
    assert_eq!(
        journal,
        vec![
            entree(MESSAGE_DE_LA_GENESE, 0),
            entree(MESSAGE_DE_LA_NAISSANCE, HEURE),
            entree("Idée du 14 novembre notée", HEURE),
            entree(geste, HEURE),
            entree("Requête « Ponts lents » enregistrée", HEURE),
            entree("Requête « Ponts lents » renommée « Ponts du soir »", HEURE),
            entree("Requête « Église » enregistrée", HEURE),
            entree("Requête « Ponts du soir » supprimée", HEURE),
        ]
    );

    let absente = Err(impossible(
        "Aucune requête enregistrée ne porte cet identifiant.".to_string(),
    ));
    assert_eq!(
        services.renommer_une_requete(&renommage(ponts, "Ponts lents"), &mut contexte),
        absente
    );
    assert_eq!(
        services.renommer_une_requete(&renommage(ponts, ""), &mut contexte),
        absente
    );
    assert_eq!(
        services.supprimer_une_requete(&suppression(ponts), &mut contexte),
        absente
    );
    assert_eq!(
        services.enregistrer_une_requete(&enregistrement(" \t ", cherche("pont")), &mut contexte),
        Err(impossible("Le nom est vide.".to_string()))
    );
    assert_eq!(
        services.renommer_une_requete(&renommage(eglise, "Église"), &mut contexte),
        Err(impossible("Le nom est déjà celui-ci.".to_string()))
    );
    assert_eq!(services.journal(), Ok(journal));
    assert_eq!(
        services.etat_du_fonds().map(|etat| etat.a_annuler),
        Ok(Some(geste.to_string()))
    );
    annule(&mut services, &mut contexte, geste);
}

/// Cinq requêtes enregistrées qui posent, à elles toutes, chaque valeur de l'état, de la
/// provenance et du contenu, et chaque autre critère : « Zéphyr » (les non reliées, les importées,
/// une prise), « abeille » (une photo), « Écluse » (un MIDI versé), « Barque » (un texte versé) et
/// « cyprès » (la requête entière, `requete_entiere`), enregistrées dans cet ordre, celui de leurs
/// ULID. La liste rend chacune telle qu'elle est enregistrée, rangée par nom replié : « abeille »,
/// « Barque », « cyprès », « Écluse », « Zéphyr » ; rouverte, de même. Les services, hors de la
/// mutation, convertissent la requête du noyau en celle des commandes : une valeur mal convertie se
/// voit ici.
pub fn une_requete_enregistree_se_relit_critere_pour_critere<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    noter_l_idee(&mut services, "Un pont en mi mineur", &mut contexte);
    let enregistrees = [
        (
            "Zéphyr",
            Requete {
                etat: Some(Etat::NonReliee),
                provenance: Some(ProvenanceDeLIdee::Importee),
                contenu: Some(Contenu::Prise),
                ..Requete::default()
            },
        ),
        (
            "abeille",
            Requete {
                contenu: Some(Contenu::Photo),
                ..Requete::default()
            },
        ),
        (
            "Écluse",
            Requete {
                contenu: Some(Contenu::MidiVerse),
                ..Requete::default()
            },
        ),
        (
            "Barque",
            Requete {
                contenu: Some(Contenu::TexteVerse),
                ..Requete::default()
            },
        ),
        ("cyprès", requete_entiere()),
    ];
    for (nom, requete) in &enregistrees {
        services
            .enregistrer_une_requete(&enregistrement(nom, requete.clone()), &mut contexte)
            .expect("la requête est enregistrée");
    }
    let attendues: Vec<(String, Option<Requete>)> = [1, 3, 4, 2, 0]
        .into_iter()
        .map(|rang| {
            let (nom, requete) = &enregistrees[rang];
            (nom.to_string(), Some(requete.clone()))
        })
        .collect();
    let relues = |services: &Services<P::D, P::C>| -> Vec<(String, Option<Requete>)> {
        les_requetes(services)
            .into_iter()
            .map(|relue| (relue.nom, relue.requete))
            .collect()
    };
    assert_eq!(relues(&services), attendues);

    let services = paire.rouvrir(services, &[]);
    assert_eq!(relues(&services), attendues);
}

/// Dans un Pupitre neuf, la liste des requêtes enregistrées est vide. Enregistrer, même sous un
/// nom vide et sans critère, renommer et supprimer sont refusés, « Aucun fonds n'est né. »
/// (`Raison::Impossible`, question 20), avant tout constat du noyau et toute requête absente, sans
/// rien tirer de l'aléa ; rien ne naît : ni fonds, ni journal, ni requête, et l'état du fonds le
/// dit non né.
pub fn enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    assert_eq!(services.requetes_enregistrees(), Ok(Vec::new()));
    let non_ne = Err(impossible("Aucun fonds n'est né.".to_string()));
    assert_eq!(
        services.enregistrer_une_requete(
            &enregistrement("Ponts lents", cherche("pont")),
            &mut contexte
        ),
        non_ne
    );
    assert_eq!(
        services.enregistrer_une_requete(&enregistrement("", Requete::default()), &mut contexte),
        non_ne
    );
    let ponts = "01HF7YAT000000000000000005";
    assert_eq!(
        services.renommer_une_requete(&renommage(ponts, "Ponts du soir"), &mut contexte),
        non_ne
    );
    assert_eq!(
        services.supprimer_une_requete(&suppression(ponts), &mut contexte),
        non_ne
    );

    assert!(services.documents().fonds().is_none());
    assert_eq!(services.journal(), Ok(Vec::new()));
    assert_eq!(services.requetes_enregistrees(), Ok(Vec::new()));
    assert_eq!(services.etat_du_fonds().map(|etat| etat.ne), Ok(false));
    assert_eq!(alea.appels, 0);
}

/// Le point de revue 2. La note « Un pont en mi mineur » et la requête « Ponts lents » (la requête
/// « pont »), puis le fonds migré vers la version de schéma 2 et rouvert par une application qui
/// ne connaît que la 1, comme dans `une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme` :
/// chercher sert, la boîte sous « pont » trouve la note, et la liste des requêtes enregistrées se
/// lit. Enregistrer, même sous un nom vide, renommer et supprimer, la requête enregistrée comme une
/// requête absente, rendent le refus de `lecture_seule()`, tel quel, avant tout autre refus, sans
/// rien tirer de l'aléa ; ni le journal ni la liste ne changent.
pub fn en_lecture_seule_chercher_sert_et_enregistrer_est_refuse<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let idee = noter_l_idee(
        &mut services,
        "Un pont en mi mineur",
        &mut contexte_de_test(&mut alea),
    );
    services
        .enregistrer_une_requete(
            &enregistrement("Ponts lents", cherche("pont")),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la requête est enregistrée");
    let enregistrees = les_requetes(&services);
    let ponts = enregistrees[0].id.clone();
    let migration = migration_de_test();
    let services = paire.rouvrir(services, &migrations_de_test(&migration));
    let mut services = paire.rouvrir(services, &[]);

    let refus = services
        .documents()
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(refus.raison, Raison::VersionPlusRecente { version: 2 });
    let vue = services
        .boite(&boite_de_la_requete(cherche("pont")))
        .expect("la boîte se lit");
    assert_eq!(idees_des_lignes(&vue.lignes), vec![idee]);
    assert_eq!(vue.retenues, 1);
    assert_eq!(les_requetes(&services), enregistrees);

    let journal = services.journal();
    let absente = "01HF7YAT000000000000000009";
    let mut contexte = contexte_de_test(&mut alea);
    assert_eq!(
        services
            .enregistrer_une_requete(&enregistrement("Église", requete_entiere()), &mut contexte),
        Err(refus.clone())
    );
    assert_eq!(
        services.enregistrer_une_requete(&enregistrement("", Requete::default()), &mut contexte),
        Err(refus.clone())
    );
    assert_eq!(
        services.renommer_une_requete(&renommage(&ponts, "Ponts du soir"), &mut contexte),
        Err(refus.clone())
    );
    assert_eq!(
        services.renommer_une_requete(&renommage(absente, "Ponts du soir"), &mut contexte),
        Err(refus.clone())
    );
    assert_eq!(
        services.supprimer_une_requete(&suppression(&ponts), &mut contexte),
        Err(refus.clone())
    );
    assert_eq!(
        services.supprimer_une_requete(&suppression(absente), &mut contexte),
        Err(refus)
    );
    // Les quatre tirages de la naissance et de la note, puis celui de la requête, et aucun de plus.
    assert_eq!(alea.appels, 5);
    assert_eq!(services.journal(), journal);
    assert_eq!(les_requetes(&services), enregistrees);
}

/// Chaque clé de l'objet de la requête enregistrée `id`, et sa valeur, simple.
fn objet_de_la_requete(doc: &Automerge, id: &str) -> BTreeMap<String, ScalarValue> {
    let objet = objet_au_bout(doc, &[REQUETES_ENREGISTREES, id]);
    doc.keys(&objet)
        .map(|cle| {
            let valeur = match doc.get(&objet, cle.as_str()) {
                Ok(Some((Value::Scalar(simple), _))) => simple.into_owned(),
                autre => panic!("{cle} n'a pas de valeur simple : {autre:?}"),
            };
            (cle, valeur)
        })
        .collect()
}

/// Sur des services neufs de `paire`, sous `AleaDeTest { graine: 0, appels: 0 }`, note chacun de
/// `textes`, puis enregistre `requete` sous le nom « Ponts lents » ; rend les services et l'ULID
/// de la requête enregistrée.
fn la_requete_enregistree_apres<P: Paire>(
    paire: &mut P,
    textes: &[&str],
    requete: &Requete,
) -> (Services<P::D, P::C>, String) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    for texte in textes {
        noter_l_idee(&mut services, texte, &mut contexte);
    }
    services
        .enregistrer_une_requete(
            &enregistrement("Ponts lents", requete.clone()),
            &mut contexte,
        )
        .expect("la requête est enregistrée");
    let id = les_requetes(&services)
        .pop()
        .expect("la requête enregistrée se lit")
        .id;
    (services, id)
}

/// I01 (§ 70 · 3), une requête enregistrée : la même requête (« pont », du 1er au 30 novembre
/// 2023, les non reliées, les captées, le texte de l'idée), enregistrée dans deux fonds aux idées
/// différentes, deux notes sur `paire`, cinq sur `autre_paire`, où elle trouve une idée, puis
/// trois : les clés et les valeurs de son objet sont les mêmes, son ULID à part, et ce sont son nom
/// et les clés de ses critères, ni résultats ni nombre ; `parcourir` ne rend aucun chemin fautif.
pub fn une_requete_enregistree_ne_garde_ni_resultats_ni_nombre<P: Paire>(
    paire: &mut P,
    autre_paire: &mut P,
) {
    let requete = Requete {
        du: Some(Jour {
            annee: 2023,
            mois: 11,
            jour: 1,
        }),
        au: Some(Jour {
            annee: 2023,
            mois: 11,
            jour: 30,
        }),
        etat: Some(Etat::NonReliee),
        provenance: Some(ProvenanceDeLIdee::Captee),
        contenu: Some(Contenu::TexteDeLIdee),
        ..cherche("pont")
    };
    let (deux, id_des_deux) = la_requete_enregistree_apres(
        paire,
        &["Un pont en mi mineur", "Un refrain qui monte, en ré"],
        &requete,
    );
    let (cinq, id_des_cinq) = la_requete_enregistree_apres(
        autre_paire,
        &[
            "Un pont en mi mineur",
            "Un refrain qui monte, en ré",
            "Un pont lent, en la",
            "Une coda sans paroles",
            "Encore un pont, plus court",
        ],
        &requete,
    );
    assert_ne!(id_des_deux, id_des_cinq);
    let retenues = |services: &Services<P::D, P::C>| {
        services
            .boite(&boite_de_la_requete(requete.clone()))
            .expect("la boîte se lit")
            .retenues
    };
    assert_eq!((retenues(&deux), retenues(&cinq)), (1, 3));

    let fonds_des_deux = deux.documents().fonds().expect("le fonds est né");
    let fonds_des_cinq = cinq.documents().fonds().expect("le fonds est né");
    let objet = objet_de_la_requete(fonds_des_deux, &id_des_deux);
    assert_eq!(objet_de_la_requete(fonds_des_cinq, &id_des_cinq), objet);
    assert_eq!(
        objet.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        BTreeSet::from([NOM, TEXTE, DU, AU, ETAT, PROVENANCE, CONTENU])
    );
    assert_eq!(parcourir(fonds_des_deux), Vec::<String>::new());
    assert_eq!(parcourir(fonds_des_cinq), Vec::<String>::new());
}
