//! Les règles du protocole des captations : le type d'un format audio, la plage qu'un en-tête
//! `Range` demande, et la réponse qu'on en tire (question 27). Sans entrée-sortie : l'établi lit
//! le fichier et sert les octets de la plage que ces règles disent.

/// Les octets d'une réponse du protocole, au plus (question 27).
pub const REPONSE_AU_PLUS: u64 = 1_048_576;

/// Le type d'un format audio versé, que le protocole envoie et que la lecture simple demande à
/// l'appareil : la table fermée de la question 27, à six lignes. `None` pour tout autre format,
/// `caf` et `wma` compris, qui ne sont pas servis (R3), un MIDI et un texte aussi.
pub fn type_du_format(format: &str) -> Option<&'static str> {
    match format {
        "wav" => Some("audio/wav"),
        "flac" => Some("audio/flac"),
        "mp3" => Some("audio/mpeg"),
        "m4a" => Some("audio/mp4"),
        "ogg" => Some("audio/ogg"),
        "aiff" => Some("audio/aiff"),
        _ => None,
    }
}

/// Une plage d'octets, ses deux bornes comprises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plage {
    pub debut: u64,
    pub fin: u64,
}

impl Plage {
    /// Le nombre d'octets de la plage.
    pub fn longueur(&self) -> u64 {
        self.fin - self.debut + 1
    }
}

/// Ce que demande l'en-tête `Range` d'une requête.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demande {
    Entier,
    Plage(Plage),
    Impossible,
}

/// Des chiffres seuls, sans signe ni espace, en nombre ; un nombre trop grand vaut le plus grand.
fn nombre(texte: &str) -> Option<u64> {
    if texte.is_empty() || !texte.bytes().all(|o| o.is_ascii_digit()) {
        return None;
    }
    Some(texte.parse().unwrap_or(u64::MAX))
}

/// Lit `range` pour un fichier de `taille` octets. Absent, ou d'une autre unité que `bytes`
/// (comparée sans égard à la casse) : `Entier`. `bytes=a-b` et `bytes=a-` : de `a` à `b`, ou à la
/// fin, bornés à `taille − 1` et à `a + REPONSE_AU_PLUS − 1` ; `bytes=-n` : les `min(n,
/// REPONSE_AU_PLUS, taille)` derniers octets. `Impossible` : `a` au-delà de la fin, `b` avant `a`,
/// `-0`, un fichier vide, plusieurs plages, ou toute autre forme (des chiffres seuls, sans signe
/// ni espace, autour d'un seul `-`).
pub fn plage(range: Option<&str>, taille: u64) -> Demande {
    let Some(range) = range else {
        return Demande::Entier;
    };
    let Some((unite, spec)) = range.split_once('=') else {
        return Demande::Entier;
    };
    if !unite.eq_ignore_ascii_case("bytes") {
        return Demande::Entier;
    }
    let Some(derniere) = taille.checked_sub(1) else {
        return Demande::Impossible;
    };
    let Some((a, b)) = spec.split_once('-') else {
        return Demande::Impossible;
    };
    if a.is_empty() {
        return match nombre(b) {
            Some(n) if n > 0 => {
                let longueur = n.min(REPONSE_AU_PLUS).min(taille);
                Demande::Plage(Plage {
                    debut: taille - longueur,
                    fin: derniere,
                })
            }
            _ => Demande::Impossible,
        };
    }
    let Some(debut) = nombre(a) else {
        return Demande::Impossible;
    };
    let demandee = if b.is_empty() {
        u64::MAX
    } else if let Some(fin) = nombre(b) {
        fin
    } else {
        return Demande::Impossible;
    };
    if debut > derniere || demandee < debut {
        return Demande::Impossible;
    }
    let borne = debut.saturating_add(REPONSE_AU_PLUS - 1);
    Demande::Plage(Plage {
        debut,
        fin: demandee.min(derniere).min(borne),
    })
}

/// Une réponse du protocole : son statut, ses en-têtes, et la plage du fichier à lire, `None` sans
/// corps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Servie {
    pub statut: u16,
    pub entetes: Vec<(&'static str, String)>,
    pub plage: Option<Plage>,
}

/// Les deux en-têtes que porte chaque réponse.
fn communs() -> Vec<(&'static str, String)> {
    vec![
        ("Accept-Ranges", "bytes".to_string()),
        ("X-Content-Type-Options", "nosniff".to_string()),
    ]
}

/// La réponse à `methode` sur un fichier de `taille` octets, de type `type_mime`. `GET` ou `HEAD`
/// seuls, sinon 405 et `Allow: GET, HEAD`. `Entier` : 200 et le fichier s'il tient dans
/// `REPONSE_AU_PLUS`, 206 et ses `REPONSE_AU_PLUS` premiers octets sinon ; `Plage` : 206 ;
/// `Impossible` : 416 et `Content-Range: bytes */<taille>`. Chaque réponse porte
/// `Accept-Ranges: bytes` et `X-Content-Type-Options: nosniff` ; 200 et 206, `Content-Type` et
/// `Content-Length` ; 206, `Content-Range: bytes <début>-<fin>/<taille>`. Un fichier vide, sans
/// `Range` : 200, sans plage. `HEAD` : les en-têtes de `GET`, sans plage.
pub fn servir(methode: &str, range: Option<&str>, taille: u64, type_mime: &'static str) -> Servie {
    let tete = methode == "HEAD";
    if !tete && methode != "GET" {
        let mut entetes = communs();
        entetes.push(("Allow", "GET, HEAD".to_string()));
        return Servie {
            statut: 405,
            entetes,
            plage: None,
        };
    }
    let (statut, plage) = match plage(range, taille) {
        Demande::Impossible => {
            let mut entetes = communs();
            entetes.push(("Content-Range", format!("bytes */{taille}")));
            return Servie {
                statut: 416,
                entetes,
                plage: None,
            };
        }
        Demande::Plage(p) => (206, Some(p)),
        Demande::Entier if taille == 0 => (200, None),
        Demande::Entier if taille <= REPONSE_AU_PLUS => (
            200,
            Some(Plage {
                debut: 0,
                fin: taille - 1,
            }),
        ),
        Demande::Entier => (
            206,
            Some(Plage {
                debut: 0,
                fin: REPONSE_AU_PLUS - 1,
            }),
        ),
    };
    let mut entetes = communs();
    entetes.push(("Content-Type", type_mime.to_string()));
    entetes.push((
        "Content-Length",
        plage.map_or(0, |p| p.longueur()).to_string(),
    ));
    if let (206, Some(p)) = (statut, plage) {
        entetes.push((
            "Content-Range",
            format!("bytes {}-{}/{taille}", p.debut, p.fin),
        ));
    }
    Servie {
        statut,
        entetes,
        plage: if tete { None } else { plage },
    }
}

/// 404, avec les deux en-têtes communs : une empreinte inconnue ou mal formée, ou un contenu qui
/// n'est l'audio ni d'une prise ni d'un stem, d'un format qui a un type.
pub fn introuvable() -> Servie {
    Servie {
        statut: 404,
        entetes: communs(),
        plage: None,
    }
}
