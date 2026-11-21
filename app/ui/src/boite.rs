//! La boîte d'idées à l'écran, zone du fonds (CAP-02, CAP-06, CAP-07 ; questions 25 à 30). En
//! tête, le champ « Noter une idée », toujours présent, le compte, et sous une requête qui pose un
//! critère sa réponse, « Ranger par : », les critères de la requête en cours (« Contenu : »,
//! « Date : », « Étiquette : », « Durée : », « État : », « Provenance : »), les requêtes
//! enregistrées (les choisir, en enregistrer une, la renommer, la supprimer) et les gestes :
//! relier, annuler, les idées retirées, le journal du fonds. Dessous, la liste en fenêtre, réponse
//! de la requête en cours, des lignes de hauteur fixe que suivent son défilement et la taille de
//! la fenêtre de l'application, jamais une minuterie (I18) ; sans idée interrogée, la page du
//! fonds vide la cache et dit, par des constats, ce qu'il en est. À droite de la liste, le volet
//! de l'idée choisie, celui des idées retirées, ou celui du journal du fonds, en lecture seule ;
//! Échap ferme le volet ouvert comme « Fermer », sauf dans la recherche. Après chaque commande,
//! l'interface relit l'état du fonds et les fenêtres, et le bandeau reçoit l'issue ; seul le
//! refus d'une lecture y passe, jamais une lecture réussie. `Ctrl+Z`, hors d'un champ de texte,
//! annule le dernier geste de tri.

use commandes::{
    ANNULER, Assiette, BOITE, Boite, COMPLETER, Completer, Contenu, DELIER,
    ENREGISTRER_UNE_REQUETE, ETAT_DU_FONDS, Enregistrer, EntreeDuJournal, EtatDuFonds, Etiqueter,
    FENETRE_MAX, Faite, Fenetre, JOURNAL, LONGUEUR_D_UNE_LIGNE, LigneDeLaBoite, NOTER, Notee,
    Noter, OTER_UNE_ETIQUETTE, Ordre, POSER_UNE_ETIQUETTE, Provenance, RELIER,
    RENOMMER_UNE_REQUETE, REQUETES_ENREGISTREES, RETABLIR, RETIREES, RETIRER, Relier, Renommer,
    Requete, RequeteEnregistree, SUPPRIMER_UNE_REQUETE, Supprimer, VUE_D_UNE_IDEE, Viser,
    VueDUneIdee, VueDeLaBoite, VueDesRetirees, VueDuJournal,
};
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement, Node};
use noyau::donnees::boite::{date_de_la_liste, duree_affichee, etiquette_saisie, texte_du_compte};
use noyau::donnees::idee;
use noyau::donnees::journal::{empreinte_courte, heure_du_journal};
use noyau::donnees::repli::mots;
use noyau::donnees::requete::{self, texte_de_la_reponse};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;

use crate::app::Etat;
use crate::entete::choix;
use crate::natif;
use crate::recherche::{
    DUREES, ETATS, EtatDeLaRequete, PROVENANCES, bornes_de_la_requete, echap_ferme,
    est_la_recherche,
};

/// La marge de la fenêtre, en lignes, de chaque côté de ce qui se voit.
pub(crate) const MARGE: u32 = 5;

/// Les lignes que mesure l'étalon de la feuille de style, `calc(100 * var(--ligne))` : sa hauteur,
/// en pixels entiers, donne celle d'une ligne à un centième de pixel près.
const LIGNES_DE_L_ETALON: f64 = 100.0;

/// Ce qu'écrit une case vide (question 25).
const CASE_VIDE: &str = "—";

/// Les choix de « Ranger par : », chacun avec son libellé ; la date d'abord, l'ordre par défaut
/// (question 21).
const ORDRES: [(Ordre, &str); 5] = [
    (Ordre::Date, "date"),
    (Ordre::Etiquette, "étiquette"),
    (Ordre::Duree, "durée"),
    (Ordre::Reliee, "reliée"),
    (Ordre::Provenance, "provenance"),
];

/// Les choix de « Contenu : », chacun avec son libellé ; tout d'abord, sans critère de contenu
/// (question 22).
pub(crate) const CONTENUS: [(Option<Contenu>, &str); 6] = [
    (None, "tout"),
    (Some(Contenu::TexteDeLIdee), "texte de l'idée"),
    (Some(Contenu::Prise), "prise"),
    (Some(Contenu::Photo), "photo"),
    (Some(Contenu::MidiVerse), "MIDI versé"),
    (Some(Contenu::TexteVerse), "texte versé"),
];

/// `n` en `u32`, `u32::MAX` s'il le dépasse.
fn en_u32(n: u64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Vrai si une hauteur de ligne se lit : un nombre plus grand que zéro, ni nul, ni négatif, ni NaN.
fn lisible(hauteur_ligne_px: f64) -> bool {
    hauteur_ligne_px > 0.0
}

/// La fenêtre à demander : la première ligne vue, `floor(defilement / hauteur_ligne)`, moins
/// `MARGE` ; les lignes vues, `ceil(hauteur_vue / hauteur_ligne)`, plus `MARGE` après ; bornée par
/// `total` et par `FENETRE_MAX`. Un défilement ou une hauteur vue négatifs comptent pour zéro ;
/// sans hauteur de ligne lisible, les `FENETRE_MAX` premières lignes, toutes s'il y en a moins.
pub(crate) fn fenetre(
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
    total: u64,
) -> Fenetre {
    let plafond = u64::from(FENETRE_MAX);
    if !lisible(hauteur_ligne_px) {
        return Fenetre {
            debut: 0,
            nombre: en_u32(total.min(plafond)),
        };
    }
    // `as` sature : un quotient trop grand donne `u64::MAX`.
    let premiere = (defilement_px.max(0.0) / hauteur_ligne_px).floor() as u64;
    let vues = (hauteur_vue_px.max(0.0) / hauteur_ligne_px).ceil() as u64;
    let marge = u64::from(MARGE);
    let debut = premiere.saturating_sub(marge).min(total);
    let fin = premiere
        .saturating_add(vues)
        .saturating_add(marge)
        .min(total);
    Fenetre {
        debut: en_u32(debut),
        nombre: en_u32(fin.saturating_sub(debut).min(plafond)),
    }
}

/// Vrai si la fenêtre `demandee` porte toutes les lignes qui se voient, même en partie, et une
/// ligne au-delà de chaque bord vu, bornées par 0 et par `total` : le défilement n'a pas à en
/// demander une autre. La ligne au-delà fait partir la demande avant que le focus, qui descend de
/// ligne en ligne, n'atteigne la dernière ligne rendue. Sans hauteur de ligne lisible, rien n'est
/// couvert.
fn couvre(
    demandee: &Fenetre,
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
    total: u64,
) -> bool {
    if !lisible(hauteur_ligne_px) {
        return false;
    }
    let haut = defilement_px.max(0.0);
    let premiere = ((haut / hauteur_ligne_px).floor() as u64)
        .saturating_sub(1)
        .min(total);
    let fin = (((haut + hauteur_vue_px.max(0.0)) / hauteur_ligne_px).ceil() as u64)
        .saturating_add(1)
        .min(total);
    let debut = u64::from(demandee.debut);
    debut <= premiere && fin <= debut + u64::from(demandee.nombre)
}

/// La fenêtre que demande une liste défilée de `defilement_px`, vue sur `hauteur_vue_px`, aux
/// lignes de `hauteur_ligne_px` : celle de son défilement si `forcer`, ou si `derniere`, la
/// dernière demandée, ne porte pas toutes les lignes vues parmi les `total` lues (`couvre`) ;
/// sinon aucune. Elle n'est jamais bornée par le total lu, qu'un geste peut faire croître : la
/// relecture d'après le geste, qui reprend la dernière fenêtre demandée, porte ainsi jusqu'à
/// `MARGE` lignes nouvelles au bout de la liste.
fn fenetre_a_demander(
    derniere: Option<&Fenetre>,
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
    total: Option<u64>,
    forcer: bool,
) -> Option<Fenetre> {
    let total = total.unwrap_or(u64::MAX);
    let couverte =
        derniere.is_some_and(|d| couvre(d, defilement_px, hauteur_vue_px, hauteur_ligne_px, total));
    (forcer || !couverte)
        .then(|| fenetre(defilement_px, hauteur_vue_px, hauteur_ligne_px, u64::MAX))
}

/// Le bord d'une liste par où le focus y entre, venu du dehors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bord {
    Haut,
    Bas,
}

/// La ligne qui prend le focus venu du dehors d'une liste sur sa ligne de rang `rang`, la liste
/// défilée de `defilement_px` avant que `focus()` ne la fasse défiler, vue sur `hauteur_vue_px`,
/// aux lignes de `hauteur_ligne_px`. Tab atteint d'abord la première ligne de la fenêtre lue, et
/// Maj+Tab la dernière, chacune cachée dans sa marge, que `focus()` centrerait : une ligne
/// entièrement cachée au-dessus de la vue cède le focus à la première ligne entièrement vue,
/// `(Bord::Haut, rang)`, une ligne entièrement cachée au-dessous, à la dernière,
/// `(Bord::Bas, rang)`. Une ligne vue, même en partie, le garde ; sans ligne entièrement vue, ou
/// sans hauteur de ligne lisible, rien ne change.
fn ligne_d_entree(
    rang: u64,
    defilement_px: f64,
    hauteur_vue_px: f64,
    hauteur_ligne_px: f64,
) -> Option<(Bord, u64)> {
    if !lisible(hauteur_ligne_px) {
        return None;
    }
    let haut = defilement_px.max(0.0);
    let bas = haut + hauteur_vue_px.max(0.0);
    // Le haut de la ligne de rang `rang`, en pixels.
    let haut_de = |rang: u64| rang as f64 * hauteur_ligne_px;
    let (bord, cible) = if haut_de(rang.saturating_add(1)) <= haut {
        (Bord::Haut, (haut / hauteur_ligne_px).ceil() as u64)
    } else if haut_de(rang) >= bas {
        (
            Bord::Bas,
            ((bas / hauteur_ligne_px).floor() as u64).checked_sub(1)?,
        )
    } else {
        return None;
    };
    (haut_de(cible) >= haut && haut_de(cible.saturating_add(1)) <= bas).then_some((bord, cible))
}

/// L'idée de la ligne de rang `rang` dans sa liste, si la fenêtre lue `fenetre` la porte parmi ses
/// lignes `lignes`.
fn idee_de_rang(fenetre: &Fenetre, lignes: &[LigneDeLaBoite], rang: u64) -> Option<String> {
    let place = usize::try_from(rang.checked_sub(u64::from(fenetre.debut))?).ok()?;
    lignes.get(place).map(|ligne| ligne.idee.clone())
}

/// « Annuler : <message> », ou « Rien à annuler » (question 30).
pub(crate) fn libelle_d_annulation(a_annuler: Option<&str>) -> String {
    match a_annuler {
        Some(message) => format!("Annuler : {message}"),
        None => "Rien à annuler".to_string(),
    }
}

/// La page d'un fonds vide, quand aucune idée n'est interrogée (`considerees` vaut 0, Pupitre neuf
/// compris) : deux constats, jamais un ordre. Le premier dit la boîte d'idées vide, ce qui reste
/// vrai quand toutes les idées sont retirées : seule une idée retirée quitte la boîte, et le fonds
/// la garde. En lecture seule, le second renvoie au bandeau. Une recherche qui ne trouve rien
/// n'est pas un fonds vide : ses idées sont interrogées, et la liste vide dit son compte.
pub(crate) fn page_du_fonds_vide(
    considerees: u64,
    lecture_seule: bool,
) -> Option<[&'static str; 2]> {
    (considerees == 0).then_some([
        "La boîte d'idées est vide.",
        if lecture_seule {
            "Le fonds est en lecture seule : le bandeau dit pourquoi."
        } else {
            "Une idée se note dans le champ « Noter une idée »."
        },
    ])
}

/// `Ctrl+Z` annule un geste de tri hors d'un champ de texte seulement ; `Ctrl+Maj+Z`, qui donne
/// « Z », aussi.
pub(crate) fn ctrl_z_annule(touche: &str, ctrl: bool, dans_un_champ: bool) -> bool {
    ctrl && !dans_un_champ && touche.eq_ignore_ascii_case("z")
}

/// Noter en un geste (CAP-01 · 4) : `Entrée` dans le champ, un texte non vide une fois ses blancs
/// ôtés. La note part sans ses blancs autour, avec le décalage local du geste ; aucun titre n'est
/// demandé.
pub(crate) fn note_a_envoyer(champ: &str, touche: &str, decalage_minutes: i32) -> Option<Noter> {
    let texte = champ.trim();
    (touche == "Enter" && !texte.is_empty()).then(|| Noter {
        texte: texte.to_string(),
        decalage_minutes,
    })
}

/// La commande `Relier` des idées `a` et `b`, chacune avec l'instant de sa naissance, en
/// secondes : chacune datée, dans le message du journal, au décalage local de cet instant, que
/// rend `decalage` (`natif::decalage_a`), jamais à celui du geste. Relier et délier la partagent.
pub(crate) fn commande_de_lien(
    a: (String, i64),
    b: (String, i64),
    decalage: impl Fn(i64) -> i32,
) -> Relier {
    let (decalage_minutes, decalage_b_minutes) = (decalage(a.1), decalage(b.1));
    Relier {
        a: a.0,
        b: b.0,
        decalage_minutes,
        decalage_b_minutes,
    }
}

/// La commande `Relier` des idées cochées, si elles sont deux, chacune avec sa naissance
/// (`commande_de_lien`) ; une seule, ou trois, aucune.
pub(crate) fn relier_les_cochees(
    cochees: &[(String, i64)],
    decalage: impl Fn(i64) -> i32,
) -> Option<Relier> {
    match cochees {
        [a, b] => Some(commande_de_lien(a.clone(), b.clone(), decalage)),
        _ => None,
    }
}

/// La commande qui délie l'idée du volet `idee` de l'autre bout `autre` de l'un de ses liens,
/// chacun avec la naissance que le volet en a lue : `commande_de_lien`, par `decalage`. Un bout
/// dont la naissance ne s'est pas lue (`None`) n'a pas d'instant où lire son décalage : il prend
/// `du_geste`, le décalage du geste, et son jour, dans le message, peut s'écarter d'un de celui de
/// sa naissance. C'est la limite d'un bout illisible.
pub(crate) fn deliaison(
    idee: (String, Option<i64>),
    autre: (String, Option<i64>),
    decalage: impl Fn(i64) -> i32,
    du_geste: i32,
) -> Relier {
    let ((a, le_a), (b, le_b)) = (idee, autre);
    let mut commande = commande_de_lien(
        (a, le_a.unwrap_or_default()),
        (b, le_b.unwrap_or_default()),
        decalage,
    );
    if le_a.is_none() {
        commande.decalage_minutes = du_geste;
    }
    if le_b.is_none() {
        commande.decalage_b_minutes = du_geste;
    }
    commande
}

/// Une frappe d'`Entrée` envoie le champ, la première seulement : tenue, la touche se répète sans
/// rien renvoyer, comme `Ctrl+Z` (R13) ; pendant une composition, elle la valide et n'envoie rien.
fn envoi_au_clavier(touche: &str, tenue: bool, en_composition: bool) -> bool {
    touche == "Enter" && !tenue && !en_composition
}

/// Un texte, ou `—` s'il n'y en a pas, ou s'il est vide.
fn texte_ou_tiret(texte: Option<&str>) -> String {
    match texte {
        Some(texte) if !texte.is_empty() => texte.to_string(),
        _ => CASE_VIDE.to_string(),
    }
}

/// « reliée » ou « non reliée » : l'état se dit en mots, jamais par la seule couleur.
fn mot_de_l_etat(reliee: bool) -> &'static str {
    if reliee { "reliée" } else { "non reliée" }
}

/// « captée » ou « importée » (§ 30 · 8.1).
fn mot_de_la_provenance(provenance: &Provenance) -> &'static str {
    match provenance {
        Provenance::Captee { .. } => "captée",
        Provenance::Importee { .. } => "importée",
    }
}

/// Les cases d'une ligne : le texte, ou `—` ; les étiquettes, `#a #b`, ou `—` ; « reliée » ou
/// « non reliée » ; « captée » ou « importée » (question 25).
pub(crate) fn texte_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    texte_ou_tiret(ligne.texte.as_deref())
}

pub(crate) fn etiquettes_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    if ligne.etiquettes.is_empty() {
        return CASE_VIDE.to_string();
    }
    ligne
        .etiquettes
        .iter()
        .map(String::as_str)
        .map(etiquette_ecrite)
        .collect::<Vec<_>>()
        .join(" ")
}

/// L'étiquette `nom` écrite comme la liste l'écrit, et comme « Étiquette : » l'offre : `#nom`.
fn etiquette_ecrite(nom: &str) -> String {
    format!("#{nom}")
}

/// L'étiquette que dit la valeur du choix fait dans « Étiquette : » : « Toutes », de valeur vide,
/// n'en pose aucune (`None`), puisqu'une étiquette vide ne trouverait rien ; une autre valeur est
/// le nom de l'étiquette.
fn etiquette_du_choix(valeur: &str) -> Option<String> {
    (!valeur.is_empty()).then(|| valeur.to_string())
}

/// Les étiquettes que « Étiquette : » offre après « Toutes » : celles du fonds, `du_fonds`, et
/// l'étiquette `choisie` si le fonds ne la porte plus (une étiquette ôtée, une requête posée),
/// pour que le choix fait se voie ; chacune une fois, dans l'ordre de leurs octets.
fn etiquettes_a_choisir(du_fonds: &[String], choisie: Option<&str>) -> Vec<String> {
    let mut etiquettes = du_fonds.to_vec();
    if let Some(choisie) = choisie
        && !etiquettes.iter().any(|etiquette| etiquette == choisie)
    {
        etiquettes.push(choisie.to_string());
        etiquettes.sort();
    }
    etiquettes
}

pub(crate) fn etat_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str {
    mot_de_l_etat(ligne.reliee)
}

pub(crate) fn provenance_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str {
    mot_de_la_provenance(&ligne.provenance)
}

/// Le nom d'une ligne, que dit son bouton : sa date `date`, son texte, ses étiquettes et sa durée,
/// chacune nommée, son état et sa provenance. Les titres des colonnes, cachés aux lecteurs d'écran,
/// ne disent pas quel tiret est l'étiquette et lequel la durée ; le nom le dit.
fn nom_de_la_ligne(date: &str, ligne: &LigneDeLaBoite) -> String {
    format!(
        "{date} · {} · étiquettes {} · durée {} · {} · {}",
        texte_de_la_ligne(ligne),
        etiquettes_de_la_ligne(ligne),
        duree_affichee(ligne.duree_ms),
        etat_de_la_ligne(ligne),
        provenance_de_la_ligne(ligne)
    )
}

/// `n` en `usize`, `usize::MAX` s'il le dépasse.
fn en_usize(n: u64) -> usize {
    usize::try_from(n).unwrap_or(usize::MAX)
}

/// Un compte de la boîte et son assiette, tels que le noyau les reçoit (`texte_du_compte`,
/// `texte_de_la_reponse`) : chaque compte en `usize`, et les illisibles de l'assiette à part.
fn au_noyau(compte: u64, assiette: &Assiette) -> (usize, idee::Assiette, usize) {
    (
        en_usize(compte),
        idee::Assiette {
            considerees: en_usize(assiette.considerees),
            portant: en_usize(assiette.portant),
            decidees: en_usize(assiette.decidees),
        },
        en_usize(assiette.illisibles),
    )
}

/// Les parts « sans … » d'une vue, telles que le noyau les reçoit (`texte_de_la_reponse`),
/// chacune en `usize`.
fn sans_au_noyau(sans: &commandes::Sans) -> requete::Sans {
    requete::Sans {
        etiquette: sans.etiquette.map(en_usize),
        duree: sans.duree.map(en_usize),
        texte: sans.texte.map(en_usize),
    }
}

/// La requête pose au moins un critère : un mot dans son texte (`repli::mots`), ou un critère hors
/// du texte ; un texte sans mot, « … », n'en pose aucun. C'est l'inverse de `Requete::est_vide` du
/// noyau, recopié sur la requête des commandes, que l'interface ne convertit pas : la règle reste
/// hors mutation, et le test `pose_un_critere_dit_comme_le_noyau` garde qu'elle suit `est_vide`.
fn pose_un_critere(requete: &Requete) -> bool {
    !mots(&requete.texte).is_empty()
        || *requete
            != Requete {
                texte: requete.texte.clone(),
                ..Requete::default()
            }
}

/// La note d'une requête enregistrée par une version plus récente de Pupitre (CAP-06).
const NOTE_D_UNE_REQUETE_PLUS_RECENTE: &str =
    "Cette requête vient d'une version plus récente de Pupitre.";

/// Ce que dit, après la note, une requête d'une version plus récente : elle ne se lance pas, mais
/// choisie elle se renomme ou se supprime. Son bouton lit les deux phrases (`aria-describedby`).
const PRECISION_D_UNE_REQUETE_PLUS_RECENTE: &str =
    "Elle ne se lance pas : choisie, elle se renomme ou se supprime.";

