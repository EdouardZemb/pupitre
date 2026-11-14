//! La suite des commandes sur la paire d'adaptateurs sur fichiers : chaque test de la suite,
//! rejoué sur `FondsSurFichiers` et les `Captations` d'une racine des données jetable ; puis, sur
//! cette paire seule, des commandes coupées par une panne, avant ou après le renommage de leur
//! fichier, la première comprise, qui fait naître le fonds, et la migration de test, la même dans
//! la suite et dans les aides du stockage.

#[path = "../../services/tests/suite/mod.rs"]
mod suite;

mod commun;

use commandes::Notee;
use noyau::donnees::acces::Acces;
use noyau::donnees::gestes;
use noyau::donnees::version::Migration;
use services::{Documents, Raison, Refus, Services};
use stockage::pannes::{Etape, couper};
use stockage::{Captations, Document, FondsSurFichiers, LectureSeule, RacineDesDonnees};
use suite::{AleaDeTest, IDEE, Paire, SECONDE_IDEE, contexte_de_test, une_note};

/// La paire sur fichiers, sur la racine des données `racine` : le fonds de la racine, sans
/// document à sa naissance, et ses captations, sur l'établi ; rouverte, les services fermés, puis
/// le fonds rouvert selon les migrations données, les captations gardées.
struct PaireSurFichiers<'r> {
    racine: &'r RacineDesDonnees,
}

impl<'r> PaireSurFichiers<'r> {
    /// Les services de l'établi sur `documents` et les captations de la racine des données.
    fn services(
        &self,
        documents: FondsSurFichiers<'r>,
    ) -> Services<FondsSurFichiers<'r>, Captations<'r>> {
        Services::new(
            documents,
            self.racine.captations(),
            gestes::Appareil::Etabli,
        )
    }
}

impl<'r> Paire for PaireSurFichiers<'r> {
    type D = FondsSurFichiers<'r>;
    type C = Captations<'r>;

    fn neufs(&mut self) -> Services<FondsSurFichiers<'r>, Captations<'r>> {
        let documents = FondsSurFichiers::ouvrir(self.racine).expect("ouvrir le fonds");
        self.services(documents)
    }

