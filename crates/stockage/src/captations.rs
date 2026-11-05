//! Les captations, rangées par empreinte (§ 31 · 6.1) : chacune sous
//! `captations/<deux premiers caractères>/<empreinte>`, écrite une fois, jamais réécrite.

use std::ffi::OsStr;
use std::fs::{self, File, FileType};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::fichier::{rendre_durable, voisin};
use crate::{Erreur, RacineDesDonnees, disque};

/// Les captations d'une racine des données ouverte, dans son dossier `captations/`.
#[derive(Debug)]
pub struct Captations<'r> {
    racine: &'r RacineDesDonnees,
}

impl<'r> Captations<'r> {
    pub(crate) fn de(racine: &'r RacineDesDonnees) -> Captations<'r> {
        Captations { racine }
    }
}

impl Captations<'_> {
    /// § 31 · 6.1 : range les octets sous `captations/<deux premiers caractères>/<empreinte>`,
    /// et rend l'empreinte.
    ///
    /// Une captation déjà rangée n'est ni réécrite ni rouverte : l'empreinte est rendue sans
    /// rien écrire. Sinon, les octets s'écrivent sous le nom provisoire
    /// `voisin(captations/<empreinte>)`, synchronisé, fermé, puis relu contre l'empreinte ; le
    /// sous-dossier se crée au besoin, et, hors de Windows, `captations/` se synchronise alors ;
    /// le nom provisoire est renommé à l'empreinte, et le renommage rendu durable. Un refus
    /// avant la fin du renommage retire le nom provisoire, puis se rend :
    /// [`Erreur::CaptationAlteree`] si la relecture ne donne pas l'empreinte.
    pub fn verser(&self, octets: &[u8]) -> Result<String, Erreur> {
        let e = crate::empreinte(octets);
        let rangee = self.chemin_de(&e);
        if rangee
            .try_exists()
            .map_err(|erreur| disque(&rangee, erreur))?
        {
            return Ok(e);
        }
        let provisoire = voisin(&self.dossier().join(&e));
        if let Err(refus) = self.ranger(&provisoire, &rangee, octets, &e) {
            // Le nom provisoire peut ne pas être né : son retrait peut alors échouer, et le
            // refus rendu reste celui du rangement.
            let _ = fs::remove_file(&provisoire);
            return Err(refus);
        }
        rendre_durable(&rangee).map_err(|erreur| disque(&rangee, erreur))?;
        Ok(e)
    }

    /// Relit une captation contre son empreinte.
    ///
    /// Un nom qui n'est pas une empreinte, 64 caractères hexadécimaux minuscules, ne nomme
    /// aucune captation : [`Erreur::CaptationAbsente`], comme une empreinte qu'aucun fichier ne
    /// porte. Un contenu qui ne donne pas son empreinte rend [`Erreur::CaptationAlteree`].
    pub fn relire(&self, empreinte: &str) -> Result<Vec<u8>, Erreur> {
        let absente = || Erreur::CaptationAbsente {
            empreinte: empreinte.to_string(),
        };
        if !est_une_empreinte(empreinte) {
            return Err(absente());
        }
        let chemin = self.chemin_de(empreinte);
        let octets = match fs::read(&chemin) {
            Ok(octets) => octets,
            Err(erreur) if erreur.kind() == io::ErrorKind::NotFound => return Err(absente()),
            Err(erreur) => return Err(disque(&chemin, erreur)),
        };
        if crate::empreinte(&octets) != empreinte {
            return Err(Erreur::CaptationAlteree {
                empreinte: empreinte.to_string(),
            });
        }
        Ok(octets)
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

    /// Écrit le nom provisoire, le relit contre `e`, crée au besoin le sous-dossier de
    /// `rangee`, puis renomme le nom provisoire en `rangee`.
    fn ranger(
        &self,
        provisoire: &Path,
        rangee: &Path,
        octets: &[u8],
        e: &str,
    ) -> Result<(), Erreur> {
        ecrire(provisoire, octets).map_err(|erreur| disque(provisoire, erreur))?;
        let relus = fs::read(provisoire).map_err(|erreur| disque(provisoire, erreur))?;
        if crate::empreinte(&relus) != e {
            return Err(Erreur::CaptationAlteree {
                empreinte: e.to_string(),
            });
        }
        let sous_dossier = rangee.parent().unwrap_or(rangee);
        match fs::create_dir(sous_dossier) {
            Ok(()) => {
                // Hors de Windows, le sous-dossier né ne survit à une coupure qu'une fois
                // `captations/` synchronisé : `rendre_durable` synchronise le dossier de son
                // chemin. Un dossier ne se synchronise pas sous Windows.
                #[cfg(not(windows))]
                rendre_durable(sous_dossier).map_err(|erreur| disque(&self.dossier(), erreur))?;
            }
            Err(erreur) if erreur.kind() == io::ErrorKind::AlreadyExists => {}
            Err(erreur) => return Err(disque(sous_dossier, erreur)),
        }
        fs::rename(provisoire, rangee).map_err(|erreur| disque(rangee, erreur))
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

/// Écrit `octets` dans `chemin`, le synchronise (`sync_all`), puis le ferme.
fn ecrire(chemin: &Path, octets: &[u8]) -> io::Result<()> {
    let mut fichier = File::create(chemin)?;
    fichier.write_all(octets)?;
    fichier.sync_all()
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

/// Ce que contient `dossier` : le chemin et la sorte de chaque entrée.
fn lire_le_dossier(dossier: &Path) -> Result<Vec<(PathBuf, FileType)>, Erreur> {
    let refus = |erreur| disque(dossier, erreur);
    let mut entrees = Vec::new();
    for entree in fs::read_dir(dossier).map_err(refus)? {
        let entree = entree.map_err(refus)?;
        entrees.push((entree.path(), entree.file_type().map_err(refus)?));
    }
    Ok(entrees)
}
