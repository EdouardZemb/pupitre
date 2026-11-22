//! L'état d'un import (§ 30 · 9.2 ; questions 1, 2, 8, 14, 15, 17, 35, 37 et 38) : ce que
//! l'inspection lit du dossier à verser, par le seul contrat `Source`, et que les services gardent
//! jusqu'à l'ouverture ou à une autre inspection ; puis l'import ouvert, ce que chaque rang a
//! donné, ses jeux de stems et ses empreintes nommées, lues une fois puis tenues à jour (R26) ; et
//! les conversions entre les types de l'import des commandes et ceux du noyau. Les règles qui
//! décident (la signature, l'en-tête, le texte lisible, le jeu proposé, la place, le manifeste, le
//! rapport) sont au noyau : ce module leur passe ce qu'il lit, et garde ce qu'elles rendent.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read};

use automerge::{Automerge, ChangeHash};
use commandes::{
    DeclarationDUnDossier, DossierInspecte, ENTIER_SUR, FaitsDUneCaptation, FichierInspecte,
    JeuAVerser, NatureDuFichier, OrigineADeclarer, OrigineDeclaree, RapportDImport,
};
use noyau::donnees::entete::{self, ControleDuTexte, ENTETE_AU_PLUS};
use noyau::donnees::gestes::{Impossible, Sorte};
use noyau::donnees::identifiants::{Hacheur, empreinte};
use noyau::donnees::import::Faits;
use noyau::donnees::origine::{self, Ajout, Declaration, Origine, OriginesLues};
use noyau::donnees::rapport::{self, FichierDeLImport, ManqueDePlace, OrigineChangee, RapportLu};
use noyau::donnees::signature::{self, FormatSonore, Nature, Reconnu, SIGNATURE_AU_PLUS};
use noyau::donnees::stems::{self, TypeDeStem, WavInspecte};

use crate::lecture::{self, NomsPoses};
use crate::{Entree, Raison, Refus, SorteDEntree, Source};

/// Les octets lus d'un coup quand l'inspection calcule une empreinte en flux.
const TAMPON: usize = 65_536;

/// Le constat d'un flux de la source qui ne se lit plus, en chemin (`empreinte_en_flux`) : la
/// source ne dit pas pourquoi.
const FLUX_REFUSE: &str = "Sa lecture s'est interrompue.";

/// D'un WAV lisible, ce que le jeu proposé compare (question 37) : ses trames, sa fréquence, et
/// les octets lus de son en-tête, résumés par leur empreinte, pour ne pas garder 65 536 octets par
/// WAV ; deux débuts égaux ont la même, deux débuts distincts, deux autres.
#[derive(Clone, Debug)]
pub(crate) struct WavLu {
    trames: u64,
    frequence: u32,
    debut: String,
}

/// Un fichier du dossier à verser, tel que l'inspection l'a lu.
#[derive(Clone, Debug)]
pub(crate) struct FichierVu {
    /// Son rang dans la source (`Entree::rang`), un rang de `Source::entrees`.
    pub(crate) rang: u64,
    /// Son chemin dans le dossier versé, s'il est de l'UTF-8.
    pub(crate) chemin: Option<String>,
    /// Ses octets au parcours.
    pub(crate) octets: u64,
    /// Sa marque de dernière écriture au parcours ; `None` si le système de fichiers ne la tient
    /// pas.
    pub(crate) marque: Option<u64>,
    /// Sa nature, une étiquette ID3 résolue (`signature::apres_id3`).
    nature: Nature,
    /// Les faits de son en-tête : d'un WAV ou d'un MIDI lisibles, ceux de `entete::faits_wav` ou
    /// de `entete::faits_midi` ; d'un autre audio ou d'un texte, son format, la copie disant si le
    /// texte est lisible. `None` pour un WAV ou un MIDI dont l'en-tête ne se lit pas (illisible),
    /// et pour un fichier ignoré.
    faits: Option<Faits>,
    /// Son empreinte, calculée à l'inspection si sa taille est celle d'une captation (question 8).
    pub(crate) empreinte: Option<String>,
    /// Le fonds nomme déjà son contenu.
    connu: bool,
    /// D'un WAV lisible, ce que le jeu proposé compare.
    wav: Option<WavLu>,
}

impl FichierVu {
    /// La sorte de sa captation : une prise pour un audio, un MIDI versé seul pour un MIDI, un
    /// texte versé pour un texte ; `None` pour un fichier ignoré, qui ne se verse pas.
    pub(crate) fn sorte(&self) -> Option<Sorte> {
        match self.nature {
            Nature::Audio(_) => Some(Sorte::Prise),
            Nature::Midi => Some(Sorte::MidiVerseSeul),
            Nature::Texte(_) => Some(Sorte::TexteVerse),
            Nature::Ignore => None,
        }
    }

