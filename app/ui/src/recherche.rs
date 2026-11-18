//! La requête de l'écran et ses touches, en fonctions pures (CAP-06 ; questions 24 et 25).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "l'en-tête et le fonds s'en servent à la tâche 12 ; les tests"
    )
)]

use commandes::{Bornes, Etat, Jour, ProvenanceDeLIdee, Requete};
use leptos::prelude::*;
use noyau::donnees::requete::{self, Jour as JourDuNoyau};

use crate::boite::CONTENUS;

/// La requête en cours, que l'en-tête et le fonds partagent, tenue par `Etat` (`Etat::nouveau` la
/// crée) pour survivre au changement de vue : le texte cherché, la période (la valeur de chaque
/// champ de date), l'étiquette, et le rang choisi de chaque réglage.
#[derive(Clone, Copy)]
pub(crate) struct EtatDeLaRequete {
    pub texte: RwSignal<String>,
    pub du: RwSignal<String>,
    pub au: RwSignal<String>,
    pub etiquette: RwSignal<Option<String>>,
    pub duree: RwSignal<usize>,
    pub etat: RwSignal<usize>,
    pub provenance: RwSignal<usize>,
    pub contenu: RwSignal<usize>,
}

/// Les choix de « Durée », chacun avec son libellé : le seuil, en secondes, strictement dépassé.
pub(crate) const DUREES: [(Option<u64>, &str); 4] = [
    (None, "Toutes"),
    (Some(30), "Plus de 30 s"),
    (Some(60), "Plus d'une minute"),
    (Some(300), "Plus de 5 min"),
];

/// Les choix de « État », chacun avec son libellé.
pub(crate) const ETATS: [(Option<Etat>, &str); 3] = [
    (None, "Toutes"),
    (Some(Etat::NonReliee), "Non reliées"),
    (Some(Etat::Reliee), "Reliées"),
];

/// Les choix de « Provenance », chacun avec son libellé.
pub(crate) const PROVENANCES: [(Option<ProvenanceDeLIdee>, &str); 3] = [
    (None, "Toutes"),
    (Some(ProvenanceDeLIdee::Captee), "Captées"),
    (Some(ProvenanceDeLIdee::Importee), "Importées"),
];

/// Le rang de `valeur` dans `choix` ; 0, « Toutes », si elle n'y est pas.
fn rang_de<T: PartialEq>(choix: &[(Option<T>, &str)], valeur: &Option<T>) -> usize {
    choix.iter().position(|(v, _)| v == valeur).unwrap_or(0)
}

impl EtatDeLaRequete {
    /// Les réglages d'une requête vide : rien de cherché, chaque choix à « Toutes » ou « tout ».
    pub(crate) fn nouveau() -> Self {
        EtatDeLaRequete {
            texte: RwSignal::new(String::new()),
            du: RwSignal::new(String::new()),
            au: RwSignal::new(String::new()),
            etiquette: RwSignal::new(None),
            duree: RwSignal::new(0),
            etat: RwSignal::new(0),
            provenance: RwSignal::new(0),
            contenu: RwSignal::new(0),
        }
    }

    /// La requête que disent les réglages (le contenu par `boite::CONTENUS`).
    pub(crate) fn requete(self) -> Requete {
        Requete {
            texte: self.texte.get(),
            du: jour_du_champ(&self.du.get()),
            au: jour_du_champ(&self.au.get()),
            etiquette: self.etiquette.get(),
            plus_de_secondes: DUREES.get(self.duree.get()).and_then(|(seuil, _)| *seuil),
            etat: ETATS
                .get(self.etat.get())
                .and_then(|(etat, _)| etat.clone()),
            provenance: PROVENANCES
                .get(self.provenance.get())
                .and_then(|(provenance, _)| provenance.clone()),
            contenu: CONTENUS
                .get(self.contenu.get())
                .and_then(|(contenu, _)| contenu.clone()),
        }
    }

    /// Les réglages qui disent `requete`, ceux d'un jour illisible ou d'un seuil absent de `DUREES`
    /// vidés.
    pub(crate) fn poser(self, requete: &Requete) {
        self.texte.set(requete.texte.clone());
        self.du.set(champ_du_jour(requete.du.as_ref()));
        self.au.set(champ_du_jour(requete.au.as_ref()));
        self.etiquette.set(requete.etiquette.clone());
        self.duree.set(rang_de(&DUREES, &requete.plus_de_secondes));
        self.etat.set(rang_de(&ETATS, &requete.etat));
        self.provenance
            .set(rang_de(&PROVENANCES, &requete.provenance));
        self.contenu.set(rang_de(&CONTENUS, &requete.contenu));
    }
}

/// L'`id` du champ de recherche de l'en-tête, seul champ où Échap garde son sens du navigateur.
pub(crate) const ID_DE_LA_RECHERCHE: &str = "recherche";

/// La cible d'un événement, dite par son `id`, est le champ de recherche ; tout autre champ de
/// texte, ou une cible sans `id`, ne l'est pas.
pub(crate) fn est_la_recherche(id_de_la_cible: Option<&str>) -> bool {
    id_de_la_cible == Some(ID_DE_LA_RECHERCHE)
}

/// Le jour d'un champ de date : « AAAA-MM-JJ », ou `None` pour un champ vide ou incomplet.
pub(crate) fn jour_du_champ(valeur: &str) -> Option<Jour> {
    JourDuNoyau::depuis_iso(valeur).map(|jour| Jour {
        annee: jour.annee,
        mois: jour.mois,
        jour: jour.jour,
    })
}

