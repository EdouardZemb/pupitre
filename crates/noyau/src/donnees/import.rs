//! Les opérations nommées de l'import (§ 30 · 9.2 ; § 31 · 2 ; questions 18 à 22, 25, 26 et 31) :
//! ouvrir, verser un fichier, verser une prise en stems, clore, déclarer une origine après
//! l'import, choisir le type d'un stem, et le nom d'une copie de l'import. Le noyau calcule les
//! changements sans rien écrire ; l'heure et le décalage local sont des arguments, jamais
//! l'horloge. Les clés d'une captation sont à plat, `<empreinte>/<clé>`, sous le conteneur de sa
//! sorte.

use std::collections::{BTreeMap, BTreeSet};

use crate::donnees::changement::{Changement, Operation, Simple};
use crate::donnees::demarrage::fichier_du_voisin;
use crate::donnees::entete::{FaitsMidi, FaitsWav};
use crate::donnees::fonds::{
    BITS, CANAUX, CODAGE, DECLARATION, DEPUIS, DIVISION, DUREE, FORMAT, FREQUENCE, GENEREE, IDEE,
    IMPORTEE, IMPORTS, LE, MAIN, MANIFESTE, MIDI, MIDI_VERSES, NOM, ORIGINES_DECLAREES, OUVERT,
    PAR, PAR_LE_NOM, PRISES, PROVENANCE, RAPPORT, SAISIE, STEM, TYPE,
};
use crate::donnees::gestes::{
    Appareil, Impossible, Sorte, date_du_message, naissance_de_l_idee, oter, poser, versee,
};
use crate::donnees::origine::{Ajout, Declaration, OriginesLues, ajout_d_origine};
use crate::donnees::signature::{FormatSonore, FormatTexte};
use crate::donnees::stems::TypeDeStem;

/// Les faits d'un fichier versé, tels que l'import les écrit ; un illisible n'en a aucun
/// (question 10), son format compris.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Faits {
    /// Son format, au vocabulaire du § 30 · 9.2, jamais l'extension.
    pub format: Option<String>,
    /// Sa durée, en millisecondes, d'un WAV lisible.
    pub duree_ms: Option<u64>,
    /// Le codage de l'en-tête d'un WAV.
    pub codage: Option<u16>,
    /// Les canaux d'un WAV.
    pub canaux: Option<u16>,
    /// La fréquence d'un WAV, en hertz.
    pub frequence: Option<u32>,
    /// Les bits d'un échantillon d'un WAV.
    pub bits: Option<u16>,
    /// La division d'un MIDI, brute.
    pub division: Option<u16>,
}

impl Faits {
    /// Les faits d'un WAV lisible : le format `wav`, sa durée et les quatre faits de son en-tête.
    pub fn du_wav(faits: &FaitsWav) -> Faits {
        Faits {
            format: Some(FormatSonore::Wav.nom().to_string()),
            duree_ms: Some(faits.duree_ms()),
            codage: Some(faits.codage),
            canaux: Some(faits.canaux),
            frequence: Some(faits.frequence),
            bits: Some(faits.bits),
            division: None,
        }
    }

    /// Les faits d'un MIDI lisible : son format, `smf0` à `smf2`, et sa division.
    pub fn du_midi(faits: &FaitsMidi) -> Faits {
        Faits {
            format: Some(faits.format_smf().to_string()),
            division: Some(faits.division),
            ..Faits::default()
        }
    }

    /// Les faits d'un audio que l'import ne décode pas, FLAC, MP3… : son format seul (ADR-024,
    /// décision 6).
    pub fn d_un_audio(format: FormatSonore) -> Faits {
        Faits {
            format: Some(format.nom().to_string()),
            ..Faits::default()
        }
    }

    /// Les faits d'un texte versé lisible : son format seul.
    pub fn d_un_texte(format: FormatTexte) -> Faits {
        Faits {
            format: Some(format.nom().to_string()),
            ..Faits::default()
        }
    }

