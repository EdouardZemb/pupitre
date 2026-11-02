//! Écrit le PDF de la fiche témoin d'H09 au chemin reçu en argument :
//! `cargo run -p cahier --example temoin -- fiche-temoin.pdf`.
//!
//! La fiche est datée du 1er nov. 2026 : deux engendrements donnent le même
//! fichier, à l'octet (I25).

use std::process::ExitCode;

use cahier::assembler;
use cahier::temoin::pages_temoin;
use noyau::rendu::pages::Date;

const DATE: Date = Date {
    annee: 2026,
    mois: 11,
    jour: 1,
};

fn main() -> ExitCode {
    let Some(chemin) = std::env::args_os().nth(1) else {
        eprintln!("usage : temoin <fichier.pdf>");
        return ExitCode::FAILURE;
    };
    let pages = match pages_temoin(&DATE) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("la fiche témoin ne tient pas : {e:?}");
            return ExitCode::FAILURE;
        }
    };
    let pdf = match assembler(&pages, &DATE) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("le PDF ne s'assemble pas : {e:?}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = std::fs::write(&chemin, pdf) {
        eprintln!("écriture impossible : {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