/// Le constat d'une liste de requêtes enregistrées lue et vide, de la forme d'« Aucune idée
/// retirée. ». Rien avant la première lecture, ou quand la lecture est refusée, puisque rien n'en
/// est su ; rien non plus quand la liste porte une requête.
fn constat_des_requetes(lues: Option<&[RequeteEnregistree]>) -> Option<&'static str> {
    lues.is_some_and(<[RequeteEnregistree]>::is_empty)
        .then_some("Aucune requête enregistrée.")
}

/// Ce que montre une requête enregistrée : son nom et, si elle vient d'une version plus récente
/// (`requete: None`), la note qui dit qu'elle ne se lance pas.
fn ce_que_dit_la_requete(requete: &RequeteEnregistree) -> (String, Option<&'static str>) {
    (
        requete.nom.clone(),
        requete
            .requete
            .is_none()
            .then_some(NOTE_D_UNE_REQUETE_PLUS_RECENTE),
    )
}

/// Lance la requête enregistrée `requete` : ses réglages se posent sur la requête de l'écran, et
/// la liste suit. Vrai si elle est lancée ; celle d'une version plus récente ne l'est pas, et la
/// requête de l'écran ne change pas.
fn lancer(etat: EtatDeLaRequete, requete: &RequeteEnregistree) -> bool {
    match &requete.requete {
        Some(lue) => {
            etat.poser(lue);
            true
        }
        None => false,
    }
}

/// « Enregistrer la requête » s'offre à une requête qui pose un critère (`pose_un_critere`) et à un
/// nom qui n'est pas fait de blancs. Les autres refus, un fonds en lecture seule ou un Pupitre
/// neuf, sont ceux des services, que le bandeau dit.
fn enregistrer_possible(requete: &Requete, nom: &str) -> bool {
    pose_un_critere(requete) && !nom.trim().is_empty()
}

/// La réponse de la requête lue `requete`, que dit sa vue `vue` (`texte_de_la_reponse`) : le
/// nombre des idées trouvées, ses parts « sans … », ses illisibles et ses interrogées. Une requête
/// qui ne pose aucun critère rend toute la boîte d'idées, qui est sa réponse : aucun texte de
/// réponse ne se dit.
fn texte_de_la_requete(vue: &VueDeLaBoite, requete: &Requete) -> Option<String> {
    pose_un_critere(requete).then(|| {
        let (retenues, assiette, illisibles) = au_noyau(vue.retenues, &vue.assiette_de_la_requete);
        texte_de_la_reponse(retenues, assiette, illisibles, sans_au_noyau(&vue.sans))
    })
}

/// La date de la liste de l'instant `le`, au décalage local de cet instant (question 24).
fn date_locale(le: i64) -> String {
    date_de_la_liste(le, natif::decalage_a(le))
}

/// Ce qui nomme une idée liée : sa date et son texte, coupé à `LONGUEUR_D_UNE_LIGNE` caractères,
/// puis « retirée » si elle l'est.
fn nom_d_une_idee_liee(vue: &VueDUneIdee) -> String {
    let texte: String = texte_ou_tiret(vue.texte.as_deref())
        .chars()
        .take(LONGUEUR_D_UNE_LIGNE)
        .collect();
    let mut nom = format!("{} · {texte}", date_locale(vue.le));
    if vue.retiree.is_some() {
        nom.push_str(" · retirée");
    }
    nom
}

/// Ce que le volet dit de l'idée, en mots : reliée ou non, captée ou importée, sa durée, puis
/// « retirée » si elle l'est.
fn etat_de_l_idee(vue: &VueDUneIdee) -> String {
    let mut mots = format!(
        "{} · {} · durée {}",
        mot_de_l_etat(vue.reliee),
        mot_de_la_provenance(&vue.provenance),
        duree_affichee(vue.duree_ms)
    );
    if vue.retiree.is_some() {
        mots.push_str(" · retirée");
    }
    mots
}

/// La clé d'une ligne dans sa liste : la ligne entière, en JSON. Une ligne inchangée garde ses
/// éléments d'une fenêtre à l'autre ; une ligne changée se redessine. Le diff des lignes réinsère
/// les lignes gardées quand la fenêtre remonte, et une ligne redessinée est neuve : le focus
/// qu'elles portaient se perd, et se rend (`rendre_le_focus_perdu`).
fn cle_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    serde_json::to_string(ligne).unwrap_or_else(|_| ligne.idee.clone())
}

/// Vrai si la touche part d'un champ de texte, reconnu à la cible de l'événement : un
/// `<textarea>`, ou un `<input>` qui n'est ni une case à cocher, ni un bouton radio, ni un bouton.
pub(crate) fn dans_un_champ_de_texte(ev: &leptos::ev::KeyboardEvent) -> bool {
    let Some(cible) = ev.target() else {
        return false;
    };
    if cible.dyn_ref::<HtmlTextAreaElement>().is_some() {
        return true;
    }
    cible.dyn_ref::<HtmlInputElement>().is_some_and(|champ| {
        !matches!(
            champ.type_().as_str(),
            "checkbox"
                | "radio"
                | "button"
                | "submit"
                | "reset"
                | "image"
                | "file"
                | "range"
                | "color"
        )
    })
}

/// Le défilement de la liste montée `liste`, sa hauteur vue, et la hauteur d'une ligne, que donne
/// l'étalon monté `etalon`, en pixels ; rien si l'une ou l'autre n'est pas montée.
fn mesures(liste: NodeRef<html::Div>, etalon: NodeRef<html::Div>) -> Option<(f64, f64, f64)> {
    let liste = liste.get_untracked()?;
    let etalon = etalon.get_untracked()?;
    Some((
        f64::from(liste.scroll_top()),
        f64::from(liste.client_height()),
        f64::from(etalon.client_height()) / LIGNES_DE_L_ETALON,
    ))
}

/// L'argument d'une commande, en JSON ; rien s'il ne s'y écrit pas, et l'établi refuse alors la
/// commande, ce que le bandeau dit.
fn argument<T: Serialize>(commande: &T) -> Option<serde_json::Value> {
    serde_json::to_value(commande).ok()
}

/// Une fenêtre de la boîte, lue : la fenêtre demandée, la requête qui l'a lue, et la vue rendue,
/// sa réponse (`texte_de_la_requete`).
#[derive(Clone, Debug, PartialEq, Eq)]
struct BoiteLue {
    fenetre: Fenetre,
    requete: Requete,
    vue: VueDeLaBoite,
}

/// Une fenêtre des idées retirées, lue : la fenêtre demandée, et la vue rendue.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RetireesLues {
    fenetre: Fenetre,
    vue: VueDesRetirees,
}

/// Une fenêtre du journal du fonds, lue : la fenêtre demandée, et la vue rendue.
#[derive(Clone, Debug, PartialEq, Eq)]
struct JournalLu {
    fenetre: Fenetre,
    vue: VueDuJournal,
}

/// Ce que montre le volet, à droite de la liste : rien, l'idée choisie, les idées retirées, ou le
/// journal du fonds.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Volet {
    Aucun,
    Idee(String),
    Retirees,
    Journal,
}

/// Où va le focus quand le volet `volet` se ferme, par « Fermer » comme par Échap : au bouton de
/// la ligne qui a ouvert l'idée, si elle est dans les lignes lues `lues`, sinon à la liste
/// (`cible_apres_le_volet`) ; au bouton « Idées retirées » pour les retirées, au bouton « Journal
/// du fonds » pour le journal. `None` quand aucun volet n'est ouvert : rien ne se ferme.
fn cible_de_fermeture(volet: &Volet, lues: &[LigneDeLaBoite]) -> Option<Cible> {
    match volet {
        Volet::Aucun => None,
        Volet::Idee(id) => Some(cible_apres_le_volet(id, lues)),
        Volet::Retirees => Some(Cible::BoutonDesRetirees),
        Volet::Journal => Some(Cible::BoutonDuJournal),
    }
}

/// L'idée du volet, lue : sa vue, et chacun de ses liens, l'identifiant de l'autre bout, ce qui
/// le nomme (`nom_d_une_idee_liee`), jamais l'identifiant, et sa naissance, `None` pour un bout
/// qui ne s'est pas lu.
#[derive(Clone, Debug, PartialEq, Eq)]
struct IdeeOuverte {
    vue: VueDUneIdee,
    liens: Vec<(String, String, Option<i64>)>,
}

/// Les demandes d'une liste en fenêtre : la dernière fenêtre demandée, et son rang ; une réponse
/// qu'une demande plus récente a devancée s'ignore. Et le défilement de la liste à sa dernière
/// mesure, que `entrer_dans_la_liste` lui rend.
#[derive(Clone, Copy)]
struct Demandes {
    derniere: StoredValue<Option<Fenetre>>,
    rang: StoredValue<u64>,
    defilement: StoredValue<f64>,
}

impl Demandes {
    fn nouvelles() -> Self {
        Demandes {
            derniere: StoredValue::new(None),
            rang: StoredValue::new(0),
            defilement: StoredValue::new(0.0),
        }
    }

    /// Note `fenetre`, la dernière demandée, et rend le rang de sa demande.
    fn demander(self, fenetre: &Fenetre) -> u64 {
        self.derniere.set_value(Some(fenetre.clone()));
        self.rang.update_value(|rang| *rang = rang.wrapping_add(1));
        self.rang.get_value()
    }

    /// Vrai si la demande de rang `rang` est encore la dernière.
    fn est_la_derniere(self, rang: u64) -> bool {
        self.rang.get_value() == rang
    }
}

/// Ce que la boîte garde tant que l'établi est ouvert, qu'elle se montre ou non : l'ordre choisi,
/// la fenêtre lue, les lignes cochées, le volet et son idée, les idées retirées lues, et les champs
/// en cours. La requête en cours, elle, est à l'établi (`Etat::requete`). Rien n'en est gardé
/// d'une ouverture à l'autre. L'établi le crée, hors de la boîte : une réponse arrivée la boîte
/// cachée trouve où s'écrire.
#[derive(Clone, Copy)]
pub(crate) struct EtatDeLaBoite {
    /// Le rang de l'ordre dans `ORDRES`.
    ordre: RwSignal<usize>,
    lue: RwSignal<Option<BoiteLue>>,
    demandes: Demandes,
    /// Les idées cochées pour relier, dans l'ordre où elles l'ont été, chacune avec sa naissance,
    /// qui date son jour dans le message du lien.
    cochees: RwSignal<Vec<(String, i64)>>,
    volet: RwSignal<Volet>,
    idee: RwSignal<Option<IdeeOuverte>>,
    /// Le rang de la dernière lecture de l'idée du volet.
    lectures_de_l_idee: StoredValue<u64>,
    retirees: RwSignal<Option<RetireesLues>>,
    demandes_des_retirees: Demandes,
    /// Le journal du fonds lu, et ses demandes.
    journal: RwSignal<Option<JournalLu>>,
    demandes_du_journal: Demandes,
    /// Le champ « Noter une idée ».
    note: RwSignal<String>,
    /// Une note est partie, sans relecture encore : `Entrée` n'en envoie pas une seconde.
    note_en_cours: StoredValue<bool>,
    /// Le champ « Texte complété » du volet, et l'idée dont il est parti.
    brouillon: RwSignal<String>,
    brouillon_de: StoredValue<Option<String>>,
    /// Le champ « Étiquette » du volet.
    etiquette: RwSignal<String>,
    /// La commande d'une ligne qui a eu le focus en dernier, tant qu'aucun autre élément de la boîte
    /// ne l'a pris.
    gardee: StoredValue<Option<Gardee>>,
    /// Un geste de tri, ou l'annulation, est parti, sans relecture encore : une seconde activation
    /// s'ignore.
    geste_en_vol: StoredValue<bool>,
    /// Les requêtes enregistrées lues, rangées par nom par la commande qui les lit ; aucune avant
    /// la première lecture.
    requetes: RwSignal<Option<Vec<RequeteEnregistree>>>,
    /// L'identifiant de la requête enregistrée choisie, sur laquelle portent « Renommer » et
    /// « Supprimer ».
    requete_choisie: RwSignal<Option<String>>,
    /// Le champ « Nom » des requêtes enregistrées.
    nom_de_la_requete: RwSignal<String>,
    /// Un geste d'une requête enregistrée est parti, sans relecture encore : une seconde
    /// activation s'ignore.
    requete_en_vol: StoredValue<bool>,
}

impl EtatDeLaBoite {
    /// La boîte rangée par date, rien de lu ni de coché, aucun volet.
    pub(crate) fn nouveau() -> Self {
        EtatDeLaBoite {
            ordre: RwSignal::new(0),
            lue: RwSignal::new(None),
            demandes: Demandes::nouvelles(),
            cochees: RwSignal::new(Vec::new()),
            volet: RwSignal::new(Volet::Aucun),
            idee: RwSignal::new(None),
            lectures_de_l_idee: StoredValue::new(0),
            retirees: RwSignal::new(None),
            demandes_des_retirees: Demandes::nouvelles(),
            journal: RwSignal::new(None),
            demandes_du_journal: Demandes::nouvelles(),
            note: RwSignal::new(String::new()),
            note_en_cours: StoredValue::new(false),
            brouillon: RwSignal::new(String::new()),
            brouillon_de: StoredValue::new(None),
            etiquette: RwSignal::new(String::new()),
            gardee: StoredValue::new(None),
            geste_en_vol: StoredValue::new(false),
            requetes: RwSignal::new(None),
            requete_choisie: RwSignal::new(None),
            nom_de_la_requete: RwSignal::new(String::new()),
            requete_en_vol: StoredValue::new(false),
        }
    }

    /// Choisit la requête enregistrée `requete` : « Renommer » et « Supprimer » portent sur elle,
    /// et le champ « Nom » prend son nom.
    fn choisir_la_requete(self, requete: &RequeteEnregistree) {
        self.requete_choisie.set(Some(requete.id.clone()));
        self.nom_de_la_requete.set(requete.nom.clone());
    }

    /// Un clic sur la requête enregistrée `requete` : elle se choisit, et se lance si elle est
    /// lisible (`lancer`) ; celle d'une version plus récente se choisit seulement, pour qu'on
    /// puisse la renommer ou la supprimer.
    fn choisir_et_lancer(self, etat: EtatDeLaRequete, requete: &RequeteEnregistree) {
        self.choisir_la_requete(requete);
        lancer(etat, requete);
    }

    /// Ce qui suit un geste d'une requête enregistrée, `fait` ou refusé, une fois son issue au
    /// bandeau : rend la cible où va le focus. Supprimer, fait : la requête sort de la liste, le
    /// choix s'oublie (`oublier_la_requete`), et le focus, qui était sur « Supprimer », va au champ
    /// « Nom ». Enregistrer, fait : le champ « Nom » se vide s'il porte encore le nom envoyé.
    /// Renommer garde le choix. Un refus ne change rien. La liste se relit après chaque geste, fait
    /// ou refusé (`Ecran::geste_de_requete`).
    fn apres_un_geste_de_requete(self, geste: &GesteDeRequete, fait: bool) -> Option<Cible> {
        if !fait {
            return None;
        }
        match geste {
            GesteDeRequete::Enregistrer { nom } => {
                self.nom_de_la_requete.update(|champ| {
                    if *champ == *nom {
                        champ.clear();
                    }
                });
                None
            }
            GesteDeRequete::Renommer => None,
            GesteDeRequete::Supprimer { requete } => {
                self.oublier_la_requete(requete);
                Some(Cible::ChampDeNom)
            }
        }
    }

    /// La requête enregistrée `id` n'est plus : elle sort de la liste, et, si elle était choisie,
    /// le choix et le champ « Nom » se vident. La requête de l'écran (`Etat::requete`) reste : seule
    /// la liste des requêtes enregistrées change.
    fn oublier_la_requete(self, id: &str) {
        self.requetes.update(|lues| {
            if let Some(liste) = lues {
                liste.retain(|r| r.id != id);
            }
        });
        if self
            .requete_choisie
            .with_untracked(|choisie| choisie.as_deref() == Some(id))
        {
            self.requete_choisie.set(None);
            self.nom_de_la_requete.set(String::new());
        }
    }

    /// Montre l'idée `id` dans le volet ; vrai si elle n'y était pas. Une autre idée : l'idée lue,
    /// le champ « Étiquette », le brouillon et l'idée dont il est parti s'oublient, et rien de la
    /// précédente ne part vers elle.
    fn montrer_l_idee(self, id: &str) -> bool {
        if self
            .volet
            .with_untracked(|volet| matches!(volet, Volet::Idee(i) if i == id))
        {
            return false;
        }
        self.idee.set(None);
        self.etiquette.set(String::new());
        self.brouillon.set(String::new());
        self.brouillon_de.set_value(None);
        self.volet.set(Volet::Idee(id.to_string()));
        true
    }
}

/// La commande qui lit la fenêtre `fenetre` de la boîte, rangée par `ordre`, réponse de la requête
/// `requete`, telle quelle, et les bornes de sa date, tirées de ses jours au décalage que
/// `decalage` dit de chaque instant (`bornes_de_la_requete`, `natif::decalage_a`).
fn commande_de_la_boite(
    ordre: Ordre,
    requete: &Requete,
    decalage: impl Fn(i64) -> i32,
    fenetre: &Fenetre,
) -> Boite {
    Boite {
        ordre,
        requete: requete.clone(),
        bornes: bornes_de_la_requete(requete, decalage),
        debut: fenetre.debut,
        nombre: fenetre.nombre,
    }
}

/// La requête en cours, lue sans suivre ses réglages : une lecture de la boîte la porte.
fn requete_en_cours(etat: Etat) -> Requete {
    untrack(|| etat.requete.requete())
}

