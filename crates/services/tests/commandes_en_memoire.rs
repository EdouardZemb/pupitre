//! La suite des commandes sur la paire d'adaptateurs en mémoire : chaque test de la suite,
//! rejoué sur `DocumentsEnMemoire` et `CaptationsEnMemoire` ; puis, sur cette paire seule, les
//! services du carnet.

mod suite;

use noyau::donnees::gestes;
use noyau::donnees::version::Migration;
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
use services::{Documents, Services};
use suite::{AleaDeTest, IDEE, Paire, contexte_de_test, une_note};

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
fn la_vue_tire_reliee_et_la_duree_des_regles_du_noyau() {
    suite::la_vue_tire_reliee_et_la_duree_des_regles_du_noyau(&mut PaireEnMemoire);
}
