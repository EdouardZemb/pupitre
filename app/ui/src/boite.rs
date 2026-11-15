//! La boîte d'idées à l'écran (CAP-02 ; questions 25 à 30). En tête, le champ « Noter une idée »,
//! toujours présent, le compte, et sous un filtre son nombre, « Ranger par : », « Contenu : » et
//! les gestes : relier, annuler, les idées retirées. Dessous, la liste en fenêtre, des lignes de
//! hauteur fixe que suit le seul événement de défilement (I18) ; à sa droite, le volet de l'idée
//! choisie, ou celui des idées retirées. Après chaque commande, l'interface relit l'état du fonds
//! et les fenêtres, et le bandeau reçoit l'issue ; seul le refus d'une lecture y passe, jamais une
//! lecture réussie. `Ctrl+Z`, hors d'un champ de texte, annule le dernier geste de tri.

use commandes::{
    ANNULER, Assiette, BOITE, Boite, COMPLETER, Completer, Contenu, DELIER, ETAT_DU_FONDS,
    EtatDuFonds, Etiqueter, FENETRE_MAX, Faite, Fenetre, LONGUEUR_D_UNE_LIGNE, LigneDeLaBoite,
    NOTER, Notee, Noter, OTER_UNE_ETIQUETTE, Ordre, POSER_UNE_ETIQUETTE, Provenance, RELIER,
    RETABLIR, RETIREES, RETIRER, Relier, VUE_D_UNE_IDEE, Viser, VueDUneIdee, VueDeLaBoite,
    VueDesRetirees,
};
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys::{HtmlElement, HtmlInputElement, HtmlTextAreaElement};
use noyau::donnees::boite::{
    date_de_la_liste, duree_affichee, etiquette_saisie, texte_du_compte, texte_du_filtre,
};
use noyau::donnees::idee;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;

use crate::app::Etat;
use crate::entete::choix;
use crate::natif;

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

/// Les choix de « Contenu : », chacun avec son libellé ; tout d'abord, sans filtre (question 22).
const CONTENUS: [(Option<Contenu>, &str); 6] = [
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

/// Vrai si la fenêtre `demandee` porte toutes les lignes qui se voient, même en partie, bornées
/// par `total` : le défilement n'a pas à en demander une autre. Sans hauteur de ligne lisible, rien
/// n'est couvert.
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
    let premiere = ((haut / hauteur_ligne_px).floor() as u64).min(total);
    let fin = (((haut + hauteur_vue_px.max(0.0)) / hauteur_ligne_px).ceil() as u64).min(total);
    let debut = u64::from(demandee.debut);
    debut <= premiere && fin <= debut + u64::from(demandee.nombre)
}