/// Lit la fenêtre `fenetre` de la boîte, dans l'ordre choisi, réponse de la requête `requete`
/// (`commande_de_la_boite`). Une réponse qu'une demande plus récente a devancée s'ignore : chaque
/// frappe demande, et seule la dernière réponse compte ; un refus va au bandeau. La fenêtre posée,
/// avec la requête qui l'a lue, et ses lignes rendues au tick suivant, le focus perdu se rend
/// (`rendre_le_focus_perdu`).
async fn lire_la_boite(
    etat: Etat,
    boite: EtatDeLaBoite,
    focus: RwSignal<Option<Cible>>,
    fenetre: Fenetre,
    requete: Requete,
) {
    let rang = boite.demandes.demander(&fenetre);
    let ordre = ORDRES[boite.ordre.get_untracked()].0.clone();
    let commande = commande_de_la_boite(ordre, &requete, natif::decalage_a, &fenetre);
    let lu = natif::invoquer::<VueDeLaBoite>(BOITE, argument(&commande)).await;
    if !boite.demandes.est_la_derniere(rang) {
        return;
    }
    match lu {
        Ok(vue) => {
            boite.lue.set(Some(BoiteLue {
                fenetre,
                requete,
                vue,
            }));
            leptos::task::tick().await;
            rendre_le_focus_perdu(boite, focus, Liste::Boite);
        }
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
}

/// Lit la fenêtre `fenetre` des idées retirées, comme `lire_la_boite`.
async fn lire_les_retirees(
    etat: Etat,
    boite: EtatDeLaBoite,
    focus: RwSignal<Option<Cible>>,
    fenetre: Fenetre,
) {
    let rang = boite.demandes_des_retirees.demander(&fenetre);
    let lu = natif::invoquer::<VueDesRetirees>(RETIREES, argument(&fenetre)).await;
    if !boite.demandes_des_retirees.est_la_derniere(rang) {
        return;
    }
    match lu {
        Ok(vue) => {
            boite.retirees.set(Some(RetireesLues { fenetre, vue }));
            leptos::task::tick().await;
            rendre_le_focus_perdu(boite, focus, Liste::Retirees);
        }
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
}

/// Lit la fenêtre `fenetre` du journal du fonds, comme `lire_les_retirees` ; ses lignes n'ont
/// aucune commande, le focus n'a rien à y rendre.
async fn lire_le_journal(etat: Etat, boite: EtatDeLaBoite, fenetre: Fenetre) {
    let rang = boite.demandes_du_journal.demander(&fenetre);
    let lu = natif::invoquer::<VueDuJournal>(JOURNAL, argument(&fenetre)).await;
    if !boite.demandes_du_journal.est_la_derniere(rang) {
        return;
    }
    match lu {
        Ok(vue) => boite.journal.set(Some(JournalLu { fenetre, vue })),
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
}

/// Lit l'idée `id`, puis chacune de ses idées liées, pour les nommer. Le refus de l'idée va au
/// bandeau ; une idée liée qui ne se lit pas se nomme `CASE_VIDE`, qui le dit déjà, et son refus
/// ne va pas au bandeau. La lecture finie, si l'idée est encore celle du volet et que nulle
/// lecture plus récente ne l'a devancée, elle devient l'idée du volet ; le champ « Texte
/// complété » part de son texte, s'il n'est pas déjà parti d'elle. L'idée posée, si le contrôle
/// qui avait le focus a disparu (un geste de tri, l'idée devenue retirée), le focus va au titre
/// du volet.
async fn lire_l_idee(etat: Etat, boite: EtatDeLaBoite, focus: RwSignal<Option<Cible>>, id: String) {
    boite
        .lectures_de_l_idee
        .update_value(|rang| *rang = rang.wrapping_add(1));
    let rang = boite.lectures_de_l_idee.get_value();
    let vue = match natif::invoquer::<VueDUneIdee>(
        VUE_D_UNE_IDEE,
        Some(serde_json::Value::String(id.clone())),
    )
    .await
    {
        Ok(vue) => vue,
        Err(refus) => {
            etat.apres_un_geste(&Err(refus));
            return;
        }
    };
    let mut liens = Vec::new();
    for autre in &vue.liens {
        let (nom, le) = match natif::invoquer::<VueDUneIdee>(
            VUE_D_UNE_IDEE,
            Some(serde_json::Value::String(autre.clone())),
        )
        .await
        {
            Ok(liee) => (nom_d_une_idee_liee(&liee), Some(liee.le)),
            Err(_) => (CASE_VIDE.to_string(), None),
        };
        liens.push((autre.clone(), nom, le));
    }
    let au_volet = boite
        .volet
        .with_untracked(|volet| matches!(volet, Volet::Idee(i) if *i == id));
    if boite.lectures_de_l_idee.get_value() != rang || !au_volet {
        return;
    }
    if boite
        .brouillon_de
        .with_value(|de| de.as_deref() != Some(id.as_str()))
    {
        boite.brouillon.set(vue.texte.clone().unwrap_or_default());
        boite.brouillon_de.set_value(Some(id));
    }
    boite.idee.set(Some(IdeeOuverte { vue, liens }));
    leptos::task::tick().await;
    if actif() == Actif::Absent {
        focus.set(Some(Cible::TitreDeLIdee));
    }
}

/// Un geste d'une requête enregistrée, avec ce qu'il faut pour conclure son issue.
#[derive(Clone, Debug, PartialEq, Eq)]
enum GesteDeRequete {
    /// Enregistrer sous le nom envoyé.
    Enregistrer {
        nom: String,
    },
    Renommer,
    /// Supprimer la requête enregistrée d'identifiant `requete`.
    Supprimer {
        requete: String,
    },
}

/// Lit les requêtes enregistrées, rangées par nom par la commande ; un refus va au bandeau. Elle se
/// relit à l'ouverture de la boîte et après chaque geste fait d'une requête enregistrée.
async fn lire_les_requetes(etat: Etat, boite: EtatDeLaBoite) {
    match natif::invoquer::<Vec<RequeteEnregistree>>(REQUETES_ENREGISTREES, None).await {
        Ok(lues) => boite.requetes.set(Some(lues)),
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
}

/// Après une commande : l'état du fonds, hors de l'ouverture (`suivre_le_fonds`), puis la dernière
/// fenêtre demandée de la boîte, réponse de la requête en cours, celle des idées retirées si leur
/// volet est ouvert, et l'idée du volet ; chaque lecture rend le focus perdu, par `focus`.
async fn relire(etat: Etat, boite: EtatDeLaBoite, focus: RwSignal<Option<Cible>>) {
    match natif::invoquer::<EtatDuFonds>(ETAT_DU_FONDS, None).await {
        Ok(fonds) => etat.suivre_le_fonds(fonds, false),
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
    if let Some(fenetre) = boite.demandes.derniere.get_value() {
        lire_la_boite(etat, boite, focus, fenetre, requete_en_cours(etat)).await;
    }
    match boite.volet.get_untracked() {
        Volet::Retirees => {
            if let Some(fenetre) = boite.demandes_des_retirees.derniere.get_value() {
                lire_les_retirees(etat, boite, focus, fenetre).await;
            }
        }
        Volet::Journal => {
            if let Some(fenetre) = boite.demandes_du_journal.derniere.get_value() {
                lire_le_journal(etat, boite, fenetre).await;
            }
        }
        Volet::Idee(id) => lire_l_idee(etat, boite, focus, id).await,
        Volet::Aucun => {}
    }
}

/// Une liste en fenêtre de l'écran : celle de la boîte, ou celle des idées retirées.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Liste {
    Boite,
    Retirees,
}

/// Une commande d'une ligne : son bouton, qui ouvre le volet de l'idée, ou sa case à cocher, dans
/// la liste de la boîte ; « Rétablir », dans celle des idées retirées.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Commande {
    Choisir,
    Cocher,
    Retablir,
}

impl Commande {
    /// La liste où paraît la commande.
    fn liste(self) -> Liste {
        match self {
            Commande::Choisir | Commande::Cocher => Liste::Boite,
            Commande::Retablir => Liste::Retirees,
        }
    }
}

/// La commande d'une ligne qui a eu le focus en dernier : l'idée de la ligne, et la commande.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Gardee {
    idee: String,
    commande: Commande,
}

/// Ce qui a le focus dans le document : rien, ou `body`, où tombe le focus d'un élément déplacé ou
/// recréé ; ou un autre élément.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Actif {
    Absent,
    Ailleurs,
}

/// Où rendre le focus d'une liste : nulle part, à une commande d'une ligne, ou à la liste.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Rendu {
    Rien,
    Ligne(Gardee),
    Liste,
}

/// Où rendre le focus de la liste `liste`, ses lignes lues `lues` posées : perdu (`Actif::Absent`)
/// alors que la dernière commande focalisée, `gardee`, est d'une ligne de cette liste, il revient à
/// la même commande si la ligne est encore lue, sinon à la liste. Le focus ailleurs, aucune ligne
/// gardée, ou celle de l'autre liste : rien ne bouge.
fn focus_a_rendre(
    gardee: Option<&Gardee>,
    liste: Liste,
    lues: &[LigneDeLaBoite],
    actif: Actif,
) -> Rendu {
    let Some(gardee) = gardee else {
        return Rendu::Rien;
    };
    if actif == Actif::Ailleurs || gardee.commande.liste() != liste {
        return Rendu::Rien;
    }
    if lues.iter().any(|ligne| ligne.idee == gardee.idee) {
        Rendu::Ligne(gardee.clone())
    } else {
        Rendu::Liste
    }
}

/// La cible du focus que rend `rendu` dans la liste `liste` : la ligne, sans faire défiler la
/// liste (`Defilement::Garde`), ou la liste ; aucune pour `Rendu::Rien`. Rendu à une ligne cachée
/// dans la marge, un focus qui défilerait ramènerait la vue vers elle, et la molette ne passerait
/// plus au-delà (R35).
fn cible_du_rendu(rendu: Rendu, liste: Liste) -> Option<Cible> {
    match rendu {
        Rendu::Rien => None,
        Rendu::Ligne(gardee) => Some(Cible::Ligne {
            gardee,
            defilement: Defilement::Garde,
        }),
        Rendu::Liste => Some(match liste {
            Liste::Boite => Cible::Liste,
            Liste::Retirees => Cible::ListeDesRetirees,
        }),
    }
}

/// Où va le focus quand le volet de l'idée `idee` se ferme : au bouton de sa ligne, si elle est
/// parmi les lignes lues `lues`, et la liste défile jusqu'à elle ; sinon à la liste.
fn cible_apres_le_volet(idee: &str, lues: &[LigneDeLaBoite]) -> Cible {
    if lues.iter().any(|ligne| ligne.idee == idee) {
        Cible::Ligne {
            gardee: Gardee {
                idee: idee.to_string(),
                commande: Commande::Choisir,
            },
            defilement: Defilement::VersLaLigne,
        }
    } else {
        Cible::Liste
    }
}

/// Le défilement à remettre à la liste après le `focus()` d'une ligne : celui d'avant, `avant`,
/// si la cible garde le défilement et que `focus()` l'a changé en `apres` ; rien sinon.
fn defilement_a_remettre(defilement: Defilement, avant: i32, apres: i32) -> Option<i32> {
    (defilement == Defilement::Garde && apres != avant).then_some(avant)
}

/// Ce que le volet offre d'une idée : les gestes de tri (compléter, poser et ôter une étiquette,
/// délier), et le libellé de son dernier bouton.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Offerts {
    trier: bool,
    dernier: &'static str,
}

/// D'une idée retirée, le volet n'offre que « Rétablir » (et « Fermer ») : le noyau refuserait
/// tout geste de tri qui la touche. D'une autre, les gestes de tri et « Retirer ».
fn ce_que_le_volet_offre(retiree: bool) -> Offerts {
    if retiree {
        Offerts {
            trier: false,
            dernier: "Rétablir",
        }
    } else {
        Offerts {
            trier: true,
            dernier: "Retirer",
        }
    }
}

/// Prend le drapeau `en_vol` d'un geste : faux s'il est déjà pris, et la seconde activation
/// s'ignore ; vrai sinon, et le drapeau reste pris jusqu'à la relecture d'après le geste.
fn prendre(en_vol: StoredValue<bool>) -> bool {
    if en_vol.get_value() {
        return false;
    }
    en_vol.set_value(true);
    true
}

/// Ce qui a le focus dans le document (`Actif`).
fn actif() -> Actif {
    let document = document();
    let ailleurs = document.active_element().is_some_and(|element| {
        !document.body().is_some_and(|body| {
            let body: &Node = body.as_ref();
            element.is_same_node(Some(body))
        })
    });
    if ailleurs {
        Actif::Ailleurs
    } else {
        Actif::Absent
    }
}

/// La fenêtre de la liste `liste` posée : si le focus s'est perdu, il revient à la ligne gardée,
/// ou à la liste (`focus_a_rendre`), par `focus`.
fn rendre_le_focus_perdu(boite: EtatDeLaBoite, focus: RwSignal<Option<Cible>>, liste: Liste) {
    let actif = actif();
    let rendu = boite.gardee.with_value(|gardee| {
        let decider = |lues: &[LigneDeLaBoite]| focus_a_rendre(gardee.as_ref(), liste, lues, actif);
        match liste {
            Liste::Boite => boite.lue.with_untracked(|lue| {
                decider(
                    lue.as_ref()
                        .map_or(&[][..], |lue| lue.vue.lignes.as_slice()),
                )
            }),
            Liste::Retirees => boite.retirees.with_untracked(|lues| {
                decider(
                    lues.as_ref()
                        .map_or(&[][..], |lues| lues.vue.lignes.as_slice()),
                )
            }),
        }
    });
    if let Some(cible) = cible_du_rendu(rendu, liste) {
        focus.set(Some(cible));
    }
}

/// Ce que fait le défilement de sa liste quand une ligne reçoit le focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Defilement {
    /// La liste garde son défilement : le focus rendu après une perte ne la fait pas bouger.
    Garde,
    /// La liste défile jusqu'à la ligne, comme `focus()` le fait.
    VersLaLigne,
}

/// Où rendre le focus, dès que la cible est montée dans le document : un élément détaché, celui
/// d'un volet fermé, ne le reçoit pas. Une commande d'une ligne, chaque ligne la résout.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Cible {
    Liste,
    ChampDeNote,
    /// Le champ « Nom » des requêtes enregistrées, où va le focus après une suppression.
    ChampDeNom,
    ListeDesRetirees,
    BoutonDesRetirees,
    BoutonDuJournal,
    TitreDeLIdee,
    TitreDesRetirees,
    TitreDuJournal,
    Ligne {
        gardee: Gardee,
        defilement: Defilement,
    },
}

/// La cible qu'on atteint vraiment : tant que la page du fonds vide se montre, la liste est cachée
/// et ne peut recevoir le focus ; il va au champ « Noter une idée ». Toute autre cible reste.
fn cible_repliee(cible: Cible, fonds_vide: bool) -> Cible {
    match cible {
        Cible::Liste if fonds_vide => Cible::ChampDeNote,
        autre => autre,
    }
}

/// Où va le focus quand la page du fonds vide paraît ou s'en va (`vide`) : paraissant, elle cache
/// la liste, qui ne peut plus garder le focus ; s'il est sur elle (`liste_a_le_focus`), il va au
/// champ « Noter une idée », avant que le navigateur ne le perde. C'est le cas de la dernière idée
/// retirée : son geste a rendu le focus à la liste avant la relecture qui vide le fonds
/// (`cible_repliee` lisait encore l'ancienne fenêtre). Le fonds non vide, ou le focus ailleurs :
/// il reste où il est.
fn cible_quand_le_fonds_se_vide(vide: bool, liste_a_le_focus: bool) -> Option<Cible> {
    (vide && liste_a_le_focus).then_some(Cible::ChampDeNote)
}

/// Aucune idée n'est interrogée dans la fenêtre lue `lue` : la page du fonds vide se montre
/// (`page_du_fonds_vide`) ; rien de lu, elle ne se montre pas.
fn fonds_vide_dans(lue: Option<&BoiteLue>) -> bool {
    lue.is_some_and(|lue| lue.vue.assiette_du_compte.considerees == 0)
}

/// Ce que partagent les vues de la boîte : l'état de l'établi, celui de la boîte, le focus à
/// rendre, et les repères de ses éléments montés.
#[derive(Clone, Copy)]
struct Ecran {
    etat: Etat,
    boite: EtatDeLaBoite,
    focus: RwSignal<Option<Cible>>,
    champ_de_note: NodeRef<html::Input>,
    champ_de_nom: NodeRef<html::Input>,
    liste: NodeRef<html::Div>,
    etalon: NodeRef<html::Div>,
    liste_des_retirees: NodeRef<html::Div>,
    bouton_des_retirees: NodeRef<html::Button>,
    liste_du_journal: NodeRef<html::Div>,
    bouton_du_journal: NodeRef<html::Button>,
    titre_de_l_idee: NodeRef<html::H2>,
    titre_des_retirees: NodeRef<html::H2>,
    titre_du_journal: NodeRef<html::H2>,
}

impl Ecran {
    /// Aucune idée n'est interrogée : la page du fonds vide se montre (`fonds_vide_dans`).
    fn fonds_vide(self) -> bool {
        self.boite
            .lue
            .with_untracked(|lue| fonds_vide_dans(lue.as_ref()))
    }

    /// Vrai si la liste de la boîte, montée, a le focus.
    fn la_liste_a_le_focus(self) -> bool {
        let Some(liste) = self.liste.get_untracked() else {
            return false;
        };
        let liste: &Node = liste.as_ref();
        document()
            .active_element()
            .is_some_and(|actif| actif.is_same_node(Some(liste)))
    }

    /// La fenêtre que demande la liste montée `liste` (`fenetre_a_demander`), d'après la dernière
    /// demandée (`demandes`) et le total lu ; rien sans mesure. Le défilement mesuré s'y note.
    fn a_demander(
        self,
        liste: NodeRef<html::Div>,
        demandes: Demandes,
        total: Option<u64>,
        forcer: bool,
    ) -> Option<Fenetre> {
        let (haut, vue, ligne) = mesures(liste, self.etalon)?;
        demandes.defilement.set_value(haut);
        demandes.derniere.with_value(|derniere| {
            fenetre_a_demander(derniere.as_ref(), haut, vue, ligne, total, forcer)
        })
    }

    /// Lit la fenêtre de la boîte que demande son défilement (`a_demander`), réponse de la requête
    /// en cours, lue à la demande.
    fn demander_la_boite(self, forcer: bool) {
        let total = self
            .boite
            .lue
            .with_untracked(|lue| lue.as_ref().map(|lue| lue.vue.retenues));
        if let Some(fenetre) = self.a_demander(self.liste, self.boite.demandes, total, forcer) {
            let requete = requete_en_cours(self.etat);
            leptos::task::spawn_local(lire_la_boite(
                self.etat, self.boite, self.focus, fenetre, requete,
            ));
        }
    }

    /// Lit la fenêtre des idées retirées que demande leur défilement (`a_demander`).
    fn demander_les_retirees(self, forcer: bool) {
        let total = self
            .boite
            .retirees
            .with_untracked(|lues| lues.as_ref().map(|lues| lues.vue.total));
        let demandes = self.boite.demandes_des_retirees;
        if let Some(fenetre) = self.a_demander(self.liste_des_retirees, demandes, total, forcer) {
            leptos::task::spawn_local(lire_les_retirees(
                self.etat, self.boite, self.focus, fenetre,
            ));
        }
    }

    /// Lit la fenêtre du journal que demande son défilement (`a_demander`).
    fn demander_le_journal(self, forcer: bool) {
        let total = self
            .boite
            .journal
            .with_untracked(|lu| lu.as_ref().map(|lu| lu.vue.total));
        let demandes = self.boite.demandes_du_journal;
        if let Some(fenetre) = self.a_demander(self.liste_du_journal, demandes, total, forcer) {
            leptos::task::spawn_local(lire_le_journal(self.etat, self.boite, fenetre));
        }
    }

    /// Le focus venu du dehors de la liste `liste` (`ev`) sur sa ligne de rang `rang` : si
    /// `ligne_d_entree` le cède à une ligne entièrement vue, la liste reprend le défilement noté à
    /// sa dernière mesure, d'avant que `focus()` ne la fasse défiler, et le focus va à cette ligne
    /// sans la faire défiler. Le focus venu de la liste elle-même, la région qui le prend avant ses
    /// lignes, vient du dehors, comme celui qui revient à l'application (Alt+Tab) sur une ligne
    /// que la molette a cachée. Le focus que l'écran rend lui-même à une ligne (`Cible::Ligne`, en
    /// cours) ne se détourne pas.
    fn entrer_dans_la_liste(self, ev: &leptos::ev::FocusEvent, liste: Liste, rang: u64) {
        if self
            .focus
            .with_untracked(|cible| matches!(cible, Some(Cible::Ligne { .. })))
        {
            return;
        }
        let (conteneur, demandes) = match liste {
            Liste::Boite => (self.liste, self.boite.demandes),
            Liste::Retirees => (self.liste_des_retirees, self.boite.demandes_des_retirees),
        };
        let (Some(conteneur), Some(etalon)) =
            (conteneur.get_untracked(), self.etalon.get_untracked())
        else {
            return;
        };
        let region: &Node = conteneur.as_ref();
        let du_dehors = ev
            .related_target()
            .and_then(|venu| venu.dyn_into::<Node>().ok())
            .is_none_or(|venu| venu.is_same_node(Some(region)) || !region.contains(Some(&venu)));
        if !du_dehors {
            return;
        }
        let defilement = demandes.defilement.get_value();
        let Some((bord, cible)) = ligne_d_entree(
            rang,
            defilement,
            f64::from(conteneur.client_height()),
            f64::from(etalon.client_height()) / LIGNES_DE_L_ETALON,
        ) else {
            return;
        };
        let idee = match liste {
            Liste::Boite => self.boite.lue.with_untracked(|lue| {
                lue.as_ref()
                    .and_then(|lue| idee_de_rang(&lue.fenetre, &lue.vue.lignes, cible))
            }),
            Liste::Retirees => self.boite.retirees.with_untracked(|lues| {
                lues.as_ref()
                    .and_then(|lues| idee_de_rang(&lues.fenetre, &lues.vue.lignes, cible))
            }),
        };
        let Some(idee) = idee else {
            return;
        };
        let commande = match (liste, bord) {
            (Liste::Boite, Bord::Haut) => Commande::Choisir,
            (Liste::Boite, Bord::Bas) => Commande::Cocher,
            (Liste::Retirees, _) => Commande::Retablir,
        };
        // Le défilement noté vient de `scroll_top`, en pixels entiers.
        conteneur.set_scroll_top(defilement as i32);
        self.focus.set(Some(Cible::Ligne {
            gardee: Gardee { idee, commande },
            defilement: Defilement::Garde,
        }));
    }

    /// Un autre ordre, ou un réglage de la requête en cours changé, son texte ou un critère : la
    /// liste revient au début, et sa fenêtre se relit.
    fn au_debut(self) {
        if let Some(liste) = self.liste.get_untracked() {
            liste.set_scroll_top(0);
        }
        self.demander_la_boite(true);
    }

