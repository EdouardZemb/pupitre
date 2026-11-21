//! Le manifeste d'une prise en stems, le rapport d'import, ses comptes, sa relecture et la place
//! libre (§ 30 · 9 ; questions 2, 8, 19 et 25), sans entrée-sortie : l'établi parcourt les
//! fichiers et lit la place, le noyau en tire les faits.
//!
//! Le rapport n'est pas un message montré : il nomme l'identifiant de l'import et des empreintes,
//! que l'écran ne montre pas.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Le manifeste d'une prise en stems (question 19) : les empreintes distinctes de ses stems,
/// rangées dans l'ordre de leurs octets, chacune suivie d'un LF (`0x0A`), la dernière comprise.
pub fn manifeste(empreintes: &[&str]) -> Vec<u8> {
    let distinctes: BTreeSet<&str> = empreintes.iter().copied().collect();
    let mut octets = Vec::new();
    for empreinte in distinctes {
        octets.extend_from_slice(empreinte.as_bytes());
        octets.push(b'\n');
    }
    octets
}

/// Un compte du rapport : des fichiers, et leurs octets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Compte {
    pub fichiers: u64,
    pub octets: u64,
}

impl Compte {
    fn ajouter(&mut self, octets: u64) {
        self.fichiers = self.fichiers.saturating_add(1);
        self.octets = self.octets.saturating_add(octets);
    }
}

/// Les comptes d'un import (question 25) : vus = versés + dédoublonnés + ignorés ; illisibles et
/// sans nom sont des parts des versés.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Comptes {
    pub vus: Compte,
    pub verses: Compte,
    pub dedoublonnes: Compte,
    pub illisibles: Compte,
    pub ignores: Compte,
    pub sans_nom: Compte,
}

/// Un fichier vu par l'import, de rang `rang` (le premier fichier du parcours a le rang 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FichierDeLImport {
    /// Ignoré : ni lu au-delà de sa signature, ni versé ; un lien, une jonction compris.
    Ignore { rang: u64, octets: u64 },
    /// Lu en entier : son empreinte, s'il est illisible, s'il n'a pas de nom en UTF-8.
    Lu {
        rang: u64,
        octets: u64,
        empreinte: String,
        illisible: bool,
        sans_nom: bool,
    },
}

/// Le plus petit rang des fichiers lus de chaque contenu.
fn premiers_rangs(fichiers: &[FichierDeLImport]) -> BTreeMap<&str, u64> {
    let mut premiers: BTreeMap<&str, u64> = BTreeMap::new();
    for fichier in fichiers {
        if let FichierDeLImport::Lu {
            rang, empreinte, ..
        } = fichier
        {
            let entree = premiers.entry(empreinte.as_str()).or_insert(*rang);
            *entree = (*entree).min(*rang);
        }
    }
    premiers
}

/// Les comptes de `fichiers`, `connues` les empreintes des captations du fonds à la tête de
/// l'ouverture de l'import : un fichier lu est versé si son contenu n'était pas connu et qu'aucun
/// fichier lu de rang plus petit n'a le même contenu ; dédoublonné sinon. L'ordre de `fichiers`
/// ne compte pas : une reprise rend les comptes d'un import fait d'une traite (question 2).
pub fn comptes(fichiers: &[FichierDeLImport], connues: &BTreeSet<String>) -> Comptes {
    let premiers = premiers_rangs(fichiers);
    let mut c = Comptes::default();
    for fichier in fichiers {
        match fichier {
            FichierDeLImport::Ignore { octets, .. } => {
                c.vus.ajouter(*octets);
                c.ignores.ajouter(*octets);
            }
            FichierDeLImport::Lu {
                rang,
                octets,
                empreinte,
                illisible,
                sans_nom,
            } => {
                c.vus.ajouter(*octets);
                let est_le_premier = premiers.get(empreinte.as_str()) == Some(rang);
                if !connues.contains(empreinte) && est_le_premier {
                    c.verses.ajouter(*octets);
                    if *illisible {
                        c.illisibles.ajouter(*octets);
                    }
                    if *sans_nom {
                        c.sans_nom.ajouter(*octets);
                    }
                } else {
                    c.dedoublonnes.ajouter(*octets);
                }
            }
        }
    }
    c
}

