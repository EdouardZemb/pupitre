//! Le protocole des captations (§ 40 · 7.5 ; question 27) : WebView2 envoie chaque requête de
//! `http://captation.localhost/<empreinte>` ; elle passe au fil du fonds, une demande courte comme
//! une unité de l'import, qui dit si l'empreinte est l'audio d'une prise ou d'un stem, puis la
//! réponse que les règles du noyau en tirent (`noyau::donnees::protocole`), et lit sa plage. La
//! racine des données reste au fil : ce module ne connaît aucun chemin.

use commandes::Reponse;
use noyau::donnees::protocole::{self, Servie};
use tauri::http::header::{self, HeaderName, HeaderValue};
use tauri::http::{Request, Response, StatusCode};

use crate::fil::Ouverture;

/// L'empreinte que nomme `chemin` : `/` puis 64 chiffres hexadécimaux en minuscules, le seul nom
/// d'une captation ; tout autre chemin, `None`.
fn empreinte_du_chemin(chemin: &str) -> Option<&str> {
    let empreinte = chemin.strip_prefix('/')?;
    let hexadecimale = empreinte
        .bytes()
        .all(|octet| matches!(octet, b'0'..=b'9' | b'a'..=b'f'));
    (empreinte.len() == 64 && hexadecimale).then_some(empreinte)
}