    /// Un geste : la commande `nom`, puis son issue au bandeau (`avertissement` tire celui d'un
    /// geste fait), puis `apres`, qui reçoit vrai si le geste est fait, puis la relecture de l'état
    /// du fonds et des fenêtres (`relire`). Le drapeau `en_vol` pris (`prendre`), une seconde
    /// activation s'ignore jusqu'à la fin de la relecture.
    fn geste<R: DeserializeOwned + 'static>(
        self,
        nom: &'static str,
        commande: Option<serde_json::Value>,
        avertissement: fn(R) -> Option<String>,
        en_vol: StoredValue<bool>,
        apres: impl FnOnce(bool) + 'static,
    ) {
        if !prendre(en_vol) {
            return;
        }
        leptos::task::spawn_local(async move {
            let issue = natif::invoquer::<R>(nom, commande).await.map(avertissement);
            self.etat.apres_un_geste(&issue);
            apres(issue.is_ok());
            relire(self.etat, self.boite, self.focus).await;
            en_vol.set_value(false);
        });
    }

    /// Un geste qui rend `Faite` : un geste de tri, ou l'annulation, sous le drapeau des gestes.
    fn faire(
        self,
        nom: &'static str,
        commande: Option<serde_json::Value>,
        apres: impl FnOnce(bool) + 'static,
    ) {
        let en_vol = self.boite.geste_en_vol;
        self.geste::<Faite>(nom, commande, |faite| faite.avertissement, en_vol, apres);
    }

    /// Note l'idée `noter`, sous le drapeau des notes. Faite, elle se conclut
    /// (`conclure_la_note`).
    fn noter(self, noter: Noter) {
        let partie = noter.texte.clone();
        self.geste::<Notee>(
            NOTER,
            argument(&noter),
            |notee| notee.avertissement,
            self.boite.note_en_cours,
            move |faite| {
                if faite {
                    self.conclure_la_note(&partie);
                }
            },
        );
    }

    /// La note `partie` faite : le champ « Noter une idée » se vide, s'il la porte encore. La
    /// requête en cours reste telle quelle, et la relecture qui suit la note la montre dans la
    /// liste si elle la satisfait (point de revue 1).
    fn conclure_la_note(self, partie: &str) {
        self.boite.note.update(|champ| {
            if champ.trim() == partie {
                champ.clear();
            }
        });
    }

    /// Un geste d'une requête enregistrée : la commande `nom`, puis son issue au bandeau, puis ce
    /// qui suit (`apres_un_geste_de_requete`), dont le focus, puis la relecture de la liste des
    /// requêtes, faite ou refusée : une requête disparue entre-temps ne reste pas offerte.
    fn geste_de_requete(
        self,
        nom: &'static str,
        commande: Option<serde_json::Value>,
        geste: GesteDeRequete,
    ) {
        let (etat, boite, focus) = (self.etat, self.boite, self.focus);
        self.geste::<Faite>(
            nom,
            commande,
            |faite| faite.avertissement,
            boite.requete_en_vol,
            move |fait| {
                if let Some(cible) = boite.apres_un_geste_de_requete(&geste, fait) {
                    focus.set(Some(cible));
                }
                leptos::task::spawn_local(lire_les_requetes(etat, boite));
            },
        );
    }

    /// Enregistre la requête de l'écran sous le nom du champ « Nom ». Un nom de blancs, ou une
    /// requête sans critère, ne part pas (`enregistrer_possible`) ; les autres refus sont ceux des
    /// services, au bandeau.
    fn enregistrer_la_requete(self) {
        let requete = requete_en_cours(self.etat);
        let nom = self.boite.nom_de_la_requete.get_untracked();
        if !enregistrer_possible(&requete, &nom) {
            return;
        }
        let geste = GesteDeRequete::Enregistrer { nom: nom.clone() };
        self.geste_de_requete(
            ENREGISTRER_UNE_REQUETE,
            argument(&Enregistrer { nom, requete }),
            geste,
        );
    }

    /// Renomme la requête enregistrée choisie du nom du champ « Nom » ; un nom vide ou inchangé est
    /// un refus des services, que le bandeau dit.
    fn renommer_la_requete(self) {
        let Some(requete) = self.boite.requete_choisie.get_untracked() else {
            return;
        };
        let nom = self.boite.nom_de_la_requete.get_untracked();
        self.geste_de_requete(
            RENOMMER_UNE_REQUETE,
            argument(&Renommer { requete, nom }),
            GesteDeRequete::Renommer,
        );
    }

    /// Supprime la requête enregistrée choisie, sans confirmation (question 19) : ses critères, que
    /// la requête de l'écran porte peut-être, restent à l'écran.
    fn supprimer_la_requete(self) {
        let Some(requete) = self.boite.requete_choisie.get_untracked() else {
            return;
        };
        self.geste_de_requete(
            SUPPRIMER_UNE_REQUETE,
            argument(&Supprimer {
                requete: requete.clone(),
            }),
            GesteDeRequete::Supprimer { requete },
        );
    }

    /// Annule le dernier geste de tri ; sans geste à annuler, le refus le dit.
    fn annuler(self) {
        self.faire(ANNULER, None, |_| {});
    }

    /// Relie les deux idées cochées, chacune datée au décalage local de sa naissance
    /// (`relier_les_cochees`) ; faite, aucune ne reste cochée.
    fn relier(self) {
        let Some(relier) = self
            .boite
            .cochees
            .with_untracked(|cochees| relier_les_cochees(cochees, natif::decalage_a))
        else {
            return;
        };
        let commande = argument(&relier);
        let cochees = self.boite.cochees;
        self.faire(RELIER, commande, move |fait| {
            if fait {
                cochees.set(Vec::new());
            }
        });
    }

    /// Ouvre l'idée `id` dans le volet (`montrer_l_idee`), et y porte le focus.
    fn ouvrir(self, id: String) {
        if self.boite.montrer_l_idee(&id) {
            leptos::task::spawn_local(lire_l_idee(self.etat, self.boite, self.focus, id));
        }
        self.focus.set(Some(Cible::TitreDeLIdee));
    }

    /// Ferme le volet ; le focus va à `cible`.
    fn fermer_le_volet(self, cible: Cible) {
        self.boite.volet.set(Volet::Aucun);
        self.focus.set(Some(cible));
    }

    /// Ferme le volet de l'idée ; le focus va au bouton de la ligne qui l'a ouvert, si elle est
    /// lue, sinon à la liste (`cible_apres_le_volet`).
    fn fermer_l_idee(self) {
        self.fermer_le_volet(self.cible_de_fermeture().unwrap_or(Cible::Liste));
    }

    /// Où va le focus à la fermeture du volet ouvert (`cible_de_fermeture`), `None` sans volet.
    fn cible_de_fermeture(self) -> Option<Cible> {
        let volet = self.boite.volet.get_untracked();
        self.boite.lue.with_untracked(|lue| {
            cible_de_fermeture(
                &volet,
                lue.as_ref()
                    .map_or(&[][..], |lue| lue.vue.lignes.as_slice()),
            )
        })
    }

    /// Échap ferme le volet ouvert et rend le focus comme « Fermer » ; sans volet ouvert, rien ne
    /// se ferme.
    fn fermer_par_echap(self) {
        if let Some(cible) = self.cible_de_fermeture() {
            self.fermer_le_volet(cible);
        }
    }

    /// La commande `commande` de la ligne de l'idée `idee` a pris le focus : elle est la ligne
    /// gardée.
    fn garder(self, idee: &str, commande: Commande) {
        self.boite.gardee.set_value(Some(Gardee {
            idee: idee.to_string(),
            commande,
        }));
    }

    /// Ouvre le volet des idées retirées, qui prend le focus, ou le ferme.
    fn basculer_les_retirees(self) {
        if self.boite.volet.with_untracked(|v| *v == Volet::Retirees) {
            self.boite.volet.set(Volet::Aucun);
        } else {
            self.boite.retirees.set(None);
            self.boite.volet.set(Volet::Retirees);
            self.focus.set(Some(Cible::TitreDesRetirees));
        }
    }

    /// Ouvre le volet du journal du fonds, qui prend le focus, ou le ferme.
    fn basculer_le_journal(self) {
        if self.boite.volet.with_untracked(|v| *v == Volet::Journal) {
            self.boite.volet.set(Volet::Aucun);
        } else {
            self.boite.journal.set(None);
            self.boite.volet.set(Volet::Journal);
            self.focus.set(Some(Cible::TitreDuJournal));
        }
    }

    /// L'identifiant de l'idée du volet, s'il en montre une.
    fn idee_du_volet(self) -> Option<String> {
        self.boite.volet.with_untracked(|volet| match volet {
            Volet::Idee(id) => Some(id.clone()),
            _ => None,
        })
    }

    /// La naissance de l'idée `idee`, si c'est l'idée lue du volet ; pas encore lue, `None`.
    fn naissance_de_l_idee_du_volet(self, idee: &str) -> Option<i64> {
        self.boite.idee.with_untracked(|lue| {
            lue.as_ref()
                .filter(|lue| lue.vue.idee == idee)
                .map(|lue| lue.vue.le)
        })
    }

    /// Le décalage local qui date l'idée `idee` du volet dans le message d'un geste : celui de
    /// l'instant de sa naissance (`natif::decalage_a`) ; pas encore lue, celui du geste
    /// (`natif::decalage_du_geste`).
    fn decalage_de_l_idee_du_volet(self, idee: &str) -> i32 {
        match self.naissance_de_l_idee_du_volet(idee) {
            Some(le) => natif::decalage_a(le),
            None => natif::decalage_du_geste(),
        }
    }

    /// Complète le texte de l'idée du volet par le champ « Texte complété » ; fait, le champ
    /// repartira du texte relu.
    fn completer(self) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let commande = argument(&Completer {
            decalage_minutes: self.decalage_de_l_idee_du_volet(&idee),
            idee,
            texte: self.boite.brouillon.get_untracked(),
        });
        let brouillon_de = self.boite.brouillon_de;
        self.faire(COMPLETER, commande, move |fait| {
            if fait {
                brouillon_de.set_value(None);
            }
        });
    }

    /// Pose sur l'idée du volet l'étiquette du champ, ses blancs et un `#` de tête ôtés
    /// (`etiquette_saisie`) ; posée, le champ se vide.
    fn poser_l_etiquette(self) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let champ = self.boite.etiquette;
        let commande = argument(&Etiqueter {
            decalage_minutes: self.decalage_de_l_idee_du_volet(&idee),
            idee,
            etiquette: etiquette_saisie(&champ.get_untracked()),
        });
        self.faire(POSER_UNE_ETIQUETTE, commande, move |fait| {
            if fait {
                champ.set(String::new());
            }
        });
    }

    /// Ôte l'étiquette `etiquette` de l'idée du volet ; son « Ôter » parti, le focus va au titre du
    /// volet.
    fn oter(self, etiquette: String) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let commande = argument(&Etiqueter {
            decalage_minutes: self.decalage_de_l_idee_du_volet(&idee),
            idee,
            etiquette,
        });
        let focus = self.focus;
        self.faire(OTER_UNE_ETIQUETTE, commande, move |fait| {
            if fait {
                focus.set(Some(Cible::TitreDeLIdee));
            }
        });
    }

    /// Délie l'idée du volet de l'idée `autre`, née à `le_de_l_autre` si elle s'est lue, chacune
    /// datée au décalage local de sa naissance (`deliaison`) ; son « Délier » parti, le focus va
    /// au titre du volet.
    fn delier(self, autre: String, le_de_l_autre: Option<i64>) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let le_de_l_idee = self.naissance_de_l_idee_du_volet(&idee);
        let commande = argument(&deliaison(
            (idee, le_de_l_idee),
            (autre, le_de_l_autre),
            natif::decalage_a,
            natif::decalage_du_geste(),
        ));
        let focus = self.focus;
        self.faire(DELIER, commande, move |fait| {
            if fait {
                focus.set(Some(Cible::TitreDeLIdee));
            }
        });
    }

    /// Retire l'idée du volet, ou la rétablit si elle est retirée. Retirée, elle quitte les lignes
    /// cochées, le volet se ferme, et le focus va à la liste.
    fn retirer_ou_retablir(self) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let retiree = self
            .boite
            .idee
            .with_untracked(|i| i.as_ref().is_some_and(|i| i.vue.retiree.is_some()));
        let commande = argument(&Viser {
            idee: idee.clone(),
            decalage_minutes: self.decalage_de_l_idee_du_volet(&idee),
        });
        if retiree {
            self.faire(RETABLIR, commande, |_| {});
            return;
        }
        let (boite, focus) = (self.boite, self.focus);
        self.faire(RETIRER, commande, move |fait| {
            if fait {
                boite
                    .cochees
                    .update(|cochees| cochees.retain(|(c, _)| *c != idee));
                boite.volet.set(Volet::Aucun);
                focus.set(Some(Cible::Liste));
            }
        });
    }

    /// Rétablit l'idée retirée `idee`, née à `le`, datée au décalage local de sa naissance ; sa
    /// ligne partie, le focus va à la liste des idées retirées.
    fn retablir(self, idee: String, le: i64) {
        let commande = argument(&Viser {
            idee,
            decalage_minutes: natif::decalage_a(le),
        });
        let focus = self.focus;
        self.faire(RETABLIR, commande, move |fait| {
            if fait {
                focus.set(Some(Cible::ListeDesRetirees));
            }
        });
    }
}

/// Les choix en ligne d'un critère de la requête en cours, de groupe `groupe`, un par libellé de
/// son tableau `tableau` (`CONTENUS`, `DUREES`, `ETATS`, `PROVENANCES`), dans son ordre : le choix
/// fait écrit son rang dans `rang`, un signal de `Etat::requete`.
fn choix_d_un_critere<T>(
    groupe: &'static str,
    tableau: &'static [(Option<T>, &'static str)],
    rang: RwSignal<usize>,
) -> impl IntoView {
    tableau
        .iter()
        .enumerate()
        .map(|(valeur, (_, libelle))| {
            choix(
                groupe,
                valeur,
                libelle.to_string(),
                None,
                rang.into(),
                move |choisie| rang.set(choisie),
            )
        })
        .collect_view()
}

/// Un champ de date de « Date : », libellé `libelle` : la frappe écrit sa valeur dans `valeur`,
/// un signal de `Etat::requete`, « AAAA-MM-JJ », ou vide pour un champ vide ou incomplet ; et il
/// prend toute autre valeur que le signal reçoit (une requête posée). Un champ qui porte déjà la
/// valeur du signal ne se réécrit pas : incomplet, il vide la requête de son jour sans perdre ce
/// qui y est tapé. Son `max`, le 31 décembre 9999, borne l'année à quatre chiffres : une année à
/// cinq chiffres ne passerait pas l'aller-retour ISO, et la requête perdrait sa borne sans le dire.
fn champ_de_date(libelle: &'static str, valeur: RwSignal<String>) -> impl IntoView {
    let champ = NodeRef::<html::Input>::new();
    Effect::new(move |_| {
        let voulue = valeur.get();
        if let Some(champ) = champ.get()
            && champ.value() != voulue
        {
            champ.set_value(&voulue);
        }
    });
    view! {
        <label class="choix">
            {libelle}
            <input
                type="date"
                max="9999-12-31"
                node_ref=champ
                on:input=move |ev| valeur.set(event_target_value(&ev))
            />
        </label>
    }
}

