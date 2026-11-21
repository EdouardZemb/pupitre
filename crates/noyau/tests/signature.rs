//! Les signatures et les extensions : ce qu'est un fichier, d'après ses premiers octets puis son
//! extension (§ 30 · 9.2 ; questions 10, 11 et 12).

use noyau::donnees::signature::{
    FormatSonore, FormatTexte, Nature, Reconnu, apres_id3, extension, reconnaitre,
};

const ASF: [u8; 16] = [
    0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C,
];

fn debut(octets: &[u8]) -> Vec<u8> {
    let mut v = octets.to_vec();
    v.resize(16, 0);
    v
}

#[test]
fn chaque_signature_de_la_table_se_reconnait() {
    use FormatSonore::*;
    let cas: Vec<(Vec<u8>, Reconnu)> = vec![
        (
            debut(b"RIFF\0\0\0\0WAVE"),
            Reconnu::Nature(Nature::Audio(Wav)),
        ),
        (
            debut(b"RF64\0\0\0\0WAVE"),
            Reconnu::Nature(Nature::Audio(Wav)),
        ),
        (debut(b"RIFF\0\0\0\0RMID"), Reconnu::Nature(Nature::Ignore)),
        (debut(b"fLaC"), Reconnu::Nature(Nature::Audio(Flac))),
        (
            debut(&[0xFF, 0xFB, 0x90]),
            Reconnu::Nature(Nature::Audio(Mp3)),
        ),
        (debut(&[0xFF, 0xE0]), Reconnu::Nature(Nature::Audio(Mp3))),
        (
            debut(b"\0\0\0\x18ftypM4A "),
            Reconnu::Nature(Nature::Audio(M4a)),
        ),
        (
            debut(b"\0\0\0\x18ftypisom"),
            Reconnu::Nature(Nature::Audio(M4a)),
        ),
        (
            debut(b"\0\0\0\x14ftypqt  "),
            Reconnu::Nature(Nature::Audio(M4a)),
        ),
        (
            debut(b"\0\0\0\x18ftypheic"),
            Reconnu::Nature(Nature::Ignore),
        ),
        (
            debut(b"\0\0\0\x18ftypheix"),
            Reconnu::Nature(Nature::Ignore),
        ),
        (
            debut(b"\0\0\0\x18ftyphevc"),
            Reconnu::Nature(Nature::Ignore),
        ),
        (
            debut(b"\0\0\0\x18ftypmif1"),
            Reconnu::Nature(Nature::Ignore),
        ),
        (
            debut(b"\0\0\0\x18ftypmsf1"),
            Reconnu::Nature(Nature::Ignore),
        ),
        (
            debut(b"\0\0\0\x1cftypavif"),
            Reconnu::Nature(Nature::Ignore),
        ),
        (debut(b"OggS"), Reconnu::Nature(Nature::Audio(Ogg))),
        (
            debut(b"FORM\0\0\0\0AIFF"),
            Reconnu::Nature(Nature::Audio(Aiff)),
        ),
        (
            debut(b"FORM\0\0\0\0AIFC"),
            Reconnu::Nature(Nature::Audio(Aiff)),
        ),
        (debut(b"caff"), Reconnu::Nature(Nature::Audio(Caf))),
        (ASF.to_vec(), Reconnu::Nature(Nature::Audio(Wma))),
        (
            debut(b"MThd\0\0\0\x06\0\x01\0\x03\x01\xE0"),
            Reconnu::Nature(Nature::Midi),
        ),
        (
            debut(b"ID3\x04\0\0\0\0\x01\x48"),
            Reconnu::Id3 { suite: 210 },
        ),
        (
            debut(b"ID3\x04\0\x10\0\0\x01\x48"),
            Reconnu::Id3 { suite: 220 },
        ),
        // Les quatre octets de taille non nuls : (1 << 21) + (2 << 14) + (3 << 7) + 4 + 10 (balayage P6).
        (
            debut(b"ID3\x04\0\0\x01\x02\x03\x04"),
            Reconnu::Id3 { suite: 2_130_318 },
        ),
    ];
    for (octets, attendu) in cas {
        assert_eq!(reconnaitre(&octets, None), attendu, "{octets:02x?}");
    }
    assert_eq!(
        reconnaitre(&debut(&[0xFF, 0xDF]), None),
        Reconnu::Nature(Nature::Ignore)
    );
    assert_eq!(
        reconnaitre(&debut(b"FORM\0\0\0\0WAVE"), None),
        Reconnu::Nature(Nature::Ignore)
    );
    assert_eq!(
        reconnaitre(&debut(b"MThd\0\0\0\x07"), None),
        Reconnu::Nature(Nature::Ignore)
    );
}

#[test]
fn apres_une_etiquette_id3_flac_ou_mp3() {
    assert_eq!(apres_id3(b"fLaC"), FormatSonore::Flac);
    assert_eq!(apres_id3(&[0xFF, 0xFB, 0x90, 0x00]), FormatSonore::Mp3);
    assert_eq!(apres_id3(b"fLa"), FormatSonore::Mp3);
}

