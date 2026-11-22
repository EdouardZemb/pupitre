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
//! illisible, les commandes des requêtes enregistrées et la requête entière ; et les aides des
//! tests de la lecture du fonds importé, qui ouvrent un import et y versent par ses opérations
//! nommées, sans commande ; les octets des fichiers à verser, écrits à l'exécution, l'import
//! conduit par les commandes, unité par unité (`importer`), et les aides de ce qui le suit : les
//! déclarations d'une origine, l'origine que montre la vue d'une prise, le type d'un stem et le
//! texte versé relu ; les commandes qui ouvrent et closent un import (`ouverture`, `cloture`), un
//! WAV coupé (`wav_coupe`) et les tests de la reprise. Le fichier s'inclut par `mod suite;`
//! dans les tests des services, et par `#[path = "../../services/tests/suite/mod.rs"]` dans ceux
//! du stockage : il ne nomme que ce que les deux voient, les services, les commandes, le noyau et
//! `automerge`.

// Chaque fichier qui inclut la suite n'en emploie qu'une partie.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{
    ActorId, Automerge, Change, ChangeHash, ObjId, ObjType, Prop, ROOT, ReadDoc, ScalarValue, Value,
};
use commandes::{
    Assiette, Boite, Bornes, Captation, CaptationVue, CloreLImport, Completer, Contenu,
    DeclarationDUnDossier, ENTIER_SUR, Enregistrer, EntreeDuJournal, Etat, EtatDuFonds, Etiqueter,
    FENETRE_MAX, Faite, FaitsDUneCaptation, Fenetre, FichierVerse, ImportClos, Inspection, Issue,
    Jour, LigneDeLaBoite, NatureDuFichier, Notee, Noter, Ordre, OrigineADeclarer, OrigineDeclaree,
    OuvrirLImport, ProvenanceDeLIdee, Raison, RapportDImport, Refus, Regrouper, Relier, Renommer,
    Reponse, Requete, RequeteEnregistree, Sans, Supprimer, TypeVu, VerserUnFichier,
    VerserUnePriseEnStems, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees, VueDuJournal,
};
use noyau::donnees::boite::texte_du_compte;
use noyau::donnees::changement::{Changement, Operation, Simple};
use noyau::donnees::entete::{ENTETE_AU_PLUS, FaitsMidi, FaitsWav};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, APPARENTE, AU, BITS, CANAUX, CODAGE, CONTENEURS_DE_LA_GENESE,
    CONTENEURS_NOMMANT_UNE_CAPTATION, CONTENU, DECLARATION, DEPUIS, DIVISION, DU, DUREE, ETAT,
    ETIQUETTE, ETIQUETTES, FORMAT, FREQUENCE, GENEREE, GENESE, IDEE as IDEE_D_UNE_CAPTATION, IDEES,
    IDENTIFIANT, IMPORTS, JEU, LE, LIENS, LIGNEE, MAIN, MANIFESTE, MESSAGE_DE_LA_GENESE,
    MESSAGE_DE_LA_NAISSANCE, MIDI, MIDI_VERSES, NOM, ORIGINES_DECLAREES, OUVERT, PAR, PAR_LE_NOM,
    PHOTOS, PLUS_DE_SECONDES, PRISES, PROPRIETAIRE, PROVENANCE, RAPPORT, REQUETES_ENREGISTREES,
    RETIREE, STEM, TEXTE, TEXTES_VERSES, TYPE, VERSION_DE_SCHEMA,
};
use noyau::donnees::gestes;
use noyau::donnees::idee::cle_du_lien;
use noyau::donnees::identifiants::{empreinte, ulid};
use noyau::donnees::import::{
    self, ChoixDuType, DejaDeLaPrise, DejaLu, Faits, ImportOuvert, MidiDuStem, StemAVerser,
};
use noyau::donnees::journal::empreinte_du_journal;
use noyau::donnees::origine::{Ajout, Declaration, OriginesLues};
use noyau::donnees::rapport;
use noyau::donnees::requete::texte_de_la_reponse;
use noyau::donnees::signature::{FormatSonore, FormatTexte, SIGNATURE_AU_PLUS};
use noyau::donnees::stems::TypeDeStem;
use noyau::donnees::version::{Migration, message_de_la_migration};
use services::application;
use services::lecture::{
    self, ImportLu, NomsPoses, ProvenanceDuType, StemLu, TypeLu, lire_l_idee, teneur,
    version_de_schema,
};
use services::{Alea, Captations, Contexte, Documents, Services, Source, Versee};

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
    /// La source à verser de la paire (`Source`).
    type S: Source;
    /// Des services sur une racine des données sans document (fichiers), ou sans fonds (mémoire).
    fn neufs(&mut self) -> Services<Self::D, Self::C>;
    /// Ferme `services`, puis les rouvre selon `migrations`.
    fn rouvrir(
        &mut self,
        services: Services<Self::D, Self::C>,
        migrations: &[Migration<'_>],
    ) -> Services<Self::D, Self::C>;
    /// La source à verser du dossier `nom_du_dossier` (`None` : un nom sans forme UTF-8), ses
    /// `fichiers`, (chemin, contenu), un chemin `None` étant un nom sans forme UTF-8, et ses
    /// `liens`, des chemins : en mémoire, ou écrite dans un dossier jetable, puis parcourue.
    fn source(
        &mut self,
        nom_du_dossier: Option<&str>,
        fichiers: &[(Option<&str>, Vec<u8>)],
        liens: &[&str],
    ) -> Self::S;
    /// Change le contenu du fichier `rang` de `source` en `contenu`, et avance sa marque de
    /// dernière écriture (question 14), même à taille égale : seule la marque dit alors le
    /// changement. Ses octets et sa marque au parcours, que rend `entrees`, ne changent pas :
    /// rechoisir le dossier, c'est le parcourir de nouveau, une autre source de ses mêmes chemins.
    fn changer(&mut self, source: &mut Self::S, rang: u64, contenu: &[u8]);
    /// Refuse toute lecture du fichier `rang` de `source` tant que `refusee` est vrai, puis la
    /// rend (question 38 ; V2).
    fn refuser_la_lecture(&mut self, source: &mut Self::S, rang: u64, refusee: bool);
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

/// La commande qui relie ou délie les idées `a` et `b`, chacune au décalage local nul.
pub fn lien(a: &str, b: &str) -> Relier {
    Relier {
        a: a.to_string(),
        b: b.to_string(),
        decalage_minutes: 0,
        decalage_b_minutes: 0,
    }
}

/// La commande qui regroupe la prise `empreinte` dans l'idée `cible`, la cible et la source au
/// décalage local nul.
pub fn regroupement(empreinte: &str, cible: &str) -> Regrouper {
    Regrouper {
        captation: Captation {
            sorte: commandes::Sorte::Prise,
            empreinte: empreinte.to_string(),
        },
        cible: cible.to_string(),
        decalage_minutes: 0,
        decalage_source_minutes: 0,
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
        decalage_source_minutes: 0,
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
/// Les faits d'une captation, sous `<empreinte>/<fait>` : son idée, son format, sa durée, les faits
/// de l'en-tête d'un WAV et d'un MIDI, et la marque d'une prise en stems.
const FAITS_D_UNE_CAPTATION: [&str; 9] = [
    IDEE_D_UNE_CAPTATION,
    FORMAT,
    DUREE,
    CODAGE,
    CANAUX,
    FREQUENCE,
    BITS,
    DIVISION,
    MANIFESTE,
];
/// Les faits d'un stem, sous `<p>/stem/<s>/<fait>` : son format, sa durée et son type.
const FAITS_D_UN_STEM: [&str; 3] = [FORMAT, DUREE, TYPE];
/// Les clés de la provenance du type d'un stem, sous `<p>/stem/<s>/type/<clé>`.
const PROVENANCE_DU_TYPE: [&str; 4] = [PROVENANCE, PAR, DEPUIS, LE];
/// Les faits d'un import, sous `<i>/<fait>` : l'heure de son ouverture et son rapport.
const FAITS_D_UN_IMPORT: [&str; 2] = [OUVERT, RAPPORT];
/// Les deux familles d'une déclaration d'import, sous `<i>/<famille>/<chemin>` : l'origine du
/// dossier et son jeu de stems (balayage Q3).
const DECLARATIONS_D_UN_IMPORT: [&str; 2] = [DECLARATION, JEU];
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

/// Ce qui suit `<mot>/` en tête de `texte` ; `None` si `texte` ne commence pas ainsi.
fn apres<'t>(texte: &'t str, mot: &str) -> Option<&'t str> {
    texte.strip_prefix(mot)?.strip_prefix('/')
}

/// `reste` est `nom/<chemin>`, le chemin non vide.
fn un_nom(reste: &str) -> bool {
    apres(reste, NOM).is_some_and(|chemin| !chemin.is_empty())
}

/// `cle` est une clé d'une captation, sous `prises`, `midi_verses` ou `textes_verses` : `<e>/`
/// suivi d'un fait (`FAITS_D_UNE_CAPTATION`), de `nom/<chemin>`, ou de `stem/` et de ce que
/// `cle_d_un_stem` admet ; l'empreinte non vide.
fn cle_d_une_captation(cle: &str) -> bool {
    let Some((empreinte, reste)) = cle.split_once('/') else {
        return false;
    };
    !empreinte.is_empty()
        && (FAITS_D_UNE_CAPTATION.contains(&reste)
            || un_nom(reste)
            || apres(reste, STEM).is_some_and(cle_d_un_stem))
}

/// `sous_le_stem`, ce qui suit `<p>/stem/`, est `<s>`, l'entrée du stem (balayage Q4), ou `<s>/`
/// suivi d'un fait du stem (`FAITS_D_UN_STEM`), de `type/` et d'une clé de sa provenance
/// (`PROVENANCE_DU_TYPE`), de `nom/<chemin>` ou de `midi/<m>` ; le stem non vide, le MIDI non vide
/// et sans `/`.
fn cle_d_un_stem(sous_le_stem: &str) -> bool {
    let Some((stem, fait)) = sous_le_stem.split_once('/') else {
        return !sous_le_stem.is_empty();
    };
    !stem.is_empty()
        && (FAITS_D_UN_STEM.contains(&fait)
            || apres(fait, TYPE).is_some_and(|cle| PROVENANCE_DU_TYPE.contains(&cle))
            || un_nom(fait)
            || apres(fait, MIDI).is_some_and(|midi| !midi.is_empty() && !midi.contains('/')))
}

/// `cle` est une clé d'un import (balayage Q3) : `<i>/` suivi d'un fait (`FAITS_D_UN_IMPORT`), ou
/// de `declaration/<chemin>` ou de `jeu/<chemin>` (`DECLARATIONS_D_UN_IMPORT`), le chemin vide
/// pour le dossier versé lui-même ; l'ULID non vide.
fn cle_d_un_import(cle: &str) -> bool {
    let Some((import, reste)) = cle.split_once('/') else {
        return false;
    };
    !import.is_empty()
        && (FAITS_D_UN_IMPORT.contains(&reste)
            || DECLARATIONS_D_UN_IMPORT
                .iter()
                .any(|famille| apres(reste, famille).is_some()))
}

/// `cle` est celle d'une origine déclarée : `<e>/main`, ou `<e>/generee/<nom>` ; l'empreinte et le
/// nom non vides.
fn cle_d_une_origine(cle: &str) -> bool {
    let Some((empreinte, reste)) = cle.split_once('/') else {
        return false;
    };
    !empreinte.is_empty()
        && (reste == MAIN || apres(reste, GENEREE).is_some_and(|nom| !nom.is_empty()))
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
/// sous `prises`, `midi_verses` et `textes_verses`, les clés d'une captation
/// (`cle_d_une_captation`) ; sous `liens`, la clé d'un lien ; sous `requetes_enregistrees`, chaque
/// requête enregistrée et ses champs (`CHAMPS_D_UNE_REQUETE`), rien d'autre dessous ; sous
/// `imports`, les clés d'un import (`cle_d_un_import`) ; sous `origines_declarees`, celles d'une
/// origine (`cle_d_une_origine`).
fn permis(chemin: &[String]) -> bool {
    match chemin {
        [cle] => {
            CHAMPS_DE_LA_RACINE.contains(&cle.as_str())
                || CONTENEURS_DE_LA_GENESE.contains(&cle.as_str())
        }
        [conteneur, _] if conteneur == IDEES => true,
        [conteneur, _, champ] if conteneur == IDEES => CHAMPS_D_UNE_IDEE.contains(&champ.as_str()),
        [conteneur, _, sous, _] if conteneur == IDEES && sous == ETIQUETTES => true,
        [conteneur, cle] if CONTENEURS_NOMMANT_UNE_CAPTATION.contains(&conteneur.as_str()) => {
            cle_d_une_captation(cle)
        }
        [conteneur, cle] if conteneur == IMPORTS => cle_d_un_import(cle),
        [conteneur, cle] if conteneur == ORIGINES_DECLAREES => cle_d_une_origine(cle),
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
/// partout, chaque segment lu d'une clé (`segments_lus`), l'un de `INTERDITS_PARTOUT`, une
/// étiquette ou un identifiant de ce nom compris. Un texte libre, que Sacha ou son dossier ont
/// écrit, ne se lit pas : le chemin d'un nom d'origine, celui d'une déclaration ou d'un jeu, et le
/// nom d'un générateur (P4 ; balayage Q3).
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
        || segments_lus(chemin)
            .iter()
            .any(|segment| INTERDITS_PARTOUT.contains(segment))
}

/// Les segments d'un chemin du document qu'`interdit` lit : chaque clé coupée à chaque `/` ; d'une
/// clé à plat, jusqu'au mot qui ouvre un texte libre, lui compris (`debut_du_texte_libre`).
fn segments_lus(chemin: &[String]) -> Vec<&str> {
    match chemin {
        [conteneur, cle] => {
            let segments: Vec<&str> = cle.split('/').collect();
            let fin = debut_du_texte_libre(conteneur, &segments).unwrap_or(segments.len());
            std::iter::once(conteneur.as_str())
                .chain(segments[..fin].iter().copied())
                .collect()
        }
        _ => chemin.iter().flat_map(|cle| cle.split('/')).collect(),
    }
}

/// Le rang du premier segment d'un texte libre dans la clé à plat `segments` du conteneur
/// `conteneur` : sous un conteneur de captations, ce qui suit `nom` au deuxième segment
/// (`<e>/nom/<chemin>`), ou au quatrième sous `stem` (`<p>/stem/<s>/nom/<chemin>`) ; sous
/// `imports`, ce qui suit `declaration` ou `jeu` au deuxième (`<i>/jeu/<chemin>`) ; sous
/// `origines_declarees`, ce qui suit `generee` au deuxième (`<e>/generee/<nom>`). Aucun ailleurs.
fn debut_du_texte_libre(conteneur: &str, segments: &[&str]) -> Option<usize> {
    if CONTENEURS_NOMMANT_UNE_CAPTATION.contains(&conteneur) {
        ouvre(segments, 1, &[NOM]).or_else(|| {
            if segments.get(1) == Some(&STEM) {
                ouvre(segments, 3, &[NOM])
            } else {
                None
            }
        })
    } else if conteneur == IMPORTS {
        ouvre(segments, 1, &DECLARATIONS_D_UN_IMPORT)
    } else if conteneur == ORIGINES_DECLAREES {
        ouvre(segments, 1, &[GENEREE])
    } else {
        None
    }
}

/// Le rang qui suit `rang`, si le segment de ce rang est l'un de `mots`.
fn ouvre(segments: &[&str], rang: usize, mots: &[&str]) -> Option<usize> {
    segments
        .get(rang)
        .filter(|segment| mots.contains(*segment))
        .map(|_| rang + 1)
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
        captations_vues: Vec::new(),
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

/// Un geste qui nomme deux idées : les services passent au noyau le décalage de chacune, celui
/// qui la date, jamais l'un pour l'autre. Deux notes et une prise versée, toutes nées à `HEURE`
/// (22 h 13 UTC, le 14 novembre) : à 120 minutes, une idée est du 15 novembre ; à 0, du 14.
/// Relier à (120, 0), délier à (0, 120), regrouper la prise dans la première note à 120 pour la
/// cible et 0 pour la source qu'il retire : chaque message dit deux jours distincts, si bien qu'un
/// décalage passé deux fois, ou les deux échangés, se voit au journal.
pub fn chaque_idee_d_un_geste_se_date_a_son_propre_decalage<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note est faite");
    services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note est faite");
    let versee = verser_une_prise(
        &mut services,
        PRISE_DE_TEST,
        &fiche_wav(42_000),
        &mut contexte,
    );
    let derniere_entree = |services: &Services<P::D, P::C>| {
        services
            .journal()
            .expect("le journal se lit")
            .last()
            .cloned()
    };

    let relier = Relier {
        decalage_minutes: 120,
        decalage_b_minutes: 0,
        ..lien(IDEE, SECONDE_IDEE)
    };
    services
        .relier(&relier, &mut contexte)
        .expect("les idées se relient");
    assert_eq!(
        derniere_entree(&services),
        Some(entree(
            "Idée du 15 novembre reliée à l'idée du 14 novembre",
            HEURE
        ))
    );

    let delier = Relier {
        decalage_minutes: 0,
        decalage_b_minutes: 120,
        ..lien(IDEE, SECONDE_IDEE)
    };
    services
        .delier(&delier, &mut contexte)
        .expect("les idées se délient");
    assert_eq!(
        derniere_entree(&services),
        Some(entree(
            "Idée du 14 novembre déliée de l'idée du 15 novembre",
            HEURE
        ))
    );

    let regrouper = Regrouper {
        decalage_minutes: 120,
        decalage_source_minutes: 0,
        ..regroupement(&versee.empreinte, IDEE)
    };
    services
        .regrouper(&regrouper, &mut contexte)
        .expect("la prise se regroupe");
    assert_eq!(
        derniere_entree(&services),
        Some(entree(
            "Idée du 15 novembre : prise regroupée, idée du 14 novembre retirée",
            HEURE
        ))
    );
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
        captations_vues: vec![CaptationVue {
            captation: Captation {
                sorte: commandes::Sorte::Prise,
                empreinte: v.empreinte.clone(),
            },
            faits: FaitsDUneCaptation {
                format: Some("wav".to_string()),
                duree_ms: Some(30_000),
                codage: None,
                canaux: None,
                frequence: None,
                bits: None,
                division: None,
            },
            noms: Vec::new(),
            origine: OrigineDeclaree::Aucune,
            stems: Vec::new(),
        }],
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
        Err(impossible("Cette captation est déjà versée.".to_string()))
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
        Err(impossible(
            "Cette captation n'est pas dans le fonds.".to_string()
        ))
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
        decalage_source_minutes: 0,
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
            nom_de_fichier,
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
        assert_eq!(nom_de_fichier, None);
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
        nom_de_fichier: None,
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

/// Un texte de 250 `é`, puis « pont » : la ligne de la boîte en porte les 200 premiers caractères,
/// et la vue de l'idée le texte entier. La recherche lit tout le texte : « pont », placé après le
/// 200ᵉ caractère, trouve l'idée, et un index bâti des textes coupés se verrait.
pub fn une_ligne_borne_son_texte<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let long = format!("{} pont", "é".repeat(250));
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

    let pont = services
        .boite(&boite_de_la_requete(cherche("pont")))
        .expect("la boîte se lit");
    assert_eq!(idees_des_lignes(&pont.lignes), vec![idee]);
    assert_eq!(pont.retenues, 1);
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
/// Les bornes de `HEURE + 60`, comprise, à `HEURE + 180`, exclue, ne trouvent, par date, que C et
/// B : A, née avant, D, née à la borne exclue, et E, née après, n'y sont pas ; des bornes
/// échangées, perdues ou réduites à l'une d'elles, dans leur conversion vers le noyau, se voient.
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

    let periode = services
        .boite(&Boite {
            bornes: Bornes {
                des: Some(HEURE + 60),
                avant: Some(HEURE + 180),
            },
            ..toute_la_boite()
        })
        .expect("la boîte se lit");
    assert_eq!(
        idees_des_lignes(&periode.lignes),
        vec![c.clone(), b.clone()]
    );
    assert_eq!(periode.retenues, 2);

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

/// Le journal en fenêtre (question 29) : un fonds de trois notes, chacune un jour après la
/// précédente, porte cinq entrées : la genèse, la naissance et les trois notes. La fenêtre (0, 2)
/// en rend le nombre, deux lignes, la plus récente d'abord, et l'empreinte que le noyau calcule
/// (`empreinte_du_journal`) sur le journal entier, tel que `Services::journal` le rend ; la
/// fenêtre (4, 200), la seule genèse, sous le même nombre et la même empreinte.
pub fn le_journal_se_lit_en_fenetre<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    for (texte, heure) in [
        ("Un refrain qui monte, en ré", HEURE),
        ("Un pont en mi mineur", LENDEMAIN),
        ("Une coda sans paroles", LENDEMAIN + 86_400),
    ] {
        let mut contexte = Contexte {
            heure,
            alea: &mut alea,
        };
        services
            .noter(&une_note(texte), &mut contexte)
            .expect("la note est faite");
    }
    let entrees: Vec<(String, i64)> = services
        .journal()
        .expect("le journal se lit")
        .into_iter()
        .map(|entree| (entree.message, entree.heure))
        .collect();
    let empreinte = empreinte_du_journal(&entrees);
    assert_eq!(
        services.journal_en_fenetre(&Fenetre {
            debut: 0,
            nombre: 2,
        }),
        Ok(VueDuJournal {
            total: 5,
            empreinte: empreinte.clone(),
            lignes: vec![
                entree("Idée du 16 novembre notée", LENDEMAIN + 86_400),
                entree("Idée du 15 novembre notée", LENDEMAIN),
            ],
        })
    );
    assert_eq!(
        services.journal_en_fenetre(&Fenetre {
            debut: 4,
            nombre: 200,
        }),
        Ok(VueDuJournal {
            total: 5,
            empreinte,
            lignes: vec![entree("Genèse du fonds", 0)],
        })
    );
}

/// Le second import des tests de la lecture du fonds importé : `ulid(LENDEMAIN, [0x22; 10])`, ouvert
/// au lendemain, après `IMPORT`.
fn second_import() -> String {
    ulid(LENDEMAIN, [0x22; 10])
}

/// L'idée de rang `n` qu'un versement des tests de l'import fait naître : un ULID écrit, jamais
/// tiré.
fn idee_importee(n: usize) -> String {
    format!("01HF7YAT00000000000000{n:04}")
}

/// Les faits d'un WAV lisible de `trames` trames : PCM, mono, 8 000 Hz, 8 bits ; 8 000 trames
/// durent 1 000 ms.
fn faits_d_un_wav(trames: u64) -> Faits {
    Faits::du_wav(&FaitsWav {
        codage: 1,
        canaux: 1,
        frequence: 8_000,
        bits: 8,
        trames,
    })
}

/// Les faits d'un MIDI lisible : format 1, deux pistes, 480 divisions.
fn faits_d_un_midi() -> Faits {
    Faits::du_midi(&FaitsMidi {
        format: 1,
        pistes: 2,
        division: 480,
    })
}

/// Les faits d'un stem WAV lisible de `duree_ms` millisecondes, tels que `lecture::stems_de` les
/// lit : son format et sa durée, seuls.
fn faits_d_un_stem(duree_ms: u64) -> Faits {
    Faits {
        format: Some(FormatSonore::Wav.nom().to_string()),
        duree_ms: Some(duree_ms),
        ..Faits::default()
    }
}

/// Le fonds de `services`, qui doit être né.
fn fonds_de<D: Documents, C: Captations>(services: &Services<D, C>) -> &Automerge {
    services.documents().fonds().expect("le fonds est né")
}

/// Écrit `operation` par les documents de `services`, sans commande : elle fait naître le fonds
/// s'il n'est pas né, après la naissance des tests ; faite, sans avertissement.
fn ecrire_l_operation<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    operation: &Operation,
) {
    let ecrite = if services.documents().fonds().is_none() {
        let naissance = gestes::naissance(
            ULID_DU_DOCUMENT,
            ULID_DE_LA_LIGNEE,
            IDENTIFIANT_DE_PROPRIETAIRE,
            HEURE,
        );
        services.documents_mut().faire_naitre(&naissance, operation)
    } else {
        services.documents_mut().appliquer(operation)
    };
    assert_eq!(ecrite, Ok(None), "« {} »", operation.message);
}

/// L'import `id`, ouvert à `heure`, à l'établi.
fn ouvert_a(id: &str, heure: i64) -> ImportOuvert<'_> {
    ImportOuvert {
        import: id,
        ouvert_le: heure,
        par: gestes::Appareil::Etabli,
    }
}

/// Ouvre l'import `ouvert`, ses dossiers déclarés `declarations`, au décalage local nul.
fn ouvrir_l_import<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    ouvert: &ImportOuvert<'_>,
    declarations: &[(&str, Declaration)],
) {
    let declarations: BTreeMap<String, Declaration> = declarations
        .iter()
        .map(|(chemin, declaration)| (chemin.to_string(), declaration.clone()))
        .collect();
    let ouverture = import::ouvrir(ouvert.import, &declarations, ouvert.ouvert_le, 0)
        .expect("l'import s'ouvre");
    ecrire_l_operation(services, &ouverture);
}

