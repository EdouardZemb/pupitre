//! Les captations, rangées par empreinte (§ 31 · 6.1) : chacune sous
//! `captations/<deux premiers caractères>/<empreinte>`, écrite une fois, jamais réécrite. Au
//! démarrage, à l'ouverture du fonds, un nom provisoire dont le renommage s'est perdu prend le nom
//! de son empreinte, si le fonds la nomme.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs::{self, FileType};
use std::io;
use std::path::{Path, PathBuf};

use noyau::donnees::demarrage::{self, Provisoire};

use crate::fichier::{
    ecrire_et_synchroniser, lire, rendre_durable_apres_le_renommage, renommer, retirer, voisin,
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
}

impl Captations<'_> {
    /// § 31 · 6.1 : range les octets sous `captations/<deux premiers caractères>/<empreinte>`,
    /// et rend l'empreinte.
    ///
    /// Une captation déjà rangée n'est ni réécrite ni rouverte : l'empreinte est rendue sans
    /// rien écrire. Sinon, les octets s'écrivent sous le nom provisoire
    /// `voisin(captations/<empreinte>)`, `<empreinte>.<processus>.partiel`, ou, si ce nom est
    /// pris, `<empreinte>.<processus>-2.partiel`, `-3`… ; il est synchronisé, fermé, puis relu
    /// contre l'empreinte ; le sous-dossier se crée au besoin, et, hors de Windows,
    /// `captations/` se synchronise alors ; le nom provisoire est renommé à l'empreinte, et le
    /// renommage rendu durable. Chaque étape passe par sa fonction de `fichier`, comme une
    /// écriture à côté. Un dossier `captations/` qui ne se liste pas rend [`Erreur::Disque`],
    /// sur ce dossier, avant tout nom provisoire. Un refus avant la fin du renommage retire le nom
    /// provisoire, puis se rend : [`Erreur::CaptationAlteree`] si la relecture ne donne pas
    /// l'empreinte. Un refus de rendre le renommage durable arrive après lui : il se rend, mais
    /// la captation est déjà rangée, entière.
    pub fn verser(&self, octets: &[u8]) -> Result<String, Erreur> {
        let e = crate::empreinte(octets);
        let rangee = self.chemin_de(&e);
        if rangee
            .try_exists()
            .map_err(|erreur| disque(&rangee, erreur))?
        {
            return Ok(e);
        }
        let provisoire =
            voisin(&self.dossier().join(&e)).map_err(|erreur| disque(&self.dossier(), erreur))?;
        if let Err(refus) = self.ranger(&provisoire, &rangee, octets, &e) {
            // Le nom provisoire peut ne pas être né : son retrait peut alors échouer, et le
            // refus rendu reste celui du rangement.
            let _ = retirer(&provisoire);
            return Err(refus);
        }
        rendre_durable_apres_le_renommage(&rangee).map_err(|erreur| disque(&rangee, erreur))?;
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

    /// Range sous son empreinte chaque nom provisoire que `a_ranger` désigne (§ 31 · 6.1) ; rend
    /// ceux qui n'ont pu l'être. `nommees` : les empreintes que le fonds nomme par les clés
    /// `<empreinte>/<clé>` des trois conteneurs qui nomment une captation, les prises, les MIDI
    /// versés seuls et les textes versés (`services::lecture::empreintes_nommees`), telles qu'elles
    /// se lisent, même en lecture seule.
    ///
    /// Les noms provisoires sont les fichiers de `captations/` même, triés par nom, chacun lu en
    /// entier par `fichier::lire` : son nom, l'empreinte de son contenu, et si une captation de
    /// cette empreinte est déjà rangée (`try_exists`). Un nom qui n'est pas de l'UTF-8 n'en est
    /// pas un, `voisin` n'en formant aucun : il n'est pas lu. Pour chaque nom provisoire que
    /// `a_ranger` désigne, le sous-dossier de l'empreinte se crée au besoin (et, hors de Windows,
    /// `captations/` se synchronise alors), le nom provisoire est renommé à l'empreinte
    /// (`fichier::renommer`), puis le renommage est rendu durable
    /// (`rendre_durable_apres_le_renommage`).
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

    /// Le nom provisoire `nom` de `captations/`, lu en entier par `fichier::lire` : son nom,
    /// l'empreinte de son contenu, et si une captation de cette empreinte est déjà rangée
    /// (`try_exists`). Un refus du disque rend son erreur.
    fn lire_un_nom_provisoire(&self, nom: &str) -> io::Result<Provisoire> {
        let e = crate::empreinte(&lire(&self.dossier().join(nom))?);
        let deja_rangee = self.chemin_de(&e).try_exists()?;
        Ok(Provisoire {
            nom: nom.to_string(),
            empreinte: e,
            deja_rangee,
        })
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
        ecrire_et_synchroniser(provisoire, octets).map_err(|erreur| disque(provisoire, erreur))?;
        let relus = lire(provisoire).map_err(|erreur| disque(provisoire, erreur))?;
        if crate::empreinte(&relus) != e {
            return Err(Erreur::CaptationAlteree {
                empreinte: e.to_string(),
            });
        }
        self.creer_le_sous_dossier(rangee)
            .map_err(|(chemin, erreur)| disque(&chemin, erreur))?;
        renommer(provisoire, rangee).map_err(|erreur| disque(rangee, erreur))
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
