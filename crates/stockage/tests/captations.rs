//! Les captations, rangées par empreinte (§ 31 · 6.1) : la voie par octets, la copie en flux,
//! leurs coupures, un flux à verser refusé en chemin, et la relecture d'une plage.

mod commun;

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use commun::entrees;
use noyau::donnees::import::nom_de_la_copie;
use services::{Raison, Refus, Versement};
use stockage::fichier::voisin;
use stockage::pannes::{self, Etape};
use stockage::{Erreur, RacineDesDonnees, empreinte};

/// L'import des copies en flux des tests : un ULID écrit, jamais tiré.
const IMPORT: &str = "01HF7YAT000000000000000004";

/// Un contenu de trois tronçons d'une copie en flux, deux entiers de 1 048 576 octets, puis un
/// demi : 2 621 440 octets engendrés par le test.
fn trois_troncons() -> Vec<u8> {
    commun::octets(11, 2_621_440)
}

/// Un flux à verser qui rend `reste`, puis refuse toute lecture (`PermissionDenied`), comme un
/// fichier que le système refuse en chemin.
struct FluxRefuse<'a> {
    reste: &'a [u8],
}

impl Read for FluxRefuse<'_> {
    fn read(&mut self, tampon: &mut [u8]) -> io::Result<usize> {
        if self.reste.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Access is denied. (os error 5)",
            ));
        }
        let lus = tampon.len().min(self.reste.len());
        tampon[..lus].copy_from_slice(&self.reste[..lus]);
        self.reste = &self.reste[lus..];
        Ok(lus)
    }
}

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

// § 31 · 10, test 4 : l'empreinte de chaque objet égale son nom
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

// § 31 · 10, test 4 : aucun objet modifié après écriture
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

// § 31 · 10, test 4 : l'empreinte de chaque objet égale son nom
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
    let provisoire = voisin(&dossier.join(empreinte(&commun::octets(5, 200)))).unwrap();
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

// § 31 · 6.1 : un nom provisoire laissé par un arrêt n'est jamais retiré d'office, ni tronqué
#[test]
fn verser_laisse_intact_un_nom_provisoire_laisse() {
    let d = commun::dossier_jetable("captation-nom-laisse");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let octets = commun::octets(7, 300);
    let e = empreinte(&octets);
    let laisse = racine
        .chemin()
        .join("captations")
        .join(format!("{e}.{}.partiel", std::process::id()));
    fs::write(&laisse, commun::octets(6, 200)).expect("déposer le nom provisoire laissé");

    let panne = pannes::couper(Etape::Renommage, 1);
    let rendu = captations.verser(&octets);
    drop(panne);

    assert!(rendu.is_err(), "rendu {rendu:?}");
    assert_eq!(
        fs::read(&laisse).expect("relire le nom laissé"),
        commun::octets(6, 200)
    );
    captations.verser(&octets).expect("verser sans panne");
    assert_eq!(
        fs::read(&laisse).expect("relire le nom laissé"),
        commun::octets(6, 200)
    );
}