    /// Un texte, que la copie contrôle (`ControleDuTexte`).
    pub(crate) fn texte(&self) -> bool {
        matches!(self.nature, Nature::Texte(_))
    }

    /// Les faits que la copie verse : ceux de l'en-tête lu à l'inspection ; `None` (illisible)
    /// pour un en-tête qui ne se lit pas, ou pour un texte que la copie n'a pas dit lisible.
    pub(crate) fn faits_de_la_copie(&self, texte_lisible: bool) -> Option<Faits> {
        if self.texte() && !texte_lisible {
            None
        } else {
            self.faits.clone()
        }
    }

    /// Le dossier de son chemin, `""` pour le dossier versé ; `None` sans chemin en UTF-8 : un
    /// tel fichier n'a pas de dossier, ne rejoint aucun jeu et ne reçoit aucune origine déclarée.
    fn dossier(&self) -> Option<&str> {
        self.chemin
            .as_deref()
            .map(|chemin| chemin.rsplit_once('/').map_or("", |(dossier, _)| dossier))
    }
}

/// Lit chaque entrée de `source`, dans l'ordre de leurs rangs (question 11 ; R4 ; question 37) :
/// un lien ou une autre entrée, jamais lu, est ignoré ; un fichier se lit de `SIGNATURE_AU_PLUS`
/// octets (`signature::reconnaitre`) ; une étiquette ID3, de quatre de plus à sa suite
/// (`signature::apres_id3`) ; un WAV, de `ENTETE_AU_PLUS` depuis son début
/// (`entete::faits_wav`) ; un MIDI, de sa seule signature (`entete::faits_midi`) ; un ignoré ou un
/// texte, jamais au-delà. L'empreinte d'un audio, d'un MIDI ou d'un texte se calcule, lue en flux,
/// si sa taille est parmi `tailles` (question 8) ; il est connu si `nommees` la porte. Une lecture
/// refusée arrête l'inspection et nomme le fichier (`ne_se_lit_pas` ; question 38).
pub(crate) fn lire_les_fichiers<S: Source>(
    source: &S,
    tailles: &BTreeSet<u64>,
    nommees: &BTreeSet<String>,
) -> Result<Vec<FichierVu>, Refus> {
    source
        .entrees()
        .iter()
        .map(|entree| lire_le_fichier(source, entree, tailles, nommees))
        .collect()
}

/// L'entrée `entree` de `source`, lue comme le dit `lire_les_fichiers`.
fn lire_le_fichier<S: Source>(
    source: &S,
    entree: &Entree,
    tailles: &BTreeSet<u64>,
    nommees: &BTreeSet<String>,
) -> Result<FichierVu, Refus> {
    let mut vu = FichierVu {
        rang: entree.rang,
        chemin: entree.chemin.clone(),
        octets: entree.octets,
        marque: entree.marque,
        nature: Nature::Ignore,
        faits: None,
        empreinte: None,
        connu: false,
        wav: None,
    };
    if entree.sorte != SorteDEntree::Fichier {
        return Ok(vu);
    }
    let lire = |debut: u64, longueur: usize| {
        source
            .lire(entree.rang, debut, longueur)
            .map_err(|refus| ne_se_lit_pas(entree, &refus.message))
    };
    let debut = lire(0, SIGNATURE_AU_PLUS)?;
    vu.nature = match signature::reconnaitre(&debut, entree.extension.as_deref()) {
        Reconnu::Nature(nature) => nature,
        Reconnu::Id3 { suite } => Nature::Audio(signature::apres_id3(&lire(suite, 4)?)),
    };
    match vu.nature {
        Nature::Audio(FormatSonore::Wav) => {
            let entete = lire(0, ENTETE_AU_PLUS)?;
            let faits = entete::faits_wav(&entete, entree.octets);
            vu.wav = faits.map(|faits| WavLu {
                trames: faits.trames,
                frequence: faits.frequence,
                debut: empreinte(&entete),
            });
            vu.faits = faits.as_ref().map(Faits::du_wav);
        }
        Nature::Audio(format) => vu.faits = Some(Faits::d_un_audio(format)),
        Nature::Midi => vu.faits = entete::faits_midi(&debut).as_ref().map(Faits::du_midi),
        Nature::Texte(format) => vu.faits = Some(Faits::d_un_texte(format)),
        Nature::Ignore => return Ok(vu),
    }
    if tailles.contains(&entree.octets) {
        let calculee = empreinte_en_flux(source, entree)?;
        vu.connu = nommees.contains(&calculee);
        vu.empreinte = Some(calculee);
    }
    Ok(vu)
}

