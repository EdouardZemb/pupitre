//! Les fichiers d'ajout (§ 31 · 2.1) : chaque écriture du fonds ajoute un fichier, nommé par
//! l'empreinte de son contenu, que l'ouverture relit après le fichier complet. Un fichier d'ajout
//! altéré, manquant ou qu'Automerge refuse, comme un fichier complet illisible, ouvre le document
//! en lecture seule, en se nommant, et une écriture refusée relit le document du disque.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;
use std::path::Path;

use automerge::transaction::CommitOptions;
use automerge::{ActorId, Automerge, AutomergeError, LoadOptions, OnPartialLoad, ReadDoc};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, alterer_l_octet_du_milieu, constater_la_lecture_seule,
    deux_copies, dossier_des_ajouts, dossier_du_fonds, faire_naitre, faite, lire_le_complet,
    ouvrir, ouvrir_le_fonds, ouvrir_le_fonds_tel_quel, poids, porte_l_idee, texte_long,
};
use noyau::donnees::compaction::AJOUTS_AU_PLUS;
use noyau::donnees::fonds::{IDENTIFIANT, LIGNEE, PROPRIETAIRE};
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Document, Erreur, LectureSeule, RacineDesDonnees, empreinte};

#[test]
fn chaque_ecriture_ajoute_un_fichier_nomme_par_son_empreinte() {
    let d = commun::dossier_jetable("ajouts-un-par-ecriture");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let avant = fonds.automerge().get_heads();

    faite(
        fonds.ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        }),
        "écrire",
    );

    let ajouts = dossier_des_ajouts(racine.chemin());
    let noms = commun::entrees(&ajouts);
    assert_eq!(noms.len(), 1, "une écriture, un fichier d'ajout : {noms:?}");
    let ajout = fs::read(ajouts.join(&noms[0])).expect("lire le fichier d'ajout");
    assert_eq!(
        noms[0],
        format!("{}.automerge", empreinte(&ajout)),
        "le nom du fichier d'ajout n'est pas l'empreinte de son contenu"
    );
    let complet = lire_le_complet(racine.chemin());
    assert!(
        2 * ajout.len() < complet.len(),
        "le fichier d'ajout pèse {} octets, le fichier complet {}",
        ajout.len(),
        complet.len()
    );

    let modifications = fonds.automerge().get_changes(&avant);
    assert_eq!(modifications.len(), 1, "une écriture, une modification");
    assert_eq!(modifications[0].message(), Some("Étiquette posée"));
    assert_eq!(modifications[0].timestamp(), HEURE + 1);

    let mut relu = Automerge::load(&complet).expect("lire le fichier complet");
    assert_eq!(
        relu.get_heads(),
        avant,
        "le fichier complet porte déjà l'écriture"
    );
    relu.load_incremental(&ajout)
        .expect("lire le fichier d'ajout");
    assert_eq!(
        relu.get_heads(),
        fonds.automerge().get_heads(),
        "le fichier complet, puis le fichier d'ajout, ne mènent pas aux heads de l'écriture"
    );
}

#[test]
fn rouvrir_relit_le_complet_puis_les_ajouts() {
    let chemin = commun::dossier_jetable("ajouts-rouvrir").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
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
        "la seconde écriture",
    );
    let heads = fonds.automerge().get_heads();
    let valeur = fonds.automerge().hydrate(None);
    // Le voisin d'une écriture qu'un processus n'a pas finie : son contenu ne donne pas
    // l'empreinte de son nom, et le démarrage le retire.
    let partiel =
        dossier_des_ajouts(&chemin).join(format!("{}.automerge.4242.partiel", "0".repeat(64)));
    fs::write(&partiel, b"une ecriture interrompue").expect("déposer le .partiel");
    // La racine des données fermée, puis rouverte, comme au lancement suivant.
    drop(fonds);
    drop(racine);

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let fonds = ouvrir_le_fonds(&racine);

    assert_eq!(fonds.automerge().get_heads(), heads);
    assert_eq!(fonds.automerge().hydrate(None), valeur);
    assert!(
        !partiel.exists(),
        "le .partiel non conforme déposé dans ajouts/ reste après la réouverture"
    );
}

#[test]
fn une_ecriture_sans_modification_n_ecrit_rien() {
    let d = commun::dossier_jetable("ajouts-sans-modification");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let avant = fonds.automerge().get_heads();

    // Ôter une étiquette que l'idée ne porte pas ne modifie rien.
    faite(
        fonds.ecrire("Étiquette ôtée", HEURE + 1, |tx| {
            commun::oter_l_etiquette(tx, "idee-1", "refrain")
        }),
        "écrire",
    );

    assert_eq!(
        commun::entrees(&dossier_des_ajouts(racine.chemin())),
        Vec::<String>::new(),
        "une écriture sans modification a écrit un fichier"
    );
    assert_eq!(fonds.automerge().get_heads(), avant);
}