/// « Annuler : <message> », ou « Rien à annuler » (question 30).
pub(crate) fn libelle_d_annulation(a_annuler: Option<&str>) -> String {
    match a_annuler {
        Some(message) => format!("Annuler : {message}"),
        None => "Rien à annuler".to_string(),
    }
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
        .map(|etiquette| format!("#{etiquette}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn etat_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str {
    mot_de_l_etat(ligne.reliee)
}

pub(crate) fn provenance_de_la_ligne(ligne: &LigneDeLaBoite) -> &'static str {
    mot_de_la_provenance(&ligne.provenance)
}

/// Un compte de la boîte et son assiette, tels que le noyau les dit (`texte_du_compte`,
/// `texte_du_filtre`) : chaque compte en `usize`, `usize::MAX` s'il le dépasse.
fn au_noyau(compte: u64, assiette: &Assiette) -> (usize, idee::Assiette) {
    let en_usize = |n: u64| usize::try_from(n).unwrap_or(usize::MAX);
    (
        en_usize(compte),
        idee::Assiette {
            considerees: en_usize(assiette.considerees),
            portant: en_usize(assiette.portant),
            decidees: en_usize(assiette.decidees),
        },
    )
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
/// éléments d'une fenêtre à l'autre, et le focus qu'ils portent ; une ligne changée se redessine.
fn cle_de_la_ligne(ligne: &LigneDeLaBoite) -> String {
    serde_json::to_string(ligne).unwrap_or_else(|_| ligne.idee.clone())
}

/// Vrai si la touche part d'un champ de texte, reconnu à la cible de l'événement : un
/// `<textarea>`, ou un `<input>` qui n'est ni une case à cocher, ni un bouton radio, ni un bouton.
fn dans_un_champ_de_texte(ev: &leptos::ev::KeyboardEvent) -> bool {
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

/// Une fenêtre de la boîte, lue : la fenêtre demandée, si un filtre la restreignait, et la vue
/// rendue.
#[derive(Clone, Debug, PartialEq, Eq)]
struct BoiteLue {
    fenetre: Fenetre,
    filtree: bool,
    vue: VueDeLaBoite,
}

/// Une fenêtre des idées retirées, lue : la fenêtre demandée, et la vue rendue.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RetireesLues {
    fenetre: Fenetre,
    vue: VueDesRetirees,
}

/// Ce que montre le volet, à droite de la liste : rien, l'idée choisie, ou les idées retirées.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Volet {
    Aucun,
    Idee(String),
    Retirees,
}

/// L'idée du volet, lue : sa vue, et chacun de ses liens, l'identifiant de l'autre bout et ce qui
/// le nomme (`nom_d_une_idee_liee`), jamais l'identifiant.
#[derive(Clone, Debug, PartialEq, Eq)]
struct IdeeOuverte {
    vue: VueDUneIdee,
    liens: Vec<(String, String)>,
}

/// Les demandes d'une liste en fenêtre : la dernière fenêtre demandée, et son rang ; une réponse
/// qu'une demande plus récente a devancée s'ignore.
#[derive(Clone, Copy)]
struct Demandes {
    derniere: StoredValue<Option<Fenetre>>,
    rang: StoredValue<u64>,
}

impl Demandes {
    fn nouvelles() -> Self {
        Demandes {
            derniere: StoredValue::new(None),
            rang: StoredValue::new(0),
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

    /// Vrai si la dernière fenêtre demandée couvre les lignes vues (`couvre`).
    fn couvrent(self, haut: f64, vue: f64, ligne: f64, total: u64) -> bool {
        self.derniere.with_value(|derniere| {
            derniere
                .as_ref()
                .is_some_and(|d| couvre(d, haut, vue, ligne, total))
        })
    }
}

/// Ce que la boîte garde tant que l'établi est ouvert, qu'elle se montre ou non : l'ordre et le
/// contenu choisis, la fenêtre lue, les lignes cochées, le volet et son idée, les idées retirées
/// lues, et les champs en cours. Rien n'en est gardé d'une ouverture à l'autre. L'établi le crée,
/// hors de la boîte : une réponse arrivée la boîte cachée trouve où s'écrire.
#[derive(Clone, Copy)]
pub(crate) struct EtatDeLaBoite {
    /// Le rang de l'ordre dans `ORDRES`.
    ordre: RwSignal<usize>,
    /// Le rang du contenu dans `CONTENUS`.
    contenu: RwSignal<usize>,
    lue: RwSignal<Option<BoiteLue>>,
    demandes: Demandes,
    /// Les idées cochées pour relier, dans l'ordre où elles l'ont été.
    cochees: RwSignal<Vec<String>>,
    volet: RwSignal<Volet>,
    idee: RwSignal<Option<IdeeOuverte>>,
    /// Le rang de la dernière lecture de l'idée du volet.
    lectures_de_l_idee: StoredValue<u64>,
    retirees: RwSignal<Option<RetireesLues>>,
    demandes_des_retirees: Demandes,
    /// Le champ « Noter une idée ».
    note: RwSignal<String>,
    /// Une note est partie, sans réponse encore : `Entrée` n'en envoie pas une seconde.
    note_en_cours: StoredValue<bool>,
    /// Le champ « Texte complété » du volet, et l'idée dont il est parti.
    brouillon: RwSignal<String>,
    brouillon_de: StoredValue<Option<String>>,
    /// Le champ « Étiquette » du volet.
    etiquette: RwSignal<String>,
}

impl EtatDeLaBoite {
    /// La boîte rangée par date, sans filtre, rien de lu ni de coché, aucun volet.
    pub(crate) fn nouveau() -> Self {
        EtatDeLaBoite {
            ordre: RwSignal::new(0),
            contenu: RwSignal::new(0),
            lue: RwSignal::new(None),
            demandes: Demandes::nouvelles(),
            cochees: RwSignal::new(Vec::new()),
            volet: RwSignal::new(Volet::Aucun),
            idee: RwSignal::new(None),
            lectures_de_l_idee: StoredValue::new(0),
            retirees: RwSignal::new(None),
            demandes_des_retirees: Demandes::nouvelles(),
            note: RwSignal::new(String::new()),
            note_en_cours: StoredValue::new(false),
            brouillon: RwSignal::new(String::new()),
            brouillon_de: StoredValue::new(None),
            etiquette: RwSignal::new(String::new()),
        }
    }
}

/// Lit la fenêtre `fenetre` de la boîte, dans l'ordre et sous le filtre choisis. Une réponse
/// qu'une demande plus récente a devancée s'ignore ; un refus va au bandeau.
async fn lire_la_boite(etat: Etat, boite: EtatDeLaBoite, fenetre: Fenetre) {
    let rang = boite.demandes.demander(&fenetre);
    let filtre = CONTENUS[boite.contenu.get_untracked()].0.clone();
    let filtree = filtre.is_some();
    let commande = Boite {
        ordre: ORDRES[boite.ordre.get_untracked()].0.clone(),
        filtre,
        debut: fenetre.debut,
        nombre: fenetre.nombre,
    };
    let lu = natif::invoquer::<VueDeLaBoite>(BOITE, argument(&commande)).await;
    if !boite.demandes.est_la_derniere(rang) {
        return;
    }
    match lu {
        Ok(vue) => boite.lue.set(Some(BoiteLue {
            fenetre,
            filtree,
            vue,
        })),
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
}

/// Lit la fenêtre `fenetre` des idées retirées, comme `lire_la_boite`.
async fn lire_les_retirees(etat: Etat, boite: EtatDeLaBoite, fenetre: Fenetre) {
    let rang = boite.demandes_des_retirees.demander(&fenetre);
    let lu = natif::invoquer::<VueDesRetirees>(RETIREES, argument(&fenetre)).await;
    if !boite.demandes_des_retirees.est_la_derniere(rang) {
        return;
    }
    match lu {
        Ok(vue) => boite.retirees.set(Some(RetireesLues { fenetre, vue })),
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
}

/// Lit l'idée `id`, puis chacune de ses idées liées, pour les nommer ; chaque refus va au bandeau.
/// La lecture finie, si l'idée est encore celle du volet et que nulle lecture plus récente ne l'a
/// devancée, elle devient l'idée du volet ; le champ « Texte complété » part de son texte, s'il
/// n'est pas déjà parti d'elle.
async fn lire_l_idee(etat: Etat, boite: EtatDeLaBoite, id: String) {
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
        let nom = match natif::invoquer::<VueDUneIdee>(
            VUE_D_UNE_IDEE,
            Some(serde_json::Value::String(autre.clone())),
        )
        .await
        {
            Ok(liee) => nom_d_une_idee_liee(&liee),
            Err(refus) => {
                etat.apres_un_geste(&Err(refus));
                "idée illisible".to_string()
            }
        };
        liens.push((autre.clone(), nom));
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
}

/// Après une commande : l'état du fonds, hors de l'ouverture (`suivre_le_fonds`), puis la dernière
/// fenêtre demandée de la boîte, celle des idées retirées si leur volet est ouvert, et l'idée du
/// volet.
async fn relire(etat: Etat, boite: EtatDeLaBoite) {
    match natif::invoquer::<EtatDuFonds>(ETAT_DU_FONDS, None).await {
        Ok(fonds) => etat.suivre_le_fonds(fonds, false),
        Err(refus) => etat.apres_un_geste(&Err(refus)),
    }
    if let Some(fenetre) = boite.demandes.derniere.get_value() {
        lire_la_boite(etat, boite, fenetre).await;
    }
    match boite.volet.get_untracked() {
        Volet::Retirees => {
            if let Some(fenetre) = boite.demandes_des_retirees.derniere.get_value() {
                lire_les_retirees(etat, boite, fenetre).await;
            }
        }
        Volet::Idee(id) => lire_l_idee(etat, boite, id).await,
        Volet::Aucun => {}
    }
}

/// Où rendre le focus, dès que la cible est montée dans le document : un élément détaché, celui
/// d'un volet fermé, ne le reçoit pas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cible {
    Liste,
    ListeDesRetirees,
    BoutonDesRetirees,
    TitreDeLIdee,
    TitreDesRetirees,
}

/// Ce que partagent les vues de la boîte : l'état de l'établi, celui de la boîte, le focus à
/// rendre, et les repères de ses éléments montés.
#[derive(Clone, Copy)]
struct Ecran {
    etat: Etat,
    boite: EtatDeLaBoite,
    focus: RwSignal<Option<Cible>>,
    liste: NodeRef<html::Div>,
    etalon: NodeRef<html::Div>,
    liste_des_retirees: NodeRef<html::Div>,
    bouton_des_retirees: NodeRef<html::Button>,
    titre_de_l_idee: NodeRef<html::H2>,
    titre_des_retirees: NodeRef<html::H2>,
}

impl Ecran {
    /// La fenêtre que demande la liste montée `liste` : celle de son défilement, si `forcer`, ou si
    /// la dernière demandée (`demandes`) ne porte pas toutes les lignes vues. Le total, s'il est
    /// connu et que rien ne force, la borne ; rien sans mesure.
    fn a_demander(
        self,
        liste: NodeRef<html::Div>,
        demandes: Demandes,
        total: Option<u64>,
        forcer: bool,
    ) -> Option<Fenetre> {
        let (haut, vue, ligne) = mesures(liste, self.etalon)?;
        let total = if forcer {
            u64::MAX
        } else {
            total.unwrap_or(u64::MAX)
        };
        (forcer || !demandes.couvrent(haut, vue, ligne, total))
            .then(|| fenetre(haut, vue, ligne, total))
    }

    /// Lit la fenêtre de la boîte que demande son défilement (`a_demander`).
    fn demander_la_boite(self, forcer: bool) {
        let total = self
            .boite
            .lue
            .with_untracked(|lue| lue.as_ref().map(|lue| lue.vue.retenues));
        if let Some(fenetre) = self.a_demander(self.liste, self.boite.demandes, total, forcer) {
            leptos::task::spawn_local(lire_la_boite(self.etat, self.boite, fenetre));
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
            leptos::task::spawn_local(lire_les_retirees(self.etat, self.boite, fenetre));
        }
    }

    /// Un autre ordre ou un autre contenu : la liste revient au début, et sa fenêtre se relit.
    fn au_debut(self) {
        if let Some(liste) = self.liste.get_untracked() {
            liste.set_scroll_top(0);
        }
        self.demander_la_boite(true);
    }

    /// Un geste : la commande `nom`, puis son issue au bandeau (`avertissement` tire celui d'un
    /// geste fait), puis `apres`, qui reçoit vrai si le geste est fait, puis la relecture de l'état
    /// du fonds et des fenêtres (`relire`).
    fn geste<R: DeserializeOwned + 'static>(
        self,
        nom: &'static str,
        commande: Option<serde_json::Value>,
        avertissement: fn(R) -> Option<String>,
        apres: impl FnOnce(bool) + 'static,
    ) {
        leptos::task::spawn_local(async move {
            let issue = natif::invoquer::<R>(nom, commande).await.map(avertissement);
            self.etat.apres_un_geste(&issue);
            apres(issue.is_ok());
            relire(self.etat, self.boite).await;
        });
    }

    /// Un geste qui rend `Faite` : un geste de tri, ou l'annulation.
    fn faire(
        self,
        nom: &'static str,
        commande: Option<serde_json::Value>,
        apres: impl FnOnce(bool) + 'static,
    ) {
        self.geste::<Faite>(nom, commande, |faite| faite.avertissement, apres);
    }

    /// Note l'idée `noter`. Faite, le champ se vide, s'il porte encore la note partie.
    fn noter(self, noter: Noter) {
        let (note, en_cours) = (self.boite.note, self.boite.note_en_cours);
        if en_cours.get_value() {
            return;
        }
        en_cours.set_value(true);
        let partie = noter.texte.clone();
        self.geste::<Notee>(
            NOTER,
            argument(&noter),
            |notee| notee.avertissement,
            move |faite| {
                en_cours.set_value(false);
                if faite {
                    note.update(|champ| {
                        if champ.trim() == partie {
                            champ.clear();
                        }
                    });
                }
            },
        );
    }

    /// Annule le dernier geste de tri ; sans geste à annuler, le refus le dit.
    fn annuler(self) {
        self.faire(ANNULER, None, |_| {});
    }

    /// Relie les deux idées cochées ; faite, aucune ne reste cochée.
    fn relier(self) {
        let cochees = self.boite.cochees.get_untracked();
        let [a, b] = cochees.as_slice() else {
            return;
        };
        let commande = argument(&Relier {
            a: a.clone(),
            b: b.clone(),
            decalage_minutes: natif::decalage_du_geste(),
        });
        let cochees = self.boite.cochees;
        self.faire(RELIER, commande, move |fait| {
            if fait {
                cochees.set(Vec::new());
            }
        });
    }

    /// Ouvre l'idée `id` dans le volet, et y porte le focus.
    fn ouvrir(self, id: String) {
        let deja = self
            .boite
            .volet
            .with_untracked(|volet| matches!(volet, Volet::Idee(i) if *i == id));
        if !deja {
            self.boite.idee.set(None);
            self.boite.etiquette.set(String::new());
            self.boite.volet.set(Volet::Idee(id.clone()));
            leptos::task::spawn_local(lire_l_idee(self.etat, self.boite, id));
        }
        self.focus.set(Some(Cible::TitreDeLIdee));
    }

    /// Ferme le volet ; le focus va à `cible`.
    fn fermer_le_volet(self, cible: Cible) {
        self.boite.volet.set(Volet::Aucun);
        self.focus.set(Some(cible));
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

    /// L'identifiant de l'idée du volet, s'il en montre une.
    fn idee_du_volet(self) -> Option<String> {
        self.boite.volet.with_untracked(|volet| match volet {
            Volet::Idee(id) => Some(id.clone()),
            _ => None,
        })
    }

    /// Complète le texte de l'idée du volet par le champ « Texte complété » ; fait, le champ
    /// repartira du texte relu.
    fn completer(self) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let commande = argument(&Completer {
            idee,
            texte: self.boite.brouillon.get_untracked(),
            decalage_minutes: natif::decalage_du_geste(),
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
            idee,
            etiquette: etiquette_saisie(&champ.get_untracked()),
            decalage_minutes: natif::decalage_du_geste(),
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
            idee,
            etiquette,
            decalage_minutes: natif::decalage_du_geste(),
        });
        let focus = self.focus;
        self.faire(OTER_UNE_ETIQUETTE, commande, move |fait| {
            if fait {
                focus.set(Some(Cible::TitreDeLIdee));
            }
        });
    }

    /// Délie l'idée du volet de l'idée `autre` ; son « Délier » parti, le focus va au titre du
    /// volet.
    fn delier(self, autre: String) {
        let Some(idee) = self.idee_du_volet() else {
            return;
        };
        let commande = argument(&Relier {
            a: idee,
            b: autre,
            decalage_minutes: natif::decalage_du_geste(),
        });
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
            decalage_minutes: natif::decalage_du_geste(),
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
                    .update(|cochees| cochees.retain(|c| *c != idee));
                boite.volet.set(Volet::Aucun);
                focus.set(Some(Cible::Liste));
            }
        });
    }

    /// Rétablit l'idée retirée `idee` ; sa ligne partie, le focus va à la liste des idées retirées.
    fn retablir(self, idee: String) {
        let commande = argument(&Viser {
            idee,
            decalage_minutes: natif::decalage_du_geste(),
        });
        let focus = self.focus;
        self.faire(RETABLIR, commande, move |fait| {
            if fait {
                focus.set(Some(Cible::ListeDesRetirees));
            }
        });
    }
}

