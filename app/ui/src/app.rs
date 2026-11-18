//! L'établi : son état, sa page d'ouverture, et la place du bandeau, de l'en-tête, puis du fonds,
//! ou de la table et de la palette.

use commandes::{ETAT_DU_FONDS, EtatDuFonds, Refus};
use leptos::html;
use leptos::prelude::*;
use noyau::rendu::Instrument;
use noyau::table::{CoucheSombre, Position, Table, table};
use noyau::tonalite::Tonalite;

use crate::bandeau::{
    Bandeau, Page, PageDuBandeau, ReperesDuBandeau, VueDuBandeau, page_d_ouverture, rendre_le_focus,
};
use crate::boite::{BoiteDIdees, EtatDeLaBoite, dans_un_champ_de_texte};
use crate::entete::Entete;
use crate::natif;
use crate::panneau_palette::PanneauPalette;
use crate::recherche::{EtatDeLaRequete, slash_cherche};
use crate::vue_table::VueTable;

/// Ce que l'établi montre sous son en-tête : le fonds, ou la table d'INC-0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Vue {
    /// Le fonds, dont la zone « Boîte d'idées » porte la boîte entière.
    #[default]
    Fonds,
    /// La table de la tonalité, et sa palette.
    Table,
}

/// L'état de l'établi : la vue (le fonds), la tonalité (do majeur), la couche sombre (masquée),
/// l'instrument (les deux), la case touchée (aucune), la palette, ouverte ou fermée, le bandeau
/// (vide) et ses repères, et le dernier état du fonds reçu (aucun). Rien n'en est gardé d'une
/// ouverture à l'autre.
#[derive(Clone, Copy)]
pub struct Etat {
    /// Ce que l'établi montre sous son en-tête.
    pub vue: RwSignal<Vue>,
    pub tonalite: RwSignal<Tonalite>,
    pub couche: RwSignal<CoucheSombre>,
    pub instrument: RwSignal<Instrument>,
    pub touchee: RwSignal<Option<Position>>,
    pub palette: RwSignal<bool>,
    /// La table de la tonalité et de la couche choisies.
    pub table: Memo<Table>,
    /// Le bandeau : l'état du fonds, le refus d'un geste ou d'une lecture, les avertissements.
    pub bandeau: RwSignal<Bandeau>,
    /// Le dernier état du fonds reçu, à l'ouverture ou après une commande ; aucun avant.
    pub etat_du_fonds: RwSignal<Option<EtatDuFonds>>,
    /// Les repères du bandeau monté, qui reçoivent le focus après l'issue d'un geste.
    pub reperes_du_bandeau: ReperesDuBandeau,
    /// La requête en cours, que l'en-tête et le fonds partagent ; elle survit au changement de vue.
    pub(crate) requete: EtatDeLaRequete,
}

impl Etat {
    pub fn nouveau() -> Self {
        // Les 24 commencent par do majeur.
        let tonalite = RwSignal::new(Tonalite::les_24()[0]);
        let couche = RwSignal::new(CoucheSombre::Masquee);
        Etat {
            vue: RwSignal::new(Vue::default()),
            tonalite,
            couche,
            instrument: RwSignal::new(Instrument::LesDeux),
            touchee: RwSignal::new(None),
            palette: RwSignal::new(false),
            table: Memo::new(move |_| table(tonalite.get(), couche.get())),
            bandeau: RwSignal::new(Bandeau::default()),
            etat_du_fonds: RwSignal::new(None),
            reperes_du_bandeau: ReperesDuBandeau::nouveaux(),
            requete: EtatDeLaRequete::nouveau(),
        }
    }

    /// L'état du fonds reçu, à l'ouverture ou après une commande : le bandeau le suit (à
    /// l'ouverture, ses avertissements s'y ajoutent), et il devient le dernier reçu.
    pub fn suivre_le_fonds(self, etat: EtatDuFonds, a_l_ouverture: bool) {
        self.bandeau.update(|b| b.suivre(&etat, a_l_ouverture));
        self.etat_du_fonds.set(Some(etat));
    }

