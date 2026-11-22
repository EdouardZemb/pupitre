//! La boîte d'idées : son en-tête, ses critères, ses requêtes enregistrées et sa liste en
//! fenêtre.

use commandes::{LigneDeLaBoite, RequeteEnregistree};
use leptos::html;
use leptos::prelude::*;
use leptos::web_sys::{Element, HtmlElement};
use noyau::donnees::boite::{duree_affichee, texte_du_compte};
use wasm_bindgen::JsCast;

use crate::app::Etat;
use crate::entete::choix;
use crate::natif;
use crate::recherche::{DUREES, ETATS, PROVENANCES, echap_ferme, est_la_recherche};

use super::ecran::{Ecran, lire_les_requetes};
use super::etat::{EtatDeLaBoite, Volet, fonds_vide_dans};
use super::fenetre::Etalon;
use super::focus::{
    Cible, Commande, Liste, cible_quand_le_fonds_se_vide, cible_repliee, focus_de_la_ligne,
};
use super::gestes::{ctrl_z_annule, dans_un_champ_de_texte, envoi_au_clavier, note_a_envoyer};
use super::requetes::{
    PRECISION_D_UNE_REQUETE_PLUS_RECENTE, au_noyau, ce_que_dit_la_requete, constat_des_requetes,
    enregistrer_possible, texte_de_la_requete,
};
use super::textes::{
    CONTENUS, ORDRES, cle_de_la_ligne, date_locale, etat_de_la_ligne, etiquette_du_choix,
    etiquette_ecrite, etiquettes_a_choisir, etiquettes_de_la_ligne, libelle_d_annulation,
    nom_de_la_ligne, page_du_fonds_vide, provenance_de_la_ligne, texte_de_la_ligne,
};
use super::volets::{VoletDeLIdee, VoletDesRetirees, VoletDuJournal};

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
/// l'étalon de la boîte mesure (`Etalon::Boite`) ; la piste a la hauteur de toutes les lignes, et
/// seules celles de la fenêtre lue s'y posent, à leur rang. Chaque contrôle est un bouton, une case
/// ou un champ, nommé ; le focus suit les volets (`Cible`), entre dans une liste sur une ligne
/// entièrement vue (`entrer_dans_la_liste`), et revient à la ligne qui l'a perdu quand la fenêtre
/// se pose (`rendre_le_focus_perdu`).
#[component]
pub fn BoiteDIdees(etat: Etat, boite: EtatDeLaBoite) -> impl IntoView {
    let ecran = Ecran {
        etat,
        boite,
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
    };
    // Montés, la liste et l'étalon de la boîte donnent la fenêtre à lire. Chaque changement de la
    // requête en cours, son texte dans l'en-tête ou un critère, ramène la liste au début et la
    // relit (`au_debut`) : une demande par frappe, dont seule la dernière réponse compte.
    Effect::new(move |_| {
        let _suivie = etat.requete.requete();
        if ecran.liste.get().is_some() && ecran.etalon(Etalon::Boite).get().is_some() {
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
    let hauteur_de_la_piste = move || Etalon::Boite.hauteur(total());
    let haut_des_lignes = move || {
        let debut = boite
            .lue
            .with(|lue| lue.as_ref().map_or(0, |lue| lue.fenetre.debut));
        Etalon::Boite.hauteur(u64::from(debut))
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
                <div node_ref=ecran.etalon_de_la_boite></div>
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