#[test]
fn un_geste_refuse_n_ecrit_rien_et_le_document_reste_ouvert() {
    let d = commun::dossier_jetable("ajouts-geste-refuse");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let avant = fonds.automerge().get_heads();
    let valeur = fonds.automerge().hydrate(None);

    // Le geste écrit, puis refuse : rien de ce qu'il a écrit ne reste.
    let refus = fonds.ecrire("Idée notée", HEURE + 1, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")?;
        Err(AutomergeError::Fail)
    });

    match refus {
        Err(Erreur::Automerge(_)) => {}
        Err(autre) => panic!("attendu le refus du geste, rendu {autre:?}"),
        Ok(_) => panic!("un geste refusé s'est écrit"),
    }
    let ajouts = dossier_des_ajouts(racine.chemin());
    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "ni fichier d'ajout ni .partiel : rien ne s'écrit"
    );
    assert_eq!(fonds.automerge().get_heads(), avant);
    assert_eq!(
        fonds.automerge().hydrate(None),
        valeur,
        "ce que le geste refusé a écrit reste en mémoire"
    );

    faite(
        fonds.ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        }),
        "l'écriture suivante",
    );
    assert_eq!(
        commun::entrees(&ajouts).len(),
        1,
        "l'écriture suivante n'a pas écrit son fichier d'ajout"
    );
}

#[test]
fn un_ajout_manquant_ouvre_en_lecture_seule() {
    let chemin = commun::dossier_jetable("ajouts-manquant").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let heads_de_la_naissance = fonds.automerge().get_heads();
    let valeur_de_la_naissance = fonds.automerge().hydrate(None);
    let ajouts = dossier_des_ajouts(&chemin);
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
    let second = commun::nouveaux(&premier, &commun::entrees(&ajouts));
    assert_eq!(
        second.len(),
        1,
        "la seconde écriture n'a pas laissé un seul fichier d'ajout de plus : {second:?}"
    );
    drop(fonds);
    // La seconde modification dépend de la première, dont le fichier d'ajout manque.
    fs::remove_file(ajouts.join(&premier[0])).expect("retirer le premier fichier d'ajout");
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    // Le fichier qui manque ne se nomme pas : celui qui l'attend, si.
    let raison = LectureSeule::DependancesManquantes {
        chemin: ajouts.join(&second[0]),
    };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fonds ne nomme pas le fichier d'ajout qui attend la modification manquante"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_de_la_naissance,
        "la mémoire ne mène pas aux heads de la naissance"
    );
    assert_eq!(
        fonds.automerge().hydrate(None),
        valeur_de_la_naissance,
        "la mémoire ne porte pas la naissance seule, sans l'étiquette ni l'idée écrites"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "la mémoire porte l'idée dont la modification attend ses dépendances"
    );
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

#[test]
fn chaque_ouverture_tire_un_acteur_neuf() {
    let chemin = commun::dossier_jetable("ajouts-acteur-neuf").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    faite(
        fonds.ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        }),
        "écrire avant de rouvrir",
    );
    drop(fonds);
    drop(racine);

    let racine = RacineDesDonnees::ouvrir(&chemin).expect("rouvrir la racine des données");
    let mut fonds = ouvrir_le_fonds(&racine);
    let avant = fonds.automerge().get_heads();
    assert_eq!(
        avant.len(),
        1,
        "le fonds rouvert a une seule tête : {avant:?}"
    );
    // Rouvert, le fonds s'est compacté : `ajouts/` se relit juste avant l'écriture.
    let ajouts = dossier_des_ajouts(&chemin);
    let ajouts_avant = commun::entrees(&ajouts);
    assert!(
        ajouts_avant.is_empty(),
        "la réouverture compacte : ajouts/ est vide, aucun fichier parasite : {ajouts_avant:?}"
    );
    faite(
        fonds.ecrire("Étiquette posée", HEURE + 2, |tx| {
            commun::etiqueter(tx, "idee-1", "couplet")
        }),
        "écrire après avoir rouvert",
    );

    let doc = fonds.automerge();
    let premiere = doc
        .get_change_by_hash(&avant[0])
        .expect("la modification de la première écriture");
    let seconde = doc.get_changes(&avant);
    assert_eq!(seconde.len(), 1, "une écriture, une modification");
    assert_ne!(
        premiere.actor_id(),
        seconde[0].actor_id(),
        "l'écriture d'avant et celle d'après la réouverture ont le même acteur"
    );

    let ajout = commun::constater_un_ajout(
        doc,
        &avant,
        &ajouts,
        &ajouts_avant,
        &commun::entrees(&ajouts),
    );
    let complet = lire_le_complet(&chemin);
    assert!(
        4 * ajout.len() < complet.len(),
        "le fichier d'ajout pèse {} octets, le fichier complet {}",
        ajout.len(),
        complet.len()
    );
}

