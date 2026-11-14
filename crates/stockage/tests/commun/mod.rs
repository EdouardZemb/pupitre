//! Les aides des tests du stockage, écrites une seule fois : la naissance, l'heure et le message
//! des tests, le dossier jetable de chaque test, ce que contient un dossier et l'empreinte de
//! chacun de ses fichiers, des octets engendrés au lieu d'un fichier, un texte long, la copie d'un
//! arbre de fichiers, l'ouverture de la racine des données et du fonds, entier, selon des
//! migrations ou tel qu'il s'ouvre, la naissance du fonds et ses deux copies, un fonds bâti sans
//! racine des données, la migration de test et les migrations qui la portent, l'écriture des
//! tests, avec ou sans son message et son heure, l'écriture faite sans avertissement, les chemins
//! du fonds et des captations, les échanges de deux copies et leur réunion, la relecture du
//! dossier du fonds, l'objet à clés que porte une clé, l'idée que porte le fonds, les noms
//! nouveaux d'un dossier, le fichier d'ajout qu'une écriture ou une réception ajoute, l'octet du
//! milieu d'un fichier changé, ce que refuse un document en lecture seule, l'écriture et la
//! compaction d'abord, et les gestes de test sur le fonds.

// Chaque fichier de test n'emploie qu'une partie des aides.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{
    ActorId, Automerge, AutomergeError, Change, ChangeHash, ObjId, ObjType, ROOT, ReadDoc, Value,
};
use noyau::donnees::fonds::{
    ACTEUR_DE_LA_GENESE, ETIQUETTES, FORMAT, GENESE, IDEE, IDEES, MESSAGE_DE_LA_NAISSANCE, PRISES,
    TEXTE, VERSION_DE_SCHEMA,
};
use noyau::donnees::version::{Migration, message_de_la_migration};
use stockage::{
    Avertissement, Document, Erreur, LectureSeule, Naissance, RacineDesDonnees, empreinte,
};
use synchro::{Copie, Echange};

/// La naissance des tests : des identifiants écrits, jamais tirés.
pub const NAISSANCE: Naissance<'static> = Naissance {
    identifiant: "fonds-1",
    lignee: "lignee-1",
    proprietaire: "proprietaire-1",
};

/// L'heure de la naissance et de chaque écriture par `ecrire` ; les autres écritures des tests
/// viennent après (`HEURE + n`). Une entrée, jamais l'horloge.
pub const HEURE: i64 = 1_700_000_000;

/// Le message du premier geste, noter une idée, et de chaque écriture par `ecrire`.
pub const MESSAGE_DU_GESTE: &str = "Idée notée";

/// Le dossier jetable du test `nom`, dans le dossier temporaire du système, jamais dans le
/// dossier de l'application (I30) : `pupitre-stockage-<nom>-<processus>`. Ce qu'un passage
/// précédent y a laissé est effacé, puis le dossier est recréé, vide. C'est le seul endroit
/// des tests qui devine un chemin.
pub fn dossier_jetable(nom: &str) -> PathBuf {
    let dossier =
        std::env::temp_dir().join(format!("pupitre-stockage-{nom}-{}", std::process::id()));
    match fs::remove_dir_all(&dossier) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => panic!("effacer {} : {e}", dossier.display()),
    }
    fs::create_dir_all(&dossier).unwrap_or_else(|e| panic!("créer {} : {e}", dossier.display()));
    dossier
}

