//! Les en-têtes d'un WAV et d'un MIDI, le texte lisible (§ 30 · 9.2 ; question 10).

use noyau::donnees::entete::{
    ControleDuTexte, ENTETE_AU_PLUS, FaitsMidi, FaitsWav, faits_midi, faits_wav,
};
use noyau::donnees::gestes::ENTIER_SUR;

/// Un WAV : `RIFF`, la taille du reste, `WAVE`, `fmt `, les éventuels morceaux, puis `data`.
fn wav(
    frequence: u32,
    canaux: u16,
    bits: u16,
    trames: usize,
    data_declaree: Option<u32>,
    avant_data: &[u8],
) -> Vec<u8> {
    let alignement = canaux * bits / 8;
    let donnees = trames * alignement as usize;
    let mut o = b"RIFF".to_vec();
    let reste = 4 + 24 + avant_data.len() + 8 + donnees;
    o.extend((reste as u32).to_le_bytes());
    o.extend(b"WAVE");
    o.extend(b"fmt ");
    o.extend(16u32.to_le_bytes());
    o.extend(1u16.to_le_bytes());
    o.extend(canaux.to_le_bytes());
    o.extend(frequence.to_le_bytes());
    o.extend((frequence * alignement as u32).to_le_bytes());
    o.extend(alignement.to_le_bytes());
    o.extend(bits.to_le_bytes());
    o.extend(avant_data);
    o.extend(b"data");
    o.extend(data_declaree.unwrap_or(donnees as u32).to_le_bytes());
    o.extend(vec![0u8; donnees]);
    o
}

fn midi(format: u16, pistes: u16, division: u16) -> Vec<u8> {
    let mut o = b"MThd".to_vec();
    o.extend(6u32.to_be_bytes());
    o.extend(format.to_be_bytes());
    o.extend(pistes.to_be_bytes());
    o.extend(division.to_be_bytes());
    o
}

/// L'en-tête d'un RF64 mono 8 bits : `ds64`, dont la taille déclarée est `ds64`, donne `data` ; la
/// taille de `data` vaut `0xFFFF_FFFF`.
fn entete_rf64_selon(frequence: u32, data: u64, ds64: u32) -> Vec<u8> {
    let mut o = b"RF64".to_vec();
    o.extend(0xFFFF_FFFFu32.to_le_bytes());
    o.extend(b"WAVE");
    o.extend(b"ds64");
    o.extend(ds64.to_le_bytes());
    let mut corps = [0u64.to_le_bytes(), data.to_le_bytes(), 0u64.to_le_bytes()].concat();
    corps.resize(ds64 as usize + ds64 as usize % 2, 0);
    if ds64 >= 16 {
        corps[8..16].copy_from_slice(&data.to_le_bytes());
    }
    o.extend(corps);
    o.extend(b"fmt ");
    o.extend(16u32.to_le_bytes());
    o.extend(1u16.to_le_bytes());
    o.extend(1u16.to_le_bytes());
    o.extend(frequence.to_le_bytes());
    o.extend(frequence.to_le_bytes());
    o.extend(1u16.to_le_bytes());
    o.extend(8u16.to_le_bytes());
    o.extend(b"data");
    o.extend(0xFFFF_FFFFu32.to_le_bytes());
    o
}

fn entete_rf64(frequence: u32, data: u64) -> Vec<u8> {
    entete_rf64_selon(frequence, data, 24)
}

fn rf64(frequence: u32, trames: usize) -> Vec<u8> {
    let mut o = entete_rf64(frequence, trames as u64);
    o.extend(vec![0u8; trames]);
    o
}

fn rf64_sans_donnees(frequence: u32, data: u64) -> Vec<u8> {
    entete_rf64(frequence, data)
}

#[test]
fn un_wav_donne_ses_faits_et_sa_duree() {
    let octets = wav(8_000, 1, 8, 8_000, None, &[]);
    assert_eq!(octets.len(), 8_044);
    let faits = faits_wav(&octets[..ENTETE_AU_PLUS.min(octets.len())], 8_044).expect("lisible");
    assert_eq!(
        faits,
        FaitsWav {
            codage: 1,
            canaux: 1,
            frequence: 8_000,
            bits: 8,
            trames: 8_000
        }
    );
    assert_eq!(faits.duree_ms(), 1_000);
    let stereo = wav(44_100, 2, 16, 1_000, None, &[]);
    assert_eq!(stereo.len(), 4_044);
    assert_eq!(
        faits_wav(&stereo, 4_044).map(|f| (f.trames, f.duree_ms())),
        Some((1_000, 22))
    );
}

