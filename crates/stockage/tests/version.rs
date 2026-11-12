//! La version de schéma et les migrations (§ 31 · 2.4) : un fonds plus ancien migre à son
//! ouverture, et la migration s'écrit en un fichier d'ajout, avec son message ; deux répliques
//! qui migrent chacune de son côté écrivent la même modification ; un fonds de version de schéma
//! plus récente se lit sans écrire, même relu du disque ; une version de schéma illisible ouvre
//! le fonds en lecture seule ; un fonds illisible ne migre pas ; une migration que le disque
//! refuse n'ouvre pas le fonds.
//!
//! La migration de test est engendrée à l'exécution depuis la genèse (`migration_de_test`), et
//! entre dans la table le temps du test, par `fonds_selon` ; `fonds()` ouvre selon la table de
//! l'application, vide, qui ne connaît que la version de schéma 1.

mod commun;

use automerge::transaction::{Transactable, Transaction};
use automerge::{
    ActorId, Automerge, AutomergeError, Change, ObjId, ROOT, ReadDoc, ScalarValue, Value,
};
use commun::{
    CONTENEUR_DE_LA_MIGRATION, constater_la_lecture_seule, copier_l_arbre, deux_copies,
    dossier_des_ajouts, dossier_du_fonds, dossier_jetable, ecrire, empreintes, entrees,
    faire_naitre, faite, migration_de_test, migrations_de_test, noter, objet, octets, ouvrir,
    ouvrir_le_fonds_tel_quel, ouvrir_les_echanges, porte_l_idee, reunir, reunir_en_suivant_a,
};
use noyau::donnees::fonds::{ACTEUR_DE_LA_GENESE, VERSION_DE_SCHEMA};
use noyau::donnees::version::Migration;
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Document, Erreur, LectureSeule, RacineDesDonnees};

/// Le message de la migration vers la version de schéma 2 (§ 31 · 2.4).
const MESSAGE_DE_LA_MIGRATION: &str = "Fonds passé à la version de schéma 2";

/// Le fonds de `racine`, ouvert selon la table `migrations` (`fonds_selon`) : entier, sans
/// avertissement. Un test qui attend l'un ou l'autre ouvre par `fonds_selon` lui-même.
fn ouvrir_selon<'r>(racine: &'r RacineDesDonnees, migrations: &[Migration<'_>]) -> Document<'r> {
    let fonds = racine
        .fonds_selon(migrations)
        .expect("ouvrir le fonds selon la table")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds ouvert selon la table est en lecture seule"
    );
    assert!(
        fonds.avertissements().is_empty(),
        "le fonds ouvert selon la table a des avertissements : {:?}",
        fonds.avertissements()
    );
    fonds
}

/// La valeur simple que porte la clé `cle` de `objet`.
fn valeur(doc: &Automerge, objet: &ObjId, cle: &str) -> ScalarValue {
    match doc.get(objet, cle).expect("lire le document") {
        Some((Value::Scalar(valeur), _)) => valeur.into_owned(),
        autre => panic!("{cle} ne porte pas une valeur simple : {autre:?}"),
    }
}

/// Le nombre de modifications de `doc` qui portent le message de la migration.
fn migrations_dans(doc: &Automerge) -> usize {
    doc.get_changes(&[])
        .iter()
        .filter(|modification| modification.message() == Some(MESSAGE_DE_LA_MIGRATION))
        .count()
}

/// Pose `cle` = `texte`, une valeur simple, sous le conteneur de la migration, à la racine du
/// document ; un conteneur absent rend `AutomergeError::NotAnObject`.
fn poser(tx: &mut Transaction<'_>, cle: &str, texte: &str) -> Result<(), AutomergeError> {
    let conteneur = match tx.get(ROOT, CONTENEUR_DE_LA_MIGRATION)? {
        Some((Value::Object(_), conteneur)) => conteneur,
        _ => return Err(AutomergeError::NotAnObject),
    };
    tx.put(&conteneur, cle, texte)
}