/// Les noms de ce que contient `dossier`, dans l'ordre.
pub fn entrees(dossier: &Path) -> Vec<String> {
    let mut noms: Vec<String> = fs::read_dir(dossier)
        .unwrap_or_else(|e| panic!("lire {} : {e}", dossier.display()))
        .map(|entree| {
            entree
                .unwrap_or_else(|e| panic!("lire {} : {e}", dossier.display()))
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    noms.sort();
    noms
}

/// L'empreinte de chaque fichier sous `dossier`, à toute profondeur, par chemin relatif.
pub fn empreintes(dossier: &Path) -> BTreeMap<PathBuf, String> {
    let mut vues = BTreeMap::new();
    let mut a_lire = vec![dossier.to_path_buf()];
    while let Some(courant) = a_lire.pop() {
        let entrees =
            fs::read_dir(&courant).unwrap_or_else(|e| panic!("lire {} : {e}", courant.display()));
        for entree in entrees {
            let chemin = entree
                .unwrap_or_else(|e| panic!("lire {} : {e}", courant.display()))
                .path();
            if chemin.is_dir() {
                a_lire.push(chemin);
            } else {
                let octets =
                    fs::read(&chemin).unwrap_or_else(|e| panic!("lire {} : {e}", chemin.display()));
                let relatif = chemin
                    .strip_prefix(dossier)
                    .expect("un fichier sous le dossier")
                    .to_path_buf();
                vues.insert(relatif, empreinte(&octets));
            }
        }
    }
    vues
}

/// Un générateur congruentiel ou xorshift semé par `graine`, dont on garde les bits hauts : une
/// suite déterministe, sans période courte, sans fichier.
///
/// Ici, le générateur congruentiel de Knuth (MMIX), modulo 2⁶⁴, de période pleine. Chaque octet
/// rendu est l'octet de tête de son état : les bits bas d'un tel générateur ont des périodes
/// courtes, ses bits hauts les plus longues.
pub fn octets(graine: u8, longueur: usize) -> Vec<u8> {
    const MULTIPLICATEUR: u64 = 6_364_136_223_846_793_005;
    const INCREMENT: u64 = 1_442_695_040_888_963_407;
    let mut etat = u64::from(graine) ^ 0x9E37_79B9_7F4A_7C15;
    (0..longueur)
        .map(|_| {
            etat = etat.wrapping_mul(MULTIPLICATEUR).wrapping_add(INCREMENT);
            (etat >> 56) as u8
        })
        .collect()
}

/// 2 000 lettres de `a` à `z`, la n-ième `b'a' + octets(9, 2000)[n] % 26` : un texte que DEFLATE
/// ne ramène pas au poids d'un fichier d'ajout (`save` comprime le fichier complet, `save_after`
/// ne comprime pas les fichiers d'ajout).
pub fn texte_long() -> String {
    octets(9, 2000)
        .into_iter()
        .map(|octet| char::from(b'a' + octet % 26))
        .collect()
}

/// Copie l'arbre de `de` dans `vers`, dossiers et fichiers, comme une sauvegarde ou une copie
/// de la racine des données le ferait : le fichier `verrou` compris.
pub fn copier_l_arbre(de: &Path, vers: &Path) {
    fs::create_dir_all(vers).unwrap_or_else(|e| panic!("créer {} : {e}", vers.display()));
    let entrees = fs::read_dir(de).unwrap_or_else(|e| panic!("lire {} : {e}", de.display()));
    for entree in entrees {
        let entree = entree.unwrap_or_else(|e| panic!("lire {} : {e}", de.display()));
        let source = entree.path();
        let cible = vers.join(entree.file_name());
        let sorte = entree
            .file_type()
            .unwrap_or_else(|e| panic!("lire {} : {e}", source.display()));
        if sorte.is_dir() {
            copier_l_arbre(&source, &cible);
        } else {
            fs::copy(&source, &cible).unwrap_or_else(|e| {
                panic!("copier {} vers {} : {e}", source.display(), cible.display())
            });
        }
    }
}

/// La racine des données de `chemin`, ouverte.
pub fn ouvrir(chemin: &Path) -> RacineDesDonnees {
    RacineDesDonnees::ouvrir(chemin).unwrap_or_else(|erreur| {
        panic!(
            "ouvrir la racine des données {} : {erreur:?}",
            chemin.display()
        )
    })
}

/// Le fonds de `racine`, rouvert entier : ni en lecture seule, ni avec un avertissement. Un test
/// qui attend l'un ou l'autre l'ouvre par `ouvrir_le_fonds_tel_quel`.
pub fn ouvrir_le_fonds(racine: &RacineDesDonnees) -> Document<'_> {
    entier(ouvrir_le_fonds_tel_quel(racine))
}

/// Le fonds de `racine`, rouvert selon les migrations `migrations` (`fonds_selon`), au lieu de
/// `MIGRATIONS`, entier comme par `ouvrir_le_fonds`. Un test qui attend la lecture seule ou un
/// avertissement ouvre par `fonds_selon` lui-même.
pub fn ouvrir_le_fonds_selon<'r>(
    racine: &'r RacineDesDonnees,
    migrations: &[Migration<'_>],
) -> Document<'r> {
    let fonds = racine
        .fonds_selon(migrations)
        .expect("rouvrir le fonds selon les migrations")
        .expect("le fonds est né");
    entier(fonds)
}

