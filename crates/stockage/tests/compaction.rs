//! La compaction (§ 31 · 2.2) : elle fond les fichiers d'ajout dans un fichier complet neuf, le
//! relit, puis retire les seuls fichiers d'ajout lisibles et absorbés. Elle a lieu à
//! l'ouverture, puis dès que les ajouts pèsent plus que le fichier complet, ou qu'elle est due.
//! Coupée à chaque étape, elle ne perd rien ; à l'ouverture, son échec vient en avertissement.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;
use std::io;
use std::path::Path;

use automerge::{Automerge, Change};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_des_ajouts, dossier_du_fonds, faire_naitre, faite,
    lire_le_complet, ouvrir, ouvrir_le_fonds, ouvrir_le_fonds_tel_quel, poids, porte_l_idee,
    texte_long,
};
use noyau::donnees::fonds::{GENESE, MESSAGE_DE_LA_GENESE, MESSAGE_DE_LA_NAISSANCE};
use stockage::pannes::{Etape, alterer_la_relecture, couper};
use stockage::{Avertissement, Document, Erreur, LectureSeule, RacineDesDonnees, teneur};

#[test]
fn rouvrir_compacte_les_ajouts() {
    let chemin = commun::dossier_jetable("compaction-rouvrir").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    faite(
        fonds.ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        }),
        "écrire",
    );
    let heads = fonds.automerge().get_heads();
    let valeur = fonds.automerge().hydrate(None);
    let ajouts = dossier_des_ajouts(&chemin);
    assert_eq!(
        commun::entrees(&ajouts).len(),
        1,
        "l'écriture n'a pas laissé son fichier d'ajout"
    );
    // La racine des données fermée, puis rouverte, comme au lancement suivant.
    drop(fonds);
    drop(racine);

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "rouvrir n'a pas compacté les fichiers d'ajout"
    );
    assert_eq!(fonds.automerge().get_heads(), heads);
    assert_eq!(fonds.automerge().hydrate(None), valeur);
    let seul = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    assert_eq!(
        seul.get_heads(),
        heads,
        "le fichier complet lu seul ne mène pas aux heads d'avant"
    );
    assert_eq!(
        seul.hydrate(None),
        valeur,
        "le fichier complet lu seul ne donne pas la valeur d'avant"
    );
}

#[test]
fn les_ajouts_ne_pesent_jamais_plus_que_le_complet() {
    let chemin = commun::dossier_jetable("compaction-poids").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let complet = dossier_du_fonds(&chemin).join("complet.automerge");
    let ajouts = dossier_des_ajouts(&chemin);
    // Une compaction après l'écriture retire tous les fichiers d'ajout, le sien compris : sans
    // elle, celui de l'écriture reste.
    let mut compactions = 0;

    for n in 1..=120_i64 {
        faite(
            fonds.ecrire("Étiquette posée", HEURE + n, |tx| {
                commun::etiqueter(tx, "idee-1", &format!("etiquette-{n}"))
            }),
            &format!("l'écriture {n}"),
        );

        let noms = commun::entrees(&ajouts);
        if noms.is_empty() {
            compactions += 1;
        }
        let des_ajouts: u64 = noms.iter().map(|nom| poids(&ajouts.join(nom))).sum();
        let du_complet = poids(&complet);
        assert!(
            des_ajouts <= du_complet,
            "après l'écriture {n}, les ajouts pèsent {des_ajouts} octets, le fichier complet {du_complet}"
        );
        assert!(
            noms.len() <= 100,
            "après l'écriture {n}, {} fichiers d'ajout",
            noms.len()
        );
    }
    assert!(
        compactions >= 2,
        "le fichier complet n'a été remplacé que {compactions} fois"
    );
    drop(fonds);
    drop(racine);

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let fonds = ouvrir_le_fonds(&racine);
    assert_eq!(
        fonds.automerge().get_changes(&[]).len(),
        123,
        "la genèse, la naissance, le premier geste et cent vingt écritures"
    );
}

