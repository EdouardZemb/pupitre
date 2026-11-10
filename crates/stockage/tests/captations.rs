//! Les captations, rangées par empreinte (§ 31 · 6.1).

mod commun;

use std::fs;

use commun::entrees;
use stockage::fichier::voisin;
use stockage::{Erreur, RacineDesDonnees, empreinte};

#[test]
fn l_empreinte_est_le_sha_256_en_hexadecimal() {
    assert_eq!(
        empreinte(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        empreinte(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn une_captation_se_range_sous_son_empreinte() {
    let d = commun::dossier_jetable("captation-rangee");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let mut empreintes = Vec::new();

    // Une captation engendrée, puis une captation vide, qui a son empreinte comme une autre.
    for octets in [commun::octets(1, 4000), Vec::new()] {
        let e = captations.verser(&octets).expect("verser la captation");

        assert_eq!(e, empreinte(&octets), "verser rend l'empreinte des octets");
        assert_eq!(
            fs::read(commun::rangee(racine.chemin(), &e)).expect("lire la captation rangée"),
            octets
        );
        assert_eq!(captations.relire(&e).expect("relire la captation"), octets);
        empreintes.push(e);
    }

    let mut sous_dossiers: Vec<String> = empreintes.iter().map(|e| e[..2].to_string()).collect();
    sous_dossiers.sort();
    sous_dossiers.dedup();
    assert_eq!(
        entrees(&dossier),
        sous_dossiers,
        "captations/ n'a que ses sous-dossiers"
    );
    for sous_dossier in &sous_dossiers {
        let mut attendues: Vec<&str> = empreintes
            .iter()
            .filter(|e| e.starts_with(sous_dossier.as_str()))
            .map(String::as_str)
            .collect();
        attendues.sort();
        assert_eq!(
            entrees(&dossier.join(sous_dossier)),
            attendues,
            "le sous-dossier {sous_dossier} ne porte que ses captations"
        );
    }
}

#[test]
fn verser_deux_fois_ne_fait_qu_un_objet() {
    // L'objet est mis en lecture seule avant le second versement : s'il était réécrit, ou
    // seulement rouvert en écriture, le second versement échouerait. La lecture seule n'est pas
    // levée : `commun::dossier_jetable` efface par `std::fs::remove_dir_all`, qui l'ignore sous
    // Windows, et sous Linux seuls les droits du dossier comptent.
    let d = commun::dossier_jetable("captation-deux-fois");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let octets = commun::octets(2, 2048);
    let e = captations.verser(&octets).expect("le premier versement");
    let objet = commun::rangee(racine.chemin(), &e);
    let mut droits = fs::metadata(&objet)
        .expect("lire les droits de l'objet")
        .permissions();
    droits.set_readonly(true);
    fs::set_permissions(&objet, droits).expect("mettre l'objet en lecture seule");

    let encore = captations.verser(&octets).expect("le second versement");

    assert_eq!(encore, e, "le second versement rend la même empreinte");
    assert_eq!(entrees(&dossier), [&e[..2]], "aucun .partiel ne reste");
    assert_eq!(
        entrees(&dossier.join(&e[..2])),
        [e.as_str()],
        "un seul objet"
    );
    assert_eq!(
        fs::read(&objet).expect("relire l'objet"),
        octets,
        "l'objet a changé"
    );
}

#[test]
fn une_captation_alteree_ou_absente_se_nomme() {
    let d = commun::dossier_jetable("captation-alteree");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");

    // Une captation saine, que rien ne nomme.
    captations
        .verser(&commun::octets(3, 600))
        .expect("verser la captation saine");

    // Une captation dont un octet a changé sur le disque.
    let e = captations
        .verser(&commun::octets(4, 1000))
        .expect("verser la captation à altérer");
    let alteree = commun::rangee(racine.chemin(), &e);
    let mut octets = fs::read(&alteree).expect("lire la captation à altérer");
    octets[500] ^= 0x01;
    fs::write(&alteree, &octets).expect("altérer la captation");

    match captations.relire(&e) {
        Err(Erreur::CaptationAlteree { empreinte: nommee }) => assert_eq!(nommee, e),
        Err(autre) => panic!("attendu CaptationAlteree, rendu {autre:?}"),
        Ok(_) => panic!("la captation altérée s'est relue"),
    }

    // Une empreinte qu'aucune captation ne porte, puis un nom qui n'est pas une empreinte.
    let inconnue = empreinte(b"une captation jamais versee");
    for absente in [inconnue.as_str(), "../verrou"] {
        match captations.relire(absente) {
            Err(Erreur::CaptationAbsente { empreinte: nommee }) => assert_eq!(nommee, absente),
            Err(autre) => panic!("{absente} : attendu CaptationAbsente, rendu {autre:?}"),
            Ok(_) => panic!("{absente} : une captation absente s'est relue"),
        }
    }

    // Un nom provisoire déposé dans captations/, dont le contenu ne donne pas le nom.
    let provisoire = voisin(&dossier.join(empreinte(&commun::octets(5, 200))));
    fs::write(&provisoire, commun::octets(6, 200)).expect("déposer le nom provisoire");

    // Une captation saine, déposée hors du sous-dossier de ses deux premiers caractères.
    let egaree = commun::octets(7, 300);
    let e_egaree = empreinte(&egaree);
    let ailleurs = if e_egaree.starts_with("00") {
        "01"
    } else {
        "00"
    };
    let egaree_rangee = dossier.join(ailleurs).join(&e_egaree);
    fs::create_dir_all(dossier.join(ailleurs)).expect("créer le sous-dossier d'ailleurs");
    fs::write(&egaree_rangee, &egaree).expect("déposer la captation égarée");

    let mut attendues = vec![alteree, egaree_rangee];
    attendues.sort();
    assert_eq!(
        captations.verifier().expect("vérifier les captations"),
        attendues
    );
    assert!(
        provisoire.is_file(),
        "un nom provisoire n'est jamais retiré d'office"
    );
}
