//! Le démarrage dans `documents/` (§ 31 · 2.1) : à chaque ouverture du fonds, la garde prise,
//! chaque `.partiel` reçoit son sort. Le voisin d'un fichier d'ajout dont le contenu donne
//! l'empreinte prend sa place, ou se lit où il est, avec un avertissement ; le voisin lisible du
//! fichier complet se lit, puis la compaction d'ouverture le fond ; tout autre se retire. Un
//! voisin lisible du fichier complet fait exister le fonds, que la naissance ne refait pas. Un
//! fichier étranger reste tel quel, et rien ne se touche sans le verrou. Le fonds ouvert, chaque
//! nom provisoire de `captations/` dont le contenu entier donne une empreinte que le fonds nomme,
//! tel qu'il se lit, prend ce nom (§ 31 · 6.1) ; un nom provisoire qui ne le peut pas reste, avec
//! un avertissement.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;
use std::path::{Path, PathBuf};

use automerge::transaction::{CommitOptions, Transactable, Transaction};
use automerge::{ActorId, Automerge, AutomergeError, ROOT, ReadDoc, Value};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_des_ajouts, dossier_du_fonds, empreintes, entrees,
    faire_naitre, faite, lire_le_complet, octets, ouvrir, ouvrir_le_fonds, porte_l_idee,
    relire_le_dossier_du_fonds, texte_long,
};
use noyau::donnees::fonds::{IDEE, MIDI_VERSES, TEXTES_VERSES};
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Document, Erreur, LectureSeule, RacineDesDonnees, empreinte};

/// Le format des prises des tests du rangement, dont les captations sont des octets engendrés.
const FORMAT_DES_PRISES: &str = "wav";

/// Les idées que notent les trois écritures des tests du voisin du fichier complet, avec leur
/// texte.
const TROIS_IDEES: [(&str, &str); 3] = [
    ("idee-2", "Un pont en mineur"),
    ("idee-3", "Une coda en suspens"),
    ("idee-4", "Un couplet plus calme"),
];

/// Note l'idée `idee`, au texte `texte`, dans `fonds` : l'écriture est faite, sans avertissement.
fn noter(fonds: &mut Document<'_>, idee: &str, texte: &str) {
    faite(
        commun::ecrire(fonds, |tx| commun::noter(tx, idee, texte)),
        &format!("noter {idee}"),
    );
}

/// Les noms en `.partiel` de `dossier`, dans l'ordre.
fn partiels(dossier: &Path) -> Vec<String> {
    entrees(dossier)
        .into_iter()
        .filter(|nom| nom.ends_with(".partiel"))
        .collect()
}

/// Le seul fichier d'ajout du fonds de la racine des données `chemin`, renommé par le test en
/// son voisin `<nom>.4242.partiel`, comme une écriture dont le renommage s'est perdu après la
/// réponse. Rend le chemin du voisin.
fn l_ajout_en_voisin(chemin: &Path) -> PathBuf {
    let ajouts = dossier_des_ajouts(chemin);
    let noms = entrees(&ajouts);
    assert_eq!(noms.len(), 1, "un seul fichier d'ajout : {noms:?}");
    let voisin = ajouts.join(format!("{}.4242.partiel", noms[0]));
    fs::rename(ajouts.join(&noms[0]), &voisin).expect("renommer le fichier d'ajout en son voisin");
    voisin
}

/// Le fonds de `racine` né, puis les trois idées de `TROIS_IDEES` notées, et lâché ; puis, comme
/// une compaction dont le renommage du fichier complet s'est perdu, les fichiers d'ajout
/// absorbés déjà retirés : le dossier du fonds porte, sous le nom `nom_du_voisin`, `save()` de
/// tout ce qu'il lit, le fichier complet de la naissance à sa place, et `ajouts/` vide. Rend le
/// chemin du voisin.
fn un_voisin_du_complet_laisse(racine: &RacineDesDonnees, nom_du_voisin: &str) -> PathBuf {
    let chemin = racine.chemin();
    let mut fonds = faire_naitre(racine);
    let c0 = lire_le_complet(chemin);
    for (idee, texte) in TROIS_IDEES {
        noter(&mut fonds, idee, texte);
    }
    drop(fonds);
    let s = relire_le_dossier_du_fonds(chemin).save();
    let dossier = dossier_du_fonds(chemin);
    let voisin = dossier.join(nom_du_voisin);
    fs::write(&voisin, s).expect("écrire le voisin du fichier complet");
    fs::write(dossier.join("complet.automerge"), c0)
        .expect("remettre le fichier complet de la naissance");
    let ajouts = dossier.join("ajouts");
    for nom in entrees(&ajouts) {
        fs::remove_file(ajouts.join(nom)).expect("vider ajouts/");
    }
    voisin
}

/// `save()` d'une copie (`fork()`) du fonds `naissance`, où l'acteur `acteur` note l'idée `idee`,
/// au texte `texte`, sous `MESSAGE_DU_GESTE`, à l'heure `HEURE + 1`.
fn une_idee_de_plus(naissance: &Automerge, acteur: ActorId, idee: &str, texte: &str) -> Vec<u8> {
    let mut copie = naissance.fork();
    copie.set_actor(acteur);
    let mut tx = copie.transaction();
    commun::noter(&mut tx, idee, texte).expect("noter l'idée");
    tx.commit_with(
        CommitOptions::default()
            .with_message(MESSAGE_DU_GESTE)
            .with_time(HEURE + 1),
    );
    copie.save()
}

/// `doc` porte chacune des trois idées de `TROIS_IDEES` ; `quoi` dit ce qu'il est.
#[track_caller]
fn porte_les_trois_idees(doc: &Automerge, quoi: &str) {
    for (idee, _) in TROIS_IDEES {
        assert!(porte_l_idee(doc, idee), "{quoi} ne porte pas {idee}");
    }
}

/// Le fonds de `racine` né, au premier geste, de l'idée `idee-1` au texte long et d'une prise de
/// cette idée par empreinte de `prises` : il naît sans avertissement. Le texte long fait peser le
/// fichier complet plus que le fichier d'ajout d'une écriture qui pose une prise : aucune
/// compaction ne suit une telle écriture.
fn naitre_avec_les_prises<'r>(racine: &'r RacineDesDonnees, prises: &[&str]) -> Document<'r> {
    let fonds = racine
        .faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", &texte_long())?;
            for e in prises {
                commun::poser_une_prise(tx, e, "idee-1", FORMAT_DES_PRISES)?;
            }
            Ok(())
        })
        .expect("faire naître le fonds");
    assert!(
        fonds.avertissements().is_empty(),
        "le fonds est né avec des avertissements : {:?}",
        fonds.avertissements()
    );
    fonds
}

/// Dépose dans `captations/` de la racine des données `chemin` le nom provisoire
/// `<nom>.4242.partiel`, de contenu `contenu`, comme un versement dont le renommage s'est perdu
/// après la réponse. Rend son chemin.
fn deposer_un_nom_provisoire(chemin: &Path, nom: &str, contenu: &[u8]) -> PathBuf {
    let provisoire = chemin
        .join("captations")
        .join(format!("{nom}.4242.partiel"));
    fs::write(&provisoire, contenu)
        .unwrap_or_else(|e| panic!("déposer {} : {e}", provisoire.display()));
    provisoire
}

