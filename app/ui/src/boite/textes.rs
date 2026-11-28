//! Les textes de la boîte d'idées : ses libellés, les cases d'une ligne, ce que le volet dit
//! d'une idée, et les lignes du journal du fonds.

use commandes::{
    Contenu, EntreeDuJournal, LONGUEUR_D_UNE_LIGNE, LigneDeLaBoite, Ordre, Provenance, VueDUneIdee,
};
use noyau::donnees::boite::{date_de_la_liste, duree_affichee};
use noyau::donnees::journal::{empreinte_courte, heure_du_journal};

use crate::natif;

/// Ce qu'écrit une case vide (question 25).
pub(super) const CASE_VIDE: &str = "—";

/// Les choix de « Ranger par : », chacun avec son libellé ; la date d'abord, l'ordre par défaut
/// (question 21).
pub(super) const ORDRES: [(Ordre, &str); 5] = [
    (Ordre::Date, "date"),
    (Ordre::Etiquette, "étiquette"),
    (Ordre::Duree, "durée"),
    (Ordre::Reliee, "reliée"),
    (Ordre::Provenance, "provenance"),
];

/// Les choix de « Contenu : », chacun avec son libellé ; tout d'abord, sans critère de contenu
/// (question 22).
pub(crate) const CONTENUS: [(Option<Contenu>, &str); 6] = [
    (None, "tout"),
    (Some(Contenu::TexteDeLIdee), "texte de l'idée"),
    (Some(Contenu::Prise), "prise"),
    (Some(Contenu::Photo), "photo"),
    (Some(Contenu::MidiVerse), "MIDI versé"),
    (Some(Contenu::TexteVerse), "texte versé"),
];

/// « Annuler : <message> », ou « Rien à annuler » (question 30).
pub(super) fn libelle_d_annulation(a_annuler: Option<&str>) -> String {
    match a_annuler {
        Some(message) => format!("Annuler : {message}"),
        None => "Rien à annuler".to_string(),
    }
}

/// La page d'un fonds vide, quand aucune idée n'est interrogée (`considerees` vaut 0, Pupitre neuf
/// compris) : deux constats, jamais un ordre. Le premier dit la boîte d'idées vide, ce qui reste
/// vrai quand toutes les idées sont retirées : seule une idée retirée quitte la boîte, et le fonds
/// la garde. En lecture seule, le second renvoie au bandeau. Une recherche qui ne trouve rien
/// n'est pas un fonds vide : ses idées sont interrogées, et la liste vide dit son compte.
pub(super) fn page_du_fonds_vide(
    considerees: u64,
    lecture_seule: bool,
) -> Option<[&'static str; 2]> {
    (considerees == 0).then_some([
        "La boîte d'idées est vide.",
        if lecture_seule {
            "Le fonds est en lecture seule : le bandeau dit pourquoi."
        } else {
            "Une idée se note dans le champ « Noter une idée »."
        },
    ])
}

/// Un texte, ou `—` s'il n'y en a pas, ou s'il est vide.
pub(super) fn texte_ou_tiret(texte: Option<&str>) -> String {
    match texte {
        Some(texte) if !texte.is_empty() => texte.to_string(),
        _ => CASE_VIDE.to_string(),
    }
}

/// « reliée » ou « non reliée » : l'état se dit en mots, jamais par la seule couleur.
fn mot_de_l_etat(reliee: bool) -> &'static str {
    if reliee { "reliée" } else { "non reliée" }
}

/// « captée » ou « importée » (§ 30 · 8.1).
fn mot_de_la_provenance(provenance: &Provenance) -> &'static str {
    match provenance {
        Provenance::Captee { .. } => "captée",
        Provenance::Importee { .. } => "importée",
    }
}

/// Les cases d'une ligne : le texte, ou `—` ; les étiquettes, `#a #b`, ou `—` ; « reliée » ou
/// « non reliée » ; « captée » ou « importée » (question 25).
pub(super) fn texte_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    texte_ou_tiret(ligne.texte.as_deref())
}