/// Verse par l'import `ouvert` le fichier `empreinte` de la sorte `sorte`, ses faits `faits`, son
/// chemin `chemin` et son origine `origine`, ce que le fonds en porte lu par `lecture::deja_lu`,
/// à l'heure de l'ouverture ; un contenu neuf fait naître l'idée `idee`. Vrai si quelque chose
/// s'écrit.
// Huit arguments : les services, l'import, puis chaque donnée du versement.
#[allow(clippy::too_many_arguments)]
fn verser_le_fichier<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    ouvert: &ImportOuvert<'_>,
    sorte: gestes::Sorte,
    empreinte: &str,
    faits: &Faits,
    chemin: &str,
    origine: Option<&Ajout>,
    idee: &str,
) -> bool {
    let fonds = fonds_de(services);
    let deja = lecture::deja_lu(fonds, sorte, empreinte, &lecture::empreintes_nommees(fonds));
    let versement = import::verser_un_fichier(
        ouvert,
        sorte,
        empreinte,
        faits,
        Some(chemin),
        origine,
        &deja,
        idee,
        ouvert.ouvert_le,
        0,
    )
    .expect("le fichier se verse");
    let ecrit = versement.is_some();
    if let Some(versement) = versement {
        ecrire_l_operation(services, &versement);
    }
    ecrit
}

/// Verse par l'import `ouvert` le jeu `stems`, sous le nom `nom` et l'origine `origine`, ce que le
/// fonds porte de la prise lu par `lecture::deja_de_la_prise`, à l'heure de l'ouverture ; une prise
/// neuve fait naître l'idée `idee`. Rend l'empreinte de son manifeste, et vrai si quelque chose
/// s'écrit.
fn verser_le_jeu<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    ouvert: &ImportOuvert<'_>,
    nom: &str,
    stems: &[StemAVerser<'_>],
    origine: Option<&Ajout>,
    idee: &str,
) -> (String, bool) {
    let empreintes: Vec<&str> = stems.iter().map(|stem| stem.empreinte).collect();
    let midis: Vec<&str> = stems
        .iter()
        .filter_map(|stem| stem.midi.as_ref().map(|midi| midi.empreinte))
        .collect();
    let manifeste = empreinte(&rapport::manifeste(&empreintes));
    let deja = lecture::deja_de_la_prise(fonds_de(services), &manifeste, &empreintes, &midis);
    let versement = import::verser_une_prise_en_stems(
        ouvert,
        &manifeste,
        Some(nom),
        stems,
        origine,
        &deja,
        idee,
        ouvert.ouvert_le,
        0,
    )
    .expect("le jeu se verse");
    let ecrit = versement.is_some();
    if let Some(versement) = versement {
        ecrire_l_operation(services, &versement);
    }
    (manifeste, ecrit)
}

/// Les noms `noms`, chacun (chemin, ULID de l'import), tels que `lecture::noms_de` les rend.
fn noms_lus(noms: &[(&str, &str)]) -> Vec<(String, String)> {
    noms.iter()
        .map(|(chemin, ulid)| (chemin.to_string(), ulid.to_string()))
        .collect()
}

/// I01 sur ce que l'import écrit (§ 70 · 3 ; balayage Q3, Q4) : un import, ouvert sur deux
/// dossiers déclarés, `total` de la main de Sacha et `compte` à la fois jeu de stems et
/// `generee:total`, verse un fichier de chaque sorte de `total/`, puis le jeu `compte`, l'entrée
/// de chacun de ses deux stems comprise, dont le nom d'origine `compte/a_envoyer.wav` et un MIDI ;
/// le type proposé du premier stem est gardé, et l'import se clôt sur son rapport. `compte`,
/// `total` et `a_envoyer` n'y sont que des textes libres, chemins, nom de jeu ou de générateur :
/// `parcourir` ne refuse rien, et `lecture::stems_de` lit les deux stems, le type gardé avec sa
/// provenance. Sur une copie du fonds, une clé `prises/<e>/total` et une prise d'empreinte
/// `total`, plantées, sont les deux chemins qu'il rend.
pub fn un_fonds_importe_ne_garde_ni_compte_ni_total<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let ouvert = ouvert_a(IMPORT, HEURE);
    let total = Ajout::Generee("total".to_string());
    ouvrir_l_import(
        &mut services,
        &ouvert,
        &[
            (
                "compte",
                Declaration {
                    stems: true,
                    origine: Some(total.clone()),
                },
            ),
            (
                "total",
                Declaration {
                    stems: false,
                    origine: Some(Ajout::Main),
                },
            ),
        ],
    );
    let prise = empreinte(b"une prise de total");
    let midi_seul = empreinte(b"un midi de total");
    let texte = empreinte(b"un texte de total");
    let fichiers = [
        (
            gestes::Sorte::Prise,
            &prise,
            faits_d_un_wav(8_000),
            "total/prise.wav",
            21,
        ),
        (
            gestes::Sorte::MidiVerseSeul,
            &midi_seul,
            faits_d_un_midi(),
            "total/air.mid",
            22,
        ),
        (
            gestes::Sorte::TexteVerse,
            &texte,
            Faits::d_un_texte(FormatTexte::Brut),
            "total/notes.txt",
            23,
        ),
    ];
    for (sorte, e, faits, chemin, rang) in &fichiers {
        assert!(
            verser_le_fichier(
                &mut services,
                &ouvert,
                *sorte,
                e,
                faits,
                chemin,
                Some(&Ajout::Main),
                &idee_importee(*rang),
            ),
            "« {chemin} » ne s'écrit pas"
        );
    }
    let voix = empreinte(b"un stem de voix de compte");
    let basse = empreinte(b"un stem de basse de compte");
    let midi_de_la_basse = empreinte(b"le midi de la basse de compte");
    let faits_de_la_voix = faits_d_un_wav(8_000);
    let faits_de_la_basse = faits_d_un_wav(12_000);
    let faits_du_midi = faits_d_un_midi();
    let stems = [
        StemAVerser {
            empreinte: &voix,
            faits: &faits_de_la_voix,
            chemin: Some("compte/a_envoyer.wav"),
            midi: None,
        },
        StemAVerser {
            empreinte: &basse,
            faits: &faits_de_la_basse,
            chemin: Some("compte/basse.wav"),
            midi: Some(MidiDuStem {
                empreinte: &midi_de_la_basse,
                faits: &faits_du_midi,
                chemin: Some("compte/basse.mid"),
            }),
        },
    ];
    let (manifeste, versee) = verser_le_jeu(
        &mut services,
        &ouvert,
        "compte",
        &stems,
        Some(&total),
        &idee_importee(24),
    );
    assert!(versee, "le jeu « compte » ne s'écrit pas");
    let garde = import::choisir_le_type(
        &manifeste,
        &voix,
        true,
        TypeDeStem::Autre,
        ChoixDuType::Garder,
        Some("a_envoyer.wav"),
        HEURE,
    )
    .expect("le type proposé se garde");
    ecrire_l_operation(&mut services, &garde);
    let le_rapport = empreinte(b"le rapport de l'import de total");
    ecrire_l_operation(
        &mut services,
        &import::clore(&ouvert, &le_rapport, HEURE, 0),
    );

    let fonds = fonds_de(&services);
    assert_eq!(parcourir(fonds), Vec::<String>::new());
    let mut stems_lus = vec![
        StemLu {
            empreinte: voix.clone(),
            faits: faits_d_un_stem(1_000),
            noms: noms_lus(&[("compte/a_envoyer.wav", IMPORT)]),
            type_de_stem: Some(TypeLu {
                type_de_stem: TypeDeStem::Autre,
                provenance: ProvenanceDuType::Garde {
                    par: Some(PAR_LE_NOM.to_string()),
                    depuis: Some(voix.clone()),
                },
                le: Some(HEURE),
            }),
            midis: BTreeSet::new(),
        },
        StemLu {
            empreinte: basse.clone(),
            faits: faits_d_un_stem(1_500),
            noms: noms_lus(&[("compte/basse.wav", IMPORT)]),
            type_de_stem: None,
            midis: BTreeSet::from([midi_de_la_basse.clone()]),
        },
    ];
    stems_lus.sort_by_key(|stem| stem.empreinte.clone());
    assert_eq!(lecture::stems_de(fonds, &manifeste), stems_lus);

    let mut doc = fonds.clone();
    let prises_du_document = objet_au_bout(&doc, &[PRISES]);
    let mut tx = doc.transaction();
    tx.put(&prises_du_document, format!("{prise}/total"), true)
        .expect("poser un total sous la prise");
    tx.put(
        &prises_du_document,
        format!("total/{IDEE_D_UNE_CAPTATION}"),
        IDEE,
    )
    .expect("poser une prise d'empreinte total");
    let (plantation, _) = tx.commit();
    plantation.expect("la plantation est une modification");
    assert_eq!(
        parcourir(&doc),
        vec![
            format!("{PRISES}/{prise}/total"),
            format!("{PRISES}/total/{IDEE_D_UNE_CAPTATION}"),
        ]
    );
}

/// La durée d'une prise en stems (CAP-02 ; balayage Q4 ; question 21 ; R11) : un import verse la
/// prise en stems P1, de deux stems, de 1 000 ms, que suit un MIDI, et de 1 500 ms ; P2, de deux
/// stems, de 2 000 ms et un FLAC sans `duree` ; puis un MIDI versé seul. La vue de l'idée de P1
/// porte 1 500 ms, et P1 pour seule captation : le MIDI d'un stem n'est pas une captation de
/// l'idée ; celle de P2 ne porte aucune durée. Une requête au contenu « MIDI versé » ne rend que
/// l'idée du MIDI versé seul, et non celle de P1, dont un stem porte un MIDI.
pub fn la_duree_d_une_prise_en_stems_vient_de_ses_stems<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let ouvert = ouvert_a(IMPORT, HEURE);
    ouvrir_l_import(&mut services, &ouvert, &[]);
    let s1 = empreinte(b"un stem de 1 000 ms");
    let s2 = empreinte(b"un stem de 1 500 ms");
    let s3 = empreinte(b"un stem de 2 000 ms");
    let s4 = empreinte(b"un stem flac sans duree");
    let midi_de_s1 = empreinte(b"le midi du stem de 1 000 ms");
    let midi_seul = empreinte(b"un midi verse seul");
    let (f1, f2, f3) = (
        faits_d_un_wav(8_000),
        faits_d_un_wav(12_000),
        faits_d_un_wav(16_000),
    );
    let sans_duree = Faits::d_un_audio(FormatSonore::Flac);
    let faits_du_midi = faits_d_un_midi();
    let (p1, versee) = verser_le_jeu(
        &mut services,
        &ouvert,
        "Jeu A",
        &[
            StemAVerser {
                empreinte: &s1,
                faits: &f1,
                chemin: Some("Jeu A/voix.wav"),
                midi: Some(MidiDuStem {
                    empreinte: &midi_de_s1,
                    faits: &faits_du_midi,
                    chemin: Some("Jeu A/voix.mid"),
                }),
            },
            StemAVerser {
                empreinte: &s2,
                faits: &f2,
                chemin: Some("Jeu A/basse.wav"),
                midi: None,
            },
        ],
        None,
        &idee_importee(41),
    );
    assert!(versee, "P1 ne s'écrit pas");
    let (p2, versee) = verser_le_jeu(
        &mut services,
        &ouvert,
        "Jeu B",
        &[
            StemAVerser {
                empreinte: &s3,
                faits: &f3,
                chemin: Some("Jeu B/voix.wav"),
                midi: None,
            },
            StemAVerser {
                empreinte: &s4,
                faits: &sans_duree,
                chemin: Some("Jeu B/basse.flac"),
                midi: None,
            },
        ],
        None,
        &idee_importee(42),
    );
    assert!(versee, "P2 ne s'écrit pas");
    assert!(
        verser_le_fichier(
            &mut services,
            &ouvert,
            gestes::Sorte::MidiVerseSeul,
            &midi_seul,
            &faits_du_midi,
            "air.mid",
            None,
            &idee_importee(43),
        ),
        "le MIDI versé seul ne s'écrit pas"
    );

    let vue = services
        .vue_d_une_idee(&idee_importee(41))
        .expect("la vue de l'idée de P1 se lit");
    assert_eq!(vue.duree_ms, Some(1_500));
    assert_eq!(vue.captations, prises(&[p1.as_str()]));
    let vue = services
        .vue_d_une_idee(&idee_importee(42))
        .expect("la vue de l'idée de P2 se lit");
    assert_eq!(vue.duree_ms, None);
    assert_eq!(vue.captations, prises(&[p2.as_str()]));
    let midis_verses = services
        .boite(&boite_de_la_requete(Requete {
            contenu: Some(Contenu::MidiVerse),
            ..Requete::default()
        }))
        .expect("la boîte se lit");
    assert_eq!(
        idees_des_lignes(&midis_verses.lignes),
        vec![idee_importee(43)]
    );
}

/// Les noms d'un contenu et son premier nom vu (question 20) : le premier import verse la prise X
/// sous `C/prise.wav`, puis sous `B/copie.wav`, qui n'ajoute qu'un nom ; le second, au lendemain,
/// sous `A/prise.wav`, générée par « Atelier inventé ». `noms_de` rend les trois noms dans l'ordre
/// de leurs chemins, chacun avec l'import qui l'a vu ; le premier vu est `B/copie.wav`, le plus
/// petit chemin du premier import. `deja_lu`, `origines_de`, `origines_a` à la tête d'avant le
/// second import, `empreintes_nommees_a`, `generateurs` et `imports` lisent ce fonds.
pub fn les_noms_d_un_contenu_et_son_premier_nom_vu<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let premier = ouvert_a(IMPORT, HEURE);
    ouvrir_l_import(&mut services, &premier, &[]);
    let a_l_ouverture = fonds_de(&services).get_heads();
    let x = empreinte(b"une prise aux trois noms");
    let faits = faits_d_un_wav(8_000);
    for (chemin, rang) in [("C/prise.wav", 61), ("B/copie.wav", 62)] {
        assert!(
            verser_le_fichier(
                &mut services,
                &premier,
                gestes::Sorte::Prise,
                &x,
                &faits,
                chemin,
                None,
                &idee_importee(rang),
            ),
            "« {chemin} » ne s'écrit pas"
        );
    }
    let avant_le_second = fonds_de(&services).get_heads();
    let second = second_import();
    let ouvert = ouvert_a(&second, LENDEMAIN);
    let atelier = Ajout::Generee("Atelier inventé".to_string());
    ouvrir_l_import(
        &mut services,
        &ouvert,
        &[(
            "A",
            Declaration {
                stems: false,
                origine: Some(atelier.clone()),
            },
        )],
    );
    assert!(
        verser_le_fichier(
            &mut services,
            &ouvert,
            gestes::Sorte::Prise,
            &x,
            &faits,
            "A/prise.wav",
            Some(&atelier),
            &idee_importee(63),
        ),
        "« A/prise.wav » ne s'écrit pas"
    );

    let fonds = fonds_de(&services);
    assert_eq!(
        lecture::noms_de(fonds, PRISES, &x, None),
        noms_lus(&[
            ("A/prise.wav", second.as_str()),
            ("B/copie.wav", IMPORT),
            ("C/prise.wav", IMPORT),
        ])
    );
    assert_eq!(
        lecture::premier_nom_vu(fonds, &x),
        Some("B/copie.wav".to_string())
    );
    let de_l_atelier = OriginesLues {
        main: false,
        generees: BTreeSet::from(["Atelier inventé".to_string()]),
    };
    assert_eq!(lecture::origines_de(fonds, &x), de_l_atelier);
    assert_eq!(
        lecture::origines_a(fonds, &x, &avant_le_second),
        OriginesLues::default()
    );
    assert_eq!(
        lecture::empreintes_nommees_a(fonds, &a_l_ouverture),
        BTreeSet::new()
    );
    assert_eq!(
        lecture::empreintes_nommees_a(fonds, &avant_le_second),
        BTreeSet::from([x.clone()])
    );
    assert_eq!(
        lecture::deja_lu(
            fonds,
            gestes::Sorte::Prise,
            &x,
            &lecture::empreintes_nommees(fonds)
        ),
        DejaLu {
            captation: true,
            noms: BTreeSet::from([
                "A/prise.wav".to_string(),
                "B/copie.wav".to_string(),
                "C/prise.wav".to_string(),
            ]),
            origines: de_l_atelier.clone(),
        }
    );
    assert_eq!(
        lecture::deja_lu(fonds, gestes::Sorte::MidiVerseSeul, &x, &BTreeSet::new()),
        DejaLu {
            captation: false,
            noms: BTreeSet::new(),
            origines: de_l_atelier,
        }
    );
    assert_eq!(
        lecture::generateurs(fonds),
        BTreeSet::from(["Atelier inventé".to_string()])
    );
    assert_eq!(
        lecture::imports(fonds),
        vec![
            ImportLu {
                import: IMPORT.to_string(),
                ouvert_le: HEURE,
                rapport: None,
            },
            ImportLu {
                import: second,
                ouvert_le: LENDEMAIN,
                rapport: None,
            },
        ]
    );
}