/// L'empreinte du fichier `entree`, lu en flux de bout en bout (`Source::lire_en_flux`), par
/// `Hacheur` ; une lecture refusée, comme dans `lire_le_fichier`.
fn empreinte_en_flux<S: Source>(source: &S, entree: &Entree) -> Result<String, Refus> {
    let mut flux = source
        .lire_en_flux(entree.rang)
        .map_err(|refus| ne_se_lit_pas(entree, &refus.message))?;
    let mut hacheur = Hacheur::nouveau();
    let mut tampon = vec![0u8; TAMPON];
    loop {
        match flux.read(&mut tampon) {
            Ok(0) => return Ok(hacheur.finir()),
            Ok(lus) => hacheur.ajouter(&tampon[..lus]),
            Err(erreur) if erreur.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(ne_se_lit_pas(entree, FLUX_REFUSE)),
        }
    }
}

/// Le refus d'une lecture de l'inspection (question 38) : `Raison::Disque`, « « x.wav » ne se lit
/// pas : <erreur>. », sans nom en UTF-8 « Le fichier n° <rang> ne se lit pas : <erreur>. ».
/// L'erreur est le constat de la source, son initiale en minuscule et son point final ôté.
fn ne_se_lit_pas(entree: &Entree, erreur: &str) -> Refus {
    let erreur = erreur.strip_suffix('.').unwrap_or(erreur);
    let mut caracteres = erreur.chars();
    let erreur: String = match caracteres.next() {
        Some(initiale) => initiale.to_lowercase().chain(caracteres).collect(),
        None => String::new(),
    };
    let message = match &entree.chemin {
        Some(chemin) => format!("« {chemin} » ne se lit pas : {erreur}."),
        None => format!("Le fichier n° {} ne se lit pas : {erreur}.", entree.rang),
    };
    Refus {
        raison: Raison::Disque,
        message,
    }
}

/// Chaque dossier du dossier versé, rangé par ses segments : le dossier versé (`""`), chaque
/// dossier qui porte un fichier, et chacun de ses parents. Pour chacun : ses fichiers audio, ceux
/// de son niveau seul (question 17) ; le jeu proposé, sur les tailles et les en-têtes lus de ses
/// WAV lisibles (`stems::jeu_propose`, sans empreinte ; question 37) ; la déclaration reproposée,
/// celle que `dernieres` porte pour son chemin (question 9).
pub(crate) fn dossiers_inspectes(
    fichiers: &[FichierVu],
    dernieres: &BTreeMap<String, Declaration>,
) -> Vec<DossierInspecte> {
    let mut audio: BTreeMap<&str, Vec<&FichierVu>> = BTreeMap::from([("", Vec::new())]);
    for fichier in fichiers {
        let Some(dossier) = fichier.dossier() else {
            continue;
        };
        let mut parent = dossier;
        while !audio.contains_key(parent) {
            audio.insert(parent, Vec::new());
            parent = parent.rsplit_once('/').map_or("", |(haut, _)| haut);
        }
        if matches!(fichier.nature, Nature::Audio(_)) {
            audio.entry(dossier).or_default().push(fichier);
        }
    }
    let mut dossiers: Vec<(&str, Vec<&FichierVu>)> = audio.into_iter().collect();
    dossiers.sort_by(|(a, _), (b, _)| a.split('/').cmp(b.split('/')));
    dossiers
        .into_iter()
        .map(|(chemin, siens)| {
            let wav: Vec<WavInspecte<'_>> = siens
                .iter()
                .filter_map(|fichier| {
                    fichier.wav.as_ref().map(|wav| WavInspecte {
                        taille: fichier.octets,
                        debut: wav.debut.as_bytes(),
                        trames: wav.trames,
                        frequence: wav.frequence,
                    })
                })
                .collect();
            DossierInspecte {
                chemin: chemin.to_string(),
                audio: siens.len() as u64,
                jeu_propose: stems::jeu_propose(&wav),
                reproposee: dernieres.get(chemin).map(declaration_des_commandes),
            }
        })
        .collect()
}

/// Vrai si `source` reprend l'import qui a posé `poses` (question 2 ; R2) : il a posé au moins un
/// nom de fichier ; chacun est le chemin d'une entrée de la source ; et chaque nom de ses prises
/// en stems est le préfixe, par segments entiers (`<nom>/`), du chemin d'une entrée, ou le nom du
/// dossier versé (`Source::nom_du_dossier`), celui d'un jeu versé seul.
pub(crate) fn reprend<S: Source>(source: &S, poses: &NomsPoses) -> bool {
    let chemins: BTreeSet<&str> = source
        .entrees()
        .iter()
        .filter_map(|entree| entree.chemin.as_deref())
        .collect();
    let sous = |jeu: &str| {
        chemins.iter().any(|chemin| {
            chemin
                .strip_prefix(jeu)
                .is_some_and(|reste| reste.starts_with('/'))
        })
    };
    !poses.fichiers.is_empty()
        && poses
            .fichiers
            .keys()
            .all(|chemin| chemins.contains(chemin.as_str()))
        && poses
            .jeux
            .iter()
            .all(|jeu| source.nom_du_dossier() == Some(jeu.as_str()) || sous(jeu))
}

