//! Les tests des stems : le type proposé, le MIDI d'un stem, le jeu proposé et la durée
//! (§ 30 · 9.2 ; questions 16, 17, 18 et 37).

use noyau::donnees::stems::{
    MOTS_DES_TYPES, TypeDeStem, WavInspecte, duree_de_la_prise_en_stems, jeu_propose, mots_du_nom,
    stem_du_midi, type_propose,
};

use TypeDeStem::*;
const ATTENDUS: &[(&str, TypeDeStem)] = &[
    ("backing", Choeurs),
    ("bass", Basse),
    ("basse", Basse),
    ("batterie", Batterie),
    ("brass", Cuivres),
    ("cello", Cordes),
    ("chant", Voix),
    ("choeur", Choeurs),
    ("choeurs", Choeurs),
    ("choir", Choeurs),
    ("clavier", Clavier),
    ("claviers", Clavier),
    ("cordes", Cordes),
    ("cuivres", Cuivres),
    ("drum", Batterie),
    ("drums", Batterie),
    ("effects", Effets),
    ("effet", Effets),
    ("effets", Effets),
    ("fx", Effets),
    ("guitar", Guitare),
    ("guitare", Guitare),
    ("guitares", Guitare),
    ("guitars", Guitare),
    ("keyboard", Clavier),
    ("keys", Clavier),
    ("organ", Clavier),
    ("orgue", Clavier),
    ("pad", Synthe),
    ("perc", Percussions),
    ("percussion", Percussions),
    ("percussions", Percussions),
    ("piano", Clavier),
    ("strings", Cordes),
    ("synth", Synthe),
    ("synthe", Synthe),
    ("synths", Synthe),
    ("trompette", Cuivres),
    ("trumpet", Cuivres),
    ("violin", Cordes),
    ("violon", Cordes),
    ("vocal", Voix),
    ("vocals", Voix),
    ("voice", Voix),
    ("voix", Voix),
    ("vox", Voix),
];

#[test]
fn la_table_des_mots_est_rangee_et_chaque_mot_donne_son_type() {
    assert!(MOTS_DES_TYPES.windows(2).all(|p| p[0].0 < p[1].0));
    assert_eq!(MOTS_DES_TYPES, ATTENDUS);
    for (mot, attendu) in ATTENDUS {
        assert_eq!(
            type_propose(Some(&format!("Jeu/{mot}.wav"))),
            *attendu,
            "{mot}"
        );
    }
}

#[test]
fn le_type_se_propose_d_un_mot_entier() {
    assert_eq!(type_propose(Some("Jeu A/Voix lead.wav")), Voix);
    assert_eq!(
        type_propose(Some("Jeu A/02_Batterie-overheads.wav")),
        Batterie
    );
    assert_eq!(type_propose(Some("Jeu A/Chœurs.wav")), Choeurs);
    assert_eq!(type_propose(Some("Jeu A/SYNTHÉ pad.wav")), Synthe);
    assert_eq!(type_propose(Some("Jeu A/voix.batterie.wav")), Voix);
    assert_eq!(type_propose(Some("Basse et voix.wav")), Basse);
    assert_eq!(
        type_propose(Some("Voix/piste.wav")),
        Autre,
        "seul le dernier segment compte"
    );
}

#[test]
fn sans_mot_connu_le_type_est_autre() {
    assert_eq!(type_propose(None), Autre);
    assert_eq!(type_propose(Some("Jeu A/bassline.wav")), Autre);
    assert_eq!(type_propose(Some("Jeu A/piste 7.wav")), Autre);
    assert_eq!(
        mots_du_nom("Jeu A/02_Batterie-overheads.wav"),
        ["02", "batterie", "overheads"]
    );
    assert_eq!(
        mots_du_nom(".wav"),
        ["wav"],
        "`.wav` n'a pas d'extension : son seul mot est `wav`"
    );
}

#[test]
fn les_types_ont_leur_nom_et_leur_libelle() {
    let noms: Vec<_> = TypeDeStem::TOUS.iter().map(|t| t.nom()).collect();
    assert_eq!(
        noms,
        [
            "voix",
            "choeurs",
            "batterie",
            "percussions",
            "basse",
            "guitare",
            "clavier",
            "cordes",
            "cuivres",
            "synthe",
            "effets",
            "autre"
        ]
    );
    assert_eq!(
        (Choeurs.libelle(), Synthe.libelle(), Voix.libelle()),
        ("chœurs", "synthé", "voix")
    );
    for t in TypeDeStem::TOUS {
        assert_eq!(TypeDeStem::lire(t.nom()), Some(t));
    }
    assert_eq!(TypeDeStem::lire("chœurs"), None);
}

#[test]
fn le_midi_accompagne_le_stem_de_meme_nom() {
    let stems = ["Jeu A/basse.wav", "Jeu A/voix.wav"];
    assert_eq!(
        stem_du_midi("Jeu A/voix.mid", &stems),
        Some("Jeu A/voix.wav")
    );
    assert_eq!(stem_du_midi("Jeu A/Voix.mid", &stems), None);
    assert_eq!(stem_du_midi("Jeu A/voix 2.mid", &stems), None);
    assert_eq!(stem_du_midi("Jeu A/voix", &stems), Some("Jeu A/voix.wav"));
}

#[test]
fn trois_wav_de_meme_duree_proposent_un_jeu() {
    let w = |taille, debut: &'static [u8], trames, frequence| WavInspecte {
        taille,
        debut,
        trames,
        frequence,
    };
    let (a, b, c): (&[u8], &[u8], &[u8]) = (&[1; 44], &[2; 44], &[3; 44]);
    assert!(jeu_propose(&[
        w(8_044, a, 8_000, 8_000),
        w(8_044, b, 8_000, 8_000),
        w(8_044, c, 8_000, 8_000)
    ]));
    assert!(
        !jeu_propose(&[w(8_044, a, 8_000, 8_000), w(8_044, a, 8_000, 8_000)]),
        "même taille, même début : rien ne les distingue"
    );
    assert!(
        jeu_propose(&[w(8_044, a, 8_000, 8_000), w(8_052, a, 8_000, 8_000)]),
        "des tailles distinctes suffisent"
    );
    assert!(!jeu_propose(&[
        w(8_044, a, 8_000, 8_000),
        w(8_044, b, 8_000, 44_100)
    ]));
    // 4 000 trames à 8 000 Hz et 8 000 trames à 16 000 Hz : 500 ms chacun (balayage P12).
    assert!(
        !jeu_propose(&[w(4_044, a, 4_000, 8_000), w(8_044, b, 8_000, 16_000)]),
        "même durée, trames et fréquence autres"
    );
    assert!(!jeu_propose(&[w(8_044, a, 8_000, 8_000)]));
}

#[test]
fn la_duree_d_une_prise_en_stems() {
    assert_eq!(
        duree_de_la_prise_en_stems(&[Some(1_000), Some(1_500)]),
        Some(1_500)
    );
    assert_eq!(duree_de_la_prise_en_stems(&[Some(1_000), None]), None);
    assert_eq!(duree_de_la_prise_en_stems(&[]), None);
}
