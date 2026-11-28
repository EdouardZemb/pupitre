//! Les gestes au clavier, les commandes qu'envoient noter, relier et délier, et l'argument
//! d'une commande.

use commandes::{Noter, Relier};
use leptos::web_sys::{HtmlInputElement, HtmlTextAreaElement};
use serde::Serialize;
use wasm_bindgen::JsCast;

/// `Ctrl+Z` annule un geste de tri hors d'un champ de texte seulement ; `Ctrl+Maj+Z`, qui donne
/// « Z », aussi.
pub(super) fn ctrl_z_annule(touche: &str, ctrl: bool, dans_un_champ: bool) -> bool {
    ctrl && !dans_un_champ && touche.eq_ignore_ascii_case("z")
}

/// Noter en un geste (CAP-01 · 4) : `Entrée` dans le champ, un texte non vide une fois ses blancs
/// ôtés. La note part sans ses blancs autour, avec le décalage local du geste ; aucun titre n'est
/// demandé.
pub(super) fn note_a_envoyer(champ: &str, touche: &str, decalage_minutes: i32) -> Option<Noter> {
    let texte = champ.trim();
    (touche == "Enter" && !texte.is_empty()).then(|| Noter {
        texte: texte.to_string(),
        decalage_minutes,
    })
}

/// La commande `Relier` des idées `a` et `b`, chacune avec l'instant de sa naissance, en
/// secondes : chacune datée, dans le message du journal, au décalage local de cet instant, que
/// rend `decalage` (`natif::decalage_a`), jamais à celui du geste. Relier et délier la partagent.
fn commande_de_lien(a: (String, i64), b: (String, i64), decalage: impl Fn(i64) -> i32) -> Relier {
    let (decalage_minutes, decalage_b_minutes) = (decalage(a.1), decalage(b.1));
    Relier {
        a: a.0,
        b: b.0,
        decalage_minutes,
        decalage_b_minutes,
    }
}

/// La commande `Relier` des idées cochées, si elles sont deux, chacune avec sa naissance
/// (`commande_de_lien`) ; une seule, ou trois, aucune.
pub(super) fn relier_les_cochees(
    cochees: &[(String, i64)],
    decalage: impl Fn(i64) -> i32,
) -> Option<Relier> {
    match cochees {
        [a, b] => Some(commande_de_lien(a.clone(), b.clone(), decalage)),
        _ => None,
    }
}

/// La commande qui délie l'idée du volet `idee` de l'autre bout `autre` de l'un de ses liens,
/// chacun avec la naissance que le volet en a lue : `commande_de_lien`, par `decalage`. Un bout
/// dont la naissance ne s'est pas lue (`None`) n'a pas d'instant où lire son décalage : il prend
/// `du_geste`, le décalage du geste, et son jour, dans le message, peut s'écarter d'un de celui de
/// sa naissance. C'est la limite d'un bout illisible.
pub(super) fn deliaison(
    idee: (String, Option<i64>),
    autre: (String, Option<i64>),
    decalage: impl Fn(i64) -> i32,
    du_geste: i32,
) -> Relier {
    let ((a, le_a), (b, le_b)) = (idee, autre);
    let mut commande = commande_de_lien(
        (a, le_a.unwrap_or_default()),
        (b, le_b.unwrap_or_default()),
        decalage,
    );
    if le_a.is_none() {
        commande.decalage_minutes = du_geste;
    }
    if le_b.is_none() {
        commande.decalage_b_minutes = du_geste;
    }
    commande
}

/// Une frappe d'`Entrée` envoie le champ, la première seulement : tenue, la touche se répète sans
/// rien renvoyer, comme `Ctrl+Z` (R13) ; pendant une composition, elle la valide et n'envoie rien.
pub(super) fn envoi_au_clavier(touche: &str, tenue: bool, en_composition: bool) -> bool {
    touche == "Enter" && !tenue && !en_composition
}

/// Vrai si la touche part d'un champ de texte, reconnu à la cible de l'événement : un
/// `<textarea>`, ou un `<input>` qui n'est ni une case à cocher, ni un bouton radio, ni un bouton.
pub(crate) fn dans_un_champ_de_texte(ev: &leptos::ev::KeyboardEvent) -> bool {
    let Some(cible) = ev.target() else {
        return false;
    };
    if cible.dyn_ref::<HtmlTextAreaElement>().is_some() {
        return true;
    }
    cible.dyn_ref::<HtmlInputElement>().is_some_and(|champ| {
        !matches!(
            champ.type_().as_str(),
            "checkbox"
                | "radio"
                | "button"
                | "submit"
                | "reset"
                | "image"
                | "file"
                | "range"
                | "color"
        )
    })
}