#[test]
fn la_compaction_garde_toute_l_histoire() {
    let d = commun::dossier_jetable("compaction-histoire");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    faite(
        fonds.ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        }),
        "la première écriture",
    );
    faite(
        fonds.ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        }),
        "la deuxième écriture",
    );
    faite(
        fonds.ecrire("Texte changé", HEURE + 3, |tx| {
            commun::changer_le_texte(tx, "idee-2", "Un pont plus court")
        }),
        "la troisième écriture",
    );
    let ajouts = dossier_des_ajouts(racine.chemin());
    assert_eq!(
        commun::entrees(&ajouts).len(),
        3,
        "trois écritures, trois fichiers d'ajout, qu'aucune compaction n'a encore fondus"
    );

    fonds.compacter().expect("compacter");

    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "la compaction a laissé un fichier d'ajout absorbé"
    );
    let seul = Automerge::load(&lire_le_complet(racine.chemin())).expect("lire le fichier complet");
    assert_eq!(
        seul.get_heads(),
        fonds.automerge().get_heads(),
        "le fichier complet lu seul ne mène pas aux heads du fonds"
    );
    let modifications = seul.get_changes(&[]);
    let messages: Vec<Option<&str>> = modifications.iter().map(Change::message).collect();
    assert_eq!(
        messages,
        [
            Some(MESSAGE_DE_LA_GENESE),
            Some(MESSAGE_DE_LA_NAISSANCE),
            Some(MESSAGE_DU_GESTE),
            Some("Étiquette posée"),
            Some("Idée notée"),
            Some("Texte changé"),
        ]
    );
    let genese = Change::try_from(GENESE).expect("lire la genèse");
    assert_eq!(
        modifications[0].hash(),
        genese.hash(),
        "la première modification du fichier complet n'est pas la genèse"
    );
}

// § 31 · 10, test 9 : un fichier d'ajout altéré, puis compacter
#[test]
fn la_compaction_garde_un_ajout_altere() {
    let d = commun::dossier_jetable("compaction-altere");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(racine.chemin());
    faite(
        fonds.ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        }),
        "la première écriture",
    );
    let premier = commun::entrees(&ajouts);
    assert_eq!(
        premier.len(),
        1,
        "une écriture, un fichier d'ajout : {premier:?}"
    );
    faite(
        fonds.ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        }),
        "la seconde écriture",
    );
    let noms = commun::entrees(&ajouts);
    assert_eq!(
        noms.len(),
        2,
        "deux écritures, deux fichiers d'ajout : {noms:?}"
    );
    // Un octet changé, au milieu du fichier d'ajout de la première écriture.
    let altere = ajouts.join(&premier[0]);
    let octets = commun::alterer_l_octet_du_milieu(&altere);

    fonds.compacter().expect("compacter");

    assert_eq!(
        commun::entrees(&ajouts),
        premier,
        "seul le fichier altéré reste : celui de la seconde écriture est retiré"
    );
    assert!(
        fs::read(&altere).expect("relire le fichier altéré") == octets,
        "le fichier altéré n'est plus tel qu'il était"
    );
    drop(fonds);
    let dossier = dossier_du_fonds(racine.chemin());
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::AjoutAltere {
        chemin: altere.clone(),
    };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fonds rouvert ne nomme pas le fichier d'ajout altéré"
    );
    commun::constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
    assert!(
        fs::read(&altere).expect("relire le fichier altéré") == octets,
        "le fichier altéré n'est plus là, octet pour octet"
    );
}

#[test]
fn une_compaction_qui_echoue_apres_une_ecriture_ne_la_defait_pas() {
    let chemin = commun::dossier_jetable("compaction-apres-une-ecriture").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = racine
        .faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
        .expect("faire naître le fonds");
    let ajouts = dossier_des_ajouts(&chemin);

    let rendu = {
        let _panne = couper(Etape::Save, 1);
        commun::ecrire(&mut fonds, |tx| commun::noter(tx, "idee-2", &texte_long()))
    };

    assert!(
        matches!(
            rendu,
            Ok(Some(Avertissement::Compaction(Erreur::CompactionInfidele)))
        ),
        "attendu l'écriture faite, avec la compaction infidèle en avertissement ; rendu {rendu:?}"
    );
    let noms = commun::entrees(&ajouts);
    assert_eq!(
        noms.len(),
        1,
        "l'écriture n'a pas laissé son seul fichier d'ajout : {noms:?}"
    );
    let du_complet = poids(&dossier_du_fonds(&chemin).join("complet.automerge"));
    let de_l_ajout = poids(&ajouts.join(&noms[0]));
    assert!(
        de_l_ajout > du_complet,
        "le fichier d'ajout pèse {de_l_ajout} octets, le fichier complet {du_complet} : rien ne \
         demandait de compacter"
    );

    let suivante = commun::ecrire(&mut fonds, |tx| commun::etiqueter(tx, "idee-1", "refrain"));
    assert!(
        matches!(suivante, Ok(None)),
        "attendu l'écriture suivante faite ; rendu {suivante:?}"
    );
    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "la compaction retentée a laissé un fichier d'ajout"
    );
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    for idee in ["idee-1", "idee-2"] {
        assert!(
            porte_l_idee(fonds.automerge(), idee),
            "le fonds rouvert ne porte pas {idee}"
        );
    }
}

