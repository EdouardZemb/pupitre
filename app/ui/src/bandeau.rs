//! Le bandeau de l'établi (I29 ; question 31) : l'état du fonds, le refus d'un geste et les
//! avertissements, dits en tête de l'établi, chacun par un mot en tête, jamais par la seule
//! couleur ; et la page d'ouverture, l'établi ou le seul bandeau (question 4).

use commandes::{EtatDuFonds, Refus};
use leptos::prelude::*;

/// D'où vient un message du bandeau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    /// L'état du fonds : sa lecture seule, une version de schéma plus récente comprise.
    Etat,
    /// Le refus d'un geste, ou d'une lecture.
    Refus,
    /// Un avertissement : le geste est fait, ou le fonds ouvert, mais Sacha doit en savoir quelque
    /// chose.
    Avertissement,
}

impl Source {
    /// Le mot en tête du message, qui dit sa source : la couleur ne la dit jamais seule.
    pub fn mot(self) -> &'static str {
        match self {
            Source::Etat => "Lecture seule :",
            Source::Refus => "Refusé :",
            Source::Avertissement => "Avertissement :",
        }
    }

    /// Le rôle du message : un refus est une alerte (`alert`) ; un état ou un avertissement, un
    /// statut (`status`).
    pub fn role(self) -> &'static str {
        match self {
            Source::Refus => "alert",
            Source::Etat | Source::Avertissement => "status",
        }
    }
}

/// Un message du bandeau : sa source, son texte, sans le mot de sa source, et s'il se ferme.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MessageDuBandeau {
    pub source: Source,
    pub texte: String,
    pub fermable: bool,
}

/// Le bandeau (I29 ; question 31) : un état (lecture seule, version de schéma plus récente), qui ne
/// se ferme pas ; un refus, et des avertissements, qui restent jusqu'au geste réussi suivant ou
/// jusqu'à « Fermer ».
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Bandeau {
    /// Pourquoi le fonds n'écrit pas ; `None` s'il écrit.
    etat: Option<String>,
    /// Le refus du dernier geste refusé, ou d'une lecture.
    refus: Option<String>,
    /// Les avertissements, dans l'ordre de leur arrivée.
    avertissements: Vec<String>,
}

impl Bandeau {
    /// L'état du fonds, relu à l'ouverture et après chaque commande : sa lecture seule devient
    /// l'état ; à l'ouverture, ses avertissements s'ajoutent. Après une commande, ils ne s'ajoutent
    /// pas : l'issue du geste porte déjà le sien (`apres_un_geste`).
    pub fn suivre(&mut self, etat: &EtatDuFonds, a_l_ouverture: bool) {
        self.etat = etat
            .lecture_seule
            .as_ref()
            .map(|refus| refus.message.clone());
        if a_l_ouverture {
            self.avertissements.extend_from_slice(&etat.avertissements);
        }
    }

    /// L'issue d'un geste : réussi, le refus et les avertissements d'avant s'effacent, puis le
    /// sien s'ajoute ; refusé, son message devient le refus. Le refus d'une lecture y passe aussi,
    /// jamais une lecture réussie.
    pub fn apres_un_geste(&mut self, issue: &Result<Option<String>, Refus>) {
        match issue {
            Ok(avertissement) => {
                self.refus = None;
                self.avertissements.clear();
                if let Some(avertissement) = avertissement {
                    self.avertissements.push(avertissement.clone());
                }
            }
            Err(refus) => self.refus = Some(refus.message.clone()),
        }
    }

    /// Ferme le refus ou les avertissements ; l'état ne se ferme pas.
    pub fn fermer(&mut self, source: Source) {
        match source {
            Source::Etat => {}
            Source::Refus => self.refus = None,
            Source::Avertissement => self.avertissements.clear(),
        }
    }

    /// Les messages, un par source, l'état d'abord, puis le refus, puis les avertissements, qui
    /// font un seul message, se ferment ensemble et se suivent, séparés d'une espace.
    pub fn messages(&self) -> Vec<MessageDuBandeau> {
        let avertissements =
            (!self.avertissements.is_empty()).then(|| self.avertissements.join(" "));
        [
            (Source::Etat, self.etat.clone(), false),
            (Source::Refus, self.refus.clone(), true),
            (Source::Avertissement, avertissements, true),
        ]
        .into_iter()
        .filter_map(|(source, texte, fermable)| {
            texte.map(|texte| MessageDuBandeau {
                source,
                texte,
                fermable,
            })
        })
        .collect()
    }
}