/// Une origine qu'un import change : l'empreinte du contenu, et l'élément ajouté à son ensemble,
/// `main` ou `generee:<nom>`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrigineChangee {
    pub empreinte: String,
    pub element: String,
}

const ENTETE_DU_RAPPORT: &str = "pupitre-rapport-d-import 1";

/// Les six comptes, chacun sous sa clé, dans l'ordre du rapport.
fn cles_des_comptes(c: &Comptes) -> [(&'static str, Compte); 6] {
    [
        ("vus", c.vus),
        ("verses", c.verses),
        ("dedoublonnes", c.dedoublonnes),
        ("illisibles", c.illisibles),
        ("ignores", c.ignores),
        ("sans-nom", c.sans_nom),
    ]
}

/// Le rapport d'import, à l'octet (question 25) : en UTF-8, chaque ligne finie par LF,
/// `pupitre-rapport-d-import 1`, l'ULID de l'import, puis `vus=`, `verses=`, `dedoublonnes=`,
/// `illisibles=`, `ignores=`, `sans-nom=`, chacun suivi du nombre de fichiers, d'une espace et de
/// leurs octets ; puis `origine=<empreinte> <élément>` pour chaque origine changée, dans l'ordre
/// (empreinte, élément) ; puis `fichier-sans-nom=<rang>` pour chaque fichier lu sans nom, versé ou
/// dédoublonné, dans l'ordre des rangs.
pub fn rapport(
    import: &str,
    fichiers: &[FichierDeLImport],
    connues: &BTreeSet<String>,
    origines: &[OrigineChangee],
) -> Vec<u8> {
    let c = comptes(fichiers, connues);
    let mut texte = format!("{ENTETE_DU_RAPPORT}\n{import}\n");
    for (cle, compte) in cles_des_comptes(&c) {
        texte.push_str(&format!("{cle}={} {}\n", compte.fichiers, compte.octets));
    }
    let rangees: BTreeSet<&OrigineChangee> = origines.iter().collect();
    for origine in rangees {
        texte.push_str(&format!(
            "origine={} {}\n",
            origine.empreinte, origine.element
        ));
    }
    let sans_nom: BTreeSet<u64> = fichiers
        .iter()
        .filter_map(|f| match f {
            FichierDeLImport::Lu {
                rang,
                sans_nom: true,
                ..
            } => Some(*rang),
            _ => None,
        })
        .collect();
    for rang in sans_nom {
        texte.push_str(&format!("fichier-sans-nom={rang}\n"));
    }
    texte.into_bytes()
}

/// Un rapport d'import relu : son import, ses comptes, le nombre de ses lignes `origine=` et les
/// rangs de ses lignes `fichier-sans-nom=`, dans leur ordre.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RapportLu {
    pub import: String,
    pub comptes: Comptes,
    pub origines_changees: u64,
    pub fichiers_sans_nom: Vec<u64>,
}

/// Un entier écrit en chiffres décimaux seuls.
fn nombre(texte: &str) -> Option<u64> {
    if texte.is_empty() || !texte.bytes().all(|o| o.is_ascii_digit()) {
        return None;
    }
    texte.parse().ok()
}

/// `<cle>=<fichiers> <octets>`.
fn compte_de(ligne: &str, cle: &str) -> Option<Compte> {
    let (nom, valeur) = ligne.split_once('=')?;
    if nom != cle {
        return None;
    }
    let (fichiers, octets) = valeur.split_once(' ')?;
    Some(Compte {
        fichiers: nombre(fichiers)?,
        octets: nombre(octets)?,
    })
}