    /// Ses faits entiers, chacun sous sa clé, dans l'ordre de la structure.
    fn entiers(&self) -> [(&'static str, Option<u64>); 6] {
        [
            (DUREE, self.duree_ms),
            (CODAGE, self.codage.map(u64::from)),
            (CANAUX, self.canaux.map(u64::from)),
            (FREQUENCE, self.frequence.map(u64::from)),
            (BITS, self.bits.map(u64::from)),
            (DIVISION, self.division.map(u64::from)),
        ]
    }
}

/// Ce qu'un import ouvert donne à chacune de ses opérations : son ULID, l'heure de son ouverture
/// (le jour que disent ses messages), l'appareil.
pub struct ImportOuvert<'a> {
    /// L'ULID de l'import.
    pub import: &'a str,
    /// L'heure de son ouverture, en secondes.
    pub ouvert_le: i64,
    /// L'appareil où il se fait.
    pub par: Appareil,
}

/// Ce que le fonds porte déjà d'un contenu : sa captation, ses noms (les chemins), ses origines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DejaLu {
    /// Vrai si le fonds porte déjà sa captation.
    pub captation: bool,
    /// Les chemins de ses noms d'origine, sous `<e>/nom/<chemin>`.
    pub noms: BTreeSet<String>,
    /// Ses origines déclarées.
    pub origines: OriginesLues,
}

/// Un nom de générateur se refuse vide, une fois ses blancs ôtés (`NomDeGenerateurVide`), ou
/// porteur d'un caractère de contrôle (`char::is_control`), un retour à la ligne compris
/// (`NomDeGenerateurIllisible`) ; il se range sinon tel qu'il est reçu, sans normalisation.
fn controler_l_ajout(ajout: &Ajout) -> Result<(), Impossible> {
    let Ajout::Generee(nom) = ajout else {
        return Ok(());
    };
    if nom.trim().is_empty() {
        return Err(Impossible::NomDeGenerateurVide);
    }
    if nom.chars().any(char::is_control) {
        return Err(Impossible::NomDeGenerateurIllisible);
    }
    Ok(())
}

/// Ouvrir l'import (question 22) : `imports/<i>/ouvert` à `heure`, et chaque déclaration sous
/// `imports/<i>/declaration/<chemin>` ; « Import du 3 novembre ouvert ». Refusé : un nom de
/// générateur vide, ou qui porte un caractère de contrôle.
pub fn ouvrir(
    import: &str,
    declarations: &BTreeMap<String, Declaration>,
    heure: i64,
    decalage_minutes: i32,
) -> Result<Operation, Impossible> {
    let mut changements = vec![poser(
        &[IMPORTS, &format!("{import}/{OUVERT}")],
        Simple::Entier(heure),
    )];
    for (chemin, declaration) in declarations {
        if let Declaration::Origine(ajout) = declaration {
            controler_l_ajout(ajout)?;
        }
        changements.push(poser(
            &[IMPORTS, &format!("{import}/{DECLARATION}/{chemin}")],
            Simple::Chaine(declaration.valeur()),
        ));
    }
    Ok(Operation {
        changements,
        message: format!(
            "Import du {} ouvert",
            date_du_message(heure, decalage_minutes)
        ),
        heure,
    })
}

/// `<base>/format` si `format` est présent, puis chaque fait entier présent, sous `<base>/<clé>`,
/// dans le conteneur.
fn poser_les_faits(
    conteneur: &str,
    base: &str,
    format: Option<&String>,
    entiers: &[(&str, Option<u64>)],
) -> Vec<Changement> {
    let format = format.map(|format| {
        poser(
            &[conteneur, &format!("{base}/{FORMAT}")],
            Simple::Chaine(format.clone()),
        )
    });
    let entiers = entiers.iter().filter_map(|(cle, valeur)| {
        valeur.map(|valeur| {
            poser(
                &[conteneur, &format!("{base}/{cle}")],
                Simple::Naturel(valeur),
            )
        })
    });
    format.into_iter().chain(entiers).collect()
}

/// `<base>/nom/<chemin>` à l'ULID de l'import, dans le conteneur, si `chemin` n'est pas déjà parmi
/// `noms` (question 20) ; aucun sans chemin.
fn nom_s_il_est_absent(
    conteneur: &str,
    base: &str,
    chemin: Option<&str>,
    noms: &BTreeSet<String>,
    import: &str,
) -> Option<Changement> {
    let chemin = chemin.filter(|chemin| !noms.contains(*chemin))?;
    Some(poser(
        &[conteneur, &format!("{base}/{NOM}/{chemin}")],
        Simple::Chaine(import.to_string()),
    ))
}

