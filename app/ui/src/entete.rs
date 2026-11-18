//! L'en-tête de l'établi : le champ de recherche, l'état de synchronisation, le sélecteur
//! « Afficher », le fonds ou la table ; puis, avec la table, la tonalité, les 24 d'un geste ; la
//! couche sombre, affichée ou masquée ; l'instrument ; et l'export du cahier des tables.

use leptos::html;
use leptos::prelude::*;
use noyau::rendu::Instrument;
use noyau::table::CoucheSombre;
use noyau::tonalite::Tonalite;

use crate::app::{Etat, Vue};
use crate::export::BoutonExport;
use crate::recherche::ID_DE_LA_RECHERCHE;

/// L'état de synchronisation, toujours visible dans l'en-tête : aucun nœud n'est relié, et aucune
/// archive complète n'existe encore.
pub(crate) const ETAT_DE_SYNCHRONISATION: &str = "Sans nœud · aucune archive";

/// Un choix parmi d'autres, d'un geste : un bouton radio et son libellé.
pub(crate) fn choix<T>(
    groupe: &'static str,
    valeur: T,
    libelle: String,
    nom_complet: Option<String>,
    actuel: Signal<T>,
    choisir: impl Fn(T) + Copy + 'static,
) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    view! {
        <label class="choix">
            <input
                type="radio"
                name=groupe
                aria-label=nom_complet
                prop:checked=move || actuel.get() == valeur
                on:change=move |_| choisir(valeur)
            />
            {libelle}
        </label>
    }
}

#[component]
pub fn Entete(etat: Etat, recherche: NodeRef<html::Input>) -> impl IntoView {
    // Les 24 tonalités en deux rangs, chaque majeure au-dessus de sa relative, dans l'ordre du
    // cahier : la grille se remplit colonne par colonne, et les flèches du clavier suivent cet
    // ordre. Chaque bouton montre la tonique, et se nomme en entier.
    let tonalites = Tonalite::les_24().map(|t| {
        choix(
            "tonalite",
            t,
            t.tonique.solfege(),
            Some(t.nom()),
            etat.tonalite.into(),
            move |t| etat.choisir_tonalite(t),
        )
    });
    let couches = [
        (CoucheSombre::Masquee, "masquée"),
        (CoucheSombre::Affichee, "affichée"),
    ]
    .map(|(c, libelle)| {
        choix(
            "couche",
            c,
            libelle.to_string(),
            None,
            etat.couche.into(),
            move |c| etat.choisir_couche(c),
        )
    });
    let instruments = [
        (Instrument::Guitare, "guitare"),
        (Instrument::Clavier, "clavier"),
        (Instrument::LesDeux, "les deux"),
    ]
    .map(|(i, libelle)| {
        choix(
            "instrument",
            i,
            libelle.to_string(),
            None,
            etat.instrument.into(),
            move |i| etat.choisir_instrument(i),
        )
    });
    // Le sélecteur : le fonds, ou la table d'INC-0.
    let vues = [(Vue::Fonds, "Fonds"), (Vue::Table, "Table")].map(|(v, libelle)| {
        choix(
            "vue",
            v,
            libelle.to_string(),
            None,
            etat.vue.into(),
            move |v| etat.choisir_vue(v),
        )
    });
    // Les réglages de la table restent montés, cachés avec elle : un export en cours garde son
    // état.
    view! {
        <header class="entete">
            <div class="recherche">
                <label for=ID_DE_LA_RECHERCHE>"Chercher dans le texte des idées"</label>
                <input
                    id=ID_DE_LA_RECHERCHE
                    type="search"
                    node_ref=recherche
                    aria-keyshortcuts="/"
                    aria-describedby="aide-de-la-recherche"
                    prop:value=move || etat.requete.texte.get()
                    on:input=move |ev| etat.chercher(event_target_value(&ev))
                />
                <small id="aide-de-la-recherche">
                    "/ porte le focus ici, depuis le fonds comme depuis la table"
                </small>
            </div>
            <p class="etat-de-synchronisation">{ETAT_DE_SYNCHRONISATION}</p>
            <fieldset>
                <legend>"Afficher"</legend>
                <div class="choix-en-ligne">{vues.into_iter().collect_view()}</div>
            </fieldset>
            <div class="reglages-de-la-table" hidden=move || etat.vue.get() != Vue::Table>
                <fieldset>
                    <legend>"Tonalité"</legend>
                    <div class="tonalites">
                        <span class="mode" aria-hidden="true">"majeur"</span>
                        <span class="mode" aria-hidden="true">"mineur"</span>
                        {tonalites.into_iter().collect_view()}
                    </div>
                </fieldset>
                <fieldset>
                    <legend>"Couche sombre"</legend>
                    <div class="choix-en-ligne">{couches.into_iter().collect_view()}</div>
                </fieldset>
                <fieldset>
                    <legend>"Instrument"</legend>
                    <div class="choix-en-ligne">{instruments.into_iter().collect_view()}</div>
                </fieldset>
                <BoutonExport instrument=etat.instrument couche=etat.couche />
            </div>
        </header>
    }
}