/// La boîte d'idées. Montée, elle lit la fenêtre que sa liste montre, réponse de la requête en
/// cours ; un autre ordre, ou un réglage de la requête changé, son texte dans l'en-tête ou un
/// critère, la ramène au début. La liste suit son défilement et la taille de la fenêtre de
/// l'application, jamais une minuterie (I18) : chaque ligne a la hauteur fixe `--ligne`, que
/// l'étalon mesure ; la piste a la hauteur de toutes les lignes, et seules celles de la fenêtre lue
/// s'y posent, à leur rang. Chaque contrôle est un bouton, une case ou un champ, nommé ; le focus
/// suit les volets (`Cible`), entre dans une liste sur une ligne entièrement vue
/// (`entrer_dans_la_liste`), et revient à la ligne qui l'a perdu quand la fenêtre se pose
/// (`rendre_le_focus_perdu`).
#[component]
pub fn BoiteDIdees(etat: Etat, boite: EtatDeLaBoite) -> impl IntoView {
    let ecran = Ecran {
        etat,
        boite,
        focus: RwSignal::new(None),
        champ_de_note: NodeRef::new(),
        champ_de_nom: NodeRef::new(),
        liste: NodeRef::new(),
        etalon: NodeRef::new(),
        liste_des_retirees: NodeRef::new(),
        bouton_des_retirees: NodeRef::new(),
        liste_du_journal: NodeRef::new(),
        bouton_du_journal: NodeRef::new(),
        titre_de_l_idee: NodeRef::new(),
        titre_des_retirees: NodeRef::new(),
        titre_du_journal: NodeRef::new(),
    };
    // Montés, la liste et l'étalon donnent la fenêtre à lire. Chaque changement de la requête en
    // cours, son texte dans l'en-tête ou un critère, ramène la liste au début et la relit
    // (`au_debut`) : une demande par frappe, dont seule la dernière réponse compte.
    Effect::new(move |_| {
        let _suivie = etat.requete.requete();
        if ecran.liste.get().is_some() && ecran.etalon.get().is_some() {
            ecran.au_debut();
        }
    });
    // Les requêtes enregistrées se lisent à l'ouverture de la boîte.
    leptos::task::spawn_local(lire_les_requetes(etat, boite));
    // Le focus demandé va à sa cible, dès qu'elle est montée dans le document.
    Effect::new(move |_| {
        let Some(cible) = ecran.focus.get() else {
            return;
        };
        let element: Option<HtmlElement> = match cible_repliee(cible, ecran.fonds_vide()) {
            Cible::Liste => ecran.liste.get().map(Into::into),
            Cible::ChampDeNote => ecran.champ_de_note.get().map(Into::into),
            Cible::ChampDeNom => ecran.champ_de_nom.get().map(Into::into),
            Cible::ListeDesRetirees => ecran.liste_des_retirees.get().map(Into::into),
            Cible::BoutonDesRetirees => ecran.bouton_des_retirees.get().map(Into::into),
            Cible::BoutonDuJournal => ecran.bouton_du_journal.get().map(Into::into),
            Cible::TitreDeLIdee => ecran.titre_de_l_idee.get().map(Into::into),
            Cible::TitreDesRetirees => ecran.titre_des_retirees.get().map(Into::into),
            Cible::TitreDuJournal => ecran.titre_du_journal.get().map(Into::into),
            // Chaque ligne résout la sienne (`focus_de_la_ligne`).
            Cible::Ligne { .. } => return,
        };
        if let Some(element) = element
            && element.is_connected()
        {
            let _ = element.focus();
            ecran.focus.set(None);
        }
    });
    // La page du fonds vide paraît ou s'en va. Paraissant, elle cache la liste : si la liste a le
    // focus, il va au champ « Noter une idée » (`cible_quand_le_fonds_se_vide`). L'effet ne suit
    // que ce passage, pas chaque fenêtre lue.
    let fonds_vide_lu = Memo::new(move |_| boite.lue.with(|lue| fonds_vide_dans(lue.as_ref())));
    Effect::new(move |_| {
        let vide = fonds_vide_lu.get();
        if let Some(cible) = cible_quand_le_fonds_se_vide(vide, ecran.la_liste_a_le_focus()) {
            ecran.focus.set(Some(cible));
        }
    });
    // `Ctrl+Z`, hors d'un champ de texte, annule le dernier geste de tri tant que la boîte se
    // montre ; une touche tenue ne le répète pas. Échap ferme le volet ouvert (`echap_ferme`), sauf
    // dans la recherche de l'en-tête, où le navigateur vide le champ, et pendant une composition.
    let ecouteur = window_event_listener(leptos::ev::keydown, move |ev| {
        if !ev.repeat() && ctrl_z_annule(&ev.key(), ev.ctrl_key(), dans_un_champ_de_texte(&ev)) {
            ev.prevent_default();
            ecran.annuler();
            return;
        }
        let id_de_la_cible = ev
            .target()
            .and_then(|cible| cible.dyn_into::<Element>().ok())
            .map(|element| element.id());
        let volet_ouvert = boite.volet.with_untracked(|volet| *volet != Volet::Aucun);
        if echap_ferme(
            &ev.key(),
            ev.is_composing(),
            volet_ouvert,
            est_la_recherche(id_de_la_cible.as_deref()),
        ) {
            ev.prevent_default();
            ecran.fermer_par_echap();
        }
    });
    // La fenêtre de l'application change de taille, la liste vue aussi : chaque liste montrée
    // demande la fenêtre de lignes qui la couvre, si la dernière demandée ne la couvre plus
    // (`fenetre_a_demander`).
    let redimensionnement = window_event_listener(leptos::ev::resize, move |_| {
        ecran.demander_la_boite(false);
        if boite
            .volet
            .with_untracked(|volet| *volet == Volet::Retirees)
        {
            ecran.demander_les_retirees(false);
        }
        if boite.volet.with_untracked(|volet| *volet == Volet::Journal) {
            ecran.demander_le_journal(false);
        }
    });
    on_cleanup(move || {
        ecouteur.remove();
        redimensionnement.remove();
        boite.gardee.set_value(None);
    });
    // Le focus pris par un élément de la boîte qui n'est pas d'une ligne : aucune ligne n'est plus
    // gardée.
    let hors_des_lignes = move |ev: leptos::ev::FocusEvent| {
        let dans_une_ligne = ev
            .target()
            .and_then(|cible| cible.dyn_into::<Element>().ok())
            .is_some_and(|element| element.closest(".ligne").ok().flatten().is_some());
        if !dans_une_ligne {
            boite.gardee.set_value(None);
        }
    };

    let noter_au_clavier = move |ev: leptos::ev::KeyboardEvent| {
        if !envoi_au_clavier(&ev.key(), ev.repeat(), ev.is_composing()) {
            return;
        }
        let champ = boite.note.get_untracked();
        if let Some(noter) = note_a_envoyer(&champ, &ev.key(), natif::decalage_du_geste()) {
            ev.prevent_default();
            ecran.noter(noter);
        }
    };
    let comptes = move || {
        boite.lue.with(|lue| {
            lue.as_ref().map(|lue| {
                let (non_reliees, assiette, _) =
                    au_noyau(lue.vue.non_reliees, &lue.vue.assiette_du_compte);
                (
                    texte_du_compte(non_reliees, assiette),
                    texte_de_la_requete(&lue.vue, &lue.requete),
                )
            })
        })
    };
    let ordres = ORDRES
        .iter()
        .enumerate()
        .map(|(rang, (_, libelle))| {
            choix(
                "ordre",
                rang,
                libelle.to_string(),
                None,
                boite.ordre.into(),
                move |rang| {
                    boite.ordre.set(rang);
                    ecran.au_debut();
                },
            )
        })
        .collect_view();
    // Les critères de la requête en cours : chacun change un signal de `Etat::requete`, et l'effet
    // du montage ramène la liste au début.
    let requete = etat.requete;
    let contenus = choix_d_un_critere("contenu", &CONTENUS, requete.contenu);
    let durees = choix_d_un_critere("duree", &DUREES, requete.duree);
    let etats = choix_d_un_critere("etat", &ETATS, requete.etat);
    let provenances = choix_d_un_critere("provenance", &PROVENANCES, requete.provenance);
    let etiquettes = move || {
        let du_fonds = boite.lue.with(|lue| {
            lue.as_ref()
                .map(|lue| lue.vue.etiquettes_du_fonds.clone())
                .unwrap_or_default()
        });
        requete
            .etiquette
            .with(|choisie| etiquettes_a_choisir(&du_fonds, choisie.as_deref()))
    };
    let toutes = move || requete.etiquette.with(Option::is_none);
    let est_choisie = move |nom: &str| {
        requete
            .etiquette
            .with(|choisie| choisie.as_deref() == Some(nom))
    };
    let deux_cochees = move || boite.cochees.with(|cochees| cochees.len() == 2);
    let annulation = move || {
        etat.etat_du_fonds
            .with(|fonds| libelle_d_annulation(fonds.as_ref().and_then(|f| f.a_annuler.as_deref())))
    };
    let enregistrement_possible =
        move || enregistrer_possible(&etat.requete.requete(), &boite.nom_de_la_requete.get());
    let une_requete_est_choisie = move || boite.requete_choisie.with(Option::is_some);
    let requetes_enregistrees = move || boite.requetes.get().unwrap_or_default();
    let constat_des_requetes_lues = move || {
        boite
            .requetes
            .with(|lues| constat_des_requetes(lues.as_deref()))
    };
    let retirees_ouvertes = move || boite.volet.with(|volet| *volet == Volet::Retirees);
    let journal_ouvert = move || boite.volet.with(|volet| *volet == Volet::Journal);
    let idee_ouverte = move || boite.volet.with(|volet| matches!(volet, Volet::Idee(_)));
    // La page du fonds vide : aucune idée interrogée (`page_du_fonds_vide`), lue au moins une fois.
    let fonds_vide = move || {
        let considerees = boite.lue.with(|lue| {
            lue.as_ref()
                .map(|lue| lue.vue.assiette_du_compte.considerees)
        })?;
        let lecture_seule = etat
            .etat_du_fonds
            .with(|fonds| fonds.as_ref().is_some_and(|f| f.lecture_seule.is_some()));
        page_du_fonds_vide(considerees, lecture_seule)
    };
    let total = move || {
        boite
            .lue
            .with(|lue| lue.as_ref().map_or(0, |lue| lue.vue.retenues))
    };
    let hauteur_de_la_piste = move || {
        let lignes = total();
        format!("calc({lignes} * var(--ligne))")
    };
    let haut_des_lignes = move || {
        let debut = boite
            .lue
            .with(|lue| lue.as_ref().map_or(0, |lue| lue.fenetre.debut));
        format!("calc({debut} * var(--ligne))")
    };
    let lignes = move || {
        boite.lue.with(|lue| {
            lue.as_ref()
                .map(|lue| lue.vue.lignes.clone())
                .unwrap_or_default()
        })
    };
    view! {
        <main class="boite" aria-label="Boîte d'idées" on:focusin=hors_des_lignes>
            <div class="etalon" aria-hidden="true">
                <div node_ref=ecran.etalon></div>
            </div>
            <div class="boite-haut">
                <label class="champ noter">
                    "Noter une idée"
                    <input
                        type="text"
                        node_ref=ecran.champ_de_note
                        prop:value=move || boite.note.get()
                        on:input=move |ev| boite.note.set(event_target_value(&ev))
                        on:keydown=noter_au_clavier
                    />
                </label>
                <div class="comptes" aria-live="polite">
                    <p>{move || comptes().map(|(compte, _)| compte)}</p>
                    <p>{move || comptes().and_then(|(_, reponse)| reponse)}</p>
                </div>
                <fieldset class="reglage">
                    <legend>"Ranger par :"</legend>
                    <div class="choix-en-ligne">{ordres}</div>
                </fieldset>
                <fieldset class="reglage">
                    <legend>"Contenu :"</legend>
                    <div class="choix-en-ligne">{contenus}</div>
                </fieldset>
                <fieldset class="reglage">
                    <legend>"Date :"</legend>
                    <div class="choix-en-ligne">
                        {champ_de_date("du", requete.du)}
                        {champ_de_date("au", requete.au)}
                    </div>
                </fieldset>
                <fieldset class="reglage">
                    <legend id="legende-de-l-etiquette">"Étiquette :"</legend>
                    <select
                        aria-labelledby="legende-de-l-etiquette"
                        on:change=move |ev| {
                            requete.etiquette.set(etiquette_du_choix(&event_target_value(&ev)))
                        }
                    >
                        <option value="" prop:selected=toutes>
                            "Toutes"
                        </option>
                        <For
                            each=etiquettes
                            key=|nom: &String| nom.clone()
                            children=move |nom: String| {
                                let choisie = {
                                    let nom = nom.clone();
                                    move || est_choisie(&nom)
                                };
                                view! {
                                    <option value=nom.clone() prop:selected=choisie>
                                        {etiquette_ecrite(&nom)}
                                    </option>
                                }
                            }
                        />
                    </select>
                </fieldset>
                <fieldset class="reglage">
                    <legend>"Durée :"</legend>
                    <div class="choix-en-ligne">{durees}</div>
                </fieldset>
                <fieldset class="reglage">
                    <legend>"État :"</legend>
                    <div class="choix-en-ligne">{etats}</div>
                </fieldset>
                <fieldset class="reglage">
                    <legend>"Provenance :"</legend>
                    <div class="choix-en-ligne">{provenances}</div>
                </fieldset>
                <section class="requetes-enregistrees" aria-labelledby="titre-des-requetes">
                    <h2 id="titre-des-requetes">"Requêtes enregistrées"</h2>
                    <p hidden=move || constat_des_requetes_lues().is_none()>
                        {constat_des_requetes_lues}
                    </p>
                    <ul class="requetes">
                        <For
                            each=requetes_enregistrees
                            key=|r: &RequeteEnregistree| {
                                format!("{}|{}|{}", r.id, r.nom, r.requete.is_some())
                            }
                            children=move |r: RequeteEnregistree| {
                                let (nom, note) = ce_que_dit_la_requete(&r);
                                let id = r.id.clone();
                                let id_de_la_note = format!("note-de-la-requete-{id}");
                                let id_de_la_precision = format!("precision-de-la-requete-{id}");
                                let lues_par_le_bouton = note
                                    .map(|_| format!("{id_de_la_note} {id_de_la_precision}"));
                                let choisie = move || {
                                    boite.requete_choisie.with(|c| c.as_deref() == Some(id.as_str()))
                                };
                                view! {
                                    <li>
                                        <button
                                            type="button"
                                            class="choisir"
                                            aria-current=move || choisie().then_some("true")
                                            aria-describedby=lues_par_le_bouton
                                            on:click=move |_| {
                                                boite.choisir_et_lancer(etat.requete, &r)
                                            }
                                        >
                                            {nom}
                                        </button>
                                        {note
                                            .map(|note| {
                                                view! {
                                                    <span class="note-de-la-requete" id=id_de_la_note.clone()>
                                                        {note}
                                                    </span>
                                                    <span
                                                        class="note-de-la-requete"
                                                        id=id_de_la_precision.clone()
                                                    >
                                                        {PRECISION_D_UNE_REQUETE_PLUS_RECENTE}
                                                    </span>
                                                }
                                            })}
                                    </li>
                                }
                            }
                        />
                    </ul>
                    <label class="champ">
                        "Nom"
                        <input
                            type="text"
                            node_ref=ecran.champ_de_nom
                            prop:value=move || boite.nom_de_la_requete.get()
                            on:input=move |ev| boite.nom_de_la_requete.set(event_target_value(&ev))
                        />
                    </label>
                    <div class="gestes">
                        <button
                            type="button"
                            aria-disabled=move || (!enregistrement_possible()).then_some("true")
                            aria-describedby=move || {
                                (!enregistrement_possible()).then_some("note-d-enregistrement")
                            }
                            on:click=move |_| ecran.enregistrer_la_requete()
                        >
                            "Enregistrer la requête"
                        </button>
                        <Show when=une_requete_est_choisie>
                            <button type="button" on:click=move |_| ecran.renommer_la_requete()>
                                "Renommer"
                            </button>
                            <button type="button" on:click=move |_| ecran.supprimer_la_requete()>
                                "Supprimer"
                            </button>
                        </Show>
                    </div>
                    <Show when=move || !enregistrement_possible()>
                        <p id="note-d-enregistrement">
                            "Une requête s'enregistre avec au moins un critère et un nom."
                        </p>
                    </Show>
                </section>
                <div class="gestes">
                    <button
                        type="button"
                        aria-disabled=move || (!deux_cochees()).then_some("true")
                        on:click=move |_| ecran.relier()
                    >
                        "Relier"
                    </button>
                    <button
                        type="button"
                        aria-keyshortcuts="Control+Z"
                        on:click=move |_| ecran.annuler()
                    >
                        {annulation}
                    </button>
                    <button
                        type="button"
                        node_ref=ecran.bouton_des_retirees
                        aria-expanded=move || retirees_ouvertes().to_string()
                        aria-controls=move || retirees_ouvertes().then_some("volet-des-retirees")
                        on:click=move |_| ecran.basculer_les_retirees()
                    >
                        "Idées retirées"
                    </button>
                    <button
                        type="button"
                        node_ref=ecran.bouton_du_journal
                        aria-expanded=move || journal_ouvert().to_string()
                        aria-controls=move || journal_ouvert().then_some("volet-du-journal")
                        on:click=move |_| ecran.basculer_le_journal()
                    >
                        "Journal du fonds"
                    </button>
                </div>
            </div>
            <div class="boite-corps">
                <div class="colonne-liste">
                    <div class="colonnes" aria-hidden="true">
                        <span class="cellules">
                            <span>"Date"</span>
                            <span>"Texte"</span>
                            <span>"Étiquettes"</span>
                            <span>"Durée"</span>
                            <span>"État"</span>
                            <span>"Provenance"</span>
                        </span>
                        <span class="cocher">"Relier"</span>
                    </div>
                    {move || {
                        fonds_vide()
                            .map(|[constat, suite]| {
                                view! {
                                    <div class="fonds-vide">
                                        <p>{constat}</p>
                                        <p>{suite}</p>
                                    </div>
                                }
                            })
                    }}
                    <div
                        class="liste"
                        class:cachee=move || fonds_vide().is_some()
                        tabindex="0"
                        role="region"
                        aria-label="Idées de la boîte"
                        node_ref=ecran.liste
                        on:scroll=move |_| ecran.demander_la_boite(false)
                    >
                        <div class="piste" style:height=hauteur_de_la_piste>
                            <div class="lignes" role="list" style:top=haut_des_lignes>
                                <ForEnumerate
                                    each=lignes
                                    key=cle_de_la_ligne
                                    children={move |
                                        index: ReadSignal<usize>,
                                        ligne: LigneDeLaBoite,
                                    | ligne_de_la_boite(ecran, index, ligne)}
                                />
                            </div>
                        </div>
                    </div>
                </div>
                <Show when=idee_ouverte>
                    <VoletDeLIdee ecran=ecran />
                </Show>
                <Show when=retirees_ouvertes>
                    <VoletDesRetirees ecran=ecran />
                </Show>
                <Show when=journal_ouvert>
                    <VoletDuJournal ecran=ecran />
                </Show>
            </div>
        </main>
    }
}

/// « 25 oct. 2026 · 02:30 · Idée du 25 octobre notée » : l'heure de l'entrée au décalage
/// `decalage` de son instant (`heure_du_journal`), « — » pour l'heure 0, puis son message.
fn ligne_du_journal(entree: &EntreeDuJournal, decalage: i32) -> String {
    format!(
        "{} · {}",
        heure_du_journal(entree.heure, decalage),
        entree.message
    )
}

/// « 3 entrées · empreinte 570cda » : le nombre d'entrées du journal, et les six premiers
/// caractères de son empreinte (`empreinte_courte`) ; « 1 entrée » au singulier, « 0 entrée » de
/// même.
fn entete_du_journal(total: u64, empreinte: &str) -> String {
    let mot = if total <= 1 { "entrée" } else { "entrées" };
    format!("{total} {mot} · empreinte {}", empreinte_courte(empreinte))
}

/// Une ligne de la boîte, de rang `index` dans la fenêtre lue : un bouton qui ouvre le volet de
/// l'idée et montre ses cases (la date, le texte coupé sur une ligne, les étiquettes, la durée,
/// l'état, la provenance), puis la case à cocher pour relier. Choisie, son bouton le dit
/// (`aria-current`), en gras.
fn ligne_de_la_boite(
    ecran: Ecran,
    index: ReadSignal<usize>,
    ligne: LigneDeLaBoite,
) -> impl IntoView {
    let boite = ecran.boite;
    let date = date_locale(ligne.le);
    let texte = texte_de_la_ligne(&ligne);
    let nom_de_la_case = format!("Cocher pour relier : {date} · {texte}");
    let nom_du_bouton = nom_de_la_ligne(&date, &ligne);
    let cellules = [
        date,
        texte,
        etiquettes_de_la_ligne(&ligne),
        duree_affichee(ligne.duree_ms),
        etat_de_la_ligne(&ligne).to_string(),
        provenance_de_la_ligne(&ligne).to_string(),
    ];
    let (id, le) = (ligne.idee, ligne.le);
    let (bouton, case) = (
        NodeRef::<html::Button>::new(),
        NodeRef::<html::Input>::new(),
    );
    focus_de_la_ligne(ecran, id.clone(), move |commande| match commande {
        Commande::Choisir => bouton.get().map(Into::into),
        Commande::Cocher => case.get().map(Into::into),
        Commande::Retablir => None,
    });
    let rang = move || {
        let debut = boite
            .lue
            .with(|lue| lue.as_ref().map_or(0, |lue| lue.fenetre.debut));
        (u64::from(debut) + index.get() as u64 + 1).to_string()
    };
    let total = move || {
        boite
            .lue
            .with(|lue| lue.as_ref().map_or(0, |lue| lue.vue.retenues))
            .to_string()
    };
    let choisie = {
        let id = id.clone();
        move || {
            boite
                .volet
                .with(|volet| matches!(volet, Volet::Idee(i) if *i == id))
        }
    };
    let cochee = {
        let id = id.clone();
        move || {
            boite
                .cochees
                .with(|cochees| cochees.iter().any(|(c, _)| *c == id))
        }
    };
    let ouvrir = {
        let id = id.clone();
        move |_| ecran.ouvrir(id.clone())
    };
    // Le rang de la ligne dans la liste, compté de zéro.
    let rang_dans_la_liste = move || {
        let debut = boite
            .lue
            .with_untracked(|lue| lue.as_ref().map_or(0, |lue| lue.fenetre.debut));
        u64::from(debut) + index.get_untracked() as u64
    };
    // La commande prise se garde ; venu du dehors sur une ligne cachée, le focus va ensuite à une
    // ligne vue, qui se garde à son tour.
    let bouton_pris = {
        let id = id.clone();
        move |ev: leptos::ev::FocusEvent| {
            ecran.garder(&id, Commande::Choisir);
            ecran.entrer_dans_la_liste(&ev, Liste::Boite, rang_dans_la_liste());
        }
    };
    let case_prise = {
        let id = id.clone();
        move |ev: leptos::ev::FocusEvent| {
            ecran.garder(&id, Commande::Cocher);
            ecran.entrer_dans_la_liste(&ev, Liste::Boite, rang_dans_la_liste());
        }
    };
    let cocher = move |ev: leptos::ev::Event| {
        let coche = event_target_checked(&ev);
        boite.cochees.update(|cochees| {
            cochees.retain(|(c, _)| *c != id);
            if coche {
                cochees.push((id.clone(), le));
            }
        });
    };
    view! {
        <div class="ligne" role="listitem" aria-posinset=rang aria-setsize=total>
            <button
                type="button"
                class="choisir cellules"
                aria-label=nom_du_bouton
                aria-current=move || choisie().then_some("true")
                node_ref=bouton
                on:focus=bouton_pris
                on:click=ouvrir
            >
                {cellules
                    .into_iter()
                    .map(|cellule| view! { <span class="cellule">{cellule}</span> })
                    .collect_view()}
            </button>
            <span class="cocher">
                <input
                    type="checkbox"
                    aria-label=nom_de_la_case
                    prop:checked=cochee
                    node_ref=case
                    on:focus=case_prise
                    on:change=cocher
                />
            </span>
        </div>
    }
}

/// Le focus demandé à une commande de la ligne de l'idée `idee` (`Cible::Ligne`) : `element` la
/// résout, et elle le prend dès qu'elle est montée dans le document. Si la cible garde le
/// défilement, celui que `focus()` a fait à sa liste se remet aussitôt (`defilement_a_remettre`),
/// dans la même exécution : l'événement de défilement qui en naît trouve la vue d'avant, que la
/// fenêtre posée couvre déjà.
fn focus_de_la_ligne(
    ecran: Ecran,
    idee: String,
    element: impl Fn(Commande) -> Option<HtmlElement> + 'static,
) {
    Effect::new(move |_| {
        let demande = ecran.focus.with(|cible| match cible {
            Some(Cible::Ligne { gardee, defilement }) if gardee.idee == idee => {
                element(gardee.commande)
                    .map(|element| (element, *defilement, gardee.commande.liste()))
            }
            _ => None,
        });
        if let Some((element, defilement, liste)) = demande
            && element.is_connected()
        {
            let conteneur = match liste {
                Liste::Boite => ecran.liste,
                Liste::Retirees => ecran.liste_des_retirees,
            }
            .get_untracked();
            let avant = conteneur.as_ref().map(|conteneur| conteneur.scroll_top());
            let _ = element.focus();
            if let (Some(conteneur), Some(avant)) = (conteneur, avant)
                && let Some(remis) =
                    defilement_a_remettre(defilement, avant, conteneur.scroll_top())
            {
                conteneur.set_scroll_top(remis);
            }
            ecran.focus.set(None);
        }
    });
}

/// Une idée retirée, de rang `index` dans la fenêtre lue : sa date, son texte coupé sur une ligne,
/// ses étiquettes, et « Rétablir ».
fn ligne_retiree(ecran: Ecran, index: ReadSignal<usize>, ligne: LigneDeLaBoite) -> impl IntoView {
    let boite = ecran.boite;
    let date = date_locale(ligne.le);
    let texte = texte_de_la_ligne(&ligne);
    let nom_du_bouton = format!("Rétablir : {date} · {texte}");
    let cellules = [date, texte, etiquettes_de_la_ligne(&ligne)];
    let (id, le) = (ligne.idee, ligne.le);
    let bouton = NodeRef::<html::Button>::new();
    focus_de_la_ligne(ecran, id.clone(), move |commande| match commande {
        Commande::Retablir => bouton.get().map(Into::into),
        Commande::Choisir | Commande::Cocher => None,
    });
    // Le rang de la ligne dans la liste, compté de zéro.
    let rang_dans_la_liste = move || {
        let debut = boite
            .retirees
            .with_untracked(|lues| lues.as_ref().map_or(0, |lues| lues.fenetre.debut));
        u64::from(debut) + index.get_untracked() as u64
    };
    // La commande prise se garde ; venu du dehors sur une ligne cachée, le focus va ensuite à une
    // ligne vue, qui se garde à son tour.
    let bouton_pris = {
        let id = id.clone();
        move |ev: leptos::ev::FocusEvent| {
            ecran.garder(&id, Commande::Retablir);
            ecran.entrer_dans_la_liste(&ev, Liste::Retirees, rang_dans_la_liste());
        }
    };
    let rang = move || {
        let debut = boite
            .retirees
            .with(|lues| lues.as_ref().map_or(0, |lues| lues.fenetre.debut));
        (u64::from(debut) + index.get() as u64 + 1).to_string()
    };
    let total = move || {
        boite
            .retirees
            .with(|lues| lues.as_ref().map_or(0, |lues| lues.vue.total))
            .to_string()
    };
    view! {
        <div class="ligne" role="listitem" aria-posinset=rang aria-setsize=total>
            <span class="cellules retiree">
                {cellules
                    .into_iter()
                    .map(|cellule| view! { <span class="cellule">{cellule}</span> })
                    .collect_view()}
            </span>
            <button
                type="button"
                class="retablir"
                aria-label=nom_du_bouton
                node_ref=bouton
                on:focus=bouton_pris
                on:click=move |_| ecran.retablir(id.clone(), le)
            >
                "Rétablir"
            </button>
        </div>
    }
}