/// Les noms qu'a posés un import (R2, R7 ; question 9 ; balayage Q3) : le premier, ouvert sur
/// `Démos`, de la main de Sacha, et `Jeu A`, jeu de stems généré par « Atelier inventé », verse de
/// `Démos/` un WAV, un WAV illisible et un texte, puis le jeu `Jeu A`, une voix que suit un MIDI et
/// une basse illisible ; `deja_de_la_prise` ne lit rien de la prise avant lui, et tout après.
/// L'import clos, `noms_poses_par` rend les trois fichiers, les deux stems et le MIDI, chacun avec
/// son empreinte, les deux illisibles marqués, et le jeu à part. Le second, au lendemain, ouvert
/// sur `Jeu A` seul, déclaré jeu sans origine, reverse le même dossier : rien ne s'écrit après son
/// ouverture, et il n'a posé aucun nom. `declarations_de` lit les déclarations de chacun ;
/// `dernieres_declarations`, `Démos` du premier et `Jeu A` entière du second ; `imports`, le
/// premier clos sur son rapport.
pub fn les_noms_poses_par_un_import<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let premier = ouvert_a(IMPORT, HEURE);
    let atelier = Ajout::Generee("Atelier inventé".to_string());
    let demos = Declaration {
        stems: false,
        origine: Some(Ajout::Main),
    };
    let jeu_de_l_atelier = Declaration {
        stems: true,
        origine: Some(atelier.clone()),
    };
    ouvrir_l_import(
        &mut services,
        &premier,
        &[
            ("Démos", demos.clone()),
            ("Jeu A", jeu_de_l_atelier.clone()),
        ],
    );
    let prise = empreinte(b"une prise des demos");
    let bruit = empreinte(b"un wav illisible des demos");
    let texte = empreinte(b"un texte des demos");
    let lisible = faits_d_un_wav(8_000);
    let illisible = Faits::default();
    let brut = Faits::d_un_texte(FormatTexte::Brut);
    let fichiers = [
        (
            gestes::Sorte::Prise,
            &prise,
            &lisible,
            "Démos/prise.wav",
            71,
        ),
        (
            gestes::Sorte::Prise,
            &bruit,
            &illisible,
            "Démos/bruit.wav",
            72,
        ),
        (
            gestes::Sorte::TexteVerse,
            &texte,
            &brut,
            "Démos/notes.txt",
            73,
        ),
    ];
    for (sorte, e, faits, chemin, rang) in fichiers {
        assert!(
            verser_le_fichier(
                &mut services,
                &premier,
                sorte,
                e,
                faits,
                chemin,
                Some(&Ajout::Main),
                &idee_importee(rang),
            ),
            "« {chemin} » ne s'écrit pas"
        );
    }
    let voix = empreinte(b"la voix du jeu A");
    let basse = empreinte(b"la basse illisible du jeu A");
    let midi = empreinte(b"le midi de la voix du jeu A");
    let faits_du_midi = faits_d_un_midi();
    let stems = [
        StemAVerser {
            empreinte: &voix,
            faits: &lisible,
            chemin: Some("Jeu A/voix.wav"),
            midi: Some(MidiDuStem {
                empreinte: &midi,
                faits: &faits_du_midi,
                chemin: Some("Jeu A/voix.mid"),
            }),
        },
        StemAVerser {
            empreinte: &basse,
            faits: &illisible,
            chemin: Some("Jeu A/basse.wav"),
            midi: None,
        },
    ];
    let manifeste = empreinte(&rapport::manifeste(&[voix.as_str(), basse.as_str()]));
    let deja = |services: &Services<P::D, P::C>| {
        lecture::deja_de_la_prise(
            fonds_de(services),
            &manifeste,
            &[voix.as_str(), basse.as_str()],
            &[midi.as_str()],
        )
    };
    assert_eq!(
        deja(&services),
        DejaDeLaPrise {
            prise: DejaLu::default(),
            stems: BTreeMap::from([
                (voix.clone(), DejaLu::default()),
                (basse.clone(), DejaLu::default()),
            ]),
            midis: BTreeMap::from([(midi.clone(), DejaLu::default())]),
            liens_midi: BTreeSet::new(),
        }
    );
    let (verse, versee) = verser_le_jeu(
        &mut services,
        &premier,
        "Jeu A",
        &stems,
        Some(&atelier),
        &idee_importee(74),
    );
    assert!(versee, "le jeu « Jeu A » ne s'écrit pas");
    assert_eq!(verse, manifeste);
    let de_l_atelier = OriginesLues {
        main: false,
        generees: BTreeSet::from(["Atelier inventé".to_string()]),
    };
    let porte = |chemin: &str| DejaLu {
        captation: true,
        noms: BTreeSet::from([chemin.to_string()]),
        origines: de_l_atelier.clone(),
    };
    assert_eq!(
        deja(&services),
        DejaDeLaPrise {
            prise: porte("Jeu A"),
            stems: BTreeMap::from([
                (voix.clone(), porte("Jeu A/voix.wav")),
                (basse.clone(), porte("Jeu A/basse.wav")),
            ]),
            midis: BTreeMap::from([(midi.clone(), porte("Jeu A/voix.mid"))]),
            liens_midi: BTreeSet::from([(voix.clone(), midi.clone())]),
        }
    );
    let le_rapport = empreinte(b"le rapport du premier import");
    ecrire_l_operation(
        &mut services,
        &import::clore(&premier, &le_rapport, HEURE, 0),
    );
    let poses = NomsPoses {
        fichiers: BTreeMap::from([
            ("Démos/prise.wav".to_string(), (prise.clone(), false)),
            ("Démos/bruit.wav".to_string(), (bruit.clone(), true)),
            ("Démos/notes.txt".to_string(), (texte.clone(), false)),
            ("Jeu A/voix.wav".to_string(), (voix.clone(), false)),
            ("Jeu A/basse.wav".to_string(), (basse.clone(), true)),
            ("Jeu A/voix.mid".to_string(), (midi.clone(), false)),
        ]),
        jeux: BTreeSet::from(["Jeu A".to_string()]),
    };
    assert_eq!(lecture::noms_poses_par(fonds_de(&services), IMPORT), poses);

    let second = second_import();
    let ouvert = ouvert_a(&second, LENDEMAIN);
    let jeu_sans_origine = Declaration {
        stems: true,
        origine: None,
    };
    ouvrir_l_import(
        &mut services,
        &ouvert,
        &[("Jeu A", jeu_sans_origine.clone())],
    );
    let apres_l_ouverture = fonds_de(&services).get_heads();
    for (sorte, e, faits, chemin, rang) in fichiers {
        assert!(
            !verser_le_fichier(
                &mut services,
                &ouvert,
                sorte,
                e,
                faits,
                chemin,
                None,
                &idee_importee(rang + 10),
            ),
            "« {chemin} » s'écrit de nouveau"
        );
    }
    let (_, reversee) = verser_le_jeu(
        &mut services,
        &ouvert,
        "Jeu A",
        &stems,
        None,
        &idee_importee(84),
    );
    assert!(!reversee, "le jeu « Jeu A » s'écrit de nouveau");

    let fonds = fonds_de(&services);
    assert_eq!(fonds.get_heads(), apres_l_ouverture);
    assert_eq!(
        lecture::noms_poses_par(fonds, &second),
        NomsPoses::default()
    );
    assert_eq!(lecture::noms_poses_par(fonds, IMPORT), poses);
    assert_eq!(
        lecture::declarations_de(fonds, IMPORT),
        BTreeMap::from([
            ("Démos".to_string(), demos.clone()),
            ("Jeu A".to_string(), jeu_de_l_atelier),
        ])
    );
    assert_eq!(
        lecture::declarations_de(fonds, &second),
        BTreeMap::from([("Jeu A".to_string(), jeu_sans_origine.clone())])
    );
    assert_eq!(
        lecture::dernieres_declarations(fonds),
        BTreeMap::from([
            ("Démos".to_string(), demos),
            ("Jeu A".to_string(), jeu_sans_origine),
        ])
    );
    assert_eq!(
        lecture::imports(fonds),
        vec![
            ImportLu {
                import: IMPORT.to_string(),
                ouvert_le: HEURE,
                rapport: Some(le_rapport),
            },
            ImportLu {
                import: second,
                ouvert_le: LENDEMAIN,
                rapport: None,
            },
        ]
    );
    assert_eq!(parcourir(fonds), Vec::<String>::new());
}

/// Un nom posé hors du jeu n'est pas parmi ceux du stem (balayage Q19) : la prise en stems P nomme
/// son stem S `Jeu A/voix.wav` ; un second import verse le même contenu seul, sous
/// `Voix seule.wav` (`import::verser_un_fichier`), que le fonds nomme déjà (`DejaLu::captation`
/// vrai) : il reçoit `prises/<S>/nom/Voix seule.wav`, sans `idee`, et aucune idée ne naît. Le
/// `StemLu` de S que rend `stems_de(P)` ne porte que `Jeu A/voix.wav`, et
/// `noms_de(doc, PRISES, S, None)` rend `Voix seule.wav` ; le premier nom vu de S est celui du
/// jeu ; `noms_poses_par` rend à chaque import le sien, S lisible par le format de son stem.
pub fn un_nom_pose_hors_du_jeu_n_est_pas_parmi_ceux_du_stem<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let premier = ouvert_a(IMPORT, HEURE);
    ouvrir_l_import(
        &mut services,
        &premier,
        &[(
            "Jeu A",
            Declaration {
                stems: true,
                origine: None,
            },
        )],
    );
    let s = empreinte(b"une voix du jeu A, versee aussi seule");
    let faits = faits_d_un_wav(8_000);
    let (p, versee) = verser_le_jeu(
        &mut services,
        &premier,
        "Jeu A",
        &[StemAVerser {
            empreinte: &s,
            faits: &faits,
            chemin: Some("Jeu A/voix.wav"),
            midi: None,
        }],
        None,
        &idee_importee(81),
    );
    assert!(versee, "le jeu « Jeu A » ne s'écrit pas");
    let second = second_import();
    let ouvert = ouvert_a(&second, LENDEMAIN);
    ouvrir_l_import(&mut services, &ouvert, &[]);
    let fonds = fonds_de(&services);
    assert!(
        lecture::deja_lu(
            fonds,
            gestes::Sorte::Prise,
            &s,
            &lecture::empreintes_nommees(fonds)
        )
        .captation,
        "le fonds ne nomme pas le stem S"
    );
    assert!(
        verser_le_fichier(
            &mut services,
            &ouvert,
            gestes::Sorte::Prise,
            &s,
            &faits,
            "Voix seule.wav",
            None,
            &idee_importee(82),
        ),
        "« Voix seule.wav » ne s'écrit pas"
    );

    let fonds = fonds_de(&services);
    assert!(
        lecture::lire_la_captation(fonds, gestes::Sorte::Prise, &s).is_none(),
        "S versé seul a sa captation"
    );
    assert!(
        lecture::lire_l_idee(fonds, &idee_importee(82)).is_none(),
        "S versé seul a fait naître une idée"
    );
    let du_jeu = noms_lus(&[("Jeu A/voix.wav", IMPORT)]);
    assert_eq!(
        lecture::stems_de(fonds, &p),
        vec![StemLu {
            empreinte: s.clone(),
            faits: faits_d_un_stem(1_000),
            noms: du_jeu.clone(),
            type_de_stem: None,
            midis: BTreeSet::new(),
        }]
    );
    assert_eq!(
        lecture::noms_de(fonds, PRISES, &s, None),
        noms_lus(&[("Voix seule.wav", second.as_str())])
    );
    assert_eq!(
        lecture::noms_de(fonds, PRISES, &p, Some(s.as_str())),
        du_jeu
    );
    assert_eq!(
        lecture::premier_nom_vu(fonds, &s),
        Some("Jeu A/voix.wav".to_string())
    );
    assert_eq!(
        lecture::noms_poses_par(fonds, &second),
        NomsPoses {
            fichiers: BTreeMap::from([("Voix seule.wav".to_string(), (s.clone(), false))]),
            jeux: BTreeSet::new(),
        }
    );
    assert_eq!(
        lecture::noms_poses_par(fonds, IMPORT),
        NomsPoses {
            fichiers: BTreeMap::from([("Jeu A/voix.wav".to_string(), (s.clone(), false))]),
            jeux: BTreeSet::from(["Jeu A".to_string()]),
        }
    );
    assert_eq!(parcourir(fonds), Vec::<String>::new());
}

// Les services de l'import (tâche 12) : l'inspection, l'ouverture, les versements et la clôture,
// conduits par les commandes, sur la source à verser de chaque paire.

/// La place libre des imports des tests, assez grande pour chacun.
pub const PLACE_LIBRE: u64 = 1 << 40;

/// Le nom du dossier versé des tests de l'import.
pub const DOSSIER_VERSE: &str = "Démos d'automne";

/// L'en-tête d'un WAV de `wav`, en octets : `RIFF`, `fmt ` de 16 octets, puis `data`.
const ENTETE_WAV: usize = 44;

/// Les octets d'un WAV PCM de `trames` trames, à `frequence` Hz, de `canaux` canaux de `bits`
/// bits : l'en-tête de `ENTETE_WAV` octets, puis ses données, à zéro. Écrits à l'exécution (P12).
pub fn wav(frequence: u32, canaux: u16, bits: u16, trames: u64) -> Vec<u8> {
    let alignement = canaux * (bits / 8);
    let donnees = trames * u64::from(alignement);
    let taille =
        u32::try_from(donnees).expect("les données d'un WAV des tests tiennent en 32 bits");
    let mut octets = Vec::with_capacity(ENTETE_WAV + taille as usize);
    octets.extend_from_slice(b"RIFF");
    octets.extend_from_slice(&(36 + taille).to_le_bytes());
    octets.extend_from_slice(b"WAVEfmt ");
    octets.extend_from_slice(&16u32.to_le_bytes());
    octets.extend_from_slice(&1u16.to_le_bytes());
    octets.extend_from_slice(&canaux.to_le_bytes());
    octets.extend_from_slice(&frequence.to_le_bytes());
    octets.extend_from_slice(&(frequence * u32::from(alignement)).to_le_bytes());
    octets.extend_from_slice(&alignement.to_le_bytes());
    octets.extend_from_slice(&bits.to_le_bytes());
    octets.extend_from_slice(b"data");
    octets.extend_from_slice(&taille.to_le_bytes());
    octets.resize(ENTETE_WAV + taille as usize, 0);
    octets
}

/// `octets`, un WAV de `wav`, ses données remplies de `valeur` : deux valeurs, deux contenus.
fn rempli(mut octets: Vec<u8>, valeur: u8) -> Vec<u8> {
    octets[ENTETE_WAV..].fill(valeur);
    octets
}

/// Un WAV des tests de l'import : 8 000 Hz, mono, 8 bits, `trames` trames, ses données remplies de
/// `valeur`.
pub fn wav_de(trames: u64, valeur: u8) -> Vec<u8> {
    rempli(wav(8_000, 1, 8, trames), valeur)
}

/// Les octets de l'en-tête d'un MIDI : `MThd`, sa longueur, 6, puis `format`, `pistes` et
/// `division`, en gros-boutiste. Écrits à l'exécution (P12).
pub fn midi(format: u16, pistes: u16, division: u16) -> Vec<u8> {
    let mut octets = b"MThd\x00\x00\x00\x06".to_vec();
    octets.extend_from_slice(&format.to_be_bytes());
    octets.extend_from_slice(&pistes.to_be_bytes());
    octets.extend_from_slice(&division.to_be_bytes());
    octets
}

/// Les octets d'un M4A : une boîte `ftyp` de 24 octets, de marque `M4A `, puis une boîte `mdat`
/// vide. Écrits à l'exécution (P12).
pub fn m4a() -> Vec<u8> {
    let mut octets = 24u32.to_be_bytes().to_vec();
    octets.extend_from_slice(b"ftypM4A ");
    octets.extend_from_slice(&0u32.to_be_bytes());
    octets.extend_from_slice(b"M4A isom");
    octets.extend_from_slice(&8u32.to_be_bytes());
    octets.extend_from_slice(b"mdat");
    octets
}

/// Les octets d'une archive zip : `PK\x03\x04`, puis `octets` octets à zéro.
pub fn zip(octets: usize) -> Vec<u8> {
    let mut archive = b"PK\x03\x04".to_vec();
    archive.resize(4 + octets, 0);
    archive
}

/// Les octets d'un PDF : `%PDF-1.7`, puis `octets` octets à zéro.
pub fn pdf(octets: usize) -> Vec<u8> {
    let mut document = b"%PDF-1.7".to_vec();
    document.resize(8 + octets, 0);
    document
}

/// Les fichiers de `fichiers`, (chemin, contenu), chacun nommé, comme `Paire::source` les reçoit.
fn nommes(fichiers: &[(String, Vec<u8>)]) -> Vec<(Option<&str>, Vec<u8>)> {
    fichiers
        .iter()
        .map(|(chemin, contenu)| (Some(chemin.as_str()), contenu.clone()))
        .collect()
}

/// La déclaration d'un jeu de stems, sans origine (balayage Q3).
fn jeu_de_stems() -> commandes::Declaration {
    commandes::Declaration {
        stems: true,
        origine: OrigineADeclarer::Aucune,
    }
}

/// Ce qu'a rendu un import conduit par `importer` : son inspection, son ouverture, chaque fichier
/// versé, dans l'ordre des versements, et sa clôture.
pub struct ImportConduit {
    pub inspection: Inspection,
    pub ouvert: commandes::ImportOuvert,
    pub verses: Vec<FichierVerse>,
    pub clos: ImportClos,
}

/// Verse le fichier `rang` de l'import `import`, au décalage local nul, qui doit se verser.
pub fn verser_le_rang<D: Documents, C: Captations, S: Source>(
    services: &mut Services<D, C>,
    source: &S,
    import: &str,
    rang: u64,
    contexte: &mut Contexte<'_>,
) -> FichierVerse {
    services
        .verser_un_fichier(
            source,
            &VerserUnFichier {
                import: import.to_string(),
                rang,
                decalage_minutes: 0,
            },
            contexte,
        )
        .unwrap_or_else(|refus| panic!("le fichier n° {rang} ne se verse pas : {refus:?}"))
}