/// La captation `e` n'est pas rangée dans `racine` : `relire` rend `CaptationAbsente`. `quoi` dit
/// quand.
#[track_caller]
fn pas_rangee(racine: &RacineDesDonnees, e: &str, quoi: &str) {
    match racine.captations().relire(e) {
        Err(Erreur::CaptationAbsente { .. }) => {}
        autre => panic!("{quoi} : attendu la captation absente, rendu {autre:?}"),
    }
}

/// Pose, sous le conteneur `conteneur` de la racine du document, la clé à plat `<e>/idee`, qui
/// nomme la captation `e`, de l'idée `idee`. Un conteneur absent rend
/// `AutomergeError::NotAnObject`.
fn nommer_sous(
    tx: &mut Transaction<'_>,
    conteneur: &str,
    e: &str,
    idee: &str,
) -> Result<(), AutomergeError> {
    let objet = match tx.get(ROOT, conteneur)? {
        Some((Value::Object(_), objet)) => objet,
        _ => return Err(AutomergeError::NotAnObject),
    };
    tx.put(&objet, format!("{e}/{IDEE}"), idee)
}

// § 31 · 10, test 8 : renommage perdu après la réponse, le voisin entier reste
#[test]
fn le_voisin_conforme_d_un_ajout_prend_sa_place() {
    let chemin = commun::dossier_jetable("demarrage-voisin-d-ajout").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    drop(fonds);
    let voisin = l_ajout_en_voisin(&chemin);

    let fonds = ouvrir_le_fonds(&racine);

    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert ne porte pas l'écriture dont le voisin est resté"
    );
    assert!(!voisin.exists(), "le voisin n'a pas pris sa place");
    assert_eq!(
        partiels(&dossier_des_ajouts(&chemin)),
        Vec::<String>::new(),
        "ajouts/ garde un .partiel"
    );
}

#[test]
fn un_voisin_d_ajout_qui_ne_se_met_pas_en_place_se_lit_et_avertit() {
    let chemin = commun::dossier_jetable("demarrage-voisin-d-ajout-restant").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    drop(fonds);
    let voisin = l_ajout_en_voisin(&chemin);
    let octets = fs::read(&voisin).expect("lire le voisin");

    // Le premier renommage de l'ouverture est celui du démarrage.
    let ouvert = {
        let _panne = couper(Etape::Renommage, 1);
        racine.fonds()
    };

    let fonds = ouvert
        .expect("un voisin qui ne prend pas sa place n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds ouvert ne porte pas l'écriture du voisin resté"
    );
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le voisin resté met le fonds en lecture seule"
    );
    match fonds.avertissements() {
        [Avertissement::VoisinRestant { chemin: reste, .. }] => {
            assert_eq!(
                reste, &voisin,
                "l'avertissement ne nomme pas le voisin resté"
            )
        }
        autres => panic!("attendu un seul VoisinRestant, rendu {autres:?}"),
    }
    assert!(
        fs::read(&voisin).expect("relire le voisin resté") == octets,
        "le voisin resté n'est plus là, octet pour octet"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "rouvert sans panne, le fonds ne porte pas l'écriture du voisin"
    );
    assert_eq!(
        partiels(&dossier_des_ajouts(&chemin)),
        Vec::<String>::new(),
        "rouvert sans panne, ajouts/ garde un .partiel"
    );
}

// Le voisin d'un fichier d'ajout mis en place au démarrage, son renommage non rendu durable : le
// fonds s'ouvre avec `NonDurable`, sur le fichier d'ajout. La compaction d'ouverture, que ce
// fichier d'ajout lu demande, le fond aussitôt dans un fichier complet neuf, rendu durable, puis
// le retire : l'avertissement nomme un fichier qui n'est plus, et la compaction, réussie, n'est
// plus due.
#[test]
fn un_voisin_d_ajout_mis_en_place_non_rendu_durable_avertit() {
    let chemin = commun::dossier_jetable("demarrage-voisin-d-ajout-non-durable").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    drop(fonds);
    let voisin = l_ajout_en_voisin(&chemin);
    let nom_du_voisin = voisin
        .file_name()
        .expect("le voisin a un nom")
        .to_string_lossy()
        .into_owned();
    let ajout = voisin.with_file_name(
        nom_du_voisin
            .strip_suffix(".4242.partiel")
            .expect("le voisin du test finit en .4242.partiel"),
    );

    // Le premier renommage rendu durable de l'ouverture est celui du démarrage ; la compaction
    // d'ouverture rend le sien durable après lui.
    let ouvert = {
        let _panne = couper(Etape::RendreDurable, 1);
        racine.fonds()
    };

    let mut fonds = ouvert
        .expect("un renommage non rendu durable n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds ouvert est en lecture seule"
    );
    match fonds.avertissements() {
        [Avertissement::NonDurable { chemin: nomme, .. }] => assert_eq!(
            nomme, &ajout,
            "l'avertissement ne nomme pas le fichier d'ajout mis en place"
        ),
        autres => panic!("attendu un seul NonDurable, rendu {autres:?}"),
    }
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds ouvert ne porte pas l'écriture du voisin mis en place"
    );
    assert!(!voisin.exists(), "le voisin n'a pas pris sa place");
    // La compaction d'ouverture a fondu le fichier d'ajout que nomme l'avertissement, puis l'a
    // retiré : le fichier complet, relu seul, porte l'écriture.
    assert_eq!(
        entrees(&dossier_des_ajouts(&chemin)),
        Vec::<String>::new(),
        "la compaction d'ouverture n'a pas retiré le fichier d'ajout mis en place"
    );
    let seul = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    assert!(
        porte_l_idee(&seul, "idee-2"),
        "le fichier complet relu seul ne porte pas l'écriture du voisin mis en place"
    );
    assert_eq!(
        seul.get_heads(),
        fonds.automerge().get_heads(),
        "le fichier complet relu seul ne mène pas aux heads du fonds ouvert"
    );
    // Aucune compaction n'est due : l'écriture suivante, légère, garde son fichier d'ajout.
    noter(&mut fonds, "idee-3", "Une coda en suspens");
    assert_eq!(
        entrees(&dossier_des_ajouts(&chemin)).len(),
        1,
        "l'écriture suivante a compacté le fonds : la compaction d'ouverture est restée due"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    for idee in ["idee-2", "idee-3"] {
        assert!(
            porte_l_idee(fonds.automerge(), idee),
            "rouvert sans panne, le fonds ne porte pas {idee}"
        );
    }
}