/// L'élément d'origine `ajout` du contenu `empreinte`, sous `origines_declarees` : `<e>/main`, ou
/// `<e>/generee/<nom>`, à `true`.
fn element_d_origine(empreinte: &str, ajout: &Ajout) -> Changement {
    let cle = match ajout {
        Ajout::Main => format!("{empreinte}/{MAIN}"),
        Ajout::Generee(nom) => format!("{empreinte}/{GENEREE}/{nom}"),
    };
    poser(&[ORIGINES_DECLAREES, &cle], Simple::Vrai)
}

/// L'élément que `ajout_d_origine` garde de `origine` pour le contenu `empreinte`, qui porte déjà
/// les origines `lues` ; aucun sans origine, ou si rien ne change.
fn origine_s_il_change(
    empreinte: &str,
    lues: &OriginesLues,
    origine: Option<&Ajout>,
) -> Option<Changement> {
    let ajout = ajout_d_origine(lues, origine?)?;
    Some(element_d_origine(empreinte, &ajout))
}

/// Le message d'un versement qui ne fait naître aucune idée : « Import du 3 novembre : » suivi de
/// chaque sorte d'ajout qui a des changements, dans cet ordre, séparées par une virgule : « nom
/// d'origine ajouté », « MIDI d'un stem ajouté », « origine déclarée » ; le jour celui de
/// l'ouverture de l'import. `None` si rien ne s'ajoute.
fn message_des_ajouts(
    ouvert: &ImportOuvert<'_>,
    noms: &[Changement],
    midis: &[Changement],
    origines: &[Changement],
    decalage_minutes: i32,
) -> Option<String> {
    let ajouts: Vec<&str> = [
        (noms, "nom d'origine ajouté"),
        (midis, "MIDI d'un stem ajouté"),
        (origines, "origine déclarée"),
    ]
    .into_iter()
    .filter(|(changements, _)| !changements.is_empty())
    .map(|(_, ajout)| ajout)
    .collect();
    if ajouts.is_empty() {
        return None;
    }
    Some(format!(
        "Import du {} : {}",
        date_du_message(ouvert.ouvert_le, decalage_minutes),
        ajouts.join(", ")
    ))
}

/// Verser un fichier : un contenu neuf (`!deja.captation`) fait naître son idée, `importee`, par
/// l'appareil, depuis l'import, à `heure`, et écrit sa captation sous le conteneur de sa sorte
/// (`<e>/idee`, puis chaque fait présent : `format` ; d'une prise, `duree`, `codage`, `canaux`,
/// `frequence`, `bits` ; d'un MIDI, `division`), « Idée du 3 novembre : prise versée » (comme
/// `gestes::verser`). Puis, neuf ou connu : `<e>/nom/<chemin>` à l'ULID de l'import s'il est
/// absent (question 20) ; l'élément d'origine que `ajout_d_origine` garde, sous
/// `origines_declarees`. Un contenu connu à qui rien ne s'ajoute : `Ok(None)` ; sinon « Import du
/// 3 novembre : nom d'origine ajouté », « … : origine déclarée », ou les deux, séparés par une
/// virgule, le jour celui de l'ouverture de l'import.
///
/// Il ne passe pas par `gestes::verser`, qui refuse un contenu connu et écrit toujours `format` :
/// il partage avec lui la naissance de l'idée, `gestes::naissance_de_l_idee`. Les faits sont ceux
/// que porte `faits`, que ses constructeurs donnent à chaque sorte ; aucun n'est refusé ici, la
/// durée d'un WAV étant déjà bornée par `entete::faits_wav`.
// Dix arguments : chaque donnée du versement, puis ce que le fonds porte déjà du contenu.
#[allow(clippy::too_many_arguments)]
pub fn verser_un_fichier(
    ouvert: &ImportOuvert<'_>,
    sorte: Sorte,
    empreinte: &str,
    faits: &Faits,
    chemin: Option<&str>,
    origine: Option<&Ajout>,
    deja: &DejaLu,
    idee: &str,
    heure: i64,
    decalage_minutes: i32,
) -> Result<Option<Operation>, Impossible> {
    let conteneur = sorte.conteneur();
    let nom = nom_s_il_est_absent(conteneur, empreinte, chemin, &deja.noms, ouvert.import);
    let declaree = origine_s_il_change(empreinte, &deja.origines, origine);
    let neuf = !deja.captation;
    let (mut changements, message) = if neuf {
        let mut changements =
            naissance_de_l_idee(idee, IMPORTEE, ouvert.par, Some(ouvert.import), heure);
        changements.push(poser(
            &[conteneur, &format!("{empreinte}/{IDEE}")],
            Simple::Chaine(idee.to_string()),
        ));
        changements.extend(poser_les_faits(
            conteneur,
            empreinte,
            faits.format.as_ref(),
            &faits.entiers(),
        ));
        let message = format!(
            "Idée du {} : {}",
            date_du_message(heure, decalage_minutes),
            versee(sorte)
        );
        (changements, message)
    } else {
        let Some(message) = message_des_ajouts(
            ouvert,
            nom.as_slice(),
            &[],
            declaree.as_slice(),
            decalage_minutes,
        ) else {
            return Ok(None);
        };
        (Vec::new(), message)
    };
    changements.extend(nom);
    changements.extend(declaree);
    Ok(Some(Operation {
        changements,
        message,
        heure,
    }))
}