/// Conduit un import entier de `source`, unité par unité, comme l'interface (tâche 22) :
/// l'inspection, la place libre `PLACE_LIBRE` ; l'ouverture, ses dossiers déclarés
/// `declarations`, au décalage local nul ; chaque fichier hors des jeux, par rang ; puis chaque
/// jeu, ses fichiers, puis sa prise en stems ; puis la clôture. Chaque unité doit se faire. Les
/// identifiants se tirent de l'aléa de `contexte`, que les imports d'un test se partagent.
pub fn importer<D: Documents, C: Captations, S: Source>(
    services: &mut Services<D, C>,
    source: &S,
    declarations: &[(&str, commandes::Declaration)],
    contexte: &mut Contexte<'_>,
) -> ImportConduit {
    let inspection = services
        .inspecter(source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let commande = OuvrirLImport {
        declarations: declarations
            .iter()
            .map(|(chemin, declaration)| DeclarationDUnDossier {
                chemin: chemin.to_string(),
                declaration: declaration.clone(),
            })
            .collect(),
        decalage_minutes: 0,
    };
    let ouvert = services
        .ouvrir_l_import(source, &commande, PLACE_LIBRE, contexte)
        .expect("l'import s'ouvre");
    let mut verses = Vec::new();
    for rang in &ouvert.rangs {
        verses.push(verser_le_rang(
            services,
            source,
            &ouvert.import,
            *rang,
            contexte,
        ));
    }
    for jeu in &ouvert.jeux {
        for rang in &jeu.rangs {
            verses.push(verser_le_rang(
                services,
                source,
                &ouvert.import,
                *rang,
                contexte,
            ));
        }
        services
            .verser_une_prise_en_stems(
                source,
                &VerserUnePriseEnStems {
                    import: ouvert.import.clone(),
                    dossier: jeu.dossier.clone(),
                    decalage_minutes: 0,
                },
                contexte,
            )
            .unwrap_or_else(|refus| {
                panic!("le jeu « {} » ne se verse pas : {refus:?}", jeu.dossier)
            });
    }
    let clos = services
        .clore_l_import(
            source,
            &CloreLImport {
                import: ouvert.import.clone(),
                decalage_minutes: 0,
            },
            contexte,
        )
        .expect("l'import se clôt");
    ImportConduit {
        inspection,
        ouvert,
        verses,
        clos,
    }
}

/// Un compte du rapport de `fichiers` fichiers et `octets` octets.
fn compte(fichiers: u64, octets: u64) -> commandes::Compte {
    commandes::Compte { fichiers, octets }
}

/// Les contenus de `fichiers`, (chemin, contenu).
fn contenus(fichiers: &[(String, Vec<u8>)]) -> Vec<&[u8]> {
    fichiers
        .iter()
        .map(|(_, contenu)| contenu.as_slice())
        .collect()
}

/// Le compte des contenus de `contenus`.
fn compte_des(contenus: &[&[u8]]) -> commandes::Compte {
    compte(
        contenus.len() as u64,
        contenus.iter().map(|contenu| contenu.len() as u64).sum(),
    )
}

/// Le rapport d'un import `import` qui ne compte que des versés et des dédoublonnés : ni
/// illisible, ni ignoré, ni fichier sans nom, ni origine changée.
fn rapport_de(
    import: &str,
    vus: commandes::Compte,
    verses: commandes::Compte,
    dedoublonnes: commandes::Compte,
) -> RapportDImport {
    RapportDImport {
        import: import.to_string(),
        vus,
        verses,
        dedoublonnes,
        illisibles: compte(0, 0),
        ignores: compte(0, 0),
        sans_nom: compte(0, 0),
        origines_changees: 0,
        fichiers_sans_nom: Vec::new(),
    }
}

/// Les clés du conteneur `conteneur` de la racine du fonds ; un conteneur absent, aucune.
fn cles_du_conteneur(fonds: &Automerge, conteneur: &str) -> Vec<String> {
    let Ok(Some((Value::Object(ObjType::Map), objet))) = fonds.get(ROOT, conteneur) else {
        return Vec::new();
    };
    fonds.keys(&objet).collect()
}

/// Le premier segment de chaque clé du conteneur `conteneur` de la racine du fonds : sous
/// `prises`, chaque prise, une prise en stems comprise, sans ses stems.
fn premiers_segments(fonds: &Automerge, conteneur: &str) -> BTreeSet<String> {
    cles_du_conteneur(fonds, conteneur)
        .into_iter()
        .filter_map(|cle| cle.split_once('/').map(|(premier, _)| premier.to_string()))
        .collect()
}

/// Le format du contenu `empreinte` sous `conteneur`, `<empreinte>/format`, lu du fonds : absent,
/// ou d'un autre type qu'une chaîne, `None`.
fn format_de(fonds: &Automerge, conteneur: &str, empreinte: &str) -> Option<String> {
    let Ok(Some((Value::Object(ObjType::Map), objet))) = fonds.get(ROOT, conteneur) else {
        return None;
    };
    let (valeur, _) = fonds
        .get(&objet, format!("{empreinte}/{FORMAT}").as_str())
        .ok()??;
    valeur.into_string().ok()
}

/// La vue de la seule idée du fonds de `services`, qui doit n'en porter qu'une.
fn la_seule_idee<D: Documents, C: Captations>(services: &Services<D, C>) -> VueDUneIdee {
    let idees = services.idees().expect("les idées se lisent");
    assert_eq!(idees.len(), 1, "les idées : {idees:?}");
    services
        .vue_d_une_idee(&idees[0])
        .expect("la seule idée se lit")
}

/// CAP-03 · 1 : 77 WAV de 8 000 Hz, mono, 8 bits, de 1 à 77 trames (le rang r, r trames), puis
/// trois copies des trois premiers dans `Copies/`. Importés : 77 idées, chacune importée par
/// l'import ; 78 captations, les 77 et le rapport ; le rapport compte 80 vus, 77 versés et 3
/// dédoublonnés, leurs octets ceux de leurs contenus.
pub fn cap03_1_quatre_vingts_fichiers_dont_trois_doublons_donnent_soixante_dix_sept_captations<
    P: Paire,
>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let originaux: Vec<(String, Vec<u8>)> = (1..=77u8)
        .map(|r| (format!("Essai {r:02}.wav"), wav_de(u64::from(r), r)))
        .collect();
    let copies: Vec<(String, Vec<u8>)> = originaux[..3]
        .iter()
        .map(|(chemin, contenu)| (format!("Copies/{chemin}"), contenu.clone()))
        .collect();
    let tous: Vec<(String, Vec<u8>)> = originaux.iter().chain(&copies).cloned().collect();
    let source = paire.source(Some(DOSSIER_VERSE), &nommes(&tous), &[]);

    let conduit = importer(&mut services, &source, &[], &mut contexte);

    let import = conduit.ouvert.import.clone();
    let idees = services.idees().expect("les idées se lisent");
    assert_eq!(idees.len(), 77);
    for idee in &idees {
        assert_eq!(
            services.vue_d_une_idee(idee).map(|vue| vue.provenance),
            Ok(commandes::Provenance::Importee {
                par: commandes::Appareil::Etabli,
                depuis: import.clone(),
            }),
            "la provenance de l'idée {idee}"
        );
    }
    assert_eq!(lecture::empreintes_nommees(fonds_de(&services)).len(), 78);
    assert_eq!(
        conduit.clos.rapport,
        rapport_de(
            &import,
            compte_des(&contenus(&tous)),
            compte_des(&contenus(&originaux)),
            compte_des(&contenus(&copies)),
        )
    );
}

/// CAP-03 · 2 : le dossier de CAP-03 · 1, importé une seconde fois, n'ajoute ni idée ni captation
/// que son rapport ; chaque clé neuve du document est sous `imports/<second import>/` ; le second
/// rapport compte 80 vus, aucun versé, 80 dédoublonnés.
pub fn cap03_2_le_meme_dossier_importe_deux_fois_n_ajoute_que_son_rapport<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let originaux: Vec<(String, Vec<u8>)> = (1..=77u8)
        .map(|r| (format!("Essai {r:02}.wav"), wav_de(u64::from(r), r)))
        .collect();
    let copies: Vec<(String, Vec<u8>)> = originaux[..3]
        .iter()
        .map(|(chemin, contenu)| (format!("Copies/{chemin}"), contenu.clone()))
        .collect();
    let tous: Vec<(String, Vec<u8>)> = originaux.iter().chain(&copies).cloned().collect();
    let source = paire.source(Some(DOSSIER_VERSE), &nommes(&tous), &[]);
    importer(&mut services, &source, &[], &mut contexte);
    let fonds = fonds_de(&services);
    let chemins_avant: BTreeSet<Vec<String>> = chaque_valeur(fonds)
        .into_iter()
        .map(|(chemin, _)| chemin)
        .collect();
    let nommees_avant = lecture::empreintes_nommees(fonds);
    let idees_avant = services.idees();

    let second = importer(&mut services, &source, &[], &mut contexte);

    let fonds = fonds_de(&services);
    assert_eq!(services.idees(), idees_avant);
    let nommees_apres = lecture::empreintes_nommees(fonds);
    assert_eq!(nommees_apres.len(), nommees_avant.len() + 1);
    assert!(nommees_apres.is_superset(&nommees_avant));
    let sous_le_second = format!("{}/", second.ouvert.import);
    for (chemin, _) in chaque_valeur(fonds) {
        if !chemins_avant.contains(&chemin) {
            assert!(
                chemin.len() == 2 && chemin[0] == IMPORTS && chemin[1].starts_with(&sous_le_second),
                "une clé neuve hors du second import : {chemin:?}"
            );
        }
    }
    let tout = compte_des(&contenus(&tous));
    assert_eq!(
        second.clos.rapport,
        rapport_de(&second.ouvert.import, tout.clone(), compte(0, 0), tout)
    );
}

/// CAP-03 · 3 : `Démo d'hiver 2 - 01_06_2015 18.42.m4a`, une boîte `ftyp`, se verse en prise
/// `m4a` ; son premier nom vu, relu, égale ce nom, octet pour octet. Sa ligne de la boîte, sans
/// texte, porte ce nom (`nom_de_fichier`, question 4) ; son texte complété, elle ne porte plus que
/// son texte.
pub fn cap03_3_un_nom_d_origine_se_relit_intact<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let nom = "Démo d'hiver 2 - 01_06_2015 18.42.m4a";
    let source = paire.source(Some(DOSSIER_VERSE), &[(Some(nom), m4a())], &[]);

    importer(&mut services, &source, &[], &mut contexte);

    let fonds = fonds_de(&services);
    let e = empreinte(&m4a());
    assert_eq!(
        format_de(fonds, PRISES, &e).as_deref(),
        Some(FormatSonore::M4a.nom())
    );
    let premier = lecture::premier_nom_vu(fonds, &e).expect("le M4A a un nom");
    assert_eq!(premier.as_bytes(), nom.as_bytes());
    let vue = la_seule_idee(&services);
    assert_eq!(vue.captations_vues.len(), 1);
    assert_eq!(vue.captations_vues[0].noms, vec![nom.to_string()]);

    let lignes = la_boite(&services).lignes;
    assert_eq!(idees_des_lignes(&lignes), vec![vue.idee.clone()]);
    assert_eq!(lignes[0].texte, None);
    assert_eq!(lignes[0].nom_de_fichier.as_deref(), Some(nom));

    let texte = "Un refrain qui revient";
    services
        .completer(&completion(&vue.idee, texte), &mut contexte)
        .expect("le texte se complète");
    let lignes = la_boite(&services).lignes;
    assert_eq!(idees_des_lignes(&lignes), vec![vue.idee.clone()]);
    assert_eq!(lignes[0].texte.as_deref(), Some(texte));
    assert_eq!(lignes[0].nom_de_fichier, None);
}

/// CAP-03 · 4, en part, sans morceaux : `Pont v1.wav`, `Pont v2.wav`, `Pont.mid` et `Pont.txt`
/// donnent quatre idées, une captation chacune, et aucun lien.
pub fn cap03_4_des_noms_semblables_ne_creent_ni_lien_ni_regroupement<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Pont v1.wav"), wav_de(8_000, 1)),
            (Some("Pont v2.wav"), wav_de(8_000, 2)),
            (Some("Pont.mid"), midi(1, 2, 480)),
            (Some("Pont.txt"), b"Un pont qui hesite\n".to_vec()),
        ],
        &[],
    );

    importer(&mut services, &source, &[], &mut contexte);

    let idees = services.idees().expect("les idées se lisent");
    assert_eq!(idees.len(), 4);
    for idee in &idees {
        let vue = services.vue_d_une_idee(idee).expect("l'idée se lit");
        assert_eq!(vue.captations.len(), 1, "les captations de l'idée {idee}");
        assert_eq!(
            vue.captations_vues.len(),
            1,
            "les captations vues de {idee}"
        );
        assert_eq!(
            vue.liens,
            Vec::<String>::new(),
            "les liens de l'idée {idee}"
        );
    }
    assert_eq!(
        cles_du_conteneur(fonds_de(&services), LIENS),
        Vec::<String>::new()
    );
}

/// Les dix stems de CAP-03 · 7 : le nom de chacun, et le type que la table propose pour lui.
const STEMS_DU_JEU: [(&str, commandes::TypeDeStem); 10] = [
    ("basse", commandes::TypeDeStem::Basse),
    ("batterie", commandes::TypeDeStem::Batterie),
    ("chant", commandes::TypeDeStem::Voix),
    ("choeurs", commandes::TypeDeStem::Choeurs),
    ("clavier", commandes::TypeDeStem::Clavier),
    ("cordes", commandes::TypeDeStem::Cordes),
    ("cuivres", commandes::TypeDeStem::Cuivres),
    ("guitare", commandes::TypeDeStem::Guitare),
    ("percussions", commandes::TypeDeStem::Percussions),
    ("synthe", commandes::TypeDeStem::Synthe),
];

/// CAP-03 · 7 : `Jeu A` déclaré jeu de stems, dix WAV de 8 000 trames à 8 000 Hz aux contenus
/// distincts, neuf MIDI aux noms de neuf d'entre eux : une prise, une idée, dix stems dont neuf
/// avec leur MIDI, chaque type proposé (`TypeVu::Propose`), aucune clé `type` ; 21 captations,
/// les dix, les neuf, le manifeste et le rapport ; le rapport compte 19 versés ; un second import
/// ne verse que son rapport.
pub fn cap03_7_un_jeu_de_dix_wav_et_neuf_midi_donne_une_prise_en_stems<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let mut fichiers: Vec<(String, Vec<u8>)> = Vec::new();
    for (rang, (nom, _)) in (1..).zip(STEMS_DU_JEU) {
        fichiers.push((format!("Jeu A/{nom}.wav"), wav_de(8_000, rang)));
        if rang < 10 {
            fichiers.push((format!("Jeu A/{nom}.mid"), midi(1, 2, 96 * u16::from(rang))));
        }
    }
    let source = paire.source(Some(DOSSIER_VERSE), &nommes(&fichiers), &[]);

    let conduit = importer(
        &mut services,
        &source,
        &[("Jeu A", jeu_de_stems())],
        &mut contexte,
    );

    let vue = la_seule_idee(&services);
    assert_eq!(vue.captations_vues.len(), 1);
    let prise = &vue.captations_vues[0];
    assert_eq!(prise.stems.len(), 10);
    let mut avec_un_midi = 0;
    for stem in &prise.stems {
        let nom = stem.noms[0]
            .strip_prefix("Jeu A/")
            .and_then(|nom| nom.strip_suffix(".wav"))
            .expect("le nom d'un stem du jeu");
        let (_, propose) = STEMS_DU_JEU
            .iter()
            .find(|(mot, _)| *mot == nom)
            .expect("un stem du jeu");
        assert_eq!(stem.type_de_stem, TypeVu::Propose(propose.clone()), "{nom}");
        if let Some(midi) = &stem.midi {
            avec_un_midi += 1;
            assert_eq!(midi.noms, vec![format!("Jeu A/{nom}.mid")]);
        }
    }
    assert_eq!(avec_un_midi, 9);
    let fonds = fonds_de(&services);
    let fin = format!("/{TYPE}");
    let milieu = format!("/{TYPE}/");
    for (chemin, _) in chaque_valeur(fonds) {
        assert!(
            !chemin
                .iter()
                .any(|cle| cle.ends_with(&fin) || cle.contains(&milieu)),
            "une clé de type : {chemin:?}"
        );
    }
    assert_eq!(lecture::empreintes_nommees(fonds).len(), 21);
    assert_eq!(conduit.clos.rapport.verses.fichiers, 19);

    importer(
        &mut services,
        &source,
        &[("Jeu A", jeu_de_stems())],
        &mut contexte,
    );

    assert_eq!(services.idees().map(|idees| idees.len()), Ok(1));
    assert_eq!(lecture::empreintes_nommees(fonds_de(&services)).len(), 22);
}

/// CAP-03 · 8, livré en part : trois WAV de même durée, aux contenus distincts, dans `Trio/` ;
/// l'inspection propose un jeu de ce dossier ; ouvert sans déclaration, trois prises, trois idées.
pub fn cap03_8_trois_wav_de_meme_duree_proposent_un_jeu_et_sans_declaration_trois_prises<
    P: Paire,
>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Trio/a.wav"), wav_de(8_000, 1)),
            (Some("Trio/b.wav"), wav_de(8_000, 2)),
            (Some("Trio/c.wav"), wav_de(8_000, 3)),
        ],
        &[],
    );

    let conduit = importer(&mut services, &source, &[], &mut contexte);

    let trio = conduit
        .inspection
        .dossiers
        .iter()
        .find(|dossier| dossier.chemin == "Trio")
        .expect("l'inspection voit le dossier « Trio »");
    assert!(trio.jeu_propose, "aucun jeu n'est proposé de « Trio »");
    assert_eq!(trio.audio, 3);
    assert_eq!(conduit.ouvert.jeux, Vec::<commandes::JeuAVerser>::new());
    assert_eq!(services.idees().map(|idees| idees.len()), Ok(3));
    assert_eq!(premiers_segments(fonds_de(&services), PRISES).len(), 3);
}

/// R4 ; question 37 ; V1 : dans un Pupitre neuf, dans `Jeu A/`, trois WAV de 100 000 trames,
/// 8 000 Hz, mono, 8 bits (100 044 octets chacun), leurs données distinctes dès leur premier
/// octet, un texte et un MIDI. Après l'inspection, chaque WAV est lu de `SIGNATURE_AU_PLUS +
/// ENTETE_AU_PLUS` octets au plus, jamais en entier ; le texte et le MIDI, de `SIGNATURE_AU_PLUS`
/// au plus ; le jeu est proposé. Puis, dans `Jeu B/`, deux WAV de 100 044 octets aux 65 536
/// premiers octets égaux : aucun jeu n'est proposé, et leurs octets lus restent sous la même
/// borne.
pub fn un_premier_import_ne_lit_que_signatures_et_en_tetes<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let borne = (SIGNATURE_AU_PLUS + ENTETE_AU_PLUS) as u64;
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Jeu A/a.wav"), wav_de(100_000, 1)),
            (Some("Jeu A/b.wav"), wav_de(100_000, 2)),
            (Some("Jeu A/c.wav"), wav_de(100_000, 3)),
            (Some("Jeu A/notes.txt"), b"Une basse ronde\n".to_vec()),
            (Some("Jeu A/a.mid"), midi(1, 2, 480)),
        ],
        &[],
    );

    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");

    for entree in source.entrees() {
        let chemin = entree.chemin.as_deref().expect("chaque fichier est nommé");
        let lus = source.octets_lus(entree.rang);
        if chemin.ends_with(".wav") {
            assert_eq!(entree.octets, 100_044);
            assert!(lus <= borne && lus < 100_044, "{chemin} : {lus} octets lus");
        } else {
            assert!(
                lus <= SIGNATURE_AU_PLUS as u64,
                "{chemin} : {lus} octets lus"
            );
        }
    }
    let jeu_a = inspection
        .dossiers
        .iter()
        .find(|dossier| dossier.chemin == "Jeu A")
        .expect("l'inspection voit « Jeu A »");
    assert!(jeu_a.jeu_propose, "aucun jeu n'est proposé de « Jeu A »");

    let x = wav_de(100_000, 7);
    let mut y = x.clone();
    y[70_000] = 8;
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Jeu B/x.wav"), x), (Some("Jeu B/y.wav"), y)],
        &[],
    );
    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let jeu_b = inspection
        .dossiers
        .iter()
        .find(|dossier| dossier.chemin == "Jeu B")
        .expect("l'inspection voit « Jeu B »");
    assert!(!jeu_b.jeu_propose, "un jeu est proposé de « Jeu B »");
    for entree in source.entrees() {
        let lus = source.octets_lus(entree.rang);
        assert!(lus <= borne, "{:?} : {lus} octets lus", entree.chemin);
    }
}

