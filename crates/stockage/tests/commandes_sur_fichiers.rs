//! La suite des commandes sur la paire d'adaptateurs sur fichiers : chaque test de la suite,
//! rejoué sur `FondsSurFichiers` et les `Captations` d'une racine des données jetable ; puis, sur
//! cette paire seule, des commandes coupées par une panne, avant ou après le renommage de leur
//! fichier, la première comprise, qui fait naître le fonds, une annulation refusée par le disque,
//! qui se refait, un geste de tri refusé par le disque, qui ne change pas ce qu'annuler vise, un
//! geste de tri et son annulation coupés après le renommage, faits avec un avertissement, un
//! fichier d'ajout altéré et une captation coupée, chaque refus ou avertissement disant ses
//! chemins relatifs à la racine des données, l'état du fonds qui dit sa lecture seule et
//! l'avertissement de sa naissance, de même, la boîte qui se lit en lecture seule, la migration
//! de test, la même dans la suite et dans les aides du stockage ; et, de l'import, une reprise
//! après une copie coupée entre deux tronçons, la clôture qui retire les copies de l'import, une
//! ouverture coupée après la clôture d'un import ouvert ailleurs, qui ne le reclôt pas, et une
//! clôture dont le retrait d'une copie est refusé, faite avec un avertissement. La source à
//! verser de la paire est un dossier jetable, écrit par le test, puis parcouru (`SourceDeTest`).

#[path = "../../services/tests/suite/mod.rs"]
mod suite;

mod commun;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use commandes::{
    Appareil, EntreeDuJournal, Faite, LigneDeLaBoite, Notee, Provenance, RapportDImport,
    VerserUnFichier,
};
use noyau::donnees::acces::Acces;
use noyau::donnees::gestes;
use noyau::donnees::import::{copie_de, nom_de_la_copie};
use noyau::donnees::version::Migration;
use services::{Documents, Entree, Raison, Refus, Services, Source, lecture};
use stockage::pannes::{Etape, couper};
use stockage::source::SourceSurFichiers;
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
    type S = SourceDeTest;

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

    /// Écrit le dossier dans son propre dossier jetable, chaque fichier à son chemin, un chemin
    /// `None` sous un nom sans forme UTF-8 (`nom_sans_forme_utf8`), chaque lien vers un fichier
    /// absent ; puis le parcourt (`SourceSurFichiers::parcourir`).
    fn source(
        &mut self,
        nom_du_dossier: Option<&str>,
        fichiers: &[(Option<&str>, Vec<u8>)],
        liens: &[&str],
    ) -> SourceDeTest {
        let rang = SOURCES.fetch_add(1, Ordering::Relaxed);
        let jetable = commun::dossier_jetable(&format!("commandes-source-{rang}"));
        let dossier = match nom_du_dossier {
            Some(nom) => jetable.join(nom),
            None => jetable.join(nom_sans_forme_utf8(0)),
        };
        fs::create_dir_all(&dossier)
            .unwrap_or_else(|e| panic!("créer {} : {e}", dossier.display()));
        let mut sans_nom = Vec::new();
        for (chemin, contenu) in fichiers {
            let ecrit = match chemin {
                Some(chemin) => dossier.join(chemin),
                None => {
                    let rang = u8::try_from(sans_nom.len()).expect("peu de fichiers sans nom");
                    let ecrit = dossier.join(nom_sans_forme_utf8(rang));
                    sans_nom.push(ecrit.clone());
                    ecrit
                }
            };
            creer_le_parent(&ecrit);
            fs::write(&ecrit, contenu)
                .unwrap_or_else(|e| panic!("écrire {} : {e}", ecrit.display()));
        }
        for lien in liens {
            let chemin = dossier.join(lien);
            creer_le_parent(&chemin);
            lier(&dossier.join("cible absente"), &chemin);
        }
        let source = SourceSurFichiers::parcourir(&dossier).expect("parcourir la source à verser");
        let mut sans_nom = sans_nom.into_iter();
        let chemins = source
            .entrees()
            .iter()
            .map(|entree| match &entree.chemin {
                Some(chemin) => dossier.join(chemin),
                None => sans_nom
                    .next()
                    .expect("chaque entrée sans nom est un fichier écrit sans nom"),
            })
            .collect();
        SourceDeTest {
            source,
            chemins,
            ouverts: BTreeMap::new(),
        }
    }

    /// Récrit le fichier, puis le date d'une seconde après sa marque de dernière écriture d'avant
    /// (`File::set_modified`, dans le test seul) : le changement ne dépend pas de la finesse de
    /// l'horloge du système de fichiers.
    fn changer(&mut self, source: &mut SourceDeTest, rang: u64, contenu: &[u8]) {
        let chemin = source.chemin(rang).to_path_buf();
        let avant = fs::metadata(&chemin)
            .and_then(|metadonnees| metadonnees.modified())
            .unwrap_or_else(|e| panic!("lire la date de {} : {e}", chemin.display()));
        fs::write(&chemin, contenu)
            .unwrap_or_else(|e| panic!("récrire {} : {e}", chemin.display()));
        File::options()
            .write(true)
            .open(&chemin)
            .and_then(|fichier| fichier.set_modified(avant + Duration::from_secs(1)))
            .unwrap_or_else(|e| panic!("dater {} : {e}", chemin.display()));
    }

    /// Refuse la lecture du fichier (`refuser`), puis vérifie qu'il ne s'ouvre plus, et sinon
    /// panique en le disant (V14) ; ou la rend (`rendre`).
    fn refuser_la_lecture(&mut self, source: &mut SourceDeTest, rang: u64, refusee: bool) {
        let chemin = source.chemin(rang).to_path_buf();
        if refusee {
            if let Some(ouvert) = refuser(&chemin) {
                source.ouverts.insert(rang, ouvert);
            }
            if File::open(&chemin).is_ok() {
                panic!(
                    "les droits ôtés ne refusent pas la lecture : le test tourne-t-il sous root ?"
                );
            }
        } else {
            source.ouverts.remove(&rang);
            rendre(&chemin);
        }
    }
}

/// Le rang de la prochaine source à verser des tests sur fichiers : chacune a son dossier jetable.
static SOURCES: AtomicU64 = AtomicU64::new(0);

/// La source à verser des tests sur fichiers : le dossier écrit par le test, parcouru par
/// `SourceSurFichiers`, à qui chaque lecture est confiée ; le chemin de chaque entrée, dans l'ordre
/// des rangs, pour la changer ou en refuser la lecture ; et chaque fichier gardé ouvert sans
/// partage, sous Windows, dont la lecture est refusée.
struct SourceDeTest {
    source: SourceSurFichiers,
    chemins: Vec<PathBuf>,
    ouverts: BTreeMap<u64, File>,
}

impl SourceDeTest {
    /// Le chemin de l'entrée `rang` ; un rang sans entrée est une faute du test.
    fn chemin(&self, rang: u64) -> &Path {
        let indice = rang
            .checked_sub(1)
            .and_then(|indice| usize::try_from(indice).ok())
            .expect("un rang compté dès 1");
        &self.chemins[indice]
    }
}

impl Source for SourceDeTest {
    fn nom_du_dossier(&self) -> Option<&str> {
        self.source.nom_du_dossier()
    }

