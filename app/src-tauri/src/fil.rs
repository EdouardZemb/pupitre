//! Le fil du fonds : un fil dédié possède la racine des données et les services de l'établi, sur
//! fichiers, et sert chaque demande qu'il reçoit par un canal, l'une après l'autre. La racine des
//! données, le fonds et les services ne quittent jamais ce fil : seules les demandes et leurs
//! réponses le traversent.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError, mpsc};
use std::thread::{self, JoinHandle};

use commandes::Reponse;
use noyau::donnees::gestes;
use services::{Contexte, Raison, Refus, Services};
use stockage::{FondsSurFichiers, RacineDesDonnees, refus_a_l_ouverture};

use crate::systeme::{self, AleaDuSysteme};

/// Les services de l'établi, sur fichiers : ils empruntent la racine des données que le fil
/// possède.
pub type ServicesSurFichiers<'r> = Services<FondsSurFichiers<'r>, stockage::Captations<'r>>;

/// Une demande au fil : une fermeture, qu'il appelle sur ses services et le contexte du geste, et
/// qui envoie elle-même sa réponse à qui l'attend.
type Demande = Box<dyn for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) + Send>;

/// L'ouverture du fonds, que Tauri garde dans son état (`Manager::manage`) : le fil qui sert les
/// commandes, ou le refus d'ouvrir, que chaque commande rend.
pub enum Ouverture {
    /// Le fonds est ouvert : le fil sert les commandes.
    Ouverte(FilDuFonds),
    /// L'ouverture est refusée : chaque commande rend ce refus.
    Refusee(Refus),
}

/// Le fil du fonds : il ouvre la racine des données, puis le fonds, possède les services, et sert
/// chaque demande reçue, l'une après l'autre ; à chaque demande, le contexte du geste porte
/// `systeme::heure()` et `systeme::AleaDuSysteme`.
pub struct FilDuFonds {
    /// L'émetteur des demandes ; `None` une fois le fil fermé.
    emetteur: Mutex<Option<mpsc::Sender<Demande>>>,
    /// Le fil ; `None` une fois sa fin attendue.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "la tranche 6 ferme le fil avant de renommer `donnees/` ; les tests"
        )
    )]
    fil: Mutex<Option<JoinHandle<()>>>,
}

/// Le refus d'un fil tombé ou fermé.
fn ne_repond_plus() -> Refus {
    Refus {
        raison: Raison::Impossible,
        message: "Le fonds ne répond plus.".to_string(),
    }
}

/// Ouvre le fil sur la racine des données `racine`, et attend le résultat de l'ouverture : le fil
/// qui sert les demandes, ou le refus d'ouvrir la racine des données (`refus_a_l_ouverture`), ou
/// le fonds (tel quel). Refusée, l'ouverture attend aussi la fin du fil, qui a rendu ce qu'il
/// avait pris. Un fil qui ne se lance pas, ou qui tombe avant de dire son ouverture : « Le fonds ne
/// répond plus. ».
pub fn ouvrir(racine: PathBuf) -> Ouverture {
    let (envoi_de_l_ouverture, ouverture) = mpsc::channel();
    let (emetteur, demandes) = mpsc::channel();
    let lance = thread::Builder::new()
        .name("fonds".to_string())
        .spawn(move || servir(&racine, envoi_de_l_ouverture, demandes));
    let Ok(fil) = lance else {
        return Ouverture::Refusee(ne_repond_plus());
    };
    match ouverture.recv() {
        Ok(Ok(())) => Ouverture::Ouverte(FilDuFonds {
            emetteur: Mutex::new(Some(emetteur)),
            fil: Mutex::new(Some(fil)),
        }),
        Ok(Err(refus)) => {
            let _ = fil.join();
            Ouverture::Refusee(refus)
        }
        Err(mpsc::RecvError) => {
            let _ = fil.join();
            Ouverture::Refusee(ne_repond_plus())
        }
    }
}

