//! Les identifiants (§ 30 · 2 ; § 41 · 6) : l'ULID d'une idée ou d'une lignée, l'UUID de
//! l'identifiant de propriétaire et l'empreinte d'une captation. Le noyau les forme de ce qu'il
//! reçoit : l'heure et l'aléa sont des arguments, jamais l'horloge ni le hasard.

use std::fmt::Write;

use sha2::{Digest, Sha256};
use ulid::Ulid;

/// La plus grande heure d'un ULID, en millisecondes : ses 48 bits de temps.
const HEURE_MAX_MS: u64 = (1 << 48) - 1;

/// L'ULID de l'heure `heure`, en secondes (négative, elle compte pour 0 ; au-delà de 2⁴⁸ − 1 ms,
/// la plus grande heure d'un ULID, elle s'y borne), et de dix octets d'aléa (§ 30 · 2 ; § 41 · 6) :
/// `Ulid::from_parts(heure × 1 000, aléa)`, en 26 caractères de Crockford.
pub fn ulid(heure: i64, alea: [u8; 10]) -> String {
    let ms = (heure.max(0) as u64).saturating_mul(1000).min(HEURE_MAX_MS);
    // L'aléa dans les dix octets de poids faible : quatre-vingts bits, que `from_parts` ne masque
    // jamais.
    let mut octets = [0u8; 16];
    octets[6..].copy_from_slice(&alea);
    Ulid::from_parts(ms, u128::from_be_bytes(octets)).to_string()
}

/// L'UUID version 4 de seize octets reçus (RFC 9562) : l'octet 6 prend `0100` en tête, l'octet 8
/// `10` ; en minuscules, 8-4-4-4-12.
pub fn uuid_v4(mut octets: [u8; 16]) -> String {
    // `+` et non `|` : sur les bits que le masque a mis à zéro, `|` changé en `^` ne changerait
    // rien, et ce mutant équivalent survivrait.
    octets[6] = (octets[6] & 0x0F) + 0x40;
    octets[8] = (octets[8] & 0x3F) + 0x80;
    let mut texte = String::with_capacity(36);
    for (rang, octet) in octets.iter().enumerate() {
        if matches!(rang, 4 | 6 | 8 | 10) {
            texte.push('-');
        }
        let _ = write!(texte, "{octet:02x}");
    }
    texte
}

/// Le SHA-256 de `octets`, en 64 caractères hexadécimaux minuscules : l'empreinte d'une captation.
pub fn empreinte(octets: &[u8]) -> String {
    let condense = Sha256::digest(octets);
    let mut e = String::with_capacity(64);
    for octet in condense.iter() {
        let _ = write!(e, "{octet:02x}");
    }
    e
}