#[test]
fn un_ajout_altere_ouvre_en_lecture_seule_et_se_nomme() {
    let chemin = commun::dossier_jetable("ajouts-altere").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);
    faite(
        fonds.ecrire("Idée notée", HEURE + 1, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
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
            commun::noter(tx, "idee-3", "Une coda en suspens")
        }),
        "la seconde écriture",
    );
    let second = commun::nouveaux(&premier, &commun::entrees(&ajouts));
    assert_eq!(
        second.len(),
        1,
        "la seconde écriture n'a pas laissé un seul fichier d'ajout de plus : {second:?}"
    );
    drop(fonds);
    // Un octet changé, au milieu du fichier d'ajout de la seconde écriture.
    let altere = ajouts.join(&second[0]);
    alterer_l_octet_du_milieu(&altere);
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::AjoutAltere { chemin: altere };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fonds ne nomme pas le fichier d'ajout altéré"
    );
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "l'idée de la première écriture ne se lit pas"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-3"),
        "l'idée du fichier d'ajout altéré se lit"
    );
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

#[test]
fn un_ajout_qu_automerge_refuse_ouvre_en_lecture_seule() {
    let chemin = commun::dossier_jetable("ajouts-refuse-par-automerge").join("donnees");
    let racine = ouvrir(&chemin);
    drop(faire_naitre(&racine));
    let complet = Automerge::load(&lire_le_complet(&chemin)).expect("lire le fichier complet");
    let h = complet.get_heads();
    let ajouts = dossier_des_ajouts(&chemin);
    // Deux modifications du même acteur au même rang, chacune dans son fichier d'ajout, nommé
    // par l'empreinte de son contenu : l'une se charge, l'autre plus.
    let mut ecrits = Vec::new();
    for (idee, texte) in [
        ("idee-2", "Un pont en mineur"),
        ("idee-3", "Une coda en suspens"),
    ] {
        let mut double = complet.fork();
        double.set_actor(ActorId::from(b"acteur-en-double".as_slice()));
        let mut tx = double.transaction();
        commun::noter(&mut tx, idee, texte).expect("noter l'idée");
        tx.commit_with(
            CommitOptions::default()
                .with_message(MESSAGE_DU_GESTE)
                .with_time(HEURE + 1),
        );
        let octets = double.save_after(&h);
        let nom = format!("{}.automerge", empreinte(&octets));
        fs::write(ajouts.join(&nom), &octets).expect("écrire le fichier d'ajout");
        ecrits.push((nom, idee));
    }
    ecrits.sort();
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let (premier, idee_du_premier) = &ecrits[0];
    let (second, idee_du_second) = &ecrits[1];
    let raison = LectureSeule::AjoutIllisible {
        chemin: ajouts.join(second),
    };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fonds ne nomme pas le second fichier d'ajout, dans l'ordre des noms ({premier} puis \
         {second})"
    );
    assert!(
        porte_l_idee(fonds.automerge(), idee_du_premier),
        "l'idée du premier fichier d'ajout ne se lit pas"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), idee_du_second),
        "l'idée du fichier d'ajout qu'Automerge refuse se lit"
    );
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

