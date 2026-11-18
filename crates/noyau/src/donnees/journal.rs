//! Le journal du fonds, tel que sa vue le montre (question 29) : ses entrées, chacune le message
//! d'une modification et son heure, rangées la plus récente d'abord ; l'heure de chacune, dite au
//! décalage local de son instant ; et l'empreinte qui dit « le même journal » sur deux répliques,
//! quel que soit l'ordre de leur historique. Les entrées viennent de `services::lecture::journal`.

use std::cmp::Reverse;
use std::fmt::Write as _;

use crate::donnees::boite::date_de_la_liste;
use crate::donnees::gestes::heure_locale;
use crate::donnees::identifiants::empreinte;

/// Ce que montre l'heure 0, celle de la genèse et des migrations (§ 31 · 2.4).
const SANS_HEURE: &str = "—";

/// Le nombre de caractères de l'empreinte que la vue montre.
const LONGUEUR_DE_L_EMPREINTE_COURTE: usize = 6;

/// Les entrées (message, heure), reçues dans l'ordre de l'historique, rangées la plus récente
/// d'abord ; à égalité d'heure, la plus tardive dans l'historique d'abord (question 29).
pub fn ranger_le_journal(entrees: Vec<(String, i64)>) -> Vec<(String, i64)> {
    let mut rangees = entrees;
    // L'historique à rebours, puis un tri stable par heure décroissante : à égalité d'heure, le
    // rebours reste.
    rangees.reverse();
    rangees.sort_by_key(|(_, heure)| Reverse(*heure));
    rangees
}

/// « 25 oct. 2026 · 02:30 » : la date de la liste et l'heure, sur deux chiffres, au décalage
/// `decalage_minutes` ; « — » pour l'heure 0, celle de la genèse et des migrations (§ 31 · 2.4).
/// L'heure du jour se tire de l'heure locale (`gestes::heure_locale`), celle d'où `date_civile`
/// tire le jour.
pub fn heure_du_journal(heure: i64, decalage_minutes: i32) -> String {
    if heure == 0 {
        return SANS_HEURE.to_string();
    }
    let secondes_du_jour = heure_locale(heure, decalage_minutes).rem_euclid(86_400);
    format!(
        "{} · {:02}:{:02}",
        date_de_la_liste(heure, decalage_minutes),
        secondes_du_jour / 3_600,
        secondes_du_jour % 3_600 / 60
    )
}

/// « Le même journal » (question 29) : les entrées rangées par heure, puis par message, octet pour
/// octet, chacune écrite `<heure>`, une tabulation, `<message>`, un saut de ligne ; le SHA-256 de ce
/// texte (`identifiants::empreinte`). L'ordre de l'historique n'y compte pas. Sa limite : le texte
/// n'échappe rien, et un message qui porte un retour à la ligne suivi de chiffres et d'une
/// tabulation peut faire coïncider deux journaux différents. Aucun champ de l'interface ne laisse
/// passer un retour à la ligne.
pub fn empreinte_du_journal(entrees: &[(String, i64)]) -> String {
    let mut rangees: Vec<(i64, &str)> = entrees
        .iter()
        .map(|(message, heure)| (*heure, message.as_str()))
        .collect();
    rangees.sort_unstable();
    let mut texte = String::new();
    for (heure, message) in rangees {
        let _ = writeln!(texte, "{heure}\t{message}");
    }
    empreinte(texte.as_bytes())
}

/// Les six premiers caractères d'une empreinte, que la vue montre ; une empreinte plus courte,
/// entière.
pub fn empreinte_courte(empreinte: &str) -> &str {
    empreinte
        .get(..LONGUEUR_DE_L_EMPREINTE_COURTE)
        .unwrap_or(empreinte)
}