/// Le corps du fil : il ouvre la racine des données `chemin`, puis le fonds, et envoie le résultat
/// de l'ouverture par `ouverture` ; ouvert, il sert chaque demande de `demandes`, dans l'ordre de
/// leur arrivée, jusqu'à ce que tombe le dernier émetteur, puis rend les services, et enfin la
/// racine des données et son verrou. Une demande qui panique fait tomber le fil : la racine des
/// données et son verrou tombent avec lui, sans `catch_unwind`.
fn servir(
    chemin: &Path,
    ouverture: mpsc::Sender<Result<(), Refus>>,
    demandes: mpsc::Receiver<Demande>,
) {
    let racine = match RacineDesDonnees::ouvrir(chemin) {
        Ok(racine) => racine,
        Err(erreur) => {
            let _ = ouverture.send(Err(refus_a_l_ouverture(erreur)));
            return;
        }
    };
    let documents = match FondsSurFichiers::ouvrir(&racine) {
        Ok(documents) => documents,
        Err(refus) => {
            let _ = ouverture.send(Err(refus));
            return;
        }
    };
    let mut services = Services::new(documents, racine.captations(), gestes::Appareil::Etabli);
    let _ = ouverture.send(Ok(()));
    drop(ouverture);
    while let Ok(demande) = demandes.recv() {
        let mut alea = AleaDuSysteme;
        let mut contexte = Contexte {
            heure: systeme::heure(),
            alea: &mut alea,
        };
        demande(&mut services, &mut contexte);
    }
    drop(services);
    // Le verrou se rend en dernier : la racine des données ne se ferme qu'une fois les services
    // rendus.
    drop(racine);
}

/// Envoie `f` au fil par `emetteur`, et attend sa réponse. L'envoi refusé (le fil est tombé) ou la
/// réponse perdue (le fil est tombé pendant la demande, ou avant de la prendre) : « Le fonds ne
/// répond plus. ». L'émetteur tombe dès l'envoi : il ne retient pas le fil pendant l'attente.
fn envoyer<T, F>(emetteur: mpsc::Sender<Demande>, f: F) -> Reponse<T>
where
    T: Send + 'static,
    F: for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<T>
        + Send
        + 'static,
{
    let (envoi, reponse) = mpsc::channel();
    let demande: Demande = Box::new(move |services, contexte| {
        // Qui a demandé attend la réponse ; s'il ne l'attend plus, elle se perd sans dommage.
        let _ = envoi.send(f(services, contexte));
    });
    let envoyee = emetteur.send(demande);
    drop(emetteur);
    envoyee.map_err(|_| ne_repond_plus())?;
    // La mpsc standard, quand tombe le récepteur (le fil panique), jette les messages en tampon,
    // et leurs émetteurs de réponse avec eux : chaque demande en attente reçoit le refus ; sans
    // cela, elle attendrait jusqu'à `fermer`.
    reponse.recv().unwrap_or_else(|_| Err(ne_repond_plus()))
}

impl Ouverture {
    /// Demande `f` au fil, et attend sa réponse ; refusée, ou fil tombé : le refus.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "les tests ; les commandes passent par `demander`")
    )]
    pub fn demander_bloquant<T: Send + 'static>(
        &self,
        f: impl for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<T>
        + Send
        + 'static,
    ) -> Reponse<T> {
        envoyer(self.emetteur()?, f)
    }

    /// La même demande, attendue hors du fil de l'IPC (`tauri::async_runtime::spawn_blocking`), sur
    /// un émetteur cloné.
    pub async fn demander<T: Send + 'static>(
        &self,
        f: impl for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<T>
        + Send
        + 'static,
    ) -> Reponse<T> {
        let emetteur = self.emetteur()?;
        tauri::async_runtime::spawn_blocking(move || envoyer(emetteur, f))
            .await
            .unwrap_or_else(|_| Err(ne_repond_plus()))
    }

    /// Ferme le fil : l'émetteur tombe, le fil finit sa demande en cours, rend les services, puis
    /// la racine des données et son verrou, et `fermer` attend sa fin. Sans effet sur une
    /// `Refusee`.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "la tranche 6 ferme le fil avant de renommer `donnees/` ; les tests"
        )
    )]
    pub fn fermer(&self) {
        if let Ouverture::Ouverte(fil) = self {
            fil.fermer();
        }
    }

    /// L'émetteur des demandes, cloné ; refusée, le refus d'ouvrir ; le fil fermé, « Le fonds ne
    /// répond plus. ».
    fn emetteur(&self) -> Result<mpsc::Sender<Demande>, Refus> {
        match self {
            Ouverture::Ouverte(fil) => fil
                .emetteur
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone()
                .ok_or_else(ne_repond_plus),
            Ouverture::Refusee(refus) => Err(refus.clone()),
        }
    }
}