#[test]
fn un_complet_illisible_ouvre_en_lecture_seule_et_se_nomme() {
    // Trois fichiers complets illisibles : un octet changé au milieu, que la somme de contrôle de
    // son tronçon refuse ; vide, qu'Automerge lirait comme un document neuf, sans genèse ; entier,
    // mais suivi d'octets en trop, que seul `OnPartialLoad::Ignore` relit.
    for alteration in ["octet-du-milieu", "vide", "octets-en-trop"] {
        let chemin = commun::dossier_jetable(&format!("ajouts-complet-illisible-{alteration}"))
            .join("donnees");
        let racine = ouvrir(&chemin);
        let mut fonds = faire_naitre(&racine);
        let rendu = commun::ecrire_a(&mut fonds, MESSAGE_DU_GESTE, HEURE + 1, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        });
        assert!(
            matches!(rendu, Ok(None)),
            "{alteration} : attendu l'écriture faite, sans avertissement ; rendu {rendu:?}"
        );
        let heads_ecrites = fonds.automerge().get_heads();
        let valeur_ecrite = fonds.automerge().hydrate(None);
        drop(fonds);
        let dossier = dossier_du_fonds(&chemin);
        let complet = dossier.join("complet.automerge");
        let altere = match alteration {
            "octet-du-milieu" => alterer_l_octet_du_milieu(&complet),
            "vide" => {
                fs::write(&complet, b"").expect("vider le fichier complet");
                Vec::new()
            }
            _ => {
                let mut octets = lire_le_complet(&chemin);
                octets.extend_from_slice(b"des octets en trop");
                fs::write(&complet, &octets).expect("allonger le fichier complet");
                octets
            }
        };
        // Ce qui se lit, calculé à part : ce que `load_with_options` relit des octets altérés, ou
        // un document neuf s'il les refuse, puis chaque fichier d'ajout.
        let mut lisible = Automerge::load_with_options(
            &altere,
            LoadOptions::new().on_partial_load(OnPartialLoad::Ignore),
        )
        .unwrap_or_else(|_| Automerge::new());
        let ajouts = dossier_des_ajouts(&chemin);
        let noms = commun::entrees(&ajouts);
        assert_eq!(
            noms.len(),
            1,
            "{alteration} : une écriture, un fichier d'ajout : {noms:?}"
        );
        for nom in &noms {
            let octets = fs::read(ajouts.join(nom)).expect("lire le fichier d'ajout");
            // Un fichier d'ajout qu'Automerge refuse ne se charge pas.
            let _ = lisible.load_incremental(&octets);
        }
        let avant = commun::empreintes(&dossier);

        let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

        let raison = LectureSeule::CompletIllisible { chemin: complet };
        assert_eq!(
            fonds.lecture_seule(),
            Some(&raison),
            "{alteration} : le fonds ne nomme pas le fichier complet illisible"
        );
        assert_eq!(
            fonds.automerge().get_heads(),
            lisible.get_heads(),
            "{alteration} : la mémoire ne mène pas aux heads de ce qui se lit"
        );
        assert_eq!(
            fonds.automerge().hydrate(None),
            lisible.hydrate(None),
            "{alteration} : la mémoire ne montre pas ce qui se lit"
        );
        match alteration {
            "vide" => assert!(
                fonds.automerge().get_heads().is_empty(),
                "vide : la mémoire n'est pas un document neuf : {:?}",
                fonds.automerge().get_heads()
            ),
            "octets-en-trop" => {
                assert_eq!(
                    fonds.automerge().get_heads(),
                    heads_ecrites,
                    "octets-en-trop : la mémoire ne porte pas le fichier complet, puis le fichier \
                     d'ajout"
                );
                assert_eq!(
                    fonds.automerge().hydrate(None),
                    valeur_ecrite,
                    "octets-en-trop : la mémoire ne montre pas ce qu'avait écrit le fonds"
                );
            }
            _ => {}
        }
        constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
    }
}

#[test]
fn une_chaine_d_ajouts_nomme_celui_qui_attend_le_manquant() {
    // La variante retenue est la première où un fichier d'ajout venu après le deuxième passe
    // avant lui dans l'ordre des noms : il attend une modification présente, mais en attente,
    // et ne doit pas se nommer à la place du deuxième, qui attend celle qui manque.
    let (variante, complet, contenus, noms) = (0..16)
        .find_map(|variante| {
            let (complet, contenus) = une_chaine_d_ajouts(variante);
            let noms: Vec<String> = contenus
                .iter()
                .map(|contenu| format!("{}.automerge", empreinte(contenu)))
                .collect();
            let premisse = noms[2..].iter().any(|nom| nom < &noms[1]);
            premisse.then_some((variante, complet, contenus, noms))
        })
        .expect(
            "aucune variante ne range un fichier d'ajout venu après le deuxième avant lui : en \
             ajouter",
        );
    let chemin = commun::dossier_jetable("ajouts-chaine").join("donnees");
    let racine = ouvrir(&chemin);
    let dossier = dossier_du_fonds(&chemin);
    let ajouts = dossier.join("ajouts");
    fs::create_dir_all(&ajouts).expect("créer ajouts/");
    fs::write(dossier.join("complet.automerge"), &complet).expect("écrire le fichier complet");
    for (nom, contenu) in noms.iter().zip(&contenus) {
        fs::write(ajouts.join(nom), contenu).expect("écrire un fichier d'ajout");
    }
    // Le fichier d'ajout de la première écriture manque : la deuxième l'attend, et chacune des
    // suivantes attend la précédente, présente.
    fs::remove_file(ajouts.join(&noms[0])).expect("retirer le premier fichier d'ajout");
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::DependancesManquantes {
        chemin: ajouts.join(&noms[1]),
    };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "variante {variante} : le fonds ne nomme pas le fichier d'ajout qui attend celui qui \
         manque, dans {noms:?}"
    );
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

#[test]
fn un_ajout_altere_l_emporte_sur_une_dependance_manquante() {
    let chemin = commun::dossier_jetable("ajouts-altere-et-manquant").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);
    let noms = noter_des_idees(&mut fonds, &ajouts, &["idee-2", "idee-3", "idee-4"]);
    drop(fonds);
    // La deuxième écriture attend la première, dont le fichier d'ajout manque ; le fichier
    // d'ajout de la troisième est altéré.
    fs::remove_file(ajouts.join(&noms[0])).expect("retirer le premier fichier d'ajout");
    let altere = ajouts.join(&noms[2]);
    alterer_l_octet_du_milieu(&altere);
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    // Les fichiers d'ajout passent avant les dépendances.
    let raison = LectureSeule::AjoutAltere { chemin: altere };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fichier d'ajout altéré ne l'emporte pas sur la dépendance manquante"
    );
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

