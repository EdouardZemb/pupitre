//! Les jours civils et les bornes d'une période : ce qu'un critère de date compare (question 9 ;
//! CAP-06). Rien ici ne lit l'horloge : le décalage local se reçoit, pour un instant. Puis les
//! critères d'une requête, et le texte de sa réponse (question 7). Enfin les requêtes enregistrées :
//! leur écriture, leur lecture, tolérante à une clé ou une valeur d'une version plus récente, et
//! leur rangement par nom.

use std::collections::BTreeMap;

use super::boite::{Contenu, accorde};
use super::changement::{Changement, Operation, Simple};
use super::fonds::{
    AU, CAPTEE, CONTENU, DU, ETAT, ETIQUETTE, IMPORTEE, NOM, PLUS_DE_SECONDES, PROVENANCE,
    REQUETES_ENREGISTREES, TEXTE,
};
use super::gestes::{ENTIER_SUR, Impossible, chemin, oter, poser};
use super::idee::Assiette;
use super::repli::{mots, replier};

/// Un jour civil (question 9) : l'année, le mois de 1 à 12, le jour de 1 à 31.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Jour {
    pub annee: i32,
    pub mois: u8,
    pub jour: u8,
}

/// Une année bissextile : divisible par 4, sauf les siècles non divisibles par 400.
fn est_bissextile(annee: i32) -> bool {
    (annee % 4 == 0 && annee % 100 != 0) || annee % 400 == 0
}

/// Le nombre de jours du mois `mois` (de 1 à 12) de l'année `annee`.
fn jours_du_mois(annee: i32, mois: u8) -> u8 {
    match mois {
        2 if est_bissextile(annee) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// `chiffres` n'est fait que de chiffres ASCII : sa valeur, sinon `None`.
fn nombre(chiffres: &str) -> Option<u32> {
    if chiffres.bytes().all(|c| c.is_ascii_digit()) {
        chiffres.parse().ok()
    } else {
        None
    }
}

impl Jour {
    /// « AAAA-MM-JJ », la forme d'un champ de date et du document : quatre chiffres, deux, deux ; un
    /// jour qui n'existe pas (le 30 février, un mois 13), ou toute autre forme : `None`.
    pub fn depuis_iso(texte: &str) -> Option<Jour> {
        let octets = texte.as_bytes();
        if octets.len() != 10 || octets[4] != b'-' || octets[7] != b'-' {
            return None;
        }
        let annee = nombre(texte.get(0..4)?)?;
        let mois = nombre(texte.get(5..7)?)?;
        let jour = nombre(texte.get(8..10)?)?;
        let annee = i32::try_from(annee).ok()?;
        let mois = u8::try_from(mois).ok()?;
        let jour = u8::try_from(jour).ok()?;
        if !(1..=12).contains(&mois) || jour < 1 || jour > jours_du_mois(annee, mois) {
            return None;
        }
        Some(Jour { annee, mois, jour })
    }

    /// « AAAA-MM-JJ ».
    pub fn en_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.annee, self.mois, self.jour)
    }

    /// Minuit UTC de ce jour, en secondes depuis 1970 (« days from civil », H. Hinnant).
    pub fn minuit_utc(self) -> i64 {
        let mois = i64::from(self.mois);
        // L'année commence en mars : janvier et février comptent pour l'année d'avant.
        let annee = i64::from(self.annee) - i64::from(mois <= 2);
        let ere = annee.div_euclid(400);
        let annee_de_l_ere = annee - ere * 400;
        // `mp`, le mois depuis mars (0) jusqu'à février (11).
        let mp = (mois + 9) % 12;
        let jour_de_l_annee = (153 * mp + 2) / 5 + i64::from(self.jour) - 1;
        let jour_de_l_ere =
            annee_de_l_ere * 365 + annee_de_l_ere / 4 - annee_de_l_ere / 100 + jour_de_l_annee;
        // 719 468 jours vont du 1er mars de l'an 0 au 1er janvier 1970.
        (ere * 146_097 + jour_de_l_ere - 719_468) * 86_400
    }
}