// § 31 · 10, test 4 : couper l'écriture à chaque étape
#[test]
fn verser_coupe_a_chaque_etape_ne_laisse_aucune_captation_alteree() {
    let octets = commun::octets(7, 500);
    let e = empreinte(&octets);

    // Avant la fin du renommage : rien n'est rangé, aucun nom provisoire ne reste, et la
    // captation se verse ensuite, sans panne.
    for (i, etape) in [
        Etape::VoisinEcrit,
        Etape::SyncAll,
        Etape::Relecture,
        Etape::Renommage,
    ]
    .into_iter()
    .enumerate()
    {
        let d = commun::dossier_jetable(&format!("captation-coupee-{i}"));
        let racine =
            RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
        let captations = racine.captations();
        let dossier = racine.chemin().join("captations");
        let panne = pannes::couper(etape, 1);

        let rendu = captations.verser(&octets);

        assert!(
            matches!(rendu, Err(Erreur::Disque { .. })),
            "{etape:?} : rendu {rendu:?}"
        );
        let trahies = captations.verifier().expect("vérifier les captations");
        assert!(trahies.is_empty(), "{etape:?} : {trahies:?}");
        assert!(
            !commun::rangee(racine.chemin(), &e).exists(),
            "{etape:?} : la captation est rangée"
        );
        assert!(
            fichiers_a_meme(&dossier).is_empty(),
            "{etape:?} : un nom provisoire reste dans captations/"
        );

        drop(panne);
        assert_eq!(
            captations.verser(&octets).expect("verser sans panne"),
            e,
            "{etape:?}"
        );
        assert_eq!(
            captations.relire(&e).expect("relire la captation"),
            octets,
            "{etape:?}"
        );
    }

    // Après le renommage : le refus se rend, mais la captation est rangée, intacte.
    let d = commun::dossier_jetable("captation-coupee-rendre-durable");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let panne = pannes::couper(Etape::RendreDurable, 1);

    let rendu = captations.verser(&octets);

    assert!(
        matches!(rendu, Err(Erreur::Disque { .. })),
        "RendreDurable : rendu {rendu:?}"
    );
    drop(panne);
    assert_eq!(
        fs::read(commun::rangee(racine.chemin(), &e)).expect("lire la captation rangée"),
        octets,
        "la captation rangée a changé"
    );
    assert_eq!(captations.relire(&e).expect("relire la captation"), octets);
    let trahies = captations.verifier().expect("vérifier les captations");
    assert!(trahies.is_empty(), "{trahies:?}");
    assert!(
        fichiers_a_meme(&dossier).is_empty(),
        "un nom provisoire reste dans captations/"
    );
}

// La voie par octets passe par la copie en flux : un tronçon coupé la refuse.
#[test]
fn verser_par_octets_passe_par_le_flux() {
    let d = commun::dossier_jetable("verser-par-le-flux");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let octets = commun::octets(13, 900);
    let e = empreinte(&octets);

    let rendu = {
        let _panne = pannes::couper(Etape::Troncon, 1);
        captations.verser(&octets)
    };

    assert!(
        matches!(rendu, Err(Erreur::Disque { .. })),
        "le premier tronçon coupé : rendu {rendu:?}"
    );
    assert!(
        matches!(captations.relire(&e), Err(Erreur::CaptationAbsente { .. })),
        "la captation est rangée"
    );
    assert_eq!(
        entrees(&dossier),
        Vec::<String>::new(),
        "captations/ porte une captation ou un nom provisoire"
    );
}

// § 31 · 10, test 4 : couper l'écriture à chaque étape, sur la copie en flux (I14)
#[test]
fn i14_la_copie_en_flux_coupee_a_chaque_etape_ne_laisse_aucune_captation_alteree() {
    let contenu = trois_troncons();
    let e = empreinte(&contenu);
    let deja = commun::octets(12, 700);
    let coupures = [
        (Etape::VoisinEcrit, 1),
        (Etape::Troncon, 1),
        (Etape::Troncon, 2),
        (Etape::Troncon, 3),
        (Etape::SyncAll, 1),
        (Etape::Relecture, 1),
        (Etape::Renommage, 1),
        (Etape::RendreDurable, 1),
    ];

    for (i, (etape, rang)) in coupures.into_iter().enumerate() {
        let d = commun::dossier_jetable(&format!("copie-coupee-{i}"));
        let racine =
            RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
        let captations = racine.captations();
        let dossier = racine.chemin().join("captations");
        // Une captation déjà rangée, que la copie coupée ne réécrit pas.
        let e_deja = captations
            .verser(&deja)
            .expect("verser la captation déjà rangée");
        let avant = commun::empreintes(&dossier);

        let rendu = {
            let _panne = pannes::couper(etape, rang);
            captations.verser_en_flux(&nom_de_la_copie(IMPORT, 1), &mut contenu.as_slice(), None)
        };

        // Avant la fin du renommage, la copie est refusée ; après, le refus de rendre le
        // renommage durable se rend, mais la captation est rangée, entière.
        assert!(
            matches!(rendu, Err(Erreur::Disque { .. })),
            "{etape:?} au rang {rang} : rendu {rendu:?}"
        );
        let trahies = captations.verifier().expect("vérifier les captations");
        assert!(trahies.is_empty(), "{etape:?} au rang {rang} : {trahies:?}");
        let apres = commun::empreintes(&dossier);
        for (chemin, empreinte_d_avant) in &avant {
            assert_eq!(
                apres.get(chemin),
                Some(empreinte_d_avant),
                "{etape:?} au rang {rang} : {} a changé",
                chemin.display()
            );
        }
        assert!(
            fichiers_a_meme(&dossier).is_empty(),
            "{etape:?} au rang {rang} : un nom provisoire reste dans captations/"
        );
        if etape == Etape::RendreDurable {
            assert_eq!(
                captations.relire(&e).expect("relire la captation rangée"),
                contenu,
                "RendreDurable : la captation n'est pas rangée, entière"
            );
        } else {
            assert!(
                matches!(captations.relire(&e), Err(Erreur::CaptationAbsente { .. })),
                "{etape:?} au rang {rang} : la captation est rangée"
            );
        }
        assert_eq!(
            captations
                .relire(&e_deja)
                .expect("relire la captation déjà rangée"),
            deja,
            "{etape:?} au rang {rang}"
        );
    }
}