#[test]
fn la_taille_de_data_se_borne_au_fichier() {
    let octets = wav(8_000, 1, 8, 4_000, Some(8_000), &[]);
    assert_eq!(octets.len(), 4_044);
    assert_eq!(
        faits_wav(&octets, 4_044).map(|f| (f.trames, f.duree_ms())),
        Some((4_000, 500))
    );
}

#[test]
fn un_morceau_avant_data_et_un_morceau_impair_se_sautent() {
    // `LIST` de 9 octets, complété d'un octet, entre `fmt ` et `data`.
    let liste = [b"LIST".as_slice(), &9u32.to_le_bytes(), b"INFOabcde", &[0]].concat();
    let octets = wav(8_000, 1, 8, 800, None, &liste);
    assert_eq!(
        faits_wav(&octets, octets.len() as u64).map(|f| f.trames),
        Some(800)
    );
}

#[test]
fn un_rf64_prend_sa_taille_dans_ds64() {
    let octets = rf64(8_000, 800);
    assert_eq!(
        faits_wav(&octets, octets.len() as u64).map(|f| f.trames),
        Some(800)
    );
}

#[test]
fn la_duree_se_calcule_sans_debordement() {
    let faits = FaitsWav {
        codage: 1,
        canaux: 1,
        frequence: 48_000,
        bits: 8,
        trames: 20_000_000_000_000_000,
    };
    assert_eq!(faits.duree_ms(), 416_666_666_666_666);
}

#[test]
fn une_duree_au_dela_d_entier_sur_est_illisible() {
    // 1 000 Hz, mono, 8 bits : une trame vaut une milliseconde ; la taille du fichier, `u64::MAX`,
    // ne borne rien.
    let au_plus = rf64_sans_donnees(1_000, ENTIER_SUR);
    assert_eq!(
        faits_wav(&au_plus, u64::MAX).map(|f| f.duree_ms()),
        Some(ENTIER_SUR)
    );
    let au_dela = rf64_sans_donnees(1_000, ENTIER_SUR + 1);
    assert_eq!(faits_wav(&au_dela, u64::MAX), None);
}

#[test]
fn un_en_tete_incoherent_est_illisible() {
    let bon = wav(8_000, 1, 8, 80, None, &[]);
    assert!(faits_wav(&bon[..36], 124).is_none()); // sans l'en-tête de `data`
    assert!(faits_wav(&bon[..20], 124).is_none()); // `fmt ` coupé
    let mut sans_frequence = bon.clone();
    sans_frequence[24..28].copy_from_slice(&0u32.to_le_bytes());
    assert!(faits_wav(&sans_frequence, 124).is_none());
    let mut sans_bloc = bon.clone();
    sans_bloc[32..34].copy_from_slice(&0u16.to_le_bytes());
    assert!(faits_wav(&sans_bloc, 124).is_none());
    assert!(faits_wav(b"RIFF\0\0\0\0AVI ", 12).is_none());
    // `data` au-delà des 65 536 premiers octets : un morceau `JUNK` de 70 000 octets avant lui.
    let loin = wav(
        8_000,
        1,
        8,
        80,
        None,
        &[
            b"JUNK".as_slice(),
            &70_000u32.to_le_bytes(),
            &vec![0; 70_000],
        ]
        .concat(),
    );
    assert!(faits_wav(&loin[..ENTETE_AU_PLUS], loin.len() as u64).is_none());
}

#[test]
fn un_wav_aux_morceaux_sans_fin_est_illisible() {
    // Des morceaux de taille nulle jusqu'au bout des 65 536 octets, sans `data` : le parcours
    // finit. 12 + 8 191 × 8 = 65 540 octets.
    let mut octets = b"RIFF\0\0\0\0WAVE".to_vec();
    octets.extend(b"JUNK\0\0\0\0".repeat(8_191));
    assert_eq!(octets.len(), 65_540);
    assert!(faits_wav(&octets, octets.len() as u64).is_none());
}