/// Les bornes d'une date, en secondes : `des` comprise, `avant` exclue ; une borne absente ne borne pas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bornes {
    pub des: Option<i64>,
    pub avant: Option<i64>,
}

impl Bornes {
    /// `heure` est dans les bornes.
    pub fn contient(self, heure: i64) -> bool {
        self.des.is_none_or(|des| heure >= des) && self.avant.is_none_or(|avant| heure < avant)
    }
}

/// Le début du jour civil `jour` à l'heure locale que dit `decalage` (en minutes, pour un instant) :
/// `m` le minuit UTC du jour, `d = m − decalage(m) × 60`, puis `m − decalage(d) × 60`.
pub fn debut_du_jour(jour: Jour, decalage: impl Fn(i64) -> i32) -> i64 {
    debut_depuis_minuit(jour.minuit_utc(), decalage)
}

fn debut_depuis_minuit(minuit: i64, decalage: impl Fn(i64) -> i32) -> i64 {
    let approche = minuit - i64::from(decalage(minuit)) * 60;
    minuit - i64::from(decalage(approche)) * 60
}

/// La période « du … au … », bornes comprises (question 9) : `des`, le début du jour `du` ; `avant`,
/// le début du jour qui suit `au` (son minuit UTC plus 86 400, puis le même calcul).
pub fn bornes_de_la_periode(
    du: Option<Jour>,
    au: Option<Jour>,
    decalage: impl Fn(i64) -> i32,
) -> Bornes {
    Bornes {
        des: du.map(|jour| debut_du_jour(jour, &decalage)),
        avant: au.map(|jour| debut_depuis_minuit(jour.minuit_utc() + 86_400, &decalage)),
    }
}

/// L'état d'une idée, pour une requête : reliée ou non.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etat {
    /// Reliée, au sens d'`idee::reliee`.
    Reliee,
    /// Non reliée.
    NonReliee,
}

/// La provenance d'une idée (§ 30 · 8.1, question 11) : captée ou importée.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceDeLIdee {
    /// Née d'une capture.
    Captee,
    /// Née d'un fichier versé.
    Importee,
}

/// Une requête (CAP-06) : une clé par sorte de critère, chacune facultative ; tous posés, ils se
/// combinent par « et » (question 7). `texte` : le texte saisi, tel quel ; ses mots (`repli::mots`)
/// sont les fragments cherchés, et sans mot, il ne pose aucun critère. `du`, `au` : la période, que
/// la réponse juge sur les bornes qu'on lui donne. `plus_de_secondes` : la durée affichée, en
/// secondes entières, strictement plus grande (question 8). `etiquette` : comparée octet pour octet
/// (question 10).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Requete {
    /// Le texte saisi, tel quel.
    pub texte: String,
    /// Le premier jour de la période, compris.
    pub du: Option<Jour>,
    /// Le dernier jour de la période, compris.
    pub au: Option<Jour>,
    /// L'étiquette cherchée.
    pub etiquette: Option<String>,
    /// Le seuil de la durée affichée, en secondes entières.
    pub plus_de_secondes: Option<u64>,
    /// L'état cherché.
    pub etat: Option<Etat>,
    /// La provenance cherchée.
    pub provenance: Option<ProvenanceDeLIdee>,
    /// Le contenu cherché.
    pub contenu: Option<Contenu>,
}

impl Requete {
    /// Aucun critère : ni mot, ni jour, ni étiquette, ni seuil, ni état, ni provenance, ni contenu.
    /// Hors de son texte, la requête est celle par défaut, et son texte n'a aucun mot.
    pub fn est_vide(&self) -> bool {
        mots(&self.texte).is_empty()
            && Requete {
                texte: String::new(),
                ..self.clone()
            } == Requete::default()
    }
}