#[test]
fn une_copie_en_flux_alteree_a_la_relecture_est_refusee() {
    let d = commun::dossier_jetable("copie-alteree");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let contenu = trois_troncons();
    let e = empreinte(&contenu);

    let rendu = {
        let _panne = pannes::alterer_la_relecture(1);
        captations.verser_en_flux(&nom_de_la_copie(IMPORT, 2), &mut contenu.as_slice(), None)
    };

    match rendu {
        Err(Erreur::CaptationAlteree { empreinte: nommee }) => assert_eq!(nommee, e),
        autre => panic!("attendu CaptationAlteree, rendu {autre:?}"),
    }
    assert!(
        fichiers_a_meme(&dossier).is_empty(),
        "un nom provisoire reste dans captations/"
    );
    assert!(
        matches!(captations.relire(&e), Err(Erreur::CaptationAbsente { .. })),
        "la copie altérée est rangée"
    );
}

#[test]
fn une_copie_deja_rangee_retire_son_nom_provisoire() {
    let d = commun::dossier_jetable("copie-deja-rangee");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let contenu = commun::octets(14, 3000);
    let e = empreinte(&contenu);

    let premiere = captations
        .verser_en_flux(&nom_de_la_copie(IMPORT, 1), &mut contenu.as_slice(), None)
        .expect("la première copie");
    assert_eq!(
        premiere,
        Versement {
            empreinte: e.clone(),
            octets: 3000,
            deja_rangee: false,
            rangee: true,
        }
    );
    let rangee_avant = commun::empreintes(&dossier);

    let seconde = captations
        .verser_en_flux(&nom_de_la_copie(IMPORT, 2), &mut contenu.as_slice(), None)
        .expect("la seconde copie");

    assert_eq!(
        seconde,
        Versement {
            empreinte: e,
            octets: 3000,
            deja_rangee: true,
            rangee: true,
        }
    );
    assert!(
        fichiers_a_meme(&dossier).is_empty(),
        "un .partiel reste dans captations/"
    );
    assert_eq!(
        commun::empreintes(&dossier),
        rangee_avant,
        "la seconde copie a changé captations/"
    );
}

// R20 : un fichier changé depuis l'inspection ne range rien
#[test]
fn une_copie_qui_ne_donne_pas_l_empreinte_attendue_ne_range_rien() {
    let d = commun::dossier_jetable("copie-autre-empreinte");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    let contenu = commun::octets(15, 2000);
    let e = empreinte(&contenu);
    let autre = empreinte(&commun::octets(16, 2000));

    let versement = captations
        .verser_en_flux(
            &nom_de_la_copie(IMPORT, 1),
            &mut contenu.as_slice(),
            Some(&autre),
        )
        .expect("la copie se fait, sans rien ranger");

    assert_eq!(
        versement,
        Versement {
            empreinte: e.clone(),
            octets: 2000,
            deja_rangee: false,
            rangee: false,
        }
    );
    assert!(
        matches!(captations.relire(&e), Err(Erreur::CaptationAbsente { .. })),
        "une captation porte l'empreinte calculée"
    );
    assert_eq!(captations.taille(&e).expect("lire la taille"), None);
    assert_eq!(
        entrees(&dossier),
        Vec::<String>::new(),
        "captations/ porte une captation ou un .partiel"
    );
}