#[test]
fn de_deux_ajouts_alteres_le_premier_par_son_nom_se_nomme() {
    let chemin = commun::dossier_jetable("ajouts-deux-alteres").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);
    let noms = noter_des_idees(&mut fonds, &ajouts, &["idee-2", "idee-3"]);
    drop(fonds);
    for nom in &noms {
        alterer_l_octet_du_milieu(&ajouts.join(nom));
    }
    let premier = noms.iter().min().expect("deux fichiers d'ajout");
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);

    let mut fonds = ouvrir_le_fonds_tel_quel(&racine);

    let raison = LectureSeule::AjoutAltere {
        chemin: ajouts.join(premier),
    };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "le fonds ne nomme pas le premier des deux fichiers d'ajout altérés, dans l'ordre des \
         noms : {noms:?}"
    );
    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
}

/// Écrit dans `fonds` une idée courte pour chaque nom de `idees`, et rend le nom du fichier
/// d'ajout de chaque écriture, dans leur ordre : chacune est faite sans avertissement, et en
/// laisse un seul de plus dans `ajouts`.
fn noter_des_idees(fonds: &mut Document<'_>, ajouts: &Path, idees: &[&str]) -> Vec<String> {
    let mut noms = Vec::new();
    for (rang, idee) in (1_i64..).zip(idees) {
        let avant = commun::entrees(ajouts);
        let rendu = commun::ecrire_a(fonds, MESSAGE_DU_GESTE, HEURE + rang, |tx| {
            commun::noter(tx, idee, "Une phrase courte")
        });
        assert!(
            matches!(rendu, Ok(None)),
            "l'écriture {rang} n'est pas faite sans avertissement : {rendu:?}"
        );
        let nouveau = commun::nouveaux(&avant, &commun::entrees(ajouts));
        assert_eq!(
            nouveau.len(),
            1,
            "l'écriture {rang} n'a pas laissé un seul fichier d'ajout de plus : {nouveau:?}"
        );
        noms.extend(nouveau);
    }
    noms
}

/// Les octets d'une chaîne de quatre écritures, chacune attendant la précédente, sur un fonds
/// que le test bâtit : le fichier complet de la naissance, puis le fichier d'ajout de chaque
/// écriture, dans leur ordre. Un seul acteur, fixe, des heures fixes, et des textes que
/// `variante` fait différer : les mêmes octets à chaque passage.
fn une_chaine_d_ajouts(variante: usize) -> (Vec<u8>, Vec<Vec<u8>>) {
    let mut doc = commun::batir_un_fonds(
        ActorId::from(b"acteur-de-la-chaine".as_slice()),
        &[
            (IDENTIFIANT, NAISSANCE.identifiant),
            (LIGNEE, NAISSANCE.lignee),
            (PROPRIETAIRE, NAISSANCE.proprietaire),
        ],
    );
    let complet = doc.save();
    let mut contenus = Vec::new();
    for rang in 1..=4_i64 {
        let avant = doc.get_heads();
        let mut tx = doc.transaction();
        commun::noter(
            &mut tx,
            &format!("idee-{}", rang + 1),
            &format!("Variante {variante}, phrase {rang}"),
        )
        .expect("noter l'idée");
        tx.commit_with(
            CommitOptions::default()
                .with_message(MESSAGE_DU_GESTE)
                .with_time(HEURE + rang),
        );
        contenus.push(doc.save_after(&avant));
    }
    (complet, contenus)
}