/// Le volet de l'idée choisie : son titre, qui prend le focus à l'ouverture, « Fermer », le texte
/// entier, ce que l'idée est, en mots, « Texte complété » et « Compléter », ses étiquettes, chacune
/// avec « Ôter », « Étiquette » et « Poser l'étiquette », ses liens, chacun avec « Délier », et
/// « Retirer », ou « Rétablir » pour une idée retirée. Les champs et leurs boutons restent montés
/// quand l'idée se relit : le focus qu'ils portent y reste.
#[component]
fn VoletDeLIdee(ecran: Ecran) -> impl IntoView {
    let boite = ecran.boite;
    let titre = move || {
        boite.idee.with(|idee| match idee {
            Some(idee) => format!("Idée du {}", date_locale(idee.vue.le)),
            None => "Idée".to_string(),
        })
    };
    let texte = move || {
        boite.idee.with(|idee| {
            idee.as_ref()
                .map(|i| texte_ou_tiret(i.vue.texte.as_deref()))
        })
    };
    let ce_qu_elle_est = move || {
        boite
            .idee
            .with(|idee| idee.as_ref().map(|i| etat_de_l_idee(&i.vue)))
    };
    let etiquettes = move || {
        boite.idee.with(|idee| {
            idee.as_ref()
                .map(|i| i.vue.etiquettes.clone())
                .unwrap_or_default()
        })
    };
    let liens = move || {
        boite
            .idee
            .with(|idee| idee.as_ref().map(|i| i.liens.clone()).unwrap_or_default())
    };
    let offerts = move || {
        ce_que_le_volet_offre(
            boite
                .idee
                .with(|idee| idee.as_ref().is_some_and(|i| i.vue.retiree.is_some())),
        )
    };
    let trier = move || offerts().trier;
    let poser_au_clavier = move |ev: leptos::ev::KeyboardEvent| {
        if envoi_au_clavier(&ev.key(), ev.repeat(), ev.is_composing()) {
            ev.prevent_default();
            ecran.poser_l_etiquette();
        }
    };
    view! {
        <aside id="volet-de-l-idee" class="volet" aria-labelledby="titre-de-l-idee">
            <div class="volet-haut">
                <h2 id="titre-de-l-idee" tabindex="-1" node_ref=ecran.titre_de_l_idee>
                    {titre}
                </h2>
                <button
                    type="button"
                    aria-keyshortcuts="Escape"
                    on:click=move |_| ecran.fermer_l_idee()
                >
                    "Fermer"
                </button>
            </div>
            <p class="texte-entier">{texte}</p>
            <p>{ce_qu_elle_est}</p>
            <Show when=trier>
                <label class="champ">
                    "Texte complété"
                    <textarea
                        rows="3"
                        prop:value=move || boite.brouillon.get()
                        on:input=move |ev| boite.brouillon.set(event_target_value(&ev))
                    ></textarea>
                </label>
                <div>
                    <button type="button" on:click=move |_| ecran.completer()>
                        "Compléter"
                    </button>
                </div>
            </Show>
            <h3>"Étiquettes"</h3>
            <p hidden=move || !etiquettes().is_empty()>{CASE_VIDE}</p>
            <ul class="puces">
                <For
                    each=etiquettes
                    key=|etiquette: &String| etiquette.clone()
                    children=move |etiquette: String| {
                        let nom = format!("Ôter l'étiquette #{etiquette}");
                        let a_oter = etiquette.clone();
                        view! {
                            <li>
                                <span>"#" {etiquette}</span>
                                <Show when=trier>
                                    <button
                                        type="button"
                                        aria-label=nom.clone()
                                        on:click={
                                            let a_oter = a_oter.clone();
                                            move |_| ecran.oter(a_oter.clone())
                                        }
                                    >
                                        "Ôter"
                                    </button>
                                </Show>
                            </li>
                        }
                    }
                />
            </ul>
            <Show when=trier>
                <label class="champ">
                    "Étiquette"
                    <input
                        type="text"
                        prop:value=move || boite.etiquette.get()
                        on:input=move |ev| boite.etiquette.set(event_target_value(&ev))
                        on:keydown=poser_au_clavier
                    />
                </label>
                <div>
                    <button type="button" on:click=move |_| ecran.poser_l_etiquette()>
                        "Poser l'étiquette"
                    </button>
                </div>
            </Show>
            <h3>"Liens"</h3>
            <p hidden=move || !liens().is_empty()>{CASE_VIDE}</p>
            <ul class="puces">
                <For
                    each=liens
                    key=|lien: &(String, String, Option<i64>)| lien.clone()
                    children=move |(autre, nom, le): (String, String, Option<i64>)| {
                        let nom_du_bouton = format!("Délier : {nom}");
                        view! {
                            <li>
                                <span>{nom}</span>
                                <Show when=trier>
                                    <button
                                        type="button"
                                        aria-label=nom_du_bouton.clone()
                                        on:click={
                                            let autre = autre.clone();
                                            move |_| ecran.delier(autre.clone(), le)
                                        }
                                    >
                                        "Délier"
                                    </button>
                                </Show>
                            </li>
                        }
                    }
                />
            </ul>
            <div>
                <button type="button" on:click=move |_| ecran.retirer_ou_retablir()>
                    {move || offerts().dernier}
                </button>
            </div>
        </aside>
    }
}

/// Le volet des idées retirées : son titre, qui prend le focus à l'ouverture, « Fermer », qui rend
/// le focus à « Idées retirées », et leur liste, en fenêtre comme celle de la boîte, chacune avec
/// « Rétablir ». Monté, il lit la fenêtre que sa liste montre.
#[component]
fn VoletDesRetirees(ecran: Ecran) -> impl IntoView {
    let boite = ecran.boite;
    Effect::new(move |_| {
        if ecran.liste_des_retirees.get().is_some() && ecran.etalon.get().is_some() {
            ecran.demander_les_retirees(true);
        }
    });
    let total = move || {
        boite
            .retirees
            .with(|lues| lues.as_ref().map(|lues| lues.vue.total))
    };
    let hauteur_de_la_piste = move || {
        let lignes = total().unwrap_or(0);
        format!("calc({lignes} * var(--ligne))")
    };
    let haut_des_lignes = move || {
        let debut = boite
            .retirees
            .with(|lues| lues.as_ref().map_or(0, |lues| lues.fenetre.debut));
        format!("calc({debut} * var(--ligne))")
    };
    let lignes = move || {
        boite.retirees.with(|lues| {
            lues.as_ref()
                .map(|lues| lues.vue.lignes.clone())
                .unwrap_or_default()
        })
    };
    view! {
        <aside
            id="volet-des-retirees"
            class="volet volet-des-retirees"
            aria-labelledby="titre-des-retirees"
        >
            <div class="volet-haut">
                <h2 id="titre-des-retirees" tabindex="-1" node_ref=ecran.titre_des_retirees>
                    "Idées retirées"
                </h2>
                <button
                    type="button"
                    aria-keyshortcuts="Escape"
                    on:click=move |_| ecran.fermer_par_echap()
                >
                    "Fermer"
                </button>
            </div>
            <p hidden=move || total() != Some(0)>"Aucune idée retirée."</p>
            <div
                class="liste"
                tabindex="0"
                role="region"
                aria-label="Idées retirées"
                node_ref=ecran.liste_des_retirees
                on:scroll=move |_| ecran.demander_les_retirees(false)
            >
                <div class="piste" style:height=hauteur_de_la_piste>
                    <div class="lignes" role="list" style:top=haut_des_lignes>
                        <ForEnumerate
                            each=lignes
                            key=cle_de_la_ligne
                            children={move |index: ReadSignal<usize>, ligne: LigneDeLaBoite| {
                                ligne_retiree(ecran, index, ligne)
                            }}
                        />
                    </div>
                </div>
            </div>
        </aside>
    }
}