/// `fonds`, rouvert, s'il est entier : ni en lecture seule, ni avec un avertissement ; sinon,
/// panique.
fn entier(fonds: Document<'_>) -> Document<'_> {
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds rouvert est en lecture seule"
    );
    assert!(
        fonds.avertissements().is_empty(),
        "le fonds rouvert a des avertissements : {:?}",
        fonds.avertissements()
    );
    fonds
}

/// Le fonds de `racine`, rouvert tel qu'il s'ouvre : en lecture seule, ou avec des
/// avertissements, s'il en a.
pub fn ouvrir_le_fonds_tel_quel(racine: &RacineDesDonnees) -> Document<'_> {
    racine
        .fonds()
        .expect("rouvrir le fonds")
        .expect("le fonds est né")
}

/// Le fonds né d'une idée au texte long : `idee-1`, `texte_long()`, `MESSAGE_DU_GESTE`, `HEURE`.
/// Il naît sans avertissement.
pub fn faire_naitre(racine: &RacineDesDonnees) -> Document<'_> {
    let fonds = racine
        .faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            noter(tx, "idee-1", &texte_long())
        })
        .expect("faire naître le fonds");
    assert!(
        fonds.avertissements().is_empty(),
        "le fonds est né avec des avertissements : {:?}",
        fonds.avertissements()
    );
    fonds
}

/// Le fonds né (`faire_naitre`) dans `<dossier jetable>/a`, fermé, puis copié dans `<…>/b` : A, B.
/// Rend les chemins des deux racines des données, A puis B.
pub fn deux_copies(nom: &str) -> (PathBuf, PathBuf) {
    let d = dossier_jetable(nom);
    let a = d.join("a");
    let b = d.join("b");
    {
        let racine = RacineDesDonnees::ouvrir(&a).expect("ouvrir la racine des données d'A");
        faire_naitre(&racine);
    }
    copier_l_arbre(&a, &b);
    (a, b)
}

/// Un fonds que le test bâtit en mémoire, sans racine des données : la genèse, puis une naissance
/// qui écrit chaque couple (clé, chaîne) de `champs` à la racine du document, sous
/// `MESSAGE_DE_LA_NAISSANCE`, à l'heure `HEURE`, par l'acteur `acteur`.
pub fn batir_un_fonds(acteur: ActorId, champs: &[(&str, &str)]) -> Automerge {
    let mut bati = Automerge::new();
    bati.set_actor(acteur);
    bati.apply_changes([Change::try_from(GENESE).expect("lire la genèse")])
        .expect("appliquer la genèse");
    let mut tx = bati.transaction();
    for &(cle, valeur) in champs {
        tx.put(ROOT, cle, valeur)
            .expect("écrire un champ de la naissance");
    }
    tx.commit_with(
        CommitOptions::default()
            .with_message(MESSAGE_DE_LA_NAISSANCE)
            .with_time(HEURE),
    );
    bati
}

/// Le conteneur de la racine du document que crée la migration de test : un objet à clés, où la
/// donnée nouvelle d'un objet s'écrit en valeurs simples, sous `<objet>/<élément>` (§ 31 · 2.4).
pub const CONTENEUR_DE_LA_MIGRATION: &str = "elements_de_test";

/// La migration de test vers la version de schéma 2, engendrée à l'exécution depuis `GENESE`, de
/// façon déterministe, comme la recette de la genèse (`genese_par_la_recette`, dans
/// `tests/genese.rs`) : l'acteur `ACTEUR_DE_LA_GENESE`, qui pose `VERSION_DE_SCHEMA` à 2 et crée
/// `CONTENEUR_DE_LA_MIGRATION`, un objet à clés, à la racine du document, sous le message
/// `message_de_la_migration(2)`, à l'heure 0. Rend les octets de sa modification
/// (`raw_bytes()`), qui ne dépend que de la genèse.
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

