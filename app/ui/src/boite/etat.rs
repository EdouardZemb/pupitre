//! Ce que la boîte d'idées garde tant que l'établi est ouvert : les fenêtres lues et leurs
//! demandes, le volet et son idée.

use commandes::{
    Fenetre, LigneDeLaBoite, Requete, RequeteEnregistree, VueDUneIdee, VueDeLaBoite,
    VueDesRetirees, VueDuJournal,
};
use leptos::prelude::*;

use crate::recherche::EtatDeLaRequete;

use super::focus::{Cible, Gardee, cible_apres_le_volet};
use super::requetes::{GesteDeRequete, lancer};

/// Une fenêtre de la boîte, lue : la fenêtre demandée, la requête qui l'a lue, et la vue rendue,
/// sa réponse (`texte_de_la_requete`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct BoiteLue {
    pub(super) fenetre: Fenetre,
    pub(super) requete: Requete,
    pub(super) vue: VueDeLaBoite,
}

/// Une fenêtre des idées retirées, lue : la fenêtre demandée, et la vue rendue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RetireesLues {
    pub(super) fenetre: Fenetre,
    pub(super) vue: VueDesRetirees,
}

/// Une fenêtre du journal du fonds, lue : la fenêtre demandée, et la vue rendue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JournalLu {
    pub(super) fenetre: Fenetre,
    pub(super) vue: VueDuJournal,
}

/// Ce que montre le volet, à droite de la liste : rien, l'idée choisie, les idées retirées, ou le
/// journal du fonds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Volet {
    Aucun,
    Idee(String),
    Retirees,
    Journal,
}

/// Où va le focus quand le volet `volet` se ferme, par « Fermer » comme par Échap : au bouton de
/// la ligne qui a ouvert l'idée, si elle est dans les lignes lues `lues`, sinon à la liste
/// (`cible_apres_le_volet`) ; au bouton « Idées retirées » pour les retirées, au bouton « Journal
/// du fonds » pour le journal. `None` quand aucun volet n'est ouvert : rien ne se ferme.
pub(super) fn cible_de_fermeture(volet: &Volet, lues: &[LigneDeLaBoite]) -> Option<Cible> {
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
pub(super) struct IdeeOuverte {
    pub(super) vue: VueDUneIdee,
    pub(super) liens: Vec<(String, String, Option<i64>)>,
}

/// Les demandes d'une liste en fenêtre : la dernière fenêtre demandée, et son rang ; une réponse
/// qu'une demande plus récente a devancée s'ignore. Et le défilement de la liste à sa dernière
/// mesure, que `entrer_dans_la_liste` lui rend.
#[derive(Clone, Copy)]
pub(super) struct Demandes {
    pub(super) derniere: StoredValue<Option<Fenetre>>,
    rang: StoredValue<u64>,
    pub(super) defilement: StoredValue<f64>,
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
    pub(super) fn demander(self, fenetre: &Fenetre) -> u64 {
        self.derniere.set_value(Some(fenetre.clone()));
        self.rang.update_value(|rang| *rang = rang.wrapping_add(1));
        self.rang.get_value()
    }

    /// Vrai si la demande de rang `rang` est encore la dernière.
    pub(super) fn est_la_derniere(self, rang: u64) -> bool {
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
    pub(super) ordre: RwSignal<usize>,
    pub(super) lue: RwSignal<Option<BoiteLue>>,
    pub(super) demandes: Demandes,
    /// Les idées cochées pour relier, dans l'ordre où elles l'ont été, chacune avec sa naissance,
    /// qui date son jour dans le message du lien.
    pub(super) cochees: RwSignal<Vec<(String, i64)>>,
    pub(super) volet: RwSignal<Volet>,
    pub(super) idee: RwSignal<Option<IdeeOuverte>>,
    /// Le rang de la dernière lecture de l'idée du volet.
    pub(super) lectures_de_l_idee: StoredValue<u64>,
    pub(super) retirees: RwSignal<Option<RetireesLues>>,
    pub(super) demandes_des_retirees: Demandes,
    /// Le journal du fonds lu, et ses demandes.
    pub(super) journal: RwSignal<Option<JournalLu>>,
    pub(super) demandes_du_journal: Demandes,
    /// Le champ « Noter une idée ».
    pub(super) note: RwSignal<String>,
    /// Une note est partie, sans relecture encore : `Entrée` n'en envoie pas une seconde.
    pub(super) note_en_cours: StoredValue<bool>,
    /// Le champ « Texte complété » du volet, et l'idée dont il est parti.
    pub(super) brouillon: RwSignal<String>,
    pub(super) brouillon_de: StoredValue<Option<String>>,
    /// Le champ « Étiquette » du volet.
    pub(super) etiquette: RwSignal<String>,
    /// La commande d'une ligne qui a eu le focus en dernier, tant qu'aucun autre élément de la boîte
    /// ne l'a pris.
    pub(super) gardee: StoredValue<Option<Gardee>>,
    /// Un geste de tri, ou l'annulation, est parti, sans relecture encore : une seconde activation
    /// s'ignore.
    pub(super) geste_en_vol: StoredValue<bool>,
    /// Les requêtes enregistrées lues, rangées par nom par la commande qui les lit ; aucune avant
    /// la première lecture.
    pub(super) requetes: RwSignal<Option<Vec<RequeteEnregistree>>>,
    /// L'identifiant de la requête enregistrée choisie, sur laquelle portent « Renommer » et
    /// « Supprimer ».
    pub(super) requete_choisie: RwSignal<Option<String>>,
    /// Le champ « Nom » des requêtes enregistrées.
    pub(super) nom_de_la_requete: RwSignal<String>,
    /// Un geste d'une requête enregistrée est parti, sans relecture encore : une seconde
    /// activation s'ignore.
    pub(super) requete_en_vol: StoredValue<bool>,
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
    pub(super) fn choisir_et_lancer(self, etat: EtatDeLaRequete, requete: &RequeteEnregistree) {
        self.choisir_la_requete(requete);
        lancer(etat, requete);
    }

    /// Ce qui suit un geste d'une requête enregistrée, `fait` ou refusé, une fois son issue au
    /// bandeau : rend la cible où va le focus. Supprimer, fait : la requête sort de la liste, le
    /// choix s'oublie (`oublier_la_requete`), et le focus, qui était sur « Supprimer », va au champ
    /// « Nom ». Enregistrer, fait : le champ « Nom » se vide s'il porte encore le nom envoyé.
    /// Renommer garde le choix. Un refus ne change rien. La liste se relit après chaque geste, fait
    /// ou refusé (`Ecran::geste_de_requete`).
    pub(super) fn apres_un_geste_de_requete(
        self,
        geste: &GesteDeRequete,
        fait: bool,
    ) -> Option<Cible> {
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
    pub(super) fn montrer_l_idee(self, id: &str) -> bool {
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

/// Aucune idée n'est interrogée dans la fenêtre lue `lue` : la page du fonds vide se montre
/// (`page_du_fonds_vide`) ; rien de lu, elle ne se montre pas.
pub(super) fn fonds_vide_dans(lue: Option<&BoiteLue>) -> bool {
    lue.is_some_and(|lue| lue.vue.assiette_du_compte.considerees == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boite::tests::{enregistree, ligne_de};
    use crate::recherche::jour_du_champ;

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
}
