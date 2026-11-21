//! Les captations, rangées par empreinte (§ 31 · 6.1) : chacune sous
//! `captations/<deux premiers caractères>/<empreinte>`, écrite une fois, jamais réécrite, par une
//! copie en flux. Au démarrage, à l'ouverture du fonds, un nom provisoire dont le renommage s'est
//! perdu prend le nom de son empreinte, si le fonds la nomme.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::{self, File, FileType};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use noyau::donnees::demarrage::{self, Provisoire};
use noyau::donnees::import::copie_de;
use services::Versement;

use crate::fichier::{
    copier_et_synchroniser, empreinte_en_flux, rendre_durable_apres_le_renommage, renommer,
    retirer, voisin,
};
use crate::{Avertissement, Erreur, RacineDesDonnees, disque};

/// Les captations d'une racine des données ouverte, dans son dossier `captations/`.
#[derive(Debug)]
pub struct Captations<'r> {
    racine: &'r RacineDesDonnees,
}

impl<'r> Captations<'r> {
    pub(crate) fn de(racine: &'r RacineDesDonnees) -> Captations<'r> {
        Captations { racine }
    }

    /// La racine des données de ces captations : l'adaptateur dit chaque chemin d'un refus
    /// relatif à elle.
    pub(crate) fn racine(&self) -> &'r RacineDesDonnees {
        self.racine
    }
}