/// L'argument d'une commande, en JSON ; rien s'il ne s'y écrit pas, et l'établi refuse alors la
/// commande, ce que le bandeau dit.
pub(super) fn argument<T: Serialize>(commande: &T) -> Option<serde_json::Value> {
    serde_json::to_value(commande).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boite::tests::decalage_de_paris;

    #[test]
    fn une_touche_tenue_n_envoie_rien() {
        // La première frappe d'`Entrée` envoie le champ ; tenue, la touche se répète sans rien
        // renvoyer, comme `Ctrl+Z`.
        assert!(envoi_au_clavier("Enter", false, false));
        assert!(!envoi_au_clavier("Enter", true, false));
        // Pendant une composition (un accent en cours), `Entrée` la valide et n'envoie rien.
        assert!(!envoi_au_clavier("Enter", false, true));
        assert!(!envoi_au_clavier("a", false, false));
        assert!(!envoi_au_clavier("Tab", false, false));
    }

    #[test]
    fn ctrl_z_n_annule_un_geste_que_hors_d_un_champ() {
        assert!(ctrl_z_annule("z", true, false));
        // `Ctrl+Maj+Z` donne « Z ».
        assert!(ctrl_z_annule("Z", true, false));
        assert!(!ctrl_z_annule("z", true, true));
        assert!(!ctrl_z_annule("z", false, false));
        assert!(!ctrl_z_annule("y", true, false));
    }

    #[test]
    fn cap01_4_noter_se_fait_en_un_geste_sans_titre() {
        // Une part de CAP-01 · 4 : à l'établi, `Entrée` dans le champ note l'idée, sans demande de
        // titre ; la capture du carnet vient au lot 2.
        assert_eq!(
            note_a_envoyer("Une idée de pont", "Enter", 60),
            Some(Noter {
                texte: "Une idée de pont".to_string(),
                decalage_minutes: 60
            })
        );
        assert_eq!(note_a_envoyer("Une idée de pont", "a", 60), None);
        assert_eq!(note_a_envoyer("Une idée de pont", "Tab", 60), None);
        assert_eq!(note_a_envoyer(" \t  ", "Enter", 60), None);
        assert_eq!(note_a_envoyer("", "Enter", 60), None);
        // La note part sans ses blancs autour, avec le décalage du geste.
        assert_eq!(
            note_a_envoyer("  Une idée de pont ", "Enter", -300),
            Some(Noter {
                texte: "Une idée de pont".to_string(),
                decalage_minutes: -300
            })
        );
    }

    #[test]
    fn relier_les_cochees_date_chaque_idee_a_sa_naissance() {
        // L'une née le 24 octobre 2026 à 22 h 30 UTC, à l'heure d'été ; l'autre le 17 novembre
        // à 9 h UTC, à l'heure d'hiver.
        let a = ("idee-a".to_string(), 1_792_881_000);
        let b = ("idee-b".to_string(), 1_794_906_000);
        assert_eq!(
            relier_les_cochees(&[a.clone(), b.clone()], decalage_de_paris),
            Some(Relier {
                a: "idee-a".to_string(),
                b: "idee-b".to_string(),
                decalage_minutes: 120,
                decalage_b_minutes: 60,
            })
        );
        assert_eq!(
            relier_les_cochees(std::slice::from_ref(&a), decalage_de_paris),
            None
        );
        assert_eq!(
            relier_les_cochees(
                &[a, b, ("idee-c".to_string(), 1_793_750_400)],
                decalage_de_paris
            ),
            None
        );
    }

    #[test]
    fn delier_depuis_le_volet_date_chaque_bout() {
        // L'idée du volet née le 24 octobre 2026 à 22 h 30 UTC, l'autre bout le 17 novembre à
        // 9 h UTC : chacun au décalage de son instant, jamais à celui du geste.
        assert_eq!(
            deliaison(
                ("idee-du-volet".to_string(), Some(1_792_881_000)),
                ("autre-bout".to_string(), Some(1_794_906_000)),
                decalage_de_paris,
                -300,
            ),
            Relier {
                a: "idee-du-volet".to_string(),
                b: "autre-bout".to_string(),
                decalage_minutes: 120,
                decalage_b_minutes: 60,
            }
        );
        // Un bout dont la naissance ne s'est pas lue prend le décalage du geste : la limite.
        let sans_naissance = deliaison(
            ("idee-du-volet".to_string(), Some(1_792_881_000)),
            ("autre-bout".to_string(), None),
            decalage_de_paris,
            -300,
        );
        assert_eq!(
            (
                sans_naissance.decalage_minutes,
                sans_naissance.decalage_b_minutes
            ),
            (120, -300)
        );
        let volet_sans_naissance = deliaison(
            ("idee-du-volet".to_string(), None),
            ("autre-bout".to_string(), Some(1_794_906_000)),
            decalage_de_paris,
            -300,
        );
        assert_eq!(
            (
                volet_sans_naissance.decalage_minutes,
                volet_sans_naissance.decalage_b_minutes
            ),
            (-300, 60)
        );
    }
}