/// Ce que l'inspection garde, dans les services, jusqu'à l'ouverture ou à une autre : chaque
/// fichier vu, la taille à verser et ses fichiers (question 8), l'import resté ouvert qu'elle
/// reprend, et ceux qu'elle ne reprend pas, que l'ouverture clôt d'abord (question 35).
pub(crate) struct InspectionGardee {
    /// Chaque fichier vu, dans l'ordre de leurs rangs.
    pub(crate) fichiers: Vec<FichierVu>,
    /// La taille à verser : les octets des audio, MIDI et textes dont le fonds ne nomme pas le
    /// contenu, en entier, avant toute borne.
    pub(crate) a_verser: u64,
    /// Le nombre de ces fichiers.
    a_verser_fichiers: u64,
    /// L'import resté ouvert que la source reprend.
    pub(crate) reprise: Option<String>,
    /// Les imports restés ouverts que la source ne reprend pas.
    pub(crate) a_clore: Vec<String>,
}

impl InspectionGardee {
    /// L'inspection de `fichiers`, sa taille à verser comptée, avec `reprise` et `a_clore`.
    pub(crate) fn de(
        fichiers: Vec<FichierVu>,
        reprise: Option<String>,
        a_clore: Vec<String>,
    ) -> Self {
        let a_verser: Vec<u64> = fichiers
            .iter()
            .filter(|fichier| fichier.sorte().is_some() && !fichier.connu)
            .map(|fichier| fichier.octets)
            .collect();
        InspectionGardee {
            a_verser: a_verser
                .iter()
                .fold(0u64, |somme, octets| somme.saturating_add(*octets)),
            a_verser_fichiers: a_verser.len() as u64,
            fichiers,
            reprise,
            a_clore,
        }
    }

    /// La place (`rapport::place`) : la taille à verser, ses fichiers, `poids_du_fonds` et
    /// `libre`, comparés en entiers, avant toute borne.
    pub(crate) fn place(&self, poids_du_fonds: u64, libre: u64) -> Result<(), ManqueDePlace> {
        rapport::place(self.a_verser, self.a_verser_fichiers, poids_du_fonds, libre)
    }
}

/// Ce qu'un rang de l'import a donné.
#[derive(Clone, Debug)]
pub(crate) enum Donne {
    /// Versé, dédoublonné, ou compté de nouveau à la reprise sans être relu : l'empreinte de son
    /// contenu, et s'il est illisible.
    Lu { empreinte: String, illisible: bool },
    /// Le fichier d'un jeu de stems, copié, qui attend sa prise en stems : son empreinte, ses
    /// faits, et s'il est illisible.
    Stem {
        empreinte: String,
        faits: Faits,
        illisible: bool,
    },
}

impl Donne {
    /// L'empreinte de son contenu.
    fn empreinte(&self) -> &str {
        match self {
            Donne::Lu { empreinte, .. } | Donne::Stem { empreinte, .. } => empreinte,
        }
    }

    /// Vrai s'il est illisible.
    fn illisible(&self) -> bool {
        match self {
            Donne::Lu { illisible, .. } | Donne::Stem { illisible, .. } => *illisible,
        }
    }
}

/// Un jeu de stems de l'import : ses fichiers audio, chacun un stem ; le MIDI qui accompagne un
/// stem, sous le rang de son stem (question 17) ; et s'il est versé, sa prise en stems faite.
#[derive(Clone, Debug)]
pub(crate) struct JeuOuvert {
    audio: Vec<u64>,
    midis: BTreeMap<u64, u64>,
    verse: bool,
}

/// Les jeux de stems de `fichiers`, d'après `declarations` (question 17) : pour chaque dossier
/// déclaré jeu de stems, ses fichiers audio de son niveau, chacun un stem, et chaque MIDI de son
/// niveau qui accompagne l'un d'eux (`stems::stem_du_midi`), le premier par rang, un seul par
/// stem ; un autre fichier du dossier se verse seul. Un dossier déclaré jeu de stems qui n'a aucun
/// fichier audio : `Impossible::JeuSansAudio`.
pub(crate) fn jeux_de(
    fichiers: &[FichierVu],
    declarations: &BTreeMap<String, Declaration>,
) -> Result<BTreeMap<String, JeuOuvert>, Impossible> {
    let mut jeux = BTreeMap::new();
    for (dossier, declaration) in declarations {
        if !declaration.stems {
            continue;
        }
        let siens: Vec<&FichierVu> = fichiers
            .iter()
            .filter(|fichier| fichier.dossier() == Some(dossier.as_str()))
            .collect();
        let audio: Vec<&FichierVu> = siens
            .iter()
            .copied()
            .filter(|fichier| matches!(fichier.nature, Nature::Audio(_)))
            .collect();
        if audio.is_empty() {
            return Err(Impossible::JeuSansAudio);
        }
        let chemins: Vec<&str> = audio
            .iter()
            .filter_map(|fichier| fichier.chemin.as_deref())
            .collect();
        let mut midis = BTreeMap::new();
        for midi in siens
            .iter()
            .filter(|fichier| fichier.nature == Nature::Midi)
        {
            let stem = midi
                .chemin
                .as_deref()
                .and_then(|chemin| stems::stem_du_midi(chemin, &chemins))
                .and_then(|stem| {
                    audio
                        .iter()
                        .find(|fichier| fichier.chemin.as_deref() == Some(stem))
                });
            if let Some(stem) = stem {
                midis.entry(stem.rang).or_insert(midi.rang);
            }
        }
        jeux.insert(
            dossier.clone(),
            JeuOuvert {
                audio: audio.iter().map(|fichier| fichier.rang).collect(),
                midis,
                verse: false,
            },
        );
    }
    Ok(jeux)
}