// § 31 · 10, test 8 : un voisin incomplet n'est pas lu comme une écriture
#[test]
fn un_voisin_incomplet_ou_non_conforme_est_retire() {
    let chemin = commun::dossier_jetable("demarrage-voisins-retires").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    let heads = fonds.automerge().get_heads();
    let ajouts = dossier_des_ajouts(&chemin);

    // Le voisin vidé d'une opération refusée : son retrait coupé, il reste, vide.
    let refus = {
        let _renommage = couper(Etape::Renommage, 1);
        let _retrait = couper(Etape::Retrait, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-3", "Une coda en suspens")
        })
    };
    assert!(
        matches!(refus, Err(Erreur::Disque { .. })),
        "attendu l'opération refusée ; rendu {refus:?}"
    );
    let restes = partiels(&ajouts);
    assert_eq!(
        restes.len(),
        1,
        "l'opération refusée n'a pas laissé son voisin : {restes:?}"
    );
    let vide = ajouts.join(&restes[0]);
    assert!(
        fs::read(&vide).expect("lire le voisin vidé").is_empty(),
        "le voisin de l'opération refusée n'est pas vidé"
    );

    // Une écriture dont le test ôte le fichier d'ajout : son contenu fait les voisins tronqué et
    // vide, comme une écriture coupée avant son renommage.
    let avant = entrees(&ajouts);
    noter(&mut fonds, "idee-4", "Un couplet plus calme");
    let noms = commun::nouveaux(&avant, &entrees(&ajouts));
    assert_eq!(noms.len(), 1, "un seul fichier d'ajout de plus : {noms:?}");
    let ajout = ajouts.join(&noms[0]);
    let contenu = fs::read(&ajout).expect("lire le fichier d'ajout");
    fs::remove_file(&ajout).expect("ôter le fichier d'ajout");
    drop(fonds);

    let documents = chemin.join("documents");
    let deposes = [
        (
            ajouts.join(format!("{}.4242.partiel", noms[0])),
            contenu[..contenu.len() / 2].to_vec(),
        ),
        (ajouts.join(format!("{}.4243.partiel", noms[0])), Vec::new()),
        (
            dossier_du_fonds(&chemin).join("complet.automerge.4242.partiel"),
            b"pas un document".to_vec(),
        ),
        (
            documents.join("x.4242.partiel"),
            b"hors du dossier d'un document".to_vec(),
        ),
    ];
    for (depose, octets) in &deposes {
        fs::write(depose, octets).unwrap_or_else(|e| panic!("déposer {} : {e}", depose.display()));
    }

    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(
        fonds.automerge().get_heads(),
        heads,
        "le fonds rouvert n'a pas les heads d'avant"
    );
    assert!(
        !vide.exists(),
        "le voisin vidé de l'opération refusée n'est pas retiré"
    );
    for (depose, _) in &deposes {
        assert!(!depose.exists(), "{} n'est pas retiré", depose.display());
    }
    assert_eq!(
        entrees(&documents),
        ["fonds"],
        "documents/ garde un .partiel"
    );
    assert_eq!(
        entrees(&dossier_du_fonds(&chemin)),
        ["ajouts", "complet.automerge"],
        "le dossier du fonds garde un .partiel"
    );
    assert_eq!(
        entrees(&ajouts),
        Vec::<String>::new(),
        "la compaction d'ouverture a laissé un fichier dans ajouts/"
    );
}

// § 31 · 10, test 8 : un voisin non conforme dont le retrait est refusé n'est pas lu comme une écriture
// Le retrait d'un `.partiel` que le démarrage retire est tu s'il est refusé, et se retente à
// l'ouverture suivante (§ 31 · 2.1).
#[test]
fn un_voisin_non_conforme_qui_reste_ne_se_lit_pas() {
    let chemin = commun::dossier_jetable("demarrage-voisin-non-conforme-reste").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    let heads = fonds.automerge().get_heads();
    let valeur = fonds.automerge().hydrate(None);
    // Une modification valide que le fonds n'a pas, sous un nom dont elle ne donne pas
    // l'empreinte : chargée, elle changerait les heads.
    let mut copie = fonds.automerge().fork();
    let mut tx = copie.transaction();
    commun::noter(&mut tx, "idee-3", "Une coda en suspens").expect("noter l'idée");
    tx.commit_with(
        CommitOptions::default()
            .with_message(MESSAGE_DU_GESTE)
            .with_time(HEURE + 1),
    );
    let de_plus = copie.save_after(&heads);
    drop(fonds);
    let ajouts = dossier_des_ajouts(&chemin);
    let nom = format!("{}.automerge.4242.partiel", "0".repeat(64));
    let partiel = ajouts.join(&nom);
    fs::write(&partiel, &de_plus).expect("déposer le .partiel non conforme");

    // Le dossier du fonds et `documents/` n'ont aucun `.partiel`, et la compaction d'ouverture
    // retire après le démarrage : le premier retrait de l'ouverture est celui du `.partiel`.
    let fonds = {
        let _panne = couper(Etape::Retrait, 1);
        ouvrir_le_fonds(&racine)
    };

    assert_eq!(
        fonds.automerge().get_heads(),
        heads,
        "le .partiel non conforme resté a changé les heads du fonds"
    );
    assert_eq!(
        fonds.automerge().hydrate(None),
        valeur,
        "le .partiel non conforme resté a changé le contenu du fonds"
    );
    assert!(
        fs::read(&partiel).expect("relire le .partiel resté") == de_plus,
        "le .partiel dont le retrait est refusé n'est plus là, octet pour octet"
    );
    assert_eq!(
        entrees(&ajouts),
        [nom.as_str()],
        "la compaction d'ouverture n'a pas retiré le seul fichier d'ajout"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    assert_eq!(
        fonds.automerge().get_heads(),
        heads,
        "rouvert sans panne, le fonds n'a pas les heads d'avant"
    );
    assert!(
        !partiel.exists(),
        "rouvert sans panne, le .partiel non conforme n'est pas retiré"
    );
}

// § 31 · 10, test 9 : renommage du fichier complet perdu, les ajouts absorbés déjà retirés, puis rouvrir
#[test]
fn le_voisin_du_complet_se_lit_puis_se_fond() {
    let chemin = commun::dossier_jetable("demarrage-voisin-du-complet").join("donnees");
    let racine = ouvrir(&chemin);
    let voisin = un_voisin_du_complet_laisse(&racine, "complet.automerge.4242.partiel");

    let fonds = ouvrir_le_fonds(&racine);

    porte_les_trois_idees(fonds.automerge(), "le fonds rouvert");
    assert!(
        !voisin.exists(),
        "le voisin du fichier complet n'est pas retiré"
    );
    let seul = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    porte_les_trois_idees(&seul, "le fichier complet relu seul");
    assert_eq!(
        seul.get_heads(),
        fonds.automerge().get_heads(),
        "le fichier complet relu seul ne mène pas aux heads du fonds rouvert"
    );
}

// § 31 · 10, test 9 : de même quand le voisin laissé porte le numéro du processus qui rouvre
#[test]
fn un_voisin_au_numero_du_processus_n_est_pas_ecrase() {
    let chemin = commun::dossier_jetable("demarrage-voisin-du-processus").join("donnees");
    let racine = ouvrir(&chemin);
    let nom_du_voisin = format!("complet.automerge.{}.partiel", std::process::id());
    let voisin = un_voisin_du_complet_laisse(&racine, &nom_du_voisin);
    let du_voisin = empreinte(&fs::read(&voisin).expect("lire le voisin laissé"));

    // Le voisin laissé se lit, sans renommage : le premier renommage de l'ouverture est celui de
    // sa compaction.
    let ouvert = {
        let _panne = couper(Etape::Renommage, 1);
        racine.fonds()
    };

    let fonds = ouvert
        .expect("une compaction d'ouverture qui échoue n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds ouvert est en lecture seule"
    );
    assert!(
        matches!(
            fonds.avertissements(),
            [Avertissement::Compaction(Erreur::Disque { .. })]
        ),
        "attendu la compaction refusée par le disque en avertissement ; rendu {:?}",
        fonds.avertissements()
    );
    porte_les_trois_idees(fonds.automerge(), "le fonds ouvert");
    assert_eq!(
        empreinte(&fs::read(&voisin).expect("relire le voisin laissé")),
        du_voisin,
        "le voisin laissé n'a plus son empreinte"
    );
    assert_eq!(
        entrees(&dossier_du_fonds(&chemin)),
        ["ajouts", "complet.automerge", nom_du_voisin.as_str()],
        "le voisin de la compaction refusée reste à côté du fichier complet"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    porte_les_trois_idees(fonds.automerge(), "rouvert sans panne, le fonds");
}

// § 31 · 10, test 8 : à la naissance, renommage du premier fichier complet perdu, puis relancer
#[test]
fn a_la_naissance_le_voisin_du_premier_complet_fait_exister_le_fonds() {
    let chemin = commun::dossier_jetable("demarrage-voisin-de-la-naissance").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let dossier = dossier_du_fonds(&chemin);
    let complet = dossier.join("complet.automerge");
    let voisin = dossier.join("complet.automerge.4242.partiel");
    fs::rename(&complet, &voisin).expect("renommer le fichier complet en son voisin");
    let du_voisin = empreinte(&fs::read(&voisin).expect("lire le voisin"));

    let seconde = racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
        commun::noter(tx, "idee-2", "Un couplet plus calme")
    });

    match seconde {
        Err(Erreur::FondsDejaNe) => {}
        Err(autre) => panic!("attendu FondsDejaNe, rendu {autre:?}"),
        Ok(_) => {
            panic!("le fonds est né une seconde fois, à côté du voisin de son fichier complet")
        }
    }
    assert_eq!(
        empreinte(&fs::read(&voisin).expect("relire le voisin")),
        du_voisin,
        "le voisin n'a plus son empreinte"
    );
    assert!(
        !complet.exists(),
        "la seconde naissance a écrit un fichier complet"
    );

    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-1"),
        "le fonds ouvert ne porte pas l'idée de sa naissance"
    );
    assert!(complet.exists(), "le fichier complet n'est pas en place");
    assert!(!voisin.exists(), "le voisin n'est pas retiré");
}

