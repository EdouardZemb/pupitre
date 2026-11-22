//! L'écran de la boîte d'idées : ce que partagent ses vues, ses lectures et ses gestes.

use commandes::{
    ANNULER, BOITE, COMPLETER, Completer, DELIER, ENREGISTRER_UNE_REQUETE, ETAT_DU_FONDS,
    Enregistrer, EtatDuFonds, Etiqueter, Faite, Fenetre, JOURNAL, NOTER, Notee, Noter,
    OTER_UNE_ETIQUETTE, POSER_UNE_ETIQUETTE, RELIER, RENOMMER_UNE_REQUETE, REQUETES_ENREGISTREES,
    RETABLIR, RETIREES, RETIRER, Renommer, Requete, RequeteEnregistree, SUPPRIMER_UNE_REQUETE,
    Supprimer, VUE_D_UNE_IDEE, Viser, VueDUneIdee, VueDeLaBoite, VueDesRetirees, VueDuJournal,
};
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys::Node;
use noyau::donnees::boite::etiquette_saisie;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;

use crate::app::Etat;
use crate::natif;

use super::etat::{
    BoiteLue, Demandes, EtatDeLaBoite, IdeeOuverte, JournalLu, RetireesLues, Volet,
    cible_de_fermeture, fonds_vide_dans,
};
use super::fenetre::{
    Bord, Etalon, LIGNES_DE_L_ETALON, fenetre_a_demander, idee_de_rang, ligne_d_entree, mesures,
};
use super::focus::{
    Actif, Cible, Commande, Defilement, Gardee, Liste, actif, prendre, rendre_le_focus_perdu,
};
use super::gestes::{argument, deliaison, relier_les_cochees};
use super::requetes::{
    GesteDeRequete, commande_de_la_boite, enregistrer_possible, requete_en_cours,
};
use super::textes::{CASE_VIDE, ORDRES, nom_d_une_idee_liee};

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