// § 31 · 10, test 8 : relire du disque après une opération refusée, puis une opération
#[test]
fn une_ecriture_refusee_relit_le_document_du_disque() {
    // A et B, deux copies du fonds : B dit, après le refus, ce qu'A transmet.
    let (chemin, chemin_b) = deux_copies("ajouts-relu-du-disque");
    let racine = ouvrir(&chemin);
    let mut fonds = ouvrir_le_fonds(&racine);
    let heads_avant = fonds.automerge().get_heads();
    let acteur_avant = fonds.automerge().get_actor().clone();
    let ajouts = dossier_des_ajouts(&chemin);
    let ajouts_avant = commun::entrees(&ajouts);

    let refus = {
        let _panne = couper(Etape::Renommage, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(rendu) => panic!("un renommage refusé a laissé l'écriture faite : {rendu:?}"),
    }
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "la relecture du disque a mis le document en lecture seule"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_avant,
        "relu du disque, le document n'a pas les heads d'avant l'écriture refusée"
    );
    // La mémoire relue a un autre acteur qu'avant : `Automerge::load` en tire un, et
    // `documents.rs` en tire un explicitement aussi.
    assert_ne!(
        fonds.automerge().get_actor(),
        &acteur_avant,
        "la relecture du disque n'a pas tiré un acteur neuf"
    );
    assert_eq!(
        commun::entrees(&ajouts),
        ajouts_avant,
        "l'écriture refusée a laissé un fichier dans ajouts/"
    );
    match racine.fonds() {
        Err(Erreur::FondsDejaOuvert) => {}
        Err(autre) => panic!("attendu FondsDejaOuvert, rendu {autre:?}"),
        Ok(_) => panic!("la relecture du disque a rendu la garde du fonds"),
    }

    // Réunie avec B, A ne lui transmet pas l'écriture refusée, et n'en reçoit rien.
    let racine_b = ouvrir(&chemin_b);
    let mut b = ouvrir_le_fonds(&racine_b);
    commun::reunir(&mut fonds, &mut b);
    for (nom, doc) in [("A", fonds.automerge()), ("B", b.automerge())] {
        assert!(
            !porte_l_idee(doc, "idee-2"),
            "réunie, {nom} porte l'idée de l'écriture refusée"
        );
        assert_eq!(
            doc.get_heads(),
            heads_avant,
            "réunie, {nom} n'a pas les heads d'avant l'écriture refusée"
        );
    }
    assert_eq!(
        commun::entrees(&ajouts),
        ajouts_avant,
        "la réunion a écrit dans ajouts/ d'A"
    );

    match commun::ecrire(&mut fonds, |tx| {
        commun::noter(tx, "idee-3", "Une coda en suspens")
    }) {
        Ok(None) => {}
        autre => panic!("attendu l'écriture faite, rendu {autre:?}"),
    }
    commun::constater_un_ajout(
        fonds.automerge(),
        &heads_avant,
        &ajouts,
        &ajouts_avant,
        &commun::entrees(&ajouts),
    );
    assert_eq!(
        fonds.automerge().get_changes(&heads_avant).len(),
        1,
        "la mémoire porte encore l'écriture refusée"
    );
}

// § 31 · 10, test 8 : interrompre à chaque étape, puis rouvrir
#[test]
fn une_ecriture_coupee_a_chaque_etape() {
    // Avant la fin du renommage : refusée, sans voisin qui reste, et le fonds rouvert a les heads
    // d'avant.
    for (i, etape) in [
        Etape::VoisinEcrit,
        Etape::SyncAll,
        Etape::Relecture,
        Etape::Renommage,
    ]
    .into_iter()
    .enumerate()
    {
        let chemin = commun::dossier_jetable(&format!("ajouts-coupee-{i}")).join("donnees");
        let racine = ouvrir(&chemin);
        let mut fonds = faire_naitre(&racine);
        let avant = fonds.automerge().get_heads();
        let ajouts = dossier_des_ajouts(&chemin);
        let ajouts_avant = commun::entrees(&ajouts);

        let rendu = {
            let _panne = couper(etape, 1);
            commun::ecrire(&mut fonds, |tx| {
                commun::noter(tx, "idee-2", "Un pont en mineur")
            })
        };

        assert!(
            matches!(rendu, Err(Erreur::Disque { .. })),
            "{etape:?} : rendu {rendu:?}"
        );
        assert_eq!(
            commun::entrees(&ajouts),
            ajouts_avant,
            "{etape:?} : ajouts/ n'est plus tel qu'avant l'écriture refusée, un .partiel ou un \
             fichier d'ajout en plus"
        );
        drop(fonds);
        let fonds = ouvrir_le_fonds(&racine);
        assert_eq!(
            fonds.automerge().get_heads(),
            avant,
            "{etape:?} : le fonds rouvert n'a pas les heads d'avant l'écriture refusée"
        );
    }

    // Après le renommage : faite, avec un avertissement ; le fichier d'ajout est en place, et le
    // fonds rouvert porte l'idée.
    let chemin = commun::dossier_jetable("ajouts-coupee-rendre-durable").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match rendu {
        Ok(Some(Avertissement::NonDurable { chemin: ajout, .. })) => {
            assert_eq!(
                ajout.parent(),
                Some(ajouts.as_path()),
                "RendreDurable : l'avertissement ne nomme pas un fichier d'ajout : {}",
                ajout.display()
            );
            assert!(
                ajout.is_file(),
                "RendreDurable : le fichier d'ajout n'est pas en place"
            );
        }
        autre => panic!("RendreDurable : attendu NonDurable, rendu {autre:?}"),
    }
    drop(fonds);
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "RendreDurable : le fonds rouvert ne porte pas l'idée écrite"
    );
}

