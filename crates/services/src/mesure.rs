//! Le fonds de mesure, le test 13 du § 31 · 10, entier (question 28) : engendré par les seules
//! opérations nommées de `Services`, celles de l'import comprises, sur des sources à verser en
//! mémoire (`SourceEnMemoire`), sous un aléa à lui, déterministe, à partir d'une heure reçue.
//! Restent absents la date, l'appareil et le diapason mesuré de chaque prise, que l'import du
//! lot 1 n'écrit pas (questions 28, 29). Les acteurs, un neuf toutes les sept opérations, viennent
//! des documents en mémoire (`changer_d_acteur_tous_les`, question 29), non du générateur.

use std::ops::Range;

use automerge::{Automerge, ChangeHash};
use commandes::{
    Captation, CloreLImport, Completer, Declaration, DeclarationDUnDossier, DeclarerUneOrigine,
    Etiqueter, OrigineADeclarer, OuvrirLImport, Portee, Regrouper, Relier, VerserUnFichier,
    VerserUnePriseEnStems,
};
use noyau::donnees::gestes;
use noyau::donnees::identifiants::empreinte;
use noyau::donnees::rapport;

use crate::source_en_memoire::SourceEnMemoire;
use crate::{Alea, Captations, Contexte, Documents, Raison, Refus, Services, lecture};

/// Les groupes du fonds de mesure, chacun de trois prises : de chacun reste une idée visible.
const GROUPES: usize = 2_000;

/// Les groupes d'un import, au plus : les 2 000 groupes viennent de vingt imports.
const GROUPES_PAR_IMPORT: usize = 100;

/// La part des groupes, en pour cent, dont la première prise est une prise en stems : les
/// premiers groupes du fonds, un au moins.
const PRISES_EN_STEMS_POUR_CENT: usize = 5;

/// La part des groupes, en pour mille, dont la troisième prise reçoit une origine déclarée après
/// l'import : les premiers groupes du fonds, un au moins.
const DECLARATIONS_POUR_MILLE: usize = 25;

/// Les stems d'une prise en stems, chacun un WAV qu'accompagne son MIDI.
const STEMS_PAR_JEU: usize = 12;

/// Les générateurs des imports déclarés générés, l'un après l'autre : des noms inventés.
const GENERATEURS: [&str; 3] = ["Brumelo", "Tissarde", "Cordelune"];

/// Le second nom inventé, sous lequel des captations générées se déclarent après l'import.
const SECOND_GENERATEUR: &str = "Vellanor";

/// Chaque chemin d'un fichier versé fait ces octets, son extension comprise : 80 en moyenne.
const OCTETS_D_UN_CHEMIN: usize = 80;

/// L'extension d'un WAV ou d'un MIDI, son point compris : `.wav`, `.mid`.
const OCTETS_D_UNE_EXTENSION: usize = 4;

/// Les lettres que tire un chemin après son début.
const LETTRES: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";

/// La fréquence de chaque WAV, en hertz : ses seize trames d'un octet, mono, durent 2 ms.
const FREQUENCE_D_UN_WAV: u32 = 8_000;

/// Les octets de `data` de chaque WAV, après son en-tête de 44 octets, tirés de l'aléa.
const DONNEES_D_UN_WAV: u32 = 16;

/// Les octets de chaque MIDI après son en-tête de 14 octets, tirés de l'aléa.
const OCTETS_D_UN_MIDI_APRES_SON_EN_TETE: usize = 16;

/// La place libre que reçoit chaque import, à son inspection et à son ouverture : sans limite.
const PLACE_LIBRE: u64 = u64::MAX;

/// Les textes que reçoit chaque idée visible, complétés l'un après l'autre.
const TEXTES_PAR_IDEE: usize = 3;

/// Les étiquettes de chaque idée visible, distinctes.
const ETIQUETTES_PAR_IDEE: usize = 3;

/// Un texte gagne un mot tant qu'il fait moins de ces octets ; le plus long mot en faisant neuf,
/// chaque texte en fait de 200 à 209.
const OCTETS_D_UN_TEXTE: usize = 200;

/// Les syllabes des mots inventés des textes et des étiquettes.
const SYLLABES: [&str; 24] = [
    "la", "mi", "ré", "do", "sol", "fa", "si", "ta", "po", "nu", "ver", "bri", "cla", "mon", "ton",
    "pen", "lu", "sa", "gra", "fle", "cor", "dan", "vo", "ri",
];

