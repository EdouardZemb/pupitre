//! La commande du cahier des tables : `cahier --date AAAA-MM-JJ
//! [--instrument guitare|clavier|les-deux] [--couche masquee|affichee] --sortie <fichier>`.
//!
//! La date du rendu est une entrée, jamais l'horloge (I25). L'instrument vaut `les-deux` par
//! défaut, et la couche sombre, `masquee` (REN-01). Une erreur d'usage sort en 2 ; une page qui
//! ne tient pas, un PDF qui ne s'assemble pas ou un fichier qui ne s'écrit pas, en 1.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use cahier::engendrer;
use noyau::rendu::Instrument;
use noyau::rendu::pages::Date;
use noyau::table::CoucheSombre;

const USAGE: &str = concat!(
    "usage : cahier --date AAAA-MM-JJ [--instrument guitare|clavier|les-deux] ",
    "[--couche masquee|affichee] --sortie <fichier>"
);

/// Ce que la commande a reçu.
#[derive(Debug, PartialEq)]
struct Commande {
    date: Date,
    instrument: Instrument,
    couche: CoucheSombre,
    sortie: PathBuf,
}

/// Lit les arguments, sans le nom du programme. Chaque option se donne une fois, suivie de sa
/// valeur ; une option inconnue, une valeur absente ou fausse sont des erreurs d'usage.
fn lire(args: impl IntoIterator<Item = OsString>) -> Result<Commande, String> {
    let mut date = None;
    let mut instrument = None;
    let mut couche = None;
    let mut sortie = None;
    let mut args = args.into_iter();
    while let Some(option) = args.next() {
        let option = option.to_string_lossy().into_owned();
        let deja = match option.as_str() {
            "--date" => date.is_some(),
            "--instrument" => instrument.is_some(),
            "--couche" => couche.is_some(),
            "--sortie" => sortie.is_some(),
            _ => return Err(format!("option inconnue : « {option} »")),
        };
        if deja {
            return Err(format!("{option} est donnée deux fois"));
        }
        let valeur = args
            .next()
            .ok_or_else(|| format!("{option} attend une valeur"))?;
        match option.as_str() {
            "--date" => {
                let texte = valeur.to_string_lossy();
                date = Some(Date::depuis_iso(&texte).ok_or_else(|| {
                    format!("date invalide : « {texte} », attendue sous la forme AAAA-MM-JJ")
                })?);
            }
            "--instrument" => {
                instrument = Some(match valeur.to_string_lossy().as_ref() {
                    "guitare" => Instrument::Guitare,
                    "clavier" => Instrument::Clavier,
                    "les-deux" => Instrument::LesDeux,
                    autre => {
                        return Err(format!(
                            "instrument inconnu : « {autre} », attendu guitare, clavier ou les-deux"
                        ));
                    }
                });
            }
            "--couche" => {
                couche = Some(match valeur.to_string_lossy().as_ref() {
                    "masquee" => CoucheSombre::Masquee,
                    "affichee" => CoucheSombre::Affichee,
                    autre => {
                        return Err(format!(
                            "couche inconnue : « {autre} », attendue masquee ou affichee"
                        ));
                    }
                });
            }
            _ => sortie = Some(PathBuf::from(valeur)),
        }
    }
    Ok(Commande {
        date: date.ok_or("--date manque")?,
        instrument: instrument.unwrap_or_default(),
        couche: couche.unwrap_or_default(),
        sortie: sortie.ok_or("--sortie manque")?,
    })
}

fn main() -> ExitCode {
    let commande = match lire(std::env::args_os().skip(1)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cahier : {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let pdf = match engendrer(commande.instrument, commande.couche, &commande.date) {
        Ok(pdf) => pdf,
        Err(e) => {
            eprintln!("cahier : {e}");
            return ExitCode::from(1);
        }
    };
    if let Err(e) = std::fs::write(&commande.sortie, pdf) {
        eprintln!(
            "cahier : {} ne s'écrit pas : {e}",
            commande.sortie.display()
        );
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lire_(args: &[&str]) -> Result<Commande, String> {
        lire(args.iter().map(OsString::from))
    }

    const DATE: Date = Date {
        annee: 2026,
        mois: 11,
        jour: 1,
    };

    #[test]
    fn l_instrument_vaut_les_deux_par_defaut_et_l_ordre_est_libre() {
        assert_eq!(
            lire_(&["--sortie", "cahier.pdf", "--date", "2026-11-01"]),
            Ok(Commande {
                date: DATE,
                instrument: Instrument::LesDeux,
                couche: CoucheSombre::Masquee,
                sortie: PathBuf::from("cahier.pdf"),
            })
        );
        for (nom, i) in [
            ("guitare", Instrument::Guitare),
            ("clavier", Instrument::Clavier),
            ("les-deux", Instrument::LesDeux),
        ] {
            let c = lire_(&[
                "--date",
                "2026-11-01",
                "--instrument",
                nom,
                "--sortie",
                "c.pdf",
            ]);
            assert_eq!(c.map(|c| c.instrument), Ok(i), "{nom}");
        }
    }

    #[test]
    fn la_couche_sombre_vaut_masquee_par_defaut() {
        // REN-01 : sans `--couche`, la couche sombre est masquée.
        let c = lire_(&["--date", "2026-11-01", "--sortie", "c.pdf"]);
        assert_eq!(c.map(|c| c.couche), Ok(CoucheSombre::Masquee));
        for (nom, couche) in [
            ("masquee", CoucheSombre::Masquee),
            ("affichee", CoucheSombre::Affichee),
        ] {
            let c = lire_(&["--couche", nom, "--date", "2026-11-01", "--sortie", "c.pdf"]);
            assert_eq!(c.map(|c| c.couche), Ok(couche), "{nom}");
        }
    }

    #[test]
    fn un_usage_faux_se_dit() {
        let cas: [(&[&str], &str); 10] = [
            (&[], "--date manque"),
            (&["--date", "2026-11-01"], "--sortie manque"),
            (&["--date"], "--date attend une valeur"),
            (
                &["--date", "1er nov.", "--sortie", "c.pdf"],
                "date invalide : « 1er nov. », attendue sous la forme AAAA-MM-JJ",
            ),
            (
                &["--date", "2026-11-01", "--date", "2026-11-02"],
                "--date est donnée deux fois",
            ),
            (
                &["--instrument", "violon"],
                "instrument inconnu : « violon », attendu guitare, clavier ou les-deux",
            ),
            (
                &["--couche", "grise"],
                "couche inconnue : « grise », attendue masquee ou affichee",
            ),
            // Les valeurs s'écrivent sans accent, comme les noms des options.
            (
                &["--couche", "affichée"],
                "couche inconnue : « affichée », attendue masquee ou affichee",
            ),
            (
                &["--couche", "masquee", "--couche", "affichee"],
                "--couche est donnée deux fois",
            ),
            (&["--aide"], "option inconnue : « --aide »"),
        ];
        for (args, message) in cas {
            assert_eq!(lire_(args), Err(message.to_string()), "{args:?}");
        }
    }
}