/// Ce que l'établi ouvre : tout, ou, dans l'application, devant un refus de l'état du fonds (une
/// racine tenue, une racine que le disque refuse), la page du seul bandeau (question 4). Hors de
/// l'application (l'interface servie seule, dans un navigateur), tout s'ouvre, comme la table
/// d'INC-0, et la boîte dit son refus en bandeau.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Page {
    /// L'établi entier, son bandeau en tête.
    Etabli,
    /// Le seul bandeau, qui dit ce refus.
    BandeauSeul(String),
}

/// La page qu'ouvre l'état du fonds `etat`, lu à l'ouverture : son refus, dans l'application, la
/// page du seul bandeau ; tout autre cas, l'établi.
pub(crate) fn page_d_ouverture(
    etat: &Result<EtatDuFonds, Refus>,
    dans_l_application: bool,
) -> Page {
    match etat {
        Err(refus) if dans_l_application => Page::BandeauSeul(refus.message.clone()),
        _ => Page::Etabli,
    }
}

/// Le texte d'un message : le mot de sa source, en gras, puis ce qu'il dit, sous le rôle de sa
/// source.
fn texte_du_message(source: Source, texte: String) -> impl IntoView {
    let role = source.role();
    let mot = source.mot();
    view! {
        <p role=role>
            <strong>{mot}</strong>
            " "
            {texte}
        </p>
    }
}