/// La boîte d'idées. Montée, elle lit la fenêtre que sa liste montre ; un autre ordre ou un autre
/// contenu la ramène au début. La liste suit le seul événement de défilement (I18) : chaque ligne
/// a la hauteur fixe `--ligne`, que l'étalon mesure ; la piste a la hauteur de toutes les lignes,
/// et seules celles de la fenêtre lue s'y posent, à leur rang. Chaque contrôle est un bouton, une
/// case ou un champ, nommé ; le focus suit les volets (`Cible`).
#[component]
pub fn BoiteDIdees(etat: Etat, boite: EtatDeLaBoite) -> impl IntoView {
    let ecran = Ecran {
        etat,
        boite,
        focus: RwSignal::new(None),
        liste: NodeRef::new(),
        etalon: NodeRef::new(),
        liste_des_retirees: NodeRef::new(),
        bouton_des_retirees: NodeRef::new(),
        titre_de_l_idee: NodeRef::new(),
        titre_des_retirees: NodeRef::new(),
    };
    // Montés, la liste et l'étalon donnent la fenêtre à lire.
    Effect::new(move |_| {
        if ecran.liste.get().is_some() && ecran.etalon.get().is_some() {
            ecran.demander_la_boite(true);
        }
    });
    // Le focus demandé va à sa cible, dès qu'elle est montée dans le document.
    Effect::new(move |_| {
        let Some(cible) = ecran.focus.get() else {
            return;
        };
        let element: Option<HtmlElement> = match cible {
            Cible::Liste => ecran.liste.get().map(Into::into),
            Cible::ListeDesRetirees => ecran.liste_des_retirees.get().map(Into::into),
            Cible::BoutonDesRetirees => ecran.bouton_des_retirees.get().map(Into::into),
            Cible::TitreDeLIdee => ecran.titre_de_l_idee.get().map(Into::into),
            Cible::TitreDesRetirees => ecran.titre_des_retirees.get().map(Into::into),
        };
        if let Some(element) = element
            && element.is_connected()
        {
            let _ = element.focus();
            ecran.focus.set(None);
        }
    });
    // `Ctrl+Z`, hors d'un champ de texte, annule le dernier geste de tri tant que la boîte se
    // montre ; une touche tenue ne le répète pas.
    let ecouteur = window_event_listener(leptos::ev::keydown, move |ev| {
        if !ev.repeat() && ctrl_z_annule(&ev.key(), ev.ctrl_key(), dans_un_champ_de_texte(&ev)) {
            ev.prevent_default();
            ecran.annuler();
        }
    });
    on_cleanup(move || ecouteur.remove());

    let noter_au_clavier = move |ev: leptos::ev::KeyboardEvent| {
        if ev.is_composing() {
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
                let (non_reliees, assiette) =
                    au_noyau(lue.vue.non_reliees, &lue.vue.assiette_du_compte);
                let filtre = lue.filtree.then(|| {
                    let (retenues, assiette) =
                        au_noyau(lue.vue.retenues, &lue.vue.assiette_du_filtre);
                    texte_du_filtre(retenues, assiette)
                });
                (texte_du_compte(non_reliees, assiette), filtre)
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
    let contenus = CONTENUS
        .iter()
        .enumerate()
        .map(|(rang, (_, libelle))| {
            choix(
                "contenu",
                rang,
                libelle.to_string(),
                None,
                boite.contenu.into(),
                move |rang| {
                    boite.contenu.set(rang);
                    ecran.au_debut();
                },
            )
        })
        .collect_view();
    let deux_cochees = move || boite.cochees.with(|cochees| cochees.len() == 2);
    let annulation = move || {
        etat.etat_du_fonds
            .with(|fonds| libelle_d_annulation(fonds.as_ref().and_then(|f| f.a_annuler.as_deref())))
    };
    let retirees_ouvertes = move || boite.volet.with(|volet| *volet == Volet::Retirees);
    let idee_ouverte = move || boite.volet.with(|volet| matches!(volet, Volet::Idee(_)));
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
        <main class="boite" aria-label="Boîte d'idées">
            <div class="etalon" aria-hidden="true">
                <div node_ref=ecran.etalon></div>
            </div>
            <div class="boite-haut">
                <label class="champ noter">
                    "Noter une idée"
                    <input
                        type="text"
                        prop:value=move || boite.note.get()
                        on:input=move |ev| boite.note.set(event_target_value(&ev))
                        on:keydown=noter_au_clavier
                    />
                </label>
                <div class="comptes" aria-live="polite">
                    <p>{move || comptes().map(|(compte, _)| compte)}</p>
                    <p>{move || comptes().and_then(|(_, filtre)| filtre)}</p>
                </div>
                <fieldset class="reglage">
                    <legend>"Ranger par :"</legend>
                    <div class="choix-en-ligne">{ordres}</div>
                </fieldset>
                <fieldset class="reglage">
                    <legend>"Contenu :"</legend>
                    <div class="choix-en-ligne">{contenus}</div>
                </fieldset>
                <div class="gestes">
                    <button
                        type="button"
                        aria-disabled=move || (!deux_cochees()).then_some("true")
                        on:click=move |_| ecran.relier()
                    >
                        "Relier"
                    </button>
                    <button type="button" on:click=move |_| ecran.annuler()>
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
                    <div
                        class="liste"
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
            </div>
        </main>
    }
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
    let cellules = [
        date,
        texte,
        etiquettes_de_la_ligne(&ligne),
        duree_affichee(ligne.duree_ms),
        etat_de_la_ligne(&ligne).to_string(),
        provenance_de_la_ligne(&ligne).to_string(),
    ];
    let id = ligne.idee;
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
        move || boite.cochees.with(|cochees| cochees.contains(&id))
    };
    let ouvrir = {
        let id = id.clone();
        move |_| ecran.ouvrir(id.clone())
    };
    let cocher = move |ev: leptos::ev::Event| {
        let coche = event_target_checked(&ev);
        boite.cochees.update(|cochees| {
            cochees.retain(|c| *c != id);
            if coche {
                cochees.push(id.clone());
            }
        });
    };
    view! {
        <div class="ligne" role="listitem" aria-posinset=rang aria-setsize=total>
            <button
                type="button"
                class="choisir cellules"
                aria-current=move || choisie().then_some("true")
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
                    on:change=cocher
                />
            </span>
        </div>
    }
}

