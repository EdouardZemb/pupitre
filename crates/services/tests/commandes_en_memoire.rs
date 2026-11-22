//! La suite des commandes sur la paire d'adaptateurs en mémoire : chaque test de la suite,
//! rejoué sur `DocumentsEnMemoire` et `CaptationsEnMemoire` ; puis, sur cette paire seule, les
//! services du carnet, un renommage d'une requête enregistrée concurrent de sa suppression, sur
//! deux répliques, les requêtes enregistrées d'une version plus récente, une idée illisible dans
//! l'assiette de la boîte, la requête des photos, le générateur du fonds de mesure, une copie en
//! flux qui ne donne pas l'empreinte attendue ou dont le flux est refusé, et la source à verser en
//! mémoire.

mod suite;

use std::collections::BTreeSet;
use std::io::{self, Read};

use automerge::{ROOT, ReadDoc};
use commandes::Assiette;
use noyau::donnees::fonds::REQUETES_ENREGISTREES;
use noyau::donnees::gestes;
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::import::nom_de_la_copie;
use noyau::donnees::requete::{
    Requete as RequeteDuNoyau, RequeteEnregistreeLue, renommer_une_requete, supprimer_une_requete,
};
use noyau::donnees::version::Migration;
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::mesure::{Engendre, engendrer_le_fonds_de_mesure_selon};
use services::source_en_memoire::SourceEnMemoire;
use services::{
    Captations, Documents, Raison, Refus, Services, Source, Versement, application, lecture,
};
use suite::{AleaDeTest, HEURE, IDEE, Paire, SECONDE_IDEE, contexte_de_test, une_note};

/// La paire en mémoire : sans fonds à sa naissance, sur l'établi ; rouverte des octets de son
/// fonds, ou neuve sans eux, les captations gardées.
struct PaireEnMemoire;

impl Paire for PaireEnMemoire {
    type D = DocumentsEnMemoire;
    type C = CaptationsEnMemoire;

    fn neufs(&mut self) -> Services<DocumentsEnMemoire, CaptationsEnMemoire> {
        Services::new(
            DocumentsEnMemoire::neuf(),
            CaptationsEnMemoire::default(),
            gestes::Appareil::Etabli,
        )
    }