// Balayage Q8 : un fichier à verser que le système refuse en chemin n'est pas un refus du disque
// de la racine des données
#[test]
fn un_flux_refuse_en_chemin_rend_le_fichier_a_verser_et_ne_range_rien() {
    let d = commun::dossier_jetable("copie-flux-refuse");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let dossier = racine.chemin().join("captations");
    // Un tronçon entier s'écrit sous le nom provisoire avant que le flux refuse la suite.
    let troncon = commun::octets(17, 1_048_576);

    let rendu = captations.verser_en_flux(
        &nom_de_la_copie(IMPORT, 1),
        &mut FluxRefuse { reste: &troncon },
        None,
    );

    match rendu {
        Err(Erreur::FichierAVerser { erreur }) => {
            assert_eq!(erreur.kind(), io::ErrorKind::PermissionDenied)
        }
        autre => panic!("attendu le refus du fichier à verser, rendu {autre:?}"),
    }
    assert_eq!(
        entrees(&dossier),
        Vec::<String>::new(),
        "captations/ porte une captation ou un .partiel"
    );
    assert_eq!(
        captations.tailles().expect("lire les tailles"),
        BTreeSet::new(),
        "une captation est rangée"
    );

    // Par l'adaptateur : un refus du disque, au constat du fichier à verser, sans le texte du
    // système.
    let refus = services::Captations::verser_en_flux(
        &captations,
        &nom_de_la_copie(IMPORT, 2),
        &mut FluxRefuse { reste: &troncon },
        None,
    );
    assert_eq!(
        refus,
        Err(Refus {
            raison: Raison::Disque,
            message: "Le fichier à verser est refusé par le système.".to_string(),
        })
    );
    assert_eq!(
        entrees(&dossier),
        Vec::<String>::new(),
        "par l'adaptateur, captations/ porte une captation ou un .partiel"
    );
}

#[test]
fn relire_une_plage_rend_ses_octets() {
    let d = commun::dossier_jetable("captation-plage");
    let racine =
        RacineDesDonnees::ouvrir(&d.join("donnees")).expect("ouvrir la racine des données");
    let captations = racine.captations();
    let contenu: Vec<u8> = (0..=255u8).cycle().take(1000).collect();
    let e = captations.verser(&contenu).expect("verser la captation");

    assert_eq!(
        captations
            .relire_une_plage(&e, 10, 5)
            .expect("relire la plage (10, 5)"),
        [10, 11, 12, 13, 14]
    );
    assert_eq!(
        captations
            .relire_une_plage(&e, 998, 10)
            .expect("relire la plage (998, 10)"),
        [230, 231]
    );
    assert_eq!(
        captations
            .relire_une_plage(&e, 1000, 4)
            .expect("relire la plage (1 000, 4)"),
        Vec::<u8>::new()
    );
    assert_eq!(captations.taille(&e).expect("lire la taille"), Some(1000));
    assert_eq!(
        captations
            .taille(&empreinte(b"une captation jamais versee"))
            .expect("lire la taille d'une captation absente"),
        None
    );
    assert_eq!(
        captations.tailles().expect("lire les tailles"),
        BTreeSet::from([1000])
    );
}

/// Les noms des fichiers posés à même `dossier`, sans ses sous-dossiers : dans `captations/`, ceux
/// des noms provisoires.
fn fichiers_a_meme(dossier: &Path) -> Vec<String> {
    entrees(dossier)
        .into_iter()
        .filter(|nom| dossier.join(nom).is_file())
        .collect()
}