/// Question 38 ; V2 : un WAV `Démos/a.wav` dont la lecture est refusée ; l'inspection rend
/// `Raison::Disque`, son message nomme le fichier ; rien ne s'écrit ; la lecture rendue, la même
/// inspection passe.
pub fn un_fichier_qui_ne_se_lit_pas_arrete_l_inspection<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Démos/a.wav"), wav_de(8_000, 1))],
        &[],
    );
    paire.refuser_la_lecture(&mut source, 1, true);

    let refus = services
        .inspecter(&source, PLACE_LIBRE)
        .expect_err("l'inspection d'un fichier refusé est refusée");

    assert_eq!(refus.raison, Raison::Disque);
    assert!(
        refus
            .message
            .starts_with("« Démos/a.wav » ne se lit pas : "),
        "{}",
        refus.message
    );
    assert!(services.documents().fonds().is_none(), "le fonds est né");
    assert_eq!(services.captations().tailles(), Ok(BTreeSet::new()));
    paire.refuser_la_lecture(&mut source, 1, false);
    assert!(
        services.inspecter(&source, PLACE_LIBRE).is_ok(),
        "la lecture rendue, l'inspection est refusée"
    );
}

/// Question 19 ; R11 : un WAV versé en prise, puis son dossier déclaré jeu de stems, avec ce seul
/// WAV : deux prises, l'empreinte du manifeste distincte de celle du WAV.
pub fn un_jeu_d_un_seul_contenu_donne_une_prise_en_stems_distincte<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let voix = wav_de(8_000, 5);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Solo/voix.wav"), voix.clone())],
        &[],
    );
    importer(&mut services, &source, &[], &mut contexte);

    importer(
        &mut services,
        &source,
        &[("Solo", jeu_de_stems())],
        &mut contexte,
    );

    let e = empreinte(&voix);
    let manifeste = empreinte(&rapport::manifeste(&[e.as_str()]));
    assert_ne!(manifeste, e);
    assert_eq!(
        premiers_segments(fonds_de(&services), PRISES),
        BTreeSet::from([e, manifeste])
    );
}

/// CAP-03 · 10 : `A/x.wav` et `B/y.wav`, du même contenu, donnent une captation, de deux noms.
pub fn cap03_10_un_contenu_sous_deux_noms_dans_deux_sous_dossiers_est_un_objet_a_deux_noms<
    P: Paire,
>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let contenu = wav_de(8_000, 4);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("A/x.wav"), contenu.clone()),
            (Some("B/y.wav"), contenu.clone()),
        ],
        &[],
    );

    importer(&mut services, &source, &[], &mut contexte);

    let e = empreinte(&contenu);
    let fonds = fonds_de(&services);
    assert_eq!(
        premiers_segments(fonds, PRISES),
        BTreeSet::from([e.clone()])
    );
    let vue = la_seule_idee(&services);
    assert_eq!(vue.captations_vues.len(), 1);
    assert_eq!(
        vue.captations_vues[0].noms,
        vec!["A/x.wav".to_string(), "B/y.wav".to_string()]
    );
}

/// CAP-03 · 11 : une archive zip (`PK\x03\x04` puis 5 000 octets), un PDF (`%PDF-1.7` puis
/// 2 000 octets) et un WAV ; le rapport compte deux ignorés, de 5 004 et 2 008 octets ; ni
/// l'archive ni le PDF ne sont lus au-delà de 16 octets.
pub fn cap03_11_une_archive_et_un_pdf_sont_ignores_sans_etre_ouverts<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Archives/demos.zip"), zip(5_000)),
            (Some("Partitions/pont.pdf"), pdf(2_000)),
            (Some("pont.wav"), wav_de(8_000, 6)),
        ],
        &[],
    );

    let conduit = importer(&mut services, &source, &[], &mut contexte);

    assert_eq!(conduit.clos.rapport.ignores, compte(2, 5_004 + 2_008));
    for entree in source.entrees() {
        if entree.chemin.as_deref() != Some("pont.wav") {
            assert!(
                source.octets_lus(entree.rang) <= 16,
                "{:?} : {} octets lus",
                entree.chemin,
                source.octets_lus(entree.rang)
            );
        }
    }
}

/// CAP-03 · 13 : un WAV de 8 044 octets, une place libre de 4 000 octets : l'inspection dit le
/// manque (8 044 octets à verser, 12 140 demandés, 4 000 libres, 8 140 de manque ; balayage Q5) ;
/// l'ouverture est refusée de ce message ; le fonds n'est pas né, et aucune captation n'est
/// versée.
pub fn cap03_13_une_taille_a_verser_plus_grande_que_la_place_libre_ne_verse_rien<P: Paire>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Essai.wav"), wav_de(8_000, 1))],
        &[],
    );
    let manque = "Verser 0,1 Go demande 0,1 Go de place libre, deux fois le poids du fonds \
                  compris ; il y en a 0,0 Go : il manque 0,1 Go.";

    let inspection = services
        .inspecter(&source, 4_000)
        .expect("le dossier s'inspecte");

    assert_eq!(inspection.a_verser, 8_044);
    assert_eq!(inspection.libre, 4_000);
    assert_eq!(inspection.manque.as_deref(), Some(manque));
    assert_eq!(
        services.ouvrir_l_import(
            &source,
            &OuvrirLImport {
                declarations: Vec::new(),
                decalage_minutes: 0,
            },
            4_000,
            &mut contexte,
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: manque.to_string(),
        })
    );
    assert!(services.documents().fonds().is_none(), "le fonds est né");
    assert_eq!(services.captations().tailles(), Ok(BTreeSet::new()));
}

/// Point de revue 1 : deux copies d'un contenu, versées à rebours de leurs rangs ; la première
/// versée est versée, la seconde dédoublonnée ; le rapport compte un versé, un dédoublonné ; le
/// contenu a deux noms.
pub fn deux_copies_d_un_contenu_versees_dans_le_desordre_comptent_un_verse<P: Paire>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let contenu = wav_de(8_000, 9);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("A/un.wav"), contenu.clone()),
            (Some("B/deux.wav"), contenu.clone()),
        ],
        &[],
    );
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let ouvert = services
        .ouvrir_l_import(
            &source,
            &OuvrirLImport {
                declarations: Vec::new(),
                decalage_minutes: 0,
            },
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre");
    assert_eq!(ouvert.rangs, vec![1, 2]);

    let deuxieme = verser_le_rang(&mut services, &source, &ouvert.import, 2, &mut contexte);
    let premier = verser_le_rang(&mut services, &source, &ouvert.import, 1, &mut contexte);
    let clos = services
        .clore_l_import(
            &source,
            &CloreLImport {
                import: ouvert.import.clone(),
                decalage_minutes: 0,
            },
            &mut contexte,
        )
        .expect("l'import se clôt");

    assert_eq!(deuxieme.issue, Issue::Verse);
    assert_eq!(premier.issue, Issue::Dedoublonne);
    let un = compte_des(&[contenu.as_slice()]);
    assert_eq!(
        clos.rapport,
        rapport_de(
            &ouvert.import,
            compte_des(&[contenu.as_slice(), contenu.as_slice()]),
            un.clone(),
            un
        )
    );
    assert_eq!(
        la_seule_idee(&services).captations_vues[0].noms,
        vec!["A/un.wav".to_string(), "B/deux.wav".to_string()]
    );
}

/// Point de revue 2 : une archive zip nommée `faux.wav` est ignorée, lue de ses 16 premiers
/// octets, jamais au-delà, ni à l'inspection, ni à l'import.
pub fn un_zip_nomme_wav_est_ignore_et_lu_de_seize_octets<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(Some(DOSSIER_VERSE), &[(Some("faux.wav"), zip(3_000))], &[]);

    let conduit = importer(&mut services, &source, &[], &mut contexte);

    assert_eq!(conduit.inspection.fichiers.len(), 1);
    assert_eq!(
        conduit.inspection.fichiers[0].nature,
        NatureDuFichier::Ignore
    );
    assert_eq!(conduit.inspection.a_verser, 0);
    assert_eq!(conduit.ouvert.rangs, Vec::<u64>::new());
    assert_eq!(source.octets_lus(1), 16);
    assert_eq!(conduit.clos.rapport.ignores, compte(1, 3_004));
}

/// Point de revue 3 ; balayage Q14 : un WAV versé en prise, puis le même contenu, stem d'un jeu
/// déclaré ; la prise garde son idée, la prise en stems nomme le stem, le rapport le compte
/// dédoublonné. Puis un stem dont le contenu, déjà prise, porte `generee/Atelier inventé`, dans un
/// jeu déclaré `{ stems: true, origine: Main }` : il garde l'origine générée, sans clé
/// `origines_declarees/<s>/main`.
pub fn un_stem_deja_prise_reste_la_prise_de_son_idee<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let voix = wav_de(8_000, 11);
    let basse = wav_de(8_000, 12);
    let premiere = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Seule/voix.wav"), voix.clone())],
        &[],
    );
    importer(&mut services, &premiere, &[], &mut contexte);
    let idee_de_la_voix = la_seule_idee(&services).idee;

    let seconde = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Jeu A/basse.wav"), basse.clone()),
            (Some("Jeu A/voix.wav"), voix.clone()),
        ],
        &[],
    );
    let conduit = importer(
        &mut services,
        &seconde,
        &[("Jeu A", jeu_de_stems())],
        &mut contexte,
    );

    let fonds = fonds_de(&services);
    let e_voix = empreinte(&voix);
    let e_basse = empreinte(&basse);
    assert_eq!(
        lecture::lire_la_captation(fonds, gestes::Sorte::Prise, &e_voix).map(|lue| lue.idee),
        Some(idee_de_la_voix)
    );
    let p = empreinte(&rapport::manifeste(&[e_voix.as_str(), e_basse.as_str()]));
    let du_jeu = lecture::stems_de(fonds, &p);
    let stem_voix = du_jeu
        .iter()
        .find(|stem| stem.empreinte == e_voix)
        .expect("la prise en stems porte la voix");
    assert_eq!(
        stem_voix.noms,
        noms_lus(&[("Jeu A/voix.wav", conduit.ouvert.import.as_str())])
    );
    assert_eq!(
        conduit.clos.rapport.dedoublonnes,
        compte_des(&[voix.as_slice()])
    );
    assert_eq!(conduit.clos.rapport.verses, compte_des(&[basse.as_slice()]));

    let nappe = wav_de(8_000, 13);
    let pad = wav_de(8_000, 14);
    let atelier = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Atelier/nappe.wav"), nappe.clone())],
        &[],
    );
    importer(
        &mut services,
        &atelier,
        &[(
            "Atelier",
            commandes::Declaration {
                stems: false,
                origine: OrigineADeclarer::Generee {
                    nom: "Atelier inventé".to_string(),
                },
            },
        )],
        &mut contexte,
    );
    let jeu_b = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Jeu B/nappe.wav"), nappe.clone()),
            (Some("Jeu B/pad.wav"), pad),
        ],
        &[],
    );
    importer(
        &mut services,
        &jeu_b,
        &[(
            "Jeu B",
            commandes::Declaration {
                stems: true,
                origine: OrigineADeclarer::Main,
            },
        )],
        &mut contexte,
    );

    assert_eq!(
        lecture::origines_de(fonds_de(&services), &empreinte(&nappe)),
        OriginesLues {
            main: false,
            generees: BTreeSet::from(["Atelier inventé".to_string()]),
        }
    );
}

/// Balayage Q3 : `Jeu A` déclaré `{ stems: true, origine: Generee { nom: "Atelier inventé" } }`
/// donne une prise en stems générée par « Atelier inventé », et `lecture::declarations_de` rend,
/// pour `Jeu A`, le jeu et l'origine.
pub fn un_jeu_declare_genere_porte_son_origine<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Jeu A/nappe.wav"), wav_de(8_000, 21)),
            (Some("Jeu A/pad.wav"), wav_de(8_000, 22)),
        ],
        &[],
    );

    let conduit = importer(
        &mut services,
        &source,
        &[(
            "Jeu A",
            commandes::Declaration {
                stems: true,
                origine: OrigineADeclarer::Generee {
                    nom: "Atelier inventé".to_string(),
                },
            },
        )],
        &mut contexte,
    );

    let vue = la_seule_idee(&services);
    assert_eq!(vue.captations_vues.len(), 1);
    assert_eq!(
        vue.captations_vues[0].origine,
        OrigineDeclaree::Generee {
            noms: vec!["Atelier inventé".to_string()],
        }
    );
    assert_eq!(
        lecture::declarations_de(fonds_de(&services), &conduit.ouvert.import),
        BTreeMap::from([(
            "Jeu A".to_string(),
            Declaration {
                stems: true,
                origine: Some(Ajout::Generee("Atelier inventé".to_string())),
            },
        )])
    );
}

/// Balayage Q6 : l'inspection d'une place libre de `u64::MAX` rend `ENTIER_SUR`, et ne dit aucun
/// manque.
pub fn la_place_libre_rendue_se_borne_a_entier_sur<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Essai.wav"), wav_de(8_000, 1))],
        &[],
    );

    let inspection = services
        .inspecter(&source, u64::MAX)
        .expect("le dossier s'inspecte");

    assert_eq!(inspection.libre, ENTIER_SUR);
    assert_eq!(inspection.manque, None);
}

/// Un texte en Latin-1 se verse sans format, illisible, et le rapport le compte à part.
pub fn un_texte_illisible_est_verse_et_compte_a_part<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let texte = b"Un \xe9t\xe9 en r\xe9\n".to_vec();
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Paroles.txt"), texte.clone())],
        &[],
    );

    let conduit = importer(&mut services, &source, &[], &mut contexte);

    let fonds = fonds_de(&services);
    let e = empreinte(&texte);
    assert!(
        lecture::lire_la_captation(fonds, gestes::Sorte::TexteVerse, &e).is_some(),
        "le texte n'est pas versé"
    );
    assert_eq!(format_de(fonds, TEXTES_VERSES, &e), None);
    assert_eq!(conduit.clos.rapport.verses, compte_des(&[texte.as_slice()]));
    assert_eq!(
        conduit.clos.rapport.illisibles,
        compte_des(&[texte.as_slice()])
    );
}

/// R23 : un WAV sans nom en UTF-8 se verse, sans clé `nom` ; le rapport compte un fichier sans
/// nom, et sa ligne `fichier-sans-nom=<rang>` se relit.
pub fn un_fichier_sans_nom_est_verse_et_compte_a_part<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let contenu = wav_de(8_000, 31);
    let source = paire.source(Some(DOSSIER_VERSE), &[(None, contenu.clone())], &[]);
    assert_eq!(source.entrees()[0].chemin, None);

    let conduit = importer(&mut services, &source, &[], &mut contexte);

    let fonds = fonds_de(&services);
    let e = empreinte(&contenu);
    assert!(
        lecture::lire_la_captation(fonds, gestes::Sorte::Prise, &e).is_some(),
        "le WAV sans nom n'est pas versé"
    );
    assert_eq!(
        lecture::noms_de(fonds, PRISES, &e, None),
        Vec::<(String, String)>::new()
    );
    assert_eq!(
        conduit.clos.rapport.sans_nom,
        compte_des(&[contenu.as_slice()])
    );
    assert_eq!(conduit.clos.rapport.fichiers_sans_nom, vec![1]);
    let rapport_du_fonds = lecture::imports(fonds)
        .into_iter()
        .find_map(|lu| lu.rapport)
        .expect("l'import clos a son rapport");
    let octets = services
        .captations()
        .relire(&rapport_du_fonds)
        .expect("le rapport se relit");
    assert!(
        String::from_utf8_lossy(&octets).contains("\nfichier-sans-nom=1\n"),
        "{}",
        String::from_utf8_lossy(&octets)
    );
    assert_eq!(
        rapport::lire_le_rapport(&octets).map(|lu| lu.fichiers_sans_nom),
        Some(vec![1])
    );
}

/// La place compte deux fois le poids du fonds (`Automerge::save`) : sur un fonds né, une place
/// libre égale à la taille à verser, plus deux fois ce poids, plus 4 096, suffit ; un octet de
/// moins manque.
pub fn la_place_compte_deux_fois_le_poids_du_fonds<P: Paire>(paire: &mut P) {
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
        .expect("la note fait naître le fonds");
    let contenu = wav_de(8_000, 1);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Essai.wav"), contenu.clone())],
        &[],
    );
    let poids = fonds_de(&services).save().len() as u64;
    let juste = contenu.len() as u64 + 2 * poids + 4_096;

    let suffit = services
        .inspecter(&source, juste)
        .expect("le dossier s'inspecte");
    let manque = services
        .inspecter(&source, juste - 1)
        .expect("le dossier s'inspecte");

    assert_eq!(suffit.manque, None);
    assert!(manque.manque.is_some(), "un octet de moins ne manque pas");
}

/// L'import dans un Pupitre neuf fait naître le fonds par son ouverture : le journal porte la
/// genèse, la naissance, l'ouverture, le versement, puis la clôture, dans cet ordre.
pub fn l_import_dans_un_pupitre_neuf_fait_naitre_le_fonds_par_son_ouverture<P: Paire>(
    paire: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("Essai.wav"), wav_de(8_000, 1))],
        &[],
    );

    importer(&mut services, &source, &[], &mut contexte);

    assert_eq!(
        services.journal().map(|entrees| entrees
            .into_iter()
            .map(|entree| entree.message)
            .collect::<Vec<_>>()),
        Ok(vec![
            MESSAGE_DE_LA_GENESE.to_string(),
            MESSAGE_DE_LA_NAISSANCE.to_string(),
            "Import du 14 novembre ouvert".to_string(),
            "Idée du 14 novembre : prise versée".to_string(),
            "Import du 14 novembre clos".to_string(),
        ])
    );
}

/// L'avertissement d'une déclaration « de la main de Sacha » qu'une cible générée garde
/// (question 22 ; V9).
const ORIGINE_GARDEE: &str = "L'origine reste générée : une origine déclarée ne se retire pas.";

/// L'origine générée par le générateur `nom`, à déclarer.
fn generee(nom: &str) -> OrigineADeclarer {
    OrigineADeclarer::Generee {
        nom: nom.to_string(),
    }
}

/// La déclaration d'un dossier, sans jeu de stems, à l'origine `origine`.
fn origine_du_dossier(origine: OrigineADeclarer) -> commandes::Declaration {
    commandes::Declaration {
        stems: false,
        origine,
    }
}

/// La déclaration de l'origine `origine` sur la prise de contenu `contenu`.
fn declaration_sur_la_prise(
    contenu: &[u8],
    origine: OrigineADeclarer,
) -> commandes::DeclarerUneOrigine {
    commandes::DeclarerUneOrigine {
        portee: commandes::Portee::Captation(Captation {
            sorte: commandes::Sorte::Prise,
            empreinte: empreinte(contenu),
        }),
        origine,
    }
}

/// La déclaration de l'origine `origine` sur les fichiers de l'import `import` sous `prefixe`.
fn declaration_par_prefixe(
    import: &str,
    prefixe: &str,
    origine: OrigineADeclarer,
) -> commandes::DeclarerUneOrigine {
    commandes::DeclarerUneOrigine {
        portee: commandes::Portee::Prefixe {
            import: import.to_string(),
            prefixe: prefixe.to_string(),
        },
        origine,
    }
}

/// L'origine déclarée que montre la vue de l'idée de la prise de contenu `contenu`, sur la
/// captation vue de cette prise.
fn origine_vue<D: Documents, C: Captations>(
    services: &Services<D, C>,
    contenu: &[u8],
) -> OrigineDeclaree {
    origine_vue_de(services, &empreinte(contenu))
}

/// L'origine déclarée que montre la vue de l'idée de la prise d'empreinte `e`, sur la captation
/// vue de cette prise : un fichier versé seul, ou le manifeste d'une prise en stems.
fn origine_vue_de<D: Documents, C: Captations>(
    services: &Services<D, C>,
    e: &str,
) -> OrigineDeclaree {
    let idee = lecture::lire_la_captation(fonds_de(services), gestes::Sorte::Prise, e)
        .expect("la prise est dans le fonds")
        .idee;
    services
        .vue_d_une_idee(&idee)
        .expect("l'idée de la prise se lit")
        .captations_vues
        .into_iter()
        .find(|vue| vue.captation.empreinte == e)
        .expect("la vue montre la prise")
        .origine
}

/// Les messages du journal du fonds de `services`, dans l'ordre de l'historique.
pub fn messages_du_journal<D: Documents, C: Captations>(services: &Services<D, C>) -> Vec<String> {
    services
        .journal()
        .expect("le journal se lit")
        .into_iter()
        .map(|entree| entree.message)
        .collect()
}