/// Crée le dossier du fonds de la racine des données `chemin`, et y dépose, seul, un voisin vide
/// du fichier complet, `complet.automerge.4242.partiel`. Rend son chemin.
fn deposer_un_voisin_du_complet_vide(chemin: &Path) -> PathBuf {
    let dossier = dossier_du_fonds(chemin);
    fs::create_dir_all(&dossier).expect("créer le dossier du fonds");
    let voisin = dossier.join("complet.automerge.4242.partiel");
    fs::write(&voisin, b"").expect("déposer le voisin vide");
    voisin
}

#[test]
fn un_voisin_du_complet_vide_laisse_pupitre_neuf() {
    let d = commun::dossier_jetable("demarrage-voisin-du-complet-vide");
    let chemin = d.join("donnees");
    let racine = ouvrir(&chemin);
    let voisin = deposer_un_voisin_du_complet_vide(&chemin);

    match racine.fonds() {
        Ok(None) => {}
        Ok(Some(_)) => panic!("un voisin vide du fichier complet a fait exister le fonds"),
        Err(autre) => panic!("attendu aucun fonds ; rendu {autre:?}"),
    }
    assert!(!voisin.exists(), "le voisin vide n'est pas retiré");

    drop(faire_naitre(&racine));

    // Son retrait refusé, le voisin vide reste quand la lecture du dossier, puis la naissance, le
    // lisent : ni l'une ni l'autre ne le compte. Rien d'autre ne se retire à l'ouverture d'un
    // fonds qui n'est pas né : le premier retrait de l'ouverture est le sien.
    let chemin = d.join("retrait-refuse");
    let racine = ouvrir(&chemin);
    let voisin = deposer_un_voisin_du_complet_vide(&chemin);

    let ouvert = {
        let _panne = couper(Etape::Retrait, 1);
        racine.fonds()
    };

    match ouvert {
        Ok(None) => {}
        Ok(Some(_)) => panic!("retrait refusé : le voisin vide a fait exister le fonds"),
        Err(autre) => panic!("retrait refusé : attendu aucun fonds ; rendu {autre:?}"),
    }
    assert_eq!(
        fs::read(&voisin).expect("relire le voisin vide resté"),
        Vec::<u8>::new(),
        "retrait refusé : le voisin vide n'est plus là, vide"
    );

    drop(faire_naitre(&racine));

    assert_eq!(
        fs::read(&voisin).expect("relire le voisin vide après la naissance"),
        Vec::<u8>::new(),
        "la naissance a touché au voisin vide"
    );
    drop(ouvrir_le_fonds(&racine));
    assert!(
        !voisin.exists(),
        "rouvert sans panne, le voisin vide n'est pas retiré"
    );
}

// § 31 · 2.1 : un nom déjà pris reçoit un suffixe ; deux voisins laissés sous le même numéro de
// processus
#[test]
fn deux_voisins_du_complet_se_lisent_puis_se_fondent() {
    let chemin = commun::dossier_jetable("demarrage-deux-voisins-du-complet").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let naissance = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    let dossier = dossier_du_fonds(&chemin);
    // Ni fichier complet, ni `ajouts/`, comme une copie qui ne garde pas un dossier vide :
    // l'ouverture lit un `ajouts/` absent comme vide, et ne le crée pas.
    fs::remove_file(dossier.join("complet.automerge")).expect("ôter le fichier complet");
    fs::remove_dir(dossier.join("ajouts")).expect("ôter ajouts/, vide");
    // Deux copies de la naissance, chacune avec une idée que l'autre n'a pas : chacun des deux
    // voisins doit être lu.
    let premier = dossier.join("complet.automerge.4242.partiel");
    let second = dossier.join("complet.automerge.4242-2.partiel");
    let du_premier = une_idee_de_plus(&naissance, ActorId::random(), "idee-3", "Une coda");
    let du_second = une_idee_de_plus(&naissance, ActorId::random(), "idee-2", "Un pont");
    fs::write(&premier, du_premier).expect("écrire le premier voisin");
    fs::write(&second, du_second).expect("écrire le second voisin");

    let fonds = ouvrir_le_fonds(&racine);

    for (idee, voisin) in [("idee-3", "premier"), ("idee-2", "second")] {
        assert!(
            porte_l_idee(fonds.automerge(), idee),
            "le fonds rouvert ne porte pas l'idée du {voisin} voisin"
        );
    }
    assert!(!premier.exists(), "le premier voisin n'est pas retiré");
    assert!(!second.exists(), "le second voisin n'est pas retiré");
    assert_eq!(
        entrees(&dossier),
        ["complet.automerge"],
        "le fonds rouvert n'a pas son seul fichier complet"
    );
}

