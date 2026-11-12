//! La teneur d'un document (§ 31 · 2.6) : le texte déterministe de son arbre, qui sert à
//! comparer deux documents (§ 31 · 10). Le noyau ne voit que la valeur que `stockage` tire du
//! document ; il l'écrit sans l'altérer, octet pour octet, les clés dans l'ordre de leurs octets.

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::donnees::fonds::{LIGNEE, LIGNEES_REMPLACEES};

/// Une valeur que présente un document, que `stockage` tire du document.
#[derive(Clone, Debug, PartialEq)]
pub enum Valeur {
    /// `Map` ou `Table` : une seule sorte.
    ObjetACles(BTreeMap<String, Valeur>),
    /// Une liste.
    Liste(Vec<Valeur>),
    /// Un objet `Text`.
    Texte(String),
    /// Une chaîne.
    Str(String),
    /// Un entier signé.
    Int(i64),
    /// Un entier non signé.
    Uint(u64),
    /// Un flottant.
    F64(f64),
    /// Un compteur.
    Counter(i64),
    /// Un instant.
    Timestamp(i64),
    /// Un booléen.
    Boolean(bool),
    /// Des octets.
    Bytes(Vec<u8>),
    /// Rien.
    Null,
    /// `ScalarValue::Unknown`, de ce code de type.
    Inconnu {
        /// Le code de type que le document porte.
        type_code: u8,
    },
}

/// Ce que la teneur refuse d'écrire, avec le chemin de la valeur : chaque clé, et l'indice
/// décimal dans une liste.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErreurDeTeneur {
    /// Un flottant qui n'est pas un nombre.
    NaN {
        /// Le chemin de la valeur.
        chemin: Vec<String>,
    },
    /// Une valeur d'un type que cette version ne connaît pas.
    Inconnu {
        /// Le chemin de la valeur.
        chemin: Vec<String>,
        /// Son code de type.
        type_code: u8,
    },
}

/// Le texte déterministe de la teneur de la racine du document, sans `LIGNEE` ni
/// `LIGNEES_REMPLACEES`.
pub fn ecrire_la_teneur(racine: &BTreeMap<String, Valeur>) -> Result<String, ErreurDeTeneur> {
    let mut sortie = String::new();
    let mut chemin = Vec::new();
    ecrire_un_objet(&mut sortie, racine, &mut chemin, true)?;
    Ok(sortie)
}

fn ecrire_une_chaine(sortie: &mut String, texte: &str) {
    sortie.push('"');
    for c in texte.chars() {
        if c == '\\' || c == '"' {
            sortie.push('\\');
        }
        sortie.push(c);
    }
    sortie.push('"');
}

fn ecrire_un_objet(
    sortie: &mut String,
    objet: &BTreeMap<String, Valeur>,
    chemin: &mut Vec<String>,
    racine: bool,
) -> Result<(), ErreurDeTeneur> {
    sortie.push('{');
    let mut premiere = true;
    for (cle, valeur) in objet {
        if racine && (cle == LIGNEE || cle == LIGNEES_REMPLACEES) {
            continue;
        }
        if !premiere {
            sortie.push(',');
        }
        premiere = false;
        ecrire_une_chaine(sortie, cle);
        sortie.push(':');
        chemin.push(cle.clone());
        ecrire_une_valeur(sortie, valeur, chemin)?;
        chemin.pop();
    }
    sortie.push('}');
    Ok(())
}

/// Le flottant sous sa forme la plus courte : `{}` ou `{:e}`, le premier à égalité.
fn ecrire_un_flottant(sortie: &mut String, x: f64) {
    let plein = format!("{x}");
    let scientifique = format!("{x:e}");
    sortie.push_str(if scientifique.len() < plein.len() {
        &scientifique
    } else {
        &plein
    });
}

fn ecrire_une_valeur(
    sortie: &mut String,
    valeur: &Valeur,
    chemin: &mut Vec<String>,
) -> Result<(), ErreurDeTeneur> {
    match valeur {
        Valeur::ObjetACles(objet) => ecrire_un_objet(sortie, objet, chemin, false)?,
        Valeur::Liste(elements) => {
            sortie.push('[');
            for (indice, element) in elements.iter().enumerate() {
                if indice > 0 {
                    sortie.push(',');
                }
                chemin.push(indice.to_string());
                ecrire_une_valeur(sortie, element, chemin)?;
                chemin.pop();
            }
            sortie.push(']');
        }
        Valeur::Texte(texte) => {
            sortie.push('t');
            ecrire_une_chaine(sortie, texte);
        }
        Valeur::Str(texte) => {
            sortie.push('s');
            ecrire_une_chaine(sortie, texte);
        }
        Valeur::Int(n) => {
            let _ = write!(sortie, "i{n}");
        }
        Valeur::Uint(n) => {
            let _ = write!(sortie, "u{n}");
        }
        Valeur::Counter(n) => {
            let _ = write!(sortie, "c{n}");
        }
        Valeur::Timestamp(n) => {
            let _ = write!(sortie, "d{n}");
        }
        Valeur::F64(x) => {
            if x.is_nan() {
                return Err(ErreurDeTeneur::NaN {
                    chemin: chemin.clone(),
                });
            }
            sortie.push('f');
            ecrire_un_flottant(sortie, *x);
        }
        Valeur::Boolean(b) => sortie.push_str(if *b { "b1" } else { "b0" }),
        Valeur::Bytes(octets) => {
            sortie.push('x');
            for octet in octets {
                let _ = write!(sortie, "{octet:02x}");
            }
        }
        Valeur::Null => sortie.push('n'),
        Valeur::Inconnu { type_code } => {
            return Err(ErreurDeTeneur::Inconnu {
                chemin: chemin.clone(),
                type_code: *type_code,
            });
        }
    }
    Ok(())
}