pub(super) fn etiquettes_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    if ligne.etiquettes.is_empty() {
        return CASE_VIDE.to_string();
    }
    ligne
        .etiquettes
        .iter()
        .map(String::as_str)
        .map(etiquette_ecrite)
        .collect::<Vec<_>>()
        .join(" ")
}

/// L'étiquette `nom` écrite comme la liste l'écrit, et comme « Étiquette : » l'offre : `#nom`.
pub(super) fn etiquette_ecrite(nom: &str) -> String {
    format!("#{nom}")
}

/// L'étiquette que dit la valeur du choix fait dans « Étiquette : » : « Toutes », de valeur vide,
/// n'en pose aucune (`None`), puisqu'une étiquette vide ne trouverait rien ; une autre valeur est
/// le nom de l'étiquette.
pub(super) fn etiquette_du_choix(valeur: &str) -> Option<String> {
    (!valeur.is_empty()).then(|| valeur.to_string())
}

/// Les étiquettes que « Étiquette : » offre après « Toutes » : celles du fonds, `du_fonds`, et
/// l'étiquette `choisie` si le fonds ne la porte plus (une étiquette ôtée, une requête posée),
/// pour que le choix fait se voie ; chacune une fois, dans l'ordre de leurs octets.
pub(super) fn etiquettes_a_choisir(du_fonds: &[String], choisie: Option<&str>) -> Vec<String> {
    let mut etiquettes = du_fonds.to_vec();
    if let Some(choisie) = choisie
        && !etiquettes.iter().any(|etiquette| etiquette == choisie)
    {
        etiquettes.push(choisie.to_string());
        etiquettes.sort();
    }
    etiquettes
}

pub(super) fn etat_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str {
    mot_de_l_etat(ligne.reliee)
}

pub(super) fn provenance_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str {
    mot_de_la_provenance(&ligne.provenance)
}

/// Le nom d'une ligne, que dit son bouton : sa date `date`, son texte, ses étiquettes et sa durée,
/// chacune nommée, son état et sa provenance. Les titres des colonnes, cachés aux lecteurs d'écran,
/// ne disent pas quel tiret est l'étiquette et lequel la durée ; le nom le dit.
pub(super) fn nom_de_la_ligne(date: &str, ligne: &LigneDeLaBoite) -> String {
    format!(
        "{date} · {} · étiquettes {} · durée {} · {} · {}",
        texte_de_la_ligne(ligne),
        etiquettes_de_la_ligne(ligne),
        duree_affichee(ligne.duree_ms),
        etat_de_la_ligne(ligne),
        provenance_de_la_ligne(ligne)
    )
}

/// La date de la liste de l'instant `le`, au décalage local de cet instant (question 24).
pub(super) fn date_locale(le: i64) -> String {
    date_de_la_liste(le, natif::decalage_a(le))
}

/// Ce qui nomme une idée liée : sa date et son texte, coupé à `LONGUEUR_D_UNE_LIGNE` caractères,
/// puis « retirée » si elle l'est.
pub(super) fn nom_d_une_idee_liee(vue: &VueDUneIdee) -> String {
    let texte: String = texte_ou_tiret(vue.texte.as_deref())
        .chars()
        .take(LONGUEUR_D_UNE_LIGNE)
        .collect();
    let mut nom = format!("{} · {texte}", date_locale(vue.le));
    if vue.retiree.is_some() {
        nom.push_str(" · retirée");
    }
    nom
}

/// Ce que le volet dit de l'idée, en mots : reliée ou non, captée ou importée, sa durée, puis
/// « retirée » si elle l'est.
pub(super) fn etat_de_l_idee(vue: &VueDUneIdee) -> String {
    let mut mots = format!(
        "{} · {} · durée {}",
        mot_de_l_etat(vue.reliee),
        mot_de_la_provenance(&vue.provenance),
        duree_affichee(vue.duree_ms)
    );
    if vue.retiree.is_some() {
        mots.push_str(" · retirée");
    }
    mots
}

