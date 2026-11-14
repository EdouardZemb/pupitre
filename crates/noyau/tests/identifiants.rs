//! Les identifiants (§ 30 · 2 ; § 41 · 6) : l'ULID d'une heure et de dix octets d'aléa, l'UUID
//! version 4 de seize octets, et l'empreinte d'une captation, que le noyau forme de ce qu'il
//! reçoit.

use noyau::donnees::identifiants::{empreinte, ulid, uuid_v4};

#[test]
fn l_ulid_porte_l_heure_en_millisecondes_puis_l_alea() {
    assert_eq!(
        ulid(1_794_441_600, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        "01M76GVQ00041061050R3GG28A"
    );
    assert_eq!(
        ulid(1_700_000_000, [0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        "01HF7YAT000000000000000001"
    );
    // Des secondes prises pour des millisecondes donneraient le préfixe 0001JN7W80.
    assert_eq!(&ulid(1_700_000_000, [0; 10])[..10], "01HF7YAT00");
}

#[test]
fn une_heure_negative_compte_pour_zero() {
    assert_eq!(ulid(0, [0; 10]), "00000000000000000000000000");
    assert_eq!(ulid(-1, [0; 10]), "00000000000000000000000000");
}

#[test]
fn une_heure_au_dela_de_2_puissance_48_millisecondes_s_y_borne() {
    // 2⁴⁸ − 1 ms, la plus grande heure d'un ULID : from_parts masquerait l'heure, sans erreur
    // (00000000AR0000000000000000), et l'ULID ne se rangerait plus comme elle.
    assert_eq!(ulid(281_474_976_711, [0; 10]), "7ZZZZZZZZZ0000000000000000");
    assert_eq!(ulid(i64::MAX, [0; 10]), "7ZZZZZZZZZ0000000000000000");
    assert_eq!(ulid(281_474_976_710, [0; 10]), "7ZZZZZZZBG0000000000000000"); // sous la borne
}

#[test]
fn deux_ulid_se_rangent_comme_leurs_heures() {
    assert!(ulid(1_700_000_000, [0xFF; 10]) < ulid(1_700_000_001, [0; 10]));
}

#[test]
fn l_uuid_v4_pose_sa_version_et_sa_variante() {
    assert_eq!(uuid_v4([0; 16]), "00000000-0000-4000-8000-000000000000");
    assert_eq!(uuid_v4([0xFF; 16]), "ffffffff-ffff-4fff-bfff-ffffffffffff");
    let suite: [u8; 16] = core::array::from_fn(|i| i as u8);
    assert_eq!(uuid_v4(suite), "00010203-0405-4607-8809-0a0b0c0d0e0f");
    let tire = [
        0x91, 0x91, 0x08, 0xF7, 0x52, 0xD1, 0x33, 0x20, 0x5B, 0xAC, 0xF8, 0x47, 0xDB, 0x41, 0x48,
        0xA8,
    ];
    assert_eq!(uuid_v4(tire), "919108f7-52d1-4320-9bac-f847db4148a8");
}

#[test]
fn l_empreinte_est_le_sha256_en_hexadecimal() {
    assert_eq!(
        empreinte(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