    /// L'issue d'un geste, ou le refus d'une lecture, jamais une lecture réussie : le bandeau la
    /// dit ; si le « Fermer » qui avait le focus vient de se cacher, le bandeau le rend
    /// (`rendre_le_focus`).
    pub fn apres_un_geste(self, issue: &Result<Option<String>, Refus>) {
        self.bandeau.update(|b| b.apres_un_geste(issue));
        rendre_le_focus(self.bandeau, self.reperes_du_bandeau);
    }

    /// Montre le fonds ou la table. Quitter la table ferme sa palette, et la case reste
    /// touchée : la table revenue ne rend pas le focus à une palette restée ouverte.
    pub fn choisir_vue(self, v: Vue) {
        if v != Vue::Table {
            self.palette.set(false);
        }
        self.vue.set(v);
    }

    /// Cherche `texte` : il devient le texte de la requête, et le fonds se montre.
    pub fn chercher(self, texte: String) {
        self.requete.texte.set(texte);
        self.choisir_vue(Vue::Fonds);
    }

    /// Une autre tonalité : la case touchée s'oublie, et la palette se ferme.
    pub fn choisir_tonalite(self, t: Tonalite) {
        self.tonalite.set(t);
        self.oublier_la_case();
    }

    /// La couche sombre affichée ou masquée : la case touchée s'oublie, et la palette se ferme.
    /// Une case sombre touchée ne survit donc pas au masquage de sa couche.
    pub fn choisir_couche(self, c: CoucheSombre) {
        self.couche.set(c);
        self.oublier_la_case();
    }

    /// Un autre instrument : la case reste touchée, sa cible en inverse, et la palette ouverte
    /// montre aussitôt les diagrammes de cet instrument.
    pub fn choisir_instrument(self, i: Instrument) {
        self.instrument.set(i);
    }

    /// Toucher une case : sa cible passe en inverse, et « Palette » s'affiche au bas de la case.
    /// Une palette ouverte suit la case touchée.
    pub fn toucher(self, p: Position) {
        self.touchee.set(Some(p));
    }

    /// Ouvrir la palette de la case touchée ; sans case touchée, rien ne s'ouvre.
    pub fn ouvrir_palette(self) {
        if self.touchee.get_untracked().is_some() {
            self.palette.set(true);
        }
    }

    pub fn fermer_palette(self) {
        self.palette.set(false);
    }

    fn oublier_la_case(self) {
        self.touchee.set(None);
        self.palette.set(false);
    }
}

