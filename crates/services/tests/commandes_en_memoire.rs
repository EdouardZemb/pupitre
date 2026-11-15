//! La suite des commandes sur la paire d'adaptateurs en mémoire : chaque test de la suite,
//! rejoué sur `DocumentsEnMemoire` et `CaptationsEnMemoire` ; puis, sur cette paire seule, les
//! services du carnet, une idée illisible dans l'assiette de la boîte et le générateur du fonds
//! de mesure.

mod suite;

use commandes::Assiette;
use noyau::donnees::gestes;
use noyau::donnees::version::Migration;
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::mesure::{Engendre, engendrer_le_fonds_de_mesure_selon};
use services::{Documents, Services};
use suite::{AleaDeTest, HEURE, IDEE, Paire, contexte_de_test, une_note};

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

/// Un document écrit par le test, dont l'idée `SECONDE_IDEE` porte sa date en texte
/// (`suite::un_document_dont_une_idee_est_illisible`), rouvert en mémoire : la boîte ne montre que
/// l'idée qui se lit, et son assiette compte l'autre parmi les considérées, non parmi celles qui
/// portent la donnée ; le texte du compte la dit illisible.
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