/// Le chemin d'écriture des tests du stockage par une fermeture sur la transaction
/// (`Document::ecrire`, sous `pannes`), avec `ecrire_a` qu'il appelle : une fermeture ne se
/// traduit pas en changements, et les commandes écrivent par `Document::appliquer`. Les tests
/// d'H07 écrivent aussi par `Document::ecrire`. Écrit `geste` dans `document`, sous le message `MESSAGE_DU_GESTE`, à
/// l'heure `HEURE`, et rend ce que rend l'écriture : faite (`None`), faite avec un
/// avertissement, ou refusée.
// L'erreur est celle de `Document::ecrire`, que l'aide rend telle quelle. Clippy épargne la
// méthode, exportée par le stockage, mais pas l'aide, qui ne l'est pas.
#[allow(clippy::result_large_err)]
pub fn ecrire(
    document: &mut Document<'_>,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Option<Avertissement>, Erreur> {
    ecrire_a(document, MESSAGE_DU_GESTE, HEURE, geste)
}

/// L'écriture d'`ecrire`, sous le message `message`, à l'heure `heure` : le chemin d'écriture
/// des tests nouveaux qui donnent à chaque écriture son message et son heure.
// L'erreur est rendue telle quelle, comme par `ecrire`.
#[allow(clippy::result_large_err)]
pub fn ecrire_a(
    document: &mut Document<'_>,
    message: &str,
    heure: i64,
    geste: impl FnOnce(&mut Transaction<'_>) -> Result<(), AutomergeError>,
) -> Result<Option<Avertissement>, Erreur> {
    document.ecrire(message, heure, geste)
}

/// L'écriture `quoi`, que `rendu` dit, est faite sans avertissement : `Ok(None)`. Les écritures
/// des tests d'H07 l'exigent : aucune panne n'y est armée, et un avertissement y est un verdict à
/// instruire, comme un refus.
#[track_caller]
pub fn faite(rendu: Result<Option<Avertissement>, Erreur>, quoi: &str) {
    assert!(
        matches!(rendu, Ok(None)),
        "{quoi} : attendu l'écriture faite, sans avertissement ; rendu {rendu:?}"
    );
}

/// `documents/fonds/`, dans la racine des données `racine_des_donnees`.
pub fn dossier_du_fonds(racine_des_donnees: &Path) -> PathBuf {
    racine_des_donnees.join("documents").join("fonds")
}

/// `documents/fonds/ajouts/`, dans la racine des données `racine_des_donnees`.
pub fn dossier_des_ajouts(racine_des_donnees: &Path) -> PathBuf {
    dossier_du_fonds(racine_des_donnees).join("ajouts")
}

/// Le fichier complet du fonds de la racine des données `racine_des_donnees`, relu du disque.
pub fn lire_le_complet(racine_des_donnees: &Path) -> Vec<u8> {
    fs::read(dossier_du_fonds(racine_des_donnees).join("complet.automerge"))
        .expect("lire le fichier complet")
}

/// Le poids de `chemin` sur le disque, en octets.
pub fn poids(chemin: &Path) -> u64 {
    fs::metadata(chemin)
        .unwrap_or_else(|e| panic!("lire {} : {e}", chemin.display()))
        .len()
}

/// Le chemin où se range la captation `e` dans la racine des données `racine_des_donnees` :
/// `captations/<deux premiers caractères>/<e>`.
pub fn rangee(racine_des_donnees: &Path, e: &str) -> PathBuf {
    racine_des_donnees.join("captations").join(&e[..2]).join(e)
}

/// Les échanges de deux copies, vus de chacune : celui d'`a` avec `b`, puis celui de `b` avec `a`.
pub fn ouvrir_les_echanges(a: &Document<'_>, b: &Document<'_>) -> (Echange, Echange) {
    let copie_a = a.copie().expect("la copie d'A");
    let copie_b = b.copie().expect("la copie de B");
    let ea = Echange::ouvrir(&copie_a, &copie_b).expect("ouvrir l'échange d'A");
    let eb = Echange::ouvrir(&copie_b, &copie_a).expect("ouvrir l'échange de B");
    (ea, eb)
}

/// Réunit deux copies sans réseau : chacune envoie tant qu'elle a à dire, en octets, comme le
/// fera le transport du lot 3. Chaque réception, d'`a` comme de `b`, est faite sans
/// avertissement : aucune panne n'y est armée. Rend le nombre de messages ; au-delà de mille, la
/// réunion ne s'achève pas, et c'est un échec.
pub fn reunir(a: &mut Document<'_>, b: &mut Document<'_>) -> usize {
    let (mut ea, mut eb) = ouvrir_les_echanges(a, b);
    reunir_en_suivant_a(a, &mut ea, b, &mut eb, |_, _, rendu| {
        assert!(
            matches!(rendu, Ok(None)),
            "A reçoit le message de B, sans avertissement : rendu {rendu:?}"
        );
    })
}

/// La réunion de `reunir`, sur les échanges `ea` d'`a` et `eb` de `b`, qui suit chaque réception
/// d'`a` : `recue` la reçoit, avec `a` après la réception, ses heads d'avant et ce que la
/// réception a rendu, et la réunion continue quoi qu'elle ait rendu. Chaque réception de `b` doit
/// être faite sans avertissement : un test qui arme une panne la fait tomber sur une réception
/// d'`a`, `b` n'ayant alors rien de neuf à recevoir. Rend le nombre de messages ; au-delà de
/// mille, la réunion ne s'achève pas, et c'est un échec.
pub fn reunir_en_suivant_a(
    a: &mut Document<'_>,
    ea: &mut Echange,
    b: &mut Document<'_>,
    eb: &mut Echange,
    mut recue: impl FnMut(&Document<'_>, &[ChangeHash], Result<Option<Avertissement>, Erreur>),
) -> usize {
    let mut n = 0;
    loop {
        let de_a = a.prochain_message(ea);
        if let Some(m) = &de_a {
            let rendu = b.recevoir(eb, m);
            assert!(
                matches!(rendu, Ok(None)),
                "B reçoit le message d'A, sans avertissement : rendu {rendu:?}"
            );
            n += 1;
        }
        let de_b = b.prochain_message(eb);
        if let Some(m) = &de_b {
            let heads_d_avant = a.automerge().get_heads();
            let rendu = a.recevoir(ea, m);
            n += 1;
            recue(a, &heads_d_avant, rendu);
        }
        if de_a.is_none() && de_b.is_none() {
            return n;
        }
        assert!(n <= 1000, "la réunion ne s'achève pas");
    }
}

/// Relit `documents/fonds/` à part, sans `RacineDesDonnees` : `Automerge::load` de
/// `complet.automerge`, puis `load_incremental` de chaque `ajouts/*.automerge`. Les noms en
/// `.partiel`, de `documents/fonds/` comme de `ajouts/`, sont ignorés ; `fonds()`, lui, lit ceux
/// que son démarrage lit ou met en place.
pub fn relire_le_dossier_du_fonds(racine_des_donnees: &Path) -> Automerge {
    let dossier = dossier_du_fonds(racine_des_donnees);
    let complet = dossier.join("complet.automerge");
    let octets = fs::read(&complet).unwrap_or_else(|e| panic!("lire {} : {e}", complet.display()));
    let mut relu =
        Automerge::load(&octets).unwrap_or_else(|e| panic!("relire {} : {e}", complet.display()));
    let ajouts = dossier.join("ajouts");
    for nom in entrees(&ajouts) {
        // `<empreinte>.automerge.<processus>.partiel` n'est pas un fichier d'ajout.
        if !nom.ends_with(".automerge") {
            continue;
        }
        let chemin = ajouts.join(&nom);
        let octets =
            fs::read(&chemin).unwrap_or_else(|e| panic!("lire {} : {e}", chemin.display()));
        if let Err(e) = relu.load_incremental(&octets) {
            panic!("relire {} : {e}", chemin.display());
        }
    }
    relu
}

/// L'objet à clés (`ObjType::Map`) que porte `cle` sous `parent` ; sinon, panique.
pub fn objet(doc: &Automerge, parent: &ObjId, cle: &str) -> ObjId {
    match doc.get(parent, cle).expect("lire le document") {
        Some((Value::Object(ObjType::Map), objet)) => objet,
        autre => panic!("{cle} ne porte pas un objet à clés : {autre:?}"),
    }
}

/// Les idées de `doc` portent l'idée `idee`.
pub fn porte_l_idee(doc: &Automerge, idee: &str) -> bool {
    doc.get(objet(doc, &ROOT, IDEES), idee)
        .expect("lire les idées")
        .is_some()
}

/// Les noms de `apres` qui ne sont pas dans `avant`, dans l'ordre : ce qu'un dossier a gagné.
pub fn nouveaux(avant: &[String], apres: &[String]) -> Vec<String> {
    apres
        .iter()
        .filter(|nom| !avant.contains(nom))
        .cloned()
        .collect()
}

/// Le fichier d'ajout qu'une écriture ou une réception de `doc` a écrit dans `ajouts`, qu'elle a
/// fait passer des noms `avant` aux noms `apres` : un seul fichier de plus, nommé par l'empreinte
/// de son contenu, qui vaut `save_after` des heads `heads_d_avant` de `doc`, celles d'avant
/// l'écriture ou la réception. Rend son contenu.
pub fn constater_un_ajout(
    doc: &Automerge,
    heads_d_avant: &[ChangeHash],
    ajouts: &Path,
    avant: &[String],
    apres: &[String],
) -> Vec<u8> {
    assert_eq!(
        apres.len(),
        avant.len() + 1,
        "pas un seul fichier d'ajout de plus : {avant:?}, puis {apres:?}"
    );
    let noms = nouveaux(avant, apres);
    assert_eq!(noms.len(), 1, "un seul fichier nouveau : {noms:?}");
    let contenu = fs::read(ajouts.join(&noms[0])).expect("lire le fichier d'ajout");
    assert_eq!(
        noms[0],
        format!("{}.automerge", empreinte(&contenu)),
        "le nom du fichier d'ajout n'est pas l'empreinte de son contenu"
    );
    assert!(
        contenu == doc.save_after(heads_d_avant),
        "le fichier d'ajout n'est pas save_after des heads d'avant"
    );
    contenu
}

/// Change l'octet du milieu du fichier `chemin` (`^ 0xFF`), puis rend les octets altérés.
pub fn alterer_l_octet_du_milieu(chemin: &Path) -> Vec<u8> {
    let mut octets = fs::read(chemin).unwrap_or_else(|e| panic!("lire {} : {e}", chemin.display()));
    let milieu = octets.len() / 2;
    octets[milieu] ^= 0xFF;
    fs::write(chemin, &octets).unwrap_or_else(|e| panic!("altérer {} : {e}", chemin.display()));
    octets
}

/// Ce que refuse `document`, en lecture seule pour la raison `raison` (§ 31 · 2.1), qu'il écrive
/// une opération ou se compacte : `ecrire` et `compacter` rendent `Erreur::LectureSeule(raison)`.
pub fn constater_le_refus_d_ecrire(document: &mut Document<'_>, raison: &LectureSeule) {
    match ecrire(document, |tx| noter(tx, "idee-9", "Une phrase refusée")) {
        Err(Erreur::LectureSeule(rendue)) => assert_eq!(
            &rendue, raison,
            "l'écriture ne rend pas la raison de la lecture seule"
        ),
        autre => panic!("attendu LectureSeule à l'écriture, rendu {autre:?}"),
    }
    match document.compacter() {
        Err(Erreur::LectureSeule(rendue)) => assert_eq!(
            &rendue, raison,
            "la compaction ne rend pas la raison de la lecture seule"
        ),
        autre => panic!("attendu LectureSeule à la compaction, rendu {autre:?}"),
    }
}

/// Ce que refuse `document`, en lecture seule pour la raison `raison` (§ 31 · 2.1) : `ecrire` et
/// `compacter` rendent `Erreur::LectureSeule(raison)` (`constater_le_refus_d_ecrire`),
/// `prochain_message` rend `None`, et `recevoir` rend `Erreur::LectureSeule(_)`. Ces quatre
/// appels faits, chaque fichier sous `dossier`, le dossier du fonds, a encore son empreinte
/// d'`avant`, prise avant l'ouverture, ou dès que le document est en lecture seule.
/// L'échange s'ouvre sur la copie de `NAISSANCE`, qu'un document illisible peut ne plus porter.
pub fn constater_la_lecture_seule(
    document: &mut Document<'_>,
    raison: &LectureSeule,
    dossier: &Path,
    avant: &BTreeMap<PathBuf, String>,
) {
    constater_le_refus_d_ecrire(document, raison);
    let copie = Copie {
        proprietaire: NAISSANCE.proprietaire.to_owned(),
        document: NAISSANCE.identifiant.to_owned(),
        lignee: NAISSANCE.lignee.to_owned(),
    };
    let mut echange = Echange::ouvrir(&copie, &copie).expect("ouvrir l'échange");
    assert_eq!(
        document.prochain_message(&mut echange),
        None,
        "un document en lecture seule a envoyé un message"
    );
    let mut de_l_autre =
        Echange::ouvrir(&copie, &copie).expect("ouvrir l'échange de l'autre copie");
    let message = de_l_autre
        .prochain_message(document.automerge())
        .expect("le message de l'autre copie");
    match document.recevoir(&mut echange, &message) {
        Err(Erreur::LectureSeule(_)) => {}
        autre => panic!("attendu LectureSeule à la réception, rendu {autre:?}"),
    }
    assert_eq!(
        empreintes(dossier),
        *avant,
        "un fichier du fonds a changé depuis l'ouverture en lecture seule"
    );
}

// Les gestes de test, qu'appellent les fermetures `geste` : les opérations nommées arrivent au
// lot 1. Une idée ou un conteneur absent rend `AutomergeError::NotAnObject`.

/// Crée l'idée `idee` sous les idées, avec son texte et son objet des étiquettes, vide, dans la
/// même modification : les conteneurs d'un objet naissent avec lui (§ 31 · 2.4).
pub fn noter(tx: &mut Transaction<'_>, idee: &str, texte: &str) -> Result<(), AutomergeError> {
    let idees = conteneur(tx, &ROOT, IDEES)?;
    let nee = tx.put_object(&idees, idee, ObjType::Map)?;
    tx.put(&nee, TEXTE, texte)?;
    tx.put_object(&nee, ETIQUETTES, ObjType::Map)?;
    Ok(())
}

/// Remplace le texte de l'idée `idee`.
pub fn changer_le_texte(
    tx: &mut Transaction<'_>,
    idee: &str,
    texte: &str,
) -> Result<(), AutomergeError> {
    let idee = l_idee(tx, idee)?;
    tx.put(&idee, TEXTE, texte)
}

/// Pose l'étiquette `etiquette` sur l'idée `idee`.
pub fn etiqueter(
    tx: &mut Transaction<'_>,
    idee: &str,
    etiquette: &str,
) -> Result<(), AutomergeError> {
    let etiquettes = conteneur(tx, &l_idee(tx, idee)?, ETIQUETTES)?;
    tx.put(&etiquettes, etiquette, true)
}

/// Ôte l'étiquette `etiquette` de l'idée `idee` ; n'écrit rien si l'étiquette manque.
pub fn oter_l_etiquette(
    tx: &mut Transaction<'_>,
    idee: &str,
    etiquette: &str,
) -> Result<(), AutomergeError> {
    let etiquettes = conteneur(tx, &l_idee(tx, idee)?, ETIQUETTES)?;
    if tx.get(&etiquettes, etiquette)?.is_none() {
        return Ok(());
    }
    tx.delete(&etiquettes, etiquette)
}

/// Pose, sous les prises, la prise d'empreinte `empreinte`, de l'idée `idee` et au format
/// `format`.
pub fn poser_une_prise(
    tx: &mut Transaction<'_>,
    empreinte: &str,
    idee: &str,
    format: &str,
) -> Result<(), AutomergeError> {
    let prises = conteneur(tx, &ROOT, PRISES)?;
    let prise = tx.put_object(&prises, empreinte, ObjType::Map)?;
    tx.put(&prise, IDEE, idee)?;
    tx.put(&prise, FORMAT, format)
}

/// L'objet de l'idée `idee`, sous les idées.
fn l_idee(tx: &Transaction<'_>, idee: &str) -> Result<ObjId, AutomergeError> {
    let idees = conteneur(tx, &ROOT, IDEES)?;
    conteneur(tx, &idees, idee)
}

/// L'objet que porte la clé `cle` de `parent` ; une clé absente, ou qui porte une valeur
/// simple, rend `AutomergeError::NotAnObject`.
fn conteneur(tx: &Transaction<'_>, parent: &ObjId, cle: &str) -> Result<ObjId, AutomergeError> {
    match tx.get(parent, cle)? {
        Some((Value::Object(_), objet)) => Ok(objet),
        _ => Err(AutomergeError::NotAnObject),
    }
}