/// La clé d'une ligne dans sa liste : la ligne entière, en JSON. Une ligne inchangée garde ses
/// éléments d'une fenêtre à l'autre ; une ligne changée se redessine. Le diff des lignes réinsère
/// les lignes gardées quand la fenêtre remonte, et une ligne redessinée est neuve : le focus
/// qu'elles portaient se perd, et se rend (`rendre_le_focus_perdu`).
pub(super) fn cle_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    serde_json::to_string(ligne).unwrap_or_else(|_| ligne.idee.clone())
}

/// « 25 oct. 2026 · 02:30 · Idée du 25 octobre notée » : l'heure de l'entrée au décalage
/// `decalage` de son instant (`heure_du_journal`), « — » pour l'heure 0, puis son message.
pub(super) fn ligne_du_journal(entree: &EntreeDuJournal, decalage: i32) -> String {
    format!(
        "{} · {}",
        heure_du_journal(entree.heure, decalage),
        entree.message
    )
}

/// « 3 entrées · empreinte 570cda » : le nombre d'entrées du journal, et les six premiers
/// caractères de son empreinte (`empreinte_courte`) ; « 1 entrée » au singulier, « 0 entrée » de
/// même.
pub(super) fn entete_du_journal(total: u64, empreinte: &str) -> String {
    let mot = if total <= 1 { "entrée" } else { "entrées" };
    format!("{total} {mot} · empreinte {}", empreinte_courte(empreinte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boite::tests::ligne;
    use commandes::Appareil;

    #[test]
    fn le_fonds_vide_se_dit_par_des_constats() {
        // Dès qu'une idée est interrogée, il n'y a pas de page.
        assert_eq!(page_du_fonds_vide(1, false), None);
        assert_eq!(page_du_fonds_vide(12, true), None);
        assert_eq!(
            page_du_fonds_vide(0, false),
            Some([
                "La boîte d'idées est vide.",
                "Une idée se note dans le champ « Noter une idée »."
            ])
        );
        assert_eq!(
            page_du_fonds_vide(0, true),
            Some([
                "La boîte d'idées est vide.",
                "Le fonds est en lecture seule : le bandeau dit pourquoi."
            ])
        );
        for lecture_seule in [false, true] {
            for texte in page_du_fonds_vide(0, lecture_seule).unwrap() {
                // Un constat, jamais un ordre : aucun texte ne commence par un verbe à
                // l'impératif, rétablir compris.
                for verbe in [
                    "Notez",
                    "Importez",
                    "Créez",
                    "Ajoutez",
                    "Commencez",
                    "Rétablissez",
                ] {
                    assert!(!texte.starts_with(verbe), "{texte}");
                }
                // Un constat vrai : retirer n'efface rien, et toutes les idées retirées, le fonds
                // en porte encore. Aucun texte ne dit que le fonds n'a pas d'idée.
                assert!(!texte.contains("aucune idée"), "{texte}");
            }
        }
    }

    #[test]
    fn le_nom_d_une_ligne_nomme_chaque_case() {
        assert_eq!(
            nom_de_la_ligne("3 novembre 2026", &ligne(Some("Une idée de pont"), &[])),
            "3 novembre 2026 · Une idée de pont · étiquettes — · durée — · non reliée · captée"
        );
        let mut l = ligne(None, &["pont", "refrain"]);
        l.duree_ms = Some(83_000);
        l.reliee = true;
        assert_eq!(
            nom_de_la_ligne("3 novembre 2026", &l),
            "3 novembre 2026 · — · étiquettes #pont #refrain · durée 1:23 · reliée · captée"
        );
    }

    #[test]
    fn le_bouton_d_annulation_dit_le_geste_vise() {
        assert_eq!(
            libelle_d_annulation(Some("Idée du 3 novembre : étiquette #pont posée")),
            "Annuler : Idée du 3 novembre : étiquette #pont posée"
        );
        assert_eq!(libelle_d_annulation(None), "Rien à annuler");
    }

    #[test]
    fn un_texte_piege_se_montre_tel_quel() {
        let piege = "<img src=x onerror=alert(1)>";
        assert_eq!(texte_de_la_ligne(&ligne(Some(piege), &[])), piege);
    }

    #[test]
    fn une_case_vide_s_ecrit_d_un_tiret_cadratin() {
        let vide = ligne(None, &[]);
        assert_eq!(texte_de_la_ligne(&vide), "—");
        assert_eq!(etiquettes_de_la_ligne(&vide), "—");
        assert_eq!(texte_de_la_ligne(&ligne(Some(""), &[])), "—");
        assert_eq!(
            etiquettes_de_la_ligne(&ligne(None, &["pont", "refrain"])),
            "#pont #refrain"
        );
    }

    #[test]
    fn l_etat_et_la_provenance_se_disent_en_mots() {
        // Aucune information par la seule couleur : l'état et la provenance sont des mots.
        let mut l = ligne(Some("Une idée de pont"), &[]);
        assert_eq!(etat_de_la_ligne(&l), "non reliée");
        assert_eq!(provenance_de_la_ligne(&l), "captée");
        l.reliee = true;
        l.provenance = Provenance::Importee {
            par: Appareil::Etabli,
            depuis: "import-invente".to_string(),
        };
        assert_eq!(etat_de_la_ligne(&l), "reliée");
        assert_eq!(provenance_de_la_ligne(&l), "importée");
    }

    #[test]
    fn le_choix_toutes_ne_pose_aucune_etiquette() {
        // « Toutes », de valeur vide, ne pose aucun critère : une étiquette vide ne trouverait
        // rien.
        assert_eq!(etiquette_du_choix(""), None);
        assert_eq!(etiquette_du_choix("pont"), Some("pont".to_string()));
        // Une étiquette du fonds s'écrit comme la liste l'écrit.
        assert_eq!(etiquette_ecrite("pont"), "#pont");
        let du_fonds = ["pont".to_string(), "refrain".to_string()];
        assert_eq!(etiquettes_a_choisir(&du_fonds, None), du_fonds);
        assert_eq!(etiquettes_a_choisir(&du_fonds, Some("refrain")), du_fonds);
        // L'étiquette choisie que le fonds ne porte plus reste offerte, à son rang : le choix
        // se voit.
        assert_eq!(
            etiquettes_a_choisir(&du_fonds, Some("couplet")),
            ["couplet", "pont", "refrain"]
        );
    }

    #[test]
    fn chaque_ligne_du_journal_dit_son_heure_et_son_message() {
        // L'heure 0, celle de la genèse : « — ».
        let genese = EntreeDuJournal {
            message: "Genèse du fonds".to_string(),
            heure: 0,
        };
        assert_eq!(ligne_du_journal(&genese, 0), "— · Genèse du fonds");
        // Une heure au décalage de son instant : le jour, puis l'heure du jour sur deux chiffres.
        let notee = EntreeDuJournal {
            message: "Idée du 25 octobre notée".to_string(),
            heure: 1_792_888_200,
        };
        assert_eq!(
            ligne_du_journal(&notee, 120),
            "25 oct. 2026 · 02:30 · Idée du 25 octobre notée"
        );
    }

    #[test]
    fn l_entete_du_journal_dit_son_nombre_et_son_empreinte() {
        let empreinte = "570cda0f1e2d3c4b5a69788796a5b4c3d2e1f00112233445566778899aabbccd";
        assert_eq!(
            entete_du_journal(1, empreinte),
            "1 entrée · empreinte 570cda"
        );
        assert_eq!(
            entete_du_journal(3, empreinte),
            "3 entrées · empreinte 570cda"
        );
    }
}
