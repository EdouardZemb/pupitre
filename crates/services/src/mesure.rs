//! Le fonds de mesure (§ 31 · 10, test 13), dans la part qu'en écrit la tranche 2 : engendré par
//! les seules opérations nommées de `Services`, sous un aléa à lui, déterministe, à partir d'une
//! heure reçue. La tranche 5 le complète : les stems et leurs MIDI, les noms d'origine, les
//! origines déclarées, un acteur par ouverture.

use commandes::{Captation, Completer, Etiqueter, Regrouper, Relier};
use noyau::donnees::gestes;
use noyau::donnees::identifiants::ulid;

use crate::{Alea, Captations, Contexte, Documents, Refus, Services, lecture};

/// Les groupes du fonds de mesure : de chacun reste une idée visible.
const GROUPES: usize = 2_000;

/// Les prises d'un groupe, versées par le même import.
const PRISES_PAR_GROUPE: usize = 3;

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

/// Les octets d'une prise, tirés de l'aléa.
const OCTETS_D_UNE_PRISE: usize = 16;

/// La plus courte durée d'une prise, en millisecondes.
const DUREE_LA_PLUS_COURTE_MS: u64 = 5_000;

/// La plus longue durée d'une prise, en millisecondes.
const DUREE_LA_PLUS_LONGUE_MS: u64 = 600_000;

/// La graine de l'aléa, fixe : le fonds de mesure est le même à chaque passage.
const GRAINE: u64 = 13;

/// Ce qu'a engendré le générateur : les idées visibles et les idées retirées, telles que le fonds
/// les lit à la fin, et les opérations nommées faites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Engendre {
    /// Les idées non retirées.
    pub idees_visibles: usize,
    /// Les idées retirées.
    pub idees_retirees: usize,
    /// Les opérations nommées faites, une par commande.
    pub operations: usize,
}

/// Le fonds de mesure du test 13 (§ 31 · 10), dans la part qu'en écrit la tranche 2, par les
/// seules opérations de `services` : 6 000 fiches de prises versées, importées à l'établi, par un
/// import tiré pour chaque groupe de trois ; chaque groupe regroupé dans sa première idée
/// (4 000 regroupements, qui retirent 4 000 idées et en laissent 2 000 visibles) ; pour chaque
/// idée visible, trois textes complétés l'un après l'autre, chacun d'au moins 200 octets (203 en
/// moyenne), et trois étiquettes ; 1 000 liens, l'idée visible 2k reliée à l'idée 2k + 1. L'aléa
/// est le sien, déterministe (splitmix64, d'une graine fixe) ; l'heure avance d'une seconde par
/// opération, depuis `heure`, au décalage local nul. `engendrer_le_fonds_de_mesure_selon`, sur
/// 2 000 groupes.
pub fn engendrer_le_fonds_de_mesure<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    heure: i64,
) -> Result<Engendre, Refus> {
    engendrer_le_fonds_de_mesure_selon(services, heure, GROUPES)
}

