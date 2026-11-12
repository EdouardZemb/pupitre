//! La compaction (§ 31 · 2.2) : quand fondre les fichiers d'ajout dans le fichier complet, et
//! lesquels retirer ensuite. L'adaptateur `Documents` lit et écrit les fichiers ; ces règles ne
//! voient que ce qu'il en a lu, et l'accès du document.

use crate::donnees::acces::Acces;

/// Au-delà de ce nombre de fichiers d'ajout, une écriture compacte un document qui écrit.
pub const AJOUTS_AU_PLUS: usize = 100;

/// Point 1 : à l'ouverture d'un document qui écrit, s'il y a un fichier d'ajout, ou le voisin
/// entier du fichier complet. Un document en lecture seule ne se compacte pas.
pub fn a_l_ouverture(acces: Acces, ajouts: usize, voisin_du_complet: bool) -> bool {
    acces.ecrit() && (ajouts > 0 || voisin_du_complet)
}

/// Point 1 : après une écriture, dans un document qui écrit, quand une compaction est due, ou
/// dès que les ajouts pèsent ensemble plus que le fichier complet, ou passent AJOUTS_AU_PLUS.
/// Un document en lecture seule ne se compacte pas.
pub fn apres_une_ecriture(
    acces: Acces,
    due: bool,
    octets_du_complet: u64,
    octets_des_ajouts: u64,
    ajouts: usize,
) -> bool {
    acces.ecrit() && (due || octets_des_ajouts > octets_du_complet || ajouts > AJOUTS_AU_PLUS)
}

/// Ce que l'adaptateur a lu d'un fichier d'ajout, ou du voisin du fichier complet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lu {
    /// Son chemin relatif au dossier du document, `/` pour séparateur, tel que l'adaptateur le
    /// retirera : `ajouts/<empreinte>.automerge`, ou `complet.automerge.<processus>.partiel`.
    pub nom: String,
    /// Lisible : un fichier d'ajout dont le contenu donne l'empreinte de son nom et que
    /// `load_incremental` charge, ou un voisin du fichier complet non vide que `load` relit sans
    /// erreur.
    pub lisible: bool,
    /// Chacune de ses modifications est dans le fichier complet relu.
    pub absorbe: bool,
}

/// Points 3 et 4 : les seuls fichiers lisibles et absorbés, dans l'ordre reçu. Un fichier
/// illisible n'est jamais retiré : un fichier altéré se lit sans erreur jusqu'à son premier
/// tronçon altéré, et seule l'empreinte de son nom le dit.
pub fn a_retirer(lus: &[Lu]) -> Vec<String> {
    lus.iter()
        .filter(|lu| lu.lisible && lu.absorbe)
        .map(|lu| lu.nom.clone())
        .collect()
}