/// L'inverse de `rapport` (balayage P14) : `None` si la première ligne n'est pas
/// `pupitre-rapport-d-import 1` (une autre version), si une ligne manque ou si une ligne est mal
/// formée. Les services n'en relisent pas le format : ils appellent cette règle (tâche 13).
pub fn lire_le_rapport(octets: &[u8]) -> Option<RapportLu> {
    let texte = std::str::from_utf8(octets).ok()?;
    let corps = texte.strip_suffix('\n')?;
    let mut lignes = corps.split('\n');
    if lignes.next()? != ENTETE_DU_RAPPORT {
        return None;
    }
    let import = lignes.next()?;
    if import.is_empty() {
        return None;
    }
    let mut c = Comptes::default();
    let cles = cles_des_comptes(&c).map(|(cle, _)| cle);
    let mut lus = [Compte::default(); 6];
    for (lu, cle) in lus.iter_mut().zip(cles) {
        *lu = compte_de(lignes.next()?, cle)?;
    }
    let [vus, verses, dedoublonnes, illisibles, ignores, sans_nom] = lus;
    c.vus = vus;
    c.verses = verses;
    c.dedoublonnes = dedoublonnes;
    c.illisibles = illisibles;
    c.ignores = ignores;
    c.sans_nom = sans_nom;
    let mut origines_changees: u64 = 0;
    let mut fichiers_sans_nom = Vec::new();
    for ligne in lignes {
        if let Some(origine) = ligne.strip_prefix("origine=") {
            if !fichiers_sans_nom.is_empty() {
                return None;
            }
            let (empreinte, element) = origine.split_once(' ')?;
            if empreinte.is_empty() || element.is_empty() {
                return None;
            }
            origines_changees = origines_changees.checked_add(1)?;
        } else {
            let rang = ligne.strip_prefix("fichier-sans-nom=")?;
            fichiers_sans_nom.push(nombre(rang)?);
        }
    }
    Some(RapportLu {
        import: import.to_string(),
        comptes: c,
        origines_changees,
        fichiers_sans_nom,
    })
}

/// Les octets laissés pour chaque fichier à verser, en plus de ses octets (question 8).
pub const MARGE_PAR_FICHIER: u64 = 4_096;

/// La place qui manque : la taille à verser, la place qu'il faut (le besoin), la place libre, et
/// ce qui manque.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManqueDePlace {
    pub a_verser: u64,
    pub besoin: u64,
    pub libre: u64,
    pub manque: u64,
}

/// On verse si le besoin, `a_verser` + 2 × `poids_du_fonds` + `MARGE_PAR_FICHIER` × `fichiers`,
/// tient dans `libre`, calculé en `u128` ; sinon le besoin et le manque, chacun borné à
/// `u64::MAX`.
pub fn place(
    a_verser: u64,
    fichiers: u64,
    poids_du_fonds: u64,
    libre: u64,
) -> Result<(), ManqueDePlace> {
    let besoin = u128::from(a_verser)
        + 2 * u128::from(poids_du_fonds)
        + u128::from(MARGE_PAR_FICHIER) * u128::from(fichiers);
    match besoin.checked_sub(u128::from(libre)) {
        None | Some(0) => Ok(()),
        Some(manque) => Err(ManqueDePlace {
            a_verser,
            besoin: u64::try_from(besoin).unwrap_or(u64::MAX),
            libre,
            manque: u64::try_from(manque).unwrap_or(u64::MAX),
        }),
    }
}

/// `octets` en Go décimaux (10⁹), à une décimale, la virgule en séparateur : « 4,0 Go » ; arrondi
/// au dixième supérieur si `vers_le_haut`, inférieur sinon.
pub fn go(octets: u64, vers_le_haut: bool) -> String {
    const DIXIEME: u128 = 100_000_000;
    let octets = u128::from(octets);
    let dixiemes = if vers_le_haut {
        octets.div_ceil(DIXIEME)
    } else {
        octets / DIXIEME
    };
    format!("{},{} Go", dixiemes / 10, dixiemes % 10)
}

/// « Verser 4,0 Go demande 4,1 Go de place libre, deux fois le poids du fonds compris ; il y en a
/// 1,0 Go : il manque 3,1 Go. » : la taille à verser, le besoin et le manque arrondis au dixième
/// supérieur, la place libre au dixième inférieur.
impl fmt::Display for ManqueDePlace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Verser {} demande {} de place libre, deux fois le poids du fonds compris ; il y en a \
             {} : il manque {}.",
            go(self.a_verser, true),
            go(self.besoin, true),
            go(self.libre, false),
            go(self.manque, true)
        )
    }
}
