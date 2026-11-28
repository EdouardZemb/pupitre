//! Les volets, à droite de la liste : l'idée choisie, les idées retirées et le journal du
//! fonds.

use commandes::LigneDeLaBoite;
use leptos::html;
use leptos::prelude::*;

use crate::natif;

use super::ecran::Ecran;
use super::fenetre::Etalon;
use super::focus::{Commande, Liste, ce_que_le_volet_offre, focus_de_la_ligne};
use super::gestes::envoi_au_clavier;
use super::textes::{
    CASE_VIDE, cle_de_la_ligne, date_locale, entete_du_journal, etat_de_l_idee,
    etiquettes_de_la_ligne, ligne_du_journal, texte_de_la_ligne, texte_ou_tiret,
};

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
pub(super) fn VoletDeLIdee(ecran: Ecran) -> impl IntoView {
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
/// « Rétablir ». Ses lignes ont la hauteur de celles de la boîte, que mesure l'étalon de la boîte.
/// Monté, il lit la fenêtre que sa liste montre.
#[component]
pub(super) fn VoletDesRetirees(ecran: Ecran) -> impl IntoView {
    let boite = ecran.boite;
    Effect::new(move |_| {
        if ecran.liste_des_retirees.get().is_some() && ecran.etalon(Etalon::Boite).get().is_some() {
            ecran.demander_les_retirees(true);
        }
    });
    let total = move || {
        boite
            .retirees
            .with(|lues| lues.as_ref().map(|lues| lues.vue.total))
    };
    let hauteur_de_la_piste = move || Etalon::Boite.hauteur(total().unwrap_or(0));
    let haut_des_lignes = move || {
        let debut = boite
            .retirees
            .with(|lues| lues.as_ref().map_or(0, |lues| lues.fenetre.debut));
        Etalon::Boite.hauteur(u64::from(debut))
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
/// idées retirées. Chaque ligne se date au décalage de son instant (`natif::decalage_a`) ; elle
/// tient trois lignes de texte, où son message passe à la ligne, et ne se coupe qu'au-delà de la
/// troisième. Sa hauteur, `--ligne-du-journal`, n'est pas celle d'une ligne de la boîte : l'étalon
/// du journal, monté dans le volet, la mesure (`Etalon::Journal`), et la fenêtre, la piste et le
/// haut des lignes rendues se calculent sur elle. Monté, il lit la fenêtre que sa liste montre.
#[component]
pub(super) fn VoletDuJournal(ecran: Ecran) -> impl IntoView {
    let boite = ecran.boite;
    Effect::new(move |_| {
        if ecran.liste_du_journal.get().is_some() && ecran.etalon(Etalon::Journal).get().is_some() {
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
    let hauteur_de_la_piste = move || Etalon::Journal.hauteur(total().unwrap_or(0));
    let haut_des_lignes = move || {
        let debut = boite
            .journal
            .with(|lu| lu.as_ref().map_or(0, |lu| lu.fenetre.debut));
        Etalon::Journal.hauteur(u64::from(debut))
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
            <div class="etalon" aria-hidden="true">
                <div node_ref=ecran.etalon_du_journal></div>
            </div>
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
