//! Le journal au noyau (question 29) : ses entrées rangées la plus récente d'abord, l'heure de
//! chacune dite au décalage local de son instant, la genèse par un tiret, et l'empreinte qui dit
//! « le même journal », sur des entrées inventées.

use noyau::donnees::journal::{
    empreinte_courte, empreinte_du_journal, heure_du_journal, ranger_le_journal,
};

#[test]
fn l_heure_du_journal_se_dit_a_l_instant_et_la_genese_par_un_tiret() {
    assert_eq!(heure_du_journal(0, 60), "—");
    assert_eq!(heure_du_journal(1_792_888_200, 120), "25 oct. 2026 · 02:30"); // 0 h 30 UTC, heure d'été
    assert_eq!(heure_du_journal(1_792_891_800, 60), "25 oct. 2026 · 02:30"); // 1 h 30 UTC, heure d'hiver
    assert_eq!(
        heure_du_journal(1_798_759_800, 60),
        "1er janv. 2027 · 00:30"
    );
    assert_eq!(heure_du_journal(1_792_886_460, 0), "25 oct. 2026 · 00:01");
}

#[test]
fn le_journal_se_range_le_plus_recent_d_abord() {
    let recu = vec![
        ("Genèse du fonds".into(), 0),
        ("A".into(), 10),
        ("B".into(), 10),
        ("C".into(), 5),
    ];
    assert_eq!(
        ranger_le_journal(recu),
        [
            ("B".to_string(), 10),
            ("A".to_string(), 10),
            ("C".to_string(), 5),
            ("Genèse du fonds".to_string(), 0)
        ]
    );
}

#[test]
fn le_meme_journal_a_la_meme_empreinte() {
    let journal = [
        ("Genèse du fonds".to_string(), 0),
        ("Fonds créé".to_string(), 1_792_888_200),
        ("Idée du 25 octobre notée".to_string(), 1_792_888_220),
    ];
    let e = empreinte_du_journal(&journal);
    assert_eq!(
        e,
        "570cda108d563a266092b9cc5b4e884c691ca3ee7b228c032d594e8219b54dce"
    );
    assert_eq!(empreinte_courte(&e), "570cda");
    let mut a_rebours = journal.clone();
    a_rebours.reverse();
    assert_eq!(empreinte_du_journal(&a_rebours), e);
    let mut autre = journal.clone();
    autre[2].1 += 1;
    assert_ne!(empreinte_du_journal(&autre), e);
}