/// Un stem à verser dans une prise en stems, et le MIDI qui l'accompagne.
pub struct StemAVerser<'a> {
    /// Son empreinte.
    pub empreinte: &'a str,
    /// Ses faits : seuls son format et sa durée s'écrivent.
    pub faits: &'a Faits,
    /// Son chemin, s'il en a un en UTF-8.
    pub chemin: Option<&'a str>,
    /// Le MIDI qui l'accompagne (question 17).
    pub midi: Option<MidiDuStem<'a>>,
}

/// Le MIDI d'un stem.
pub struct MidiDuStem<'a> {
    /// Son empreinte.
    pub empreinte: &'a str,
    /// Ses faits : seuls son format et sa division s'écrivent.
    pub faits: &'a Faits,
    /// Son chemin, s'il en a un en UTF-8.
    pub chemin: Option<&'a str>,
}

/// Ce que le fonds porte déjà d'une prise en stems : elle (son manifeste), ses noms, par
/// empreinte, ce que portent ses stems et leurs MIDI, et les liens d'un stem à son MIDI déjà posés
/// sous la prise.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DejaDeLaPrise {
    /// La prise, sous l'empreinte de son manifeste.
    pub prise: DejaLu,
    /// Ses stems, par empreinte : leurs noms sous `prises/<p>/stem/<s>/nom/…`.
    pub stems: BTreeMap<String, DejaLu>,
    /// Les MIDI de ses stems, par empreinte : leur entrée sous `midi_verses`, `captation` vrai
    /// s'ils l'ont déjà.
    pub midis: BTreeMap<String, DejaLu>,
    /// Les liens déjà posés sous la prise, `prises/<p>/stem/<s>/midi/<m>` : (le stem, le MIDI).
    pub liens_midi: BTreeSet<(String, String)>,
}