/// CAP-03 · 9 : un dossier `Atelier` déclaré généré par « Atelier inventé » (balayage Q3) ;
/// chacune de ses prises porte cette origine ; la déclarer `Aucune` est refusé ; le reverser, le
/// dossier déclaré de la main de Sacha, la laisse générée ; « de la main de Sacha » déclarée sur
/// l'une rend l'avertissement, sans rien écrire ; l'origine qu'elle porte déjà, déclarée de
/// nouveau, est faite sans avertissement ni écriture, comme `main` sur une prise du dossier `Main`
/// (R16) ; puis `main` déclarée par le préfixe d'un dossier qui porte une prise générée et une
/// prise sans origine pose `main` sur la seconde, en une opération, et rend l'avertissement (V9) ;
/// une prise d'un dossier non déclaré n'a aucune origine.
pub fn cap03_9_une_origine_generee_se_garde_et_ne_se_retire_pas<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 41);
    let b = wav_de(8_000, 42);
    let c = wav_de(8_000, 43);
    let d = wav_de(8_000, 44);
    let n = wav_de(8_000, 45);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Atelier/a.wav"), a.clone()),
            (Some("Atelier/b.wav"), b.clone()),
            (Some("Libre/d.wav"), d.clone()),
            (Some("Main/c.wav"), c.clone()),
        ],
        &[],
    );
    let atelier = OrigineDeclaree::Generee {
        noms: vec!["Atelier inventé".to_string()],
    };

    importer(
        &mut services,
        &source,
        &[
            ("Atelier", origine_du_dossier(generee("Atelier inventé"))),
            ("Main", origine_du_dossier(OrigineADeclarer::Main)),
        ],
        &mut contexte,
    );

    assert_eq!(origine_vue(&services, &a), atelier);
    assert_eq!(origine_vue(&services, &b), atelier);
    assert_eq!(origine_vue(&services, &c), OrigineDeclaree::Main);
    assert_eq!(origine_vue(&services, &d), OrigineDeclaree::Aucune);
    assert_eq!(
        services.declarer_une_origine(
            &declaration_sur_la_prise(&a, OrigineADeclarer::Aucune),
            &mut contexte
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Une origine déclarée ne se retire pas.".to_string(),
        })
    );

    importer(
        &mut services,
        &source,
        &[("Atelier", origine_du_dossier(OrigineADeclarer::Main))],
        &mut contexte,
    );

    assert_eq!(origine_vue(&services, &a), atelier);
    assert_eq!(origine_vue(&services, &b), atelier);
    let avant = messages_du_journal(&services);
    assert_eq!(
        services.declarer_une_origine(
            &declaration_sur_la_prise(&a, OrigineADeclarer::Main),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: Some(ORIGINE_GARDEE.to_string()),
        })
    );
    assert_eq!(
        services.declarer_une_origine(
            &declaration_sur_la_prise(&a, generee("Atelier inventé")),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(
        services.declarer_une_origine(
            &declaration_sur_la_prise(&c, OrigineADeclarer::Main),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(messages_du_journal(&services), avant);
    assert_eq!(origine_vue(&services, &a), atelier);
    assert!(
        !lecture::origines_de(fonds_de(&services), &empreinte(&a)).main,
        "« de la main de Sacha » s'est écrite sur une prise générée"
    );

    let melee = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Mêlé/g.wav"), a.clone()),
            (Some("Mêlé/n.wav"), n.clone()),
        ],
        &[],
    );
    let conduit = importer(&mut services, &melee, &[], &mut contexte);
    let avant = messages_du_journal(&services);

    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&conduit.ouvert.import, "Mêlé", OrigineADeclarer::Main),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: Some(ORIGINE_GARDEE.to_string()),
        })
    );

    let apres = messages_du_journal(&services);
    assert_eq!(apres.len(), avant.len() + 1);
    assert_eq!(
        apres.last().map(String::as_str),
        Some("Origine déclarée pour « Mêlé/ » : de la main de Sacha")
    );
    assert_eq!(origine_vue(&services, &n), OrigineDeclaree::Main);
    assert_eq!(origine_vue(&services, &a), atelier);
    assert_eq!(origine_vue(&services, &d), OrigineDeclaree::Aucune);
}

/// Balayage Q13 : `Démos/a.wav` et `Démos 2015/b.wav` ; le préfixe `Démos` ne prend que `a.wav`,
/// et le préfixe `Démos/`, sa barre finale ôtée, de même, la portée du message dite « « Démos/ » »
/// dans les deux cas ; le préfixe `Démo`, qui n'est pas un segment entier, ne prend rien.
pub fn declarer_une_origine_par_prefixe_ne_prend_que_les_segments_entiers<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 51);
    let b = wav_de(8_000, 52);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Démos/a.wav"), a.clone()),
            (Some("Démos 2015/b.wav"), b.clone()),
        ],
        &[],
    );
    let import = importer(&mut services, &source, &[], &mut contexte)
        .ouvert
        .import;

    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&import, "Démos", OrigineADeclarer::Main),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(
        messages_du_journal(&services).last().map(String::as_str),
        Some("Origine déclarée pour « Démos/ » : de la main de Sacha")
    );
    assert_eq!(origine_vue(&services, &a), OrigineDeclaree::Main);
    assert_eq!(origine_vue(&services, &b), OrigineDeclaree::Aucune);

    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&import, "Démos/", generee("Forge inventée")),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(
        messages_du_journal(&services).last().map(String::as_str),
        Some("Origine déclarée pour « Démos/ » : générée par Forge inventée")
    );
    assert_eq!(
        origine_vue(&services, &a),
        OrigineDeclaree::Generee {
            noms: vec!["Forge inventée".to_string()],
        }
    );
    assert_eq!(origine_vue(&services, &b), OrigineDeclaree::Aucune);

    let avant = messages_du_journal(&services);
    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&import, "Démo", OrigineADeclarer::Main),
            &mut contexte
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Cette captation n'est pas dans le fonds.".to_string(),
        })
    );
    assert_eq!(messages_du_journal(&services), avant);
}

/// R6 : un dossier `Démos/` de deux WAV, importé deux fois ; le second import ne pose aucun nom,
/// et sa table est seule gardée : la déclaration par le préfixe `Démos` qui nomme le premier
/// import est refusée ; celle qui nomme le second prend les deux captations. Après une nouvelle
/// inspection, la même commande est refusée, « Ce dossier n'est plus inspecté. ».
pub fn declarer_par_prefixe_apres_un_import_rejoue<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 61);
    let b = wav_de(8_000, 62);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Démos/a.wav"), a.clone()),
            (Some("Démos/b.wav"), b.clone()),
        ],
        &[],
    );
    let premier = importer(&mut services, &source, &[], &mut contexte);
    let second = importer(&mut services, &source, &[], &mut contexte);
    let plus_inspecte = Err(Refus {
        raison: Raison::Impossible,
        message: "Ce dossier n'est plus inspecté.".to_string(),
    });

    assert_eq!(
        lecture::noms_poses_par(fonds_de(&services), &second.ouvert.import),
        NomsPoses::default()
    );
    assert_eq!(second.clos.rapport.dedoublonnes.fichiers, 2);
    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&premier.ouvert.import, "Démos", OrigineADeclarer::Main),
            &mut contexte
        ),
        plus_inspecte
    );
    let par_le_second =
        declaration_par_prefixe(&second.ouvert.import, "Démos", OrigineADeclarer::Main);
    assert_eq!(
        services.declarer_une_origine(&par_le_second, &mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    let fonds = fonds_de(&services);
    assert!(lecture::origines_de(fonds, &empreinte(&a)).main);
    assert!(lecture::origines_de(fonds, &empreinte(&b)).main);

    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");

    assert_eq!(
        services.declarer_une_origine(&par_le_second, &mut contexte),
        plus_inspecte
    );
}

/// CAP-03 et ADR-024, § 3 : la déclaration d'un dossier versé porte sur la prise en stems de ce
/// dossier, dont le nom d'origine est celui du dossier, comme sur ses stems. `Jeu A`, importé jeu
/// de stems sans origine déclarée, puis `main` par le préfixe `Jeu A` : la prise que montre la vue
/// et ses deux stems portent `main`, en une seule opération ; `Libre/c.wav`, hors du préfixe,
/// aucune ; la même déclaration, de nouveau, n'écrit rien (R16). Puis un jeu au niveau du dossier
/// versé lui-même (`""`) : le préfixe `Jeu A` ne prend rien de ce second import, le préfixe `/`,
/// le préfixe vide, prend sa prise et ses stems.
pub fn declarer_par_prefixe_prend_la_prise_en_stems_de_son_dossier<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let basse = wav_de(8_000, 111);
    let voix = wav_de(8_000, 112);
    let c = wav_de(8_000, 113);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Jeu A/basse.wav"), basse.clone()),
            (Some("Jeu A/voix.wav"), voix.clone()),
            (Some("Libre/c.wav"), c.clone()),
        ],
        &[],
    );
    let import = importer(
        &mut services,
        &source,
        &[("Jeu A", jeu_de_stems())],
        &mut contexte,
    )
    .ouvert
    .import;
    let e_basse = empreinte(&basse);
    let e_voix = empreinte(&voix);
    let prise = empreinte(&rapport::manifeste(&[e_basse.as_str(), e_voix.as_str()]));
    assert_eq!(origine_vue_de(&services, &prise), OrigineDeclaree::Aucune);
    let avant = messages_du_journal(&services);

    let par_le_jeu = declaration_par_prefixe(&import, "Jeu A", OrigineADeclarer::Main);
    assert_eq!(
        services.declarer_une_origine(&par_le_jeu, &mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    let apres = messages_du_journal(&services);
    assert_eq!(apres.len(), avant.len() + 1);
    assert_eq!(
        apres.last().map(String::as_str),
        Some("Origine déclarée pour « Jeu A/ » : de la main de Sacha")
    );
    assert_eq!(origine_vue_de(&services, &prise), OrigineDeclaree::Main);
    let de_la_main = OriginesLues {
        main: true,
        generees: BTreeSet::new(),
    };
    let fonds = fonds_de(&services);
    assert_eq!(lecture::origines_de(fonds, &prise), de_la_main);
    assert_eq!(lecture::origines_de(fonds, &e_basse), de_la_main);
    assert_eq!(lecture::origines_de(fonds, &e_voix), de_la_main);
    assert_eq!(origine_vue(&services, &c), OrigineDeclaree::Aucune);

    assert_eq!(
        services.declarer_une_origine(&par_le_jeu, &mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(messages_du_journal(&services), apres);

    let nappe = wav_de(8_000, 114);
    let pad = wav_de(8_000, 115);
    let au_niveau_du_dossier = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("nappe.wav"), nappe.clone()),
            (Some("pad.wav"), pad.clone()),
        ],
        &[],
    );
    let second = importer(
        &mut services,
        &au_niveau_du_dossier,
        &[("", jeu_de_stems())],
        &mut contexte,
    )
    .ouvert
    .import;
    let e_nappe = empreinte(&nappe);
    let e_pad = empreinte(&pad);
    let seconde_prise = empreinte(&rapport::manifeste(&[e_nappe.as_str(), e_pad.as_str()]));
    assert_eq!(
        origine_vue_de(&services, &seconde_prise),
        OrigineDeclaree::Aucune
    );
    let avant = messages_du_journal(&services);
    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&second, "Jeu A", OrigineADeclarer::Main),
            &mut contexte
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Cette captation n'est pas dans le fonds.".to_string(),
        })
    );
    assert_eq!(messages_du_journal(&services), avant);

    assert_eq!(
        services.declarer_une_origine(
            &declaration_par_prefixe(&second, "/", generee("Forge inventée")),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    let apres = messages_du_journal(&services);
    assert_eq!(apres.len(), avant.len() + 1);
    assert_eq!(
        apres.last().map(String::as_str),
        Some("Origine déclarée pour « / » : générée par Forge inventée")
    );
    let de_la_forge = OrigineDeclaree::Generee {
        noms: vec!["Forge inventée".to_string()],
    };
    assert_eq!(origine_vue_de(&services, &seconde_prise), de_la_forge);
    let fonds = fonds_de(&services);
    for e in [&e_nappe, &e_pad] {
        assert_eq!(
            lecture::origines_de(fonds, e).generees,
            BTreeSet::from(["Forge inventée".to_string()])
        );
    }
    assert_eq!(origine_vue_de(&services, &prise), OrigineDeclaree::Main);
}

/// Le type d'un stem de la seule prise du fonds de `services`, tel que la vue le montre.
fn type_du_stem<D: Documents, C: Captations>(services: &Services<D, C>, stem: &str) -> TypeVu {
    la_seule_idee(services).captations_vues[0]
        .stems
        .iter()
        .find(|vu| vu.empreinte == stem)
        .expect("la prise porte ce stem")
        .type_de_stem
        .clone()
}

/// La chaîne que porte la clé `cle` sous `prises`, lue du fonds : absente, ou d'un autre type,
/// `None`.
fn chaine_sous_les_prises(fonds: &Automerge, cle: &str) -> Option<String> {
    let Ok(Some((Value::Object(ObjType::Map), objet))) = fonds.get(ROOT, PRISES) else {
        return None;
    };
    let (valeur, _) = fonds.get(&objet, cle).ok()??;
    valeur.into_string().ok()
}

/// Balayage Q15 : `Jeu A` déclaré jeu de stems, `voix.wav` et `basse.wav`. Le type de la voix,
/// proposé (`Voix`), puis gardé (`par` à `nom`, `depuis` au stem), puis choisi (`Choeurs`, sans
/// `par` ni `depuis`) ; la basse, proposée (`Basse`), choisie (`Voix`) sans avoir été gardée :
/// `choisir_le_type` ôte `type/par` et `type/depuis`, que rien n'a posés. Chaque choix a son
/// message ; un stem que la prise ne porte pas est refusé.
pub fn garder_puis_choisir_le_type_d_un_stem<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let voix = wav_de(8_000, 71);
    let basse = wav_de(8_000, 72);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Jeu A/basse.wav"), basse.clone()),
            (Some("Jeu A/voix.wav"), voix.clone()),
        ],
        &[],
    );
    importer(
        &mut services,
        &source,
        &[("Jeu A", jeu_de_stems())],
        &mut contexte,
    );
    let prise = la_seule_idee(&services).captations_vues[0]
        .captation
        .empreinte
        .clone();
    let e_voix = empreinte(&voix);
    let e_basse = empreinte(&basse);
    let choix = |stem: &str, choix: commandes::ChoixDuType| commandes::ChoisirLeType {
        prise: prise.clone(),
        stem: stem.to_string(),
        choix,
    };
    let sous_le_type = |stem: &str, cle: &str| format!("{prise}/{STEM}/{stem}/{TYPE}/{cle}");
    assert_eq!(
        type_du_stem(&services, &e_voix),
        TypeVu::Propose(commandes::TypeDeStem::Voix)
    );
    assert_eq!(
        type_du_stem(&services, &e_basse),
        TypeVu::Propose(commandes::TypeDeStem::Basse)
    );

    assert_eq!(
        services.choisir_le_type_d_un_stem(
            &choix(&e_voix, commandes::ChoixDuType::Garder),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(
        type_du_stem(&services, &e_voix),
        TypeVu::Garde(commandes::TypeDeStem::Voix)
    );
    let fonds = fonds_de(&services);
    assert_eq!(
        chaine_sous_les_prises(fonds, &sous_le_type(&e_voix, PAR)).as_deref(),
        Some(PAR_LE_NOM)
    );
    assert_eq!(
        chaine_sous_les_prises(fonds, &sous_le_type(&e_voix, DEPUIS)).as_deref(),
        Some(e_voix.as_str())
    );
    assert_eq!(
        messages_du_journal(&services).last().map(String::as_str),
        Some("Type de « Jeu A/voix.wav » : voix")
    );

    assert_eq!(
        services.choisir_le_type_d_un_stem(
            &choix(
                &e_voix,
                commandes::ChoixDuType::Choisir(commandes::TypeDeStem::Choeurs)
            ),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(
        type_du_stem(&services, &e_voix),
        TypeVu::Choisi(commandes::TypeDeStem::Choeurs)
    );
    let fonds = fonds_de(&services);
    assert_eq!(
        chaine_sous_les_prises(fonds, &sous_le_type(&e_voix, PAR)),
        None
    );
    assert_eq!(
        chaine_sous_les_prises(fonds, &sous_le_type(&e_voix, DEPUIS)),
        None
    );
    assert_eq!(
        messages_du_journal(&services).last().map(String::as_str),
        Some("Type de « Jeu A/voix.wav » : chœurs")
    );

    assert_eq!(
        services.choisir_le_type_d_un_stem(
            &choix(
                &e_basse,
                commandes::ChoixDuType::Choisir(commandes::TypeDeStem::Voix)
            ),
            &mut contexte
        ),
        Ok(Faite {
            avertissement: None
        })
    );
    assert_eq!(
        type_du_stem(&services, &e_basse),
        TypeVu::Choisi(commandes::TypeDeStem::Voix)
    );
    let fonds = fonds_de(&services);
    assert_eq!(
        chaine_sous_les_prises(fonds, &sous_le_type(&e_basse, PAR)),
        None
    );
    assert_eq!(
        chaine_sous_les_prises(fonds, &sous_le_type(&e_basse, DEPUIS)),
        None
    );
    assert_eq!(
        messages_du_journal(&services).last().map(String::as_str),
        Some("Type de « Jeu A/basse.wav » : voix")
    );

    let avant = messages_du_journal(&services);
    assert_eq!(
        services.choisir_le_type_d_un_stem(
            &choix(
                &empreinte(b"un stem d'une autre prise"),
                commandes::ChoixDuType::Garder
            ),
            &mut contexte
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Ce stem n'est pas dans cette prise.".to_string(),
        })
    );
    assert_eq!(messages_du_journal(&services), avant);
}

/// Le texte versé de contenu `contenu`, tel que `lire_un_texte_verse` le rend.
fn texte_lu<D: Documents, C: Captations>(
    services: &Services<D, C>,
    contenu: &[u8],
) -> Reponse<commandes::TexteLu> {
    services.lire_un_texte_verse(&Captation {
        sorte: commandes::Sorte::TexteVerse,
        empreinte: empreinte(contenu),
    })
}

/// Balayage P3, Q9 ; I31 : un ChordPro en UTF-8 avec sa marque, un texte en UTF-16
/// petit-boutiste, que sa marque `FF FE` fait reconnaître en texte, non en prise `mp3`, un texte
/// en Latin-1, illisible, et un Markdown qui porte `<script>` et `&amp;` ; chacun se relit décodé,
/// sans sa marque, le dernier octet pour octet ; l'illisible, `None`. Une prise, ou une empreinte
/// qu'aucun texte versé ne porte, est refusée.
pub fn lire_un_texte_verse_le_rend_tel_quel<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let accords = "{title: Pont du soir}\n[Am]Un pont qui hésite\n";
    let chordpro = [b"\xEF\xBB\xBF".as_slice(), accords.as_bytes()].concat();
    let refrain = "Un refrain, là-bas\n";
    let mut utf16 = vec![0xFF, 0xFE];
    for unite in refrain.encode_utf16() {
        utf16.extend(unite.to_le_bytes());
    }
    let latin = b"Un \xe9t\xe9 en r\xe9\n".to_vec();
    let piege = "<script>alert('x')</script> &amp; &lt;b&gt;\n";
    let prise = wav_de(8_000, 73);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Accords.cho"), chordpro.clone()),
            (Some("Latin.txt"), latin.clone()),
            (Some("Paroles.txt"), utf16.clone()),
            (Some("Piege.md"), piege.as_bytes().to_vec()),
            (Some("pont.wav"), prise.clone()),
        ],
        &[],
    );

    importer(&mut services, &source, &[], &mut contexte);

    let fonds = fonds_de(&services);
    let e_utf16 = empreinte(&utf16);
    assert!(
        lecture::lire_la_captation(fonds, gestes::Sorte::TexteVerse, &e_utf16).is_some(),
        "le texte UTF-16 n'est pas un texte versé"
    );
    assert!(
        lecture::lire_la_captation(fonds, gestes::Sorte::Prise, &e_utf16).is_none(),
        "le texte UTF-16 est versé en prise"
    );
    assert_eq!(
        format_de(fonds, TEXTES_VERSES, &e_utf16).as_deref(),
        Some(FormatTexte::Brut.nom())
    );
    assert_eq!(
        texte_lu(&services, &chordpro),
        Ok(commandes::TexteLu {
            texte: Some(accords.to_string()),
        })
    );
    assert_eq!(
        texte_lu(&services, &utf16),
        Ok(commandes::TexteLu {
            texte: Some(refrain.to_string()),
        })
    );
    assert_eq!(
        texte_lu(&services, &latin),
        Ok(commandes::TexteLu { texte: None })
    );
    let relu = texte_lu(&services, piege.as_bytes())
        .expect("le Markdown se relit")
        .texte
        .expect("le Markdown est lisible");
    assert_eq!(relu.as_bytes(), piege.as_bytes());

    let absente = Err(Refus {
        raison: Raison::Impossible,
        message: "Cette captation n'est pas dans le fonds.".to_string(),
    });
    assert_eq!(
        services.lire_un_texte_verse(&Captation {
            sorte: commandes::Sorte::Prise,
            empreinte: empreinte(&prise),
        }),
        absente
    );
    assert_eq!(
        texte_lu(&services, b"un texte jamais vers\xc3\xa9"),
        absente
    );
}