#[test]
fn une_relecture_qui_echoue_met_le_document_en_lecture_seule() {
    let chemin = commun::dossier_jetable("ajouts-relecture-manquee").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let heads_avant = fonds.automerge().get_heads();

    // La première relecture est celle du voisin, la seconde celle du fichier complet, quand le
    // document se relit du disque.
    let refus = {
        let _renommage = couper(Etape::Renommage, 1);
        let _relecture = couper(Etape::Relecture, 2);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(rendu) => panic!("un renommage refusé a laissé l'écriture faite : {rendu:?}"),
    }
    let dossier = dossier_du_fonds(&chemin);
    let avant = commun::empreintes(&dossier);
    let raison = LectureSeule::RelectureManquee {
        chemin: dossier.join("complet.automerge"),
    };
    assert_eq!(
        fonds.lecture_seule(),
        Some(&raison),
        "la relecture manquée ne met pas le document en lecture seule sur le fichier complet"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "la mémoire montre l'écriture refusée"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_avant,
        "la mémoire n'a pas les heads d'avant l'écriture refusée"
    );

    constater_la_lecture_seule(&mut fonds, &raison, &dossier, &avant);
    racine
        .captations()
        .verser(&commun::octets(3, 100))
        .expect("une captation se verse toujours");
    drop(fonds);

    let fonds = ouvrir_le_fonds(&racine);
    assert_eq!(
        fonds.lecture_seule(),
        None,
        "rouvert sans panne, le fonds est en lecture seule"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert porte l'écriture refusée"
    );
}

#[test]
fn une_relecture_sans_fichier_complet_met_le_document_en_lecture_seule() {
    // Le fichier complet ôté pendant que le document est ouvert : la relecture du disque, sans
    // fichier complet ni voisin qui le remplace, ne lit qu'un document sans genèse, jamais entier.
    let chemin = commun::dossier_jetable("ajouts-relecture-sans-complet").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    faite(
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        }),
        "la première écriture",
    );
    let complet = dossier_du_fonds(&chemin).join("complet.automerge");
    fs::remove_file(&complet).expect("ôter le fichier complet");

    let refus = {
        let _panne = couper(Etape::Renommage, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-3", "Une coda en suspens")
        })
    };

    assert!(
        matches!(refus, Err(Erreur::Disque { .. })),
        "attendu l'écriture refusée ; rendu {refus:?}"
    );
    assert_eq!(
        fonds.lecture_seule(),
        Some(&LectureSeule::CompletIllisible { chemin: complet }),
        "relu sans fichier complet, le document n'est pas en lecture seule sur le fichier complet"
    );
}

#[test]
fn un_fichier_a_la_place_des_ajouts_refuse_l_ecriture_puis_la_relecture() {
    // Le refus vient du disque, sans panne : le voisin ne se crée pas dans `ajouts/`, devenu un
    // fichier, et la relecture du disque ne lit pas `ajouts/`.
    let chemin = commun::dossier_jetable("ajouts-remplace-par-un-fichier").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let heads_avant = fonds.automerge().get_heads();
    let ajouts = dossier_des_ajouts(&chemin);
    let de_cote = ajouts.with_file_name("ajouts-de-cote");
    fs::rename(&ajouts, &de_cote).expect("mettre ajouts/ de côté");
    fs::write(&ajouts, b"un fichier a la place de ajouts/").expect("poser un fichier");

    let refus = commun::ecrire(&mut fonds, |tx| {
        commun::noter(tx, "idee-2", "Un pont en mineur")
    });

    match refus {
        Err(Erreur::Disque { .. }) => {}
        Err(autre) => panic!("attendu Disque, rendu {autre:?}"),
        Ok(rendu) => panic!("une écriture sans dossier ajouts/ est faite : {rendu:?}"),
    }
    assert_eq!(
        fonds.lecture_seule(),
        Some(&LectureSeule::RelectureManquee {
            chemin: ajouts.clone()
        }),
        "la relecture manquée ne met pas le document en lecture seule sur ajouts/"
    );
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "la mémoire montre l'écriture refusée"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_avant,
        "la mémoire n'a pas les heads d'avant l'écriture refusée"
    );
    drop(fonds);

    fs::remove_file(&ajouts).expect("ôter le fichier posé à la place de ajouts/");
    fs::rename(&de_cote, &ajouts).expect("rendre ajouts/");
    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "ajouts/, mis de côté, a changé"
    );
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        !porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert porte l'écriture refusée"
    );
    assert_eq!(
        fonds.automerge().get_heads(),
        heads_avant,
        "le fonds rouvert n'a pas les heads d'avant l'écriture refusée"
    );
}