/// Le volet du journal du fonds, en lecture seule : son titre, qui prend le focus à l'ouverture,
/// « Fermer », qui rend le focus à « Journal du fonds », son en-tête (le nombre d'entrées et
/// l'empreinte), et la liste de ses entrées, la plus récente d'abord, en fenêtre comme celle des
/// idées retirées. Chaque ligne se date au décalage de son instant (`natif::decalage_a`). Monté,
/// il lit la fenêtre que sa liste montre.
#[component]
fn VoletDuJournal(ecran: Ecran) -> impl IntoView {
    let boite = ecran.boite;
    Effect::new(move |_| {
        if ecran.liste_du_journal.get().is_some() && ecran.etalon.get().is_some() {
            ecran.demander_le_journal(true);
        }
    });
    let total = move || boite.journal.with(|lu| lu.as_ref().map(|lu| lu.vue.total));
    let entete = move || {
        boite.journal.with(|lu| {
            lu.as_ref()
                .map(|lu| entete_du_journal(lu.vue.total, &lu.vue.empreinte))
                .unwrap_or_default()
        })
    };
    let hauteur_de_la_piste = move || {
        let lignes = total().unwrap_or(0);
        format!("calc({lignes} * var(--ligne))")
    };
    let haut_des_lignes = move || {
        let debut = boite
            .journal
            .with(|lu| lu.as_ref().map_or(0, |lu| lu.fenetre.debut));
        format!("calc({debut} * var(--ligne))")
    };
    // Une ligne par entrée, sa clé son rang dans le journal et son texte : deux entrées peuvent
    // porter le même message à la même heure.
    let lignes = move || {
        boite.journal.with(|lu| {
            lu.as_ref()
                .map(|lu| {
                    let debut = u64::from(lu.fenetre.debut);
                    lu.vue
                        .lignes
                        .iter()
                        .enumerate()
                        .map(|(rang, entree)| {
                            let texte = ligne_du_journal(entree, natif::decalage_a(entree.heure));
                            (debut + rang as u64, texte)
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
    };
    view! {
        <aside
            id="volet-du-journal"
            class="volet volet-du-journal"
            aria-labelledby="titre-du-journal"
        >
            <div class="volet-haut">
                <h2 id="titre-du-journal" tabindex="-1" node_ref=ecran.titre_du_journal>
                    "Journal du fonds"
                </h2>
                <button
                    type="button"
                    aria-keyshortcuts="Escape"
                    on:click=move |_| ecran.fermer_par_echap()
                >
                    "Fermer"
                </button>
            </div>
            <p>{entete}</p>
            <div
                class="liste"
                tabindex="0"
                role="region"
                aria-label="Entrées du journal"
                node_ref=ecran.liste_du_journal
                on:scroll=move |_| ecran.demander_le_journal(false)
            >
                <div class="piste" style:height=hauteur_de_la_piste>
                    <div class="lignes" role="list" style:top=haut_des_lignes>
                        <For
                            each=lignes
                            key=|(rang, texte)| (*rang, texte.clone())
                            children=move |(rang, texte)| {
                                view! {
                                    <div
                                        class="ligne"
                                        role="listitem"
                                        aria-posinset=(rang + 1).to_string()
                                        aria-setsize=move || total().unwrap_or(0).to_string()
                                    >
                                        <span class="cellules journal">
                                            <span class="cellule">{texte}</span>
                                        </span>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>
        </aside>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recherche::{EtatDeLaRequete, jour_du_champ};
    use commandes::{Appareil, Bornes, Provenance};

    /// Une ligne inventée, captée sur l'établi, sans durée ni lien.
    fn ligne(texte: Option<&str>, etiquettes: &[&str]) -> LigneDeLaBoite {
        LigneDeLaBoite {
            idee: "idee-inventee".to_string(),
            le: 1_793_750_400,
            texte: texte.map(str::to_string),
            etiquettes: etiquettes.iter().map(|e| e.to_string()).collect(),
            duree_ms: None,
            reliee: false,
            provenance: Provenance::Captee {
                par: Appareil::Etabli,
            },
            retiree: None,
        }
    }

    #[test]
    fn la_fenetre_borne_les_lignes_rendues() {
        // Deux mille idées, des lignes de 40 px, une vue de 800 px : vingt lignes vues.
        let a = |defilement_px| fenetre(defilement_px, 800.0, 40.0, 2_000);
        assert_eq!(
            a(0.0),
            Fenetre {
                debut: 0,
                nombre: 25
            }
        );
        assert_eq!(
            a(40_000.0),
            Fenetre {
                debut: 995,
                nombre: 30
            }
        );
        assert_eq!(
            a(79_200.0),
            Fenetre {
                debut: 1_975,
                nombre: 25
            }
        );
        assert_eq!(a(200_000.0).nombre, 0);
        // Des quotients qui ne tombent pas juste : la première ligne vue s'arrondit vers le bas, et
        // le nombre de lignes vues vers le haut.
        assert_eq!(
            fenetre(40_020.0, 790.0, 40.0, 2_000),
            Fenetre {
                debut: 995,
                nombre: 30
            }
        );
        // Jamais plus de trente lignes : les vingt vues, plus deux marges.
        for defilement_px in (0..=80_000).step_by(7) {
            let rendues = a(f64::from(defilement_px)).nombre;
            assert!(
                rendues <= 20 + 2 * MARGE,
                "{rendues} lignes à {defilement_px} px"
            );
        }
        // Une vue plus haute que `FENETRE_MAX` lignes : la fenêtre s'y borne.
        assert_eq!(
            fenetre(0.0, 1_000_000.0, 40.0, 1_000_000).nombre,
            FENETRE_MAX
        );
        // Sans hauteur de ligne lisible : les `FENETRE_MAX` premières lignes, toutes s'il y en a
        // moins.
        assert_eq!(
            fenetre(40_000.0, 800.0, 0.0, 2_000),
            Fenetre {
                debut: 0,
                nombre: FENETRE_MAX
            }
        );
        assert_eq!(
            fenetre(0.0, 800.0, f64::NAN, 12),
            Fenetre {
                debut: 0,
                nombre: 12
            }
        );
    }

    #[test]
    fn la_fenetre_demandee_couvre_les_lignes_vues() {
        let demandee = fenetre(40_000.0, 800.0, 40.0, 2_000);
        let couverte = |defilement_px| couvre(&demandee, defilement_px, 800.0, 40.0, 2_000);
        // La fenêtre de 995 à 1 025 porte les lignes vues, de 1 000 à 1 019, et une ligne au-delà
        // de chaque bord, 999 et 1 020.
        assert!(couverte(40_000.0));
        // Quatre lignes plus bas, de 1 004 à 1 023 : la ligne au-delà, 1 024, est dans la fenêtre.
        assert!(couverte(40_160.0));
        // Cinq lignes plus bas, de 1 005 à 1 024 : la ligne au-delà, 1 025, n'y est pas, et le
        // défilement demande une autre fenêtre avant que Tab n'atteigne la dernière ligne rendue.
        assert!(!couverte(40_200.0));
        assert!(!couverte(40_220.0));
        // Quatre lignes plus haut, de 996 à 1 015, la ligne 995 au-delà ; cinq lignes plus haut, la
        // ligne au-delà, 994, manque.
        assert!(couverte(39_840.0));
        assert!(!couverte(39_800.0));
        assert!(!couverte(39_760.0));
        // Au bout de la liste, les lignes vues s'arrêtent à la dernière.
        let au_bout = fenetre(79_200.0, 800.0, 40.0, 2_000);
        assert!(couvre(&au_bout, 79_200.0, 800.0, 40.0, 2_000));
        // Sans hauteur de ligne lisible, rien n'est couvert.
        assert!(!couvre(&demandee, 40_000.0, 800.0, 0.0, 2_000));
    }

    #[test]
    fn la_fenetre_demandee_au_bout_porte_les_idees_qu_un_geste_ajoute() {
        // Deux mille idées lues, la liste défilée jusqu'au bout : la fenêtre demandée va cinq
        // lignes au-delà de la dernière, que la relecture d'après une note reprend et montre.
        let au_bout = Fenetre {
            debut: 1_975,
            nombre: 30,
        };
        assert_eq!(
            fenetre_a_demander(None, 79_200.0, 800.0, 40.0, Some(2_000), false),
            Some(au_bout.clone())
        );
        // Une idée notée, la même fenêtre porte encore les lignes vues : rien ne se redemande.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 79_240.0, 800.0, 40.0, Some(2_001), false),
            None
        );
        // Forcée, elle se redemande.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 79_200.0, 800.0, 40.0, Some(2_000), true),
            Some(au_bout.clone())
        );
        // Revenue au début, la liste ne voit plus ses lignes dans la dernière fenêtre : une autre
        // se demande.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 0.0, 800.0, 40.0, Some(2_000), false),
            Some(Fenetre {
                debut: 0,
                nombre: 25
            })
        );
        // Sans total lu, rien ne borne les lignes vues.
        assert_eq!(
            fenetre_a_demander(Some(&au_bout), 79_200.0, 800.0, 40.0, None, false),
            None
        );
    }

    /// Une ligne inventée de l'idée `idee`.
    fn ligne_de(idee: &str) -> LigneDeLaBoite {
        LigneDeLaBoite {
            idee: idee.to_string(),
            ..ligne(Some("Une idée de pont"), &[])
        }
    }

    /// La commande `commande` de la ligne de l'idée `idee`.
    fn gardee(idee: &str, commande: Commande) -> Gardee {
        Gardee {
            idee: idee.to_string(),
            commande,
        }
    }

    #[test]
    fn le_focus_perdu_revient_a_la_ligne_gardee_ou_a_sa_liste() {
        let lues = [ligne_de("a"), ligne_de("b")];
        let case_de_b = gardee("b", Commande::Cocher);
        // La fenêtre posée, le focus est tombé sur `body` : la ligne gardée, encore lue, le
        // reprend, sur la même commande.
        assert_eq!(
            focus_a_rendre(Some(&case_de_b), Liste::Boite, &lues, Actif::Absent),
            Rendu::Ligne(case_de_b.clone())
        );
        // Sortie de la fenêtre lue, le focus va à la liste.
        let sortie = gardee("c", Commande::Choisir);
        assert_eq!(
            focus_a_rendre(Some(&sortie), Liste::Boite, &lues, Actif::Absent),
            Rendu::Liste
        );
        // Le focus ailleurs, rien ne bouge.
        assert_eq!(
            focus_a_rendre(Some(&case_de_b), Liste::Boite, &lues, Actif::Ailleurs),
            Rendu::Rien
        );
        assert_eq!(
            focus_a_rendre(Some(&sortie), Liste::Boite, &lues, Actif::Ailleurs),
            Rendu::Rien
        );
        // Aucune ligne gardée, ou une ligne de l'autre liste : rien ne bouge.
        assert_eq!(
            focus_a_rendre(None, Liste::Boite, &lues, Actif::Absent),
            Rendu::Rien
        );
        let retablir_b = gardee("b", Commande::Retablir);
        assert_eq!(
            focus_a_rendre(Some(&retablir_b), Liste::Boite, &lues, Actif::Absent),
            Rendu::Rien
        );
        // Les idées retirées, de même, sur « Rétablir ».
        assert_eq!(
            focus_a_rendre(Some(&retablir_b), Liste::Retirees, &lues, Actif::Absent),
            Rendu::Ligne(retablir_b.clone())
        );
        assert_eq!(
            focus_a_rendre(
                Some(&gardee("c", Commande::Retablir)),
                Liste::Retirees,
                &lues,
                Actif::Absent
            ),
            Rendu::Liste
        );
        assert_eq!(
            focus_a_rendre(Some(&case_de_b), Liste::Retirees, &lues, Actif::Absent),
            Rendu::Rien
        );
    }

    #[test]
    fn fermer_le_volet_rend_le_focus_a_la_ligne_qui_l_a_ouvert() {
        let lues = [ligne_de("a"), ligne_de("b")];
        // La ligne revient à la vue : la liste défile jusqu'à elle.
        assert_eq!(
            cible_apres_le_volet("b", &lues),
            Cible::Ligne {
                gardee: gardee("b", Commande::Choisir),
                defilement: Defilement::VersLaLigne
            }
        );
        // Sa ligne sortie de la fenêtre lue, le focus va à la liste.
        assert_eq!(cible_apres_le_volet("c", &lues), Cible::Liste);
    }

    #[test]
    fn le_fonds_vide_se_dit_par_des_constats() {
        // Dès qu'une idée est interrogée, il n'y a pas de page.
        assert_eq!(page_du_fonds_vide(1, false), None);
        assert_eq!(page_du_fonds_vide(12, true), None);
        assert_eq!(
            page_du_fonds_vide(0, false),
            Some([
                "La boîte d'idées est vide.",
                "Une idée se note dans le champ « Noter une idée »."
            ])
        );
        assert_eq!(
            page_du_fonds_vide(0, true),
            Some([
                "La boîte d'idées est vide.",
                "Le fonds est en lecture seule : le bandeau dit pourquoi."
            ])
        );
        for lecture_seule in [false, true] {
            for texte in page_du_fonds_vide(0, lecture_seule).unwrap() {
                // Un constat, jamais un ordre : aucun texte ne commence par un verbe à
                // l'impératif, rétablir compris.
                for verbe in [
                    "Notez",
                    "Importez",
                    "Créez",
                    "Ajoutez",
                    "Commencez",
                    "Rétablissez",
                ] {
                    assert!(!texte.starts_with(verbe), "{texte}");
                }
                // Un constat vrai : retirer n'efface rien, et toutes les idées retirées, le fonds
                // en porte encore. Aucun texte ne dit que le fonds n'a pas d'idée.
                assert!(!texte.contains("aucune idée"), "{texte}");
            }
        }
    }

    #[test]
    fn la_liste_cachee_ne_recoit_pas_le_focus() {
        // Fonds vide : la liste est cachée, le focus va au champ « Noter une idée ».
        assert_eq!(cible_repliee(Cible::Liste, true), Cible::ChampDeNote);
        // Sinon, ou pour toute autre cible, rien ne change.
        assert_eq!(cible_repliee(Cible::Liste, false), Cible::Liste);
        assert_eq!(
            cible_repliee(Cible::BoutonDesRetirees, true),
            Cible::BoutonDesRetirees
        );
        assert_eq!(
            cible_repliee(Cible::TitreDeLIdee, true),
            Cible::TitreDeLIdee
        );
    }

    #[test]
    fn le_fonds_devenu_vide_rend_au_champ_de_note_le_focus_de_la_liste() {
        // La dernière idée retirée, la liste a reçu le focus avant la relecture, et la page du
        // fonds vide la cache : il va au champ « Noter une idée ».
        assert_eq!(
            cible_quand_le_fonds_se_vide(true, true),
            Some(Cible::ChampDeNote)
        );
        // Le focus ailleurs, ou le fonds non vide : il reste où il est.
        assert_eq!(cible_quand_le_fonds_se_vide(true, false), None);
        assert_eq!(cible_quand_le_fonds_se_vide(false, true), None);
        assert_eq!(cible_quand_le_fonds_se_vide(false, false), None);
    }

    #[test]
    fn echap_ferme_le_volet_et_rend_le_focus_comme_fermer() {
        let lues = [ligne_de("a"), ligne_de("b")];
        // Le volet de l'idée : le focus va à la ligne qui l'a ouvert, comme « Fermer ».
        assert_eq!(
            cible_de_fermeture(&Volet::Idee("b".to_string()), &lues),
            Some(cible_apres_le_volet("b", &lues))
        );
        // Sa ligne sortie de la fenêtre lue, à la liste.
        assert_eq!(
            cible_de_fermeture(&Volet::Idee("c".to_string()), &lues),
            Some(Cible::Liste)
        );
        // Le volet des retirées : le focus va au bouton « Idées retirées », comme « Fermer ».
        assert_eq!(
            cible_de_fermeture(&Volet::Retirees, &lues),
            Some(Cible::BoutonDesRetirees)
        );
        // Le volet du journal : le focus va au bouton « Journal du fonds », comme « Fermer ».
        assert_eq!(
            cible_de_fermeture(&Volet::Journal, &lues),
            Some(Cible::BoutonDuJournal)
        );
        // Aucun volet : rien ne se ferme, le focus ne bouge pas.
        assert_eq!(cible_de_fermeture(&Volet::Aucun, &lues), None);
    }

    #[test]
    fn le_focus_rendu_apres_une_perte_ne_fait_pas_defiler_la_liste() {
        // Le focus rendu à la ligne gardée laisse la liste où la molette l'a menée, même si la
        // ligne est cachée dans la marge.
        let case_de_b = gardee("b", Commande::Cocher);
        assert_eq!(
            cible_du_rendu(Rendu::Ligne(case_de_b.clone()), Liste::Boite),
            Some(Cible::Ligne {
                gardee: case_de_b,
                defilement: Defilement::Garde
            })
        );
        assert_eq!(
            cible_du_rendu(Rendu::Liste, Liste::Boite),
            Some(Cible::Liste)
        );
        assert_eq!(
            cible_du_rendu(Rendu::Liste, Liste::Retirees),
            Some(Cible::ListeDesRetirees)
        );
        assert_eq!(cible_du_rendu(Rendu::Rien, Liste::Boite), None);
        // Le défilement que `focus()` a fait se remet ; sans défilement, rien ne se remet ; celui
        // de « Fermer » reste.
        assert_eq!(
            defilement_a_remettre(Defilement::Garde, 39_600, 40_000),
            Some(39_600)
        );
        assert_eq!(
            defilement_a_remettre(Defilement::Garde, 39_600, 39_600),
            None
        );
        assert_eq!(
            defilement_a_remettre(Defilement::VersLaLigne, 39_600, 40_000),
            None
        );
    }

    #[test]
    fn une_touche_tenue_n_envoie_rien() {
        // La première frappe d'`Entrée` envoie le champ ; tenue, la touche se répète sans rien
        // renvoyer, comme `Ctrl+Z`.
        assert!(envoi_au_clavier("Enter", false, false));
        assert!(!envoi_au_clavier("Enter", true, false));
        // Pendant une composition (un accent en cours), `Entrée` la valide et n'envoie rien.
        assert!(!envoi_au_clavier("Enter", false, true));
        assert!(!envoi_au_clavier("a", false, false));
        assert!(!envoi_au_clavier("Tab", false, false));
    }

    #[test]
    fn le_volet_d_une_idee_retiree_n_offre_que_retablir() {
        assert_eq!(
            ce_que_le_volet_offre(false),
            Offerts {
                trier: true,
                dernier: "Retirer"
            }
        );
        // Ni « Compléter », ni « Ôter », ni « Poser l'étiquette », ni « Délier » : le noyau les
        // refuserait tous.
        assert_eq!(
            ce_que_le_volet_offre(true),
            Offerts {
                trier: false,
                dernier: "Rétablir"
            }
        );
    }

    #[test]
    fn un_geste_en_vol_ignore_une_seconde_activation() {
        let en_vol = StoredValue::new(false);
        assert!(prendre(en_vol));
        // Deux « Rétablir », deux « Retirer », ou `Entrée` tenue : la seconde s'ignore.
        assert!(!prendre(en_vol));
        assert!(en_vol.get_value());
        // La réponse lue, le geste suivant part.
        en_vol.set_value(false);
        assert!(prendre(en_vol));
    }

    #[test]
    fn ouvrir_une_autre_idee_oublie_le_brouillon_de_la_precedente() {
        let boite = EtatDeLaBoite::nouveau();
        assert!(boite.montrer_l_idee("a"));
        boite.brouillon.set("Un pont en mi mineur".to_string());
        boite.brouillon_de.set_value(Some("a".to_string()));
        boite.etiquette.set("pont".to_string());
        // La même idée : rien ne s'oublie.
        assert!(!boite.montrer_l_idee("a"));
        assert_eq!(boite.brouillon.get_untracked(), "Un pont en mi mineur");
        assert_eq!(boite.etiquette.get_untracked(), "pont");
        // Une autre : le brouillon, l'idée dont il est parti et l'étiquette s'oublient.
        assert!(boite.montrer_l_idee("b"));
        assert_eq!(boite.volet.get_untracked(), Volet::Idee("b".to_string()));
        assert_eq!(boite.idee.get_untracked(), None);
        assert_eq!(boite.brouillon.get_untracked(), "");
        assert_eq!(boite.brouillon_de.get_value(), None);
        assert_eq!(boite.etiquette.get_untracked(), "");
    }

    #[test]
    fn le_nom_d_une_ligne_nomme_chaque_case() {
        assert_eq!(
            nom_de_la_ligne("3 novembre 2026", &ligne(Some("Une idée de pont"), &[])),
            "3 novembre 2026 · Une idée de pont · étiquettes — · durée — · non reliée · captée"
        );
        let mut l = ligne(None, &["pont", "refrain"]);
        l.duree_ms = Some(83_000);
        l.reliee = true;
        assert_eq!(
            nom_de_la_ligne("3 novembre 2026", &l),
            "3 novembre 2026 · — · étiquettes #pont #refrain · durée 1:23 · reliée · captée"
        );
    }

    #[test]
    fn ctrl_z_n_annule_un_geste_que_hors_d_un_champ() {
        assert!(ctrl_z_annule("z", true, false));
        // `Ctrl+Maj+Z` donne « Z ».
        assert!(ctrl_z_annule("Z", true, false));
        assert!(!ctrl_z_annule("z", true, true));
        assert!(!ctrl_z_annule("z", false, false));
        assert!(!ctrl_z_annule("y", true, false));
    }

    #[test]
    fn le_bouton_d_annulation_dit_le_geste_vise() {
        assert_eq!(
            libelle_d_annulation(Some("Idée du 3 novembre : étiquette #pont posée")),
            "Annuler : Idée du 3 novembre : étiquette #pont posée"
        );
        assert_eq!(libelle_d_annulation(None), "Rien à annuler");
    }

    #[test]
    fn cap01_4_noter_se_fait_en_un_geste_sans_titre() {
        // Une part de CAP-01 · 4 : à l'établi, `Entrée` dans le champ note l'idée, sans demande de
        // titre ; la capture du carnet vient au lot 2.
        assert_eq!(
            note_a_envoyer("Une idée de pont", "Enter", 60),
            Some(Noter {
                texte: "Une idée de pont".to_string(),
                decalage_minutes: 60
            })
        );
        assert_eq!(note_a_envoyer("Une idée de pont", "a", 60), None);
        assert_eq!(note_a_envoyer("Une idée de pont", "Tab", 60), None);
        assert_eq!(note_a_envoyer(" \t  ", "Enter", 60), None);
        assert_eq!(note_a_envoyer("", "Enter", 60), None);
        // La note part sans ses blancs autour, avec le décalage du geste.
        assert_eq!(
            note_a_envoyer("  Une idée de pont ", "Enter", -300),
            Some(Noter {
                texte: "Une idée de pont".to_string(),
                decalage_minutes: -300
            })
        );
    }

    /// Le décalage local de l'heure de Paris, en minutes : l'heure d'été (120) jusqu'au
    /// 25 octobre 2026 à 1 h UTC, l'heure d'hiver (60) ensuite.
    fn decalage_de_paris(le: i64) -> i32 {
        if le < 1_792_890_000 { 120 } else { 60 }
    }

    #[test]
    fn relier_les_cochees_date_chaque_idee_a_sa_naissance() {
        // L'une née le 24 octobre 2026 à 22 h 30 UTC, à l'heure d'été ; l'autre le 17 novembre
        // à 9 h UTC, à l'heure d'hiver.
        let a = ("idee-a".to_string(), 1_792_881_000);
        let b = ("idee-b".to_string(), 1_794_906_000);
        assert_eq!(
            relier_les_cochees(&[a.clone(), b.clone()], decalage_de_paris),
            Some(Relier {
                a: "idee-a".to_string(),
                b: "idee-b".to_string(),
                decalage_minutes: 120,
                decalage_b_minutes: 60,
            })
        );
        assert_eq!(
            relier_les_cochees(std::slice::from_ref(&a), decalage_de_paris),
            None
        );
        assert_eq!(
            relier_les_cochees(
                &[a, b, ("idee-c".to_string(), 1_793_750_400)],
                decalage_de_paris
            ),
            None
        );
    }

    #[test]
    fn delier_depuis_le_volet_date_chaque_bout() {
        // L'idée du volet née le 24 octobre 2026 à 22 h 30 UTC, l'autre bout le 17 novembre à
        // 9 h UTC : chacun au décalage de son instant, jamais à celui du geste.
        assert_eq!(
            deliaison(
                ("idee-du-volet".to_string(), Some(1_792_881_000)),
                ("autre-bout".to_string(), Some(1_794_906_000)),
                decalage_de_paris,
                -300,
            ),
            Relier {
                a: "idee-du-volet".to_string(),
                b: "autre-bout".to_string(),
                decalage_minutes: 120,
                decalage_b_minutes: 60,
            }
        );
        // Un bout dont la naissance ne s'est pas lue prend le décalage du geste : la limite.
        let sans_naissance = deliaison(
            ("idee-du-volet".to_string(), Some(1_792_881_000)),
            ("autre-bout".to_string(), None),
            decalage_de_paris,
            -300,
        );
        assert_eq!(
            (
                sans_naissance.decalage_minutes,
                sans_naissance.decalage_b_minutes
            ),
            (120, -300)
        );
        let volet_sans_naissance = deliaison(
            ("idee-du-volet".to_string(), None),
            ("autre-bout".to_string(), Some(1_794_906_000)),
            decalage_de_paris,
            -300,
        );
        assert_eq!(
            (
                volet_sans_naissance.decalage_minutes,
                volet_sans_naissance.decalage_b_minutes
            ),
            (-300, 60)
        );
    }

    #[test]
    fn un_texte_piege_se_montre_tel_quel() {
        let piege = "<img src=x onerror=alert(1)>";
        assert_eq!(texte_de_la_ligne(&ligne(Some(piege), &[])), piege);
    }

    #[test]
    fn une_case_vide_s_ecrit_d_un_tiret_cadratin() {
        let vide = ligne(None, &[]);
        assert_eq!(texte_de_la_ligne(&vide), "—");
        assert_eq!(etiquettes_de_la_ligne(&vide), "—");
        assert_eq!(texte_de_la_ligne(&ligne(Some(""), &[])), "—");
        assert_eq!(
            etiquettes_de_la_ligne(&ligne(None, &["pont", "refrain"])),
            "#pont #refrain"
        );
    }

    #[test]
    fn l_etat_et_la_provenance_se_disent_en_mots() {
        // Aucune information par la seule couleur : l'état et la provenance sont des mots.
        let mut l = ligne(Some("Une idée de pont"), &[]);
        assert_eq!(etat_de_la_ligne(&l), "non reliée");
        assert_eq!(provenance_de_la_ligne(&l), "captée");
        l.reliee = true;
        l.provenance = Provenance::Importee {
            par: Appareil::Etabli,
            depuis: "import-invente".to_string(),
        };
        assert_eq!(etat_de_la_ligne(&l), "reliée");
        assert_eq!(provenance_de_la_ligne(&l), "importée");
    }

    #[test]
    fn l_etalon_de_la_feuille_de_style_mesure_cent_lignes() {
        let css = include_str!("../style.css");
        assert!(
            css.contains("--ligne: 2.5rem;"),
            "la hauteur d'une ligne manque à la feuille de style"
        );
        let etalon = format!("height: calc({LIGNES_DE_L_ETALON} * var(--ligne));");
        assert!(
            css.contains(&etalon),
            "{etalon} manque à la feuille de style"
        );
    }

    /// La valeur que déclare pour `propriete` une règle de la feuille de style `css` dont le
    /// sélecteur, seul ou dernier d'une liste, ouvre une ligne et vaut `selecteur` : la première
    /// règle qui la déclare ; rien si aucune ne la déclare.
    fn declaree<'a>(css: &'a str, selecteur: &str, propriete: &str) -> Option<&'a str> {
        let tete = format!("\n{selecteur} {{");
        let declaration = format!("\n  {propriete}: ");
        css.match_indices(&tete).find_map(|(debut, _)| {
            let regle = &css[debut + tete.len()..];
            let regle = &regle[..regle.find('}').unwrap_or(regle.len())];
            let valeur = &regle[regle.find(&declaration)? + declaration.len()..];
            Some(valeur[..valeur.find(';').unwrap_or(valeur.len())].trim())
        })
    }

    /// Une longueur de la feuille de style en rem, sans son unité.
    fn en_rem(valeur: &str) -> f64 {
        valeur
            .strip_suffix("rem")
            .and_then(|nombre| nombre.parse().ok())
            .unwrap_or_else(|| panic!("{valeur} n'est pas une longueur en rem"))
    }

    #[test]
    fn une_ligne_du_journal_tient_trois_lignes_et_son_etalon_en_mesure_cent() {
        // Coupé sur une ligne, un message long du journal perdait son verbe (« Requête « Une
        // requête au nom très long… »). Chaque ligne du journal a une hauteur fixe qui tient trois
        // lignes de texte, où le message passe à la ligne ; il ne se coupe qu'au-delà de la
        // troisième. La hauteur et l'interligne s'écrivent en rem : ils suivent la police, à 100 %
        // comme à 200 %.
        let css = include_str!("../style.css");
        let lue = |selecteur: &str, propriete: &str| {
            declaree(css, selecteur, propriete)
                .unwrap_or_else(|| panic!("{selecteur} ne déclare pas {propriete}"))
        };
        let message = ".cellules.journal .cellule";
        assert_eq!(lue(message, "white-space"), "normal");
        assert_eq!(lue(message, "display"), "-webkit-box");
        assert_eq!(lue(message, "-webkit-box-orient"), "vertical");
        let lignes_de_texte: u32 = lue(message, "-webkit-line-clamp")
            .parse()
            .expect("un nombre de lignes");
        assert_eq!(lignes_de_texte, 3);
        let interligne = en_rem(lue(message, "line-height"));
        let hauteur = en_rem(lue(".volet-du-journal", "--ligne-du-journal"));
        // Les trois lignes de texte et une marge d'un demi-rem au moins, le filet du bas compris,
        // tiennent dans la ligne : rien ne déborde sur la suivante.
        assert!(
            hauteur >= f64::from(lignes_de_texte) * interligne + 0.5,
            "une ligne du journal de {hauteur} rem ne tient pas trois lignes de {interligne} rem"
        );
        assert_eq!(
            lue(".volet-du-journal .ligne", "height"),
            "var(--ligne-du-journal)"
        );
        // Le second étalon, dans le volet du journal, mesure cent lignes du journal, comme le
        // premier mesure cent lignes de la boîte.
        assert_eq!(
            lue(".volet-du-journal .etalon > div", "height"),
            format!("calc({LIGNES_DE_L_ETALON} * var(--ligne-du-journal))")
        );
        // Les lignes de la boîte et des idées retirées gardent leur hauteur et leur coupe sur une
        // ligne.
        assert_eq!(lue(".ligne", "height"), "var(--ligne)");
        assert_eq!(lue(".cellule", "white-space"), "nowrap");
        assert_eq!(lue(".cellule", "text-overflow"), "ellipsis");
    }

    #[test]
    fn la_liste_et_le_volet_ne_grandissent_pas_avec_leur_contenu() {
        // `.boite-corps` passe à la ligne (`flex-wrap: wrap`) : chaque rangée y prend au moins la
        // hauteur de son contenu. Sans `contain: size`, la liste grandit avec sa piste, la page
        // défile à sa place, et la liste, qui ne défile plus, ne demande plus de fenêtre de lignes
        // à son défilement ; le volet ne défile pas non plus.
        let css = include_str!("../style.css");
        for selecteur in [".colonne-liste", ".volet"] {
            let tete = format!("\n{selecteur} {{");
            let debut = css
                .find(&tete)
                .unwrap_or_else(|| panic!("la règle {selecteur} manque à la feuille de style"));
            let regle = &css[debut..];
            let regle = &regle[..regle.find('}').unwrap_or(regle.len())];
            assert!(
                regle.contains("contain: size;"),
                "la règle {selecteur} ne porte pas `contain: size;`"
            );
        }
    }

    #[test]
    fn le_focus_venu_du_dehors_entre_sur_une_ligne_entierement_vue() {
        // Des lignes de 40 px, une vue de 320 px défilée de 1 296 px : les lignes 33 à 39 sont
        // entièrement vues, la 32 et la 40 en partie.
        let a = |rang| ligne_d_entree(rang, 1_296.0, 320.0, 40.0);
        // Tab atteint d'abord la première ligne de la fenêtre, cachée dans la marge du haut.
        assert_eq!(a(27), Some((Bord::Haut, 33)));
        assert_eq!(a(31), Some((Bord::Haut, 33)));
        // Une ligne vue, même en partie, garde le focus.
        assert_eq!(a(32), None);
        assert_eq!(a(33), None);
        assert_eq!(a(39), None);
        assert_eq!(a(40), None);
        // Maj+Tab, venu d'en dessous, atteint d'abord une ligne cachée dans la marge du bas.
        assert_eq!(a(41), Some((Bord::Bas, 39)));
        assert_eq!(a(44), Some((Bord::Bas, 39)));
        // Aux bords exacts, une vue de 1 280 à 1 600 px : la ligne 31 finit au haut de la vue, la
        // ligne 40 commence à son bas ; les lignes 32 et 39 sont entièrement vues.
        let b = |rang| ligne_d_entree(rang, 1_280.0, 320.0, 40.0);
        assert_eq!(b(31), Some((Bord::Haut, 32)));
        assert_eq!(b(32), None);
        assert_eq!(b(39), None);
        assert_eq!(b(40), Some((Bord::Bas, 39)));
        // La liste en haut : la première ligne est vue, rien ne change.
        assert_eq!(ligne_d_entree(0, 0.0, 320.0, 40.0), None);
        assert_eq!(ligne_d_entree(12, 0.0, 320.0, 40.0), Some((Bord::Bas, 7)));
        // Une vue plus basse qu'une ligne n'en voit aucune entière : rien ne change.
        assert_eq!(ligne_d_entree(0, 50.0, 30.0, 40.0), None);
        assert_eq!(ligne_d_entree(5, 50.0, 30.0, 40.0), None);
        // Sans hauteur de ligne lisible, rien ne change.
        assert_eq!(ligne_d_entree(0, 1_296.0, 320.0, 0.0), None);
        assert_eq!(ligne_d_entree(0, 1_296.0, 320.0, f64::NAN), None);
    }

    #[test]
    fn l_idee_d_un_rang_se_lit_dans_la_fenetre() {
        let lignes: Vec<LigneDeLaBoite> = ["a", "b", "c"]
            .iter()
            .map(|id| LigneDeLaBoite {
                idee: id.to_string(),
                ..ligne(None, &[])
            })
            .collect();
        let fenetre = Fenetre {
            debut: 10,
            nombre: 3,
        };
        assert_eq!(idee_de_rang(&fenetre, &lignes, 10).as_deref(), Some("a"));
        assert_eq!(idee_de_rang(&fenetre, &lignes, 12).as_deref(), Some("c"));
        assert_eq!(idee_de_rang(&fenetre, &lignes, 9), None);
        assert_eq!(idee_de_rang(&fenetre, &lignes, 13), None);
    }

    #[test]
    fn la_commande_porte_la_requete_en_cours_et_ses_bornes() {
        // La requête « eglise » du 25 octobre au 25 octobre 2026, jour de 25 heures à Paris : la
        // date se borne du minuit local du 25 (22 h UTC le 24) à celui du 26 (23 h UTC le 25).
        let jour = jour_du_champ("2026-10-25");
        let requete = Requete {
            texte: "eglise".to_string(),
            du: jour.clone(),
            au: jour,
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        };
        let fenetre = Fenetre {
            debut: 40,
            nombre: 25,
        };
        assert_eq!(
            commande_de_la_boite(Ordre::Duree, &requete, decalage_de_paris, &fenetre),
            Boite {
                ordre: Ordre::Duree,
                requete: requete.clone(),
                bornes: Bornes {
                    des: Some(1_792_879_200),
                    avant: Some(1_792_969_200)
                },
                debut: 40,
                nombre: 25,
            }
        );
        // Sans jour, rien ne borne la date.
        assert_eq!(
            commande_de_la_boite(
                Ordre::Date,
                &Requete::default(),
                decalage_de_paris,
                &fenetre
            )
            .bornes,
            Bornes::default()
        );
    }

    /// Une vue de la boîte écrite pour le test : deux idées trouvées parmi six interrogées, dont
    /// deux sans texte, et aucune ligne.
    fn vue_de_la_reponse() -> VueDeLaBoite {
        VueDeLaBoite {
            non_reliees: 5,
            assiette_du_compte: Assiette {
                considerees: 6,
                portant: 6,
                decidees: 6,
                illisibles: 0,
            },
            retenues: 2,
            assiette_de_la_requete: Assiette {
                considerees: 6,
                portant: 4,
                decidees: 4,
                illisibles: 0,
            },
            sans: commandes::Sans {
                etiquette: None,
                duree: None,
                texte: Some(2),
            },
            etiquettes_du_fonds: vec!["pont".to_string()],
            lignes: Vec::new(),
        }
    }

    #[test]
    fn la_reponse_ne_se_dit_que_d_une_requete_posee() {
        let vue = vue_de_la_reponse();
        // Sans critère, la liste est toute la boîte : aucune réponse ne se dit.
        assert_eq!(texte_de_la_requete(&vue, &Requete::default()), None);
        // Un texte sans mot ne pose aucun critère (CAP-06).
        let sans_mot = Requete {
            texte: " … ".to_string(),
            ..Requete::default()
        };
        assert_eq!(texte_de_la_requete(&vue, &sans_mot), None);
        let eglise = Requete {
            texte: "eglise".to_string(),
            ..Requete::default()
        };
        assert_eq!(
            texte_de_la_requete(&vue, &eglise).as_deref(),
            Some("2 idées · 2 sans texte · 6 interrogées")
        );
        // Un critère hors du texte, seul, se répond de même.
        let prise = Requete {
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        };
        assert_eq!(
            texte_de_la_requete(&vue, &prise).as_deref(),
            Some("2 idées · 2 sans texte · 6 interrogées")
        );
    }

    #[test]
    fn chaque_frappe_demande_et_seule_la_derniere_reponse_compte() {
        // Trois frappes de suite, chacune sa demande : aucune minuterie ne les retient (I18).
        let demandes = Demandes::nouvelles();
        let fenetres = [0, 1, 2].map(|debut| Fenetre { debut, nombre: 25 });
        let rangs = fenetres.clone().map(|fenetre| demandes.demander(&fenetre));
        assert_eq!(demandes.derniere.get_value(), Some(fenetres[2].clone()));
        // Les deux premières répondent après la troisième : seule la dernière est retenue.
        let mut retenue = None;
        for (rang, reponse) in [
            (rangs[2], "troisième"),
            (rangs[0], "première"),
            (rangs[1], "deuxième"),
        ] {
            if demandes.est_la_derniere(rang) {
                retenue = Some(reponse);
            }
        }
        assert_eq!(retenue, Some("troisième"));
    }

    /// Un écran de la boîte, ses repères jamais montés.
    fn ecran() -> Ecran {
        Ecran {
            etat: Etat::nouveau(),
            boite: EtatDeLaBoite::nouveau(),
            focus: RwSignal::new(None),
            champ_de_note: NodeRef::new(),
            champ_de_nom: NodeRef::new(),
            liste: NodeRef::new(),
            etalon: NodeRef::new(),
            liste_des_retirees: NodeRef::new(),
            bouton_des_retirees: NodeRef::new(),
            liste_du_journal: NodeRef::new(),
            bouton_du_journal: NodeRef::new(),
            titre_de_l_idee: NodeRef::new(),
            titre_des_retirees: NodeRef::new(),
            titre_du_journal: NodeRef::new(),
        }
    }

    /// Chaque signal de la requête de l'écran, tel qu'il est, écrit.
    fn signaux(requete: EtatDeLaRequete) -> [String; 8] {
        [
            requete.texte.get_untracked(),
            requete.du.get_untracked(),
            requete.au.get_untracked(),
            format!("{:?}", requete.etiquette.get_untracked()),
            requete.duree.get_untracked().to_string(),
            requete.etat.get_untracked().to_string(),
            requete.provenance.get_untracked().to_string(),
            requete.contenu.get_untracked().to_string(),
        ]
    }

    #[test]
    fn noter_ne_touche_pas_la_requete_en_cours() {
        // Le point de revue 1, côté écran : la note faite vide son champ, et la requête en cours
        // reste ; la relecture qui suit la note la montre si elle la satisfait.
        let owner = Owner::new();
        owner.set();
        let ecran = ecran();
        let requete = ecran.etat.requete;
        let jour = jour_du_champ("2026-10-25");
        requete.poser(&Requete {
            texte: "eglise".to_string(),
            du: jour.clone(),
            au: jour,
            etiquette: Some("pont".to_string()),
            plus_de_secondes: Some(60),
            etat: Some(commandes::Etat::NonReliee),
            provenance: Some(commandes::ProvenanceDeLIdee::Captee),
            contenu: Some(Contenu::TexteDeLIdee),
        });
        let avant = signaux(requete);
        ecran.boite.note.set("  Une idée d'église ".to_string());
        ecran.conclure_la_note("Une idée d'église");
        assert_eq!(ecran.boite.note.get_untracked(), "");
        assert_eq!(signaux(requete), avant);
        // Le champ réécrit depuis le départ de la note garde ce qui y est écrit.
        ecran.boite.note.set("Une autre idée".to_string());
        ecran.conclure_la_note("Une idée d'église");
        assert_eq!(ecran.boite.note.get_untracked(), "Une autre idée");
        assert_eq!(signaux(requete), avant);
    }

    #[test]
    fn le_choix_toutes_ne_pose_aucune_etiquette() {
        // « Toutes », de valeur vide, ne pose aucun critère : une étiquette vide ne trouverait
        // rien.
        assert_eq!(etiquette_du_choix(""), None);
        assert_eq!(etiquette_du_choix("pont"), Some("pont".to_string()));
        // Une étiquette du fonds s'écrit comme la liste l'écrit.
        assert_eq!(etiquette_ecrite("pont"), "#pont");
        let du_fonds = ["pont".to_string(), "refrain".to_string()];
        assert_eq!(etiquettes_a_choisir(&du_fonds, None), du_fonds);
        assert_eq!(etiquettes_a_choisir(&du_fonds, Some("refrain")), du_fonds);
        // L'étiquette choisie que le fonds ne porte plus reste offerte, à son rang : le choix
        // se voit.
        assert_eq!(
            etiquettes_a_choisir(&du_fonds, Some("couplet")),
            ["couplet", "pont", "refrain"]
        );
    }

    #[test]
    fn pose_un_critere_dit_comme_le_noyau() {
        // `pose_un_critere` recopie, hors mutation, la règle de `Requete::est_vide` du noyau : sur
        // chaque paire, la requête des commandes et celle du noyau, écrites à la main aux mêmes
        // valeurs, elle dit comme lui.
        use noyau::donnees::boite::Contenu as ContenuDuNoyau;
        use noyau::donnees::requete::{
            Etat as EtatDuNoyau, Jour as JourDuNoyau, ProvenanceDeLIdee as ProvenanceDuNoyau,
            Requete as RequeteDuNoyau,
        };
        let jour = commandes::Jour {
            annee: 2026,
            mois: 10,
            jour: 25,
        };
        let jour_du_noyau = JourDuNoyau {
            annee: 2026,
            mois: 10,
            jour: 25,
        };
        let paires = [
            (
                "la requête par défaut",
                Requete::default(),
                RequeteDuNoyau::default(),
                false,
            ),
            (
                "un texte « … »",
                Requete {
                    texte: " … ".to_string(),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    texte: " … ".to_string(),
                    ..RequeteDuNoyau::default()
                },
                false,
            ),
            (
                "un texte de blancs",
                Requete {
                    texte: " \t\u{a0} ".to_string(),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    texte: " \t\u{a0} ".to_string(),
                    ..RequeteDuNoyau::default()
                },
                false,
            ),
            (
                "un texte « eglise »",
                Requete {
                    texte: "eglise".to_string(),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    texte: "eglise".to_string(),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "des blancs et un seuil de durée",
                Requete {
                    texte: "  ".to_string(),
                    plus_de_secondes: Some(30),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    texte: "  ".to_string(),
                    plus_de_secondes: Some(30),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "du",
                Requete {
                    du: Some(jour.clone()),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    du: Some(jour_du_noyau),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "au",
                Requete {
                    au: Some(jour),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    au: Some(jour_du_noyau),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "etiquette",
                Requete {
                    etiquette: Some("pont".to_string()),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    etiquette: Some("pont".to_string()),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "plus_de_secondes",
                Requete {
                    plus_de_secondes: Some(60),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    plus_de_secondes: Some(60),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "etat",
                Requete {
                    etat: Some(commandes::Etat::NonReliee),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    etat: Some(EtatDuNoyau::NonReliee),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "provenance",
                Requete {
                    provenance: Some(commandes::ProvenanceDeLIdee::Importee),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    provenance: Some(ProvenanceDuNoyau::Importee),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
            (
                "contenu",
                Requete {
                    contenu: Some(Contenu::Prise),
                    ..Requete::default()
                },
                RequeteDuNoyau {
                    contenu: Some(ContenuDuNoyau::Prise),
                    ..RequeteDuNoyau::default()
                },
                true,
            ),
        ];
        for (nom, des_commandes, du_noyau, pose) in paires {
            assert_eq!(
                pose_un_critere(&des_commandes),
                !du_noyau.est_vide(),
                "{nom}"
            );
            assert_eq!(pose_un_critere(&des_commandes), pose, "{nom}");
        }
    }

    /// Une requête enregistrée inventée, lisible ou, sans requête, d'une version plus récente.
    fn enregistree(nom: &str, requete: Option<Requete>) -> RequeteEnregistree {
        RequeteEnregistree {
            id: "requete-inventee".to_string(),
            nom: nom.to_string(),
            requete,
        }
    }

    #[test]
    fn une_requete_plus_recente_se_nomme_et_ne_se_lance_pas() {
        let owner = Owner::new();
        owner.set();
        let lisible = Requete {
            texte: "pont".into(),
            etat: Some(commandes::Etat::NonReliee),
            ..Requete::default()
        };
        // Une requête lisible se nomme, sans note, et se lance : ses réglages se posent.
        assert_eq!(
            ce_que_dit_la_requete(&enregistree("Les ponts", Some(lisible.clone()))),
            ("Les ponts".to_string(), None)
        );
        let etat = EtatDeLaRequete::nouveau();
        assert!(lancer(
            etat,
            &enregistree("Les ponts", Some(lisible.clone()))
        ));
        assert_eq!(etat.requete(), lisible);
        // Une requête d'une version plus récente se nomme, avec sa note, et ne change rien.
        let plus_recente = enregistree("Venue d'ailleurs", None);
        assert_eq!(
            ce_que_dit_la_requete(&plus_recente),
            (
                "Venue d'ailleurs".to_string(),
                Some("Cette requête vient d'une version plus récente de Pupitre.")
            )
        );
        assert!(!lancer(etat, &plus_recente));
        assert_eq!(etat.requete(), lisible);
    }

    /// Une requête lancée inventée, ses réglages posés sur l'écran par un clic.
    fn requete_lancee() -> Requete {
        Requete {
            texte: "refrain".into(),
            du: jour_du_champ("2026-10-25"),
            etiquette: Some("pont".into()),
            plus_de_secondes: Some(60),
            ..Requete::default()
        }
    }

    #[test]
    fn supprimer_la_requete_en_cours_laisse_ses_criteres() {
        let owner = Owner::new();
        owner.set();
        let boite = EtatDeLaBoite::nouveau();
        let etat = EtatDeLaRequete::nouveau();
        let lancee = requete_lancee();
        let enregistree = enregistree("Mes refrains", Some(lancee.clone()));
        boite.requetes.set(Some(vec![enregistree.clone()]));
        boite.choisir_et_lancer(etat, &enregistree);
        assert_eq!(boite.requete_choisie.get(), Some(enregistree.id.clone()));
        // Le geste « Supprimer » fait : la liste des requêtes enregistrées change, le choix
        // s'oublie, le focus va au champ « Nom » ; la requête de l'écran, non.
        let cible = boite.apres_un_geste_de_requete(
            &GesteDeRequete::Supprimer {
                requete: enregistree.id.clone(),
            },
            true,
        );
        assert_eq!(cible, Some(Cible::ChampDeNom));
        assert_eq!(boite.requetes.get(), Some(Vec::new()));
        assert_eq!(boite.requete_choisie.get(), None);
        assert_eq!(boite.nom_de_la_requete.get(), "");
        assert_eq!(etat.requete(), lancee);
        assert_eq!(etat.texte.get(), "refrain");
        assert_eq!(etat.du.get(), "2026-10-25");
        assert_eq!(etat.etiquette.get(), Some("pont".to_string()));
        assert_eq!(etat.duree.get(), 2);
    }

    #[test]
    fn un_refus_de_supprimer_laisse_le_choix_et_le_focus() {
        let owner = Owner::new();
        owner.set();
        let boite = EtatDeLaBoite::nouveau();
        let etat = EtatDeLaRequete::nouveau();
        let enregistree = enregistree("Mes refrains", Some(requete_lancee()));
        boite.requetes.set(Some(vec![enregistree.clone()]));
        boite.choisir_et_lancer(etat, &enregistree);
        let cible = boite.apres_un_geste_de_requete(
            &GesteDeRequete::Supprimer {
                requete: enregistree.id.clone(),
            },
            false,
        );
        assert_eq!(cible, None);
        assert_eq!(boite.requete_choisie.get(), Some(enregistree.id.clone()));
        assert_eq!(boite.requetes.get(), Some(vec![enregistree]));
        assert_eq!(boite.nom_de_la_requete.get(), "Mes refrains");
        assert_eq!(etat.requete(), requete_lancee());
    }

    #[test]
    fn renommer_fait_laisse_le_choix_sur_le_meme_identifiant() {
        let owner = Owner::new();
        owner.set();
        let boite = EtatDeLaBoite::nouveau();
        let etat = EtatDeLaRequete::nouveau();
        let enregistree = enregistree("Mes refrains", Some(requete_lancee()));
        boite.choisir_et_lancer(etat, &enregistree);
        boite.nom_de_la_requete.set("Refrains du pont".to_string());
        let cible = boite.apres_un_geste_de_requete(&GesteDeRequete::Renommer, true);
        assert_eq!(cible, None);
        assert_eq!(boite.requete_choisie.get(), Some(enregistree.id.clone()));
        assert_eq!(boite.nom_de_la_requete.get(), "Refrains du pont");
        assert_eq!(etat.requete(), requete_lancee());
        // Refusé, de même.
        let cible = boite.apres_un_geste_de_requete(&GesteDeRequete::Renommer, false);
        assert_eq!(cible, None);
        assert_eq!(boite.requete_choisie.get(), Some(enregistree.id));
    }

    #[test]
    fn enregistrer_fait_vide_le_champ_qui_porte_encore_le_nom_envoye() {
        let owner = Owner::new();
        owner.set();
        let boite = EtatDeLaBoite::nouveau();
        boite.nom_de_la_requete.set("Mes refrains".to_string());
        let geste = GesteDeRequete::Enregistrer {
            nom: "Mes refrains".to_string(),
        };
        // Refusé : le champ garde le nom.
        assert_eq!(boite.apres_un_geste_de_requete(&geste, false), None);
        assert_eq!(boite.nom_de_la_requete.get(), "Mes refrains");
        // Fait, le champ ayant changé entre-temps : il garde ce qui y est tapé.
        boite.nom_de_la_requete.set("Autre nom".to_string());
        assert_eq!(boite.apres_un_geste_de_requete(&geste, true), None);
        assert_eq!(boite.nom_de_la_requete.get(), "Autre nom");
        // Fait, le champ portant encore le nom envoyé : il se vide.
        boite.nom_de_la_requete.set("Mes refrains".to_string());
        assert_eq!(boite.apres_un_geste_de_requete(&geste, true), None);
        assert_eq!(boite.nom_de_la_requete.get(), "");
    }

    #[test]
    fn choisir_une_requete_lisible_pose_ses_reglages() {
        let owner = Owner::new();
        owner.set();
        let boite = EtatDeLaBoite::nouveau();
        let etat = EtatDeLaRequete::nouveau();
        let enregistree = enregistree("Mes refrains", Some(requete_lancee()));
        boite.choisir_et_lancer(etat, &enregistree);
        assert_eq!(boite.requete_choisie.get(), Some(enregistree.id));
        assert_eq!(boite.nom_de_la_requete.get(), "Mes refrains");
        assert_eq!(etat.requete(), requete_lancee());
    }

    #[test]
    fn choisir_une_requete_plus_recente_la_choisit_sans_rien_poser() {
        let owner = Owner::new();
        owner.set();
        let boite = EtatDeLaBoite::nouveau();
        let etat = EtatDeLaRequete::nouveau();
        let deja = Requete {
            texte: "pont".into(),
            ..Requete::default()
        };
        etat.poser(&deja);
        let plus_recente = enregistree("Venue d'ailleurs", None);
        boite.choisir_et_lancer(etat, &plus_recente);
        assert_eq!(boite.requete_choisie.get(), Some(plus_recente.id));
        assert_eq!(etat.requete(), deja);
    }

    #[test]
    fn une_liste_vide_de_requetes_enregistrees_le_dit() {
        // Lue et vide : un constat, de la forme d'« Aucune idée retirée. ».
        let aucune: [RequeteEnregistree; 0] = [];
        assert_eq!(
            constat_des_requetes(Some(aucune.as_slice())),
            Some("Aucune requête enregistrée.")
        );
        // Pas encore lue, ou sa lecture refusée : rien, puisque rien n'en est su.
        assert_eq!(constat_des_requetes(None), None);
        // Une requête, même d'une version plus récente : la liste la montre, sans constat.
        let une = [enregistree("Venue d'ailleurs", None)];
        assert_eq!(constat_des_requetes(Some(une.as_slice())), None);
    }

    #[test]
    fn une_requete_plus_recente_dit_qu_elle_ne_se_lance_pas() {
        assert_eq!(
            PRECISION_D_UNE_REQUETE_PLUS_RECENTE,
            "Elle ne se lance pas : choisie, elle se renomme ou se supprime."
        );
    }

    #[test]
    fn enregistrer_ne_s_offre_qu_a_une_requete_posee() {
        let posee = Requete {
            texte: "refrain".into(),
            ..Requete::default()
        };
        assert!(enregistrer_possible(&posee, "Mes refrains"));
        assert!(!enregistrer_possible(&Requete::default(), "Mes refrains"));
        assert!(!enregistrer_possible(&posee, ""));
        assert!(!enregistrer_possible(&posee, " 	 "));
        // Un texte sans mot ne pose aucun critère.
        let sans_mot = Requete {
            texte: " … ".into(),
            ..Requete::default()
        };
        assert!(!enregistrer_possible(&sans_mot, "Mes refrains"));
    }

    #[test]
    fn chaque_ligne_du_journal_dit_son_heure_et_son_message() {
        // L'heure 0, celle de la genèse : « — ».
        let genese = EntreeDuJournal {
            message: "Genèse du fonds".to_string(),
            heure: 0,
        };
        assert_eq!(ligne_du_journal(&genese, 0), "— · Genèse du fonds");
        // Une heure au décalage de son instant : le jour, puis l'heure du jour sur deux chiffres.
        let notee = EntreeDuJournal {
            message: "Idée du 25 octobre notée".to_string(),
            heure: 1_792_888_200,
        };
        assert_eq!(
            ligne_du_journal(&notee, 120),
            "25 oct. 2026 · 02:30 · Idée du 25 octobre notée"
        );
    }

    #[test]
    fn l_entete_du_journal_dit_son_nombre_et_son_empreinte() {
        let empreinte = "570cda0f1e2d3c4b5a69788796a5b4c3d2e1f00112233445566778899aabbccd";
        assert_eq!(
            entete_du_journal(1, empreinte),
            "1 entrée · empreinte 570cda"
        );
        assert_eq!(
            entete_du_journal(3, empreinte),
            "3 entrées · empreinte 570cda"
        );
    }
}
