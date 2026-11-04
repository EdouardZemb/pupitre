//! La compaction (§ 31 · 2.2) : quand fondre les fichiers d'ajout dans le fichier complet, et
//! lesquels retirer ensuite. L'adaptateur `Documents` lit et écrit les fichiers ; ces règles ne
//! voient que ce qu'il en a lu.

/// Au-delà de ce nombre de fichiers d'ajout, une écriture compacte le document.
pub const AJOUTS_AU_PLUS: usize = 100;

/// Point 1 : à l'ouverture, s'il y a un fichier d'ajout, ou le voisin entier du fichier complet.
pub fn a_l_ouverture(ajouts: usize, voisin_du_complet: bool) -> bool {
    ajouts > 0 || voisin_du_complet
}

/// Point 1 : après une écriture, dès que les ajouts pèsent ensemble plus que le fichier
/// complet, ou passent AJOUTS_AU_PLUS.
pub fn apres_une_ecriture(octets_du_complet: u64, octets_des_ajouts: u64, ajouts: usize) -> bool {
    octets_des_ajouts > octets_du_complet || ajouts > AJOUTS_AU_PLUS
}

/// Ce que l'adaptateur a lu d'un fichier d'ajout, ou du voisin du fichier complet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lu {
    /// Le nom du fichier, tel que l'adaptateur le retirera.
    pub nom: String,
    /// Son contenu donne l'empreinte que porte son nom.
    pub conforme: bool,
    /// Chacune de ses modifications est dans le fichier complet relu.
    pub absorbe: bool,
}

/// Points 3 et 4 : si le fichier complet relu mène au même point que le document en mémoire,
/// les seuls fichiers conformes à leur empreinte et absorbés, dans l'ordre reçu ; sinon aucun.
/// Un fichier non conforme n'est jamais retiré : un fichier altéré se lit sans erreur jusqu'à
/// son premier tronçon altéré, et seule l'empreinte de son nom le dit.
pub fn a_retirer(meme_point: bool, lus: &[Lu]) -> Vec<String> {
    if !meme_point {
        return Vec::new();
    }
    lus.iter()
        .filter(|lu| lu.conforme && lu.absorbe)
        .map(|lu| lu.nom.clone())
        .collect()
}