/// La graine de l'aléa du générateur, fixe : le fonds de mesure est le même à chaque passage. Le
/// test 13 sème de `GRAINE + 1` les acteurs des documents en mémoire.
pub const GRAINE: u64 = 13;

/// Ce qu'a engendré le générateur : les idées visibles et les idées retirées, telles que le fonds
/// les lit à la fin, et les opérations nommées faites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Engendre {
    /// Les idées non retirées.
    pub idees_visibles: usize,
    /// Les idées retirées.
    pub idees_retirees: usize,
    /// Les opérations nommées faites : chaque commande qui change les têtes du fonds en compte
    /// une ; la copie d'un stem ou de son MIDI, qui ne l'écrit pas, aucune.
    pub operations: usize,
}

/// Le test 13 entier (question 28), par les opérations de l'import : vingt imports, chacun d'une
/// source en mémoire de 300 idées à faire naître (100 groupes de trois prises), dont, aux groupes
/// 1 à 100 du fonds, ceux du premier import, une prise en stems de douze WAV et de douze MIDI au
/// lieu de la première prise ; chaque WAV d'un en-tête canonique de 44 octets (PCM, 8 000 Hz, mono,
/// 8 bits, `data` de 16 octets) suivi de 16 octets de données tirés de l'aléa, sa durée lue de
/// l'en-tête ; chaque MIDI de 14 octets d'en-tête valides (format 1, pistes et division tirées de
/// l'aléa) suivis de 16 octets d'aléa ; chaque chemin de 80 octets en moyenne. Les imports 1 à 10
/// déclarent leur dossier `{ stems: false, origine: Generee }`, l'un de trois noms inventés, les
/// imports 11 à 20 `{ stems: false, origine: Main }` ; les 100 jeux sont déclarés
/// `{ stems: true, origine: Aucune }`, leur origine leur venant du dossier du dessus
/// (`declaration_du_fichier`) ; l'ouverture reste une opération. Après eux, cinquante captations
/// générées déclarées sous un second nom ; puis les gestes de la tranche 2 : 4 000 regroupements,
/// trois textes et trois étiquettes par idée visible, 1 000 liens (balayage Q3, Q10).
///
/// Le compte : 20 ouvertures, 20 clôtures, 5 900 versements de prises, 100 prises en stems,
/// 50 déclarations d'origine, 4 000 regroupements, 6 000 textes, 6 000 étiquettes et 1 000 liens,
/// soit 23 090 opérations. Les copies des 1 200 stems et de leurs 1 200 MIDI n'en font aucune ;
/// leurs entrées, `prises/<p>/stem/<s>` et `midi_verses/<m>`, pèsent pourtant dans le fonds. La
/// date, l'appareil et le diapason mesuré de chaque prise restent absents : l'import du lot 1 ne
/// les écrit pas. `engendrer_le_fonds_de_mesure_selon`, sur 2 000 groupes.
pub fn engendrer_le_fonds_de_mesure<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    heure: i64,
) -> Result<Engendre, Refus> {
    engendrer_le_fonds_de_mesure_selon(services, heure, GROUPES)
}

