//! Le bandeau de l'établi (I29 ; question 31) : l'état du fonds, le refus d'un geste et les
//! avertissements, dits en tête de l'établi, chacun par un mot en tête, jamais par la seule
//! couleur ; et la page d'ouverture, l'établi ou le seul bandeau (question 4).

use commandes::{EtatDuFonds, Refus};
use leptos::html;
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
    /// Les sources, dans l'ordre de leurs emplacements : l'état, le refus, les avertissements.
    pub const TOUTES: [Source; 3] = [Source::Etat, Source::Refus, Source::Avertissement];

    /// Le rang de la source dans `TOUTES`.
    pub fn rang(self) -> usize {
        match self {
            Source::Etat => 0,
            Source::Refus => 1,
            Source::Avertissement => 2,
        }
    }

    /// L'identifiant fixe de la région de son emplacement, que vise son « Fermer »
    /// (`aria-describedby`).
    pub fn id(self) -> &'static str {
        match self {
            Source::Etat => "bandeau-etat",
            Source::Refus => "bandeau-refus",
            Source::Avertissement => "bandeau-avertissements",
        }
    }

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

/// Un emplacement du bandeau, toujours monté : sa source, et ses lignes, chacune un message de
/// cette source, sans son mot ; aucune quand rien ne le remplit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Emplacement {
    pub source: Source,
    pub lignes: Vec<String>,
}

impl Emplacement {
    /// Il porte un message qui se ferme : un refus ou des avertissements, jamais l'état.
    pub fn fermable(&self) -> bool {
        self.source != Source::Etat && !self.lignes.is_empty()
    }
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

    /// Les messages rangés par emplacement, un par source, dans l'ordre de `Source::TOUTES` :
    /// l'état, le refus, puis les avertissements, une ligne chacun, dans l'ordre de leur arrivée.
    /// Un emplacement que rien ne remplit reste, vide.
    pub fn emplacements(&self) -> [Emplacement; 3] {
        Source::TOUTES.map(|source| Emplacement {
            source,
            lignes: match source {
                Source::Etat => self.etat.iter().cloned().collect(),
                Source::Refus => self.refus.iter().cloned().collect(),
                Source::Avertissement => self.avertissements.clone(),
            },
        })
    }