impl FilDuFonds {
    /// L'émetteur tombe : le fil sert les demandes déjà envoyées, puis sort dès que tombe le
    /// dernier émetteur cloné, qu'une demande lâche dès son envoi ; puis sa fin s'attend. Le verrou
    /// de `fil` reste pris pendant l'attente : un second `fermer` attend la même fin, puis ne
    /// trouve plus rien à attendre.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "la tranche 6 ferme le fil avant de renommer `donnees/` ; les tests"
        )
    )]
    fn fermer(&self) {
        drop(
            self.emetteur
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take(),
        );
        let mut fil = self.fil.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(fin) = fil.take() {
            // Un fil tombé rend sa panique : le refus des demandes suivantes la dit déjà.
            let _ = fin.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::dossier_jetable;
    use commandes::{Appareil, Notee, Noter, Provenance};

    /// La fermeture de `noter`, qui note `texte`, `decalage_minutes: 60`.
    fn noter_une(
        texte: &str,
    ) -> impl for<'r> FnOnce(&mut ServicesSurFichiers<'r>, &mut Contexte<'_>) -> Reponse<Notee>
    + Send
    + 'static {
        let commande = Noter {
            texte: texte.to_string(),
            decalage_minutes: 60,
        };
        move |services, contexte| services.noter(&commande, contexte)
    }

    /// Ce que porte la racine des données : chaque fichier sous `documents/` et `captations/`, son
    /// chemin relatif et ses octets, dans l'ordre des chemins ; le fichier `verrou`, par sa seule
    /// présence et sa taille, jamais par ses octets : sous Windows, le verrou que tient la première
    /// ouverture peut en refuser la lecture.
    #[derive(Debug, PartialEq, Eq)]
    struct Photographie {
        fichiers: Vec<(String, Vec<u8>)>,
        verrou: Option<u64>,
    }

    fn photographie(racine: &Path) -> Photographie {
        let mut fichiers = Vec::new();
        for nom in ["documents", "captations"] {
            fichiers_sous(racine, &racine.join(nom), &mut fichiers);
        }
        fichiers.sort();
        let verrou = std::fs::metadata(racine.join("verrou"))
            .ok()
            .map(|m| m.len());
        Photographie { fichiers, verrou }
    }

    /// Chaque fichier sous `dossier`, en descendant, son chemin relatif à `racine`, ses composants
    /// joints par `/`, et ses octets.
    fn fichiers_sous(racine: &Path, dossier: &Path, fichiers: &mut Vec<(String, Vec<u8>)>) {
        let entrees = std::fs::read_dir(dossier)
            .unwrap_or_else(|e| panic!("lire {} : {e}", dossier.display()));
        for entree in entrees {
            let chemin = entree.expect("une entrée").path();
            if chemin.is_dir() {
                fichiers_sous(racine, &chemin, fichiers);
            } else {
                let relatif = chemin
                    .strip_prefix(racine)
                    .expect("un chemin sous la racine des données")
                    .components()
                    .map(|composant| composant.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let octets = std::fs::read(&chemin)
                    .unwrap_or_else(|e| panic!("lire {} : {e}", chemin.display()));
                fichiers.push((relatif, octets));
            }
        }
    }

    #[test]
    fn le_fil_note_puis_relit_l_idee() {
        let racine = dossier_jetable("fil-note").join("donnees");
        let ouverture = ouvrir(racine);
        let notee = ouverture
            .demander_bloquant(noter_une("Une idée de pont"))
            .unwrap();
        let vue = ouverture
            .demander_bloquant(move |services, _| services.vue_d_une_idee(&notee.idee))
            .unwrap();
        assert_eq!(vue.texte.as_deref(), Some("Une idée de pont"));
        assert_eq!(
            vue.provenance,
            Provenance::Captee {
                par: Appareil::Etabli
            }
        );
    }

    #[test]
    fn un_second_processus_devant_une_racine_tenue_s_arrete_en_le_disant() {
        let racine = dossier_jetable("fil-tenue").join("donnees");
        let premiere = ouvrir(racine.clone());
        premiere.demander_bloquant(noter_une("Une idée")).unwrap();
        let avant = photographie(&racine);
        let seconde = ouvrir(racine.clone());
        let Ouverture::Refusee(refus) = &seconde else {
            panic!("la seconde ouverture devait être refusée")
        };
        assert_eq!(refus.raison, Raison::Impossible);
        assert_eq!(
            refus.message,
            "Un autre processus tient la racine des données."
        );
        // Aucune commande servie, et rien d'écrit sous donnees/ (question 4 ; LE-6, règle 19).
        assert_eq!(
            seconde
                .demander_bloquant(noter_une("Une autre"))
                .unwrap_err(),
            refus.clone()
        );
        assert_eq!(photographie(&racine), avant);
    }

    #[test]
    fn le_fil_ferme_la_racine_quand_on_le_ferme() {
        let racine = dossier_jetable("fil-ferme").join("donnees");
        let premiere = std::sync::Arc::new(ouvrir(racine.clone()));
        // Une demande est en cours quand `fermer` est appelé : le fil la finit, puis rend la
        // racine des données et son verrou, et `fermer` attend sa fin. Sans cette attente, la
        // racine des données est encore tenue quand `ouvrir` la demande de nouveau.
        let (commence, a_commence) = mpsc::channel();
        let demandeur = {
            let premiere = premiere.clone();
            std::thread::spawn(move || {
                premiere.demander_bloquant(move |_, _| -> Reponse<()> {
                    commence.send(()).unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    Ok(())
                })
            })
        };
        a_commence.recv().unwrap();
        premiere.fermer();
        if let Ouverture::Refusee(refus) = ouvrir(racine) {
            panic!(
                "après fermer, la racine des données est encore tenue : {}",
                refus.message
            );
        }
        assert_eq!(demandeur.join().unwrap(), Ok(()));
        let refus = premiere
            .demander_bloquant(noter_une("Une idée"))
            .unwrap_err();
        assert_eq!(refus.message, "Le fonds ne répond plus.");
    }

    #[test]
    fn une_racine_que_le_disque_refuse_se_dit_par_son_chemin() {
        let dossier = dossier_jetable("fil-disque");
        let racine = dossier.join("donnees");
        std::fs::write(&racine, b"un fichier, pas un dossier").unwrap();
        let Ouverture::Refusee(refus) = ouvrir(racine.clone()) else {
            panic!("l'ouverture devait être refusée")
        };
        assert_eq!(refus.raison, Raison::Disque);
        // Le stockage joint les composants du chemin par `/`, sous Windows aussi (`chemin_dit`),
        // puis dit le constat du disque (`constat_du_disque`), qui suit le genre de l'erreur :
        // « , le disque l'a refusé » pour `AlreadyExists`, attendu ici, « est absent » ou « est
        // refusé par le système » si le système en rend un autre ; le test exige le chemin entier
        // en tête, non le genre.
        let dit = racine.display().to_string().replace('\\', "/");
        assert!(
            refus.message.starts_with(&format!("{dit} "))
                || refus.message.starts_with(&format!("{dit},")),
            "{}",
            refus.message
        );
    }

    #[test]
    fn deux_demandes_concurrentes_sont_servies_l_une_apres_l_autre() {
        let racine = dossier_jetable("fil-concurrence").join("donnees");
        let ouverture = std::sync::Arc::new(ouvrir(racine));
        let fils: Vec<_> = (0..2)
            .map(|n| {
                let ouverture = ouverture.clone();
                std::thread::spawn(move || {
                    (0..10).all(|i| {
                        ouverture
                            .demander_bloquant(noter_une(&format!("Idée {n}-{i}")))
                            .is_ok()
                    })
                })
            })
            .collect();
        assert!(fils.into_iter().all(|fil| fil.join().unwrap()));
        assert_eq!(
            ouverture
                .demander_bloquant(|services, _| services.idees())
                .unwrap()
                .len(),
            20
        );
    }

    #[test]
    fn une_demande_qui_panique_laisse_un_refus_et_non_une_attente() {
        let racine = dossier_jetable("fil-panique").join("donnees");
        let ouverture = ouvrir(racine.clone());
        let refus = ouverture
            .demander_bloquant(|_, _| -> Reponse<()> { panic!("un défaut planté") })
            .unwrap_err();
        assert_eq!(refus.message, "Le fonds ne répond plus.");
        assert_eq!(
            ouverture
                .demander_bloquant(noter_une("Une idée"))
                .unwrap_err(),
            refus
        );
        // Le fil tombé a rendu la racine des données et son verrou.
        ouverture.fermer();
        if let Ouverture::Refusee(refus) = ouvrir(racine) {
            panic!(
                "après la panique, la racine des données est encore tenue : {}",
                refus.message
            );
        }
    }
}