#[test]
fn une_compaction_infidele_n_ecrit_rien() {
    let chemin = commun::dossier_jetable("compaction-infidele").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);
    ecritures_courtes(&mut fonds, &ajouts, 2);
    let complet_avant = lire_le_complet(&chemin);

    let rendu = {
        let _panne = couper(Etape::Save, 1);
        fonds.compacter()
    };

    assert!(
        matches!(rendu, Err(Erreur::CompactionInfidele)),
        "attendu CompactionInfidele, rendu {rendu:?}"
    );
    assert_eq!(
        commun::entrees(&dossier_du_fonds(&chemin)),
        ["ajouts", "complet.automerge"],
        "la compaction infidèle a laissé un fichier à côté du fichier complet"
    );
    assert!(
        lire_le_complet(&chemin) == complet_avant,
        "la compaction infidèle a écrit le fichier complet"
    );
    assert_eq!(
        commun::entrees(&ajouts).len(),
        2,
        "la compaction infidèle a retiré un fichier d'ajout"
    );
}

#[test]
fn une_compaction_dont_le_complet_se_relit_autre_ne_retire_rien() {
    let chemin = commun::dossier_jetable("compaction-relue-autre").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);
    ecritures_courtes(&mut fonds, &ajouts, 2);
    let complet_avant = lire_le_complet(&chemin);
    let ajouts_avant = commun::entrees(&ajouts);

    let rendu = {
        // La première relecture de `compacter` est celle du voisin du fichier complet.
        let _panne = alterer_la_relecture(1);
        fonds.compacter()
    };

    match rendu {
        Err(Erreur::Disque { erreur, .. }) => assert_eq!(
            erreur.kind(),
            io::ErrorKind::InvalidData,
            "le voisin relu autre ne rend pas InvalidData"
        ),
        autre => panic!("attendu Disque, rendu {autre:?}"),
    }
    assert_eq!(
        commun::entrees(&dossier_du_fonds(&chemin)),
        ["ajouts", "complet.automerge"],
        "le voisin relu autre est resté à côté du fichier complet"
    );
    assert!(
        lire_le_complet(&chemin) == complet_avant,
        "le voisin relu autre a pris la place du fichier complet"
    );
    assert_eq!(
        commun::entrees(&ajouts),
        ajouts_avant,
        "la compaction a retiré un fichier d'ajout"
    );
}