/// Un fichier copié d'un jeu, prêt pour sa prise en stems : son empreinte, ses faits, son chemin.
pub(crate) struct Copie {
    pub(crate) empreinte: String,
    pub(crate) faits: Faits,
    pub(crate) chemin: Option<String>,
}

/// L'import ouvert : son ULID, l'heure de son ouverture, la tête qui suit son ouverture, ses
/// déclarations, les fichiers que son inspection a vus, ses jeux de stems, ce que chaque rang a
/// donné, et les empreintes nommées, lues une fois à l'ouverture, puis tenues à jour à chaque
/// versement (R26).
pub(crate) struct EtatDeLImport {
    /// Son ULID.
    pub(crate) import: String,
    /// L'heure de son ouverture, en secondes : le jour de ses messages « Import du … ».
    pub(crate) ouvert_le: i64,
    /// La tête qui suit son ouverture, où se lisent les captations connues de son rapport.
    pub(crate) tete: Vec<ChangeHash>,
    /// Ses déclarations, traduites pour le noyau.
    pub(crate) declarations: BTreeMap<String, Declaration>,
    /// Les fichiers que son inspection a vus, dans l'ordre de leurs rangs.
    fichiers: Vec<FichierVu>,
    /// Ses jeux de stems, par dossier.
    jeux: BTreeMap<String, JeuOuvert>,
    /// Ce que chaque rang a donné.
    pub(crate) donnes: BTreeMap<u64, Donne>,
    /// Les empreintes que nomme le fonds, tenues à jour.
    pub(crate) nommees: BTreeSet<String>,
}

impl EtatDeLImport {
    /// L'import `import`, ouvert à `ouvert_le`, sa tête `tete`, ses déclarations, les fichiers de
    /// son inspection, ses jeux et les empreintes nommées `nommees` ; aucun rang n'a rien donné.
    pub(crate) fn nouveau(
        import: String,
        ouvert_le: i64,
        tete: Vec<ChangeHash>,
        declarations: BTreeMap<String, Declaration>,
        fichiers: Vec<FichierVu>,
        jeux: BTreeMap<String, JeuOuvert>,
        nommees: BTreeSet<String>,
    ) -> Self {
        EtatDeLImport {
            import,
            ouvert_le,
            tete,
            declarations,
            fichiers,
            jeux,
            donnes: BTreeMap::new(),
            nommees,
        }
    }

    /// La reprise (question 2 ; R7) : chaque fichier dont un nom porte l'ULID de l'import (`poses`)
    /// est compté de nouveau, sans être relu, illisible si sa captation n'a pas de `format` ;
    /// chaque jeu dont la prise en stems porte le nom, son chemin, ou `nom_du_dossier` pour le
    /// dossier versé, est versé.
    pub(crate) fn reprendre(&mut self, poses: &NomsPoses, nom_du_dossier: Option<&str>) {
        for fichier in &self.fichiers {
            let pose = fichier
                .chemin
                .as_ref()
                .and_then(|chemin| poses.fichiers.get(chemin));
            if let Some((empreinte, illisible)) = pose {
                self.donnes.insert(
                    fichier.rang,
                    Donne::Lu {
                        empreinte: empreinte.clone(),
                        illisible: *illisible,
                    },
                );
            }
        }
        for (dossier, jeu) in &mut self.jeux {
            let nom = if dossier.is_empty() {
                nom_du_dossier
            } else {
                Some(dossier.as_str())
            };
            jeu.verse = nom.is_some_and(|nom| poses.jeux.contains(nom));
        }
    }

    /// Le fichier de rang `rang` que l'inspection a vu.
    pub(crate) fn fichier(&self, rang: u64) -> Option<&FichierVu> {
        usize::try_from(rang.checked_sub(1)?)
            .ok()
            .and_then(|indice| self.fichiers.get(indice))
            .filter(|fichier| fichier.rang == rang)
    }

