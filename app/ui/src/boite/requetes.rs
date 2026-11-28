//! La requête en cours et les requêtes enregistrées : leurs critères, leur réponse, et la
//! commande qui lit la boîte d'idées.

use commandes::{Assiette, Boite, Fenetre, Ordre, Requete, RequeteEnregistree, VueDeLaBoite};
use leptos::prelude::*;
use noyau::donnees::idee;
use noyau::donnees::repli::mots;
use noyau::donnees::requete::{self, texte_de_la_reponse};

use crate::app::Etat;
use crate::recherche::{EtatDeLaRequete, bornes_de_la_requete};

/// `n` en `usize`, `usize::MAX` s'il le dépasse.
fn en_usize(n: u64) -> usize {
    usize::try_from(n).unwrap_or(usize::MAX)
}

/// Un compte de la boîte et son assiette, tels que le noyau les reçoit (`texte_du_compte`,
/// `texte_de_la_reponse`) : chaque compte en `usize`, et les illisibles de l'assiette à part.
pub(super) fn au_noyau(compte: u64, assiette: &Assiette) -> (usize, idee::Assiette, usize) {
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
pub(super) const PRECISION_D_UNE_REQUETE_PLUS_RECENTE: &str =
    "Elle ne se lance pas : choisie, elle se renomme ou se supprime.";

/// Le constat d'une liste de requêtes enregistrées lue et vide, de la forme d'« Aucune idée
/// retirée. ». Rien avant la première lecture, ou quand la lecture est refusée, puisque rien n'en
/// est su ; rien non plus quand la liste porte une requête.
pub(super) fn constat_des_requetes(lues: Option<&[RequeteEnregistree]>) -> Option<&'static str> {
    lues.is_some_and(<[RequeteEnregistree]>::is_empty)
        .then_some("Aucune requête enregistrée.")
}

/// Ce que montre une requête enregistrée : son nom et, si elle vient d'une version plus récente
/// (`requete: None`), la note qui dit qu'elle ne se lance pas.
pub(super) fn ce_que_dit_la_requete(
    requete: &RequeteEnregistree,
) -> (String, Option<&'static str>) {
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
pub(super) fn lancer(etat: EtatDeLaRequete, requete: &RequeteEnregistree) -> bool {
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
pub(super) fn enregistrer_possible(requete: &Requete, nom: &str) -> bool {
    pose_un_critere(requete) && !nom.trim().is_empty()
}

/// La réponse de la requête lue `requete`, que dit sa vue `vue` (`texte_de_la_reponse`) : le
/// nombre des idées trouvées, ses parts « sans … », ses illisibles et ses interrogées. Une requête
/// qui ne pose aucun critère rend toute la boîte d'idées, qui est sa réponse : aucun texte de
/// réponse ne se dit.
pub(super) fn texte_de_la_requete(vue: &VueDeLaBoite, requete: &Requete) -> Option<String> {
    pose_un_critere(requete).then(|| {
        let (retenues, assiette, illisibles) = au_noyau(vue.retenues, &vue.assiette_de_la_requete);
        texte_de_la_reponse(retenues, assiette, illisibles, sans_au_noyau(&vue.sans))
    })
}

/// La commande qui lit la fenêtre `fenetre` de la boîte, rangée par `ordre`, réponse de la requête
/// `requete`, telle quelle, et les bornes de sa date, tirées de ses jours au décalage que
/// `decalage` dit de chaque instant (`bornes_de_la_requete`, `natif::decalage_a`).
pub(super) fn commande_de_la_boite(
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
pub(super) fn requete_en_cours(etat: Etat) -> Requete {
    untrack(|| etat.requete.requete())
}

/// Un geste d'une requête enregistrée, avec ce qu'il faut pour conclure son issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum GesteDeRequete {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boite::tests::{decalage_de_paris, enregistree};
    use crate::recherche::jour_du_champ;
    use commandes::{Bornes, Contenu};

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
}
