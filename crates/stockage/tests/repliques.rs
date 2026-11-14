//! Les répliques du fonds, conduites par les services sur la paire de fichiers : deux copies
//! d'une même racine des données, modifiées hors ligne, puis réunies par l'échange de `synchro`
//! (H07). Le fichier inclut la suite des commandes, pour son aléa et ses commandes.

#[path = "../../services/tests/suite/mod.rs"]
mod suite;

mod commun;

use automerge::{ROOT, ReadDoc};
use noyau::donnees::fonds::LIENS;
use noyau::donnees::gestes;
use noyau::donnees::idee::cle_du_lien;
use services::Services;
use stockage::{Captations, FondsSurFichiers, RacineDesDonnees};
use suite::{AleaDeTest, contexte_de_test, lien, une_note};

/// Les services de l'établi sur le fonds et les captations de `racine`.
fn services_sur(racine: &RacineDesDonnees) -> Services<FondsSurFichiers<'_>, Captations<'_>> {
    Services::new(
        FondsSurFichiers::ouvrir(racine).expect("ouvrir le fonds"),
        racine.captations(),
        gestes::Appareil::Etabli,
    )
}

/// A et B notées par les services dans une racine des données, fermée, puis copiée ; chaque
/// copie rouverte avec son aléa ; sur la première, A reliée à B, sur la seconde, B reliée à A,
/// hors ligne ; puis les deux copies réunies. Chacune ne porte sous `liens` qu'une clé, celle du
/// lien de A et B, ses bouts dans l'ordre de leurs octets (§ 30 · 9.3), où chaque copie a posé
/// `true` : deux valeurs, égales.
#[test]
fn cap02_7_deux_liens_poses_en_sens_contraires_n_en_font_qu_un() {
    let d = commun::dossier_jetable("repliques-cap02-7");
    let (chemin_a, chemin_b) = (d.join("a"), d.join("b"));
    let (a, b) = {
        let racine = commun::ouvrir(&chemin_a);
        let mut services = services_sur(&racine);
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
        (a, b)
    };
    commun::copier_l_arbre(&chemin_a, &chemin_b);
    let racine_a = commun::ouvrir(&chemin_a);
    let racine_b = commun::ouvrir(&chemin_b);
    let mut premiere = services_sur(&racine_a);
    let mut seconde = services_sur(&racine_b);
    let mut alea_de_la_premiere = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut alea_de_la_seconde = AleaDeTest {
        graine: 1,
        appels: 0,
    };
    premiere
        .relier(
            &lien(&a, &b),
            &mut contexte_de_test(&mut alea_de_la_premiere),
        )
        .expect("A est reliée à B, sur la première copie");
    seconde
        .relier(
            &lien(&b, &a),
            &mut contexte_de_test(&mut alea_de_la_seconde),
        )
        .expect("B est reliée à A, sur la seconde copie");
    commun::reunir(
        premiere
            .documents_mut()
            .document_mut()
            .expect("le fonds de la première copie"),
        seconde
            .documents_mut()
            .document_mut()
            .expect("le fonds de la seconde copie"),
    );

    let cle = cle_du_lien(&a, &b);
    for (nom, copie) in [("première", &premiere), ("seconde", &seconde)] {
        let fonds = copie
            .documents()
            .document()
            .expect("le fonds est né")
            .automerge();
        let liens = commun::objet(fonds, &ROOT, LIENS);
        let cles: Vec<String> = fonds.keys(&liens).collect();
        // L'attendu faussé un temps, pour voir le rouge : deux clés sous `liens`.
        assert_eq!(
            cles.len(),
            2,
            "les clés des liens de la {nom} copie : {cles:?}"
        );
        let valeurs: Vec<Option<bool>> = fonds
            .get_all(&liens, cle.as_str())
            .expect("lire le lien")
            .into_iter()
            .map(|(valeur, _)| valeur.as_bool())
            .collect();
        assert_eq!(
            valeurs,
            [Some(true), Some(true)],
            "les valeurs du lien de la {nom} copie"
        );
    }
}