    /// Le dossier du jeu de stems dont le fichier `rang` est un stem, ou le MIDI d'un stem.
    pub(crate) fn jeu_de(&self, rang: u64) -> Option<&str> {
        self.jeux
            .iter()
            .find(|(_, jeu)| jeu.audio.contains(&rang) || jeu.midis.values().any(|m| *m == rang))
            .map(|(dossier, _)| dossier.as_str())
    }

    /// Le jeu `dossier`, s'il est un jeu de stems de l'import, et s'il est versé.
    pub(crate) fn jeu_verse(&self, dossier: &str) -> Option<bool> {
        self.jeux.get(dossier).map(|jeu| jeu.verse)
    }

    /// Note le jeu `dossier` versé.
    pub(crate) fn noter_le_jeu_verse(&mut self, dossier: &str) {
        if let Some(jeu) = self.jeux.get_mut(dossier) {
            jeu.verse = true;
        }
    }

    /// Les rangs à verser un à un, hors des jeux, dans leur ordre : chaque fichier qui n'est pas
    /// ignoré, et qui n'a encore rien donné.
    pub(crate) fn rangs_hors_des_jeux(&self) -> Vec<u64> {
        self.fichiers
            .iter()
            .filter(|fichier| fichier.sorte().is_some())
            .map(|fichier| fichier.rang)
            .filter(|rang| self.jeu_de(*rang).is_none() && !self.donnes.contains_key(rang))
            .collect()
    }

    /// Chaque jeu qui n'est pas versé, avec ses rangs, stems et MIDI, dans leur ordre, qui n'ont
    /// encore rien donné.
    pub(crate) fn jeux_a_verser(&self) -> Vec<JeuAVerser> {
        self.jeux
            .iter()
            .filter(|(_, jeu)| !jeu.verse)
            .map(|(dossier, jeu)| {
                let mut rangs: Vec<u64> = jeu
                    .audio
                    .iter()
                    .chain(jeu.midis.values())
                    .copied()
                    .filter(|rang| !self.donnes.contains_key(rang))
                    .collect();
                rangs.sort_unstable();
                JeuAVerser {
                    dossier: dossier.clone(),
                    rangs,
                }
            })
            .collect()
    }

    /// Les stems copiés du jeu `dossier`, dans l'ordre de leurs rangs, chacun avec son MIDI s'il en
    /// a un ; `None` si le dossier n'est pas un jeu de l'import, ou si l'un de ses fichiers n'est
    /// pas encore copié.
    pub(crate) fn stems_du_jeu(&self, dossier: &str) -> Option<Vec<(Copie, Option<Copie>)>> {
        let jeu = self.jeux.get(dossier)?;
        jeu.audio
            .iter()
            .map(|rang| {
                let midi = match jeu.midis.get(rang) {
                    Some(midi) => Some(self.copie(*midi)?),
                    None => None,
                };
                Some((self.copie(*rang)?, midi))
            })
            .collect()
    }

    /// Le fichier `rang`, copié pour un jeu.
    fn copie(&self, rang: u64) -> Option<Copie> {
        match self.donnes.get(&rang)? {
            Donne::Stem {
                empreinte, faits, ..
            } => Some(Copie {
                empreinte: empreinte.clone(),
                faits: faits.clone(),
                chemin: self.fichier(rang)?.chemin.clone(),
            }),
            Donne::Lu { .. } => None,
        }
    }

    /// Vrai si chaque fichier qui n'est pas ignoré a donné quelque chose, et si chaque jeu est
    /// versé.
    pub(crate) fn tout_est_verse(&self) -> bool {
        self.fichiers
            .iter()
            .filter(|fichier| fichier.sorte().is_some())
            .all(|fichier| self.donnes.contains_key(&fichier.rang))
            && self.jeux.values().all(|jeu| jeu.verse)
    }

    /// Les fichiers du rapport (`rapport::FichierDeLImport`) : chaque fichier ignoré, ses octets ;
    /// chaque fichier qui a donné quelque chose, ses octets au parcours, son empreinte, s'il est
    /// illisible et s'il n'a pas de nom en UTF-8.
    pub(crate) fn fichiers_du_rapport(&self) -> Vec<FichierDeLImport> {
        self.fichiers
            .iter()
            .filter_map(
                |fichier| match (fichier.sorte(), self.donnes.get(&fichier.rang)) {
                    (None, _) => Some(FichierDeLImport::Ignore {
                        rang: fichier.rang,
                        octets: fichier.octets,
                    }),
                    (Some(_), Some(donne)) => Some(FichierDeLImport::Lu {
                        rang: fichier.rang,
                        octets: fichier.octets,
                        empreinte: donne.empreinte().to_string(),
                        illisible: donne.illisible(),
                        sans_nom: fichier.chemin.is_none(),
                    }),
                    (Some(_), None) => None,
                },
            )
            .collect()
    }