// § 31 · 10, test 9 : compacter en s'interrompant à chaque étape, puis rouvrir, et revenir avant chaque opération
#[test]
fn une_compaction_coupee_a_chaque_etape_ne_perd_rien() {
    for etape in [
        Etape::Save,
        Etape::VoisinEcrit,
        Etape::SyncAll,
        Etape::Relecture,
        Etape::Renommage,
        Etape::RendreDurable,
        Etape::Retrait,
    ] {
        let chemin =
            commun::dossier_jetable(&format!("compaction-coupee-{etape:?}")).join("donnees");
        let racine = ouvrir(&chemin);
        let mut fonds = faire_naitre(&racine);
        let dossier = dossier_du_fonds(&chemin);
        let ajouts = dossier.join("ajouts");
        ecritures_courtes(&mut fonds, &ajouts, 3);
        let complet_avant = lire_le_complet(&chemin);
        let memoire = fonds.automerge().clone();

        let rendu = {
            let _panne = couper(etape, 1);
            fonds.compacter()
        };

        assert!(rendu.is_err(), "{etape:?} : la compaction coupée a réussi");
        assert_eq!(
            commun::entrees(&dossier),
            ["ajouts", "complet.automerge"],
            "{etape:?} : un voisin reste à côté du fichier complet"
        );
        // Avant la fin du renommage, le fichier complet est celui d'avant ; après, il porte déjà
        // toute la mémoire.
        if matches!(etape, Etape::RendreDurable | Etape::Retrait) {
            let seul = Automerge::load(&lire_le_complet(&chemin)).unwrap_or_else(|e| {
                panic!("{etape:?} : lire le fichier complet de la compaction coupée : {e}")
            });
            assert_eq!(
                seul.get_heads(),
                memoire.get_heads(),
                "{etape:?} : le fichier complet en place ne mène pas aux heads de la mémoire"
            );
        } else {
            assert!(
                lire_le_complet(&chemin) == complet_avant,
                "{etape:?} : le fichier complet n'est plus celui d'avant"
            );
        }
        if etape == Etape::Retrait {
            assert!(
                !commun::entrees(&ajouts).is_empty(),
                "Retrait : la compaction coupée a retiré tous les fichiers d'ajout"
            );
        }
        drop(fonds);

        let rouvert = ouvrir_le_fonds(&racine);
        let doc = rouvert.automerge();
        assert_eq!(
            rouvert.lecture_seule(),
            None,
            "{etape:?} : le fonds rouvert est en lecture seule"
        );
        // La compaction de l'ouverture, sans panne, réussit : aucun avertissement.
        assert!(
            rouvert.avertissements().is_empty(),
            "{etape:?} : le fonds rouvert a des avertissements : {:?}",
            rouvert.avertissements()
        );
        assert_eq!(
            doc.get_heads(),
            memoire.get_heads(),
            "{etape:?} : le fonds rouvert n'a pas les heads d'avant"
        );
        assert_eq!(
            doc.hydrate(None),
            memoire.hydrate(None),
            "{etape:?} : le fonds rouvert n'a pas la valeur d'avant"
        );
        // § 31 · 10, test 9 : la teneur ne change pas
        assert_eq!(
            teneur(doc).expect("la teneur du fonds rouvert"),
            teneur(&memoire).expect("la teneur de la mémoire d'avant"),
            "{etape:?} : la teneur du fonds rouvert n'est pas celle d'avant"
        );
        for modification in doc.get_changes(&[]) {
            let hash = modification.hash();
            let rouvert_la = doc.fork_at(&[hash]).unwrap_or_else(|e| {
                panic!("{etape:?} : revenir à {hash:?} dans le fonds rouvert : {e}")
            });
            let avant_la = memoire.fork_at(&[hash]).unwrap_or_else(|e| {
                panic!("{etape:?} : revenir à {hash:?} dans la mémoire d'avant : {e}")
            });
            assert_eq!(
                rouvert_la.hydrate(None),
                avant_la.hydrate(None),
                "{etape:?} : revenu à {hash:?}, le fonds rouvert et la mémoire d'avant diffèrent"
            );
        }
        if etape == Etape::Retrait {
            // L'ouverture a compacté le fonds, sans panne : c'est la compaction qui suit celle
            // qui a été coupée.
            assert_eq!(
                commun::entrees(&ajouts),
                Vec::<String>::new(),
                "Retrait : la compaction suivante, sans panne, a laissé un fichier d'ajout"
            );
        }
    }
}

#[test]
fn une_compaction_d_ouverture_infidele_ouvre_avec_un_avertissement() {
    une_compaction_d_ouverture_qui_echoue(
        "compaction-ouverture-infidele",
        Etape::Save,
        |avertissements| {
            matches!(
                avertissements,
                [Avertissement::Compaction(Erreur::CompactionInfidele)]
            )
        },
    );
}

#[test]
fn une_compaction_d_ouverture_refusee_par_le_disque_ouvre_avec_un_avertissement() {
    une_compaction_d_ouverture_qui_echoue(
        "compaction-ouverture-refusee",
        Etape::Renommage,
        |avertissements| {
            matches!(
                avertissements,
                [Avertissement::Compaction(Erreur::Disque { .. })]
            )
        },
    );
}