#[test]
fn un_fichier_etranger_est_laisse_tel_quel() {
    let chemin = commun::dossier_jetable("demarrage-fichier-etranger").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    let heads = fonds.automerge().get_heads();
    drop(fonds);
    // Un dans chacun des trois dossiers que le démarrage parcourt.
    let etrangers = [
        chemin.join("documents").join("desktop.ini"),
        dossier_du_fonds(&chemin).join("Thumbs.db"),
        dossier_des_ajouts(&chemin).join("desktop.ini"),
    ];
    for etranger in &etrangers {
        fs::write(etranger, "inventé")
            .unwrap_or_else(|e| panic!("déposer {} : {e}", etranger.display()));
    }

    let mut fonds = ouvrir_le_fonds(&racine);
    fonds.compacter().expect("compacter");

    for etranger in &etrangers {
        assert!(
            fs::read(etranger).unwrap_or_else(|e| panic!("relire {} : {e}", etranger.display()))
                == "inventé".as_bytes(),
            "{} n'est plus là, octet pour octet",
            etranger.display()
        );
    }
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "un fichier étranger met le fonds en lecture seule"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads,
        "le fonds rouvert n'a pas les heads d'avant"
    );
}

#[test]
fn le_demarrage_ne_touche_rien_sans_le_verrou() {
    let chemin = commun::dossier_jetable("demarrage-sans-le-verrou").join("donnees");
    let racine = ouvrir(&chemin);
    // Le fonds nomme la captation `e` : un rangement lancé sans le verrou renommerait son nom
    // provisoire, comme un démarrage retirerait le `.partiel` non conforme d'`ajouts/`.
    let du_contenu = octets(5, 300);
    let e = empreinte(&du_contenu);
    drop(naitre_avec_les_prises(&racine, &[&e]));
    let documents = chemin.join("documents");
    let captations = chemin.join("captations");
    fs::write(
        dossier_des_ajouts(&chemin).join(format!("{}.automerge.4242.partiel", "0".repeat(64))),
        b"une ecriture interrompue",
    )
    .expect("déposer un .partiel dans ajouts/");
    deposer_un_nom_provisoire(&chemin, &e, &du_contenu);
    let documents_avant = empreintes(&documents);
    let captations_avant = empreintes(&captations);

    match RacineDesDonnees::ouvrir(&chemin) {
        Err(Erreur::RacineDesDonneesTenue) => {}
        Err(autre) => panic!("attendu RacineDesDonneesTenue, rendu {autre:?}"),
        Ok(_) => panic!("la seconde ouverture a pris le verrou que tient la première"),
    }

    // Jamais `empreintes` de la racine des données : sous Windows, le verrou que tient la racine
    // ouverte peut refuser la lecture du fichier `verrou`.
    assert_eq!(
        empreintes(&documents),
        documents_avant,
        "documents/ a changé sans le verrou"
    );
    assert_eq!(
        empreintes(&captations),
        captations_avant,
        "captations/ a changé sans le verrou"
    );
    drop(racine);
}

// § 31 · 2.1 : l'ouverture ne crée pas `ajouts/` ; la première écriture le recrée.
#[test]
fn une_ecriture_recree_ajouts_s_il_manque() {
    let chemin = commun::dossier_jetable("demarrage-ajouts-absent-ecriture").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let ajouts = dossier_des_ajouts(&chemin);
    fs::remove_dir(&ajouts).expect("ôter ajouts/, vide");
    let mut fonds = ouvrir_le_fonds(&racine);
    assert!(!ajouts.exists(), "l'ouverture a créé ajouts/");

    noter(&mut fonds, "idee-2", "Un pont en mineur");

    let noms = entrees(&ajouts);
    assert_eq!(
        noms.len(),
        1,
        "ajouts/ recréé n'a pas un seul fichier d'ajout : {noms:?}"
    );
    drop(fonds);
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "rouvert, le fonds ne porte pas l'écriture faite sans ajouts/"
    );
}

// § 31 · 2.1 : de même pour ce que la synchronisation reçoit.
#[test]
fn une_reception_recree_ajouts_s_il_manque() {
    let (chemin_a, chemin_b) = commun::deux_copies("demarrage-ajouts-absent-reception");
    let ajouts_a = dossier_des_ajouts(&chemin_a);
    fs::remove_dir(&ajouts_a).expect("ôter ajouts/ d'A, vide");
    let racine_a = ouvrir(&chemin_a);
    let racine_b = ouvrir(&chemin_b);
    let mut a = ouvrir_le_fonds(&racine_a);
    let mut b = ouvrir_le_fonds(&racine_b);
    noter(&mut b, "idee-2", "Un pont en mineur");

    // Chaque réception d'A est faite, sans avertissement.
    commun::reunir(&mut a, &mut b);

    assert!(
        porte_l_idee(a.automerge(), "idee-2"),
        "A ne porte pas l'idée reçue de B"
    );
    assert!(
        !entrees(&ajouts_a).is_empty(),
        "la réception n'a rien écrit dans ajouts/ d'A"
    );
    drop(a);
    let a = ouvrir_le_fonds(&racine_a);
    assert!(
        porte_l_idee(a.automerge(), "idee-2"),
        "rouvert, A ne porte pas l'idée reçue de B"
    );
}

// Un fonds ouvert par son seul voisin du fichier complet, sa compaction d'ouverture en échec :
// le fichier complet absent pèse 0 octet, et l'écriture suivante retente la compaction due.
#[test]
fn sans_fichier_complet_l_ecriture_suivante_retente_la_compaction() {
    let chemin = commun::dossier_jetable("demarrage-compaction-sans-complet").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let dossier = dossier_du_fonds(&chemin);
    let complet = dossier.join("complet.automerge");
    let voisin = dossier.join("complet.automerge.4242.partiel");
    fs::rename(&complet, voisin).expect("renommer le fichier complet en son voisin");

    // Le voisin se lit sans renommage : le premier renommage de l'ouverture est celui de sa
    // compaction.
    let ouvert = {
        let _panne = couper(Etape::Renommage, 1);
        racine.fonds()
    };

    let mut fonds = ouvert
        .expect("une compaction d'ouverture qui échoue n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert!(
        matches!(
            fonds.avertissements(),
            [Avertissement::Compaction(Erreur::Disque { .. })]
        ),
        "attendu la compaction d'ouverture refusée en avertissement ; rendu {:?}",
        fonds.avertissements()
    );
    assert!(
        !complet.exists(),
        "la compaction d'ouverture coupée a mis un fichier complet en place"
    );

    noter(&mut fonds, "idee-2", "Un pont en mineur");

    assert_eq!(
        entrees(&dossier),
        ["ajouts", "complet.automerge"],
        "la compaction retentée n'a pas mis le fichier complet à la place du voisin"
    );
    drop(fonds);
    let fonds = ouvrir_le_fonds(&racine);
    for idee in ["idee-1", "idee-2"] {
        assert!(
            porte_l_idee(fonds.automerge(), idee),
            "rouvert, le fonds ne porte pas {idee}"
        );
    }
}

