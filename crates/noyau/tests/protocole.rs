//! Les tests des règles du protocole des captations : les types, les plages et les réponses
//! (question 27).

use noyau::donnees::protocole::{
    Demande, Plage, Servie, introuvable, plage, servir, type_du_format,
};

const TROIS_MILLIONS: u64 = 3_000_000;

fn entete<'a>(servie: &'a Servie, nom: &str) -> Option<&'a str> {
    servie
        .entetes
        .iter()
        .find(|(n, _)| *n == nom)
        .map(|(_, valeur)| valeur.as_str())
}

#[test]
fn chaque_format_a_son_type() {
    for (format, attendu) in [
        ("wav", "audio/wav"),
        ("flac", "audio/flac"),
        ("mp3", "audio/mpeg"),
        ("m4a", "audio/mp4"),
        ("ogg", "audio/ogg"),
        ("aiff", "audio/aiff"),
    ] {
        assert_eq!(type_du_format(format), Some(attendu), "{format}");
    }
    for format in ["caf", "wma", "smf1", "chordpro", "brut", "", "WAV"] {
        assert_eq!(type_du_format(format), None, "{format}");
    }
}

#[test]
fn une_requete_sans_plage_qui_tient_rend_200() {
    let s = servir("GET", None, 1_000, "audio/wav");
    assert_eq!(
        (s.statut, s.plage),
        (200, Some(Plage { debut: 0, fin: 999 }))
    );
    assert_eq!(
        (entete(&s, "Content-Length"), entete(&s, "Content-Type")),
        (Some("1000"), Some("audio/wav"))
    );
    assert_eq!(entete(&s, "Content-Range"), None);
    let borne = servir("GET", None, 1_048_576, "audio/wav");
    assert_eq!(
        (borne.statut, borne.plage),
        (
            200,
            Some(Plage {
                debut: 0,
                fin: 1_048_575
            })
        )
    );
    assert_eq!(servir("GET", None, 1_048_577, "audio/wav").statut, 206);
    let vide = servir("GET", None, 0, "audio/wav");
    assert_eq!(
        (vide.statut, vide.plage, entete(&vide, "Content-Length")),
        (200, None, Some("0"))
    );
}

#[test]
fn une_requete_sans_plage_au_dela_de_la_borne_rend_206() {
    let s = servir("GET", None, TROIS_MILLIONS, "audio/wav");
    assert_eq!(
        (s.statut, s.plage),
        (
            206,
            Some(Plage {
                debut: 0,
                fin: 1_048_575
            })
        )
    );
    assert_eq!(entete(&s, "Content-Range"), Some("bytes 0-1048575/3000000"));
    assert_eq!(entete(&s, "Content-Length"), Some("1048576"));
}

#[test]
fn les_plages_se_bornent() {
    let p = |range: &str, taille| plage(Some(range), taille);
    let pl = |debut, fin| Demande::Plage(Plage { debut, fin });
    assert_eq!(p("bytes=0-", TROIS_MILLIONS), pl(0, 1_048_575));
    assert_eq!(p("bytes=100-199", TROIS_MILLIONS), pl(100, 199));
    assert_eq!(
        p("bytes=2999990-", TROIS_MILLIONS),
        pl(2_999_990, 2_999_999)
    );
    assert_eq!(
        p("bytes=2999990-4000000", TROIS_MILLIONS),
        pl(2_999_990, 2_999_999)
    );
    assert_eq!(p("bytes=-500", TROIS_MILLIONS), pl(2_999_500, 2_999_999));
    assert_eq!(
        p("bytes=-5000000", TROIS_MILLIONS),
        pl(1_951_424, 2_999_999)
    );
    assert_eq!(p("bytes=-5000", 1_000), pl(0, 999));
    assert_eq!(p("Bytes=0-9", 1_000), pl(0, 9));
    assert_eq!(p("bytes=7-7", 1_000), pl(7, 7));
    assert_eq!(p("bytes=999-", 1_000), pl(999, 999));
    assert_eq!(p("bytes=-1", 1_000), pl(999, 999));
    assert_eq!(p("bytes=0-99999999999999999999999", 1_000), pl(0, 999));
    let s = servir("GET", Some("bytes=100-199"), TROIS_MILLIONS, "audio/wav");
    assert_eq!(
        (
            s.statut,
            entete(&s, "Content-Range"),
            entete(&s, "Content-Length")
        ),
        (206, Some("bytes 100-199/3000000"), Some("100"))
    );
}

#[test]
fn une_plage_impossible_rend_416() {
    for range in [
        "bytes=3000000-",
        "bytes=-0",
        "bytes=5-4",
        "bytes=0-1,5-6",
        "bytes=abc",
        "bytes=",
        "bytes= 1-2",
        "bytes=+1-2",
        "bytes=1--2",
    ] {
        assert_eq!(
            plage(Some(range), TROIS_MILLIONS),
            Demande::Impossible,
            "{range}"
        );
    }
    assert_eq!(plage(Some("bytes=1000-"), 1_000), Demande::Impossible);
    assert_eq!(plage(Some("bytes=0-"), 0), Demande::Impossible);
    assert_eq!(plage(Some("bytes=-5"), 0), Demande::Impossible);
    let s = servir("GET", Some("bytes=3000000-"), TROIS_MILLIONS, "audio/wav");
    assert_eq!(
        (s.statut, s.plage, entete(&s, "Content-Range")),
        (416, None, Some("bytes */3000000"))
    );
}

#[test]
fn une_autre_unite_vaut_une_requete_sans_plage() {
    assert_eq!(plage(Some("items=0-5"), 1_000), Demande::Entier);
    assert_eq!(plage(None, 1_000), Demande::Entier);
    assert_eq!(servir("GET", None, 0, "audio/wav").statut, 200);
}

#[test]
fn head_rend_les_en_tetes_de_get_sans_corps() {
    let (get, head) = (
        servir("GET", None, TROIS_MILLIONS, "audio/wav"),
        servir("HEAD", None, TROIS_MILLIONS, "audio/wav"),
    );
    assert_eq!(
        (head.statut, &head.entetes, head.plage),
        (get.statut, &get.entetes, None)
    );
}

#[test]
fn une_autre_methode_rend_405() {
    for methode in ["POST", "PUT", "DELETE", "get", "OPTIONS"] {
        let s = servir(methode, None, 1_000, "audio/wav");
        assert_eq!(
            (s.statut, entete(&s, "Allow"), s.plage),
            (405, Some("GET, HEAD"), None),
            "{methode}"
        );
    }
}

#[test]
fn chaque_reponse_porte_nosniff_et_accept_ranges() {
    for s in [
        servir("GET", None, 1_000, "audio/wav"),
        servir("GET", Some("bytes=-0"), 1_000, "audio/wav"),
        servir("POST", None, 1_000, "audio/wav"),
        introuvable(),
    ] {
        assert_eq!(
            (
                entete(&s, "X-Content-Type-Options"),
                entete(&s, "Accept-Ranges")
            ),
            (Some("nosniff"), Some("bytes")),
            "{}",
            s.statut
        );
    }
    assert_eq!((introuvable().statut, introuvable().plage), (404, None));
}