/// Le fonds de mesure (`engendrer_le_fonds_de_mesure`) sur `groupes` groupes de trois prises, au
/// lieu de 2 000, l'idée visible 2k reliée à l'idée 2k + 1 ; sans fonds, le premier versement le
/// fait naître. Le premier refus d'une opération arrête le générateur, et se rend tel quel. Rend
/// les idées visibles et retirées que lit le fonds à la fin, et les opérations faites.
pub fn engendrer_le_fonds_de_mesure_selon<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    heure: i64,
    groupes: usize,
) -> Result<Engendre, Refus> {
    let mut generateur = Generateur {
        alea: Splitmix64 { etat: GRAINE },
        heure,
        operations: 0,
    };
    let mut visibles = Vec::with_capacity(groupes);
    for _ in 0..groupes {
        visibles.push(engendrer_un_groupe(services, &mut generateur)?);
    }
    let (paires, _) = visibles.as_chunks::<2>();
    for [a, b] in paires {
        let lien = Relier {
            a: a.clone(),
            b: b.clone(),
            decalage_minutes: 0,
        };
        services.relier(&lien, &mut generateur.operation())?;
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

/// Un groupe : trois prises versées, importées à l'établi par un même import, tiré, chacune sous
/// l'idée qui naît avec elle ; la deuxième et la troisième regroupées dans l'idée de la première,
/// ce qui retire les leurs ; puis trois textes complétés dans cette idée, l'un après l'autre, et
/// trois étiquettes posées. Rend l'identifiant de l'idée qui reste visible.
fn engendrer_un_groupe<D: Documents, C: Captations>(
    services: &mut Services<D, C>,
    generateur: &mut Generateur,
) -> Result<String, Refus> {
    let provenance = gestes::Provenance::Importee {
        par: gestes::Appareil::Etabli,
        depuis: generateur.import(),
    };
    let mut versees = Vec::with_capacity(PRISES_PAR_GROUPE);
    for _ in 0..PRISES_PAR_GROUPE {
        let octets = generateur.octets_d_une_prise();
        let fiche = gestes::Fiche {
            format: "wav".to_string(),
            duree_ms: Some(generateur.duree_ms()),
        };
        versees.push(services.verser_une_fiche(
            gestes::Sorte::Prise,
            &octets,
            &fiche,
            &provenance,
            0,
            &mut generateur.operation(),
        )?);
    }
    let idee = versees[0].idee.clone();
    for versee in &versees[1..] {
        let regroupement = Regrouper {
            captation: Captation {
                sorte: commandes::Sorte::Prise,
                empreinte: versee.empreinte.clone(),
            },
            cible: idee.clone(),
            decalage_minutes: 0,
        };
        services.regrouper(&regroupement, &mut generateur.operation())?;
    }
    for _ in 0..TEXTES_PAR_IDEE {
        let completion = Completer {
            idee: idee.clone(),
            texte: generateur.texte(),
            decalage_minutes: 0,
        };
        services.completer(&completion, &mut generateur.operation())?;
    }
    for etiquette in generateur.etiquettes() {
        let etiquetage = Etiqueter {
            idee: idee.clone(),
            etiquette,
            decalage_minutes: 0,
        };
        services.poser_une_etiquette(&etiquetage, &mut generateur.operation())?;
    }
    Ok(idee)
}

/// Ce que le générateur tient d'une opération à l'autre : son aléa, l'heure de l'opération
/// suivante, en secondes, et les opérations déjà faites.
struct Generateur {
    alea: Splitmix64,
    heure: i64,
    operations: usize,
}

impl Generateur {
    /// Le contexte de l'opération suivante : son heure, et l'aléa du générateur ; l'opération
    /// compte, et l'heure avance d'une seconde.
    fn operation(&mut self) -> Contexte<'_> {
        let heure = self.heure;
        self.heure += 1;
        self.operations += 1;
        Contexte {
            heure,
            alea: &mut self.alea,
        }
    }

    /// L'identifiant d'un import : un ULID à l'heure de l'opération suivante, de dix octets
    /// d'aléa (§ 30 · 2).
    fn import(&mut self) -> String {
        let mut alea = [0u8; 10];
        self.alea.remplir(&mut alea);
        ulid(self.heure, alea)
    }

    /// Les octets d'une prise, tirés de l'aléa.
    fn octets_d_une_prise(&mut self) -> [u8; OCTETS_D_UNE_PRISE] {
        let mut octets = [0u8; OCTETS_D_UNE_PRISE];
        self.alea.remplir(&mut octets);
        octets
    }

    /// La durée d'une prise, en millisecondes, de 5 000 à 600 000.
    fn duree_ms(&mut self) -> u64 {
        let etendue = DUREE_LA_PLUS_LONGUE_MS - DUREE_LA_PLUS_COURTE_MS + 1;
        DUREE_LA_PLUS_COURTE_MS + self.alea.suivant() % etendue
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

/// L'aléa du générateur : splitmix64, de la graine `etat` ; le même tirage à chaque passage.
struct Splitmix64 {
    etat: u64,
}

impl Splitmix64 {
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

impl Alea for Splitmix64 {
    /// `octets`, huit par tirage, dans l'ordre petit-boutiste ; le dernier tirage, tronqué.
    fn remplir(&mut self, octets: &mut [u8]) {
        for morceau in octets.chunks_mut(8) {
            let tirage = self.suivant().to_le_bytes();
            morceau.copy_from_slice(&tirage[..morceau.len()]);
        }
    }
}