    /// Les contenus que l'import a versés, dont une origine a pu changer : chaque fichier qui a
    /// donné quelque chose, et le manifeste de chaque jeu dont chaque stem est copié ou compté.
    pub(crate) fn empreintes_versees(&self) -> BTreeSet<String> {
        let mut versees: BTreeSet<String> = self
            .donnes
            .values()
            .map(|donne| donne.empreinte().to_string())
            .collect();
        for jeu in self.jeux.values() {
            let stems: Vec<&str> = jeu
                .audio
                .iter()
                .filter_map(|rang| self.donnes.get(rang))
                .map(Donne::empreinte)
                .collect();
            if stems.len() == jeu.audio.len() {
                versees.insert(empreinte(&rapport::manifeste(&stems)));
            }
        }
        versees
    }

    /// La table de l'import (R6) : le chemin de chaque fichier qui a donné quelque chose, versé ou
    /// dédoublonné, vers l'empreinte de son contenu.
    pub(crate) fn table(&self) -> BTreeMap<String, String> {
        self.fichiers
            .iter()
            .filter_map(|fichier| {
                let donne = self.donnes.get(&fichier.rang)?;
                Some((fichier.chemin.clone()?, donne.empreinte().to_string()))
            })
            .collect()
    }
}

/// Les origines que l'import a ajoutées au contenu `empreinte` : chaque élément de
/// `lecture::origines_de` que `lecture::origines_a`, à la tête `tete` de son ouverture, ne porte
/// pas, `main` d'abord, puis chaque générateur.
pub(crate) fn origines_changees(
    fonds: &Automerge,
    empreinte: &str,
    tete: &[ChangeHash],
) -> Vec<OrigineChangee> {
    let avant = lecture::origines_a(fonds, empreinte, tete);
    let apres = lecture::origines_de(fonds, empreinte);
    let main = (apres.main && !avant.main).then_some(Ajout::Main);
    let generees = apres
        .generees
        .difference(&avant.generees)
        .map(|nom| Ajout::Generee(nom.clone()));
    main.into_iter()
        .chain(generees)
        .map(|ajout| OrigineChangee {
            empreinte: empreinte.to_string(),
            element: ajout.element(),
        })
        .collect()
}

/// Le flux d'un fichier à verser, contrôlé en chemin : d'un texte, chaque tronçon lu passe par
/// `ControleDuTexte` (question 10).
pub(crate) struct FluxControle<'f> {
    flux: Box<dyn io::Read + 'f>,
    controle: Option<ControleDuTexte>,
}

impl<'f> FluxControle<'f> {
    /// Le flux `flux`, contrôlé s'il est celui d'un texte (`texte`).
    pub(crate) fn de(flux: Box<dyn io::Read + 'f>, texte: bool) -> Self {
        FluxControle {
            flux,
            controle: texte.then(ControleDuTexte::nouveau),
        }
    }

    /// Vrai si le flux d'un texte forme un texte lisible ; vrai pour tout autre flux.
    pub(crate) fn lisible(&self) -> bool {
        self.controle.as_ref().is_none_or(ControleDuTexte::lisible)
    }
}

impl io::Read for FluxControle<'_> {
    fn read(&mut self, tampon: &mut [u8]) -> io::Result<usize> {
        let lus = self.flux.read(tampon)?;
        if let Some(controle) = self.controle.as_mut() {
            controle.ajouter(&tampon[..lus]);
        }
        Ok(lus)
    }
}

/// Les déclarations des commandes, traduites pour le noyau (balayage Q3) : `stems` tel quel ;
/// `Aucune` en `None`, `Main` en `Some(Ajout::Main)`, `Generee { nom }` en
/// `Some(Ajout::Generee(nom))`. Une déclaration vide (`stems` faux, `Aucune`) ne s'y garde pas ;
/// de deux entrées du même chemin, la dernière l'emporte, vide comprise.
pub(crate) fn declarations_du_noyau(
    declarations: &[DeclarationDUnDossier],
) -> BTreeMap<String, Declaration> {
    let mut traduites = BTreeMap::new();
    for DeclarationDUnDossier {
        chemin,
        declaration,
    } in declarations
    {
        let traduite = Declaration {
            stems: declaration.stems,
            origine: match &declaration.origine {
                OrigineADeclarer::Aucune => None,
                OrigineADeclarer::Main => Some(Ajout::Main),
                OrigineADeclarer::Generee { nom } => Some(Ajout::Generee(nom.clone())),
            },
        };
        if traduite == Declaration::default() {
            traduites.remove(chemin);
        } else {
            traduites.insert(chemin.clone(), traduite);
        }
    }
    traduites
}

/// La déclaration des commandes qui répond à celle du noyau : l'inverse de
/// `declarations_du_noyau`, `None` en `Aucune`.
pub(crate) fn declaration_des_commandes(declaration: &Declaration) -> commandes::Declaration {
    commandes::Declaration {
        stems: declaration.stems,
        origine: match &declaration.origine {
            None => OrigineADeclarer::Aucune,
            Some(Ajout::Main) => OrigineADeclarer::Main,
            Some(Ajout::Generee(nom)) => OrigineADeclarer::Generee { nom: nom.clone() },
        },
    }
}