/// Le même sur `groupes` groupes (balayage Q10) : les groupes répartis en imports de 100 au plus,
/// la première moitié des imports, arrondie au-dessus, déclarée générée, les autres `main` ; les
/// prises en stems aux 5 premiers pour cent des groupes, au moins une ; les déclarations après
/// l'import à 2,5 % des groupes, au moins une, chacune sur la troisième prise de l'un des premiers
/// groupes, qui sont ceux des imports générés. Pour 2 groupes : un import, une prise en stems, une
/// déclaration, `Engendre { idees_visibles: 2, idees_retirees: 4, operations: 26 }` (l'ouverture,
/// cinq versements de prises, la prise en stems, la clôture, la déclaration, 4 regroupements,
/// 6 textes, 6 étiquettes, un lien ; les copies des stems et de leurs MIDI ne font aucune
/// opération).
///
/// Chaque prise naît sous son idée, à son versement ; après les déclarations, chaque groupe à son
/// tour (`trier_un_groupe`) ; enfin l'idée visible 2k reliée à l'idée 2k + 1. Sans fonds, la
/// première ouverture le fait naître. L'aléa est le sien (`AleaDuGenerateur`, de `GRAINE`), d'où
/// se tirent aussi les identifiants que forment les services ; l'heure avance d'une seconde par
/// commande, depuis `heure`, au décalage local nul. Le premier refus d'une commande arrête le
/// générateur, et se rend tel quel. Rend les idées visibles et retirées que lit le fonds à la
/// fin, et les opérations faites.
pub fn engendrer_le_fonds_de_mesure_selon<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    heure: i64,
    groupes: usize,
) -> Result<Engendre, Refus> {
    let mut generateur = Generateur {
        alea: AleaDuGenerateur::de(GRAINE),
        heure,
        operations: 0,
    };
    let imports = groupes.div_ceil(GROUPES_PAR_IMPORT);
    let generes = imports.div_ceil(2);
    let en_stems = (groupes * PRISES_EN_STEMS_POUR_CENT / 100).max(1);
    let declarations = (groupes * DECLARATIONS_POUR_MILLE / 1_000).max(1);
    let mut verses = Vec::with_capacity(groupes);
    for rang in 0..imports {
        let origine = if rang < generes {
            OrigineADeclarer::Generee {
                nom: GENERATEURS[rang % GENERATEURS.len()].to_string(),
            }
        } else {
            OrigineADeclarer::Main
        };
        let debut = rang * GROUPES_PAR_IMPORT;
        let siens = debut..groupes.min(debut + GROUPES_PAR_IMPORT);
        verses.extend(importer(
            services,
            &mut generateur,
            rang,
            siens,
            en_stems,
            origine,
        )?);
    }
    for groupe in verses.iter().take(declarations) {
        let declaration = DeclarerUneOrigine {
            portee: Portee::Captation(prise(&groupe.autres[1])),
            origine: OrigineADeclarer::Generee {
                nom: SECOND_GENERATEUR.to_string(),
            },
        };
        generateur.conduire(services, |services, contexte| {
            services.declarer_une_origine(&declaration, contexte)
        })?;
    }
    let mut visibles = Vec::with_capacity(verses.len());
    for groupe in &verses {
        visibles.push(trier_un_groupe(services, &mut generateur, groupe)?);
    }
    let (paires, _) = visibles.as_chunks::<2>();
    for [a, b] in paires {
        let lien = Relier {
            a: a.clone(),
            b: b.clone(),
            decalage_minutes: 0,
            decalage_b_minutes: 0,
        };
        generateur.conduire(services, |services, contexte| {
            services.relier(&lien, contexte)
        })?;
    }
    let idees = services
        .documents()
        .fonds()
        .map(lecture::idees)
        .unwrap_or_default();
    let idees_retirees = idees.iter().filter(|idee| idee.retiree.is_some()).count();
    Ok(Engendre {
        idees_visibles: idees.len() - idees_retirees,
        idees_retirees,
        operations: generateur.operations,
    })
}

/// Un groupe versé : l'empreinte de sa première prise, ou de sa prise en stems (celle de son
/// manifeste, `rapport::manifeste` des empreintes de ses stems), et celles de ses deux autres
/// prises.
struct GroupeVerse {
    premiere: String,
    autres: [String; 2],
}

