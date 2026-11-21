//! Les en-têtes d'un WAV et d'un MIDI et le contrôle d'un texte versé (§ 30 · 9.2 ; questions 10
//! et 16), sans entrée-sortie : l'établi lit les premiers octets d'un fichier, le noyau en tire
//! les faits.

use super::gestes::ENTIER_SUR;

/// Les octets du début d'un WAV où l'établi cherche son en-tête (`fmt ` et l'en-tête de `data`) :
/// au-delà, le WAV est illisible (question 10).
pub const ENTETE_AU_PLUS: usize = 65_536;

/// La taille de `data`, dans un RF64 seul, dont la vraie taille est dans `ds64`.
const TAILLE_RF64: u32 = 0xFFFF_FFFF;

/// Les faits d'un WAV lus dans son en-tête (§ 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaitsWav {
    pub codage: u16,
    pub canaux: u16,
    pub frequence: u32,
    pub bits: u16,
    pub trames: u64,
}

impl FaitsWav {
    /// ⌊trames × 1 000 / fréquence⌋, calculé en `u128` (question 16).
    pub fn duree_ms(&self) -> u64 {
        let ms = (u128::from(self.trames) * 1_000)
            .checked_div(u128::from(self.frequence))
            .unwrap_or(0);
        u64::try_from(ms).unwrap_or(u64::MAX)
    }
}

fn u16_le(o: &[u8], rang: usize) -> Option<u16> {
    Some(u16::from_le_bytes(o.get(rang..rang + 2)?.try_into().ok()?))
}

fn u32_le(o: &[u8], rang: usize) -> Option<u32> {
    Some(u32::from_le_bytes(o.get(rang..rang + 4)?.try_into().ok()?))
}

fn u64_le(o: &[u8], rang: usize) -> Option<u64> {
    Some(u64::from_le_bytes(o.get(rang..rang + 8)?.try_into().ok()?))
}

/// Ce que dit `fmt ` : codage, canaux, fréquence, alignement d'un bloc, bits.
struct Fmt {
    codage: u16,
    canaux: u16,
    frequence: u32,
    alignement: u16,
    bits: u16,
}

/// Les faits d'un WAV : `debut`, ses `ENTETE_AU_PLUS` premiers octets au plus ; `taille`, celle du
/// fichier. `RIFF` ou `RF64` puis `WAVE` ; les morceaux depuis l'octet 12, chacun `<id><taille u32
/// LE><données>`, complété à un nombre pair ; `fmt ` (16 octets au moins) donne le codage, les
/// canaux, la fréquence, l'alignement d'un bloc et les bits ; `data` donne les trames : sa taille,
/// bornée à ce que le fichier porte après son en-tête, divisée par l'alignement. Dans un `RF64`
/// seul, `ds64` (16 octets au moins) donne la taille de `data` quand celle de son morceau vaut
/// `0xFFFF_FFFF`. Dans un `RIFF`, `ds64` est un morceau inconnu, sauté comme les autres, et
/// `0xFFFF_FFFF` une taille de `data` comme une autre, bornée au fichier : un WAV écrit en flux,
/// un enregistrement interrompu (question 16). `None` (illisible) sans `fmt ` ou sans l'en-tête de
/// `data` dans `debut`, si la fréquence ou l'alignement vaut 0, si la durée passe
/// `gestes::ENTIER_SUR`, ou pour un `RF64` dont la taille de `data` renvoie à un `ds64` absent, ou
/// dont le `ds64` a moins de 16 octets. Le parcours des morceaux est borné : chacun avance d'au
/// moins huit octets, et `debut` n'en compte pas plus que son nombre de morceaux.
pub fn faits_wav(debut: &[u8], taille: u64) -> Option<FaitsWav> {
    let forme = debut.get(0..4)?;
    if forme != b"RIFF" && forme != b"RF64" {
        return None;
    }
    let rf64 = forme == b"RF64";
    if debut.get(8..12)? != b"WAVE" {
        return None;
    }
    let mut fmt: Option<Fmt> = None;
    let mut taille_ds64: Option<u64> = None;
    let mut position = 12usize;
    for _ in 0..debut.len() {
        let id = debut.get(position..position + 4)?;
        let morceau = u32_le(debut, position + 4)?;
        let corps = position + 8;
        match id {
            b"fmt " => {
                if morceau < 16 {
                    return None;
                }
                fmt = Some(Fmt {
                    codage: u16_le(debut, corps)?,
                    canaux: u16_le(debut, corps + 2)?,
                    frequence: u32_le(debut, corps + 4)?,
                    alignement: u16_le(debut, corps + 12)?,
                    bits: u16_le(debut, corps + 14)?,
                });
            }
            b"ds64" if rf64 => {
                if morceau < 16 {
                    return None;
                }
                taille_ds64 = Some(u64_le(debut, corps + 8)?);
            }
            b"data" => {
                let f = fmt?;
                if f.frequence == 0 || f.alignement == 0 {
                    return None;
                }
                let declaree = if rf64 && morceau == TAILLE_RF64 {
                    taille_ds64?
                } else {
                    u64::from(morceau)
                };
                let portee = declaree.min(taille.saturating_sub(corps as u64));
                let faits = FaitsWav {
                    codage: f.codage,
                    canaux: f.canaux,
                    frequence: f.frequence,
                    bits: f.bits,
                    trames: portee / u64::from(f.alignement),
                };
                return (faits.duree_ms() <= ENTIER_SUR).then_some(faits);
            }
            _ => {}
        }
        position = corps
            .checked_add(morceau as usize)?
            .checked_add(morceau as usize % 2)?;
    }
    None
}