/// La valeur du champ de date qui dit `jour` ; vide pour un jour absent ou illisible.
fn champ_du_jour(jour: Option<&Jour>) -> String {
    jour.map(|jour| {
        JourDuNoyau {
            annee: jour.annee,
            mois: jour.mois,
            jour: jour.jour,
        }
        .en_iso()
    })
    .filter(|iso| JourDuNoyau::depuis_iso(iso).is_some())
    .unwrap_or_default()
}

/// Les bornes de la période de `requete`, au décalage que `decalage` dit de chaque instant.
pub(crate) fn bornes_de_la_requete(requete: &Requete, decalage: impl Fn(i64) -> i32) -> Bornes {
    let jour_du_noyau = |jour: &Jour| JourDuNoyau {
        annee: jour.annee,
        mois: jour.mois,
        jour: jour.jour,
    };
    let bornes = requete::bornes_de_la_periode(
        requete.du.as_ref().map(jour_du_noyau),
        requete.au.as_ref().map(jour_du_noyau),
        decalage,
    );
    Bornes {
        des: bornes.des,
        avant: bornes.avant,
    }
}

/// `/` cherche (question 24) : la touche produite `/`, sans Ctrl, Alt ni Méta (AltGr arrive comme
/// Ctrl + Alt), ni tenue, ni en composition, hors d'un champ de texte ; Maj admise.
pub(crate) fn slash_cherche(
    touche: &str,
    ctrl: bool,
    alt: bool,
    meta: bool,
    tenue: bool,
    en_composition: bool,
    dans_un_champ: bool,
) -> bool {
    touche == "/" && !ctrl && !alt && !meta && !tenue && !en_composition && !dans_un_champ
}

/// Échap ferme le volet ouvert (question 25) : la touche `Escape`, un volet ouvert, hors d'une
/// composition, hors de la recherche, où Échap garde son sens du navigateur : vider le champ, sans
/// fermer de volet.
pub(crate) fn echap_ferme(
    touche: &str,
    en_composition: bool,
    volet_ouvert: bool,
    dans_la_recherche: bool,
) -> bool {
    touche == "Escape" && volet_ouvert && !en_composition && !dans_la_recherche
}

#[cfg(test)]
mod tests {
    use commandes::Contenu;

    use super::*;

    #[test]
    fn slash_porte_le_focus_hors_d_un_champ_seulement() {
        assert!(slash_cherche("/", false, false, false, false, false, false)); // AZERTY : Maj + :
        assert!(!slash_cherche("/", false, false, false, false, false, true)); // dans un champ, elle s'écrit
        assert!(!slash_cherche("/", true, false, false, false, false, false));
        assert!(!slash_cherche("/", false, true, false, false, false, false));
        assert!(!slash_cherche("/", true, true, false, false, false, false)); // AltGr
        assert!(!slash_cherche("/", false, false, true, false, false, false));
        assert!(!slash_cherche("/", false, false, false, true, false, false)); // tenue
        assert!(!slash_cherche("/", false, false, false, false, true, false)); // en composition
        assert!(!slash_cherche(
            "!", false, false, false, false, false, false
        ));
        assert!(!slash_cherche(
            ":", false, false, false, false, false, false
        ));
    }

    #[test]
    fn echap_ne_ferme_que_le_volet_ouvert() {
        assert!(echap_ferme("Escape", false, true, false));
        assert!(!echap_ferme("Escape", false, false, false));
        assert!(!echap_ferme("Escape", true, true, false));
        assert!(!echap_ferme("Esc", false, true, false));
        assert!(!echap_ferme("Escape", false, true, true)); // dans la recherche, Échap vide le champ
        assert!(est_la_recherche(Some(ID_DE_LA_RECHERCHE)));
        assert!(!est_la_recherche(Some("note")));
        assert!(!est_la_recherche(Some("")));
        assert!(!est_la_recherche(None));
    }

    #[test]
    fn un_champ_de_date_vide_ne_borne_pas() {
        assert_eq!(jour_du_champ(""), None);
        assert_eq!(jour_du_champ("2026-10"), None);
        assert_eq!(
            jour_du_champ("2026-10-25"),
            Some(Jour {
                annee: 2026,
                mois: 10,
                jour: 25
            })
        );
    }

    #[test]
    fn les_bornes_de_la_requete_se_tirent_de_ses_jours() {
        let paris = |t: i64| {
            if (1_774_746_000..1_792_890_000).contains(&t) {
                120
            } else {
                60
            }
        };
        let jour = jour_du_champ("2026-10-25");
        let requete = Requete {
            du: jour.clone(),
            au: jour,
            ..Requete::default()
        };
        assert_eq!(
            bornes_de_la_requete(&requete, paris),
            Bornes {
                des: Some(1_792_879_200),
                avant: Some(1_792_969_200)
            }
        );
        assert_eq!(
            bornes_de_la_requete(&Requete::default(), paris),
            Bornes::default()
        );
    }

    #[test]
    fn la_requete_de_l_ecran_suit_ses_reglages_et_s_y_repose() {
        let owner = Owner::new();
        owner.set();
        let etat = EtatDeLaRequete::nouveau();
        assert_eq!(etat.requete(), Requete::default());
        let requete = Requete {
            texte: "eglise".into(),
            du: jour_du_champ("2026-10-25"),
            etiquette: Some("pont".into()),
            plus_de_secondes: Some(60),
            etat: Some(Etat::NonReliee),
            provenance: Some(ProvenanceDeLIdee::Importee),
            contenu: Some(Contenu::Prise),
            ..Requete::default()
        };
        etat.poser(&requete);
        assert_eq!(etat.requete(), requete);
        etat.poser(&Requete {
            plus_de_secondes: Some(45),
            ..Requete::default()
        });
        assert_eq!(etat.requete().plus_de_secondes, None);
    }
}
