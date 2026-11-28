//! Où va le focus : la commande gardée d'une ligne, la cible rendue après une perte ou à la
//! fermeture d'un volet, et celle qui remplace la liste quand la page du fonds vide la cache.
//! Elle dit aussi ce que le volet offre au clavier, « Rétablir » et « Retirer », et le drapeau du
//! geste en vol.

use commandes::LigneDeLaBoite;
use leptos::prelude::*;
use leptos::web_sys::{HtmlElement, Node};

use super::ecran::Ecran;
use super::etat::EtatDeLaBoite;

/// Une liste en fenêtre de l'écran : celle de la boîte, ou celle des idées retirées.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Liste {
    Boite,
    Retirees,
}

/// Une commande d'une ligne : son bouton, qui ouvre le volet de l'idée, ou sa case à cocher, dans
/// la liste de la boîte ; « Rétablir », dans celle des idées retirées.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Commande {
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
pub(super) struct Gardee {
    pub(super) idee: String,
    pub(super) commande: Commande,
}

/// Ce qui a le focus dans le document : rien, ou `body`, où tombe le focus d'un élément déplacé ou
/// recréé ; ou un autre élément.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Actif {
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
pub(super) fn cible_apres_le_volet(idee: &str, lues: &[LigneDeLaBoite]) -> Cible {
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
pub(super) struct Offerts {
    pub(super) trier: bool,
    pub(super) dernier: &'static str,
}

/// D'une idée retirée, le volet n'offre que « Rétablir » (et « Fermer ») : le noyau refuserait
/// tout geste de tri qui la touche. D'une autre, les gestes de tri et « Retirer ».
pub(super) fn ce_que_le_volet_offre(retiree: bool) -> Offerts {
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
pub(super) fn prendre(en_vol: StoredValue<bool>) -> bool {
    if en_vol.get_value() {
        return false;
    }
    en_vol.set_value(true);
    true
}

/// Ce qui a le focus dans le document (`Actif`).
pub(super) fn actif() -> Actif {
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
pub(super) fn rendre_le_focus_perdu(
    boite: EtatDeLaBoite,
    focus: RwSignal<Option<Cible>>,
    liste: Liste,
) {
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
pub(super) enum Defilement {
    /// La liste garde son défilement : le focus rendu après une perte ne la fait pas bouger.
    Garde,
    /// La liste défile jusqu'à la ligne, comme `focus()` le fait.
    VersLaLigne,
}

/// Où rendre le focus, dès que la cible est montée dans le document : un élément détaché, celui
/// d'un volet fermé, ne le reçoit pas. Une commande d'une ligne, chaque ligne la résout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Cible {
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
pub(super) fn cible_repliee(cible: Cible, fonds_vide: bool) -> Cible {
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
pub(super) fn cible_quand_le_fonds_se_vide(vide: bool, liste_a_le_focus: bool) -> Option<Cible> {
    (vide && liste_a_le_focus).then_some(Cible::ChampDeNote)
}

/// Le focus demandé à une commande de la ligne de l'idée `idee` (`Cible::Ligne`) : `element` la
/// résout, et elle le prend dès qu'elle est montée dans le document. Si la cible garde le
/// défilement, celui que `focus()` a fait à sa liste se remet aussitôt (`defilement_a_remettre`),
/// dans la même exécution : l'événement de défilement qui en naît trouve la vue d'avant, que la
/// fenêtre posée couvre déjà.
pub(super) fn focus_de_la_ligne(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boite::tests::ligne_de;

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
}