// § 31 · 10, test 8 : une opération dite faite ne manque pas
#[test]
fn un_rendre_durable_refuse_apres_le_renommage_laisse_l_operation_faite() {
    let chemin = commun::dossier_jetable("ajouts-non-durable").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(&chemin);

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    match rendu {
        Ok(Some(Avertissement::NonDurable { chemin: ajout, .. })) => assert_eq!(
            ajout.parent(),
            Some(ajouts.as_path()),
            "l'avertissement ne nomme pas un fichier d'ajout : {}",
            ajout.display()
        ),
        autre => panic!("attendu NonDurable, rendu {autre:?}"),
    }
    drop(fonds);
    let fonds = ouvrir_le_fonds(&racine);
    assert!(
        porte_l_idee(fonds.automerge(), "idee-2"),
        "le fonds rouvert ne porte pas l'opération dite faite"
    );
    drop(fonds);

    // La naissance : le fichier complet en place, son renommage non rendu durable.
    let neuve = ouvrir(&commun::dossier_jetable("ajouts-naissance-non-durable").join("donnees"));
    let nee = {
        let _panne = couper(Etape::RendreDurable, 1);
        neuve.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", "Un refrain qui monte")
        })
    };

    let document = nee.expect("la naissance est faite, avec un avertissement");
    let complet = dossier_du_fonds(neuve.chemin()).join("complet.automerge");
    match document.avertissements() {
        [Avertissement::NonDurable { chemin, .. }] => assert_eq!(
            chemin, &complet,
            "l'avertissement de la naissance ne nomme pas le fichier complet"
        ),
        autres => panic!("attendu un seul NonDurable, rendu {autres:?}"),
    }
    drop(document);
    assert!(
        neuve.fonds().expect("rouvrir le fonds").is_some(),
        "le fonds né sans renommage durable ne se rouvre pas"
    );
}

#[test]
fn un_renommage_non_durable_rend_la_compaction_due() {
    // L'écriture : son fichier d'ajout en place, son renommage non rendu durable.
    let chemin = commun::dossier_jetable("ajouts-compaction-due").join("donnees");
    let racine = ouvrir(&chemin);
    let mut fonds = faire_naitre(&racine);

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        commun::ecrire(&mut fonds, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
    };

    assert!(
        matches!(rendu, Ok(Some(Avertissement::NonDurable { .. }))),
        "attendu l'écriture faite, avec NonDurable ; rendu {rendu:?}"
    );
    constater_la_compaction_due(&mut fonds, &chemin, "après l'écriture non durable");
    drop(fonds);

    // La naissance : le fichier complet en place, son renommage non rendu durable. Le fonds naît
    // d'une idée au texte long, pour que les fichiers d'ajout pèsent moins que le fichier complet.
    let chemin = commun::dossier_jetable("ajouts-compaction-due-naissance").join("donnees");
    let neuve = ouvrir(&chemin);
    let nee = {
        let _panne = couper(Etape::RendreDurable, 1);
        neuve.faire_naitre_le_fonds(&NAISSANCE, MESSAGE_DU_GESTE, HEURE, |tx| {
            commun::noter(tx, "idee-1", &texte_long())
        })
    };

    let mut document = nee.expect("la naissance est faite, avec un avertissement");
    assert!(
        matches!(
            document.avertissements(),
            [Avertissement::NonDurable { .. }]
        ),
        "attendu un seul NonDurable à la naissance, rendu {:?}",
        document.avertissements()
    );
    constater_la_compaction_due(&mut document, &chemin, "après la naissance non durable");
}

/// La compaction due de `fonds`, dans la racine des données `chemin` : l'écriture suivante,
/// courte, est faite sans avertissement, et laisse `ajouts/` vide, alors que rien d'autre ne
/// demandait de compacter. Les fichiers d'ajout, celui de l'écriture compris, pèsent ensemble
/// moins que le fichier complet, et ne passent pas `AJOUTS_AU_PLUS`.
fn constater_la_compaction_due(fonds: &mut Document<'_>, chemin: &Path, quand: &str) {
    let ajouts = dossier_des_ajouts(chemin);
    let noms = commun::entrees(&ajouts);
    let des_ajouts: u64 = noms.iter().map(|nom| poids(&ajouts.join(nom))).sum();
    let du_complet = poids(&dossier_du_fonds(chemin).join("complet.automerge"));
    let heads_avant = fonds.automerge().get_heads();

    let rendu = commun::ecrire(fonds, |tx| commun::etiqueter(tx, "idee-1", "refrain"));

    assert!(
        matches!(rendu, Ok(None)),
        "{quand} : attendu l'écriture suivante faite, sans avertissement ; rendu {rendu:?}"
    );
    // Ce que l'écriture suivante a écrit, et que la compaction a retiré.
    let du_suivant = fonds.automerge().save_after(&heads_avant).len() as u64;
    assert!(
        des_ajouts + du_suivant <= du_complet,
        "{quand} : les fichiers d'ajout pèsent {} octets, le fichier complet {du_complet} : la \
         compaction due n'était pas seule à la demander",
        des_ajouts + du_suivant
    );
    assert!(
        noms.len() < AJOUTS_AU_PLUS,
        "{quand} : {} fichiers d'ajout, celui de l'écriture suivante compris, passent \
         AJOUTS_AU_PLUS : la compaction due n'était pas seule à la demander",
        noms.len() + 1
    );
    assert_eq!(
        commun::entrees(&ajouts),
        Vec::<String>::new(),
        "{quand} : la compaction due n'a pas suivi l'écriture suivante"
    );
}