/// Lit les requêtes enregistrées, rangées par nom par la commande ; un refus va au bandeau. Elle se
/// relit à l'ouverture de la boîte et après chaque geste fait d'une requête enregistrée.
pub(super) async fn lire_les_requetes(etat: Etat, boite: EtatDeLaBoite) {
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

/// Ce que partagent les vues de la boîte : l'état de l'établi, celui de la boîte, le focus à
/// rendre, et les repères de ses éléments montés.
#[derive(Clone, Copy)]
pub(super) struct Ecran {
    pub(super) etat: Etat,
    pub(super) boite: EtatDeLaBoite,
    pub(super) focus: RwSignal<Option<Cible>>,
    pub(super) champ_de_note: NodeRef<html::Input>,
    pub(super) champ_de_nom: NodeRef<html::Input>,
    pub(super) liste: NodeRef<html::Div>,
    pub(super) etalon_de_la_boite: NodeRef<html::Div>,
    pub(super) liste_des_retirees: NodeRef<html::Div>,
    pub(super) bouton_des_retirees: NodeRef<html::Button>,
    pub(super) liste_du_journal: NodeRef<html::Div>,
    pub(super) etalon_du_journal: NodeRef<html::Div>,
    pub(super) bouton_du_journal: NodeRef<html::Button>,
    pub(super) titre_de_l_idee: NodeRef<html::H2>,
    pub(super) titre_des_retirees: NodeRef<html::H2>,
    pub(super) titre_du_journal: NodeRef<html::H2>,
}

impl Ecran {
    /// L'étalon `etalon`, monté ou non (`Etalon`).
    pub(super) fn etalon(self, etalon: Etalon) -> NodeRef<html::Div> {
        match etalon {
            Etalon::Boite => self.etalon_de_la_boite,
            Etalon::Journal => self.etalon_du_journal,
        }
    }

    /// Aucune idée n'est interrogée : la page du fonds vide se montre (`fonds_vide_dans`).
    pub(super) fn fonds_vide(self) -> bool {
        self.boite
            .lue
            .with_untracked(|lue| fonds_vide_dans(lue.as_ref()))
    }

    /// Vrai si la liste de la boîte, montée, a le focus.
    pub(super) fn la_liste_a_le_focus(self) -> bool {
        let Some(liste) = self.liste.get_untracked() else {
            return false;
        };
        let liste: &Node = liste.as_ref();
        document()
            .active_element()
            .is_some_and(|actif| actif.is_same_node(Some(liste)))
    }

    /// La fenêtre que demande la liste montée `liste` (`fenetre_a_demander`), ses lignes mesurées
    /// par l'étalon `etalon`, d'après la dernière demandée (`demandes`) et le total lu ; rien sans
    /// mesure. Le défilement mesuré s'y note.
    fn a_demander(
        self,
        liste: NodeRef<html::Div>,
        etalon: Etalon,
        demandes: Demandes,
        total: Option<u64>,
        forcer: bool,
    ) -> Option<Fenetre> {
        let (haut, vue, ligne) = mesures(liste, self.etalon(etalon))?;
        demandes.defilement.set_value(haut);
        demandes.derniere.with_value(|derniere| {
            fenetre_a_demander(derniere.as_ref(), haut, vue, ligne, total, forcer)
        })
    }

    /// Lit la fenêtre de la boîte que demande son défilement (`a_demander`), réponse de la requête
    /// en cours, lue à la demande.
    pub(super) fn demander_la_boite(self, forcer: bool) {
        let total = self
            .boite
            .lue
            .with_untracked(|lue| lue.as_ref().map(|lue| lue.vue.retenues));
        let demandes = self.boite.demandes;
        if let Some(fenetre) = self.a_demander(self.liste, Etalon::Boite, demandes, total, forcer) {
            let requete = requete_en_cours(self.etat);
            leptos::task::spawn_local(lire_la_boite(
                self.etat, self.boite, self.focus, fenetre, requete,
            ));
        }
    }

    /// Lit la fenêtre des idées retirées que demande leur défilement (`a_demander`).
    pub(super) fn demander_les_retirees(self, forcer: bool) {
        let total = self
            .boite
            .retirees
            .with_untracked(|lues| lues.as_ref().map(|lues| lues.vue.total));
        let demandes = self.boite.demandes_des_retirees;
        let liste = self.liste_des_retirees;
        if let Some(fenetre) = self.a_demander(liste, Etalon::Boite, demandes, total, forcer) {
            leptos::task::spawn_local(lire_les_retirees(
                self.etat, self.boite, self.focus, fenetre,
            ));
        }
    }

    /// Lit la fenêtre du journal que demande son défilement (`a_demander`), ses lignes mesurées par
    /// l'étalon du journal, jamais par celui de la boîte.
    pub(super) fn demander_le_journal(self, forcer: bool) {
        let total = self
            .boite
            .journal
            .with_untracked(|lu| lu.as_ref().map(|lu| lu.vue.total));
        let demandes = self.boite.demandes_du_journal;
        let liste = self.liste_du_journal;
        if let Some(fenetre) = self.a_demander(liste, Etalon::Journal, demandes, total, forcer) {
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
    pub(super) fn entrer_dans_la_liste(self, ev: &leptos::ev::FocusEvent, liste: Liste, rang: u64) {
        if self
            .focus
            .with_untracked(|cible| matches!(cible, Some(Cible::Ligne { .. })))
        {
            return;
        }
        // Les lignes de la boîte et celles des idées retirées ont la même hauteur, que mesure
        // l'étalon de la boîte.
        let (conteneur, etalon, demandes) = match liste {
            Liste::Boite => (self.liste, Etalon::Boite, self.boite.demandes),
            Liste::Retirees => (
                self.liste_des_retirees,
                Etalon::Boite,
                self.boite.demandes_des_retirees,
            ),
        };
        let (Some(conteneur), Some(etalon)) = (
            conteneur.get_untracked(),
            self.etalon(etalon).get_untracked(),
        ) else {
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
    pub(super) fn au_debut(self) {
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
    pub(super) fn noter(self, noter: Noter) {
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
    pub(super) fn enregistrer_la_requete(self) {
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
    pub(super) fn renommer_la_requete(self) {
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
    pub(super) fn supprimer_la_requete(self) {
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
    pub(super) fn annuler(self) {
        self.faire(ANNULER, None, |_| {});
    }

    /// Relie les deux idées cochées, chacune datée au décalage local de sa naissance
    /// (`relier_les_cochees`) ; faite, aucune ne reste cochée.
    pub(super) fn relier(self) {
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
    pub(super) fn ouvrir(self, id: String) {
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
    pub(super) fn fermer_l_idee(self) {
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
    pub(super) fn fermer_par_echap(self) {
        if let Some(cible) = self.cible_de_fermeture() {
            self.fermer_le_volet(cible);
        }
    }

    /// La commande `commande` de la ligne de l'idée `idee` a pris le focus : elle est la ligne
    /// gardée.
    pub(super) fn garder(self, idee: &str, commande: Commande) {
        self.boite.gardee.set_value(Some(Gardee {
            idee: idee.to_string(),
            commande,
        }));
    }

    /// Ouvre le volet des idées retirées, qui prend le focus, ou le ferme.
    pub(super) fn basculer_les_retirees(self) {
        if self.boite.volet.with_untracked(|v| *v == Volet::Retirees) {
            self.boite.volet.set(Volet::Aucun);
        } else {
            self.boite.retirees.set(None);
            self.boite.volet.set(Volet::Retirees);
            self.focus.set(Some(Cible::TitreDesRetirees));
        }
    }

    /// Ouvre le volet du journal du fonds, qui prend le focus, ou le ferme.
    pub(super) fn basculer_le_journal(self) {
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
    pub(super) fn completer(self) {
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
    pub(super) fn poser_l_etiquette(self) {
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
    pub(super) fn oter(self, etiquette: String) {
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
    pub(super) fn delier(self, autre: String, le_de_l_autre: Option<i64>) {
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
    pub(super) fn retirer_ou_retablir(self) {
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
    pub(super) fn retablir(self, idee: String, le: i64) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recherche::{EtatDeLaRequete, jour_du_champ};
    use commandes::Contenu;

    /// Un écran de la boîte, ses repères jamais montés.
    fn ecran() -> Ecran {
        Ecran {
            etat: Etat::nouveau(),
            boite: EtatDeLaBoite::nouveau(),
            focus: RwSignal::new(None),
            champ_de_note: NodeRef::new(),
            champ_de_nom: NodeRef::new(),
            liste: NodeRef::new(),
            etalon_de_la_boite: NodeRef::new(),
            liste_des_retirees: NodeRef::new(),
            bouton_des_retirees: NodeRef::new(),
            liste_du_journal: NodeRef::new(),
            etalon_du_journal: NodeRef::new(),
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
}