/// Les parts « sans … » d'une réponse (question 7) : `Some` pour chaque critère posé dont une idée
/// peut manquer l'attribut (l'étiquette, la durée, le texte), le nombre d'idées lisibles qui le
/// manquent et satisfont chacun des autres critères qu'elles portent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sans {
    /// Les idées sans étiquette.
    pub etiquette: Option<usize>,
    /// Les idées sans durée.
    pub duree: Option<usize>,
    /// Les idées sans texte.
    pub texte: Option<usize>,
}

/// « N idée(s) », puis « · K sans étiquette », « · K sans durée », « · K sans texte », chaque part
/// non nulle, dans cet ordre, puis « · K illisible(s) » si K > 0, puis « · M interrogée(s) »,
/// M = `assiette.considerees` ; singulier pour 0 et 1 (question 7).
pub fn texte_de_la_reponse(
    retenues: usize,
    assiette: Assiette,
    illisibles: usize,
    sans: Sans,
) -> String {
    let mut parts = vec![accorde(retenues, "idée", "idées")];
    for (part, attribut) in [
        (sans.etiquette, "étiquette"),
        (sans.duree, "durée"),
        (sans.texte, "texte"),
    ] {
        if let Some(k) = part.filter(|&k| k > 0) {
            parts.push(format!("{k} sans {attribut}"));
        }
    }
    if illisibles > 0 {
        parts.push(accorde(illisibles, "illisible", "illisibles"));
    }
    parts.push(accorde(assiette.considerees, "interrogée", "interrogées"));
    parts.join(" · ")
}

/// La chaîne que le document range pour un état : `reliee` ou `non_reliee`. Un `match` exhaustif :
/// une variante ajoutée sans sa chaîne ne compile pas. C'est la seule correspondance ; la lecture
/// la parcourt à l'envers, sur la liste des variantes.
fn chaine_de_l_etat(etat: Etat) -> &'static str {
    match etat {
        Etat::Reliee => "reliee",
        Etat::NonReliee => "non_reliee",
    }
}

/// Les états, chacun une fois, que la lecture parcourt.
const ETATS: [Etat; 2] = [Etat::Reliee, Etat::NonReliee];

/// La chaîne que le document range pour une provenance : celles d'une idée (`CAPTEE`, `IMPORTEE`).
fn chaine_de_la_provenance(provenance: ProvenanceDeLIdee) -> &'static str {
    match provenance {
        ProvenanceDeLIdee::Captee => CAPTEE,
        ProvenanceDeLIdee::Importee => IMPORTEE,
    }
}

/// Les provenances, chacune une fois, que la lecture parcourt.
const PROVENANCES: [ProvenanceDeLIdee; 2] =
    [ProvenanceDeLIdee::Captee, ProvenanceDeLIdee::Importee];

/// La chaîne que le document range pour un contenu.
fn chaine_du_contenu(contenu: Contenu) -> &'static str {
    match contenu {
        Contenu::TexteDeLIdee => "texte_de_l_idee",
        Contenu::Prise => "prise",
        Contenu::Photo => "photo",
        Contenu::MidiVerse => "midi_verse",
        Contenu::TexteVerse => "texte_verse",
    }
}

/// Les contenus, chacun une fois, que la lecture parcourt.
const CONTENUS: [Contenu; 5] = [
    Contenu::TexteDeLIdee,
    Contenu::Prise,
    Contenu::Photo,
    Contenu::MidiVerse,
    Contenu::TexteVerse,
];

/// La valeur de `tous` dont `chaine_de` rend `chaine` ; une chaîne inconnue, `None`.
fn valeur_de<T: Copy>(tous: &[T], chaine_de: fn(T) -> &'static str, chaine: &str) -> Option<T> {
    tous.iter()
        .copied()
        .find(|&valeur| chaine_de(valeur) == chaine)
}