/// L'import n° `rang` + 1, ses groupes `groupes`, rangs du fonds dès 0 : sa source en mémoire,
/// « Séance <n> », porte chaque groupe dans son dossier, `groupe-<n>` ; la première prise d'un
/// groupe de rang inférieur à `en_stems` est un jeu de stems, `groupe-<n>/stems`, de douze WAV
/// `stem-<s>-…`, chacun avec son MIDI, du même nom à l'extension près ; les autres prises sont des
/// WAV `prise-<p>-…`. Le dossier versé déclaré `{ stems: false, origine }`, chaque jeu
/// `{ stems: true, origine: Aucune }`. L'import se conduit comme l'interface le conduit :
/// l'inspection, l'ouverture, chaque fichier hors des jeux, chaque jeu (ses fichiers, puis sa
/// prise en stems), puis la clôture, la place libre sans limite. Rend chaque groupe versé.
fn importer<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    generateur: &mut Generateur,
    rang: usize,
    groupes: Range<usize>,
    en_stems: usize,
    origine: OrigineADeclarer,
) -> Result<Vec<GroupeVerse>, Refus> {
    let mut fichiers = Vec::new();
    let mut declarations = vec![DeclarationDUnDossier {
        chemin: String::new(),
        declaration: Declaration {
            stems: false,
            origine,
        },
    }];
    let mut verses = Vec::with_capacity(groupes.len());
    for groupe in groupes {
        let dossier = format!("groupe-{:04}", groupe + 1);
        let premiere = if groupe < en_stems {
            let jeu = format!("{dossier}/stems");
            let mut stems = Vec::with_capacity(STEMS_PAR_JEU);
            for stem in 1..=STEMS_PAR_JEU {
                let radical = generateur.radical(&format!("{jeu}/stem-{stem:02}-"));
                let wav = generateur.wav();
                stems.push(empreinte(&wav));
                fichiers.push((Some(format!("{radical}.wav")), wav));
                fichiers.push((Some(format!("{radical}.mid")), generateur.midi()));
            }
            declarations.push(DeclarationDUnDossier {
                chemin: jeu,
                declaration: Declaration {
                    stems: true,
                    origine: OrigineADeclarer::Aucune,
                },
            });
            let stems: Vec<&str> = stems.iter().map(String::as_str).collect();
            empreinte(&rapport::manifeste(&stems))
        } else {
            generateur.prise(&dossier, 1, &mut fichiers)
        };
        let autres = [
            generateur.prise(&dossier, 2, &mut fichiers),
            generateur.prise(&dossier, 3, &mut fichiers),
        ];
        verses.push(GroupeVerse { premiere, autres });
    }
    let nom = format!("Séance {:02}", rang + 1);
    let source = SourceEnMemoire::new(Some(nom.as_str()), fichiers, Vec::new());
    services.inspecter(&source, PLACE_LIBRE)?;
    let ouverture = OuvrirLImport {
        declarations,
        decalage_minutes: 0,
    };
    let ouvert = generateur.conduire(services, |services, contexte| {
        services.ouvrir_l_import(&source, &ouverture, PLACE_LIBRE, contexte)
    })?;
    let verser = |services: &mut Services<D, C>, generateur: &mut Generateur, rang: u64| {
        let versement = VerserUnFichier {
            import: ouvert.import.clone(),
            rang,
            decalage_minutes: 0,
        };
        generateur.conduire(services, |services, contexte| {
            services.verser_un_fichier(&source, &versement, contexte)
        })
    };
    for rang in &ouvert.rangs {
        verser(services, generateur, *rang)?;
    }
    for jeu in &ouvert.jeux {
        for rang in &jeu.rangs {
            verser(services, generateur, *rang)?;
        }
        let prise_en_stems = VerserUnePriseEnStems {
            import: ouvert.import.clone(),
            dossier: jeu.dossier.clone(),
            decalage_minutes: 0,
        };
        generateur.conduire(services, |services, contexte| {
            services.verser_une_prise_en_stems(&source, &prise_en_stems, contexte)
        })?;
    }
    let cloture = CloreLImport {
        import: ouvert.import.clone(),
        decalage_minutes: 0,
    };
    generateur.conduire(services, |services, contexte| {
        services.clore_l_import(&source, &cloture, contexte)
    })?;
    Ok(verses)
}

/// Les gestes de tri d'un groupe : la deuxième et la troisième prise regroupées dans l'idée de la
/// première, que le fonds lit (`lecture::lire_la_captation`), ce qui retire les leurs ; puis trois
/// textes complétés dans cette idée, l'un après l'autre, et trois étiquettes posées. Rend
/// l'identifiant de l'idée qui reste visible.
fn trier_un_groupe<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    generateur: &mut Generateur,
    groupe: &GroupeVerse,
) -> Result<String, Refus> {
    let idee = services
        .documents()
        .fonds()
        .and_then(|fonds| lecture::lire_la_captation(fonds, gestes::Sorte::Prise, &groupe.premiere))
        .map(|captation| captation.idee)
        .ok_or_else(|| Refus {
            raison: Raison::Impossible,
            message: "La première prise d'un groupe n'est pas dans le fonds.".to_string(),
        })?;
    for autre in &groupe.autres {
        let regroupement = Regrouper {
            captation: prise(autre),
            cible: idee.clone(),
            decalage_minutes: 0,
            decalage_source_minutes: 0,
        };
        generateur.conduire(services, |services, contexte| {
            services.regrouper(&regroupement, contexte)
        })?;
    }
    for _ in 0..TEXTES_PAR_IDEE {
        let completion = Completer {
            idee: idee.clone(),
            texte: generateur.texte(),
            decalage_minutes: 0,
        };
        generateur.conduire(services, |services, contexte| {
            services.completer(&completion, contexte)
        })?;
    }
    for etiquette in generateur.etiquettes() {
        let etiquetage = Etiqueter {
            idee: idee.clone(),
            etiquette,
            decalage_minutes: 0,
        };
        generateur.conduire(services, |services, contexte| {
            services.poser_une_etiquette(&etiquetage, contexte)
        })?;
    }
    Ok(idee)
}