// La règle de sûreté du démarrage : un voisin du fichier complet que le disque refuse de lire
// fait échouer l'ouverture et la naissance, la garde rendue, et Pupitre ne paraît jamais neuf.
#[test]
fn un_voisin_du_complet_que_le_disque_refuse_fait_echouer_l_ouverture() {
    let chemin = commun::dossier_jetable("demarrage-voisin-refuse-par-le-disque").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let dossier = dossier_du_fonds(&chemin);
    let voisin = dossier.join("complet.automerge.4242.partiel");
    fs::rename(dossier.join("complet.automerge"), &voisin)
        .expect("renommer le fichier complet en son voisin");

    // Sans fichier complet, la première lecture de l'ouverture est celle du démarrage, la seconde
    // celle de `lire_le_dossier`.
    for rang in [1, 2] {
        let ouvert = {
            let _panne = couper(Etape::Relecture, rang);
            racine.fonds()
        };
        match ouvert {
            Err(Erreur::Disque { chemin: refuse, .. }) => assert_eq!(
                refuse, voisin,
                "lecture {rang} refusée : l'erreur ne nomme pas le voisin"
            ),
            Err(autre) => panic!("lecture {rang} refusée : attendu Disque, rendu {autre:?}"),
            Ok(None) => panic!("lecture {rang} refusée : Pupitre paraît neuf, prêt à renaître"),
            Ok(Some(_)) => panic!("lecture {rang} refusée : le fonds s'est ouvert sans son voisin"),
        }
        assert!(
            voisin.exists(),
            "lecture {rang} refusée : le voisin n'est plus là"
        );
    }
    let naissance = {
        let _panne = couper(Etape::Relecture, 1);
        racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-2", "Un couplet plus calme")
        })
    };
    assert!(
        matches!(naissance, Err(Erreur::Disque { .. })),
        "la lecture du voisin refusée, attendu Disque à la naissance ; rendu {naissance:?}"
    );

    // La garde rendue, le fonds s'ouvre sans panne, avec l'idée de sa naissance.
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-1"),
        "sans panne, le fonds ouvert ne porte pas l'idée de sa naissance"
    );
}

// Deux voisins lisibles du fichier complet, une modification du même acteur au même rang dans
// chacun : le premier, dans l'ordre des noms, se charge ; Automerge refuse le second après lui.
#[test]
fn un_voisin_du_complet_qu_automerge_refuse_ouvre_en_lecture_seule() {
    let chemin = commun::dossier_jetable("demarrage-voisin-refuse-par-automerge").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let naissance = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    let dossier = dossier_du_fonds(&chemin);
    fs::remove_file(dossier.join("complet.automerge")).expect("ôter le fichier complet");
    // `….4242-2.partiel` passe avant `….4242.partiel` dans l'ordre des noms.
    let premier = dossier.join("complet.automerge.4242-2.partiel");
    let second = dossier.join("complet.automerge.4242.partiel");
    let en_double = || ActorId::from(b"acteur-en-double".as_slice());
    let du_premier = une_idee_de_plus(&naissance, en_double(), "idee-2", "Un pont en mineur");
    let du_second = une_idee_de_plus(&naissance, en_double(), "idee-3", "Une coda en suspens");
    fs::write(premier, du_premier).expect("écrire le premier voisin");
    fs::write(&second, du_second).expect("écrire le second voisin");
    let avant = empreintes(&dossier);

    let mut fonds = commun::ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::CompletIllisible { chemin: second };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fonds ne nomme pas le voisin qu'Automerge refuse"
    );
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "l'idée du premier voisin ne se lit pas"
    );
    commun::constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
    drop(fonds);

    let seconde = racine.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
        commun::noter(tx, "idee-4", "Un couplet plus calme")
    });
    match seconde {
        Err(Erreur::FondsDejaNe) => {}
        Err(autre) => panic!("attendu FondsDejaNe, rendu {autre:?}"),
        Ok(_) => panic!("le fonds est né une seconde fois, à côté de ses voisins lisibles"),
    }
    assert_eq!(
        empreintes(&dossier),
        avant,
        "un fichier du fonds a changé depuis l'ouverture en lecture seule"
    );
}

// § 31 · 2.1 : un `ajouts/` qui ne peut être recréé refuse l'écriture, et la mémoire se relit du
// disque.
#[test]
fn une_ecriture_qui_ne_peut_recreer_ajouts_est_refusee() {
    let chemin = commun::dossier_jetable("demarrage-ajouts-non-recree").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let mut fonds = ouvrir_le_fonds(&racine);
    let dossier = dossier_du_fonds(&chemin);
    // Le dossier du document ôté, le fonds ouvert : `create_dir` d'`ajouts/` échoue sans son
    // parent.
    fs::remove_dir_all(&dossier).expect("ôter le dossier du document");

    let refus = commun::ecrire(&mut fonds, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    });

    match refus {
        Err(Erreur::Disque { chemin: refuse, .. }) => assert_eq!(
            refuse,
            dossier_des_ajouts(&chemin),
            "le refus ne nomme pas ajouts/"
        ),
        autre => panic!("attendu l'écriture refusée par le disque ; rendu {autre:?}"),
    }
    // Relue d'un dossier absent, la mémoire est un document neuf, sans genèse : en lecture seule,
    // sur le fichier complet absent.
    assert_eq!(
        fonds.lecture_seule(),
        Some(&LectureSeule::CompletIllisible {
            chemin: dossier.join("complet.automerge")
        }),
        "relu d'un dossier absent, le fonds n'est pas en lecture seule sur le fichier complet"
    );
    assert!(
        fonds.automerge().get_heads().is_empty(),
        "la mémoire relue d'un dossier absent porte des modifications"
    );
    assert!(
        !dossier.exists(),
        "l'écriture refusée a recréé le dossier du document"
    );
}

// § 31 · 2.1 : un `ajouts/` recréé dont la création n'a pu être rendue durable se retire, et
// l'écriture suivante le recrée. Hors de Windows seulement, où un dossier se synchronise. Aucune
// panne ne s'injecte dans cette synchronisation : sans droit de lecture, le dossier du document
// accepte un dossier nouveau, mais ne s'ouvre pas, et sa synchronisation échoue.
#[cfg(unix)]
#[test]
fn un_ajouts_recree_non_rendu_durable_se_retire() {
    use std::os::unix::fs::PermissionsExt;

    let chemin = commun::dossier_jetable("demarrage-ajouts-non-durable").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let dossier = dossier_du_fonds(&chemin);
    let ajouts = dossier_des_ajouts(&chemin);
    fs::remove_dir(&ajouts).expect("ôter ajouts/, vide");
    let mut fonds = ouvrir_le_fonds(&racine);
    let droits = |mode: u32| {
        fs::set_permissions(&dossier, fs::Permissions::from_mode(mode))
            .unwrap_or_else(|e| panic!("changer les droits de {} : {e}", dossier.display()))
    };

    droits(0o300);
    let se_liste = fs::read_dir(&dossier).is_ok();
    let refus = commun::ecrire(&mut fonds, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    });
    // Les droits rendus avant tout constat : le dossier jetable doit pouvoir s'effacer.
    droits(0o755);

    assert!(
        !se_liste,
        "sans droit de lecture, le dossier du document se liste : ce processus passe outre les droits"
    );
    match refus {
        Err(Erreur::Disque { chemin: refuse, .. }) => assert_eq!(
            refuse, dossier,
            "le refus ne nomme pas le dossier du document"
        ),
        autre => panic!("attendu l'écriture refusée par le disque ; rendu {autre:?}"),
    }
    assert!(
        !ajouts.exists(),
        "ajouts/, dont la création n'a pu être rendue durable, reste"
    );
    drop(fonds);

    let mut fonds = ouvrir_le_fonds(&racine);
    noter(&mut fonds, "idee-2", "Un pont en mineur");
    assert_eq!(
        entrees(&ajouts).len(),
        1,
        "l'écriture suivante n'a pas recréé ajouts/ avec son fichier d'ajout"
    );
}

