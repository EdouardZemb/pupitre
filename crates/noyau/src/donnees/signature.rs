//! Les signatures et les extensions : ce qu'est un fichier que l'import rencontre, d'après ses
//! premiers octets, puis son extension (§ 30 · 9.2 ; questions 10, 11 et 12). Sans entrée-sortie :
//! l'appelant lit le début du fichier et nous le donne.

/// Les octets qu'on lit d'un fichier pour le reconnaître, au plus (question 11) : un fichier
/// ignoré n'est jamais lu au-delà.
pub const SIGNATURE_AU_PLUS: usize = 16;

/// Les formats audio que l'import reconnaît à leur signature (§ 30 · 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatSonore {
    Wav,
    Flac,
    Mp3,
    M4a,
    Ogg,
    Aiff,
    Caf,
    Wma,
}

impl FormatSonore {
    /// Les huit formats, dans l'ordre de la déclaration.
    pub const TOUS: [FormatSonore; 8] = [
        FormatSonore::Wav,
        FormatSonore::Flac,
        FormatSonore::Mp3,
        FormatSonore::M4a,
        FormatSonore::Ogg,
        FormatSonore::Aiff,
        FormatSonore::Caf,
        FormatSonore::Wma,
    ];

    /// `wav`, `flac`, `mp3`, `m4a`, `ogg`, `aiff`, `caf`, `wma` : la valeur de `<e>/format`.
    pub fn nom(self) -> &'static str {
        match self {
            FormatSonore::Wav => "wav",
            FormatSonore::Flac => "flac",
            FormatSonore::Mp3 => "mp3",
            FormatSonore::M4a => "m4a",
            FormatSonore::Ogg => "ogg",
            FormatSonore::Aiff => "aiff",
            FormatSonore::Caf => "caf",
            FormatSonore::Wma => "wma",
        }
    }
}

/// Les formats d'un texte versé, d'après son extension (question 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FormatTexte {
    Brut,
    Markdown,
    Chordpro,
}

impl FormatTexte {
    /// `brut`, `markdown`, `chordpro`.
    pub fn nom(self) -> &'static str {
        match self {
            FormatTexte::Brut => "brut",
            FormatTexte::Markdown => "markdown",
            FormatTexte::Chordpro => "chordpro",
        }
    }
}

/// Ce qu'est un fichier, d'après sa signature ou son extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nature {
    Audio(FormatSonore),
    Midi,
    Texte(FormatTexte),
    Ignore,
}

/// Ce que dit le début d'un fichier : sa nature, ou une étiquette ID3, dont la suite, à l'octet
/// `suite`, dit FLAC ou MP3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reconnu {
    Nature(Nature),
    Id3 { suite: u64 },
}

const ASF: [u8; 16] = [
    0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C,
];

/// Les codes de format d'image que porte une boîte `ftyp` (question 36) : ni audio, ni vidéo.
const CODES_IMAGE: [&[u8]; 6] = [b"heic", b"heix", b"hevc", b"mif1", b"msf1", b"avif"];

fn nature(n: Nature) -> Reconnu {
    Reconnu::Nature(n)
}

/// La nature d'un fichier d'après `debut`, ses `SIGNATURE_AU_PLUS` premiers octets au plus (moins
/// s'il est plus court), puis son extension, sans égard à la casse ; la signature l'emporte. Un
/// fichier qu'elle dit ignoré n'est jamais lu au-delà de `debut`.
pub fn reconnaitre(debut: &[u8], extension: Option<&str>) -> Reconnu {
    let debut = &debut[..debut.len().min(SIGNATURE_AU_PLUS)];
    let a = |de: usize, fin: usize, valeur: &[u8]| debut.get(de..fin) == Some(valeur);

    if (a(0, 4, b"RIFF") || a(0, 4, b"RF64")) && a(8, 12, b"WAVE") {
        return nature(Nature::Audio(FormatSonore::Wav));
    }
    if a(0, 4, b"RIFF") && a(8, 12, b"RMID") {
        return nature(Nature::Ignore);
    }
    if a(0, 4, b"fLaC") {
        return nature(Nature::Audio(FormatSonore::Flac));
    }
    if debut.len() >= 10 && a(0, 3, b"ID3") {
        let taille = debut[6..10]
            .iter()
            .fold(0u64, |reste, octet| reste * 128 + u64::from(octet & 0x7F));
        let pied = if debut[5] & 0x10 != 0 { 10 } else { 0 };
        return Reconnu::Id3 {
            suite: 10 + taille + pied,
        };
    }
    if debut.first() == Some(&0xFF) && debut.get(1).is_some_and(|&o| o >= 0xE0 && o != 0xFE) {
        return nature(Nature::Audio(FormatSonore::Mp3));
    }
    if a(4, 8, b"ftyp")
        && let Some(code) = debut.get(8..12)
    {
        return nature(if CODES_IMAGE.contains(&code) {
            Nature::Ignore
        } else {
            Nature::Audio(FormatSonore::M4a)
        });
    }
    if a(0, 4, b"OggS") {
        return nature(Nature::Audio(FormatSonore::Ogg));
    }
    if a(0, 4, b"FORM") && (a(8, 12, b"AIFF") || a(8, 12, b"AIFC")) {
        return nature(Nature::Audio(FormatSonore::Aiff));
    }
    if a(0, 4, b"caff") {
        return nature(Nature::Audio(FormatSonore::Caf));
    }
    if debut.starts_with(&ASF) {
        return nature(Nature::Audio(FormatSonore::Wma));
    }
    if debut.starts_with(b"MThd\0\0\0\x06") {
        return nature(Nature::Midi);
    }
    let Some(extension) = extension else {
        return nature(Nature::Ignore);
    };
    let est = |noms: &[&str]| noms.iter().any(|n| extension.eq_ignore_ascii_case(n));
    if est(&["txt"]) {
        nature(Nature::Texte(FormatTexte::Brut))
    } else if est(&["md", "markdown"]) {
        nature(Nature::Texte(FormatTexte::Markdown))
    } else if est(&["cho", "crd", "chopro", "chord", "chordpro", "pro"]) {
        nature(Nature::Texte(FormatTexte::Chordpro))
    } else {
        nature(Nature::Ignore)
    }
}

/// FLAC si `quatre`, les quatre octets qui suivent une étiquette ID3, valent `fLaC` ; MP3 sinon.
pub fn apres_id3(quatre: &[u8]) -> FormatSonore {
    if quatre.starts_with(b"fLaC") {
        FormatSonore::Flac
    } else {
        FormatSonore::Mp3
    }
}

/// L'extension d'un segment de chemin, comme `Path::extension` : ce qui suit son dernier point,
/// `None` sans point ou quand le seul point le commence.
pub fn extension(segment: &str) -> Option<&str> {
    let (nom, ext) = segment.rsplit_once('.')?;
    if nom.is_empty() { None } else { Some(ext) }
}
