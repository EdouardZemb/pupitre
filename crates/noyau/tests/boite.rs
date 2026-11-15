//! La boîte d'idées au noyau : l'ordre, le filtre et le compte (CAP-02), sur un jeu inventé de
//! six idées.

use std::collections::BTreeSet;

use noyau::donnees::boite::{Contenu, EnBoite, Ordre, boite, contenus_de, retirees};
use noyau::donnees::gestes::{Appareil, IdeeLue, Provenance, Sorte};
use noyau::donnees::idee::Assiette;

/// Une idée `id`, née à `le`, avec ses étiquettes ; captée à l'établi, ou importée.
fn idee(id: &str, le: i64, etiquettes: &[&str], captee: bool) -> IdeeLue {
    IdeeLue {
        id: id.to_string(),
        provenance: if captee {
            Provenance::Captee {
                par: Appareil::Etabli,
            }
        } else {
            Provenance::Importee {
                par: Appareil::Etabli,
                depuis: "01IMPORT".into(),
            }
        },
        le,
        texte: None,
        etiquettes: etiquettes.iter().map(|e| e.to_string()).collect(),
        retiree: None,
        liens: BTreeSet::new(),
    }
}

/// Une idée `id`, avec ou sans texte.
fn idee_avec_texte(id: &str, texte: Option<&str>) -> IdeeLue {
    IdeeLue {
        texte: texte.map(str::to_string),
        ..idee(id, 100, &[], true)
    }
}

fn en_boite(idee: IdeeLue, contenus: &[Contenu], duree_ms: Option<u64>, reliee: bool) -> EnBoite {
    EnBoite {
        idee,
        contenus: contenus.iter().copied().collect(),
        duree_ms,
        reliee,
    }
}

fn retiree_a(mut e: EnBoite, quand: i64) -> EnBoite {
    e.idee.retiree = Some(quand);
    e
}

fn ids_de(idees: Vec<EnBoite>) -> Vec<String> {
    idees.into_iter().map(|e| e.idee.id).collect()
}

/// Les six idées du jeu : A à E, non retirées, et F, retirée à 500.
fn jeu() -> Vec<EnBoite> {
    use Contenu::*;
    vec![
        en_boite(
            idee("01A", 100, &["pont"], true),
            &[TexteDeLIdee, Prise],
            Some(47_000),
            false,
        ),
        en_boite(
            idee("01B", 300, &[], false),
            &[Prise, MidiVerse],
            None,
            true,
        ),
        en_boite(
            idee("01C", 200, &["Pont", "refrain"], false),
            &[Prise, TexteVerse],
            Some(12_000),
            false,
        ),
        en_boite(
            idee("01D", 300, &["élan"], true),
            &[TexteDeLIdee, Prise],
            Some(47_000),
            true,
        ),
        en_boite(
            idee("01E", 200, &["pont"], true),
            &[TexteDeLIdee],
            None,
            false,
        ),
        retiree_a(
            en_boite(idee("01F", 400, &["pont"], true), &[Prise], None, false),
            500,
        ),
    ]
}

/// F (retirée à 500), G (`01G`, retirée à 400), H (`01H`, retirée à 500) et E, non retirée.
fn jeu_des_retirees() -> Vec<EnBoite> {
    let simple = |id: &str| en_boite(idee(id, 100, &[], true), &[], None, false);
    vec![
        retiree_a(simple("01H"), 500),
        simple("01E"),
        retiree_a(simple("01G"), 400),
        retiree_a(simple("01F"), 500),
    ]
}

#[test]
fn chaque_ordre_est_total_et_se_departage_par_la_date_puis_l_identifiant() {
    let ids = |ordre| ids_de(boite(jeu(), 0, ordre, None).retenues);
    assert_eq!(ids(Ordre::Date), ["01B", "01D", "01C", "01E", "01A"]);
    // `Pont` (P, 0x50) passe avant `pont` ; `élan` (0xC3) après ; sans étiquette, en fin.
    assert_eq!(ids(Ordre::Etiquette), ["01C", "01E", "01A", "01D", "01B"]);
    assert_eq!(ids(Ordre::Duree), ["01D", "01A", "01C", "01B", "01E"]);
    assert_eq!(ids(Ordre::Reliee), ["01C", "01E", "01A", "01B", "01D"]);
    assert_eq!(ids(Ordre::Provenance), ["01D", "01E", "01A", "01B", "01C"]);
}

#[test]
fn le_filtre_retient_un_contenu_a_la_fois() {
    let ids = |filtre| ids_de(boite(jeu(), 0, Ordre::Date, Some(filtre)).retenues);
    // Une idée qui a une durée a une prise (`idee::duree`) ; B a une prise sans durée ; F, retirée, n'entre jamais.
    assert_eq!(ids(Contenu::Prise), ["01B", "01D", "01C", "01A"]);
    assert_eq!(ids(Contenu::Photo), Vec::<&str>::new()); // « 0 idée » (LE-6, règle 6)
    assert_eq!(ids(Contenu::MidiVerse), ["01B"]);
    assert_eq!(ids(Contenu::TexteVerse), ["01C"]);
    assert_eq!(ids(Contenu::TexteDeLIdee), ["01D", "01E", "01A"]);
}

#[test]
fn le_compte_des_non_reliees_ne_suit_pas_le_filtre() {
    for filtre in [None, Some(Contenu::Prise), Some(Contenu::Photo)] {
        let b = boite(jeu(), 0, Ordre::Date, filtre);
        assert_eq!(b.non_reliees, 3); // A, C, E ; F, retirée, n'y compte pas
        assert_eq!(
            b.assiette,
            Assiette {
                considerees: 5,
                portant: 5,
                decidees: 5
            }
        );
    }
}

#[test]
fn une_idee_illisible_compte_parmi_les_interrogees() {
    let b = boite(jeu(), 1, Ordre::Date, None);
    assert_eq!(b.non_reliees, 3);
    assert_eq!(
        b.assiette,
        Assiette {
            considerees: 6,
            portant: 5,
            decidees: 5
        }
    );
}

#[test]
fn les_retirees_se_rangent_par_la_date_du_retrait() {
    // F (retirée à 500), G (à 400), H (à 500, identifiant après F) : F, H, G ; E, non retirée, n'y entre pas.
    assert_eq!(ids_de(retirees(jeu_des_retirees())), ["01F", "01H", "01G"]);
}

#[test]
fn les_contenus_d_une_idee_viennent_de_son_texte_de_ses_sortes_et_de_la_photo() {
    let avec_texte = idee_avec_texte("01A", Some("Une idée de pont"));
    assert_eq!(
        contenus_de(&avec_texte, [Sorte::Prise, Sorte::MidiVerseSeul], false),
        BTreeSet::from([Contenu::TexteDeLIdee, Contenu::Prise, Contenu::MidiVerse])
    );
    let sans_texte = idee_avec_texte("01B", None);
    assert_eq!(
        contenus_de(&sans_texte, [Sorte::TexteVerse], true),
        BTreeSet::from([Contenu::Photo, Contenu::TexteVerse])
    );
}