#[test]
fn migrer_a_l_ouverture_ecrit_la_migration_avec_son_message() {
    let chemin = dossier_jetable("version-migrer").join("donnees");
    let m = migration_de_test();
    let table = migrations_de_test(&m);
    let racine = ouvrir(&chemin);
    let heads_de_la_naissance = faire_naitre(&racine).automerge().get_heads();

    let fonds = ouvrir_selon(&racine, &table);

    let doc = fonds.automerge();
    assert_eq!(
        valeur(doc, &ROOT, VERSION_DE_SCHEMA),
        ScalarValue::Uint(2),
        "le fonds migré n'est pas à la version de schéma 2"
    );
    let de_plus = doc.get_changes(&heads_de_la_naissance);
    assert_eq!(de_plus.len(), 1, "la migration, une modification de plus");
    let attendue = Change::try_from(m.as_slice()).expect("lire la migration de test");
    assert_eq!(
        de_plus[0].hash(),
        attendue.hash(),
        "la modification de plus n'est pas la migration de test"
    );
    assert_eq!(de_plus[0].message(), Some(MESSAGE_DE_LA_MIGRATION));
    assert_eq!(de_plus[0].timestamp(), 0, "la migration porte l'heure 0");
    assert_eq!(*de_plus[0].actor_id(), ActorId::from(ACTEUR_DE_LA_GENESE));
    objet(doc, &ROOT, CONTENEUR_DE_LA_MIGRATION);
    let heads_migrees = doc.get_heads();
    drop(fonds);

    let fonds = ouvrir_selon(&racine, &table);
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_migrees,
        "rouvert, le fonds n'a pas les heads de la migration"
    );
    assert!(
        fonds.automerge().get_changes(&heads_migrees).is_empty(),
        "rouvert, le fonds a une modification de plus"
    );
}

// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que N
#[test]
fn un_document_n_plus_1_se_lit_sans_ecrire() {
    let chemin = dossier_jetable("version-n-plus-1").join("donnees");
    let m = migration_de_test();
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    drop(ouvrir_selon(&racine, &migrations_de_test(&m)));
    let dossier = dossier_du_fonds(&chemin);
    let avant = empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::VersionPlusRecente { version: 2 };
    assert_eq!(fonds.lecture_seule(), Some(&raison));
    match ecrire(&mut fonds, |tx| noter(tx, "idee-2", "Une phrase refusée")) {
        Err(Erreur::LectureSeule(rendue)) => assert_eq!(
            rendue, raison,
            "l'écriture ne rend pas la raison de la lecture seule"
        ),
        autre => panic!("attendu LectureSeule à l'écriture, rendu {autre:?}"),
    }
    match fonds.compacter() {
        Err(Erreur::LectureSeule(rendue)) => assert_eq!(
            rendue, raison,
            "la compaction ne rend pas la raison de la lecture seule"
        ),
        autre => panic!("attendu LectureSeule à la compaction, rendu {autre:?}"),
    }
    racine
        .captations()
        .verser(&octets(4, 100))
        .expect("une captation se verse");
    assert_eq!(
        empreintes(&dossier),
        avant,
        "un fichier du fonds plus récent a changé"
    );
}

// § 31 · 10, test 10 : migrer deux répliques chacune de son côté, puis les fusionner
#[test]
fn deux_migrations_separees_n_en_font_qu_une() {
    let (chemin_a, chemin_b) = deux_copies("version-deux-migrations");
    let m = migration_de_test();
    let table = migrations_de_test(&m);
    let racine_a = ouvrir(&chemin_a);
    let racine_b = ouvrir(&chemin_b);
    let mut a = ouvrir_selon(&racine_a, &table);
    let mut b = ouvrir_selon(&racine_b, &table);

    reunir(&mut a, &mut b);

    assert_eq!(migrations_dans(a.automerge()), 1, "A, une seule migration");
    assert_eq!(migrations_dans(b.automerge()), 1, "B, une seule migration");
    assert_eq!(
        a.automerge().get_heads(),
        b.automerge().get_heads(),
        "les deux répliques n'ont pas les mêmes heads"
    );
}