// § 31 · 10, test 4 : renommage perdu après la réponse, la captation nommée par un document prend son nom au démarrage
#[test]
fn une_captation_nommee_prend_son_nom_au_demarrage() {
    let chemin = commun::dossier_jetable("demarrage-nom-provisoire-range").join("donnees");
    let racine = ouvrir(&chemin);
    let du_contenu = octets(5, 300);
    let e = empreinte(&du_contenu);
    let entier = octets(9, 300);
    let e2 = empreinte(&entier);
    drop(naitre_avec_les_prises(&racine, &[&e, &e2]));
    let captations = chemin.join("captations");
    assert!(
        !captations.join(&e[..2]).exists(),
        "le sous-dossier de la captation existe avant le démarrage"
    );
    let provisoire = deposer_un_nom_provisoire(&chemin, &e, &du_contenu);
    let non_nomme = octets(8, 100);
    let e8 = empreinte(&non_nomme);
    let restants = [
        deposer_un_nom_provisoire(&chemin, &e8, &non_nomme),
        // Un versement coupé : la moitié des octets de `e2`, sous son nom provisoire.
        deposer_un_nom_provisoire(&chemin, &e2, &entier[..entier.len() / 2]),
    ];
    let restants_avant: Vec<Vec<u8>> = restants
        .iter()
        .map(|restant| fs::read(restant).expect("lire un nom provisoire déposé"))
        .collect();

    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(
        racine
            .captations()
            .relire(&e)
            .expect("relire la captation nommée"),
        du_contenu,
        "la captation nommée n'a pas pris son nom"
    );
    assert!(
        !provisoire.exists(),
        "le nom provisoire de la captation nommée reste"
    );
    for (restant, avant) in restants.iter().zip(&restants_avant) {
        assert!(
            fs::read(restant).unwrap_or_else(|e| panic!("relire {} : {e}", restant.display()))
                == *avant,
            "{} n'est plus là, octet pour octet",
            restant.display()
        );
    }
    pas_rangee(&racine, &e2, "le versement coupé de e2");
    pas_rangee(&racine, &e8, "le contenu non nommé");
    assert_eq!(
        racine
            .captations()
            .verifier()
            .expect("vérifier les captations"),
        Vec::<PathBuf>::new(),
        "une captation rangée trahit son nom"
    );
    drop(fonds);
}

// § 31 · 10, test 4 : renommage perdu après la réponse, la captation nommée par un document prend son nom au démarrage
#[test]
#[ignore = "attend la genèse de la v0.3.0, qui crée midi_verses et textes_verses"]
fn un_midi_ou_un_texte_verse_nomme_prend_son_nom_au_demarrage() {
    let chemin = commun::dossier_jetable("demarrage-midi-et-texte-ranges").join("donnees");
    let racine = ouvrir(&chemin);
    let versees = [
        (MIDI_VERSES, octets(6, 300)),
        (TEXTES_VERSES, octets(7, 300)),
    ];
    let mut fonds = naitre_avec_les_prises(&racine, &[]);
    for (conteneur, contenu) in &versees {
        let e = empreinte(contenu);
        faite(
            commun::ecrire(&mut fonds, |tx| nommer_sous(tx, conteneur, &e, "idee-1")),
            &format!("nommer {e} sous {conteneur}"),
        );
    }
    drop(fonds);
    let mut provisoires = Vec::new();
    for (conteneur, contenu) in &versees {
        let e = empreinte(contenu);
        provisoires.push(deposer_un_nom_provisoire(&chemin, &e, contenu));
        pas_rangee(
            &racine,
            &e,
            &format!("avant le démarrage, la captation nommée sous {conteneur}"),
        );
    }

    let fonds = ouvrir_le_fonds(&racine);

    for ((conteneur, contenu), provisoire) in versees.iter().zip(&provisoires) {
        assert_eq!(
            racine
                .captations()
                .relire(&empreinte(contenu))
                .unwrap_or_else(|erreur| {
                    panic!("relire la captation nommée sous {conteneur} : {erreur:?}")
                }),
            *contenu,
            "la captation nommée sous {conteneur} n'a pas pris son nom"
        );
        assert!(
            !provisoire.exists(),
            "le nom provisoire de la captation nommée sous {conteneur} reste"
        );
    }
    drop(fonds);
}

#[test]
fn un_nom_provisoire_qui_ne_se_range_pas_avertit() {
    let d = commun::dossier_jetable("demarrage-nom-provisoire-restant");
    let du_contenu = octets(5, 300);
    let e = empreinte(&du_contenu);

    // Le renommage refusé : le fonds n'a ni voisin ni fichier d'ajout, et le premier renommage de
    // l'ouverture est celui du rangement. La lecture refusée : le fichier complet se lit d'abord,
    // puis le nom provisoire.
    for (sorte, panne, rang) in [
        ("renommage", Etape::Renommage, 1),
        ("relecture", Etape::Relecture, 2),
    ] {
        let chemin = d.join(sorte);
        let racine = ouvrir(&chemin);
        drop(naitre_avec_les_prises(&racine, &[&e]));
        let provisoire = deposer_un_nom_provisoire(&chemin, &e, &du_contenu);

        let ouvert = {
            let _panne = couper(panne, rang);
            racine.fonds()
        };

        let fonds = ouvert
            .unwrap_or_else(|erreur| {
                panic!("{sorte} refusé : un nom provisoire resté empêche d'ouvrir ; {erreur:?}")
            })
            .expect("le fonds est né");
        assert_eq!(
            fonds.lecture_seule(),
            None,
            "{sorte} refusé : le fonds ouvert est en lecture seule"
        );
        match fonds.avertissements() {
            [Avertissement::NomProvisoireRestant { chemin: reste, .. }] => assert_eq!(
                reste, &provisoire,
                "{sorte} refusé : l'avertissement ne nomme pas le nom provisoire"
            ),
            autres => {
                panic!("{sorte} refusé : attendu un seul NomProvisoireRestant, rendu {autres:?}")
            }
        }
        assert!(
            fs::read(&provisoire).expect("relire le nom provisoire resté") == du_contenu,
            "{sorte} refusé : le nom provisoire n'est plus là, octet pour octet"
        );
        pas_rangee(&racine, &e, &format!("{sorte} refusé"));
        drop(fonds);

        let fonds = ouvrir_le_fonds(&racine);
        assert_eq!(
            racine
                .captations()
                .relire(&e)
                .expect("relire la captation rangée"),
            du_contenu,
            "{sorte} refusé, puis rouvert sans panne : la captation n'a pas pris son nom"
        );
        assert!(
            !provisoire.exists(),
            "{sorte} refusé, puis rouvert sans panne : le nom provisoire reste"
        );
        drop(fonds);
    }
}