    fn rouvrir(
        &mut self,
        services: Services<DocumentsEnMemoire, CaptationsEnMemoire>,
        migrations: &[Migration<'_>],
    ) -> Services<DocumentsEnMemoire, CaptationsEnMemoire> {
        let (documents, captations) = services.en_parties();
        let documents = match documents.octets() {
            Some(octets) => {
                DocumentsEnMemoire::de(&octets, migrations).expect("le fonds se rouvre")
            }
            None => DocumentsEnMemoire::neuf(),
        };
        Services::new(documents, captations, gestes::Appareil::Etabli)
    }
}

#[test]
fn cap01_6_une_idee_notee_a_l_etabli_est_captee_datee_sans_prise_ni_duree() {
    suite::une_idee_notee_est_captee_datee_sans_prise_ni_duree(&mut PaireEnMemoire);
}

#[test]
fn noter_fait_naitre_le_fonds_en_trois_modifications() {
    suite::noter_fait_naitre_le_fonds_en_trois_modifications(&mut PaireEnMemoire);
}

#[test]
fn une_note_vide_refusee_ne_fait_rien_naitre() {
    suite::une_note_vide_refusee_ne_fait_rien_naitre(&mut PaireEnMemoire);
}

#[test]
fn une_note_au_decalage_local_dit_le_jour_decale() {
    suite::une_note_au_decalage_local_dit_le_jour_decale(&mut PaireEnMemoire);
}

#[test]
fn chaque_idee_d_un_geste_se_date_a_son_propre_decalage() {
    suite::chaque_idee_d_un_geste_se_date_a_son_propre_decalage(&mut PaireEnMemoire);
}

#[test]
fn une_empreinte_qu_aucune_captation_ne_porte_ne_se_relit_pas() {
    suite::une_empreinte_qu_aucune_captation_ne_porte_ne_se_relit_pas(&mut PaireEnMemoire);
}

/// Des services bâtis sur le carnet : l'idée qu'ils notent est captée au carnet, ce que dit sa
/// vue ; les services passent leur appareil au noyau.
#[test]
fn les_services_du_carnet_notent_une_idee_captee_au_carnet() {
    let mut services = Services::new(
        DocumentsEnMemoire::neuf(),
        CaptationsEnMemoire::default(),
        gestes::Appareil::Carnet,
    );
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    services
        .noter(
            &une_note("Un refrain qui monte, en ré"),
            &mut contexte_de_test(&mut alea),
        )
        .expect("la note est faite");
    let vue = services
        .vue_d_une_idee(IDEE)
        .expect("la vue de l'idée se lit");
    assert_eq!(
        vue.provenance,
        commandes::Provenance::Captee {
            par: commandes::Appareil::Carnet,
        }
    );
}

// § 31 · 10, test 10 : ouvrir un document de version de schéma N + 1 avec une application qui ne connaît que la version de schéma N
#[test]
fn une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme() {
    suite::une_commande_sur_un_fonds_plus_recent_est_refusee_et_le_nomme(&mut PaireEnMemoire);
}

#[test]
fn une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule() {
    let services = suite::une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule(
        &mut PaireEnMemoire,
    );
    // En mémoire, le refus nomme la migration qui ne s'applique pas.
    let refus = services
        .documents()
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(
        refus.message,
        "Une migration de ce fonds ne s'applique pas : il se lit sans écrire ni échanger."
    );
}

#[test]
fn cap02_4_chaque_geste_de_tri_parait_au_journal_comme_une_operation_nommee() {
    suite::chaque_geste_de_tri_parait_au_journal(&mut PaireEnMemoire);
}

#[test]
fn cap02_8_l_idee_videe_par_un_regroupement_est_retiree_et_l_annulation_la_retablit() {
    suite::l_idee_videe_par_un_regroupement_est_retiree(&mut PaireEnMemoire);
}

#[test]
fn cap02_3_deux_prises_regroupees_restent_deux_captations_d_empreinte_inchangee() {
    suite::deux_prises_regroupees_restent_deux_captations(&mut PaireEnMemoire);
}

#[test]
fn cap02_5_une_idee_retiree_quitte_la_liste_et_se_retablit_telle_qu_elle_etait() {
    suite::une_idee_retiree_quitte_la_liste_et_se_retablit_telle_qu_elle_etait(&mut PaireEnMemoire);
}

#[test]
fn un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait() {
    suite::un_geste_de_tri_sans_fonds_est_refuse_et_rien_ne_nait(&mut PaireEnMemoire);
}

#[test]
fn une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change() {
    suite::une_fiche_versee_deux_fois_est_refusee_et_rien_ne_change(&mut PaireEnMemoire);
}

#[test]
fn un_geste_qui_touche_une_idee_retiree_est_refuse() {
    suite::un_geste_qui_touche_une_idee_retiree_est_refuse(&mut PaireEnMemoire);
}

#[test]
fn regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse() {
    suite::regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse(&mut PaireEnMemoire);
}

#[test]
fn une_cle_d_une_autre_forme_sous_les_liens_n_est_pas_un_lien() {
    suite::une_cle_d_une_autre_forme_sous_les_liens_n_est_pas_un_lien(&mut PaireEnMemoire);
}

#[test]
fn la_vue_tire_reliee_et_la_duree_des_regles_du_noyau() {
    suite::la_vue_tire_reliee_et_la_duree_des_regles_du_noyau(&mut PaireEnMemoire);
}

#[test]
fn cap02_6_annuler_rend_le_fonds_d_avant_et_le_journal_porte_le_geste_puis_son_annulation() {
    suite::annuler_rend_le_fonds_d_avant(&mut PaireEnMemoire);
}

#[test]
fn annuler_ne_vise_que_le_dernier_geste_de_tri_depuis_l_ouverture() {
    suite::annuler_ne_vise_que_le_dernier_geste_de_tri_depuis_l_ouverture(&mut PaireEnMemoire);
}

#[test]
fn un_geste_refuse_ne_change_pas_ce_qu_annuler_vise() {
    suite::un_geste_refuse_ne_change_pas_ce_qu_annuler_vise(&mut PaireEnMemoire);
}

#[test]
fn la_reouverture_vide_ce_qu_annuler_vise() {
    suite::la_reouverture_vide_ce_qu_annuler_vise(&mut PaireEnMemoire);
}

#[test]
fn une_annulation_ne_s_annule_pas() {
    suite::une_annulation_ne_s_annule_pas(&mut PaireEnMemoire);
}

#[test]
fn chaque_operation_laisse_une_seule_modification_avec_son_message() {
    suite::chaque_operation_laisse_une_seule_modification_avec_son_message(&mut PaireEnMemoire);
}

#[test]
fn l_historique_ne_porte_que_des_operations_connues() {
    suite::l_historique_ne_porte_que_des_operations_connues(&mut PaireEnMemoire);
}

// § 31 · 10, test 7 : chercher les champs interdits du § 30 · 13
#[test]
fn rien_d_engendre_n_est_stocke() {
    suite::rien_d_engendre_n_est_stocke(&mut PaireEnMemoire);
}

#[test]
fn aucun_counter_ni_objet_text_dans_un_document_des_operations() {
    suite::aucun_counter_ni_objet_text_dans_un_document_des_operations(&mut PaireEnMemoire);
}

#[test]
fn cap02_1_aucun_compteur_d_anciennete() {
    suite::aucun_compteur_d_anciennete(&mut PaireEnMemoire);
}

#[test]
fn cap02_3_la_boite_montre_ensemble_deux_prises_regroupees() {
    suite::deux_prises_regroupees_paraissent_ensemble_dans_la_boite(&mut PaireEnMemoire);
}

#[test]
fn cap02_5_retirer_ote_l_idee_de_la_boite_et_retablir_l_y_rend() {
    suite::retirer_ote_l_idee_de_la_boite_et_retablir_l_y_rend(&mut PaireEnMemoire);
}

#[test]
fn cap07_4_le_nombre_d_idees_non_reliees_suit_chaque_geste() {
    suite::le_nombre_d_idees_non_reliees_suit_chaque_geste(&mut PaireEnMemoire);
}

#[test]
fn i31_un_texte_piege_revient_tel_quel() {
    suite::un_texte_piege_revient_tel_quel(&mut PaireEnMemoire);
}

#[test]
fn une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop() {
    suite::une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop(&mut PaireEnMemoire);
}

#[test]
fn la_boite_d_un_pupitre_neuf() {
    suite::la_boite_d_un_pupitre_neuf(&mut PaireEnMemoire);
}

#[test]
fn une_ligne_borne_son_texte() {
    suite::une_ligne_borne_son_texte(&mut PaireEnMemoire);
}

#[test]
fn l_etat_du_fonds_dit_le_geste_qu_annuler_vise() {
    suite::l_etat_du_fonds_dit_le_geste_qu_annuler_vise(&mut PaireEnMemoire);
}

#[test]
fn les_retirees_se_rangent_par_la_date_du_retrait() {
    suite::les_retirees_se_rangent_par_la_date_du_retrait(&mut PaireEnMemoire);
}

#[test]
fn la_requete_passe_au_noyau() {
    suite::la_requete_passe_au_noyau(&mut PaireEnMemoire, &mut PaireEnMemoire);
}

#[test]
fn cap06_7_plus_d_une_minute_declare_son_assiette() {
    suite::plus_d_une_minute_declare_son_assiette(&mut PaireEnMemoire);
}

#[test]
fn cap06_8_une_idee_retiree_n_est_ni_trouvee_ni_comptee() {
    suite::une_idee_retiree_n_est_ni_trouvee_ni_comptee(&mut PaireEnMemoire);
}

#[test]
fn cap02_5_une_idee_retiree_quitte_les_requetes() {
    suite::une_idee_retiree_quitte_les_requetes(&mut PaireEnMemoire);
}

#[test]
fn cap06_9_une_prise_sans_duree_compte_parmi_les_sans_duree() {
    suite::une_prise_sans_duree_compte_parmi_les_sans_duree(&mut PaireEnMemoire);
}

#[test]
fn cap06_10_eglise_trouve_eglise_du_port() {
    suite::eglise_trouve_eglise_du_port(&mut PaireEnMemoire);
}

#[test]
fn i09_chaque_reponse_declare_son_assiette() {
    suite::chaque_reponse_declare_son_assiette(&mut PaireEnMemoire);
}

#[test]
fn i13_l_index_reconstruit_rend_les_memes_reponses() {
    suite::l_index_reconstruit_rend_les_memes_reponses(&mut PaireEnMemoire);
}

#[test]
fn chercher_n_ecrit_rien() {
    suite::chercher_n_ecrit_rien(&mut PaireEnMemoire);
}

#[test]
fn noter_pendant_une_requete_la_laisse_en_place() {
    suite::noter_pendant_une_requete_la_laisse_en_place(&mut PaireEnMemoire);
}

#[test]
fn enregistrer_renommer_et_supprimer_une_requete() {
    suite::enregistrer_renommer_et_supprimer_une_requete(&mut PaireEnMemoire);
}

#[test]
fn une_requete_enregistree_se_relit_critere_pour_critere() {
    suite::une_requete_enregistree_se_relit_critere_pour_critere(&mut PaireEnMemoire);
}

#[test]
fn enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait() {
    suite::enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait(&mut PaireEnMemoire);
}

#[test]
fn en_lecture_seule_chercher_sert_et_enregistrer_est_refuse() {
    suite::en_lecture_seule_chercher_sert_et_enregistrer_est_refuse(&mut PaireEnMemoire);
}

#[test]
fn i01_une_requete_enregistree_ne_garde_ni_resultats_ni_nombre() {
    suite::une_requete_enregistree_ne_garde_ni_resultats_ni_nombre(
        &mut PaireEnMemoire,
        &mut PaireEnMemoire,
    );
}

#[test]
fn le_journal_se_lit_en_fenetre() {
    suite::le_journal_se_lit_en_fenetre(&mut PaireEnMemoire);
}

#[test]
fn i01_un_fonds_importe_ne_garde_ni_compte_ni_total() {
    suite::un_fonds_importe_ne_garde_ni_compte_ni_total(&mut PaireEnMemoire);
}

#[test]
fn la_duree_d_une_prise_en_stems_vient_de_ses_stems() {
    suite::la_duree_d_une_prise_en_stems_vient_de_ses_stems(&mut PaireEnMemoire);
}

#[test]
fn les_noms_d_un_contenu_et_son_premier_nom_vu() {
    suite::les_noms_d_un_contenu_et_son_premier_nom_vu(&mut PaireEnMemoire);
}

#[test]
fn les_noms_poses_par_un_import() {
    suite::les_noms_poses_par_un_import(&mut PaireEnMemoire);
}

#[test]
fn un_nom_pose_hors_du_jeu_n_est_pas_parmi_ceux_du_stem() {
    suite::un_nom_pose_hors_du_jeu_n_est_pas_parmi_ceux_du_stem(&mut PaireEnMemoire);
}

/// Un fonds où « Ponts lents » est enregistrée (le texte « pont », l'étiquette `lent`, plus d'une
/// minute), et deux répliques de son document (`Automerge::fork`) : l'une renomme la requête
/// « Ponts du soir » (`renommer_une_requete`), l'autre la supprime (`supprimer_une_requete`),
/// chacune par `application::appliquer`. Fusionnées (`Automerge::merge`), dans un sens comme dans
/// l'autre, la suppression l'emporte : le renommage ne fait pas renaître l'objet effacé, la clé de
/// la requête manque à `requetes_enregistrees`, et `lecture::requetes_enregistrees` ne rend aucune
/// requête, ni nommée sans critère, ni autre. À rebours, le renommage fusionné avec une réplique où
/// rien n'est supprimé, une étiquette posée sur l'idée, rend, dans les deux sens, la requête
/// renommée, ses critères entiers.
#[test]
fn un_renommage_concurrent_d_une_suppression_ne_laisse_aucune_requete_sans_critere() {
    let mut services = PaireEnMemoire.neufs();
    let mut alea = AleaDeTest {
        graine: 0,
        appels: 0,
    };
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note est faite");
    services
        .enregistrer_une_requete(
            &commandes::Enregistrer {
                nom: "Ponts lents".to_string(),
                requete: commandes::Requete {
                    texte: "pont".to_string(),
                    etiquette: Some("lent".to_string()),
                    plus_de_secondes: Some(60),
                    ..commandes::Requete::default()
                },
            },
            &mut contexte,
        )
        .expect("la requête est enregistrée");
    let fonds = services.documents().fonds().expect("le fonds est né");
    let enregistrees = lecture::requetes_enregistrees(fonds);
    let [ponts] = enregistrees.as_slice() else {
        panic!("une seule requête enregistrée : {enregistrees:?}");
    };
    assert_eq!(
        ponts.requete,
        Some(RequeteDuNoyau {
            texte: "pont".to_string(),
            etiquette: Some("lent".to_string()),
            plus_de_secondes: Some(60),
            ..RequeteDuNoyau::default()
        })
    );

    let mut renommee = fonds.fork();
    let renommage =
        renommer_une_requete(ponts, "Ponts du soir", HEURE).expect("le renommage se calcule");
    application::appliquer(&mut renommee, &renommage).expect("le renommage s'applique");
    let mut supprimee = fonds.fork();
    application::appliquer(&mut supprimee, &supprimer_une_requete(ponts, HEURE))
        .expect("la suppression s'applique");
    for (mut l_une, mut l_autre) in [
        (renommee.clone(), supprimee.clone()),
        (supprimee, renommee.clone()),
    ] {
        l_une
            .merge(&mut l_autre)
            .expect("les deux répliques fusionnent");
        let (_, conteneur) = l_une
            .get(ROOT, REQUETES_ENREGISTREES)
            .expect("lire la racine du document")
            .expect("le conteneur des requêtes enregistrées");
        assert!(
            l_une
                .get(&conteneur, ponts.id.as_str())
                .expect("lire le conteneur des requêtes enregistrées")
                .is_none(),
            "la requête supprimée renaît du renommage"
        );
        assert_eq!(
            lecture::requetes_enregistrees(&l_une),
            Vec::<RequeteEnregistreeLue>::new()
        );
    }

    let mut sans_suppression = fonds.fork();
    let idee = lecture::lire_l_idee(&sans_suppression, IDEE).expect("l'idée se lit");
    let pose = gestes::poser_une_etiquette(&idee, "lent", HEURE, 0)
        .expect("la pose se calcule")
        .operation;
    application::appliquer(&mut sans_suppression, &pose).expect("la pose s'applique");
    let attendue = vec![RequeteEnregistreeLue {
        nom: "Ponts du soir".to_string(),
        ..ponts.clone()
    }];
    for (mut l_une, mut l_autre) in [
        (renommee.clone(), sans_suppression.clone()),
        (sans_suppression, renommee),
    ] {
        l_une
            .merge(&mut l_autre)
            .expect("les deux répliques fusionnent");
        assert_eq!(lecture::requetes_enregistrees(&l_une), attendue);
    }
}

/// Un document écrit par le test, dont le conteneur `photos` porte une clé `<empreinte>/idee`
/// qui nomme `SECONDE_IDEE` (`suite::un_document_dont_une_idee_porte_une_photo`), rouvert en
/// mémoire : la requête du contenu « photo » ne trouve que cette idée, et la boîte compte toujours
/// ses deux idées non reliées.
#[test]
fn la_requete_du_contenu_photo_lit_le_conteneur_des_photos() {
    let documents =
        DocumentsEnMemoire::de(&suite::un_document_dont_une_idee_porte_une_photo(), &[])
            .expect("le document s'ouvre");
    let services = Services::new(
        documents,
        CaptationsEnMemoire::default(),
        gestes::Appareil::Etabli,
    );
    let photo = commandes::Boite {
        requete: commandes::Requete {
            contenu: Some(commandes::Contenu::Photo),
            ..commandes::Requete::default()
        },
        ..suite::toute_la_boite()
    };
    let vue = services.boite(&photo).expect("la boîte se lit");
    assert_eq!(
        vue.lignes
            .iter()
            .map(|ligne| ligne.idee.as_str())
            .collect::<Vec<_>>(),
        vec![SECONDE_IDEE]
    );
    assert_eq!(vue.retenues, 1);
    assert_eq!(vue.non_reliees, 2);
}

/// Un document écrit par le test (`suite::un_document_aux_requetes_d_une_version_plus_recente`),
/// rouvert en mémoire : la liste des requêtes enregistrées rend « Église », dont le texte est un
/// objet, puis « Ponts lents », qui porte une sorte de critère inconnue, rangées par nom replié,
/// chacune nommée, sa requête `None`, comme d'une version plus récente de Pupitre (question 17) ;
/// ni la requête sans nom ni la valeur simple n'y paraissent.
#[test]
fn une_requete_d_une_version_plus_recente_parait_nommee_sans_requete() {
    let documents = DocumentsEnMemoire::de(
        &suite::un_document_aux_requetes_d_une_version_plus_recente(),
        &[],
    )
    .expect("le document s'ouvre");
    let services = Services::new(
        documents,
        CaptationsEnMemoire::default(),
        gestes::Appareil::Etabli,
    );
    assert_eq!(
        services.requetes_enregistrees(),
        Ok(vec![
            commandes::RequeteEnregistree {
                id: suite::REQUETE_AU_TEXTE_EN_OBJET.to_string(),
                nom: "Église".to_string(),
                requete: None,
            },
            commandes::RequeteEnregistree {
                id: suite::REQUETE_PLUS_RECENTE.to_string(),
                nom: "Ponts lents".to_string(),
                requete: None,
            },
        ])
    );
}

/// Un document écrit par le test, dont l'idée `SECONDE_IDEE` porte sa date en texte
/// (`suite::un_document_dont_une_idee_est_illisible`), rouvert en mémoire : la boîte ne montre que
/// l'idée qui se lit, et son assiette compte l'autre parmi les considérées et parmi les
/// illisibles, non parmi celles qui portent la donnée ; le texte du compte la dit illisible.
#[test]
fn une_idee_illisible_compte_a_part_dans_l_assiette() {
    let documents = DocumentsEnMemoire::de(&suite::un_document_dont_une_idee_est_illisible(), &[])
        .expect("le document s'ouvre");
    let services = Services::new(
        documents,
        CaptationsEnMemoire::default(),
        gestes::Appareil::Etabli,
    );
    let vue = services
        .boite(&suite::toute_la_boite())
        .expect("la boîte se lit");
    let assiette = &vue.assiette_du_compte;
    assert_eq!(assiette.considerees, assiette.portant + 1);
    assert_eq!(
        *assiette,
        Assiette {
            considerees: 2,
            portant: 1,
            decidees: 1,
            illisibles: 1,
        }
    );
    assert_eq!(
        vue.lignes
            .iter()
            .map(|ligne| ligne.idee.as_str())
            .collect::<Vec<_>>(),
        vec![IDEE]
    );
    let texte = suite::texte_du_compte_de(vue.non_reliees, assiette);
    assert!(texte.contains(" · 1 illisible"), "{texte}");
    assert_eq!(texte, "1 idée non reliée · 1 illisible · 2 interrogées");
}

/// Le générateur du fonds de mesure, sur deux groupes de trois prises, deux fois, chaque fois sur
/// une paire neuve : les mêmes comptes, deux idées visibles et quatre retirées en 23 opérations,
/// et la même teneur ; chaque texte des deux idées visibles fait au moins 200 octets.
#[test]
fn le_generateur_est_deterministe() {
    let engendrer = || {
        let mut services = PaireEnMemoire.neufs();
        let engendre = engendrer_le_fonds_de_mesure_selon(&mut services, HEURE, 2)
            .expect("le fonds de mesure s'engendre");
        let fonds = services.documents().fonds().expect("le fonds est né");
        let teneur = services::lecture::teneur(fonds).expect("la teneur s'écrit");
        (engendre, teneur, services)
    };
    let (engendre, teneur, services) = engendrer();
    let (autre, autre_teneur, _) = engendrer();
    assert_eq!(
        engendre,
        Engendre {
            idees_visibles: 2,
            idees_retirees: 4,
            operations: 23,
        }
    );
    assert_eq!(autre, engendre);
    assert_eq!(autre_teneur, teneur);
    let idees = services.idees().expect("les idées se lisent");
    assert_eq!(idees.len(), 2);
    for idee in &idees {
        let texte = services
            .vue_d_une_idee(idee)
            .expect("la vue de l'idée se lit")
            .texte
            .expect("l'idée a un texte");
        assert!(
            texte.len() >= 200,
            "le texte de l'idée {idee} fait {} octets : {texte}",
            texte.len()
        );
    }
}

/// V7, R20 : une copie en flux dont l'empreinte n'est pas celle qu'attend l'import (un fichier
/// changé depuis l'inspection) ne range rien en mémoire non plus ; l'empreinte calculée est rendue.
#[test]
fn une_copie_en_memoire_qui_ne_donne_pas_l_empreinte_attendue_ne_range_rien() {
    let captations = CaptationsEnMemoire::default();
    let contenu = "une prise versée en flux".as_bytes();
    let e = empreinte(contenu);
    let autre = empreinte("une autre prise".as_bytes());
    let mut flux = contenu;

    let versement = captations
        .verser_en_flux(
            &nom_de_la_copie("01HF7YAT000000000000000004", 1),
            &mut flux,
            Some(&autre),
        )
        .expect("la copie se fait, sans rien ranger");

    assert_eq!(
        versement,
        Versement {
            empreinte: e.clone(),
            octets: contenu.len() as u64,
            deja_rangee: false,
            rangee: false,
        }
    );
    assert_eq!(captations.taille(&e), Ok(None));
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

// Balayage Q8 : en mémoire aussi, un flux refusé en chemin est un refus du disque, au constat
// neutre, et rien ne se range
#[test]
fn une_copie_en_memoire_dont_le_flux_est_refuse_ne_range_rien() {
    let captations = CaptationsEnMemoire::default();
    let contenu = "une prise refusée en chemin".as_bytes();

    let rendu = captations.verser_en_flux(
        &nom_de_la_copie("01HF7YAT000000000000000004", 1),
        &mut FluxRefuse { reste: contenu },
        None,
    );

    assert_eq!(
        rendu,
        Err(Refus {
            raison: Raison::Disque,
            message: "Le fichier à verser, le disque l'a refusé.".to_string(),
        })
    );
    assert_eq!(captations.taille(&empreinte(contenu)), Ok(None));
    assert_eq!(captations.tailles(), Ok(BTreeSet::new()));
}

// La source à verser en mémoire : un rang sans entrée et un lien ne se lisent pas, une plage se
// borne au contenu, une entrée refusée l'est tant qu'elle l'est
#[test]
fn la_source_en_memoire_refuse_un_rang_sans_entree_un_lien_et_une_entree_refusee() {
    let contenu: Vec<u8> = (0..=255u8).collect();
    let mut source = SourceEnMemoire::new(
        Some("Démos"),
        vec![(Some("a/prise.wav".to_string()), contenu.clone())],
        vec!["lien".to_string()],
    );
    let sans_entree = |rang: u64| Refus {
        raison: Raison::Impossible,
        message: format!("Le dossier versé n'a pas d'entrée n° {rang}."),
    };

    assert_eq!(source.lire(1, 250, 10), Ok(contenu[250..].to_vec()));
    assert_eq!(source.lire(1, 300, 4), Ok(Vec::new()));
    assert_eq!(source.octets_lus(1), 6);
    assert_eq!(source.lire(3, 0, 1), Err(sans_entree(3)));
    assert_eq!(source.lire(0, 0, 1), Err(sans_entree(0)));
    assert_eq!(source.octets_actuels(3), Err(sans_entree(3)));
    assert_eq!(source.marque_actuelle(3), Err(sans_entree(3)));
    assert_eq!(source.octets_lus(3), 0);
    match source.lire_en_flux(2) {
        Err(refus) => assert_eq!(
            refus,
            Refus {
                raison: Raison::Impossible,
                message:
                    "L'entrée n° 2 du dossier versé n'est pas un fichier : elle ne se lit pas."
                        .to_string(),
            }
        ),
        Ok(_) => panic!("le lien se lit en flux"),
    }

    source.refuser(1, true);
    assert_eq!(
        source.lire(1, 0, 4),
        Err(Refus {
            raison: Raison::Disque,
            message: "Le fichier à verser, le disque l'a refusé.".to_string(),
        })
    );
    source.refuser(1, false);
    assert_eq!(source.lire(1, 0, 4), Ok(contenu[..4].to_vec()));
}