    /// La source du premier message qui se ferme encore (`messages`), dont le « Fermer » prend le
    /// focus après un « Fermer » ; aucune, le bandeau le prend.
    pub fn fermer_restant(&self) -> Option<Source> {
        self.messages()
            .into_iter()
            .find(|message| message.fermable)
            .map(|message| message.source)
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

/// Une ligne d'un message : le mot de sa source, en gras, puis ce qu'il dit.
fn ligne_du_message(source: Source, texte: String) -> impl IntoView {
    let mot = source.mot();
    view! {
        <p>
            <strong>{mot}</strong>
            " "
            {texte}
        </p>
    }
}

/// L'emplacement de `source`, toujours monté : sa région, au rôle de sa source et à son
/// identifiant fixe, vide sans message, ses lignes suivant le bandeau ; puis, hors de la région,
/// son « Fermer », que la région décrit (`aria-describedby`), caché tant que l'emplacement ne
/// porte rien qui se ferme. Le cadre et la marge (`message`) ne se posent que sur un emplacement
/// qui porte un message : une région ne se cache jamais.
fn vue_de_l_emplacement(
    bandeau: RwSignal<Bandeau>,
    source: Source,
    bouton: NodeRef<html::Button>,
    fermer: impl Fn(Source) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let emplacement = Memo::new(move |_| bandeau.with(|b| b.emplacements()[source.rang()].clone()));
    let id = source.id();
    let role = source.role();
    let lignes = move || {
        emplacement
            .get()
            .lignes
            .into_iter()
            .map(|texte| ligne_du_message(source, texte))
            .collect_view()
    };
    view! {
        <div class="emplacement" class:message=move || emplacement.with(|e| !e.lignes.is_empty())>
            <div id=id class="region" role=role>
                {lignes}
            </div>
            <button
                type="button"
                node_ref=bouton
                hidden=move || !emplacement.with(Emplacement::fermable)
                aria-describedby=id
                on:click=move |_| fermer(source)
            >
                "Fermer"
            </button>
        </div>
    }
}

/// Les repères du bandeau monté : son conteneur, et le « Fermer » de chaque source, dans l'ordre de
/// `Source::TOUTES`. L'état de l'établi les porte, pour rendre le focus après l'issue d'un geste
/// comme après « Fermer » (`rendre_le_focus`).
#[derive(Clone, Copy)]
pub(crate) struct ReperesDuBandeau {
    pub conteneur: NodeRef<html::Div>,
    pub boutons: [NodeRef<html::Button>; 3],
}

impl ReperesDuBandeau {
    pub fn nouveaux() -> Self {
        ReperesDuBandeau {
            conteneur: NodeRef::new(),
            boutons: Source::TOUTES.map(|_| NodeRef::new()),
        }
    }
}

/// Rend le focus, après « Fermer » ou après l'issue d'un geste, s'il est sur le « Fermer » d'un
/// emplacement qui ne porte plus rien qui se ferme : ce « Fermer » vient de se cacher (un `Ctrl+Z`
/// hors d'un champ peut effacer un refus dont le « Fermer » a le focus). Le focus va alors au
/// « Fermer » qui reste (`Bandeau::fermer_restant`), s'il se montre déjà, sinon au bandeau lui-même,
/// qui le prend (`tabindex="-1"`). Le focus ailleurs, rien ne bouge.
pub(crate) fn rendre_le_focus(bandeau: RwSignal<Bandeau>, reperes: ReperesDuBandeau) {
    let Some(actif) = document().active_element().map(leptos::web_sys::Node::from) else {
        return;
    };
    let Some(source) = Source::TOUTES.into_iter().find(|s| {
        reperes.boutons[s.rang()]
            .get_untracked()
            .is_some_and(|bouton| bouton.is_same_node(Some(&actif)))
    }) else {
        return;
    };
    if bandeau.with_untracked(|b| b.emplacements()[source.rang()].fermable()) {
        return;
    }
    let restant = bandeau
        .with_untracked(Bandeau::fermer_restant)
        .and_then(|s| reperes.boutons[s.rang()].get_untracked())
        .filter(|bouton| !bouton.hidden());
    match restant {
        Some(bouton) => {
            let _ = bouton.focus();
        }
        None => {
            if let Some(conteneur) = reperes.conteneur.get_untracked() {
                let _ = conteneur.focus();
            }
        }
    }
}

/// Le bandeau, en tête de l'établi : trois emplacements fixes, un par source, l'état d'abord
/// (`vue_de_l_emplacement`), montés sur ses repères, `reperes`. Sans message, il ne prend aucune
/// place. Après « Fermer », le focus va au « Fermer » qui reste, sinon au bandeau lui-même
/// (`rendre_le_focus`).
#[component]
pub fn VueDuBandeau(bandeau: RwSignal<Bandeau>, reperes: ReperesDuBandeau) -> impl IntoView {
    let fermer = move |source: Source| {
        bandeau.update(|b| b.fermer(source));
        rendre_le_focus(bandeau, reperes);
    };
    let emplacement = move |source: Source| {
        vue_de_l_emplacement(bandeau, source, reperes.boutons[source.rang()], fermer)
    };
    view! {
        <div class="bandeau" tabindex="-1" node_ref=reperes.conteneur>
            {emplacement(Source::Etat)}
            {emplacement(Source::Refus)}
            {emplacement(Source::Avertissement)}
        </div>
    }
}

/// La page du seul bandeau (question 4) : le refus de l'état du fonds, qui ne se ferme pas ;
/// l'établi ne s'ouvre pas, et aucune commande ne passe.
#[component]
pub fn PageDuBandeau(texte: String) -> impl IntoView {
    let role = Source::Refus.role();
    view! {
        <main class="bandeau-seul">
            <div class="bandeau">
                <div class="emplacement message">
                    <div class="region" role=role>
                        {ligne_du_message(Source::Refus, texte)}
                    </div>
                </div>
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
        // Chaque emplacement a son identifiant fixe, que vise le « Fermer » de sa source.
        assert_eq!(Source::Etat.id(), "bandeau-etat");
        assert_eq!(Source::Refus.id(), "bandeau-refus");
        assert_eq!(Source::Avertissement.id(), "bandeau-avertissements");
    }

    fn emplacement(source: Source, lignes: &[&str]) -> Emplacement {
        Emplacement {
            source,
            lignes: lignes.iter().map(|l| l.to_string()).collect(),
        }
    }

    #[test]
    fn chaque_source_a_son_emplacement_vide_sans_message() {
        let mut bandeau = Bandeau::default();
        // Sans message : les trois emplacements, dans l'ordre des sources, chacun vide.
        assert_eq!(
            bandeau.emplacements(),
            [
                emplacement(Source::Etat, &[]),
                emplacement(Source::Refus, &[]),
                emplacement(Source::Avertissement, &[]),
            ]
        );
        // L'état et deux avertissements : le refus reste à sa place, vide ; les avertissements
        // gardent leur ordre, une ligne chacun.
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
        assert_eq!(
            bandeau.emplacements(),
            [
                emplacement(Source::Etat, &["Un état inventé."]),
                emplacement(Source::Refus, &[]),
                emplacement(
                    Source::Avertissement,
                    &[
                        "Un avertissement inventé.",
                        "Un second avertissement inventé.",
                    ]
                ),
            ]
        );
        assert_eq!(
            bandeau.emplacements().map(|e| e.fermable()),
            [false, false, true]
        );
        // Un refus remplit son emplacement ; l'état ne se ferme jamais.
        bandeau.apres_un_geste(&Err(refus(Raison::Impossible, "Un refus inventé.")));
        assert_eq!(
            bandeau.emplacements()[1],
            emplacement(Source::Refus, &["Un refus inventé."])
        );
        assert_eq!(
            bandeau.emplacements().map(|e| e.fermable()),
            [false, true, true]
        );
    }

    #[test]
    fn le_focus_va_au_fermer_qui_reste() {
        let mut bandeau = Bandeau::default();
        bandeau.suivre(
            &etat(
                Some(refus(Raison::Illisible, "Un état inventé.")),
                &["Un avertissement inventé."],
            ),
            true,
        );
        bandeau.apres_un_geste(&Err(refus(Raison::Impossible, "Un refus inventé.")));
        assert_eq!(bandeau.fermer_restant(), Some(Source::Refus));
        bandeau.fermer(Source::Refus);
        assert_eq!(bandeau.fermer_restant(), Some(Source::Avertissement));
        bandeau.fermer(Source::Avertissement);
        // L'état reste, sans « Fermer » : le focus va au bandeau.
        assert_eq!(bandeau.fermer_restant(), None);
        // Le refus seul, fermé : plus rien à fermer.
        let mut bandeau = Bandeau::default();
        bandeau.apres_un_geste(&Ok(Some("Un avertissement inventé.".to_string())));
        bandeau.apres_un_geste(&Err(refus(Raison::Impossible, "Un refus inventé.")));
        bandeau.fermer(Source::Avertissement);
        assert_eq!(bandeau.fermer_restant(), Some(Source::Refus));
        bandeau.fermer(Source::Refus);
        assert_eq!(bandeau.fermer_restant(), None);
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