impl Captations<'_> {
    /// § 31 · 6.1, la voie par octets : range les octets sous
    /// `captations/<deux premiers caractères>/<empreinte>`, et rend l'empreinte.
    ///
    /// Une captation déjà rangée n'est ni réécrite ni rouverte : l'empreinte, calculée d'abord,
    /// est rendue sans rien écrire. Sinon, les octets passent par la copie en flux
    /// ([`Captations::verser_en_flux`]), `base` l'empreinte, sans empreinte attendue : le nom
    /// provisoire est `<empreinte>.<processus>.partiel`, ou, si ce nom est pris,
    /// `<empreinte>.<processus>-2.partiel`, `-3`…, et ses refus sont les siens.
    pub fn verser(&self, octets: &[u8]) -> Result<String, Erreur> {
        let e = crate::empreinte(octets);
        let rangee = self.chemin_de(&e);
        if rangee
            .try_exists()
            .map_err(|erreur| disque(&rangee, erreur))?
        {
            return Ok(e);
        }
        let mut flux = octets;
        self.verser_en_flux(&e, &mut flux, None)
            .map(|versement| versement.empreinte)
    }

    /// § 31 · 6.1, la copie en flux : copie `flux` sous un nom provisoire formé sur `base`, puis le
    /// range sous l'empreinte calculée, et rend le [`Versement`].
    ///
    /// `captations/` se liste d'abord : un dossier qui ne se liste pas rend [`Erreur::Disque`],
    /// sur lui, avant tout nom provisoire ; le nom provisoire est `voisin(captations/<base>)`.
    /// Chaque tronçon du flux, d'au plus 1 048 576 octets, s'y écrit, l'empreinte calculée en
    /// chemin (`Hacheur`) ; il est synchronisé, fermé, puis relu en flux contre elle. Une captation
    /// déjà rangée sous cette empreinte se cherche alors (`try_exists`, `deja_rangee`). Une
    /// empreinte autre qu'`attendue` : le nom provisoire se retire, rien n'est rangé, `rangee`
    /// faux (R20) ; déjà rangée : le nom provisoire se retire, et la captation rangée n'est ni
    /// réécrite ni rouverte (question 8) ; sinon, le sous-dossier se crée au besoin, et, hors de
    /// Windows, `captations/` se synchronise alors ; le nom provisoire est renommé à l'empreinte,
    /// et le renommage rendu durable. Chaque étape passe par sa fonction de `fichier`, comme une
    /// écriture à côté. Le retrait d'un nom provisoire qui ne se range pas est tu s'il est
    /// refusé : il reste, ne se range jamais par-dessus une captation rangée
    /// (`demarrage::a_ranger`), et la reprise ou la clôture d'un import retire ses copies.
    ///
    /// Un refus avant la fin du renommage retire le nom provisoire, puis se rend :
    /// [`Erreur::FichierAVerser`] si la lecture du flux échoue, [`Erreur::CaptationAlteree`] si
    /// la relecture ne donne pas l'empreinte, [`Erreur::Disque`] sinon. Un refus de rendre le
    /// renommage durable arrive après lui : il se rend, mais la captation est déjà rangée,
    /// entière.
    pub fn verser_en_flux(
        &self,
        base: &str,
        flux: &mut dyn io::Read,
        attendue: Option<&str>,
    ) -> Result<Versement, Erreur> {
        let dossier = self.dossier();
        let provisoire = voisin(&dossier.join(base)).map_err(|erreur| disque(&dossier, erreur))?;
        let versement = match self.copier_puis_ranger(&provisoire, flux, attendue) {
            Ok(versement) => versement,
            Err(refus) => {
                // Le nom provisoire peut ne pas être né : son retrait peut alors échouer, et le
                // refus rendu reste celui de la copie.
                let _ = retirer(&provisoire);
                return Err(refus);
            }
        };
        if versement.rangee && !versement.deja_rangee {
            let rangee = self.chemin_de(&versement.empreinte);
            rendre_durable_apres_le_renommage(&rangee).map_err(|erreur| disque(&rangee, erreur))?;
        }
        Ok(versement)
    }

    /// Relit une captation contre son empreinte.
    ///
    /// Un nom qui n'est pas une empreinte, 64 caractères hexadécimaux minuscules, ne nomme
    /// aucune captation : [`Erreur::CaptationAbsente`], comme une empreinte qu'aucun fichier ne
    /// porte. Un contenu qui ne donne pas son empreinte rend [`Erreur::CaptationAlteree`].
    pub fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Erreur> {
        let chemin = self.chemin_d_une_captation(empreinte)?;
        let octets = match fs::read(&chemin) {
            Ok(octets) => octets,
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => {
                return Err(absente(empreinte));
            }
            Err(erreur) => return Err(disque(&chemin, erreur)),
        };
        if crate::empreinte(&octets) != empreinte {
            return Err(Erreur::CaptationAlteree {
                empreinte: empreinte.to_string(),
            });
        }
        Ok(octets)
    }

    /// Au plus `longueur` octets de la captation `empreinte`, depuis `debut` (`seek`, puis
    /// `take`), le fichier refermé au retour ; une plage qui commence à la fin, ou après, n'en
    /// rend aucun. Une plage ne se relit pas contre l'empreinte. Un nom qui n'est pas une
    /// empreinte, ou qu'aucun fichier ne porte : [`Erreur::CaptationAbsente`], comme `relire`.
    pub fn relire_une_plage(
        &self,
        empreinte: &str,
        debut: u64,
        longueur: u64,
    ) -> Result<Vec<u8>, Erreur> {
        let chemin = self.chemin_d_une_captation(empreinte)?;
        let mut fichier = match File::open(&chemin) {
            Ok(fichier) => fichier,
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => {
                return Err(absente(empreinte));
            }
            Err(erreur) => return Err(disque(&chemin, erreur)),
        };
        fichier
            .seek(SeekFrom::Start(debut))
            .map_err(|erreur| disque(&chemin, erreur))?;
        let mut octets = Vec::new();
        fichier
            .take(longueur)
            .read_to_end(&mut octets)
            .map_err(|erreur| disque(&chemin, erreur))?;
        Ok(octets)
    }

    /// Les octets de la captation `empreinte` (`fs::metadata`), `None` si aucune ne la porte, ou
    /// si le nom n'est pas une empreinte.
    pub fn taille(&self, empreinte: &str) -> Result<Option<u64>, Erreur> {
        let Ok(chemin) = self.chemin_d_une_captation(empreinte) else {
            return Ok(None);
        };
        match fs::metadata(&chemin) {
            Ok(metadonnees) => Ok(Some(metadonnees.len())),
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(erreur) => Err(disque(&chemin, erreur)),
        }
    }

    /// Les tailles des captations rangées, chacune une fois : les fichiers des sous-dossiers de
    /// `captations/`, comme pour `verifier` ; les noms provisoires n'y entrent pas. Un refus du
    /// disque rend [`Erreur::Disque`], sur le dossier ou le fichier en cause.
    pub fn tailles(&self) -> Result<BTreeSet<u64>, Erreur> {
        let mut tailles = BTreeSet::new();
        for (sous_dossier, sorte) in lire_le_dossier(&self.dossier())? {
            if !sorte.is_dir() {
                continue;
            }
            for (chemin, sorte) in lire_le_dossier(&sous_dossier)? {
                if sorte.is_file() {
                    let metadonnees =
                        fs::metadata(&chemin).map_err(|erreur| disque(&chemin, erreur))?;
                    tailles.insert(metadonnees.len());
                }
            }
        }
        Ok(tailles)
    }

    /// Les noms provisoires de `captations/` que `import::copie_de` dit de l'import `import`,
    /// triés par nom, chacun lu en flux (`lire_un_nom_provisoire`) : son empreinte, et si elle est
    /// rangée. Un dossier qui ne se liste pas, ou une copie qui ne se lit pas, rend
    /// [`Erreur::Disque`], sur lui ou sur elle.
    pub fn copies_de_l_import(&self, import: &str) -> Result<Vec<Provisoire>, Erreur> {
        let dossier = self.dossier();
        let mut noms: Vec<String> = lire_le_dossier(&dossier)?
            .into_iter()
            .filter(|(_, sorte)| sorte.is_file())
            .filter_map(|(chemin, _)| Some(chemin.file_name()?.to_str()?.to_string()))
            .filter(|nom| copie_de(nom).is_some_and(|(de, _)| de == import))
            .collect();
        noms.sort();
        noms.into_iter()
            .map(|nom| {
                self.lire_un_nom_provisoire(&nom)
                    .map_err(|erreur| disque(&dossier.join(&nom), erreur))
            })
            .collect()
    }

    /// Retire le nom provisoire `nom` de `captations/` (`fichier::retirer`). Un nom que
    /// `import::copie_de` ne reconnaît pas n'est pas une copie d'un import, et ne se retire pas ;
    /// une copie déjà retirée non plus. Un refus du disque rend [`Erreur::Disque`], sur elle.
    pub fn retirer_une_copie(&self, nom: &str) -> Result<(), Erreur> {
        if copie_de(nom).is_none() {
            return Ok(());
        }
        let chemin = self.dossier().join(nom);
        match retirer(&chemin) {
            Ok(()) => Ok(()),
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(erreur) => Err(disque(&chemin, erreur)),
        }
    }

    /// Les captations rangées dont le contenu trahit le nom, ou rangées hors du sous-dossier de
    /// leurs deux premiers caractères. Les noms provisoires de `captations/` n'y entrent pas.
    ///
    /// Une captation rangée est un fichier d'un sous-dossier de `captations/` ; un fichier de
    /// `captations/` même porte un nom provisoire. Les chemins rendus sont dans l'ordre.
    pub fn verifier(&self) -> Result<Vec<PathBuf>, Erreur> {
        let mut trahies = Vec::new();
        for (sous_dossier, sorte) in lire_le_dossier(&self.dossier())? {
            if !sorte.is_dir() {
                continue;
            }
            let prefixe = sous_dossier.file_name().and_then(OsStr::to_str);
            for (chemin, sorte) in lire_le_dossier(&sous_dossier)? {
                if sorte.is_file() && trahit_son_nom(&chemin, prefixe)? {
                    trahies.push(chemin);
                }
            }
        }
        trahies.sort();
        Ok(trahies)
    }

    /// Range sous son empreinte chaque nom provisoire que `a_ranger` désigne (§ 31 · 6.1) ; rend
    /// ceux qui n'ont pu l'être. `nommees` : les empreintes que le fonds nomme par les clés
    /// `<empreinte>/<clé>` des trois conteneurs qui nomment une captation, les prises, les MIDI
    /// versés seuls et les textes versés (`services::lecture::empreintes_nommees`), telles qu'elles
    /// se lisent, même en lecture seule.
    ///
    /// Les noms provisoires sont les fichiers de `captations/` même, triés par nom, chacun lu en
    /// flux par `lire_un_nom_provisoire`, sans le tenir en mémoire : son nom, l'empreinte de son
    /// contenu, et si une captation de cette empreinte est déjà rangée (`try_exists`). Une copie
    /// en flux d'un import, entière et nommée, se range ainsi, comme la copie d'une capture ; une
    /// copie coupée entre deux tronçons ne donne pas l'empreinte que le fonds nomme, et reste
    /// (question 31). Un nom qui n'est pas de l'UTF-8 n'en est pas un, `voisin` n'en formant
    /// aucun : il n'est pas lu. Pour chaque nom provisoire que `a_ranger` désigne, le sous-dossier
    /// de l'empreinte se crée au besoin (et, hors de Windows, `captations/` se synchronise alors),
    /// le nom provisoire est renommé à l'empreinte (`fichier::renommer`), puis le renommage est
    /// rendu durable (`rendre_durable_apres_le_renommage`).
    ///
    /// Rien n'y fait échouer l'ouverture, et aucun nom provisoire n'est retiré : une captation se
    /// verse toujours. Un nom provisoire qui ne se lit pas, ou dont la captation rangée ne se
    /// cherche pas, n'entre pas dans `a_ranger`, reste intact, et donne
    /// [`Avertissement::NomProvisoireRestant`], comme un sous-dossier qui ne se crée pas ou un
    /// renommage refusé ; un dossier `captations/` qui ne se liste pas en donne un, sur ce
    /// dossier, et rien ne se range. Un renommage non rendu durable laisse la captation rangée,
    /// avec [`Avertissement::NonDurable`].
    pub(crate) fn ranger_les_noms_provisoires(
        &self,
        nommees: &BTreeSet<String>,
    ) -> Vec<Avertissement> {
        let dossier = self.dossier();
        let entrees = match entrees_du_dossier(&dossier) {
            Ok(entrees) => entrees,
            Err(erreur) => {
                return vec![Avertissement::NomProvisoireRestant {
                    chemin: dossier,
                    erreur,
                }];
            }
        };
        let mut noms: Vec<String> = entrees
            .into_iter()
            .filter(|(_, sorte)| sorte.is_file())
            .filter_map(|(chemin, _)| Some(chemin.file_name()?.to_str()?.to_string()))
            .collect();
        noms.sort();
        let mut avertissements = Vec::new();
        let mut provisoires = Vec::new();
        for nom in noms {
            match self.lire_un_nom_provisoire(&nom) {
                Ok(provisoire) => provisoires.push(provisoire),
                Err(erreur) => avertissements.push(Avertissement::NomProvisoireRestant {
                    chemin: dossier.join(&nom),
                    erreur,
                }),
            }
        }
        for (nom, e) in demarrage::a_ranger(&provisoires, nommees) {
            let provisoire = dossier.join(&nom);
            let rangee = self.chemin_de(&e);
            let renomme = self
                .creer_le_sous_dossier(&rangee)
                .map_err(|(_, erreur)| erreur)
                .and_then(|()| renommer(&provisoire, &rangee));
            if let Err(erreur) = renomme {
                avertissements.push(Avertissement::NomProvisoireRestant {
                    chemin: provisoire,
                    erreur,
                });
            } else if let Err(erreur) = rendre_durable_apres_le_renommage(&rangee) {
                avertissements.push(Avertissement::NonDurable {
                    chemin: rangee,
                    erreur,
                });
            }
        }
        avertissements
    }

    /// Le nom provisoire `nom` de `captations/`, lu en flux par `fichier::empreinte_en_flux`, sans
    /// le tenir en mémoire (vigilance 3) : son nom, l'empreinte de son contenu, et si une
    /// captation de cette empreinte est déjà rangée (`try_exists`). Un refus du disque rend son
    /// erreur.
    fn lire_un_nom_provisoire(&self, nom: &str) -> io::Result<Provisoire> {
        let e = empreinte_en_flux(&self.dossier().join(nom))?;
        let deja_rangee = self.chemin_de(&e).try_exists()?;
        Ok(Provisoire {
            nom: nom.to_string(),
            empreinte: e,
            deja_rangee,
        })
    }

    /// Copie `flux` sous le nom provisoire, enveloppé : une erreur de sa lecture se retient avant
    /// de remonter, et rend [`Erreur::FichierAVerser`], non un refus d'écrire le nom provisoire,
    /// [`Erreur::Disque`] sur lui. Puis relit le nom provisoire en flux contre l'empreinte
    /// calculée, cherche une captation déjà rangée, et, d'accord avec `attendue` et non déjà
    /// rangée, crée au besoin le sous-dossier, puis renomme le nom provisoire à l'empreinte ;
    /// sinon, retire le nom provisoire, son refus tu.
    fn copier_puis_ranger(
        &self,
        provisoire: &Path,
        flux: &mut dyn io::Read,
        attendue: Option<&str>,
    ) -> Result<Versement, Erreur> {
        let mut enveloppe = FluxAVerser {
            flux,
            lecture_refusee: false,
        };
        let (e, octets) = match copier_et_synchroniser(provisoire, &mut enveloppe) {
            Ok(copie) => copie,
            Err(erreur) if enveloppe.lecture_refusee => {
                return Err(Erreur::FichierAVerser { erreur });
            }
            Err(erreur) => return Err(disque(provisoire, erreur)),
        };
        let relue = empreinte_en_flux(provisoire).map_err(|erreur| disque(provisoire, erreur))?;
        if relue != e {
            return Err(Erreur::CaptationAlteree { empreinte: e });
        }
        let rangee = self.chemin_de(&e);
        let deja_rangee = rangee
            .try_exists()
            .map_err(|erreur| disque(&rangee, erreur))?;
        let accord = attendue.is_none_or(|attendue| attendue == e);
        if deja_rangee || !accord {
            let _ = retirer(provisoire);
            return Ok(Versement {
                empreinte: e,
                octets,
                deja_rangee,
                rangee: accord,
            });
        }
        self.creer_le_sous_dossier(&rangee)
            .map_err(|(chemin, erreur)| disque(&chemin, erreur))?;
        renommer(provisoire, &rangee).map_err(|erreur| disque(&rangee, erreur))?;
        Ok(Versement {
            empreinte: e,
            octets,
            deja_rangee: false,
            rangee: true,
        })
    }

    /// `captations/<deux premiers caractères>/<empreinte>`, si `empreinte` en est une ; sinon,
    /// elle ne nomme aucune captation : [`Erreur::CaptationAbsente`].
    fn chemin_d_une_captation(&self, empreinte: &str) -> Result<PathBuf, Erreur> {
        if est_une_empreinte(empreinte) {
            Ok(self.chemin_de(empreinte))
        } else {
            Err(absente(empreinte))
        }
    }

    /// Crée au besoin le sous-dossier de la captation rangée `rangee` (`fs::create_dir`) ; déjà
    /// là, il est laissé tel quel. Né, hors de Windows, il ne survit à une coupure qu'une fois
    /// `captations/` synchronisé : `rendre_durable` synchronise le dossier de son chemin. Un
    /// dossier ne se synchronise pas sous Windows. Ce n'est pas le renommage rendu durable :
    /// aucune panne ne s'y injecte. Si cette synchronisation échoue, le sous-dossier, vide, se
    /// retire (`fs::remove_dir`, son refus tu) avant que le refus se rende : un sous-dossier
    /// présent est ainsi un dossier dont la création a été rendue durable, et la tentative
    /// suivante le recrée, puis le synchronise. Un refus du disque rend son erreur, avec le
    /// dossier du geste refusé : le sous-dossier, s'il ne se crée pas ; `captations/`, s'il ne se
    /// synchronise pas.
    fn creer_le_sous_dossier(&self, rangee: &Path) -> Result<(), (PathBuf, io::Error)> {
        let sous_dossier = rangee.parent().unwrap_or(rangee);
        match fs::create_dir(sous_dossier) {
            Ok(()) => {
                #[cfg(not(windows))]
                if let Err(erreur) = crate::fichier::rendre_durable(sous_dossier) {
                    let _ = fs::remove_dir(sous_dossier);
                    return Err((self.dossier(), erreur));
                }
                Ok(())
            }
            Err(erreur) if erreur.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(erreur) => Err((sous_dossier.to_path_buf(), erreur)),
        }
    }

    /// Le dossier `captations/` de la racine des données.
    fn dossier(&self) -> PathBuf {
        self.racine.chemin().join("captations")
    }

    /// `captations/<deux premiers caractères>/<e>`, pour une empreinte `e`.
    fn chemin_de(&self, e: &str) -> PathBuf {
        self.dossier().join(&e[..2]).join(e)
    }
}

