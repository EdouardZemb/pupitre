//! La compaction (§ 31 · 2.2) : elle fond les fichiers d'ajout dans un fichier complet neuf, le
//! relit, puis retire les seuls fichiers d'ajout lisibles et absorbés. Elle a lieu à
//! l'ouverture, puis dès que les ajouts pèsent plus que le fichier complet.
//!
//! Le fonds naît d'une idée au texte long, qui ne sert qu'à la naissance : les écritures des
//! tests sont courtes, et leurs fichiers d'ajout pèsent bien moins que le fichier complet.

mod commun;

use std::fs;

use automerge::{Automerge, Change};
use commun::{
    HEURE, MESSAGE_DU_GESTE, NAISSANCE, dossier_des_ajouts, dossier_du_fonds, faire_naitre,
    lire_le_complet, ouvrir, ouvrir_le_fonds, poids, porte_l_idee, texte_long,
};
use noyau::donnees::fonds::{GENESE, MESSAGE_DE_LA_GENESE, MESSAGE_DE_LA_NAISSANCE};
use stockage::pannes::{Etape, couper};
use stockage::{Avertissement, Erreur, RacineDesDonnees};

#[test]
fn rouvrir_compacte_les_ajouts() {
    let chemin = commun::dossier_jetable("compaction-rouvrir").join("donnees");
    let racine = RacineDesDonnees::ouvrir(&chemin).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("écrire");
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
        fonds
            .ecrire("Étiquette posée", HEURE + n, |tx| {
                commun::etiqueter(tx, "idee-1", &format!("etiquette-{n}"))
            })
            .unwrap_or_else(|e| panic!("l'écriture {n} : {e:?}"));

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
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("la première écriture");
    fonds
        .ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
        .expect("la deuxième écriture");
    fonds
        .ecrire("Texte changé", HEURE + 3, |tx| {
            commun::changer_le_texte(tx, "idee-2", "Un pont plus court")
        })
        .expect("la troisième écriture");
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

#[test]
fn la_compaction_garde_un_ajout_altere() {
    let d = commun::dossier_jetable("compaction-altere");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let mut fonds = faire_naitre(&racine);
    let ajouts = dossier_des_ajouts(racine.chemin());
    fonds
        .ecrire("Étiquette posée", HEURE + 1, |tx| {
            commun::etiqueter(tx, "idee-1", "refrain")
        })
        .expect("la première écriture");
    let premier = commun::entrees(&ajouts);
    assert_eq!(
        premier.len(),
        1,
        "une écriture, un fichier d'ajout : {premier:?}"
    );
    fonds
        .ecrire("Idée notée", HEURE + 2, |tx| {
            commun::noter(tx, "idee-2", "Un pont en mineur")
        })
        .expect("la seconde écriture");
    let noms = commun::entrees(&ajouts);
    assert_eq!(
        noms.len(),
        2,
        "deux écritures, deux fichiers d'ajout : {noms:?}"
    );
    // Un octet changé, au milieu du fichier d'ajout de la première écriture.
    let altere = ajouts.join(&premier[0]);
    let mut octets = fs::read(&altere).expect("lire le fichier d'ajout");
    let milieu = octets.len() / 2;
    octets[milieu] ^= 0xFF;
    fs::write(&altere, &octets).expect("altérer le fichier d'ajout");

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
    match racine.fonds() {
        Err(Erreur::AjoutAltere { chemin }) => {
            assert_eq!(chemin, altere, "l'erreur ne nomme pas le fichier altéré");
        }
        Err(autre) => panic!("attendu AjoutAltere, rendu {autre:?}"),
        Ok(_) => panic!("le fonds s'est ouvert sur un fichier d'ajout altéré"),
    }
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