    fn entrees(&self) -> &[Entree] {
        self.source.entrees()
    }

    fn lire(&self, rang: u64, debut: u64, longueur: usize) -> Result<Vec<u8>, Refus> {
        self.source.lire(rang, debut, longueur)
    }

    fn lire_en_flux(&self, rang: u64) -> Result<Box<dyn io::Read + '_>, Refus> {
        self.source.lire_en_flux(rang)
    }

    fn octets_actuels(&self, rang: u64) -> Result<u64, Refus> {
        self.source.octets_actuels(rang)
    }

    fn marque_actuelle(&self, rang: u64) -> Result<Option<u64>, Refus> {
        self.source.marque_actuelle(rang)
    }

    fn octets_lus(&self, rang: u64) -> u64 {
        self.source.octets_lus(rang)
    }
}

/// Crée le dossier parent de `chemin`, et chacun des siens.
fn creer_le_parent(chemin: &Path) {
    if let Some(parent) = chemin.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| panic!("créer {} : {e}", parent.display()));
    }
}

/// Un nom sans forme UTF-8, distinct pour chaque `rang` : une demi-paire de substitution seule
/// (`OsString::from_wide`), suivie d'un caractère qui dit le rang.
#[cfg(windows)]
fn nom_sans_forme_utf8(rang: u8) -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0xD800, u16::from(b'0' + rang)])
}

/// Un nom sans forme UTF-8, distinct pour chaque `rang` : l'octet `0xFF` (`OsStr::from_bytes`),
/// suivi d'un caractère qui dit le rang.
#[cfg(unix)]
fn nom_sans_forme_utf8(rang: u8) -> OsString {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(&[0xFF, b'0' + rang]).to_os_string()
}

/// Un lien symbolique `chemin` vers `cible`.
#[cfg(windows)]
fn lier(cible: &Path, chemin: &Path) {
    std::os::windows::fs::symlink_file(cible, chemin)
        .unwrap_or_else(|e| panic!("créer le lien {} : {e}", chemin.display()));
}

/// Un lien symbolique `chemin` vers `cible`.
#[cfg(unix)]
fn lier(cible: &Path, chemin: &Path) {
    std::os::unix::fs::symlink(cible, chemin)
        .unwrap_or_else(|e| panic!("créer le lien {} : {e}", chemin.display()));
}

/// Refuse la lecture de `chemin` : le fichier ouvert sans partage (`OpenOptionsExt::share_mode`),
/// rendu pour être gardé ; tant qu'il l'est, nulle autre ouverture ne réussit.
#[cfg(windows)]
fn refuser(chemin: &Path) -> Option<File> {
    use std::os::windows::fs::OpenOptionsExt;
    Some(
        File::options()
            .read(true)
            .share_mode(0)
            .open(chemin)
            .unwrap_or_else(|e| panic!("ouvrir {} sans partage : {e}", chemin.display())),
    )
}

/// Rend la lecture de `chemin` : le fichier gardé ouvert sans partage s'est refermé.
#[cfg(windows)]
fn rendre(_chemin: &Path) {}

/// Refuse la lecture de `chemin` : ses droits ôtés (`0o000`).
#[cfg(unix)]
fn refuser(chemin: &Path) -> Option<File> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(chemin, fs::Permissions::from_mode(0o000))
        .unwrap_or_else(|e| panic!("ôter les droits de {} : {e}", chemin.display()));
    None
}