/// Verser une prise en stems (questions 19, 21), `manifeste` l'empreinte de son manifeste, `nom` le
/// chemin de son dossier, ou son dernier segment pour un jeu versé seul ; `None` pour un jeu versé
/// seul dont le dossier n'a pas de nom en UTF-8 : la prise n'a alors aucune clé `nom`, et ses stems
/// gardent les leurs (R23). Neuve : son idée naît ; `prises/<p>/idee`, `<p>/manifeste` à `true`,
/// `<p>/nom/<nom>` ; pour chaque stem, `<p>/stem/<s>/format` et `…/duree` s'ils sont présents,
/// `…/nom/<chemin>` ; « Idée du 3 novembre : prise en stems versée ». Neuve ou connue : pour chaque
/// MIDI d'un stem, `<p>/stem/<s>/midi/<m>` à `true` si ce lien n'est pas déjà posé
/// (`deja.liens_midi`), et, s'il n'a pas d'entrée sous `midi_verses`, `midi_verses/<m>/format` et
/// `…/division`, jamais `idee` : le manifeste ne compte que les stems audio, et un MIDI ajouté au
/// dossier d'une prise connue la rejoint (ADR-024, décision 1) ; chaque nom absent, de la prise,
/// des stems et des MIDI, et l'origine, sur la prise, chaque stem et chaque MIDI. Connue :
/// « Import du 3 novembre : » suivi de ce qui s'ajoute, « nom d'origine ajouté », « MIDI d'un stem
/// ajouté », « origine déclarée », séparés par une virgule ; `Ok(None)` si rien ne s'ajoute. Sans
/// stem : `Impossible::JeuSansAudio`. L'idée naît par `gestes::naissance_de_l_idee`.
// Neuf arguments : chaque donnée du versement, puis ce que le fonds porte déjà de la prise.
#[allow(clippy::too_many_arguments)]
pub fn verser_une_prise_en_stems(
    ouvert: &ImportOuvert<'_>,
    manifeste: &str,
    nom: Option<&str>,
    stems: &[StemAVerser<'_>],
    origine: Option<&Ajout>,
    deja: &DejaDeLaPrise,
    idee: &str,
    heure: i64,
    decalage_minutes: i32,
) -> Result<Option<Operation>, Impossible> {
    if stems.is_empty() {
        return Err(Impossible::JeuSansAudio);
    }
    let rien = DejaLu::default();
    let import = ouvert.import;
    let mut noms: Vec<Changement> = Vec::new();
    let mut midis: Vec<Changement> = Vec::new();
    let mut origines: Vec<Changement> = Vec::new();
    noms.extend(nom_s_il_est_absent(
        PRISES,
        manifeste,
        nom,
        &deja.prise.noms,
        import,
    ));
    origines.extend(origine_s_il_change(
        manifeste,
        &deja.prise.origines,
        origine,
    ));
    for stem in stems {
        let base = format!("{manifeste}/{STEM}/{}", stem.empreinte);
        let du_stem = deja.stems.get(stem.empreinte).unwrap_or(&rien);
        noms.extend(nom_s_il_est_absent(
            PRISES,
            &base,
            stem.chemin,
            &du_stem.noms,
            import,
        ));
        origines.extend(origine_s_il_change(
            stem.empreinte,
            &du_stem.origines,
            origine,
        ));
        let Some(midi) = &stem.midi else {
            continue;
        };
        let lien = (stem.empreinte.to_string(), midi.empreinte.to_string());
        if !deja.liens_midi.contains(&lien) {
            midis.push(poser(
                &[PRISES, &format!("{base}/{MIDI}/{}", midi.empreinte)],
                Simple::Vrai,
            ));
        }
        let du_midi = deja.midis.get(midi.empreinte).unwrap_or(&rien);
        if !du_midi.captation {
            midis.extend(poser_les_faits(
                MIDI_VERSES,
                midi.empreinte,
                midi.faits.format.as_ref(),
                &[(DIVISION, midi.faits.division.map(u64::from))],
            ));
        }
        noms.extend(nom_s_il_est_absent(
            MIDI_VERSES,
            midi.empreinte,
            midi.chemin,
            &du_midi.noms,
            import,
        ));
        origines.extend(origine_s_il_change(
            midi.empreinte,
            &du_midi.origines,
            origine,
        ));
    }
    let neuve = !deja.prise.captation;
    let (mut changements, message) = if neuve {
        let mut changements = naissance_de_l_idee(idee, IMPORTEE, ouvert.par, Some(import), heure);
        changements.push(poser(
            &[PRISES, &format!("{manifeste}/{IDEE}")],
            Simple::Chaine(idee.to_string()),
        ));
        changements.push(poser(
            &[PRISES, &format!("{manifeste}/{MANIFESTE}")],
            Simple::Vrai,
        ));
        for stem in stems {
            changements.extend(poser_les_faits(
                PRISES,
                &format!("{manifeste}/{STEM}/{}", stem.empreinte),
                stem.faits.format.as_ref(),
                &[(DUREE, stem.faits.duree_ms)],
            ));
        }
        let message = format!(
            "Idée du {} : prise en stems versée",
            date_du_message(heure, decalage_minutes)
        );
        (changements, message)
    } else {
        let Some(message) = message_des_ajouts(ouvert, &noms, &midis, &origines, decalage_minutes)
        else {
            return Ok(None);
        };
        (Vec::new(), message)
    };
    changements.extend(midis);
    changements.extend(noms);
    changements.extend(origines);
    Ok(Some(Operation {
        changements,
        message,
        heure,
    }))
}

/// Clore l'import : `imports/<i>/rapport` à l'empreinte du rapport ; « Import du 3 novembre
/// clos », le jour celui de son ouverture.
pub fn clore(
    ouvert: &ImportOuvert<'_>,
    rapport: &str,
    heure: i64,
    decalage_minutes: i32,
) -> Operation {
    Operation {
        changements: vec![poser(
            &[IMPORTS, &format!("{}/{RAPPORT}", ouvert.import)],
            Simple::Chaine(rapport.to_string()),
        )],
        message: format!(
            "Import du {} clos",
            date_du_message(ouvert.ouvert_le, decalage_minutes)
        ),
        heure,
    }
}

/// Une captation que vise une déclaration d'origine faite après l'import : son empreinte, ce
/// qu'elle porte d'origines, et ce qui la nomme dans un message (son premier nom vu, ou « cette
/// prise »).
pub struct Cible<'a> {
    /// Son empreinte.
    pub empreinte: &'a str,
    /// Ses origines déclarées.
    pub origines: &'a OriginesLues,
    /// Ce qui la nomme dans un message.
    pub nommee: &'a str,
}