// § 31 · 6.1 : en lecture seule, ce que le fonds nomme est ce qui se lit.
#[test]
fn en_lecture_seule_seul_ce_qui_se_lit_nomme() {
    let chemin = commun::dossier_jetable("demarrage-nom-provisoire-lecture-seule").join("donnees");
    let racine = ouvrir(&chemin);
    let du_contenu = octets(5, 300);
    let e = empreinte(&du_contenu);
    let de_e9 = octets(9, 300);
    let e9 = empreinte(&de_e9);
    let mut fonds = naitre_avec_les_prises(&racine, &[&e]);
    faite(
        commun::ecrire(&mut fonds, |tx| {
            commun::poser_une_prise(tx, &e9, "idee-1", FORMAT_DES_PRISES)
        }),
        "poser la prise de e9",
    );
    drop(fonds);
    let ajouts = dossier_des_ajouts(&chemin);
    let noms = entrees(&ajouts);
    assert_eq!(
        noms.len(),
        1,
        "ajouts/ n'a pas le seul fichier d'ajout de la prise de e9 : {noms:?}"
    );
    let ajout = ajouts.join(&noms[0]);
    assert!(
        ajout.is_file(),
        "le fichier d'ajout de la prise de e9 n'existe pas"
    );
    commun::alterer_l_octet_du_milieu(&ajout);
    let provisoire = deposer_un_nom_provisoire(&chemin, &e, &du_contenu);
    let provisoire_de_e9 = deposer_un_nom_provisoire(&chemin, &e9, &de_e9);

    let fonds = commun::ouvrir_le_fonds_tel_quel(&racine);

    assert_eq!(
        fonds.lecture_seule(),
        Some(&LectureSeule::AjoutAltere {
            chemin: ajout.clone()
        }),
        "le fonds ne nomme pas le fichier d'ajout altéré"
    );
    assert!(
        fonds.avertissements().is_empty(),
        "le fonds ouvert a des avertissements : {:?}",
        fonds.avertissements()
    );
    assert_eq!(
        racine
            .captations()
            .relire(&e)
            .expect("relire la captation que nomme ce qui se lit"),
        du_contenu,
        "la captation que nomme ce qui se lit n'a pas pris son nom"
    );
    assert!(
        !provisoire.exists(),
        "le nom provisoire de la captation que nomme ce qui se lit reste"
    );
    assert!(
        fs::read(&provisoire_de_e9).expect("relire le nom provisoire de e9") == de_e9,
        "le nom provisoire de e9 n'est plus là, octet pour octet"
    );
    pas_rangee(
        &racine,
        &e9,
        "e9, nommée par le seul fichier qui ne se lit pas",
    );
    drop(fonds);
}

// § 31 · 10, test 4 : aucun objet modifié après écriture
// De deux noms provisoires du même contenu nommé, le premier dans l'ordre des noms prend le nom de
// son empreinte ; l'autre reste, et ne se range jamais par-dessus la captation rangée (I14).
#[test]
fn de_deux_noms_provisoires_du_meme_contenu_le_premier_se_range() {
    let chemin = commun::dossier_jetable("demarrage-noms-provisoires-jumeaux").join("donnees");
    let racine = ouvrir(&chemin);
    let du_contenu = octets(5, 300);
    let e = empreinte(&du_contenu);
    drop(naitre_avec_les_prises(&racine, &[&e]));
    let captations = chemin.join("captations");
    // `-` (0x2D) passe avant `.` (0x2E) : `<e>.4242-2.partiel` est le premier dans l'ordre des
    // noms.
    let premier = captations.join(format!("{e}.4242-2.partiel"));
    fs::write(&premier, &du_contenu).expect("déposer le premier nom provisoire");
    let second = deposer_un_nom_provisoire(&chemin, &e, &du_contenu);
    let rangee = commun::rangee(&chemin, &e);

    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(
        fs::read(&rangee).expect("lire la captation rangée"),
        du_contenu,
        "la captation nommée n'a pas pris son nom"
    );
    assert!(
        !premier.exists(),
        "le premier nom provisoire, dans l'ordre des noms, n'a pas pris son nom"
    );
    assert!(
        fs::read(&second).expect("relire le second nom provisoire") == du_contenu,
        "le second nom provisoire n'est plus là, octet pour octet"
    );
    drop(fonds);
    let rangee_avant = empreintes(&captations.join(&e[..2]));

    let fonds = ouvrir_le_fonds(&racine);

    assert!(
        fs::read(&second).expect("relire le second nom provisoire, rouvert") == du_contenu,
        "rouvert, le second nom provisoire s'est rangé par-dessus la captation rangée"
    );
    assert_eq!(
        empreintes(&captations.join(&e[..2])),
        rangee_avant,
        "rouvert, la captation rangée a changé"
    );
    drop(fonds);
}

#[test]
fn un_rangement_non_rendu_durable_avertit() {
    let chemin = commun::dossier_jetable("demarrage-rangement-non-durable").join("donnees");
    let racine = ouvrir(&chemin);
    let du_contenu = octets(5, 300);
    let e = empreinte(&du_contenu);
    drop(naitre_avec_les_prises(&racine, &[&e]));
    let provisoire = deposer_un_nom_provisoire(&chemin, &e, &du_contenu);
    let rangee = commun::rangee(&chemin, &e);

    // Le fonds n'a ni voisin ni fichier d'ajout : le démarrage dans `documents/` ne renomme rien,
    // et aucune compaction ne suit l'ouverture. Le premier renommage rendu durable de l'ouverture
    // est celui du rangement ; la synchronisation de `captations/`, quand le sous-dossier naît,
    // n'en est pas un.
    let ouvert = {
        let _panne = couper(Etape::RendreDurable, 1);
        racine.fonds()
    };

    let fonds = ouvert
        .expect("un rangement non rendu durable n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds ouvert est en lecture seule"
    );
    match fonds.avertissements() {
        [Avertissement::NonDurable { chemin: nomme, .. }] => assert_eq!(
            nomme, &rangee,
            "l'avertissement ne nomme pas la captation rangée"
        ),
        autres => panic!("attendu un seul NonDurable, rendu {autres:?}"),
    }
    assert_eq!(
        fs::read(&rangee).expect("lire la captation rangée"),
        du_contenu,
        "la captation n'est pas rangée, entière"
    );
    assert!(!provisoire.exists(), "le nom provisoire reste");
    drop(fonds);

    drop(ouvrir_le_fonds(&racine));
}

#[test]
fn un_dossier_des_captations_qui_ne_se_liste_pas_avertit() {
    let chemin = commun::dossier_jetable("demarrage-captations-illisible").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    // `captations/`, vide, remplacé par un fichier : il ne se liste plus.
    let captations = chemin.join("captations");
    fs::remove_dir(&captations).expect("ôter captations/, vide");
    fs::write(&captations, "inventé").expect("remplacer captations/ par un fichier");

    let fonds = commun::ouvrir_le_fonds_tel_quel(&racine);

    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds ouvert est en lecture seule"
    );
    match fonds.avertissements() {
        [Avertissement::NomProvisoireRestant { chemin: nomme, .. }] => assert_eq!(
            nomme, &captations,
            "l'avertissement ne nomme pas captations/"
        ),
        autres => panic!("attendu un seul NomProvisoireRestant, rendu {autres:?}"),
    }
    assert!(
        fs::read(&captations).expect("relire le fichier") == "inventé".as_bytes(),
        "le fichier qui remplace captations/ n'est plus là, octet pour octet"
    );
}