/// Les comptes du rapport que `clore_l_import` a rendus, relus par `lire_un_rapport` : un WAV, un
/// WAV d'un dossier déclaré généré, une archive ignorée et un texte illisible. Un import que le
/// fonds ne porte pas, puis le même, ouvert sans rapport, sont refusés ; son rapport d'une version
/// plus récente, refusé de `Raison::Illisible`.
pub fn lire_un_rapport_rend_ses_comptes<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Archives/demos.zip"), zip(100)),
            (Some("Atelier/nappe.wav"), wav_de(8_000, 81)),
            (Some("Paroles.txt"), b"Un \xe9t\xe9\n".to_vec()),
            (Some("pont.wav"), wav_de(8_000, 82)),
        ],
        &[],
    );
    let conduit = importer(
        &mut services,
        &source,
        &[("Atelier", origine_du_dossier(generee("Atelier inventé")))],
        &mut contexte,
    );
    let rapport = conduit.clos.rapport;
    assert_eq!(rapport.verses.fichiers, 3);
    assert_eq!(rapport.ignores.fichiers, 1);
    assert_eq!(rapport.illisibles.fichiers, 1);
    assert_eq!(rapport.origines_changees, 1);

    assert_eq!(
        services.lire_un_rapport(&commandes::LireUnRapport {
            import: conduit.ouvert.import.clone(),
        }),
        Ok(rapport)
    );

    let sans_rapport = Err(Refus {
        raison: Raison::Impossible,
        message: "Cet import n'a pas de rapport.".to_string(),
    });
    let lire = commandes::LireUnRapport {
        import: IMPORT.to_string(),
    };
    assert_eq!(services.lire_un_rapport(&lire), sans_rapport);
    ouvrir_l_import(&mut services, &ouvert_a(IMPORT, HEURE), &[]);
    assert_eq!(services.lire_un_rapport(&lire), sans_rapport);
    let plus_recent = services
        .captations()
        .verser(format!("pupitre-rapport-d-import 2\n{IMPORT}\n").as_bytes())
        .expect("le rapport d'une version plus récente se verse");
    ecrire_l_operation(
        &mut services,
        &import::clore(&ouvert_a(IMPORT, HEURE), &plus_recent, HEURE, 0),
    );
    assert_eq!(
        services.lire_un_rapport(&lire),
        Err(Refus {
            raison: Raison::Illisible,
            message: "Ce rapport vient d'une version plus récente.".to_string(),
        })
    );
}

/// Question 22 : les générateurs d'un Pupitre neuf, aucun ; puis ceux d'un fonds où « Atelier
/// inventé » est déclaré sur un dossier et porté par sa prise, « Forge inventée » déclaré seul, sur
/// un dossier sans fichier, et « Studio rêvé » porté seul, déclaré après l'import sur une prise :
/// chacun une fois, dans l'ordre de leurs octets.
pub fn les_generateurs_du_fonds_chacun_une_fois<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    assert_eq!(services.generateurs(), Ok(Vec::new()));
    let nappe = wav_de(8_000, 91);
    let libre = wav_de(8_000, 92);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Atelier/nappe.wav"), nappe.clone()),
            (Some("libre.wav"), libre.clone()),
        ],
        &[],
    );
    let import = importer(
        &mut services,
        &source,
        &[
            ("Atelier", origine_du_dossier(generee("Atelier inventé"))),
            ("Vide", origine_du_dossier(generee("Forge inventée"))),
        ],
        &mut contexte,
    )
    .ouvert
    .import;
    services
        .declarer_une_origine(
            &declaration_sur_la_prise(&libre, generee("Studio rêvé")),
            &mut contexte,
        )
        .expect("l'origine se déclare");

    let fonds = fonds_de(&services);
    assert_eq!(
        lecture::origines_de(fonds, &empreinte(&nappe)).generees,
        BTreeSet::from(["Atelier inventé".to_string()])
    );
    assert_eq!(
        lecture::declarations_de(fonds, &import)
            .get("Vide")
            .and_then(|declaration| declaration.origine.clone()),
        Some(Ajout::Generee("Forge inventée".to_string()))
    );
    assert!(
        !lecture::declarations_de(fonds, &import)
            .values()
            .any(|declaration| declaration.origine
                == Some(Ajout::Generee("Studio rêvé".to_string()))),
        "« Studio rêvé » est déclaré sur un dossier"
    );
    assert_eq!(
        services.generateurs(),
        Ok(vec![
            "Atelier inventé".to_string(),
            "Forge inventée".to_string(),
            "Studio rêvé".to_string(),
        ])
    );
}

// La reprise d'un import (tâche 14) : rechoisir le dossier d'un import resté ouvert le reprend,
// sous le même ULID ; un fichier changé depuis l'inspection arrête l'import ; un import ouvert
// ailleurs se clôt à l'ouverture du suivant ; l'état du fonds dit l'import resté ouvert.

/// L'avertissement de l'état du fonds quand un import est resté ouvert (question 2).
const IMPORT_RESTE_OUVERT: &str = "Un import est resté ouvert : rechoisir son dossier le reprend.";

/// Le refus d'une reprise qui changerait les déclarations de l'ouverture (R2).
const DECLARATIONS_REPRISES: &str = "Cet import reprend ses déclarations : elles ne changent pas.";

/// Le refus du versement de `b.wav`, changé depuis l'inspection (question 14).
const B_A_CHANGE: &str = "« b.wav » a changé depuis l'inspection : l'import s'arrête, et \
                          rechoisir le dossier le reprend.";

/// La commande qui ouvre l'import, ses dossiers déclarés `declarations`, au décalage local nul.
pub fn ouverture(declarations: &[(&str, commandes::Declaration)]) -> OuvrirLImport {
    OuvrirLImport {
        declarations: declarations
            .iter()
            .map(|(chemin, declaration)| DeclarationDUnDossier {
                chemin: chemin.to_string(),
                declaration: declaration.clone(),
            })
            .collect(),
        decalage_minutes: 0,
    }
}

/// La commande qui clôt l'import `import`, au décalage local nul.
pub fn cloture(import: &str) -> CloreLImport {
    CloreLImport {
        import: import.to_string(),
        decalage_minutes: 0,
    }
}

/// Un WAV des tests de l'import coupé à 30 octets, dans son morceau `fmt `, avant l'en-tête de
/// `data` : sa signature le dit WAV, son en-tête ne se lit pas (illisible ; V8). Tous les WAV
/// coupés ont le même contenu.
pub fn wav_coupe() -> Vec<u8> {
    let mut octets = wav_de(8_000, 0);
    octets.truncate(30);
    octets
}

/// Le nombre des avertissements de l'état du fonds de `services` qui disent un import resté
/// ouvert.
fn avertit_d_un_import_reste_ouvert<D: Documents, C: Captations>(
    services: &Services<D, C>,
) -> usize {
    services
        .etat_du_fonds()
        .expect("l'état du fonds se lit")
        .avertissements
        .iter()
        .filter(|avertissement| *avertissement == IMPORT_RESTE_OUVERT)
        .count()
}

/// L'import `import`, tel que le fonds de `services` le porte.
fn import_lu<D: Documents, C: Captations>(services: &Services<D, C>, import: &str) -> ImportLu {
    lecture::imports(fonds_de(services))
        .into_iter()
        .find(|lu| lu.import == import)
        .unwrap_or_else(|| panic!("le fonds ne porte pas l'import {import}"))
}

/// Point de revue 4, sans coupure ; R7 ; V8 : huit fichiers, dont deux du même contenu, un sans
/// nom, un WAV coupé à 30 octets (illisible) et un texte en Latin-1 (illisible). Les trois premiers
/// versés, dont ce WAV et ce texte, puis les services rouverts, qui perdent leur état ;
/// l'inspection du même dossier dit la reprise de l'import, sans import ouvert ailleurs ;
/// l'ouverture le reprend, sous le même ULID, et ne rend que les rangs restants ; sa clôture rend
/// les comptes d'un import fait d'une traite du même dossier, sur `paire_neuve`, illisibles et
/// octets compris.
pub fn une_reprise_rend_le_rapport_d_une_traite<P: Paire>(paire: &mut P, paire_neuve: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let coupe = wav_coupe();
    let latin = b"Un \xe9t\xe9 en r\xe9\n".to_vec();
    let double = wav_de(8_000, 43);
    let fichiers: [(Option<&str>, Vec<u8>); 8] = [
        (Some("A coupe.wav"), coupe.clone()),
        (Some("B paroles.txt"), latin.clone()),
        (Some("C essai.wav"), wav_de(8_000, 42)),
        (Some("D double.wav"), double.clone()),
        (Some("E double.wav"), double.clone()),
        (Some("F pont.mid"), midi(1, 2, 480)),
        (Some("G notes.txt"), b"Un pont qui hesite\n".to_vec()),
        (None, wav_de(8_000, 44)),
    ];
    let source = paire.source(Some(DOSSIER_VERSE), &fichiers, &[]);
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let premiers: Vec<Option<&str>> = source.entrees()[..3]
        .iter()
        .map(|entree| entree.chemin.as_deref())
        .collect();
    assert_eq!(
        premiers,
        vec![
            Some("A coupe.wav"),
            Some("B paroles.txt"),
            Some("C essai.wav")
        ]
    );
    let ouvert = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import s'ouvre");
    for rang in 1..=3 {
        verser_le_rang(&mut services, &source, &ouvert.import, rang, &mut contexte);
    }

    let mut services = paire.rouvrir(services, &[]);
    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(ouvert.import.as_str()));
    assert!(
        !inspection.ouvert_ailleurs,
        "l'import repris est dit ouvert ailleurs"
    );
    let reprise = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import se reprend");
    assert_eq!(reprise.import, ouvert.import);
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.rangs, vec![4, 5, 6, 7, 8]);
    assert_eq!(reprise.jeux, Vec::<commandes::JeuAVerser>::new());
    for rang in &reprise.rangs {
        verser_le_rang(&mut services, &source, &ouvert.import, *rang, &mut contexte);
    }
    let clos = services
        .clore_l_import(&source, &cloture(&ouvert.import), &mut contexte)
        .expect("l'import repris se clôt");

    let mut neufs = paire_neuve.neufs();
    let mut alea_neuf = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let source_neuve = paire_neuve.source(Some(DOSSIER_VERSE), &fichiers, &[]);
    let traite = importer(
        &mut neufs,
        &source_neuve,
        &[],
        &mut contexte_de_test(&mut alea_neuf),
    );
    assert_eq!(
        clos.rapport,
        RapportDImport {
            import: ouvert.import.clone(),
            ..traite.clos.rapport
        }
    );
    assert_eq!(clos.rapport.vus.fichiers, 8);
    assert_eq!(
        clos.rapport.illisibles,
        compte_des(&[coupe.as_slice(), latin.as_slice()])
    );
    assert_eq!(clos.rapport.dedoublonnes, compte_des(&[double.as_slice()]));
    assert_eq!(clos.rapport.sans_nom.fichiers, 1);
}

/// R2 ; balayage Q3, Q4 et Q13 : `Jeu A` déclaré jeu de stems sans origine, `Atelier` déclaré
/// généré par « Atelier inventé », sans jeu ; dans `Jeu A`, un stem illisible (un WAV coupé à 30
/// octets, sans faits) et sans origine. La prise en stems versée, puis les services rouverts ;
/// l'inspection du même dossier dit la reprise et rend les déclarations de l'ouverture ; la
/// reprise ne rend que le rang d'`Atelier`, sans jeu à verser ; le stem illisible reste nommé et
/// rangé, sans copie de l'import ; la clôture rend les comptes d'un import fait d'une traite, sur
/// `paire_neuve`. Un stem sans nom en UTF-8 n'a pas de dossier et ne rejoint aucun jeu : il ne se
/// forme pas par la source, et son entrée, sa seule clé, est éprouvée au noyau.
pub fn un_import_coupe_apres_une_prise_en_stems_se_reprend<P: Paire>(
    paire: &mut P,
    paire_neuve: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let voix = wav_coupe();
    let fichiers: [(Option<&str>, Vec<u8>); 3] = [
        (Some("Atelier/nappe.wav"), wav_de(8_000, 51)),
        (Some("Jeu A/basse.wav"), wav_de(8_000, 52)),
        (Some("Jeu A/voix.wav"), voix.clone()),
    ];
    let declarations = [
        ("Jeu A", jeu_de_stems()),
        ("Atelier", origine_du_dossier(generee("Atelier inventé"))),
    ];
    let source = paire.source(Some(DOSSIER_VERSE), &fichiers, &[]);
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let ouvert = services
        .ouvrir_l_import(
            &source,
            &ouverture(&declarations),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre");
    assert_eq!(ouvert.rangs, vec![1]);
    assert_eq!(
        ouvert.jeux,
        vec![commandes::JeuAVerser {
            dossier: "Jeu A".to_string(),
            rangs: vec![2, 3],
        }]
    );
    for rang in [2, 3] {
        let verse = verser_le_rang(&mut services, &source, &ouvert.import, rang, &mut contexte);
        assert_eq!(verse.issue, Issue::Stem, "le fichier n° {rang}");
    }
    services
        .verser_une_prise_en_stems(
            &source,
            &VerserUnePriseEnStems {
                import: ouvert.import.clone(),
                dossier: "Jeu A".to_string(),
                decalage_minutes: 0,
            },
            &mut contexte,
        )
        .expect("la prise en stems se verse");

    let mut services = paire.rouvrir(services, &[]);
    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(ouvert.import.as_str()));
    assert_eq!(
        inspection.declarations_reprises,
        vec![
            DeclarationDUnDossier {
                chemin: "Atelier".to_string(),
                declaration: origine_du_dossier(generee("Atelier inventé")),
            },
            DeclarationDUnDossier {
                chemin: "Jeu A".to_string(),
                declaration: jeu_de_stems(),
            },
        ]
    );
    let reprise = services
        .ouvrir_l_import(
            &source,
            &ouverture(&declarations),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.rangs, vec![1]);
    assert_eq!(reprise.jeux, Vec::<commandes::JeuAVerser>::new());
    let e_voix = empreinte(&voix);
    assert!(
        lecture::empreintes_nommees(fonds_de(&services)).contains(&e_voix),
        "le stem illisible n'est plus nommé"
    );
    assert_eq!(
        services.captations().taille(&e_voix),
        Ok(Some(voix.len() as u64))
    );
    assert_eq!(
        services.captations().copies_de_l_import(&ouvert.import),
        Ok(Vec::new())
    );
    verser_le_rang(&mut services, &source, &ouvert.import, 1, &mut contexte);
    let clos = services
        .clore_l_import(&source, &cloture(&ouvert.import), &mut contexte)
        .expect("l'import repris se clôt");

    let mut neufs = paire_neuve.neufs();
    let mut alea_neuf = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let source_neuve = paire_neuve.source(Some(DOSSIER_VERSE), &fichiers, &[]);
    let traite = importer(
        &mut neufs,
        &source_neuve,
        &declarations,
        &mut contexte_de_test(&mut alea_neuf),
    );
    assert_eq!(
        clos.rapport,
        RapportDImport {
            import: ouvert.import.clone(),
            ..traite.clos.rapport
        }
    );
    assert_eq!(clos.rapport.illisibles, compte_des(&[voix.as_slice()]));
    assert_eq!(clos.rapport.origines_changees, 1);
}

/// R2 ; R23 : le dossier versé, sans nom en UTF-8, déclaré jeu de stems ; ses deux stems et sa
/// prise en stems versés, puis les services rouverts. La prise n'a pas de nom, ses stems gardent
/// le leur, à l'ULID de l'import : l'inspection dit la reprise ; l'ouverture reverse le jeu en
/// entier, ses stems comme sa prise, connue ; la clôture rend les comptes d'un import fait d'une
/// traite, sur `paire_neuve`.
pub fn une_reprise_dont_la_prise_n_a_pas_de_nom_reverse_le_jeu<P: Paire>(
    paire: &mut P,
    paire_neuve: &mut P,
) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let fichiers = [
        (Some("basse.wav"), wav_de(8_000, 55)),
        (Some("voix.wav"), wav_de(8_000, 56)),
    ];
    let declarations = [("", jeu_de_stems())];
    let jeu_entier = vec![commandes::JeuAVerser {
        dossier: String::new(),
        rangs: vec![1, 2],
    }];
    let source = paire.source(None, &fichiers, &[]);
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let ouvert = services
        .ouvrir_l_import(
            &source,
            &ouverture(&declarations),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre");
    assert_eq!(ouvert.jeux, jeu_entier);
    verser_le_jeu_entier(&mut services, &source, &ouvert.import, "", &mut contexte);

    let mut services = paire.rouvrir(services, &[]);
    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(ouvert.import.as_str()));
    let reprise = services
        .ouvrir_l_import(
            &source,
            &ouverture(&declarations),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.rangs, Vec::<u64>::new());
    assert_eq!(reprise.jeux, jeu_entier);
    verser_le_jeu_entier(&mut services, &source, &ouvert.import, "", &mut contexte);
    let clos = services
        .clore_l_import(&source, &cloture(&ouvert.import), &mut contexte)
        .expect("l'import repris se clôt");

    let mut neufs = paire_neuve.neufs();
    let mut alea_neuf = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let source_neuve = paire_neuve.source(None, &fichiers, &[]);
    let traite = importer(
        &mut neufs,
        &source_neuve,
        &declarations,
        &mut contexte_de_test(&mut alea_neuf),
    );
    assert_eq!(
        clos.rapport,
        RapportDImport {
            import: ouvert.import.clone(),
            ..traite.clos.rapport
        }
    );
}

