//! La suite des commandes sur la paire d'adaptateurs en mémoire : chaque test de la suite,
//! rejoué sur `DocumentsEnMemoire` et `CaptationsEnMemoire` ; puis, sur cette paire seule, les
//! services du carnet.

mod suite;

use noyau::donnees::gestes;
use noyau::donnees::version::Migration;
use services::Services;
use services::en_memoire::{CaptationsEnMemoire, DocumentsEnMemoire};
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
    suite::une_migration_qui_attend_ses_dependances_laisse_le_fonds_en_lecture_seule(
        &mut PaireEnMemoire,
    );
}