/// Ce que rend une déclaration d'origine : l'opération, s'il y a un changement ; les cibles qui
/// portaient déjà l'élément déclaré ; et celles, générées, où « de la main de Sacha » ne change
/// rien (question 22), seules à valoir un avertissement (R16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaree {
    /// L'opération, s'il y a un changement.
    pub operation: Option<Operation>,
    /// Les cibles qui portaient déjà l'élément déclaré, hors des générées que compte
    /// `generees_gardees`.
    pub deja_portees: usize,
    /// Les cibles générées où « de la main de Sacha » ne change rien, qu'elles portent déjà `main`
    /// ou non : l'écran les montre générées (`la_plus_forte`).
    pub generees_gardees: usize,
}

/// Ce que dit le message d'une déclaration de l'origine `ajout`.
fn dit_l_origine(ajout: &Ajout) -> String {
    match ajout {
        Ajout::Main => "de la main de Sacha".to_string(),
        Ajout::Generee(nom) => format!("générée par {nom}"),
    }
}

/// Déclarer une origine après l'import (question 22), sur une captation ou sur les captations d'un
/// préfixe, `portee` ce qui la nomme dans le message (« « Démo 1.wav » », « cette prise »,
/// « « Démos/ » »). `None` (`aucune`) : `Impossible::OrigineNonRetiree`. Un nom vide, ou qui porte
/// un caractère de contrôle : refusé. Chaque élément que `ajout_d_origine` garde s'écrit ;
/// « Origine déclarée pour <portee> : de la main de Sacha », ou « … : générée par <nom> ». Une
/// cible générée que « de la main de Sacha » ne change pas compte parmi `generees_gardees`, même
/// si elle porte déjà `main` (question 22 ; R16) ; une autre cible qui porte déjà l'élément, parmi
/// `deja_portees`.
pub fn declarer_une_origine(
    cibles: &[Cible<'_>],
    ajout: Option<&Ajout>,
    portee: &str,
    heure: i64,
) -> Result<Declaree, Impossible> {
    let Some(ajout) = ajout else {
        return Err(Impossible::OrigineNonRetiree);
    };
    controler_l_ajout(ajout)?;
    let mut changements = Vec::new();
    let mut deja_portees = 0;
    let mut generees_gardees = 0;
    for cible in cibles {
        if let Some(garde) = ajout_d_origine(cible.origines, ajout) {
            changements.push(element_d_origine(cible.empreinte, &garde));
        } else if *ajout == Ajout::Main && !cible.origines.generees.is_empty() {
            generees_gardees += 1;
        } else {
            deja_portees += 1;
        }
    }
    let operation = (!changements.is_empty()).then(|| Operation {
        changements,
        message: format!("Origine déclarée pour {portee} : {}", dit_l_origine(ajout)),
        heure,
    });
    Ok(Declaree {
        operation,
        deja_portees,
        generees_gardees,
    })
}

/// Ce que Sacha fait du type d'un stem : garder la proposition, ou choisir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChoixDuType {
    /// Garder le type proposé d'après le nom du stem.
    Garder,
    /// Choisir ce type.
    Choisir(TypeDeStem),
}