/// R2 : un premier import, clos, verse le jeu `Jeu A` (`basse.wav`, `voix.wav`) ; un second, du
/// même jeu dont `voix.wav` est renommé `voix reprise.wav`, verse ses stems et sa prise en stems,
/// connue : le nom de la prise et celui de `basse.wav`, posés par le premier, ne se récrivent pas,
/// et seul le stem renommé reçoit un nom à l'ULID du second. Les services rouverts, l'inspection
/// du second dossier dit la reprise ; l'ouverture reverse le jeu en entier ; la clôture rend les
/// comptes d'un second import fait d'une traite, après le même premier, sur `paire_neuve`.
pub fn une_reprise_d_une_prise_connue_au_stem_renomme_reverse_le_jeu<P: Paire>(
    paire: &mut P,
    paire_neuve: &mut P,
) {
    let basse = wav_de(8_000, 57);
    let voix = wav_de(8_000, 58);
    let premiers = [
        (Some("Jeu A/basse.wav"), basse.clone()),
        (Some("Jeu A/voix.wav"), voix.clone()),
    ];
    let seconds = [
        (Some("Jeu A/basse.wav"), basse),
        (Some("Jeu A/voix reprise.wav"), voix),
    ];
    let declarations = [("Jeu A", jeu_de_stems())];
    let jeu_entier = vec![commandes::JeuAVerser {
        dossier: "Jeu A".to_string(),
        rangs: vec![1, 2],
    }];
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let premier = paire.source(Some(DOSSIER_VERSE), &premiers, &[]);
    importer(&mut services, &premier, &declarations, &mut contexte);
    let second = paire.source(Some(DOSSIER_VERSE), &seconds, &[]);
    services
        .inspecter(&second, PLACE_LIBRE)
        .expect("le second dossier s'inspecte");
    let ouvert = services
        .ouvrir_l_import(
            &second,
            &ouverture(&declarations),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("le second import s'ouvre");
    assert_eq!(ouvert.jeux, jeu_entier);
    verser_le_jeu_entier(
        &mut services,
        &second,
        &ouvert.import,
        "Jeu A",
        &mut contexte,
    );

    let mut services = paire.rouvrir(services, &[]);
    let inspection = services
        .inspecter(&second, PLACE_LIBRE)
        .expect("le second dossier s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(ouvert.import.as_str()));
    let reprise = services
        .ouvrir_l_import(
            &second,
            &ouverture(&declarations),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("le second import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.jeux, jeu_entier);
    verser_le_jeu_entier(
        &mut services,
        &second,
        &ouvert.import,
        "Jeu A",
        &mut contexte,
    );
    let clos = services
        .clore_l_import(&second, &cloture(&ouvert.import), &mut contexte)
        .expect("le second import repris se clôt");

    let mut neufs = paire_neuve.neufs();
    let mut alea_neuf = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte_neuf = contexte_de_test(&mut alea_neuf);
    let premier_neuf = paire_neuve.source(Some(DOSSIER_VERSE), &premiers, &[]);
    importer(&mut neufs, &premier_neuf, &declarations, &mut contexte_neuf);
    let second_neuf = paire_neuve.source(Some(DOSSIER_VERSE), &seconds, &[]);
    let traite = importer(&mut neufs, &second_neuf, &declarations, &mut contexte_neuf);
    assert_eq!(
        clos.rapport,
        RapportDImport {
            import: ouvert.import.clone(),
            ..traite.clos.rapport
        }
    );
}

/// Verse chaque stem du jeu `dossier` de l'import `import`, aux rangs 1 et 2, chacun en
/// `Issue::Stem`, puis sa prise en stems.
fn verser_le_jeu_entier<D: Documents, C: Captations, S: Source>(
    services: &mut Services<D, C>,
    source: &S,
    import: &str,
    dossier: &str,
    contexte: &mut Contexte<'_>,
) {
    for rang in [1, 2] {
        let verse = verser_le_rang(services, source, import, rang, contexte);
        assert_eq!(verse.issue, Issue::Stem, "le fichier n° {rang}");
    }
    services
        .verser_une_prise_en_stems(
            source,
            &VerserUnePriseEnStems {
                import: import.to_string(),
                dossier: dossier.to_string(),
                decalage_minutes: 0,
            },
            contexte,
        )
        .expect("la prise en stems se verse");
}

/// R2 ; question 35 : un import ouvert, le dossier versé déclaré de la main de Sacha, rien versé,
/// puis les services rouverts ; l'inspection du même dossier ne dit aucune reprise, aucune
/// déclaration reprise, et dit l'import ouvert ailleurs : sans nom posé, aucun import ne se
/// reprend.
pub fn un_import_ouvert_sans_nom_ne_se_reprend_pas<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("a.wav"), wav_de(8_000, 11)),
            (Some("b.wav"), wav_de(8_000, 12)),
        ],
        &[],
    );
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    services
        .ouvrir_l_import(
            &source,
            &ouverture(&[("", origine_du_dossier(OrigineADeclarer::Main))]),
            PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre");

    let mut services = paire.rouvrir(services, &[]);
    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");

    assert_eq!(inspection.reprise, None);
    assert_eq!(
        inspection.declarations_reprises,
        Vec::<DeclarationDUnDossier>::new()
    );
    assert!(
        inspection.ouvert_ailleurs,
        "l'import ouvert sans nom n'est pas dit ouvert ailleurs"
    );
}

/// R2 ; balayage Q3 et Q13 : un import ouvert, `Démos` déclaré de la main de Sacha, sans jeu, un
/// fichier versé, puis les services rouverts ; l'inspection rend cette déclaration, reprise ; une
/// ouverture qui déclare `Démos` jeu de stems, son origine gardée, est refusée ; sans déclaration,
/// puis, inspecté de nouveau, avec les mêmes, l'import se reprend, sans rien écrire ; le second
/// fichier versé porte l'origine de l'ouverture.
pub fn une_reprise_garde_les_declarations_de_l_ouverture<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let b = wav_de(8_000, 22);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("Démos/a.wav"), wav_de(8_000, 21)),
            (Some("Démos/b.wav"), b.clone()),
        ],
        &[],
    );
    let demos = [("Démos", origine_du_dossier(OrigineADeclarer::Main))];
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(&source, &ouverture(&demos), PLACE_LIBRE, &mut contexte)
        .expect("l'import s'ouvre")
        .import;
    verser_le_rang(&mut services, &source, &import, 1, &mut contexte);

    let mut services = paire.rouvrir(services, &[]);
    let inspection = services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(import.as_str()));
    assert_eq!(
        inspection.declarations_reprises,
        vec![DeclarationDUnDossier {
            chemin: "Démos".to_string(),
            declaration: origine_du_dossier(OrigineADeclarer::Main),
        }]
    );
    let journal = messages_du_journal(&services);
    let changees = [(
        "Démos",
        commandes::Declaration {
            stems: true,
            origine: OrigineADeclarer::Main,
        },
    )];
    assert_eq!(
        services.ouvrir_l_import(&source, &ouverture(&changees), PLACE_LIBRE, &mut contexte),
        Err(Refus {
            raison: Raison::Impossible,
            message: DECLARATIONS_REPRISES.to_string(),
        })
    );
    let sans = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("sans déclaration, l'import se reprend");
    assert!(sans.repris, "l'ouverture sans déclaration ne reprend pas");
    assert_eq!(sans.rangs, vec![2]);
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte une troisième fois");
    let memes = services
        .ouvrir_l_import(&source, &ouverture(&demos), PLACE_LIBRE, &mut contexte)
        .expect("avec les mêmes déclarations, l'import se reprend");
    assert!(
        memes.repris,
        "l'ouverture aux mêmes déclarations ne reprend pas"
    );
    assert_eq!(memes.import, import);
    assert_eq!(memes.rangs, vec![2]);
    assert_eq!(messages_du_journal(&services), journal);

    verser_le_rang(&mut services, &source, &import, 2, &mut contexte);
    services
        .clore_l_import(&source, &cloture(&import), &mut contexte)
        .expect("l'import repris se clôt");
    assert_eq!(origine_vue(&services, &b), OrigineDeclaree::Main);
}

/// Question 14 : `a.wav`, `b.wav` et `c.wav` ; l'import ouvert, `a.wav` versé ; `b.wav` récrit à
/// une autre taille après l'inspection ; son versement est refusé, et le dit. Rechoisir le
/// dossier, une nouvelle source de ses mêmes chemins, le reprend : `b.wav` changé se verse, et le
/// rapport compte ses octets.
pub fn un_fichier_change_depuis_l_inspection_arrete_l_import<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 61);
    let c = wav_de(8_000, 63);
    let change = wav_de(4_000, 64);
    let mut source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("a.wav"), a.clone()),
            (Some("b.wav"), wav_de(8_000, 62)),
            (Some("c.wav"), c.clone()),
        ],
        &[],
    );
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import s'ouvre")
        .import;
    verser_le_rang(&mut services, &source, &import, 1, &mut contexte);

    paire.changer(&mut source, 2, &change);
    assert_eq!(
        services.verser_un_fichier(
            &source,
            &VerserUnFichier {
                import: import.clone(),
                rang: 2,
                decalage_minutes: 0,
            },
            &mut contexte,
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: B_A_CHANGE.to_string(),
        })
    );

    let rechoisie = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("a.wav"), a.clone()),
            (Some("b.wav"), change.clone()),
            (Some("c.wav"), c.clone()),
        ],
        &[],
    );
    let inspection = services
        .inspecter(&rechoisie, PLACE_LIBRE)
        .expect("le dossier rechoisi s'inspecte");
    assert_eq!(inspection.reprise.as_deref(), Some(import.as_str()));
    let reprise = services
        .ouvrir_l_import(&rechoisie, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.rangs, vec![2, 3]);
    assert_eq!(
        verser_le_rang(&mut services, &rechoisie, &import, 2, &mut contexte).issue,
        Issue::Verse
    );
    verser_le_rang(&mut services, &rechoisie, &import, 3, &mut contexte);
    let clos = services
        .clore_l_import(&rechoisie, &cloture(&import), &mut contexte)
        .expect("l'import repris se clôt");
    assert_eq!(
        clos.rapport.verses,
        compte_des(&[a.as_slice(), change.as_slice(), c.as_slice()])
    );
}

/// Question 14 ; R8 : `a.wav`, `b.wav` et `c.wav` ; l'import ouvert, `a.wav` versé ; `b.wav`
/// récrit à la même taille après l'inspection, sa marque de dernière écriture avancée ; son
/// versement est refusé du même message ; aucune captation ne porte le contenu changé, et aucun
/// nom provisoire de l'import ne reste. Rechoisir le dossier le reprend, et le contenu changé se
/// verse.
pub fn un_contenu_change_a_taille_egale_arrete_l_import<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 65);
    let b = wav_de(8_000, 66);
    let c = wav_de(8_000, 67);
    let change = wav_de(8_000, 68);
    assert_eq!(change.len(), b.len());
    let mut source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("a.wav"), a.clone()),
            (Some("b.wav"), b),
            (Some("c.wav"), c.clone()),
        ],
        &[],
    );
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import s'ouvre")
        .import;
    verser_le_rang(&mut services, &source, &import, 1, &mut contexte);

    paire.changer(&mut source, 2, &change);
    assert_eq!(
        services.verser_un_fichier(
            &source,
            &VerserUnFichier {
                import: import.clone(),
                rang: 2,
                decalage_minutes: 0,
            },
            &mut contexte,
        ),
        Err(Refus {
            raison: Raison::Impossible,
            message: B_A_CHANGE.to_string(),
        })
    );
    let e_change = empreinte(&change);
    assert_eq!(services.captations().taille(&e_change), Ok(None));
    assert_eq!(
        services.captations().copies_de_l_import(&import),
        Ok(Vec::new())
    );

    let rechoisie = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("a.wav"), a),
            (Some("b.wav"), change.clone()),
            (Some("c.wav"), c),
        ],
        &[],
    );
    let inspection = services
        .inspecter(&rechoisie, PLACE_LIBRE)
        .expect("le dossier rechoisi s'inspecte");
    assert_eq!(inspection.reprise.as_deref(), Some(import.as_str()));
    let reprise = services
        .ouvrir_l_import(&rechoisie, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import se reprend");
    assert_eq!(reprise.rangs, vec![2, 3]);
    assert_eq!(
        verser_le_rang(&mut services, &rechoisie, &import, 2, &mut contexte).issue,
        Issue::Verse
    );
    assert_eq!(
        services.captations().taille(&e_change),
        Ok(Some(change.len() as u64))
    );
}

/// Question 35 : un import ouvert sur `A`, un fichier versé ; l'inspection de `B` dit l'import
/// ouvert ailleurs, sans reprise ; l'ouverture de `B` clôt d'abord l'import de `A` : le journal
/// porte « Import du 14 novembre clos », puis « Import du 14 novembre ouvert » ; le rapport de `A`
/// se relit, un vu, un versé, aucun dédoublonné ni ignoré ; `B` clos, l'état du fonds ne dit plus
/// d'import resté ouvert.
pub fn un_import_ouvert_ailleurs_se_clot_a_l_ouverture_du_suivant<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 71);
    let source_a = paire.source(Some("A"), &[(Some("a.wav"), a.clone())], &[]);
    services
        .inspecter(&source_a, PLACE_LIBRE)
        .expect("A s'inspecte");
    let premier = services
        .ouvrir_l_import(&source_a, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import de A s'ouvre")
        .import;
    verser_le_rang(&mut services, &source_a, &premier, 1, &mut contexte);

    let source_b = paire.source(Some("B"), &[(Some("b.wav"), wav_de(8_000, 72))], &[]);
    let inspection = services
        .inspecter(&source_b, PLACE_LIBRE)
        .expect("B s'inspecte");
    assert_eq!(inspection.reprise, None);
    assert!(
        inspection.ouvert_ailleurs,
        "l'import de A n'est pas dit ouvert ailleurs"
    );
    let avant = messages_du_journal(&services).len();
    let second = services
        .ouvrir_l_import(&source_b, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import de B s'ouvre");
    assert!(!second.repris, "l'import de B se dit repris");
    assert_eq!(
        messages_du_journal(&services)[avant..].to_vec(),
        vec![
            "Import du 14 novembre clos".to_string(),
            "Import du 14 novembre ouvert".to_string(),
        ]
    );
    let un = compte_des(&[a.as_slice()]);
    assert_eq!(
        services.lire_un_rapport(&commandes::LireUnRapport {
            import: premier.clone(),
        }),
        Ok(rapport_de(&premier, un.clone(), un, compte(0, 0)))
    );

    verser_le_rang(&mut services, &source_b, &second.import, 1, &mut contexte);
    services
        .clore_l_import(&source_b, &cloture(&second.import), &mut contexte)
        .expect("l'import de B se clôt");
    assert_eq!(avertit_d_un_import_reste_ouvert(&services), 0);
    assert!(
        lecture::imports(fonds_de(&services))
            .iter()
            .all(|lu| lu.rapport.is_some()),
        "un import reste ouvert"
    );
}

/// Question 35 ; la décision de la session, révisée : l'import clos ailleurs se compte comme sa
/// propre clôture l'aurait fait, par la règle du noyau (`rapport::comptes`), avec les connues
/// lues à la tête de son ouverture. Un import d'avant, clos, verse un contenu ; puis un import
/// ouvert sur `A`, chacun de ses cinq fichiers versé : ce contenu déjà au fonds, un WAV coupé
/// (illisible), deux noms d'un même contenu, et un fichier sans nom ; l'ouverture de `B` le clôt ;
/// son rapport compte les quatre noms posés, le contenu déjà au fonds et le second nom du même
/// contenu dédoublonnés, le WAV coupé illisible ; ni ignoré, ni sans nom : le fonds n'en sait rien.
pub fn un_import_clos_ailleurs_se_compte_par_la_regle_du_noyau<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let connu = wav_de(8_000, 90);
    let avant = paire.source(Some("Avant"), &[(Some("connu.wav"), connu.clone())], &[]);
    importer(&mut services, &avant, &[], &mut contexte);
    let coupe = wav_coupe();
    let double = wav_de(8_000, 91);
    let source_a = paire.source(
        Some("A"),
        &[
            (Some("v.wav"), connu.clone()),
            (Some("w.wav"), coupe.clone()),
            (Some("x.wav"), double.clone()),
            (Some("y.wav"), double.clone()),
            (None, wav_de(8_000, 92)),
        ],
        &[],
    );
    services
        .inspecter(&source_a, PLACE_LIBRE)
        .expect("A s'inspecte");
    let premier = services
        .ouvrir_l_import(&source_a, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import de A s'ouvre")
        .import;
    for rang in 1..=5 {
        verser_le_rang(&mut services, &source_a, &premier, rang, &mut contexte);
    }

    let source_b = paire.source(Some("B"), &[(Some("z.wav"), wav_de(8_000, 93))], &[]);
    services
        .inspecter(&source_b, PLACE_LIBRE)
        .expect("B s'inspecte");
    services
        .ouvrir_l_import(&source_b, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import de B s'ouvre");

    let nommes = [
        connu.as_slice(),
        coupe.as_slice(),
        double.as_slice(),
        double.as_slice(),
    ];
    assert_eq!(
        services.lire_un_rapport(&commandes::LireUnRapport {
            import: premier.clone(),
        }),
        Ok(RapportDImport {
            import: premier.clone(),
            vus: compte_des(&nommes),
            verses: compte_des(&[coupe.as_slice(), double.as_slice()]),
            dedoublonnes: compte_des(&[connu.as_slice(), double.as_slice()]),
            illisibles: compte_des(&[coupe.as_slice()]),
            ignores: compte(0, 0),
            sans_nom: compte(0, 0),
            origines_changees: 0,
            fichiers_sans_nom: Vec::new(),
        })
    );
}

/// Un fonds en lecture seule (la migration qui attend ses dépendances) : l'inspection est refusée
/// du refus de `lecture_seule()`, sans lire un octet du dossier, comme l'ouverture ; aucune
/// captation n'est versée.
pub fn en_lecture_seule_l_import_est_refuse_avant_toute_captation<P: Paire>(paire: &mut P) {
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
        .expect("la note fait naître le fonds");
    let migration = migration_sur_une_autre_genese();
    let mut services = paire.rouvrir(services, &migrations_de_test(&migration));
    let refus = services
        .documents()
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[(Some("a.wav"), wav_de(8_000, 81))],
        &[],
    );

    assert_eq!(services.inspecter(&source, PLACE_LIBRE), Err(refus.clone()));
    assert_eq!(source.octets_lus(1), 0);
    assert_eq!(
        services.ouvrir_l_import(
            &source,
            &ouverture(&[]),
            PLACE_LIBRE,
            &mut contexte_de_test(&mut alea)
        ),
        Err(refus)
    );
    assert_eq!(services.captations().tailles(), Ok(BTreeSet::new()));
}

/// Question 2 : un import ouvert, un fichier versé ; tant que ces services le conduisent, l'état
/// du fonds ne dit aucun import resté ouvert ; rouverts, ils le disent, une fois (« Un import est
/// resté ouvert : rechoisir son dossier le reprend. ») ; repris, plus ; clos, ni avant ni après
/// une nouvelle réouverture.
pub fn l_etat_du_fonds_dit_qu_un_import_est_reste_ouvert<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(DOSSIER_VERSE),
        &[
            (Some("a.wav"), wav_de(8_000, 31)),
            (Some("b.wav"), wav_de(8_000, 32)),
        ],
        &[],
    );
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import s'ouvre")
        .import;
    verser_le_rang(&mut services, &source, &import, 1, &mut contexte);
    assert_eq!(avertit_d_un_import_reste_ouvert(&services), 0);

    let mut services = paire.rouvrir(services, &[]);
    assert_eq!(avertit_d_un_import_reste_ouvert(&services), 1);
    services
        .inspecter(&source, PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    let reprise = services
        .ouvrir_l_import(&source, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(avertit_d_un_import_reste_ouvert(&services), 0);
    verser_le_rang(&mut services, &source, &import, 2, &mut contexte);
    services
        .clore_l_import(&source, &cloture(&import), &mut contexte)
        .expect("l'import repris se clôt");
    assert_eq!(avertit_d_un_import_reste_ouvert(&services), 0);

    let services = paire.rouvrir(services, &[]);
    assert_eq!(avertit_d_un_import_reste_ouvert(&services), 0);
}

/// Question 35 : un import ouvert sur `A`, deux noms posés (`a.wav`, `b.wav`) ; un dossier `B`
/// sans ces noms, puis un dossier qui porte `a.wav` sans `b.wav` : l'inspection de chacun ne dit
/// aucune reprise, et dit l'import ouvert ailleurs ; l'import de `A` reste ouvert, sans rapport,
/// tant qu'aucun autre ne s'ouvre, et rechoisir `A` le reprend encore.
pub fn rechoisir_un_autre_dossier_ne_reprend_pas_l_import_ouvert<P: Paire>(paire: &mut P) {
    let mut services = paire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    let a = wav_de(8_000, 41);
    let source_a = paire.source(
        Some("A"),
        &[
            (Some("a.wav"), a.clone()),
            (Some("b.wav"), wav_de(8_000, 42)),
            (Some("c.wav"), wav_de(8_000, 43)),
        ],
        &[],
    );
    services
        .inspecter(&source_a, PLACE_LIBRE)
        .expect("A s'inspecte");
    let import = services
        .ouvrir_l_import(&source_a, &ouverture(&[]), PLACE_LIBRE, &mut contexte)
        .expect("l'import de A s'ouvre")
        .import;
    for rang in [1, 2] {
        verser_le_rang(&mut services, &source_a, &import, rang, &mut contexte);
    }

    let source_b = paire.source(
        Some("B"),
        &[
            (Some("x.wav"), wav_de(8_000, 44)),
            (Some("y.wav"), wav_de(8_000, 45)),
        ],
        &[],
    );
    let partielle = paire.source(
        Some("A"),
        &[(Some("a.wav"), a), (Some("x.wav"), wav_de(8_000, 44))],
        &[],
    );
    for autre in [&source_b, &partielle] {
        let inspection = services
            .inspecter(autre, PLACE_LIBRE)
            .expect("l'autre dossier s'inspecte");
        assert_eq!(inspection.reprise, None);
        assert!(
            inspection.ouvert_ailleurs,
            "l'import de A n'est pas dit ouvert ailleurs"
        );
        assert_eq!(import_lu(&services, &import).rapport, None);
    }

    let inspection = services
        .inspecter(&source_a, PLACE_LIBRE)
        .expect("A s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(import.as_str()));
    assert!(
        !inspection.ouvert_ailleurs,
        "l'import repris est dit ouvert ailleurs"
    );
}