/// La réponse de statut `statut`, de corps `corps`, qui porte `entetes`, dans leur ordre.
fn reponse(
    statut: StatusCode,
    entetes: &[(&'static str, String)],
    corps: Vec<u8>,
) -> Response<Vec<u8>> {
    let mut reponse = Response::new(corps);
    *reponse.status_mut() = statut;
    for (nom, valeur) in entetes {
        // Les noms et les valeurs sont ceux des règles du noyau : des mots d'en-tête et des
        // nombres, que `from_bytes` et `from_str` admettent.
        if let (Ok(nom), Ok(valeur)) = (
            HeaderName::from_bytes(nom.as_bytes()),
            HeaderValue::from_str(valeur),
        ) {
            reponse.headers_mut().append(nom, valeur);
        }
    }
    reponse
}

/// La réponse de `servie`, de corps `corps`, les octets de sa plage, ou aucun.
fn reponse_servie(servie: &Servie, corps: Vec<u8>) -> Response<Vec<u8>> {
    match StatusCode::from_u16(servie.statut) {
        Ok(statut) => reponse(statut, &servie.entetes, corps),
        Err(_) => en_echec(),
    }
}

/// 500, sans corps, avec les deux en-têtes communs (`protocole::introuvable`) : un refus du fil ou
/// du stockage, que le protocole ne dit pas.
pub(crate) fn en_echec() -> Response<Vec<u8>> {
    reponse(
        StatusCode::INTERNAL_SERVER_ERROR,
        &protocole::introuvable().entetes,
        Vec::new(),
    )
}

/// Sert une requête du protocole `captation` (§ 40 · 7.5 ; question 27) : le chemin, `/` puis 64
/// chiffres hexadécimaux en minuscules, nomme l'empreinte, sinon 404 ; au fil, par
/// `Ouverture::demander`, en une seule demande, `captation_a_servir`, puis `protocole::servir`,
/// sur la méthode de la requête et son en-tête `Range` (un en-tête qui ne se lit pas en texte
/// compte pour absent), puis, s'il y a une plage, `plage_d_une_captation`. Une empreinte que les
/// services ne servent pas : 404 (`protocole::introuvable`). Un refus du fil ou du stockage : 500,
/// sans corps, les deux en-têtes communs ; jamais un message.
pub async fn servir_une_captation(
    ouverture: &Ouverture,
    requete: &Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let Some(empreinte) = empreinte_du_chemin(requete.uri().path()) else {
        return reponse_servie(&protocole::introuvable(), Vec::new());
    };
    let empreinte = empreinte.to_string();
    let methode = requete.method().as_str().to_string();
    let range = requete
        .headers()
        .get(header::RANGE)
        .and_then(|valeur| valeur.to_str().ok())
        .map(str::to_string);
    let au_fil = ouverture
        .demander(move |services, _| -> Reponse<(Servie, Vec<u8>)> {
            let Some((type_mime, taille)) = services.captation_a_servir(&empreinte)? else {
                return Ok((protocole::introuvable(), Vec::new()));
            };
            let servie = protocole::servir(&methode, range.as_deref(), taille, type_mime);
            let corps = match servie.plage {
                Some(plage) => services.plage_d_une_captation(&empreinte, plage)?,
                None => Vec::new(),
            };
            Ok((servie, corps))
        })
        .await;
    match au_fil {
        Ok((servie, corps)) => reponse_servie(&servie, corps),
        Err(_) => en_echec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    use commandes::{CloreLImport, ImportClos, OuvrirLImport, VerserUnFichier};
    use noyau::donnees::identifiants::empreinte;
    use stockage::source::SourceSurFichiers;
    use tauri::http::{Request, Response, StatusCode};

    use crate::fil::{self, Ouverture};
    use crate::tests::dossier_jetable;

    /// La place libre que reçoivent l'inspection et l'ouverture des tests, assez grande pour
    /// chacun.
    const PLACE_LIBRE: u64 = 1 << 40;

    /// Un WAV PCM à 8 000 Hz, mono, sur 8 bits, de `trames` trames remplies de `valeur` : un
    /// en-tête de 44 octets, puis ses données. Écrit à l'exécution.
    fn wav(trames: u32, valeur: u8) -> Vec<u8> {
        let mut octets = b"RIFF".to_vec();
        octets.extend_from_slice(&(36 + trames).to_le_bytes());
        octets.extend_from_slice(b"WAVEfmt ");
        octets.extend_from_slice(&16u32.to_le_bytes());
        // Le codage PCM, un canal, la fréquence, les octets par seconde, l'alignement et les bits.
        octets.extend_from_slice(&1u16.to_le_bytes());
        octets.extend_from_slice(&1u16.to_le_bytes());
        octets.extend_from_slice(&8_000u32.to_le_bytes());
        octets.extend_from_slice(&8_000u32.to_le_bytes());
        octets.extend_from_slice(&1u16.to_le_bytes());
        octets.extend_from_slice(&8u16.to_le_bytes());
        octets.extend_from_slice(b"data");
        octets.extend_from_slice(&trames.to_le_bytes());
        octets.resize(44 + trames as usize, valeur);
        octets
    }

    /// Un import ouvert au fil du fonds : l'ouverture, la source parcourue, l'ULID de l'import et
    /// la racine des données.
    struct ImportAuFil {
        ouverture: Ouverture,
        source: Arc<SourceSurFichiers>,
        import: String,
        racine: PathBuf,
    }

    /// Écrit `wavs` sous le dossier `verse` du dossier jetable `nom`, `1.wav`, `2.wav`…, ouvre le
    /// fil sur sa racine des données, `donnees`, comme les tests de `fil` ; puis, au fil, inspecte
    /// le dossier parcouru et ouvre son import, sans déclaration.
    fn ouvrir_un_import(nom: &str, wavs: &[Vec<u8>]) -> ImportAuFil {
        let dossier = dossier_jetable(nom);
        let verse = dossier.join("verse");
        std::fs::create_dir_all(&verse)
            .unwrap_or_else(|e| panic!("créer {} : {e}", verse.display()));
        for (rang, octets) in (1..).zip(wavs) {
            let chemin = verse.join(format!("{rang}.wav"));
            std::fs::write(&chemin, octets)
                .unwrap_or_else(|e| panic!("écrire {} : {e}", chemin.display()));
        }
        let racine = dossier.join("donnees");
        let ouverture = fil::ouvrir(racine.clone());
        let source =
            Arc::new(SourceSurFichiers::parcourir(&verse).expect("le dossier versé se parcourt"));
        let inspectee = Arc::clone(&source);
        ouverture
            .demander_bloquant(move |services, _| services.inspecter(&*inspectee, PLACE_LIBRE))
            .expect("le dossier s'inspecte");
        let ouverte = Arc::clone(&source);
        let commande = OuvrirLImport {
            declarations: Vec::new(),
            decalage_minutes: 0,
        };
        let import = ouverture
            .demander_bloquant(move |services, contexte| {
                services.ouvrir_l_import(&*ouverte, &commande, PLACE_LIBRE, contexte)
            })
            .expect("l'import s'ouvre")
            .import;
        ImportAuFil {
            ouverture,
            source,
            import,
            racine,
        }
    }

    impl ImportAuFil {
        /// Verse le fichier `rang` par `Ouverture::demander`, la voie de chaque commande, qui doit
        /// le verser.
        fn verser(&self, rang: u64) {
            let source = Arc::clone(&self.source);
            let commande = VerserUnFichier {
                import: self.import.clone(),
                rang,
                decalage_minutes: 0,
            };
            tauri::async_runtime::block_on(self.ouverture.demander(move |services, contexte| {
                services.verser_un_fichier(&*source, &commande, contexte)
            }))
            .unwrap_or_else(|refus| panic!("le fichier n° {rang} ne se verse pas : {refus:?}"));
        }

        /// Clôt l'import, qui doit se clore.
        fn clore(&self) -> ImportClos {
            let source = Arc::clone(&self.source);
            let commande = CloreLImport {
                import: self.import.clone(),
                decalage_minutes: 0,
            };
            self.ouverture
                .demander_bloquant(move |services, contexte| {
                    services.clore_l_import(&*source, &commande, contexte)
                })
                .expect("l'import se clôt")
        }
    }

    /// La requête `methode` du chemin `chemin`, sous `http://captation.localhost`, avec l'en-tête
    /// `Range` s'il est donné.
    fn requete(methode: &str, chemin: &str, range: Option<&str>) -> Request<Vec<u8>> {
        let mut requete = Request::builder()
            .method(methode)
            .uri(format!("http://captation.localhost{chemin}"));
        if let Some(range) = range {
            requete = requete.header("Range", range);
        }
        requete
            .body(Vec::new())
            .unwrap_or_else(|e| panic!("la requête {chemin} se construit : {e}"))
    }

    /// La réponse du protocole à `requete`, attendue.
    fn servir(ouverture: &Ouverture, requete: &Request<Vec<u8>>) -> Response<Vec<u8>> {
        tauri::async_runtime::block_on(servir_une_captation(ouverture, requete))
    }

    /// La valeur de l'en-tête `nom` de `reponse`, s'il en a une lisible.
    fn entete<'r>(reponse: &'r Response<Vec<u8>>, nom: &str) -> Option<&'r str> {
        reponse
            .headers()
            .get(nom)
            .and_then(|valeur| valeur.to_str().ok())
    }

    /// Les deux en-têtes que porte chaque réponse du protocole.
    fn porte_les_deux_communs(reponse: &Response<Vec<u8>>) -> bool {
        entete(reponse, "Accept-Ranges") == Some("bytes")
            && entete(reponse, "X-Content-Type-Options") == Some("nosniff")
    }

    #[test]
    fn une_requete_du_protocole_passe_entre_deux_unites() {
        // Point de revue 5 : trois WAV de durées distinctes ; le premier versé, une requête de
        // ses seize premiers octets passe au fil, entre deux unités de l'import.
        let wavs = [wav(1_000, 1), wav(1_500, 2), wav(2_000, 3)];
        let import = ouvrir_un_import("protocole-entre-deux-unites", &wavs);
        import.verser(1);

        let chemin = format!("/{}", empreinte(&wavs[0]));
        let reponse = servir(
            &import.ouverture,
            &requete("GET", &chemin, Some("bytes=0-15")),
        );

        assert_eq!(reponse.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(reponse.body().as_slice(), &wavs[0][..16]);
        assert_eq!(entete(&reponse, "Content-Type"), Some("audio/wav"));
        assert_eq!(entete(&reponse, "Content-Length"), Some("16"));
        assert_eq!(entete(&reponse, "Content-Range"), Some("bytes 0-15/1044"));
        assert!(porte_les_deux_communs(&reponse));
        for rang in [2, 3] {
            import.verser(rang);
        }
        assert_eq!(import.clore().rapport.verses.fichiers, 3);
    }

    #[test]
    fn une_requete_sur_un_chemin_mal_forme_rend_404() {
        let wavs = [wav(1_000, 4)];
        let import = ouvrir_un_import("protocole-chemin-mal-forme", &wavs);
        import.verser(1);
        let e = empreinte(&wavs[0]);
        assert_ne!(e.to_uppercase(), e, "l'empreinte n'a aucune lettre");
        let entiere = servir(&import.ouverture, &requete("GET", &format!("/{e}"), None));
        assert_eq!(entiere.status(), StatusCode::OK);
        assert_eq!(entiere.body(), &wavs[0]);

        for chemin in [
            "/../x".to_string(),
            format!("/{}", e.to_uppercase()),
            format!("/{}", &e[..63]),
            format!("/{e}/x"),
        ] {
            let reponse = servir(&import.ouverture, &requete("GET", &chemin, None));
            assert_eq!(reponse.status(), StatusCode::NOT_FOUND, "{chemin}");
            assert!(reponse.body().is_empty(), "{chemin}");
            assert!(porte_les_deux_communs(&reponse), "{chemin}");
        }
    }

    #[test]
    fn un_refus_du_fil_rend_500_sans_corps() {
        // Une seconde ouverture d'une racine tenue est refusée, et chaque demande au fil rend ce
        // refus, que le protocole ne dit pas.
        let racine = dossier_jetable("protocole-refus-du-fil").join("donnees");
        let premiere = fil::ouvrir(racine.clone());
        assert!(matches!(premiere, Ouverture::Ouverte(_)));
        let ouverture = fil::ouvrir(racine);
        assert!(matches!(ouverture, Ouverture::Refusee(_)));

        let chemin = format!("/{}", empreinte(&wav(1_000, 6)));
        let reponse = servir(&ouverture, &requete("GET", &chemin, None));

        assert_eq!(reponse.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(reponse.body().is_empty());
        assert!(porte_les_deux_communs(&reponse));
        assert_eq!(reponse.headers().len(), 2, "{:?}", reponse.headers());
    }

    /// Le fichier sous `dossier`, en descendant, dont les octets sont `octets` ; il en faut un.
    #[cfg(windows)]
    fn fichier_de(dossier: &std::path::Path, octets: &[u8]) -> PathBuf {
        let mut a_lire = vec![dossier.to_path_buf()];
        while let Some(courant) = a_lire.pop() {
            let entrees = std::fs::read_dir(&courant)
                .unwrap_or_else(|e| panic!("lire {} : {e}", courant.display()));
            for entree in entrees {
                let chemin = entree.expect("une entrée").path();
                if chemin.is_dir() {
                    a_lire.push(chemin);
                } else if std::fs::read(&chemin).is_ok_and(|lus| lus == octets) {
                    return chemin;
                }
            }
        }
        panic!(
            "aucun fichier sous {} ne porte ces octets",
            dossier.display()
        )
    }

    #[cfg(windows)]
    #[test]
    fn la_lecture_d_une_plage_ne_garde_aucun_fichier_ouvert() {
        // Vigilance 5 ; balayage Q3 : le fil reste ouvert, et la racine, que tient son verrou, ne
        // se renomme pas ; le témoin et le renommage portent sur `captations/`, que le fil ne
        // tient pas, et où la lecture d'une plage ouvre puis referme la captation.
        let wavs = [wav(1_000, 5)];
        let import = ouvrir_un_import("protocole-aucun-fichier-ouvert", &wavs);
        import.verser(1);
        let captations = import.racine.join("captations");
        let ailleurs = import.racine.join("captations-ailleurs");
        let rangee = fichier_de(&captations, &wavs[0]);
        // Le témoin : la captation tenue ouverte par `File::open`, comme le stockage l'ouvre.
        let temoin = std::fs::File::open(&rangee)
            .unwrap_or_else(|e| panic!("ouvrir {} : {e}", rangee.display()));
        assert!(
            std::fs::rename(&captations, &ailleurs).is_err(),
            "un fichier tenu ouvert sous captations/ n'en empêche pas le renommage : la prémisse \
             ne tient pas"
        );
        drop(temoin);

        let chemin = format!("/{}", empreinte(&wavs[0]));
        let reponse = servir(
            &import.ouverture,
            &requete("GET", &chemin, Some("bytes=40-47")),
        );
        assert_eq!(reponse.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(reponse.body().as_slice(), &wavs[0][40..48]);

        std::fs::rename(&captations, &ailleurs)
            .unwrap_or_else(|e| panic!("après la plage, captations/ ne se renomme pas : {e}"));
        std::fs::rename(&ailleurs, &captations)
            .unwrap_or_else(|e| panic!("captations/ ne reprend pas son nom : {e}"));
        let reponse = servir(
            &import.ouverture,
            &requete("GET", &chemin, Some("bytes=0-15")),
        );
        assert_eq!(reponse.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(reponse.body().as_slice(), &wavs[0][..16]);
    }
}