/// Choisir le type d'un stem (question 18) : `prises/<p>/stem/<s>/type` à `nom()` ; gardé,
/// `…/type/provenance` à `importee`, `…/type/par` à `nom`, `…/type/depuis` à `<s>` ; choisi,
/// `…/type/provenance` à `saisie`, `…/type/par` et `…/type/depuis` ôtés s'ils existaient
/// (§ 30 · 8.1 : `saisie` n'a ni `par` ni `depuis` ; R5) ; `…/type/le` à `heure`. « Type de
/// « voix.wav » : voix », ou « Type de ce stem : voix ». `Impossible::StemAbsent` si la prise ne
/// porte pas ce stem (`existe`).
pub fn choisir_le_type(
    prise: &str,
    stem: &str,
    existe: bool,
    propose: TypeDeStem,
    choix: ChoixDuType,
    nomme: Option<&str>,
    heure: i64,
) -> Result<Operation, Impossible> {
    if !existe {
        return Err(Impossible::StemAbsent);
    }
    let base = format!("{prise}/{STEM}/{stem}/{TYPE}");
    let sous_le_type = |cle: &str| format!("{base}/{cle}");
    let (type_du_stem, provenance) = match choix {
        ChoixDuType::Garder => (
            propose,
            vec![
                poser(
                    &[PRISES, &sous_le_type(PROVENANCE)],
                    Simple::Chaine(IMPORTEE.to_string()),
                ),
                poser(
                    &[PRISES, &sous_le_type(PAR)],
                    Simple::Chaine(PAR_LE_NOM.to_string()),
                ),
                poser(
                    &[PRISES, &sous_le_type(DEPUIS)],
                    Simple::Chaine(stem.to_string()),
                ),
            ],
        ),
        ChoixDuType::Choisir(choisi) => (
            choisi,
            vec![
                poser(
                    &[PRISES, &sous_le_type(PROVENANCE)],
                    Simple::Chaine(SAISIE.to_string()),
                ),
                oter(&[PRISES, &sous_le_type(PAR)]),
                oter(&[PRISES, &sous_le_type(DEPUIS)]),
            ],
        ),
    };
    let mut changements = vec![poser(
        &[PRISES, &base],
        Simple::Chaine(type_du_stem.nom().to_string()),
    )];
    changements.extend(provenance);
    changements.push(poser(&[PRISES, &sous_le_type(LE)], Simple::Entier(heure)));
    let message = match nomme {
        Some(nom) => format!("Type de « {nom} » : {}", type_du_stem.libelle()),
        None => format!("Type de ce stem : {}", type_du_stem.libelle()),
    };
    Ok(Operation {
        changements,
        message,
        heure,
    })
}

/// Le préfixe du nom d'une copie en flux de l'import.
const PREFIXE_D_UNE_COPIE: &str = "import-";

/// Le nom d'une copie en flux de l'import (question 31) : `import-<ULID>-<rang>`, que `voisin`
/// complète de `.<processus>.partiel`.
pub fn nom_de_la_copie(import: &str, rang: u64) -> String {
    format!("{PREFIXE_D_UNE_COPIE}{import}-{rang}")
}

/// Un caractère de l'alphabet de Crockford d'un ULID, en majuscule : un chiffre, ou une lettre
/// sauf `I`, `L`, `O` et `U`.
fn de_crockford(octet: u8) -> bool {
    octet.is_ascii_digit() || (octet.is_ascii_uppercase() && !b"ILOU".contains(&octet))
}

/// L'import et le rang d'un nom provisoire de `captations/` que `nom_de_la_copie` a formé
/// (`import-<26 caractères de Crockford>-<chiffres>.<processus>[-<n>].partiel`) ; `None` sinon,
/// le nom provisoire d'une capture compris. Il ne reconnaît que les noms que `nom_de_la_copie` et
/// `voisin` forment : le voisin se lit par `demarrage::fichier_du_voisin`, puis le nom de la copie.
pub fn copie_de(nom: &str) -> Option<(&str, u64)> {
    let copie = fichier_du_voisin(nom)?.strip_prefix(PREFIXE_D_UNE_COPIE)?;
    let import = copie.get(..26)?;
    if !import.bytes().all(de_crockford) {
        return None;
    }
    let rang = copie.get(26..)?.strip_prefix('-')?;
    if !rang.bytes().all(|octet| octet.is_ascii_digit()) {
        return None;
    }
    Some((import, rang.parse().ok()?))
}