/// Les déclarations des commandes, une par chemin, dans l'ordre des chemins.
pub(crate) fn declarations_des_commandes(
    declarations: &BTreeMap<String, Declaration>,
) -> Vec<DeclarationDUnDossier> {
    declarations
        .iter()
        .map(|(chemin, declaration)| DeclarationDUnDossier {
            chemin: chemin.clone(),
            declaration: declaration_des_commandes(declaration),
        })
        .collect()
}

/// Le fichier inspecté des commandes : son rang, son chemin, ses octets, sa nature, et s'il est
/// connu.
pub(crate) fn fichier_inspecte(fichier: &FichierVu) -> FichierInspecte {
    FichierInspecte {
        rang: fichier.rang,
        chemin: fichier.chemin.clone(),
        octets: fichier.octets,
        nature: match fichier.nature {
            Nature::Audio(format) => NatureDuFichier::Audio {
                format: format.nom().to_string(),
            },
            Nature::Midi => NatureDuFichier::Midi,
            Nature::Texte(format) => NatureDuFichier::Texte {
                format: format.nom().to_string(),
            },
            Nature::Ignore => NatureDuFichier::Ignore,
        },
        connu: fichier.connu,
    }
}

/// Les faits des commandes qui répondent à ceux du noyau, champ pour champ.
pub(crate) fn faits_des_commandes(faits: &Faits) -> FaitsDUneCaptation {
    FaitsDUneCaptation {
        format: faits.format.clone(),
        duree_ms: faits.duree_ms,
        codage: faits.codage,
        canaux: faits.canaux,
        frequence: faits.frequence,
        bits: faits.bits,
        division: faits.division,
    }
}

/// L'origine déclarée que montre la vue : la plus forte de `lues` (`origine::la_plus_forte`),
/// générée avec chacun de ses noms, dans l'ordre de leurs octets ; aucune, `Aucune`.
pub(crate) fn origine_des_commandes(lues: &OriginesLues) -> OrigineDeclaree {
    match origine::la_plus_forte(lues) {
        Some(Origine::Generee(noms)) => OrigineDeclaree::Generee {
            noms: noms.into_iter().collect(),
        },
        Some(Origine::Main) => OrigineDeclaree::Main,
        None => OrigineDeclaree::Aucune,
    }
}

/// Le type de stem des commandes qui répond à celui du noyau.
pub(crate) fn type_des_commandes(type_de_stem: TypeDeStem) -> commandes::TypeDeStem {
    match type_de_stem {
        TypeDeStem::Voix => commandes::TypeDeStem::Voix,
        TypeDeStem::Choeurs => commandes::TypeDeStem::Choeurs,
        TypeDeStem::Batterie => commandes::TypeDeStem::Batterie,
        TypeDeStem::Percussions => commandes::TypeDeStem::Percussions,
        TypeDeStem::Basse => commandes::TypeDeStem::Basse,
        TypeDeStem::Guitare => commandes::TypeDeStem::Guitare,
        TypeDeStem::Clavier => commandes::TypeDeStem::Clavier,
        TypeDeStem::Cordes => commandes::TypeDeStem::Cordes,
        TypeDeStem::Cuivres => commandes::TypeDeStem::Cuivres,
        TypeDeStem::Synthe => commandes::TypeDeStem::Synthe,
        TypeDeStem::Effets => commandes::TypeDeStem::Effets,
        TypeDeStem::Autre => commandes::TypeDeStem::Autre,
    }
}

/// Le rapport des commandes qui répond au rapport relu par le noyau (`rapport::lire_le_rapport`) :
/// chaque compte, ses octets bornés à `ENTIER_SUR` (balayage Q6), ses fichiers, des rangs de
/// `Source::entrees`, sous `ENTIER_SUR` ; le nombre de ses origines changées ; les rangs de ses
/// fichiers sans nom.
pub(crate) fn rapport_des_commandes(lu: &RapportLu) -> RapportDImport {
    let compte = |compte: rapport::Compte| commandes::Compte {
        fichiers: compte.fichiers,
        octets: compte.octets.min(ENTIER_SUR),
    };
    RapportDImport {
        import: lu.import.clone(),
        vus: compte(lu.comptes.vus),
        verses: compte(lu.comptes.verses),
        dedoublonnes: compte(lu.comptes.dedoublonnes),
        illisibles: compte(lu.comptes.illisibles),
        ignores: compte(lu.comptes.ignores),
        sans_nom: compte(lu.comptes.sans_nom),
        origines_changees: lu.origines_changees,
        fichiers_sans_nom: lu.fichiers_sans_nom.clone(),
    }
}