/// Les faits d'un MIDI (§ 30 · 9.2 ; L18).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaitsMidi {
    pub format: u16,
    pub pistes: u16,
    pub division: u16,
}

impl FaitsMidi {
    /// `smf0`, `smf1` ou `smf2`.
    pub fn format_smf(&self) -> &'static str {
        match self.format {
            0 => "smf0",
            1 => "smf1",
            _ => "smf2",
        }
    }
}

fn u16_be(o: &[u8], rang: usize) -> Option<u16> {
    Some(u16::from_be_bytes(o.get(rang..rang + 2)?.try_into().ok()?))
}

/// Les faits d'un MIDI, de ses 14 premiers octets : `MThd 00 00 00 06`, le format, les pistes et la
/// division, gros-boutistes, la division brute (SMPTE compris). `None` (illisible) sous 14 octets,
/// pour un format au-delà de 2, ou un format 0 qui n'a pas une seule piste.
pub fn faits_midi(debut: &[u8]) -> Option<FaitsMidi> {
    if debut.get(0..8)? != b"MThd\x00\x00\x00\x06" {
        return None;
    }
    let format = u16_be(debut, 8)?;
    let pistes = u16_be(debut, 10)?;
    let division = u16_be(debut, 12)?;
    if format > 2 || (format == 0 && pistes != 1) {
        return None;
    }
    Some(FaitsMidi {
        format,
        pistes,
        division,
    })
}

/// L'encodage qu'un texte a montré.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Encodage {
    Inconnu,
    Utf8,
    Utf16Petit,
    Utf16Gros,
}

/// Le contrôle d'un texte versé, tronçon par tronçon (question 10) : UTF-8, `EF BB BF` admis en
/// tête ; ou UTF-16 dont les deux premiers octets sont `FF FE` (petit-boutiste) ou `FE FF`
/// (gros-boutiste), même coupés entre deux tronçons, de longueur paire, chaque demi-paire de
/// substitution appariée. Un caractère coupé entre deux tronçons se reporte au suivant ; au plus
/// trois octets se reportent.
pub struct ControleDuTexte {
    encodage: Encodage,
    report: Vec<u8>,
    refuse: bool,
}

impl Default for ControleDuTexte {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl ControleDuTexte {
    pub fn nouveau() -> Self {
        Self {
            encodage: Encodage::Inconnu,
            report: Vec::new(),
            refuse: false,
        }
    }

    pub fn ajouter(&mut self, troncon: &[u8]) {
        if self.refuse {
            return;
        }
        let mut octets = std::mem::take(&mut self.report);
        octets.extend_from_slice(troncon);
        if self.encodage == Encodage::Inconnu {
            match octets.first() {
                None => return,
                Some(0xFF | 0xFE) => {
                    if octets.len() < 2 {
                        self.report = octets;
                        return;
                    }
                    self.encodage = match (octets[0], octets[1]) {
                        (0xFF, 0xFE) => Encodage::Utf16Petit,
                        (0xFE, 0xFF) => Encodage::Utf16Gros,
                        _ => {
                            self.refuse = true;
                            return;
                        }
                    };
                    octets.drain(..2);
                }
                Some(_) => self.encodage = Encodage::Utf8,
            }
        }
        if self.encodage == Encodage::Utf8 {
            self.controler_utf8(&octets);
        } else {
            self.controler_utf16(&octets);
        }
    }

    fn controler_utf8(&mut self, octets: &[u8]) {
        if let Err(erreur) = std::str::from_utf8(octets) {
            if erreur.error_len().is_some() {
                self.refuse = true;
            } else {
                self.report = octets[erreur.valid_up_to()..].to_vec();
            }
        }
    }

    fn controler_utf16(&mut self, octets: &[u8]) {
        let gros = self.encodage == Encodage::Utf16Gros;
        let (paires, impair) = octets.split_at(octets.len() / 2 * 2);
        let mut unites: Vec<u16> = paires
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| {
                if gros {
                    u16::from_be_bytes(*p)
                } else {
                    u16::from_le_bytes(*p)
                }
            })
            .collect();
        let mut report = Vec::new();
        // Une demi-paire haute en queue attend sa basse, dans le tronçon suivant.
        if let Some(&derniere) = unites.last()
            && (0xD800..0xDC00).contains(&derniere)
        {
            unites.pop();
            report.extend_from_slice(&paires[paires.len() - 2..]);
        }
        report.extend_from_slice(impair);
        if char::decode_utf16(unites).any(|c| c.is_err()) {
            self.refuse = true;
        } else {
            self.report = report;
        }
    }

    /// Vrai si tout ce qui a été ajouté forme un texte lisible, rien ne restant reporté.
    pub fn lisible(&self) -> bool {
        !self.refuse && self.report.is_empty()
    }
}