#[test]
fn la_signature_l_emporte_sur_l_extension() {
    assert_eq!(
        reconnaitre(&debut(b"RIFF\0\0\0\0WAVE"), Some("txt")),
        Reconnu::Nature(Nature::Audio(FormatSonore::Wav))
    );
    assert_eq!(
        reconnaitre(&debut(b"PK\x03\x04"), Some("wav")),
        Reconnu::Nature(Nature::Ignore)
    );
    assert_eq!(
        reconnaitre(&debut(b"PK\x03\x04"), Some("txt")),
        Reconnu::Nature(Nature::Texte(FormatTexte::Brut))
    );
    assert_eq!(
        reconnaitre(&debut(b"%PDF-1.7"), Some("pdf")),
        Reconnu::Nature(Nature::Ignore)
    );
    // Un `RIFF` d'un autre type, ou un `RMID` sans `RIFF`, ne dit rien : l'extension décide.
    let texte = Reconnu::Nature(Nature::Texte(FormatTexte::Brut));
    assert_eq!(reconnaitre(&debut(b"RIFF\0\0\0\0AVI "), Some("txt")), texte);
    assert_eq!(reconnaitre(&debut(b"XXXX\0\0\0\0RMID"), Some("txt")), texte);
    // Le BOM d'un UTF-16 petit-boutiste n'est pas une trame MP3 (question 10 ; balayage P3).
    assert_eq!(
        reconnaitre(&debut(&[0xFF, 0xFE, 0x50, 0x00]), Some("txt")),
        Reconnu::Nature(Nature::Texte(FormatTexte::Brut))
    );
    assert_eq!(
        reconnaitre(&debut(&[0xFF, 0xFE, 0x50, 0x00]), None),
        Reconnu::Nature(Nature::Ignore)
    );
}

#[test]
fn les_extensions_des_textes_sans_egard_a_la_casse() {
    let texte = |e| reconnaitre(b"Pont du soir", Some(e));
    for e in ["txt", "TXT", "Txt"] {
        assert_eq!(
            texte(e),
            Reconnu::Nature(Nature::Texte(FormatTexte::Brut)),
            "{e}"
        );
    }
    for e in ["md", "Md", "MARKDOWN"] {
        assert_eq!(
            texte(e),
            Reconnu::Nature(Nature::Texte(FormatTexte::Markdown)),
            "{e}"
        );
    }
    for e in ["cho", "CRD", "chopro", "chord", "ChordPro", "pro"] {
        assert_eq!(
            texte(e),
            Reconnu::Nature(Nature::Texte(FormatTexte::Chordpro)),
            "{e}"
        );
    }
    for e in ["doc", "rtf", "", "tx"] {
        assert_eq!(texte(e), Reconnu::Nature(Nature::Ignore), "{e}");
    }
    assert_eq!(
        reconnaitre(b"Pont du soir", None),
        Reconnu::Nature(Nature::Ignore)
    );
}

#[test]
fn un_debut_court_ne_se_reconnait_pas_a_tort() {
    assert_eq!(reconnaitre(b"RIFF", None), Reconnu::Nature(Nature::Ignore));
    assert_eq!(
        reconnaitre(b"ID3\x04", None),
        Reconnu::Nature(Nature::Ignore)
    );
    assert_eq!(
        reconnaitre(&ASF[..15], None),
        Reconnu::Nature(Nature::Ignore)
    );
    assert_eq!(
        reconnaitre(b"", Some("txt")),
        Reconnu::Nature(Nature::Texte(FormatTexte::Brut))
    );
    assert_eq!(reconnaitre(b"", None), Reconnu::Nature(Nature::Ignore));
}

#[test]
fn l_extension_suit_path_extension() {
    for (segment, attendue) in [
        ("a.txt", Some("txt")),
        (".txt", None),
        ("a.", Some("")),
        ("a.tar.gz", Some("gz")),
        ("sans", None),
        (".a.b", Some("b")),
        ("Démo d'hiver.m4a", Some("m4a")),
    ] {
        assert_eq!(extension(segment), attendue, "{segment}");
        assert_eq!(
            std::path::Path::new(segment)
                .extension()
                .and_then(|e| e.to_str()),
            attendue,
            "{segment}"
        );
    }
}

#[test]
fn les_noms_des_formats() {
    let noms: Vec<&str> = FormatSonore::TOUS.iter().map(|f| f.nom()).collect();
    assert_eq!(
        noms,
        ["wav", "flac", "mp3", "m4a", "ogg", "aiff", "caf", "wma"]
    );
    assert_eq!(
        [
            FormatTexte::Brut.nom(),
            FormatTexte::Markdown.nom(),
            FormatTexte::Chordpro.nom()
        ],
        ["brut", "markdown", "chordpro"]
    );
}