/// L'établi. Au montage, l'état du fonds se lit (`etat_du_fonds`), et décide de la page
/// (`page_d_ouverture`) : l'établi, le bandeau en tête, puis l'en-tête, et dessous la vue choisie,
/// le fonds et sa boîte d'idées, ou la table et la place de la palette à sa droite ; ou la page du
/// seul bandeau. Rien ne s'affiche avant cette lecture. Lu, l'état du fonds passe au bandeau ; refusé,
/// son refus aussi. L'état de la boîte naît ici, et vit tant que l'établi est ouvert, que la
/// boîte se montre ou non.
#[component]
pub fn App() -> impl IntoView {
    let etat = Etat::nouveau();
    let boite = EtatDeLaBoite::nouveau();
    // Le champ de recherche de l'en-tête : `/` lui porte le focus, de toute vue.
    let champ_de_recherche = NodeRef::<html::Input>::new();
    let ecouteur = window_event_listener(leptos::ev::keydown, move |ev| {
        if slash_cherche(
            &ev.key(),
            ev.ctrl_key(),
            ev.alt_key(),
            ev.meta_key(),
            ev.repeat(),
            ev.is_composing(),
            dans_un_champ_de_texte(&ev),
        ) && let Some(champ) = champ_de_recherche.get_untracked()
        {
            ev.prevent_default();
            let _ = champ.focus();
        }
    });
    on_cleanup(move || ecouteur.remove());
    // Le bouton « Palette » de la case touchée : la palette fermée lui rend le focus.
    let bouton_palette = NodeRef::<html::Button>::new();
    let page = RwSignal::new(None::<Page>);
    leptos::task::spawn_local(async move {
        let lu = natif::invoquer::<EtatDuFonds>(ETAT_DU_FONDS, None).await;
        let ouverture = page_d_ouverture(&lu, natif::dans_l_application());
        match lu {
            Ok(fonds) => etat.suivre_le_fonds(fonds, true),
            Err(refus) => etat.apres_un_geste(&Err(refus)),
        }
        page.set(Some(ouverture));
    });
    move || match page.get() {
        None => ().into_any(),
        Some(Page::BandeauSeul(texte)) => view! { <PageDuBandeau texte=texte /> }.into_any(),
        Some(Page::Etabli) => view! {
            <div class="etabli">
                <VueDuBandeau bandeau=etat.bandeau reperes=etat.reperes_du_bandeau />
                <Entete etat=etat recherche=champ_de_recherche />
                {move || match etat.vue.get() {
                    Vue::Fonds => view! { <BoiteDIdees etat=etat boite=boite /> }.into_any(),
                    Vue::Table => view! {
                        <main class="plan">
                            <VueTable etat=etat bouton_palette=bouton_palette />
                            <PanneauPalette etat=etat bouton_palette=bouton_palette />
                        </main>
                    }
                    .into_any(),
                }}
            </div>
        }
        .into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noyau::table::Rangee;

    /// A7, le V7/ii de do majeur, au-dessus de ii.
    const A7: Position = Position {
        rangee: Rangee::Dessus,
        rang: 1,
    };

    fn touchee_et_palette_ouverte() -> Etat {
        let etat = Etat::nouveau();
        etat.toucher(A7);
        etat.ouvrir_palette();
        assert_eq!(etat.touchee.get_untracked(), Some(A7));
        assert!(etat.palette.get_untracked());
        etat
    }

    #[test]
    fn l_etabli_s_ouvre_en_do_majeur_couche_masquee_pour_les_deux_instruments() {
        let etat = Etat::nouveau();
        assert_eq!(etat.tonalite.get_untracked().nom(), "do majeur");
        assert_eq!(etat.couche.get_untracked(), CoucheSombre::Masquee);
        assert_eq!(etat.instrument.get_untracked(), Instrument::LesDeux);
        assert_eq!(etat.touchee.get_untracked(), None);
        assert!(!etat.palette.get_untracked());
    }

    #[test]
    fn sans_case_touchee_la_palette_ne_s_ouvre_pas() {
        let etat = Etat::nouveau();
        etat.ouvrir_palette();
        assert!(!etat.palette.get_untracked());
    }

    #[test]
    fn changer_de_tonalite_ou_de_couche_oublie_la_case_et_ferme_la_palette() {
        let etat = touchee_et_palette_ouverte();
        etat.choisir_tonalite(Tonalite::les_24()[1]);
        assert_eq!(etat.touchee.get_untracked(), None);
        assert!(!etat.palette.get_untracked());
        assert_eq!(etat.table.get_untracked().tonalite.nom(), "la mineur");

        let etat = touchee_et_palette_ouverte();
        etat.choisir_couche(CoucheSombre::Affichee);
        assert_eq!(etat.touchee.get_untracked(), None);
        assert!(!etat.palette.get_untracked());
        assert_eq!(
            etat.table.get_untracked().couche_sombre,
            CoucheSombre::Affichee
        );
    }

    #[test]
    fn changer_d_instrument_garde_la_case_et_la_palette() {
        let etat = touchee_et_palette_ouverte();
        etat.choisir_instrument(Instrument::Guitare);
        assert_eq!(etat.touchee.get_untracked(), Some(A7));
        assert!(etat.palette.get_untracked());
    }

    #[test]
    fn l_etabli_s_ouvre_sur_le_fonds_et_quitter_la_table_ferme_sa_palette() {
        let etat = touchee_et_palette_ouverte();
        assert_eq!(etat.vue.get_untracked(), Vue::Fonds);
        // La table, puis sa palette ouverte : la case touchée reste.
        etat.choisir_vue(Vue::Table);
        assert_eq!(etat.vue.get_untracked(), Vue::Table);
        etat.ouvrir_palette();
        assert!(etat.palette.get_untracked());
        // Revenir au fonds ferme la palette, la case reste touchée.
        etat.choisir_vue(Vue::Fonds);
        assert_eq!(etat.vue.get_untracked(), Vue::Fonds);
        assert!(!etat.palette.get_untracked());
        assert_eq!(etat.touchee.get_untracked(), Some(A7));
        // De retour à la table, la palette reste fermée, la case touchée.
        etat.choisir_vue(Vue::Table);
        assert_eq!(etat.vue.get_untracked(), Vue::Table);
        assert!(!etat.palette.get_untracked());
        assert_eq!(etat.touchee.get_untracked(), Some(A7));
    }

    #[test]
    fn chercher_depuis_la_table_passe_au_fonds() {
        let etat = Etat::nouveau();
        etat.choisir_vue(Vue::Table);
        etat.chercher("eglise".to_string());
        assert_eq!(etat.vue.get_untracked(), Vue::Fonds);
        assert_eq!(etat.requete.texte.get_untracked(), "eglise");
    }

    #[test]
    fn chercher_sur_le_fonds_n_ecrit_pas_la_vue() {
        // Le fonds montré, chaque frappe de la recherche laisse la vue : la fermeture qui rend la
        // vue ne rebâtit pas la boîte d'idées à chaque frappe. Un mémo qui lit la vue compte ses
        // calculs : une écriture, même de la même valeur, le ferait recalculer à sa lecture.
        let etat = Etat::nouveau();
        let calculs = StoredValue::new(0_usize);
        let vue = Memo::new(move |_| {
            calculs.update_value(|n| *n += 1);
            etat.vue.get()
        });
        assert_eq!(vue.get_untracked(), Vue::Fonds);
        etat.chercher("e".to_string());
        etat.chercher("eg".to_string());
        assert_eq!(vue.get_untracked(), Vue::Fonds);
        assert_eq!(calculs.get_value(), 1);
        assert_eq!(etat.requete.texte.get_untracked(), "eg");
        // Depuis la table, la recherche fait passer au fonds.
        etat.choisir_vue(Vue::Table);
        etat.chercher("egl".to_string());
        assert_eq!(vue.get_untracked(), Vue::Fonds);
    }

    #[test]
    fn l_etat_de_synchronisation_dit_sans_noeud_aucune_archive() {
        let texte = crate::entete::ETAT_DE_SYNCHRONISATION;
        assert_eq!(texte, "Sans n\u{153}ud \u{b7} aucune archive");
        assert_eq!(texte.chars().count(), 26);
        assert_eq!(texte.len(), 28);
    }

    /// La valeur d'une propriété de la feuille de style : `--page-l: 209.5;` donne 209,5.
    fn mesure(css: &str, propriete: &str) -> f32 {
        let debut = css
            .find(&format!("{propriete}:"))
            .unwrap_or_else(|| panic!("{propriete} manque à la feuille de style"))
            + propriete.len()
            + 1;
        let fin = debut + css[debut..].find(';').expect("une déclaration finit par ;");
        css[debut..fin].trim().parse().expect("une mesure en mm")
    }

    #[test]
    fn les_mesures_de_la_feuille_de_style_sont_celles_du_noyau() {
        use noyau::rendu::pages::{HAUTEUR, LARGEUR};
        use noyau::rendu::verso::fiche_d_accord;
        let css = include_str!("../style.css");
        assert_eq!(mesure(css, "--page-l"), LARGEUR);
        assert_eq!(mesure(css, "--page-h"), HAUTEUR);
        let t = table(Tonalite::les_24()[0], CoucheSombre::Masquee);
        let fiche = fiche_d_accord(&t.cases[0].accord, t.tonalite, Instrument::LesDeux)
            .expect("la fiche tient");
        assert!((mesure(css, "--fiche-l") - fiche.largeur).abs() < 1e-3);
    }
}