#[test]
fn un_midi_donne_son_format_et_sa_division() {
    assert_eq!(
        faits_midi(&midi(1, 3, 480)),
        Some(FaitsMidi {
            format: 1,
            pistes: 3,
            division: 480
        })
    );
    assert_eq!(
        faits_midi(&midi(1, 3, 480)).map(|f| f.format_smf()),
        Some("smf1")
    );
    assert_eq!(
        faits_midi(&midi(0, 1, 0xE728)).map(|f| (f.format_smf(), f.division)),
        Some(("smf0", 59_176))
    );
    assert_eq!(
        faits_midi(&midi(2, 4, 96)).map(|f| f.format_smf()),
        Some("smf2")
    );
    assert_eq!(faits_midi(&midi(3, 1, 96)), None);
    assert_eq!(faits_midi(&midi(0, 2, 96)), None);
    assert_eq!(faits_midi(&midi(0, 0, 96)), None);
    assert_eq!(faits_midi(&midi(1, 3, 480)[..13]), None);
}

fn lisible(troncons: &[&[u8]]) -> bool {
    let mut controle = ControleDuTexte::nouveau();
    for t in troncons {
        controle.ajouter(t);
    }
    controle.lisible()
}

#[test]
fn un_texte_utf8_avec_ou_sans_bom_est_lisible() {
    assert!(lisible(&["Pont du soir\n".as_bytes()]));
    assert!(lisible(&[b"\xEF\xBB\xBF", "Pont du soir".as_bytes()]));
    assert!(lisible(&[]));
}

#[test]
fn un_texte_utf16_avec_bom_est_lisible() {
    assert!(lisible(&[&[0xFF, 0xFE, 0x6C, 0x00, 0x61, 0x00]]));
    assert!(lisible(&[&[0xFE, 0xFF, 0x00, 0x6C, 0x00, 0x61]]));
    assert!(lisible(&[&[0xFF, 0xFE, 0x3D, 0xD8], &[0x00, 0xDE]])); // U+1F600, coupé entre deux tronçons
    assert!(lisible(&[&[0xFF], &[0xFE, 0x6C, 0x00]])); // le BOM coupé : un `Read` rend un octet
    assert!(lisible(&[&[0xFE], &[0xFF, 0x00, 0x6C]]));
}

#[test]
fn un_texte_ni_utf8_ni_utf16_avec_bom_est_illisible() {
    assert!(!lisible(&[b"\xE9t\xE9"])); // « été » en Latin-1
    assert!(!lisible(&[&[0xFF, 0xFE, 0x6C]])); // UTF-16 de longueur impaire
    assert!(!lisible(&[&[0xFF, 0xFE, 0x00, 0xD8]])); // demi-paire haute seule
    assert!(!lisible(&[&[0xFF, 0xFE, 0x00, 0xDC, 0x6C, 0x00]])); // demi-paire basse seule
}

#[test]
fn un_texte_coupe_au_milieu_d_un_caractere_reste_lisible() {
    assert!(lisible(&[&[0xC3], &[0xA9]])); // « é », coupé
    assert!(!lisible(&[b"pont ", &[0xC3]])); // coupé au bout du texte
    assert!(lisible(&[&[0xF0, 0x9F], &[0x98], &[0x80]]));
}

#[test]
fn un_morceau_fmt_ou_ds64_trop_court_est_illisible() {
    // `fmt ` déclare 15 octets : il en faut 16 au moins.
    let mut court = wav(8_000, 1, 8, 80, None, &[]);
    court[16..20].copy_from_slice(&15u32.to_le_bytes());
    assert!(faits_wav(&court, 124).is_none());
    // `ds64` de 16 octets, le moins qui porte la taille de `data` ; de 15, trop court.
    let juste = entete_rf64_selon(8_000, 800, 16);
    assert_eq!(faits_wav(&juste, u64::MAX).map(|f| f.trames), Some(800));
    assert!(faits_wav(&entete_rf64_selon(8_000, 800, 15), u64::MAX).is_none());
}

#[test]
fn un_wav_aux_nombreux_morceaux_vides_avant_data_se_lit() {
    // Douze morceaux `JUNK` de taille nulle entre `fmt ` et `data` : le parcours les passe tous.
    let octets = wav(8_000, 1, 8, 800, None, &b"JUNK\x00\x00\x00\x00".repeat(12));
    assert_eq!(
        faits_wav(&octets, octets.len() as u64).map(|f| f.trames),
        Some(800)
    );
    // Au plus juste : sans les données, l'en-tête de `data` est le dernier morceau des octets lus.
    let juste = &octets[..octets.len() - 800];
    assert_eq!(
        faits_wav(juste, octets.len() as u64).map(|f| f.trames),
        Some(800)
    );
}

#[test]
fn un_bom_utf16_seul_est_un_texte_vide_lisible() {
    assert!(lisible(&[&[0xFF, 0xFE]]));
    assert!(lisible(&[&[0xFE, 0xFF]]));
}