/// Le bandeau, en tête de l'établi : chaque message en texte, et un bouton « Fermer » s'il se
/// ferme. Sans message, il ne prend aucune place.
#[component]
pub fn VueDuBandeau(bandeau: RwSignal<Bandeau>) -> impl IntoView {
    view! {
        <div class="bandeau">
            {move || {
                bandeau
                    .get()
                    .messages()
                    .into_iter()
                    .map(|message| {
                        let source = message.source;
                        let fermer = message
                            .fermable
                            .then(|| {
                                view! {
                                    <button
                                        type="button"
                                        on:click=move |_| bandeau.update(|b| b.fermer(source))
                                    >
                                        "Fermer"
                                    </button>
                                }
                            });
                        view! {
                            <div class="message">
                                {texte_du_message(source, message.texte)}
                                {fermer}
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

/// La page du seul bandeau (question 4) : le refus de l'état du fonds, qui ne se ferme pas ;
/// l'établi ne s'ouvre pas, et aucune commande ne passe.
#[component]
pub fn PageDuBandeau(texte: String) -> impl IntoView {
    view! {
        <main class="bandeau-seul">
            <div class="bandeau">
                <div class="message">{texte_du_message(Source::Refus, texte)}</div>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commandes::Raison;

    fn refus(raison: Raison, message: &str) -> Refus {
        Refus {
            raison,
            message: message.to_string(),
        }
    }

    /// Un état du fonds né, sans geste à annuler.
    fn etat(lecture_seule: Option<Refus>, avertissements: &[&str]) -> EtatDuFonds {
        EtatDuFonds {
            ne: true,
            lecture_seule,
            avertissements: avertissements.iter().map(|a| a.to_string()).collect(),
            a_annuler: None,
        }
    }

    fn message(source: Source, texte: &str, fermable: bool) -> MessageDuBandeau {
        MessageDuBandeau {
            source,
            texte: texte.to_string(),
            fermable,
        }
    }

    #[test]
    fn i29_une_ecriture_refusee_s_affiche_en_bandeau() {
        let texte = "Le disque a refusé l'écriture du fonds : rien n'a changé.";
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Err(refus(Raison::Disque, texte)));
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Refus, texte, true)]
        );
    }

    #[test]
    fn un_refus_reste_jusqu_au_geste_reussi_suivant() {
        let texte = "Le disque a refusé l'écriture du fonds : rien n'a changé.";
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Err(refus(Raison::Disque, texte)));
        // Une lecture réussie de l'état du fonds ne l'efface pas.
        bandeau.suivre(&etat(None, &[]), false);
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Refus, texte, true)]
        );
        bandeau.apres_un_geste(&Ok(None));
        assert_eq!(bandeau.messages(), Vec::new());
        // « Fermer » l'efface aussi, comme les avertissements.
        bandeau.apres_un_geste(&Ok(Some("Un avertissement inventé.".to_string())));
        bandeau.apres_un_geste(&Err(refus(Raison::Disque, texte)));
        bandeau.fermer(Source::Refus);
        assert_eq!(
            bandeau.messages(),
            vec![message(
                Source::Avertissement,
                "Un avertissement inventé.",
                true
            )]
        );
        bandeau.fermer(Source::Avertissement);
        assert_eq!(bandeau.messages(), Vec::new());
    }

    #[test]
    fn un_etat_ne_se_ferme_pas() {
        let illisible = "Un fichier du fonds est altéré : le fonds se lit sans écrire.";
        let mut bandeau = Bandeau::default();
        bandeau.suivre(&etat(Some(refus(Raison::Illisible, illisible)), &[]), true);
        bandeau.fermer(Source::Etat);
        // Ni « Fermer » ni un geste réussi ne l'effacent.
        bandeau.apres_un_geste(&Ok(None));
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Etat, illisible, false)]
        );

        let plus_recente =
            "Le fonds porte une version de schéma plus récente : il se lit sans écrire.";
        let mut bandeau = Bandeau::default();
        bandeau.suivre(
            &etat(
                Some(refus(
                    Raison::VersionPlusRecente { version: 4 },
                    plus_recente,
                )),
                &[],
            ),
            false,
        );
        bandeau.fermer(Source::Etat);
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Etat, plus_recente, false)]
        );
        // Le fonds qui écrit de nouveau : l'état s'efface.
        bandeau.suivre(&etat(None, &[]), false);
        assert_eq!(bandeau.messages(), Vec::new());
    }

    #[test]
    fn les_etats_passent_avant_les_refus() {
        let mut bandeau = Bandeau::default();
        // À l'ouverture, l'état et deux avertissements ; puis un refus. Les avertissements font
        // un seul message, un par source, et viennent après le refus.
        bandeau.suivre(
            &etat(
                Some(refus(Raison::Illisible, "Un état inventé.")),
                &[
                    "Un avertissement inventé.",
                    "Un second avertissement inventé.",
                ],
            ),
            true,
        );
        bandeau.apres_un_geste(&Err(refus(Raison::Impossible, "Un refus inventé.")));
        assert_eq!(
            bandeau.messages(),
            vec![
                message(Source::Etat, "Un état inventé.", false),
                message(Source::Refus, "Un refus inventé.", true),
                message(
                    Source::Avertissement,
                    "Un avertissement inventé. Un second avertissement inventé.",
                    true
                ),
            ]
        );
    }

    #[test]
    fn l_avertissement_de_la_naissance_ne_parait_qu_une_fois() {
        // L'avertissement d'une naissance non durable revient dans l'issue du geste et dans
        // l'état du fonds relu après la commande : le bandeau ne le dit qu'une fois.
        let t = "Le fonds est né, mais le disque n'a pas confirmé son écriture.";
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Ok(Some(t.to_string())));
        bandeau.suivre(&etat(None, &[t]), false);
        assert_eq!(
            bandeau.messages(),
            vec![message(Source::Avertissement, t, true)]
        );
    }

    #[test]
    fn chaque_source_a_son_mot_et_son_role() {
        // La source se dit par un mot, jamais par la seule couleur ; un refus est une alerte.
        assert_eq!(Source::Etat.mot(), "Lecture seule :");
        assert_eq!(Source::Refus.mot(), "Refusé :");
        assert_eq!(Source::Avertissement.mot(), "Avertissement :");
        assert_eq!(Source::Etat.role(), "status");
        assert_eq!(Source::Refus.role(), "alert");
        assert_eq!(Source::Avertissement.role(), "status");
    }

    #[test]
    fn la_page_d_ouverture_d_une_racine_tenue_n_a_que_le_bandeau() {
        let tenue = "Un autre processus tient la racine des données.";
        assert_eq!(
            page_d_ouverture(&Err(refus(Raison::Impossible, tenue)), true),
            Page::BandeauSeul(tenue.to_string())
        );
        assert_eq!(page_d_ouverture(&Ok(etat(None, &[])), true), Page::Etabli);
        // Un fonds en lecture seule ouvre l'établi : le bandeau le dit.
        assert_eq!(
            page_d_ouverture(
                &Ok(etat(
                    Some(refus(Raison::Illisible, "Un état inventé.")),
                    &[]
                )),
                true
            ),
            Page::Etabli
        );
        let hors = "Hors de l'application, aucune commande ne passe.";
        assert_eq!(
            page_d_ouverture(&Err(refus(Raison::Impossible, hors)), false),
            Page::Etabli
        );
    }
}