/// Les clés que pose une requête, une par critère posé, dans l'ordre : le texte (s'il a un mot),
/// `du`, `au`, `etiquette`, `plus_de_secondes`, `etat`, `provenance`, `contenu`. La seule
/// correspondance des critères et des clés pour l'écriture ; `requete_des_champs` est son inverse.
fn cles_de(requete: &Requete) -> Vec<(&'static str, Simple)> {
    let mut cles = Vec::new();
    if !mots(&requete.texte).is_empty() {
        cles.push((TEXTE, Simple::Chaine(requete.texte.clone())));
    }
    if let Some(du) = requete.du {
        cles.push((DU, Simple::Chaine(du.en_iso())));
    }
    if let Some(au) = requete.au {
        cles.push((AU, Simple::Chaine(au.en_iso())));
    }
    if let Some(etiquette) = &requete.etiquette {
        cles.push((ETIQUETTE, Simple::Chaine(etiquette.clone())));
    }
    if let Some(secondes) = requete.plus_de_secondes {
        cles.push((PLUS_DE_SECONDES, Simple::Naturel(secondes)));
    }
    if let Some(etat) = requete.etat {
        cles.push((ETAT, Simple::Chaine(chaine_de_l_etat(etat).to_string())));
    }
    if let Some(provenance) = requete.provenance {
        let chaine = chaine_de_la_provenance(provenance);
        cles.push((PROVENANCE, Simple::Chaine(chaine.to_string())));
    }
    if let Some(contenu) = requete.contenu {
        cles.push((
            CONTENU,
            Simple::Chaine(chaine_du_contenu(contenu).to_string()),
        ));
    }
    cles
}

/// La requête que disent les clés d'une requête enregistrée, hors `nom` ; une clé inconnue, une
/// valeur inconnue ou d'un autre type, un jour qui ne se lit pas ou un seuil au-delà d'`ENTIER_SUR` :
/// `None`.
fn requete_des_champs(champs: &BTreeMap<String, Option<Simple>>) -> Option<Requete> {
    let mut requete = Requete::default();
    for (cle, valeur) in champs {
        if cle == NOM {
            continue;
        }
        let valeur = valeur.as_ref()?;
        match (cle.as_str(), valeur) {
            (TEXTE, Simple::Chaine(texte)) => requete.texte = texte.clone(),
            (DU, Simple::Chaine(jour)) => requete.du = Some(Jour::depuis_iso(jour)?),
            (AU, Simple::Chaine(jour)) => requete.au = Some(Jour::depuis_iso(jour)?),
            (ETIQUETTE, Simple::Chaine(etiquette)) => requete.etiquette = Some(etiquette.clone()),
            (PLUS_DE_SECONDES, Simple::Naturel(secondes)) if *secondes <= ENTIER_SUR => {
                requete.plus_de_secondes = Some(*secondes);
            }
            (ETAT, Simple::Chaine(chaine)) => {
                requete.etat = Some(valeur_de(&ETATS, chaine_de_l_etat, chaine)?)
            }
            (PROVENANCE, Simple::Chaine(chaine)) => {
                requete.provenance =
                    Some(valeur_de(&PROVENANCES, chaine_de_la_provenance, chaine)?);
            }
            (CONTENU, Simple::Chaine(chaine)) => {
                requete.contenu = Some(valeur_de(&CONTENUS, chaine_du_contenu, chaine)?);
            }
            _ => return None,
        }
    }
    Some(requete)
}

/// Une requête enregistrée, lue : son ULID, son nom, et sa requête, `None` si elle porte une clé ou
/// une valeur que cette version ne connaît pas (question 17).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequeteEnregistreeLue {
    /// Son identifiant, un ULID.
    pub id: String,
    /// Son nom.
    pub nom: String,
    /// Sa requête, si cette version la comprend.
    pub requete: Option<Requete>,
}