// § 31 · 10, test 10 : deux éléments différents de la donnée nouvelle d'un même objet ancien
#[test]
fn deux_elements_de_la_donnee_nouvelle_restent() {
    let (chemin_a, chemin_b) = deux_copies("version-deux-elements");
    let m = migration_de_test();
    let table = migrations_de_test(&m);
    let racine_a = ouvrir(&chemin_a);
    let racine_b = ouvrir(&chemin_b);
    let mut a = ouvrir_selon(&racine_a, &table);
    let mut b = ouvrir_selon(&racine_b, &table);
    // Des clés plates sous le conteneur de la migration : deux objets `idee-1` créés de deux
    // côtés se masqueraient.
    faite(
        ecrire(&mut a, |tx| poser(tx, "idee-1/a", "x")),
        "A pose idee-1/a",
    );
    faite(
        ecrire(&mut b, |tx| poser(tx, "idee-1/b", "y")),
        "B pose idee-1/b",
    );

    reunir(&mut a, &mut b);

    for (nom, fonds) in [("A", &a), ("B", &b)] {
        let doc = fonds.automerge();
        let conteneur = objet(doc, &ROOT, CONTENEUR_DE_LA_MIGRATION);
        assert_eq!(
            valeur(doc, &conteneur, "idee-1/a"),
            ScalarValue::from("x"),
            "{nom} : idee-1/a"
        );
        assert_eq!(
            valeur(doc, &conteneur, "idee-1/b"),
            ScalarValue::from("y"),
            "{nom} : idee-1/b"
        );
    }
}

#[test]
fn une_version_de_schema_illisible_ouvre_en_lecture_seule() {
    let chemin = dossier_jetable("version-illisible").join("donnees");
    let racine = ouvrir(&chemin);
    {
        let mut fonds = faire_naitre(&racine);
        faite(
            ecrire(&mut fonds, |tx| tx.put(ROOT, VERSION_DE_SCHEMA, "deux")),
            "poser une version de schéma qui n'est pas un entier non signé",
        );
    }
    let dossier = dossier_du_fonds(&chemin);
    let avant = empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::VersionIllisible;
    assert_eq!(fonds.lecture_seule(), Some(&raison));
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

#[test]
fn un_document_illisible_ne_migre_pas() {
    let chemin = dossier_jetable("version-document-illisible").join("donnees");
    let m = migration_de_test();
    let racine = ouvrir(&chemin);
    {
        let mut fonds = faire_naitre(&racine);
        faite(
            ecrire(&mut fonds, |tx| noter(tx, "idee-2", "Une phrase altérée")),
            "noter idee-2",
        );
    }
    let ajouts = dossier_des_ajouts(&chemin);
    let noms = entrees(&ajouts);
    assert_eq!(noms.len(), 1, "un seul fichier d'ajout : {noms:?}");
    commun::alterer_l_octet_du_milieu(&ajouts.join(&noms[0]));
    let dossier = dossier_du_fonds(&chemin);
    let avant = empreintes(&dossier);

    let fonds = racine
        .fonds_selon(&migrations_de_test(&m))
        .expect("ouvrir le fonds selon la table")
        .expect("le fonds est né");

    assert!(
        matches!(
            fonds.lecture_seule(),
            Some(LectureSeule::AjoutAltere { .. })
        ),
        "attendu AjoutAltere, rendu {:?}",
        fonds.lecture_seule()
    );
    assert_eq!(
        migrations_dans(fonds.automerge()),
        0,
        "le fonds illisible a migré"
    );
    assert_eq!(
        empreintes(&dossier),
        avant,
        "un fichier du fonds illisible a changé"
    );
}

#[test]
fn une_migration_que_le_disque_refuse_n_ouvre_pas_le_fonds() {
    let d = dossier_jetable("version-migration-refusee");
    let m = migration_de_test();
    let table = migrations_de_test(&m);

    // Né, lâché, sans fichier d'ajout, voisin ni nom provisoire : le premier renommage de
    // l'ouverture est celui de la migration.
    let chemin = d.join("refusee");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let dossier = dossier_du_fonds(&chemin);
    let avant = empreintes(&dossier);
    {
        let _panne = couper(Etape::Renommage, 1);
        match racine.fonds_selon(&table) {
            Err(Erreur::Disque { .. }) => {}
            autre => panic!("attendu Disque à la migration refusée, rendu {autre:?}"),
        }
    }
    assert_eq!(
        empreintes(&dossier),
        avant,
        "la migration refusée a changé un fichier du fonds"
    );
    // La garde rendue, le fonds s'ouvre, et migre.
    let fonds = ouvrir_selon(&racine, &table);
    assert_eq!(
        valeur(fonds.automerge(), &ROOT, VERSION_DE_SCHEMA),
        ScalarValue::Uint(2),
        "le fonds rouvert sans panne n'a pas migré"
    );
    drop(fonds);

    // Dans une racine neuve, le premier `rendre_durable` de l'ouverture est celui de la
    // migration : en place, mais non durable, elle ouvre le fonds migré, avec l'avertissement.
    let chemin = d.join("non-durable");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let _panne = couper(Etape::RendreDurable, 1);
    let fonds = racine
        .fonds_selon(&table)
        .expect("ouvrir le fonds selon la table")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "le fonds migré est en lecture seule"
    );
    assert_eq!(
        valeur(fonds.automerge(), &ROOT, VERSION_DE_SCHEMA),
        ScalarValue::Uint(2),
        "le fonds n'a pas migré"
    );
    match fonds.avertissements() {
        [Avertissement::NonDurable { chemin: ajout, .. }] => {
            assert!(
                ajout.starts_with(dossier_des_ajouts(&chemin)),
                "{} n'est pas sous ajouts/",
                ajout.display()
            );
            assert!(ajout.exists(), "{} est absent", ajout.display());
        }
        autre => panic!("attendu un seul NonDurable, rendu {autre:?}"),
    }
}

// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que N
#[test]
fn une_copie_plus_recente_relue_du_disque_reste_en_lecture_seule() {
    // Une copie plus récente dont l'écriture d'une réception est refusée se relit du disque : la
    // relecture lit la version de schéma comme l'ouverture, et la copie reste en lecture seule.
    let d = dossier_jetable("version-relue");
    let (chemin_a, chemin_b) = (d.join("a"), d.join("b"));
    let m = migration_de_test();
    let table = migrations_de_test(&m);
    {
        let racine = ouvrir(&chemin_a);
        drop(faire_naitre(&racine));
        drop(ouvrir_selon(&racine, &table));
    }
    copier_l_arbre(&chemin_a, &chemin_b);
    let racine_a = ouvrir(&chemin_a);
    let racine_b = ouvrir(&chemin_b);
    // B connaît la version de schéma 2, et note idee-2 ; A ne connaît que la version 1.
    let mut b = ouvrir_selon(&racine_b, &table);
    faite(
        ecrire(&mut b, |tx| noter(tx, "idee-2", "Un pont en mineur")),
        "B note idee-2",
    );
    let mut a = ouvrir_le_fonds_tel_quel(&racine_a);
    let raison = LectureSeule::VersionPlusRecente { version: 2 };
    assert_eq!(a.lecture_seule(), Some(&raison));
    let (mut ea, mut eb) = ouvrir_les_echanges(&a, &b);
    let mut refusees = 0;

    // A n'a rien que B n'ait : B n'écrit rien, et la panne tombe sur une réception d'A.
    {
        let _panne = couper(Etape::Renommage, 1);
        reunir_en_suivant_a(&mut a, &mut ea, &mut b, &mut eb, |a, _, rendu| {
            match rendu {
                Err(Erreur::Disque { .. }) => refusees += 1,
                Ok(None) => {}
                autre => panic!("attendu Disque ou une réception faite, rendu {autre:?}"),
            }
            assert_eq!(
                a.lecture_seule(),
                Some(&raison),
                "après une réception, A n'est plus en lecture seule"
            );
        });
    }

    assert_eq!(refusees, 1, "la réception d'A n'a pas été refusée");
    assert!(
        porte_l_idee(a.automerge(), "idee-2"),
        "A ne porte pas l'idée reçue"
    );
    match ecrire(&mut a, |tx| noter(tx, "idee-3", "Une phrase refusée")) {
        Err(Erreur::LectureSeule(rendue)) => assert_eq!(
            rendue, raison,
            "l'écriture ne rend pas la raison de la lecture seule"
        ),
        autre => panic!("attendu LectureSeule à l'écriture, rendu {autre:?}"),
    }
}
