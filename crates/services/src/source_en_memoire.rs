//! La source à verser en mémoire (§ 40 · 3, le troisième contrat) : un dossier que les tests et le
//! générateur du fonds de mesure écrivent, sans disque. Hors de la fonction `en-memoire` : le
//! générateur du test 13 et la tranche 7 l'emploient.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::io;

use noyau::donnees::signature;

use crate::contrats::plage;
use crate::{Entree, Raison, Refus, SorteDEntree, Source, l_entree, le_fichier};

/// Le constat d'une lecture refusée du fichier à verser, en mémoire : celui de la source refusée
/// (`SourceEnMemoire::refuser`) et d'un flux qui ne se lit plus (`CaptationsEnMemoire`).
pub(crate) const FICHIER_A_VERSER_REFUSE: &str = "Le fichier à verser, le disque l'a refusé.";

/// Une source à verser en mémoire : le nom du dossier, ses entrées telles que le parcours les a
/// vues, le contenu et la marque de dernière écriture de chacune à cet instant, celles dont la
/// lecture est refusée, et les octets lus de chacune.
#[derive(Debug)]
pub struct SourceEnMemoire {
    nom: Option<String>,
    entrees: Vec<Entree>,
    contenus: Vec<Vec<u8>>,
    marques: Vec<u64>,
    refusees: Vec<bool>,
    lus: RefCell<Vec<u64>>,
}

impl SourceEnMemoire {
    /// `fichiers` : (chemin, contenu), un chemin `None` étant un nom sans forme UTF-8 ; `liens` :
    /// des chemins ; rangés par leurs segments, puis numérotés dès 1.
    ///
    /// Les segments se comparent un à un, octet pour octet ; une entrée sans chemin vient après
    /// celles qui en ont un, dans l'ordre reçu. Un fichier a pour octets ceux de son contenu ; un
    /// lien, 0, rien ne s'en lit. L'extension est celle du dernier segment
    /// (`signature::extension`). Chaque entrée naît avec la marque de dernière écriture 1.
    pub fn new(
        nom_du_dossier: Option<&str>,
        fichiers: Vec<(Option<String>, Vec<u8>)>,
        liens: Vec<String>,
    ) -> Self {
        let mut vues: Vec<(Option<String>, Vec<u8>, SorteDEntree)> = fichiers
            .into_iter()
            .map(|(chemin, contenu)| (chemin, contenu, SorteDEntree::Fichier))
            .chain(
                liens
                    .into_iter()
                    .map(|chemin| (Some(chemin), Vec::new(), SorteDEntree::Lien)),
            )
            .collect();
        vues.sort_by(|a, b| par_segments(a.0.as_deref(), b.0.as_deref()));
        let mut entrees = Vec::with_capacity(vues.len());
        let mut contenus = Vec::with_capacity(vues.len());
        for (rang, (chemin, contenu, sorte)) in (1..).zip(vues) {
            let extension = chemin
                .as_deref()
                .and_then(|chemin| chemin.rsplit('/').next())
                .and_then(signature::extension)
                .map(str::to_string);
            entrees.push(Entree {
                rang,
                chemin,
                extension,
                octets: contenu.len() as u64,
                sorte,
                marque: Some(1),
            });
            contenus.push(contenu);
        }
        let nombre = entrees.len();
        SourceEnMemoire {
            nom: nom_du_dossier.map(str::to_string),
            entrees,
            contenus,
            marques: vec![1; nombre],
            refusees: vec![false; nombre],
            lus: RefCell::new(vec![0; nombre]),
        }
    }

    /// Change le contenu de l'entrée `rang`, et avance sa marque de dernière écriture d'une unité
    /// (question 14). Ses octets et sa marque au parcours, que rend `entrees`, ne changent pas.
    /// Un rang sans entrée est une faute du test : elle panique.
    pub fn changer(&mut self, rang: u64, contenu: Vec<u8>) {
        let indice = self.indice_du_test(rang);
        self.contenus[indice] = contenu;
        self.marques[indice] += 1;
    }

    /// L'entrée `rang` refuse toute lecture, de `Raison::Disque`, tant que `refusee` est vrai
    /// (question 38 ; V2) : `lire` et `lire_en_flux`. Un rang sans entrée est une faute du test :
    /// elle panique.
    pub fn refuser(&mut self, rang: u64, refusee: bool) {
        let indice = self.indice_du_test(rang);
        self.refusees[indice] = refusee;
    }

    /// L'indice de l'entrée `rang`, pour un geste du test ; un rang sans entrée panique.
    fn indice_du_test(&self, rang: u64) -> usize {
        l_entree(&self.entrees, rang).unwrap_or_else(|refus| panic!("{}", refus.message))
    }

    /// L'indice du fichier `rang`, qui se lit : un rang sans entrée ou une entrée qui n'est pas un
    /// fichier ([`le_fichier`] : `Raison::Impossible`), ou dont la lecture est refusée
    /// (`Raison::Disque`), ne se lit pas.
    fn fichier_lisible(&self, rang: u64) -> Result<usize, Refus> {
        let indice = le_fichier(&self.entrees, rang)?;
        if self.refusees[indice] {
            return Err(Refus {
                raison: Raison::Disque,
                message: FICHIER_A_VERSER_REFUSE.to_string(),
            });
        }
        Ok(indice)
    }
}

impl Source for SourceEnMemoire {
    fn nom_du_dossier(&self) -> Option<&str> {
        self.nom.as_deref()
    }

    fn entrees(&self) -> &[Entree] {
        &self.entrees
    }

    fn lire(&self, rang: u64, debut: u64, longueur: usize) -> Result<Vec<u8>, Refus> {
        let indice = self.fichier_lisible(rang)?;
        let octets = plage(&self.contenus[indice], debut, longueur as u64);
        self.lus.borrow_mut()[indice] += octets.len() as u64;
        Ok(octets.to_vec())
    }

    fn lire_en_flux(&self, rang: u64) -> Result<Box<dyn io::Read + '_>, Refus> {
        let indice = self.fichier_lisible(rang)?;
        Ok(Box::new(FluxEnMemoire {
            reste: &self.contenus[indice],
            lus: &self.lus,
            indice,
        }))
    }

    fn octets_actuels(&self, rang: u64) -> Result<u64, Refus> {
        let indice = l_entree(&self.entrees, rang)?;
        Ok(self.contenus[indice].len() as u64)
    }

    fn marque_actuelle(&self, rang: u64) -> Result<Option<u64>, Refus> {
        let indice = l_entree(&self.entrees, rang)?;
        Ok(Some(self.marques[indice]))
    }

    /// Un rang sans entrée n'a rien lu : 0.
    fn octets_lus(&self, rang: u64) -> u64 {
        l_entree(&self.entrees, rang).map_or(0, |indice| self.lus.borrow()[indice])
    }
}

/// Le flux d'une entrée en mémoire : ce qui reste de son contenu, et les octets lus qu'il compte.
struct FluxEnMemoire<'s> {
    reste: &'s [u8],
    lus: &'s RefCell<Vec<u64>>,
    indice: usize,
}

impl io::Read for FluxEnMemoire<'_> {
    fn read(&mut self, tampon: &mut [u8]) -> io::Result<usize> {
        let lus = io::Read::read(&mut self.reste, tampon)?;
        self.lus.borrow_mut()[self.indice] += lus as u64;
        Ok(lus)
    }
}

/// L'ordre des entrées : par leurs segments, comparés un à un, octet pour octet ; une entrée sans
/// chemin après celles qui en ont un.
fn par_segments(a: Option<&str>, b: Option<&str>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.split('/').cmp(b.split('/')),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