    fn rouvrir(
        &mut self,
        services: Services<FondsSurFichiers<'r>, Captations<'r>>,
        migrations: &[Migration<'_>],
    ) -> Services<FondsSurFichiers<'r>, Captations<'r>> {
        // Le document ouvert tient la garde de la racine des données jusqu'à sa fin : il se ferme
        // avant que le fonds se rouvre.
        drop(services);
        let documents =
            FondsSurFichiers::ouvrir_selon(self.racine, migrations).expect("rouvrir le fonds");
        self.services(documents)
    }
}

/// La racine des données du test `nom`, ouverte dans son dossier jetable.
fn racine_jetable(nom: &str) -> RacineDesDonnees {
    commun::ouvrir(&commun::dossier_jetable(nom).join("donnees"))
}

/// L'aléa des tests, d'une seule réplique.
fn alea_de_test() -> AleaDeTest {
    AleaDeTest {
        graine: 0,
        appels: 0,
    }
}

#[test]
fn cap01_6_une_idee_notee_a_l_etabli_est_captee_datee_sans_prise_ni_duree() {
    let racine = racine_jetable("commandes-cap01-6");
    suite::une_idee_notee_est_captee_datee_sans_prise_ni_duree(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn noter_fait_naitre_le_fonds_en_trois_modifications() {
    let racine = racine_jetable("commandes-naissance");
    suite::noter_fait_naitre_le_fonds_en_trois_modifications(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn une_note_vide_refusee_ne_fait_rien_naitre() {
    let racine = racine_jetable("commandes-note-vide");
    suite::une_note_vide_refusee_ne_fait_rien_naitre(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_note_au_decalage_local_dit_le_jour_decale() {
    let racine = racine_jetable("commandes-decalage-local");
    suite::une_note_au_decalage_local_dit_le_jour_decale(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_empreinte_qu_aucune_captation_ne_porte_ne_se_relit_pas() {
    let racine = racine_jetable("commandes-empreinte-inconnue");
    suite::une_empreinte_qu_aucune_captation_ne_porte_ne_se_relit_pas(&mut PaireSurFichiers {
        racine: &racine,
    });
}

// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que la version de schéma N
#[test]
fn une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme() {
    let racine = racine_jetable("commandes-plus-recent");
    suite::une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule() {
    let racine = racine_jetable("commandes-migration-en-attente");
    let services = suite::une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule(
        &mut PaireSurFichiers { racine: &racine },
    );
    // Sur fichiers, la raison de la lecture seule nomme la migration refusée, et ne laisse ni
    // écrire ni échanger.
    let raison = services
        .documents()
        .document()
        .and_then(Document::lecture_seule)
        .expect("le document se lit sans écrire");
    assert_eq!(raison, &LectureSeule::MigrationRefusee);
    assert_eq!(raison.acces(), Acces::Illisible);
}

#[test]
fn cap02_4_chaque_geste_de_tri_parait_au_journal_comme_une_operation_nommee() {
    let racine = racine_jetable("commandes-cap02-4");
    suite::chaque_geste_de_tri_parait_au_journal(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap02_8_l_idee_videe_par_un_regroupement_est_retiree_et_l_annulation_la_retablit() {
    let racine = racine_jetable("commandes-cap02-8");
    suite::l_idee_videe_par_un_regroupement_est_retiree(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap02_3_deux_prises_regroupees_restent_deux_captations_d_empreinte_inchangee() {
    let racine = racine_jetable("commandes-cap02-3");
    suite::deux_prises_regroupees_restent_deux_captations(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap02_5_une_idee_retiree_quitte_la_liste_et_se_retablit_telle_qu_elle_etait() {
    let racine = racine_jetable("commandes-cap02-5");
    suite::une_idee_retiree_quitte_la_liste_et_se_retablit_telle_qu_elle_etait(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait() {
    let racine = racine_jetable("commandes-tri-sans-fonds");
    suite::un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait(&mut PaireSurFichiers {
        racine: &racine,
    });
    // Le trait `Paire` ne donne aucun chemin : l'enveloppe, qui connaît la racine des données,
    // voit que le dossier du fonds n'est pas né.
    assert!(!racine.chemin().join("documents").join("fonds").exists());
}

#[test]
fn une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change() {
    let racine = racine_jetable("commandes-fiche-versee-deux-fois");
    suite::une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_geste_qui_touche_une_idee_retiree_est_refuse() {
    let racine = racine_jetable("commandes-idee-retiree-touchee");
    suite::un_geste_qui_touche_une_idee_retiree_est_refuse(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn la_vue_tire_reliee_et_la_duree_des_regles_du_noyau() {
    let racine = racine_jetable("commandes-reliee-et-duree");
    suite::la_vue_tire_reliee_et_la_duree_des_regles_du_noyau(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn la_migration_de_test_est_la_meme_dans_les_deux_crates() {
    assert_eq!(suite::migration_de_test(), commun::migration_de_test());
}

// § 31 · 10, test 8 : interrompre une écriture à chaque étape (fichier voisin écrit, synchronisé, renommé), puis rouvrir le document
#[test]
fn une_commande_coupee_avant_le_renommage_est_refusee() {
    let racine = racine_jetable("commandes-coupee-avant-le-renommage");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    services
        .noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la première note est faite");

    let rendu = {
        let _panne = couper(Etape::Renommage, 1);
        services.noter(
            &une_note("Un pont en mi mineur"),
            &mut contexte_de_test(&mut alea),
        )
    };
    assert!(
        matches!(
            rendu,
            Err(Refus {
                raison: Raison::Disque,
                ..
            })
        ),
        "attendu le refus du disque ; rendu {rendu:?}"
    );

    let services = paire.rouvrir(services, &[]);
    assert_eq!(
        services
            .vue_d_une_idee(SECONDE_IDEE)
            .map_err(|refus| refus.raison),
        Err(Raison::Impossible)
    );
    let journal = services.journal().expect("le journal se lit");
    assert_eq!(journal.len(), 3, "{journal:?}");
}

// § 31 · 10, test 8 : interrompre une écriture à chaque étape (fichier voisin écrit, synchronisé, renommé), puis rouvrir le document
#[test]
fn une_commande_coupee_apres_le_renommage_est_faite_avec_un_avertissement() {
    let racine = racine_jetable("commandes-coupee-apres-le-renommage");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    services
        .noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la première note est faite");

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        services.noter(
            &une_note("Un pont en mi mineur"),
            &mut contexte_de_test(&mut alea),
        )
    };
    match rendu {
        Ok(Notee {
            idee,
            avertissement: Some(_),
        }) => assert_eq!(idee, SECONDE_IDEE),
        autre => panic!("attendu la note faite, avec un avertissement ; rendu {autre:?}"),
    }

    let services = paire.rouvrir(services, &[]);
    let vue = services
        .vue_d_une_idee(SECONDE_IDEE)
        .expect("l'idée de la seconde note se lit");
    assert_eq!(vue.texte.as_deref(), Some("Un pont en mi mineur"));
}

// § 31 · 10, test 8 : interrompre une écriture à chaque étape (fichier voisin écrit, synchronisé, renommé), puis rouvrir le document
/// La première note, coupée après le renommage du fichier complet de la naissance : le fonds est
/// né, et la note rend l'avertissement de la naissance (`FondsSurFichiers::faire_naitre`) ;
/// rouvert, l'idée notée se lit.
#[test]
fn la_premiere_note_coupee_apres_le_renommage_fait_naitre_le_fonds_avec_un_avertissement() {
    let racine = racine_jetable("commandes-naissance-coupee-apres-le-renommage");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        services.noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
    };
    match rendu {
        Ok(Notee {
            idee,
            avertissement: Some(_),
        }) => assert_eq!(idee, IDEE),
        autre => {
            panic!("attendu la note faite, avec l'avertissement de la naissance ; rendu {autre:?}")
        }
    }

    let services = paire.rouvrir(services, &[]);
    let vue = services
        .vue_d_une_idee(IDEE)
        .expect("l'idée de la première note se lit");
    assert_eq!(vue.texte.as_deref(), Some("Un refrain qui monte, en ré"));
}

#[test]
fn le_premier_geste_refuse_laisse_la_racine_des_donnees_sans_document() {
    let chemin = commun::dossier_jetable("commandes-premier-geste-refuse").join("donnees");
    let racine = commun::ouvrir(&chemin);
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();

    let rendu = {
        let _panne = couper(Etape::Renommage, 1);
        services.noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
    };
    assert!(
        matches!(
            rendu,
            Err(Refus {
                raison: Raison::Disque,
                ..
            })
        ),
        "attendu le refus du disque ; rendu {rendu:?}"
    );
    let complet = commun::dossier_du_fonds(&chemin).join("complet.automerge");
    assert!(
        !complet.try_exists().expect("lire le dossier du fonds"),
        "le fichier complet du premier geste refusé est en place"
    );

    let services = paire.rouvrir(services, &[]);
    assert!(services.documents().fonds().is_none());
}