/// La prise d'empreinte `empreinte`, telle qu'une commande la nomme.
fn prise(empreinte: &str) -> Captation {
    Captation {
        sorte: commandes::Sorte::Prise,
        empreinte: empreinte.to_string(),
    }
}

/// Les têtes du fonds des services ; sans fonds, aucune.
fn tetes<D: Documents, C: Captations>(services: &Services<D, C>) -> Option<Vec<ChangeHash>> {
    services.documents().fonds().map(Automerge::get_heads)
}

/// Ce que le générateur tient d'une commande à l'autre : son aléa, l'heure de la commande
/// suivante, en secondes, et les opérations déjà faites.
struct Generateur {
    alea: AleaDuGenerateur,
    heure: i64,
    operations: usize,
}

impl Generateur {
    /// Conduit `commande` sur `services`, dans le contexte de la commande suivante : son heure, et
    /// l'aléa du générateur ; l'heure avance d'une seconde. La commande compte parmi les
    /// opérations si elle change les têtes du fonds ; refusée, son refus se rend tel quel.
    fn conduire<D: Documents, C: Captations, T>(
        &mut self,
        services: &mut Services<D, C>,
        commande: impl FnOnce(&mut Services<D, C>, &mut Contexte<'_>) -> Result<T, Refus>,
    ) -> Result<T, Refus> {
        let avant = tetes(services);
        let mut contexte = Contexte {
            heure: self.heure,
            alea: &mut self.alea,
        };
        self.heure += 1;
        let rendu = commande(services, &mut contexte)?;
        if tetes(services) != avant {
            self.operations += 1;
        }
        Ok(rendu)
    }

    /// Une prise versée seule, la `n`-ième d'un groupe : un WAV (`wav`), sous un chemin de
    /// `dossier` qui commence par `prise-<n>-` (`radical`), ajouté à `fichiers`. Rend son
    /// empreinte.
    fn prise(
        &mut self,
        dossier: &str,
        n: usize,
        fichiers: &mut Vec<(Option<String>, Vec<u8>)>,
    ) -> String {
        let radical = self.radical(&format!("{dossier}/prise-{n}-"));
        let wav = self.wav();
        let calculee = empreinte(&wav);
        fichiers.push((Some(format!("{radical}.wav")), wav));
        calculee
    }

    /// Un chemin sans son extension : `debut`, puis des lettres de `a` à `z` tirées de l'aléa, tant
    /// qu'avec son extension (`.wav` ou `.mid`) il fait moins de `OCTETS_D_UN_CHEMIN` octets. Le
    /// chemin entier en fait donc 80, `debut` étant plus court.
    fn radical(&mut self, debut: &str) -> String {
        let mut radical = debut.to_string();
        while radical.len() + OCTETS_D_UNE_EXTENSION < OCTETS_D_UN_CHEMIN {
            radical.push(char::from(LETTRES[self.alea.rang(LETTRES.len())]));
        }
        radical
    }

    /// Un WAV : son en-tête canonique de 44 octets, `RIFF` (sa taille, 36 + 16), `WAVE`, `fmt ` de
    /// 16 octets (PCM, mono, 8 000 Hz, 8 000 octets par seconde, un octet par trame, 8 bits), puis
    /// `data` de 16 octets ; puis ses 16 octets de données, tirés de l'aléa. Sa durée, 2 ms, se lit
    /// de l'en-tête.
    fn wav(&mut self) -> Vec<u8> {
        let mut octets = Vec::with_capacity(44 + DONNEES_D_UN_WAV as usize);
        octets.extend_from_slice(b"RIFF");
        octets.extend_from_slice(&(36 + DONNEES_D_UN_WAV).to_le_bytes());
        octets.extend_from_slice(b"WAVEfmt ");
        octets.extend_from_slice(&16u32.to_le_bytes());
        // Le codage (PCM), puis les canaux (mono).
        octets.extend_from_slice(&1u16.to_le_bytes());
        octets.extend_from_slice(&1u16.to_le_bytes());
        // La fréquence, puis les octets par seconde : mono, un octet par trame.
        octets.extend_from_slice(&FREQUENCE_D_UN_WAV.to_le_bytes());
        octets.extend_from_slice(&FREQUENCE_D_UN_WAV.to_le_bytes());
        // L'alignement d'une trame, puis les bits.
        octets.extend_from_slice(&1u16.to_le_bytes());
        octets.extend_from_slice(&8u16.to_le_bytes());
        octets.extend_from_slice(b"data");
        octets.extend_from_slice(&DONNEES_D_UN_WAV.to_le_bytes());
        let mut donnees = [0u8; DONNEES_D_UN_WAV as usize];
        self.alea.remplir(&mut donnees);
        octets.extend_from_slice(&donnees);
        octets
    }

    /// Un MIDI : son en-tête de 14 octets, `MThd`, sa longueur (6), le format 1, de 1 à 16 pistes
    /// et une division de 24 à 960 (24 fois un rang de 1 à 40), tirées de l'aléa, gros-boutistes ;
    /// puis 16 octets tirés de l'aléa.
    fn midi(&mut self) -> Vec<u8> {
        let pistes = 1 + self.alea.rang(16) as u16;
        let division = 24 * (1 + self.alea.rang(40) as u16);
        let mut octets = b"MThd\x00\x00\x00\x06".to_vec();
        octets.extend_from_slice(&1u16.to_be_bytes());
        octets.extend_from_slice(&pistes.to_be_bytes());
        octets.extend_from_slice(&division.to_be_bytes());
        let mut suite = [0u8; OCTETS_D_UN_MIDI_APRES_SON_EN_TETE];
        self.alea.remplir(&mut suite);
        octets.extend_from_slice(&suite);
        octets
    }

    /// Un mot inventé de `syllabes` syllabes, chacune tirée de `SYLLABES`.
    fn mot(&mut self, syllabes: usize) -> String {
        (0..syllabes)
            .map(|_| SYLLABES[self.alea.rang(SYLLABES.len())])
            .collect()
    }

    /// Un texte : des mots de deux ou trois syllabes, le nombre tiré avant chaque mot, séparés
    /// d'une espace, ajoutés tant que le texte fait moins de 200 octets.
    fn texte(&mut self) -> String {
        let mut texte = String::new();
        while texte.len() < OCTETS_D_UN_TEXTE {
            if !texte.is_empty() {
                texte.push(' ');
            }
            let syllabes = 2 + self.alea.rang(2);
            texte.push_str(&self.mot(syllabes));
        }
        texte
    }

    /// Trois étiquettes distinctes, de deux syllabes chacune, dans l'ordre de leur tirage : une
    /// étiquette déjà tirée se tire de nouveau.
    fn etiquettes(&mut self) -> Vec<String> {
        let mut etiquettes = Vec::with_capacity(ETIQUETTES_PAR_IDEE);
        while etiquettes.len() < ETIQUETTES_PAR_IDEE {
            let etiquette = self.mot(2);
            if !etiquettes.contains(&etiquette) {
                etiquettes.push(etiquette);
            }
        }
        etiquettes
    }
}

/// L'aléa du générateur, déterministe : splitmix64, de la graine `etat` ; le même tirage à chaque
/// passage. Public, avec son `impl Alea`, pour que les documents en mémoire du test 13 en tirent
/// leurs acteurs, d'une autre graine : aucun second générateur (R18).
pub struct AleaDuGenerateur {
    etat: u64,
}

impl AleaDuGenerateur {
    /// L'aléa semé de `graine`.
    pub fn de(graine: u64) -> Self {
        AleaDuGenerateur { etat: graine }
    }

    /// Le tirage suivant, sur 64 bits.
    fn suivant(&mut self) -> u64 {
        self.etat = self.etat.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.etat;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Un rang de 0 à `n` − 1 : le reste du tirage suivant, divisé par `n`.
    fn rang(&mut self, n: usize) -> usize {
        (self.suivant() % n as u64) as usize
    }
}

impl Alea for AleaDuGenerateur {
    /// `octets`, huit par tirage, dans l'ordre petit-boutiste ; le dernier tirage, tronqué.
    fn remplir(&mut self, octets: &mut [u8]) {
        for morceau in octets.chunks_mut(8) {
            let tirage = self.suivant().to_le_bytes();
            morceau.copy_from_slice(&tirage[..morceau.len()]);
        }
    }
}