/// Enregistrer : un objet créé sous `requetes_enregistrees/<id>` (`Changement::Creer`, sans
/// conteneur), son nom (sans ses blancs autour) et une clé par critère posé, dans la même
/// opération ; le texte n'est posé que s'il a un mot. Message : « Requête « <nom> » enregistrée ».
/// Un nom vide : `NomVide` ; aucun critère : `RequeteSansCritere` ; un seuil au-delà d'`ENTIER_SUR` :
/// `SeuilTropLong`.
pub fn enregistrer_une_requete(
    id: &str,
    nom: &str,
    requete: &Requete,
    heure: i64,
) -> Result<Operation, Impossible> {
    let nom = nom.trim();
    if nom.is_empty() {
        return Err(Impossible::NomVide);
    }
    if requete.est_vide() {
        return Err(Impossible::RequeteSansCritere);
    }
    if let Some(secondes) = requete.plus_de_secondes.filter(|&s| s > ENTIER_SUR) {
        return Err(Impossible::SeuilTropLong { secondes });
    }
    let mut changements = vec![
        Changement::Creer {
            chemin: chemin(&[REQUETES_ENREGISTREES, id]),
            conteneurs: Vec::new(),
        },
        poser(
            &[REQUETES_ENREGISTREES, id, NOM],
            Simple::Chaine(nom.to_string()),
        ),
    ];
    changements.extend(
        cles_de(requete)
            .into_iter()
            .map(|(cle, valeur)| poser(&[REQUETES_ENREGISTREES, id, cle], valeur)),
    );
    Ok(Operation {
        changements,
        message: format!("Requête « {nom} » enregistrée"),
        heure,
    })
}

/// Renommer : le nom posé de nouveau. Message : « Requête « <ancien> » renommée « <nouveau> » ». Un
/// nom vide : `NomVide` ; le même : `MemeNom`.
pub fn renommer_une_requete(
    lue: &RequeteEnregistreeLue,
    nom: &str,
    heure: i64,
) -> Result<Operation, Impossible> {
    let nom = nom.trim();
    if nom.is_empty() {
        return Err(Impossible::NomVide);
    }
    if nom == lue.nom {
        return Err(Impossible::MemeNom);
    }
    Ok(Operation {
        changements: vec![poser(
            &[REQUETES_ENREGISTREES, &lue.id, NOM],
            Simple::Chaine(nom.to_string()),
        )],
        message: format!("Requête « {} » renommée « {nom} »", lue.nom),
        heure,
    })
}

/// Supprimer : l'objet ôté d'un bloc. Message : « Requête « <nom> » supprimée ».
pub fn supprimer_une_requete(lue: &RequeteEnregistreeLue, heure: i64) -> Operation {
    Operation {
        changements: vec![oter(&[REQUETES_ENREGISTREES, &lue.id])],
        message: format!("Requête « {} » supprimée", lue.nom),
        heure,
    }
}

/// La requête enregistrée `id`, de ses clés : `champs` donne, de chaque clé de son objet, sa valeur
/// simple, ou `None` pour une valeur d'un autre type. Sans `nom` en chaîne : `None` (elle ne se
/// liste pas). Une clé inconnue, une valeur inconnue ou d'un autre type, un jour qui ne se lit pas,
/// un seuil au-delà d'`ENTIER_SUR` : la requête est `None`, le nom gardé.
pub fn lire_une_requete(
    id: &str,
    champs: &BTreeMap<String, Option<Simple>>,
) -> Option<RequeteEnregistreeLue> {
    let Some(Some(Simple::Chaine(nom))) = champs.get(NOM) else {
        return None;
    };
    Some(RequeteEnregistreeLue {
        id: id.to_string(),
        nom: nom.clone(),
        requete: requete_des_champs(champs),
    })
}

/// Range les requêtes par nom replié, puis par nom, octet pour octet, puis par ULID (question 18).
pub fn ranger_les_requetes(requetes: &mut [RequeteEnregistreeLue]) {
    requetes.sort_by(|a, b| {
        replier(&a.nom)
            .cmp(&replier(&b.nom))
            .then_with(|| a.nom.cmp(&b.nom))
    });
}