/// Le flux à verser, enveloppé par la copie en flux : une erreur de sa lecture, hors d'une lecture
/// interrompue, que la copie reprend, se retient (`lecture_refusee`) avant de remonter.
struct FluxAVerser<'f> {
    flux: &'f mut dyn io::Read,
    lecture_refusee: bool,
}

impl io::Read for FluxAVerser<'_> {
    fn read(&mut self, tampon: &mut [u8]) -> io::Result<usize> {
        self.flux.read(tampon).inspect_err(|erreur| {
            if erreur.kind() != io::ErrorKind::Interrupted {
                self.lecture_refusee = true;
            }
        })
    }
}

/// Le refus d'une captation qu'aucun fichier ne porte, ou d'un nom qui n'est pas une empreinte.
fn absente(empreinte: &str) -> Erreur {
    Erreur::CaptationAbsente {
        empreinte: empreinte.to_string(),
    }
}

/// 64 caractères hexadécimaux minuscules : le seul nom que porte une captation.
fn est_une_empreinte(nom: &str) -> bool {
    nom.len() == 64
        && nom
            .bytes()
            .all(|octet| matches!(octet, b'0'..=b'9' | b'a'..=b'f'))
}

/// Une captation rangée hors du sous-dossier `sous_dossier` de ses deux premiers caractères,
/// ou dont le contenu ne donne pas l'empreinte de son nom.
fn trahit_son_nom(chemin: &Path, sous_dossier: Option<&str>) -> Result<bool, Erreur> {
    let Some(nom) = chemin.file_name().and_then(OsStr::to_str) else {
        return Ok(true);
    };
    if nom.get(..2) != sous_dossier {
        return Ok(true);
    }
    let octets = fs::read(chemin).map_err(|erreur| disque(chemin, erreur))?;
    Ok(crate::empreinte(&octets) != nom)
}

/// Ce que contient `dossier` : le chemin et la sorte de chaque entrée. Un refus du disque rend
/// [`Erreur::Disque`], sur `dossier`.
fn lire_le_dossier(dossier: &Path) -> Result<Vec<(PathBuf, FileType)>, Erreur> {
    entrees_du_dossier(dossier).map_err(|erreur| disque(dossier, erreur))
}

/// Ce que contient `dossier` : le chemin et la sorte de chaque entrée, ou le refus du disque.
fn entrees_du_dossier(dossier: &Path) -> io::Result<Vec<(PathBuf, FileType)>> {
    let mut entrees = Vec::new();
    for entree in fs::read_dir(dossier)? {
        let entree = entree?;
        entrees.push((entree.path(), entree.file_type()?));
    }
    Ok(entrees)
}