/// Écrit dans `fonds` les `n` premières des trois écritures courtes des tests : une étiquette
/// posée, une idée notée, puis son texte changé. Chacune est faite sans avertissement, et laisse
/// son fichier d'ajout dans `ajouts`, qu'aucune compaction ne fond encore.
fn ecritures_courtes(fonds: &mut Document<'_>, ajouts: &Path, n: i64) {
    for rang in 1..=n {
        let rendu = match rang {
            1 => commun::ecrire_a(fonds, "Étiquette posée", HEURE + rang, |tx| {
                commun::etiqueter(tx, "idee-1", "refrain")
            }),
            2 => commun::ecrire_a(fonds, "Idée notée", HEURE + rang, |tx| {
                commun::noter(tx, "idee-2", "Un pont en mineur")
            }),
            _ => commun::ecrire_a(fonds, "Texte changé", HEURE + rang, |tx| {
                commun::changer_le_texte(tx, "idee-2", "Un pont plus court")
            }),
        };
        assert!(
            matches!(rendu, Ok(None)),
            "l'écriture {rang} n'est pas faite sans avertissement : {rendu:?}"
        );
    }
    let noms = commun::entrees(ajouts);
    assert_eq!(
        noms.len() as i64,
        n,
        "{n} écritures courtes, autant de fichiers d'ajout, qu'aucune compaction n'a fondus : \
         {noms:?}"
    );
}

/// Une écriture courte, le fonds lâché, puis rouvert sous `couper(etape, 1)` : l'ouverture rend
/// le fonds entier, ses avertissements reconnus par `attendu`, et les fichiers restent en place.
/// La panne levée, une écriture est faite, sans avertissement, et laisse `ajouts/` vide : la
/// compaction due l'a suivie, que rien d'autre ne demandait, les deux fichiers d'ajout pesant
/// ensemble moins que le fichier complet.
fn une_compaction_d_ouverture_qui_echoue(
    nom: &str,
    etape: Etape,
    attendu: impl Fn(&[Avertissement]) -> bool,
) {
    let chemin = commun::dossier_jetable(nom).join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let dossier = dossier_du_fonds(&chemin);
    let ajouts = dossier.join("ajouts");
    ecritures_courtes(&mut fonds, &ajouts, 1);
    drop(fonds);
    let complet_avant = lire_le_complet(&chemin);
    let noms = commun::entrees(&ajouts);
    let ajout = ajouts.join(&noms[0]);
    let ajout_avant = fs::read(&ajout).expect("lire le fichier d'ajout");
    let du_complet = poids(&dossier.join("complet.automerge"));
    let de_l_ajout = poids(&ajout);
    assert!(
        de_l_ajout < du_complet,
        "{etape:?} : le fichier d'ajout pèse {de_l_ajout} octets, le fichier complet {du_complet}"
    );

    let ouvert = {
        let _panne = couper(etape, 1);
        racine.fonds()
    };

    let mut fonds = ouvert
        .expect("une compaction d'ouverture qui échoue n'empêche pas d'ouvrir")
        .expect("le fonds est né");
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "{etape:?} : le fonds ouvert est en lecture seule"
    );
    assert!(
        attendu(fonds.avertissements()),
        "{etape:?} : avertissements rendus {:?}",
        fonds.avertissements()
    );
    assert_eq!(
        commun::entrees(&dossier),
        ["ajouts", "complet.automerge"],
        "{etape:?} : un voisin reste à côté du fichier complet"
    );
    assert!(
        lire_le_complet(&chemin) == complet_avant,
        "{etape:?} : le fichier complet n'est plus celui d'avant"
    );
    assert_eq!(
        commun::entrees(&ajouts),
        noms,
        "{etape:?} : le fichier d'ajout n'est plus en place"
    );
    assert!(
        fs::read(&ajout).expect("relire le fichier d'ajout") == ajout_avant,
        "{etape:?} : le fichier d'ajout n'est plus tel qu'il était"
    );

    let heads_avant = fonds.automerge().get_heads();
    let rendu = commun::ecrire(&mut fonds, |tx| commun::etiqueter(tx, "idee-1", "couplet"));
    assert!(
        matches!(rendu, Ok(None)),
        "{etape:?} : attendu l'écriture faite, la panne levée ; rendu {rendu:?}"
    );
    // Ce que l'écriture a écrit, et que la compaction a retiré.
    let du_second = fonds.automerge().save_after(&heads_avant).len() as u64;
    assert!(
        de_l_ajout + du_second <= du_complet,
        "{etape:?} : les fichiers d'ajout pèsent {} octets, le fichier complet {du_complet} : la \
         compaction due n'était pas seule à la demander",
        de_l_ajout + du_second
    );
    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "{etape:?} : la compaction due n'a pas suivi l'écriture"
    );
}