/// Rend la lecture de `chemin` : ses droits rendus (`0o644`).
#[cfg(unix)]
fn rendre(chemin: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(chemin, fs::Permissions::from_mode(0o644))
        .unwrap_or_else(|e| panic!("rendre les droits de {} : {e}", chemin.display()));
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
fn chaque_idee_d_un_geste_se_date_a_son_propre_decalage() {
    let racine = racine_jetable("commandes-decalage-de-chaque-idee");
    suite::chaque_idee_d_un_geste_se_date_a_son_propre_decalage(&mut PaireSurFichiers {
        racine: &racine,
    });
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
fn regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse() {
    let racine = racine_jetable("commandes-empreinte-sans-captation");
    suite::regrouper_une_empreinte_qu_aucune_captation_ne_porte_est_refuse(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn une_cle_d_une_autre_forme_sous_les_liens_n_est_pas_un_lien() {
    let racine = racine_jetable("commandes-cle-d-une-autre-forme-sous-les-liens");
    suite::une_cle_d_une_autre_forme_sous_les_liens_n_est_pas_un_lien(&mut PaireSurFichiers {
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
fn cap02_6_annuler_rend_le_fonds_d_avant_et_le_journal_porte_le_geste_puis_son_annulation() {
    let racine = racine_jetable("commandes-cap02-6");
    suite::annuler_rend_le_fonds_d_avant(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn annuler_ne_vise_que_le_dernier_geste_de_tri_depuis_l_ouverture() {
    let racine = racine_jetable("commandes-suite-des-annulations");
    suite::annuler_ne_vise_que_le_dernier_geste_de_tri_depuis_l_ouverture(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_geste_refuse_ne_change_pas_ce_qu_annuler_vise() {
    let racine = racine_jetable("commandes-geste-refuse-et-annuler");
    suite::un_geste_refuse_ne_change_pas_ce_qu_annuler_vise(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn la_reouverture_vide_ce_qu_annuler_vise() {
    let racine = racine_jetable("commandes-reouverture-et-annuler");
    suite::la_reouverture_vide_ce_qu_annuler_vise(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_annulation_ne_s_annule_pas() {
    let racine = racine_jetable("commandes-annulation-non-annulee");
    suite::une_annulation_ne_s_annule_pas(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn chaque_operation_laisse_une_seule_modification_avec_son_message() {
    let racine = racine_jetable("commandes-une-modification-par-operation");
    suite::chaque_operation_laisse_une_seule_modification_avec_son_message(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn l_historique_ne_porte_que_des_operations_connues() {
    let racine = racine_jetable("commandes-operations-connues");
    suite::l_historique_ne_porte_que_des_operations_connues(&mut PaireSurFichiers {
        racine: &racine,
    });
}

// § 31 · 10, test 7 : chercher les champs interdits du § 30 · 13
#[test]
fn rien_d_engendre_n_est_stocke() {
    let racine = racine_jetable("commandes-rien-d-engendre");
    suite::rien_d_engendre_n_est_stocke(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn aucun_counter_ni_objet_text_dans_un_document_des_operations() {
    let racine = racine_jetable("commandes-ni-compteur-ni-texte");
    suite::aucun_counter_ni_objet_text_dans_un_document_des_operations(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap02_1_aucun_compteur_d_anciennete() {
    let racine = racine_jetable("commandes-cap02-1");
    suite::aucun_compteur_d_anciennete(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap02_3_la_boite_montre_ensemble_deux_prises_regroupees() {
    let racine = racine_jetable("commandes-cap02-3-la-boite");
    suite::deux_prises_regroupees_paraissent_ensemble_dans_la_boite(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap02_5_retirer_ote_l_idee_de_la_boite_et_retablir_l_y_rend() {
    let racine = racine_jetable("commandes-cap02-5-la-boite");
    suite::retirer_ote_l_idee_de_la_boite_et_retablir_l_y_rend(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap07_4_le_nombre_d_idees_non_reliees_suit_chaque_geste() {
    let racine = racine_jetable("commandes-cap07-4");
    suite::le_nombre_d_idees_non_reliees_suit_chaque_geste(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn i31_un_texte_piege_revient_tel_quel() {
    let racine = racine_jetable("commandes-i31-texte-piege");
    suite::un_texte_piege_revient_tel_quel(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop() {
    let racine = racine_jetable("commandes-fenetre-hors-du-fonds");
    suite::une_fenetre_hors_du_fonds_rend_les_comptes_et_aucune_ligne_de_trop(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn la_boite_d_un_pupitre_neuf() {
    let racine = racine_jetable("commandes-boite-d-un-pupitre-neuf");
    suite::la_boite_d_un_pupitre_neuf(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_ligne_borne_son_texte() {
    let racine = racine_jetable("commandes-ligne-bornee");
    suite::une_ligne_borne_son_texte(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn l_etat_du_fonds_dit_le_geste_qu_annuler_vise() {
    let racine = racine_jetable("commandes-etat-geste-a-annuler");
    suite::l_etat_du_fonds_dit_le_geste_qu_annuler_vise(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn les_retirees_se_rangent_par_la_date_du_retrait() {
    let racine = racine_jetable("commandes-retirees-par-date");
    suite::les_retirees_se_rangent_par_la_date_du_retrait(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn la_requete_passe_au_noyau() {
    let racine = racine_jetable("commandes-requete");
    let racine_des_notes = racine_jetable("commandes-requete-des-notes");
    suite::la_requete_passe_au_noyau(
        &mut PaireSurFichiers { racine: &racine },
        &mut PaireSurFichiers {
            racine: &racine_des_notes,
        },
    );
}

#[test]
fn cap06_7_plus_d_une_minute_declare_son_assiette() {
    let racine = racine_jetable("commandes-cap06-7");
    suite::plus_d_une_minute_declare_son_assiette(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap06_8_une_idee_retiree_n_est_ni_trouvee_ni_comptee() {
    let racine = racine_jetable("commandes-cap06-8");
    suite::une_idee_retiree_n_est_ni_trouvee_ni_comptee(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap02_5_une_idee_retiree_quitte_les_requetes() {
    let racine = racine_jetable("commandes-cap02-5-les-requetes");
    suite::une_idee_retiree_quitte_les_requetes(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap06_9_une_prise_sans_duree_compte_parmi_les_sans_duree() {
    let racine = racine_jetable("commandes-cap06-9");
    suite::une_prise_sans_duree_compte_parmi_les_sans_duree(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap06_10_eglise_trouve_eglise_du_port() {
    let racine = racine_jetable("commandes-cap06-10");
    suite::eglise_trouve_eglise_du_port(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn i09_chaque_reponse_declare_son_assiette() {
    let racine = racine_jetable("commandes-i09");
    suite::chaque_reponse_declare_son_assiette(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn i13_l_index_reconstruit_rend_les_memes_reponses() {
    let racine = racine_jetable("commandes-i13");
    suite::l_index_reconstruit_rend_les_memes_reponses(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn chercher_n_ecrit_rien() {
    let racine = racine_jetable("commandes-chercher-n-ecrit-rien");
    suite::chercher_n_ecrit_rien(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn noter_pendant_une_requete_la_laisse_en_place() {
    let racine = racine_jetable("commandes-noter-sous-une-requete");
    suite::noter_pendant_une_requete_la_laisse_en_place(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn enregistrer_renommer_et_supprimer_une_requete() {
    let racine = racine_jetable("commandes-requetes-enregistrees");
    suite::enregistrer_renommer_et_supprimer_une_requete(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_requete_enregistree_se_relit_critere_pour_critere() {
    let racine = racine_jetable("commandes-requete-critere-pour-critere");
    suite::une_requete_enregistree_se_relit_critere_pour_critere(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait() {
    let racine = racine_jetable("commandes-requete-pupitre-neuf");
    suite::enregistrer_dans_un_pupitre_neuf_est_refuse_et_rien_ne_nait(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn en_lecture_seule_chercher_sert_et_enregistrer_est_refuse() {
    let racine = racine_jetable("commandes-requete-lecture-seule");
    suite::en_lecture_seule_chercher_sert_et_enregistrer_est_refuse(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn i01_une_requete_enregistree_ne_garde_ni_resultats_ni_nombre() {
    let racine = racine_jetable("commandes-requete-i01");
    let autre_racine = racine_jetable("commandes-requete-i01-autre");
    suite::une_requete_enregistree_ne_garde_ni_resultats_ni_nombre(
        &mut PaireSurFichiers { racine: &racine },
        &mut PaireSurFichiers {
            racine: &autre_racine,
        },
    );
}

#[test]
fn le_journal_se_lit_en_fenetre() {
    let racine = racine_jetable("commandes-journal-en-fenetre");
    suite::le_journal_se_lit_en_fenetre(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn i01_un_fonds_importe_ne_garde_ni_compte_ni_total() {
    let racine = racine_jetable("commandes-import-i01");
    suite::un_fonds_importe_ne_garde_ni_compte_ni_total(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn la_duree_d_une_prise_en_stems_vient_de_ses_stems() {
    let racine = racine_jetable("commandes-duree-prise-en-stems");
    suite::la_duree_d_une_prise_en_stems_vient_de_ses_stems(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn les_noms_d_un_contenu_et_son_premier_nom_vu() {
    let racine = racine_jetable("commandes-noms-premier-vu");
    suite::les_noms_d_un_contenu_et_son_premier_nom_vu(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn la_fenetre_nomme_chaque_ligne_sans_texte_comme_ligne_par_ligne() {
    let racine = racine_jetable("commandes-fenetre-noms");
    suite::la_fenetre_nomme_chaque_ligne_sans_texte_comme_ligne_par_ligne(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn les_noms_poses_par_un_import() {
    let racine = racine_jetable("commandes-noms-poses");
    suite::les_noms_poses_par_un_import(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn un_nom_pose_hors_du_jeu_n_est_pas_parmi_ceux_du_stem() {
    let racine = racine_jetable("commandes-nom-hors-du-jeu");
    suite::un_nom_pose_hors_du_jeu_n_est_pas_parmi_ceux_du_stem(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap03_1_quatre_vingts_fichiers_dont_trois_doublons_donnent_soixante_dix_sept_captations() {
    let racine = racine_jetable("commandes-cap03-1");
    suite::cap03_1_quatre_vingts_fichiers_dont_trois_doublons_donnent_soixante_dix_sept_captations(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn cap03_2_le_meme_dossier_importe_deux_fois_n_ajoute_que_son_rapport() {
    let racine = racine_jetable("commandes-cap03-2");
    suite::cap03_2_le_meme_dossier_importe_deux_fois_n_ajoute_que_son_rapport(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn cap03_3_un_nom_d_origine_se_relit_intact() {
    let racine = racine_jetable("commandes-cap03-3");
    suite::cap03_3_un_nom_d_origine_se_relit_intact(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn cap03_4_des_noms_semblables_ne_creent_ni_lien_ni_regroupement() {
    let racine = racine_jetable("commandes-cap03-4");
    suite::cap03_4_des_noms_semblables_ne_creent_ni_lien_ni_regroupement(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap03_7_un_jeu_de_dix_wav_et_neuf_midi_donne_une_prise_en_stems() {
    let racine = racine_jetable("commandes-cap03-7");
    suite::cap03_7_un_jeu_de_dix_wav_et_neuf_midi_donne_une_prise_en_stems(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap03_8_trois_wav_de_meme_duree_proposent_un_jeu_et_sans_declaration_trois_prises() {
    let racine = racine_jetable("commandes-cap03-8");
    suite::cap03_8_trois_wav_de_meme_duree_proposent_un_jeu_et_sans_declaration_trois_prises(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn un_premier_import_ne_lit_que_signatures_et_en_tetes() {
    let racine = racine_jetable("commandes-import-en-tetes");
    suite::un_premier_import_ne_lit_que_signatures_et_en_tetes(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_fichier_qui_ne_se_lit_pas_arrete_l_inspection() {
    let racine = racine_jetable("commandes-import-illisible");
    suite::un_fichier_qui_ne_se_lit_pas_arrete_l_inspection(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_fichier_refuse_au_versement_se_nomme() {
    let racine = racine_jetable("commandes-import-refuse-au-versement");
    suite::un_fichier_refuse_au_versement_se_nomme(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn un_jeu_d_un_seul_contenu_donne_une_prise_en_stems_distincte() {
    let racine = racine_jetable("commandes-import-jeu-seul");
    suite::un_jeu_d_un_seul_contenu_donne_une_prise_en_stems_distincte(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap03_10_un_contenu_sous_deux_noms_dans_deux_sous_dossiers_est_un_objet_a_deux_noms() {
    let racine = racine_jetable("commandes-cap03-10");
    suite::cap03_10_un_contenu_sous_deux_noms_dans_deux_sous_dossiers_est_un_objet_a_deux_noms(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn cap03_11_une_archive_et_un_pdf_sont_ignores_sans_etre_ouverts() {
    let racine = racine_jetable("commandes-cap03-11");
    suite::cap03_11_une_archive_et_un_pdf_sont_ignores_sans_etre_ouverts(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn cap03_13_une_taille_a_verser_plus_grande_que_la_place_libre_ne_verse_rien() {
    let racine = racine_jetable("commandes-cap03-13");
    suite::cap03_13_une_taille_a_verser_plus_grande_que_la_place_libre_ne_verse_rien(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn deux_copies_d_un_contenu_versees_dans_le_desordre_comptent_un_verse() {
    let racine = racine_jetable("commandes-import-desordre");
    suite::deux_copies_d_un_contenu_versees_dans_le_desordre_comptent_un_verse(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn un_zip_nomme_wav_est_ignore_et_lu_de_seize_octets() {
    let racine = racine_jetable("commandes-import-zip");
    suite::un_zip_nomme_wav_est_ignore_et_lu_de_seize_octets(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_stem_deja_prise_reste_la_prise_de_son_idee() {
    let racine = racine_jetable("commandes-import-stem-prise");
    suite::un_stem_deja_prise_reste_la_prise_de_son_idee(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn un_jeu_declare_genere_porte_son_origine() {
    let racine = racine_jetable("commandes-import-genere");
    suite::un_jeu_declare_genere_porte_son_origine(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn la_place_libre_rendue_se_borne_a_entier_sur() {
    let racine = racine_jetable("commandes-import-entier-sur");
    suite::la_place_libre_rendue_se_borne_a_entier_sur(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn un_texte_illisible_est_verse_et_compte_a_part() {
    let racine = racine_jetable("commandes-import-texte");
    suite::un_texte_illisible_est_verse_et_compte_a_part(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn un_fichier_sans_nom_est_verse_et_compte_a_part() {
    let racine = racine_jetable("commandes-import-sans-nom");
    suite::un_fichier_sans_nom_est_verse_et_compte_a_part(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn la_place_compte_deux_fois_le_poids_du_fonds() {
    let racine = racine_jetable("commandes-import-poids");
    suite::la_place_compte_deux_fois_le_poids_du_fonds(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn l_import_dans_un_pupitre_neuf_fait_naitre_le_fonds_par_son_ouverture() {
    let racine = racine_jetable("commandes-import-naissance");
    suite::l_import_dans_un_pupitre_neuf_fait_naitre_le_fonds_par_son_ouverture(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn cap03_9_une_origine_generee_se_garde_et_ne_se_retire_pas() {
    let racine = racine_jetable("commandes-cap03-9");
    suite::cap03_9_une_origine_generee_se_garde_et_ne_se_retire_pas(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn declarer_une_origine_par_prefixe_ne_prend_que_les_segments_entiers() {
    let racine = racine_jetable("commandes-import-prefixe");
    suite::declarer_une_origine_par_prefixe_ne_prend_que_les_segments_entiers(
        &mut PaireSurFichiers { racine: &racine },
    );
}

#[test]
fn declarer_par_prefixe_apres_un_import_rejoue() {
    let racine = racine_jetable("commandes-import-prefixe-rejoue");
    suite::declarer_par_prefixe_apres_un_import_rejoue(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn declarer_par_prefixe_prend_la_prise_en_stems_de_son_dossier() {
    let racine = racine_jetable("commandes-import-prefixe-jeu");
    suite::declarer_par_prefixe_prend_la_prise_en_stems_de_son_dossier(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn declarer_par_le_prefixe_d_un_dossier_sans_captation_est_refuse() {
    let racine = racine_jetable("commandes-import-prefixe-sans-captation");
    suite::declarer_par_le_prefixe_d_un_dossier_sans_captation_est_refuse(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn garder_puis_choisir_le_type_d_un_stem() {
    let racine = racine_jetable("commandes-import-type");
    suite::garder_puis_choisir_le_type_d_un_stem(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn lire_un_texte_verse_le_rend_tel_quel() {
    let racine = racine_jetable("commandes-import-texte-lu");
    suite::lire_un_texte_verse_le_rend_tel_quel(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn lire_un_rapport_rend_ses_comptes() {
    let racine = racine_jetable("commandes-import-rapport");
    suite::lire_un_rapport_rend_ses_comptes(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn les_generateurs_du_fonds_chacun_une_fois() {
    let racine = racine_jetable("commandes-import-generateurs");
    suite::les_generateurs_du_fonds_chacun_une_fois(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_reprise_rend_le_rapport_d_une_traite() {
    let racine = racine_jetable("commandes-reprise");
    let racine_neuve = racine_jetable("commandes-reprise-traite");
    suite::une_reprise_rend_le_rapport_d_une_traite(
        &mut PaireSurFichiers { racine: &racine },
        &mut PaireSurFichiers {
            racine: &racine_neuve,
        },
    );
}

#[test]
fn un_import_coupe_apres_une_prise_en_stems_se_reprend() {
    let racine = racine_jetable("commandes-reprise-jeu");
    let racine_neuve = racine_jetable("commandes-reprise-jeu-traite");
    suite::un_import_coupe_apres_une_prise_en_stems_se_reprend(
        &mut PaireSurFichiers { racine: &racine },
        &mut PaireSurFichiers {
            racine: &racine_neuve,
        },
    );
}

#[test]
fn une_reprise_dont_la_prise_n_a_pas_de_nom_reverse_le_jeu() {
    let racine = racine_jetable("commandes-reprise-prise-sans-nom");
    let racine_neuve = racine_jetable("commandes-reprise-prise-sans-nom-traite");
    suite::une_reprise_dont_la_prise_n_a_pas_de_nom_reverse_le_jeu(
        &mut PaireSurFichiers { racine: &racine },
        &mut PaireSurFichiers {
            racine: &racine_neuve,
        },
    );
}

#[test]
fn une_reprise_d_une_prise_connue_au_stem_renomme_reverse_le_jeu() {
    let racine = racine_jetable("commandes-reprise-stem-renomme");
    let racine_neuve = racine_jetable("commandes-reprise-stem-renomme-traite");
    suite::une_reprise_d_une_prise_connue_au_stem_renomme_reverse_le_jeu(
        &mut PaireSurFichiers { racine: &racine },
        &mut PaireSurFichiers {
            racine: &racine_neuve,
        },
    );
}

#[test]
fn un_import_ouvert_sans_nom_ne_se_reprend_pas() {
    let racine = racine_jetable("commandes-reprise-sans-nom");
    suite::un_import_ouvert_sans_nom_ne_se_reprend_pas(&mut PaireSurFichiers { racine: &racine });
}

#[test]
fn une_reprise_garde_les_declarations_de_l_ouverture() {
    let racine = racine_jetable("commandes-reprise-declarations");
    suite::une_reprise_garde_les_declarations_de_l_ouverture(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_fichier_change_depuis_l_inspection_arrete_l_import() {
    let racine = racine_jetable("commandes-import-fichier-change");
    suite::un_fichier_change_depuis_l_inspection_arrete_l_import(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_contenu_change_a_taille_egale_arrete_l_import() {
    let racine = racine_jetable("commandes-import-contenu-change");
    suite::un_contenu_change_a_taille_egale_arrete_l_import(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_import_ouvert_ailleurs_se_clot_a_l_ouverture_du_suivant() {
    let racine = racine_jetable("commandes-import-ailleurs");
    suite::un_import_ouvert_ailleurs_se_clot_a_l_ouverture_du_suivant(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn un_import_clos_ailleurs_se_compte_par_la_regle_du_noyau() {
    let racine = racine_jetable("commandes-import-ailleurs-comptes");
    suite::un_import_clos_ailleurs_se_compte_par_la_regle_du_noyau(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn en_lecture_seule_l_import_est_refuse_avant_toute_captation() {
    let racine = racine_jetable("commandes-import-lecture-seule");
    suite::en_lecture_seule_l_import_est_refuse_avant_toute_captation(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn l_etat_du_fonds_dit_qu_un_import_est_reste_ouvert() {
    let racine = racine_jetable("commandes-import-reste-ouvert");
    suite::l_etat_du_fonds_dit_qu_un_import_est_reste_ouvert(&mut PaireSurFichiers {
        racine: &racine,
    });
}

#[test]
fn rechoisir_un_autre_dossier_ne_reprend_pas_l_import_ouvert() {
    let racine = racine_jetable("commandes-reprise-autre-dossier");
    suite::rechoisir_un_autre_dossier_ne_reprend_pas_l_import_ouvert(&mut PaireSurFichiers {
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
    let message = match rendu {
        Err(Refus {
            raison: Raison::Disque,
            message,
        }) => message,
        autre => panic!("attendu le refus du disque ; rendu {autre:?}"),
    };
    // Le refus nomme le fichier d'ajout par son chemin relatif à la racine des données, ses
    // composants joints par `/`, sous Linux comme sous Windows : le chemin de la racine des
    // données n'y paraît pas.
    assert!(message.starts_with("documents/fonds/ajouts/"), "{message}");
    assert!(
        message.ends_with(".automerge, le disque l'a refusé."),
        "{message}"
    );
    assert!(
        !message.contains(&*racine.chemin().to_string_lossy()),
        "{message}"
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
    let message = match rendu {
        Ok(Notee {
            idee,
            avertissement: Some(message),
        }) => {
            assert_eq!(idee, SECONDE_IDEE);
            message
        }
        autre => panic!("attendu la note faite, avec un avertissement ; rendu {autre:?}"),
    };
    // L'avertissement nomme le fichier d'ajout par son chemin relatif à la racine des données,
    // ses composants joints par `/` : le chemin de la racine des données n'y paraît pas.
    assert!(message.starts_with("documents/fonds/ajouts/"), "{message}");
    assert!(
        message.ends_with(".automerge est en place, mais son renommage n'est pas rendu durable."),
        "{message}"
    );
    assert!(
        !message.contains(&*racine.chemin().to_string_lossy()),
        "{message}"
    );

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
    let message = match rendu {
        Ok(Notee {
            idee,
            avertissement: Some(message),
        }) => {
            assert_eq!(idee, IDEE);
            message
        }
        autre => {
            panic!("attendu la note faite, avec l'avertissement de la naissance ; rendu {autre:?}")
        }
    };
    // L'avertissement nomme le fichier complet par son chemin relatif à la racine des données :
    // le chemin de la racine des données n'y paraît pas.
    assert_eq!(
        message,
        "documents/fonds/complet.automerge est en place, mais son renommage n'est pas rendu durable."
    );
    assert!(
        !message.contains(&*racine.chemin().to_string_lossy()),
        "{message}"
    );

    let services = paire.rouvrir(services, &[]);
    let vue = services
        .vue_d_une_idee(IDEE)
        .expect("l'idée de la première note se lit");
    assert_eq!(vue.texte.as_deref(), Some("Un refrain qui monte, en ré"));
}

/// Deux notes, puis un octet changé au milieu du fichier d'ajout de la seconde : rouvert, le fonds
/// se lit sans écrire, et le refus de sa lecture seule nomme le fichier d'ajout par son chemin
/// relatif à la racine des données, sans le chemin de la racine des données.
#[test]
fn la_lecture_seule_d_un_ajout_altere_le_nomme_par_son_chemin_relatif() {
    let racine = racine_jetable("commandes-ajout-altere");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite");
    services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite");
    drop(services);
    let ajouts = commun::dossier_des_ajouts(racine.chemin());
    let noms = commun::entrees(&ajouts);
    assert_eq!(
        noms.len(),
        1,
        "la note B, un seul fichier d'ajout : {noms:?}"
    );
    commun::alterer_l_octet_du_milieu(&ajouts.join(&noms[0]));

    let documents = FondsSurFichiers::ouvrir(&racine).expect("rouvrir le fonds");
    let refus = documents
        .lecture_seule()
        .expect("le fonds se lit sans écrire");
    assert_eq!(refus.raison, Raison::Illisible);
    let message = refus.message;
    assert!(
        message.starts_with("Le contenu de documents/fonds/ajouts/"),
        "{message}"
    );
    assert!(
        message.ends_with(
            ".automerge ne donne pas l'empreinte que porte son nom : il n'est pas chargé, et le \
             document se lit sans écrire ni échanger."
        ),
        "{message}"
    );
    assert!(
        !message.contains(&*racine.chemin().to_string_lossy()),
        "{message}"
    );
}

/// Une prise versée, coupée avant le renommage de sa captation : le refus du disque dit « Une
/// captation », sans son chemin sous `captations/`, qui nommerait son empreinte, ni le chemin de
/// la racine des données, et le fonds ne naît pas.
#[test]
fn une_captation_coupee_avant_le_renommage_est_refusee_sans_nommer_son_empreinte() {
    let racine = racine_jetable("commandes-captation-coupee");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();

    let rendu = {
        let _panne = couper(Etape::Renommage, 1);
        services.verser_une_fiche(
            gestes::Sorte::Prise,
            suite::PRISE_DE_TEST,
            &suite::fiche_wav(30_000),
            &suite::importee(),
            0,
            &mut contexte_de_test(&mut alea),
        )
    };
    let message = match rendu {
        Err(Refus {
            raison: Raison::Disque,
            message,
        }) => message,
        autre => panic!("attendu le refus du disque ; rendu {autre:?}"),
    };
    let e = stockage::empreinte(suite::PRISE_DE_TEST);
    // Le chemin de la captation rangée nommerait son empreinte : « Une captation » le remplace.
    assert_eq!(message, "Une captation, le disque l'a refusée.");
    assert!(!message.contains(&e), "{message}");
    assert!(
        !message.contains(&*racine.chemin().to_string_lossy()),
        "{message}"
    );
    assert!(services.documents().fonds().is_none());
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

/// `#pont` posée sur A ; « annuler », coupé avant le renommage de son fichier d'ajout, est refusé
/// par le disque : A garde `#pont`, et le journal ne change pas ; la panne levée, « annuler » vise
/// encore la pose, et la défait ; rouvert, le fonds montre A sans étiquette.
#[test]
fn une_annulation_refusee_par_le_disque_se_refait() {
    let racine = racine_jetable("commandes-annulation-refusee-par-le-disque");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite");
    services
        .poser_une_etiquette(&suite::etiquetage(IDEE, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    let journal = services.journal().expect("le journal se lit");

    let rendu = {
        let _panne = couper(Etape::Renommage, 1);
        services.annuler(&mut contexte)
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
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.etiquettes),
        Ok(vec!["pont".to_string()])
    );
    assert_eq!(services.journal(), Ok(journal.clone()));

    assert_eq!(
        services.annuler(&mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    let mut attendu = journal;
    attendu.push(EntreeDuJournal {
        message: "Annulé : Idée du 14 novembre : étiquette #pont posée".to_string(),
        heure: suite::HEURE,
    });
    assert_eq!(services.journal(), Ok(attendu));
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.etiquettes),
        Ok(Vec::new())
    );

    let services = paire.rouvrir(services, &[]);
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.etiquettes),
        Ok(Vec::new())
    );
}

/// `#pont` posée sur A ; compléter A, coupé avant le renommage de son fichier d'ajout, est refusé
/// par le disque : A garde son texte, et le journal ne change pas. Le geste refusé ne devient pas
/// ce qu'annuler vise : « annuler » défait encore la pose ; rouvert, le fonds montre A de son
/// premier texte, sans étiquette.
#[test]
fn un_geste_de_tri_refuse_par_le_disque_ne_change_pas_ce_qu_annuler_vise() {
    let racine = racine_jetable("commandes-geste-de-tri-refuse-par-le-disque");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite");
    services
        .poser_une_etiquette(&suite::etiquetage(IDEE, "pont"), &mut contexte)
        .expect("l'étiquette est posée sur A");
    let journal = services.journal().expect("le journal se lit");

    let rendu = {
        let _panne = couper(Etape::Renommage, 1);
        services.completer(
            &suite::completion(IDEE, "Un refrain qui monte, puis retombe"),
            &mut contexte,
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
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.texte),
        Ok(Some("Un refrain qui monte, en ré".to_string()))
    );
    assert_eq!(services.journal(), Ok(journal.clone()));

    assert_eq!(
        services.annuler(&mut contexte),
        Ok(Faite {
            avertissement: None
        })
    );
    let mut attendu = journal;
    attendu.push(EntreeDuJournal {
        message: "Annulé : Idée du 14 novembre : étiquette #pont posée".to_string(),
        heure: suite::HEURE,
    });
    assert_eq!(services.journal(), Ok(attendu));

    let services = paire.rouvrir(services, &[]);
    let vue = services.vue_d_une_idee(IDEE).expect("l'idée A se lit");
    assert_eq!(vue.texte.as_deref(), Some("Un refrain qui monte, en ré"));
    assert_eq!(vue.etiquettes, Vec::<String>::new());
}

/// Compléter A, coupé après le renommage de son fichier d'ajout, est fait, avec un avertissement,
/// et devient ce qu'annuler vise : « annuler », coupé de même, le défait, avec un avertissement ;
/// rouvert, le fonds montre A de son premier texte.
#[test]
fn un_geste_de_tri_et_son_annulation_coupes_apres_le_renommage_rendent_un_avertissement() {
    let racine = racine_jetable("commandes-geste-de-tri-coupe-apres-le-renommage");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite");

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        services.completer(
            &suite::completion(IDEE, "Un refrain qui monte, puis retombe"),
            &mut contexte,
        )
    };
    assert!(
        matches!(
            rendu,
            Ok(Faite {
                avertissement: Some(_)
            })
        ),
        "attendu le geste fait, avec un avertissement ; rendu {rendu:?}"
    );
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.texte),
        Ok(Some("Un refrain qui monte, puis retombe".to_string()))
    );

    let rendu = {
        let _panne = couper(Etape::RendreDurable, 1);
        services.annuler(&mut contexte)
    };
    assert!(
        matches!(
            rendu,
            Ok(Faite {
                avertissement: Some(_)
            })
        ),
        "attendu l'annulation faite, avec un avertissement ; rendu {rendu:?}"
    );
    let journal = services.journal().expect("le journal se lit");
    assert_eq!(
        journal.last(),
        Some(&EntreeDuJournal {
            message: "Annulé : Idée du 14 novembre : texte complété".to_string(),
            heure: suite::HEURE,
        }),
        "{journal:?}"
    );
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.texte),
        Ok(Some("Un refrain qui monte, en ré".to_string()))
    );

    let services = paire.rouvrir(services, &[]);
    assert_eq!(
        services.vue_d_une_idee(IDEE).map(|vue| vue.texte),
        Ok(Some("Un refrain qui monte, en ré".to_string()))
    );
}

/// Deux notes, puis un octet changé au milieu du fichier d'ajout de la seconde : rouvert, l'état
/// du fonds le dit né, en lecture seule, du refus `Raison::Illisible`, qui nomme le fichier
/// d'ajout par son chemin relatif à la racine des données, écrit ici en texte ; et la boîte se lit
/// en lecture seule : elle montre la seule note d'avant l'altération, non reliée.
#[test]
fn l_etat_du_fonds_dit_la_lecture_seule() {
    let racine = racine_jetable("commandes-etat-lecture-seule");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    services
        .noter(&une_note("Un refrain qui monte, en ré"), &mut contexte)
        .expect("la note A est faite");
    services
        .noter(&une_note("Un pont en mi mineur"), &mut contexte)
        .expect("la note B est faite");
    drop(services);
    let ajouts = commun::dossier_des_ajouts(racine.chemin());
    let noms = commun::entrees(&ajouts);
    assert_eq!(
        noms.len(),
        1,
        "la note B, un seul fichier d'ajout : {noms:?}"
    );
    commun::alterer_l_octet_du_milieu(&ajouts.join(&noms[0]));

    let services = paire.neufs();
    let etat = services.etat_du_fonds().expect("l'état du fonds se lit");
    assert!(etat.ne);
    assert_eq!(
        etat.lecture_seule,
        Some(Refus {
            raison: Raison::Illisible,
            message: format!(
                "Le contenu de documents/fonds/ajouts/{} ne donne pas l'empreinte que porte son \
                 nom : il n'est pas chargé, et le document se lit sans écrire ni échanger.",
                noms[0]
            ),
        })
    );
    let vue = services
        .boite(&suite::toute_la_boite())
        .expect("la boîte se lit en lecture seule");
    assert_eq!(
        vue.lignes,
        vec![LigneDeLaBoite {
            idee: IDEE.to_string(),
            le: suite::HEURE,
            texte: Some("Un refrain qui monte, en ré".to_string()),
            etiquettes: Vec::new(),
            duree_ms: None,
            reliee: false,
            provenance: Provenance::Captee {
                par: Appareil::Etabli,
            },
            retiree: None,
            nom_de_fichier: None,
        }]
    );
    assert_eq!(vue.non_reliees, 1);
}

/// La première note, coupée après le renommage du fichier complet de la naissance, comme
/// `la_premiere_note_coupee_apres_le_renommage_fait_naitre_le_fonds_avec_un_avertissement` :
/// l'état du fonds porte l'avertissement de la naissance, qui nomme le fichier complet par son
/// chemin relatif à la racine des données, sans le chemin de la racine des données.
#[test]
fn l_etat_du_fonds_dit_l_avertissement_de_la_naissance() {
    let racine = racine_jetable("commandes-etat-avertissement-de-la-naissance");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    {
        let _panne = couper(Etape::RendreDurable, 1);
        services
            .noter(
                &une_note("Un refrain qui monte, en ré"),
                &mut contexte_de_test(&mut alea),
            )
            .expect("la première note est faite");
    }

    assert_eq!(
        services.etat_du_fonds().map(|etat| etat.avertissements),
        Ok(vec![
            "documents/fonds/complet.automerge est en place, mais son renommage n'est pas rendu \
             durable."
                .to_string()
        ])
    );
}

/// Les noms provisoires de `captations` que `import::copie_de` dit de l'import `import`, dans
/// l'ordre.
fn copies_de(captations: &Path, import: &str) -> Vec<String> {
    commun::entrees(captations)
        .into_iter()
        .filter(|nom| copie_de(nom).is_some_and(|(de, _)| de == import))
        .collect()
}

/// Point de revue 4 ; question 31 ; R1 : quatre WAV, le quatrième de 2 621 440 octets ; les trois
/// premiers versés, puis la copie du quatrième coupée avant son deuxième tronçon, comme par une
/// coupure du processus, qui coupe aussi le retrait de son nom provisoire : le versement est
/// refusé par le disque, et le nom provisoire `import-<I>-4.<processus>.partiel` reste, d'un
/// tronçon. Rouvert, le fonds le garde, son contenu n'étant pas nommé ; l'inspection du même
/// dossier dit la reprise, et l'ouverture de la reprise le retire. À la clôture, aucun nom
/// provisoire de l'import ne reste, et le rapport égale celui d'un import fait d'une traite du
/// même dossier.
#[test]
fn une_reprise_apres_une_coupure_entre_deux_troncons_ne_laisse_aucune_copie() {
    let racine = racine_jetable("commandes-reprise-coupee");
    let racine_neuve = racine_jetable("commandes-reprise-coupee-traite");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    let grand = suite::wav_de(2_621_396, 4);
    assert_eq!(grand.len(), 2_621_440);
    let fichiers = [
        (Some("a.wav"), suite::wav_de(8_000, 1)),
        (Some("b.wav"), suite::wav_de(8_000, 2)),
        (Some("c.wav"), suite::wav_de(8_000, 3)),
        (Some("d.wav"), grand),
    ];
    let source = paire.source(Some(suite::DOSSIER_VERSE), &fichiers, &[]);
    services
        .inspecter(&source, suite::PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(
            &source,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre")
        .import;
    for rang in 1..=3 {
        suite::verser_le_rang(&mut services, &source, &import, rang, &mut contexte);
    }
    let captations = racine.chemin().join("captations");

    let rendu = {
        let _troncon = couper(Etape::Troncon, 2);
        let _retrait = couper(Etape::Retrait, 1);
        services.verser_un_fichier(
            &source,
            &VerserUnFichier {
                import: import.clone(),
                rang: 4,
                decalage_minutes: 0,
            },
            &mut contexte,
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
        "la copie coupée : rendu {rendu:?}"
    );
    let copie = format!(
        "{}.{}.partiel",
        nom_de_la_copie(&import, 4),
        std::process::id()
    );
    assert_eq!(copies_de(&captations, &import), vec![copie.clone()]);
    assert_eq!(commun::poids(&captations.join(&copie)), 1_048_576);

    let mut services = paire.rouvrir(services, &[]);
    assert_eq!(copies_de(&captations, &import), vec![copie]);
    let inspection = services
        .inspecter(&source, suite::PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    assert_eq!(inspection.reprise.as_deref(), Some(import.as_str()));
    let reprise = services
        .ouvrir_l_import(
            &source,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.rangs, vec![4]);
    assert_eq!(copies_de(&captations, &import), Vec::<String>::new());
    suite::verser_le_rang(&mut services, &source, &import, 4, &mut contexte);
    let clos = services
        .clore_l_import(&source, &suite::cloture(&import), &mut contexte)
        .expect("l'import repris se clôt");
    assert_eq!(copies_de(&captations, &import), Vec::<String>::new());

    let mut neuve = PaireSurFichiers {
        racine: &racine_neuve,
    };
    let mut neufs = neuve.neufs();
    let source_neuve = neuve.source(Some(suite::DOSSIER_VERSE), &fichiers, &[]);
    let mut alea_neuf = alea_de_test();
    let traite = suite::importer(
        &mut neufs,
        &source_neuve,
        &[],
        &mut contexte_de_test(&mut alea_neuf),
    );
    assert_eq!(
        clos.rapport,
        RapportDImport {
            import: import.clone(),
            ..traite.clos.rapport
        }
    );
}

/// Question 31 : un import ouvert, son fichier versé ; un nom provisoire de cet import,
/// `import-<I>-9.1.partiel`, et un nom provisoire de capture, `<e>.1.partiel`, posés dans
/// `captations/` ; la clôture retire le premier, et laisse le second.
#[test]
fn la_cloture_retire_chaque_copie_de_l_import() {
    let racine = racine_jetable("commandes-import-copies");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(suite::DOSSIER_VERSE),
        &[(Some("a.wav"), suite::wav_de(8_000, 1))],
        &[],
    );
    services
        .inspecter(&source, suite::PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(
            &source,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre")
        .import;
    suite::verser_le_rang(&mut services, &source, &import, 1, &mut contexte);
    let captations = racine.chemin().join("captations");
    let copie = captations.join(format!("{}.1.partiel", nom_de_la_copie(&import, 9)));
    fs::write(&copie, b"une copie laissee").expect("poser la copie de l'import");
    let capture = captations.join(format!(
        "{}.1.partiel",
        stockage::empreinte(b"une capture laissee")
    ));
    fs::write(&capture, b"une capture laissee").expect("poser la copie d'une capture");

    let clos = services
        .clore_l_import(&source, &suite::cloture(&import), &mut contexte)
        .expect("l'import se clôt");

    assert_eq!(clos.avertissement, None);
    assert!(!copie.exists(), "la copie de l'import reste");
    assert!(capture.is_file(), "la copie d'une capture est retirée");
}

/// Question 35 ; le défaut (b) de la tâche 12 : un import ouvert sur `A`, un fichier versé, puis
/// le fonds rouvert, qui se compacte ; l'inspection de `B` dit l'import ouvert ailleurs.
/// L'ouverture de `B`, coupée au renommage de son fichier d'ajout, est refusée par le disque,
/// l'import de `A` déjà clos, une fois ; la seconde tentative ouvre `B` sans le reclore : le
/// journal ne porte qu'un « Import du 14 novembre clos ».
#[test]
fn une_ouverture_refusee_apres_la_cloture_d_un_import_ouvert_ailleurs_ne_le_reclot_pas() {
    let racine = racine_jetable("commandes-import-ailleurs-une-fois");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    let source_a = paire.source(Some("A"), &[(Some("a.wav"), suite::wav_de(8_000, 1))], &[]);
    services
        .inspecter(&source_a, suite::PLACE_LIBRE)
        .expect("A s'inspecte");
    let premier = services
        .ouvrir_l_import(
            &source_a,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import de A s'ouvre")
        .import;
    suite::verser_le_rang(&mut services, &source_a, &premier, 1, &mut contexte);
    // Rouvert, le fonds se compacte : les écritures qui suivent sont trop petites pour en
    // demander une autre, et chacune ne renomme que son propre fichier.
    let mut services = paire.rouvrir(services, &[]);
    let source_b = paire.source(Some("B"), &[(Some("b.wav"), suite::wav_de(8_000, 2))], &[]);
    let inspection = services
        .inspecter(&source_b, suite::PLACE_LIBRE)
        .expect("B s'inspecte");
    assert!(
        inspection.ouvert_ailleurs,
        "l'import de A n'est pas dit ouvert ailleurs"
    );
    let clos = "Import du 14 novembre clos";

    let rendu = {
        // Le premier renommage range le rapport de A ; le deuxième, le fichier d'ajout de sa
        // clôture ; le troisième, celui de l'ouverture de B.
        let _panne = couper(Etape::Renommage, 3);
        services.ouvrir_l_import(
            &source_b,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
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
        "l'ouverture coupée : rendu {rendu:?}"
    );
    let fonds = services.documents().fonds().expect("le fonds est né");
    let imports = lecture::imports(fonds);
    assert_eq!(imports.len(), 1, "les imports : {imports:?}");
    assert_eq!(imports[0].import, premier);
    assert!(imports[0].rapport.is_some(), "l'import de A n'est pas clos");
    let messages = suite::messages_du_journal(&services);
    assert_eq!(
        messages.iter().filter(|message| *message == clos).count(),
        1,
        "{messages:?}"
    );

    let second = services
        .ouvrir_l_import(
            &source_b,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("la seconde tentative ouvre l'import de B");
    assert!(!second.repris, "l'import de B se dit repris");
    let messages = suite::messages_du_journal(&services);
    assert_eq!(
        messages.iter().filter(|message| *message == clos).count(),
        1,
        "{messages:?}"
    );
    assert_eq!(
        messages.last().map(String::as_str),
        Some("Import du 14 novembre ouvert")
    );
}

/// Le doute 11 de la tâche 12 : un import repris après la réouverture du fonds, qui se compacte,
/// chacun de ses fichiers versé avant ; un nom provisoire `import-<I>-9.1.partiel` posé ; la
/// clôture, dont le retrait de cette copie est refusé par le disque, est faite : la réponse porte
/// le refus en avertissement, sans nommer la copie ; la copie reste ; l'import est clos, et une
/// seconde clôture n'en trouve plus d'ouvert.
#[test]
fn une_cloture_dont_le_retrait_d_une_copie_est_refuse_est_faite_avec_un_avertissement() {
    let racine = racine_jetable("commandes-import-copie-restee");
    let mut paire = PaireSurFichiers { racine: &racine };
    let mut services = paire.neufs();
    let mut alea = alea_de_test();
    let mut contexte = contexte_de_test(&mut alea);
    let source = paire.source(
        Some(suite::DOSSIER_VERSE),
        &[(Some("a.wav"), suite::wav_de(8_000, 1))],
        &[],
    );
    services
        .inspecter(&source, suite::PLACE_LIBRE)
        .expect("le dossier s'inspecte");
    let import = services
        .ouvrir_l_import(
            &source,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import s'ouvre")
        .import;
    suite::verser_le_rang(&mut services, &source, &import, 1, &mut contexte);
    // Rouvert, le fonds se compacte : le fichier d'ajout de la clôture n'en demande pas une
    // autre, qui retirerait ses fichiers d'ajout avant la copie.
    let mut services = paire.rouvrir(services, &[]);
    services
        .inspecter(&source, suite::PLACE_LIBRE)
        .expect("le dossier s'inspecte de nouveau");
    let reprise = services
        .ouvrir_l_import(
            &source,
            &suite::ouverture(&[]),
            suite::PLACE_LIBRE,
            &mut contexte,
        )
        .expect("l'import se reprend");
    assert!(reprise.repris, "l'ouverture ne dit pas la reprise");
    assert_eq!(reprise.rangs, Vec::<u64>::new());
    let copie = racine
        .chemin()
        .join("captations")
        .join(format!("{}.1.partiel", nom_de_la_copie(&import, 9)));
    fs::write(&copie, b"une copie laissee").expect("poser la copie de l'import");

    let rendu = {
        let _retrait = couper(Etape::Retrait, 1);
        services.clore_l_import(&source, &suite::cloture(&import), &mut contexte)
    };

    let clos = rendu.expect("la clôture écrite est faite");
    assert_eq!(
        clos.avertissement.as_deref(),
        Some("Une captation, le disque l'a refusée.")
    );
    assert!(copie.is_file(), "la copie au retrait refusé est retirée");
    assert_eq!(
        suite::messages_du_journal(&services)
            .last()
            .map(String::as_str),
        Some("Import du 14 novembre clos")
    );
    assert_eq!(
        services.clore_l_import(&source, &suite::cloture(&import), &mut contexte),
        Err(Refus {
            raison: Raison::Impossible,
            message: "Aucun import n'est ouvert.".to_string(),
        })
    );
}