/// Une idée retirée, de rang `index` dans la fenêtre lue : sa date, son texte coupé sur une ligne,
/// ses étiquettes, et « Rétablir ».
fn ligne_retiree(ecran: Ecran, index: ReadSignal<usize>, ligne: LigneDeLaBoite) -> impl IntoView {
    let boite = ecran.boite;
    let date = date_locale(ligne.le);
    let texte = texte_de_la_ligne(&ligne);
    let nom_du_bouton = format!("Rétablir : {date} · {texte}");
    let cellules = [date, texte, etiquettes_de_la_ligne(&ligne)];
    let id = ligne.idee;
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
                on:click=move |_| ecran.retablir(id.clone())
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
    let retiree = move || {
        boite
            .idee
            .with(|idee| idee.as_ref().is_some_and(|i| i.vue.retiree.is_some()))
    };
    let poser_au_clavier = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" && !ev.is_composing() {
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
                <button type="button" on:click=move |_| ecran.fermer_le_volet(Cible::Liste)>
                    "Fermer"
                </button>
            </div>
            <p class="texte-entier">{texte}</p>
            <p>{ce_qu_elle_est}</p>
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
                                <button
                                    type="button"
                                    aria-label=nom
                                    on:click=move |_| ecran.oter(a_oter.clone())
                                >
                                    "Ôter"
                                </button>
                            </li>
                        }
                    }
                />
            </ul>
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
            <h3>"Liens"</h3>
            <p hidden=move || !liens().is_empty()>{CASE_VIDE}</p>
            <ul class="puces">
                <For
                    each=liens
                    key=|lien: &(String, String)| lien.clone()
                    children=move |(autre, nom): (String, String)| {
                        let nom_du_bouton = format!("Délier : {nom}");
                        view! {
                            <li>
                                <span>{nom}</span>
                                <button
                                    type="button"
                                    aria-label=nom_du_bouton
                                    on:click=move |_| ecran.delier(autre.clone())
                                >
                                    "Délier"
                                </button>
                            </li>
                        }
                    }
                />
            </ul>
            <div>
                <button type="button" on:click=move |_| ecran.retirer_ou_retablir()>
                    {move || if retiree() { "Rétablir" } else { "Retirer" }}
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
                    on:click=move |_| ecran.fermer_le_volet(Cible::BoutonDesRetirees)
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

#[cfg(test)]
mod tests {
    use super::*;
    use commandes::{Appareil, Provenance};

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
        // La fenêtre de 995 à 1 025 porte les lignes vues, de 1 000 à 1 020, et cinq lignes plus
        // bas encore, de 1 005 à 1 025.
        assert!(couverte(40_000.0));
        assert!(couverte(40_200.0));
        // Un demi-rang de plus, la ligne 1 025 paraît : le défilement demande une autre fenêtre.
        assert!(!couverte(40_220.0));
        // Six lignes plus haut, la ligne 994 paraît.
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
}
