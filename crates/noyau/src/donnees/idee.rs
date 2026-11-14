//! Les règles d'une idée et de ses captations, sans entrée-sortie : les empreintes que nomment
//! les clés `<empreinte>/<fait>` des conteneurs des captations (§ 31 · 6.1).

use std::collections::BTreeSet;

/// Les empreintes que nomment des clés `<empreinte>/<fait>` : ce qui précède le premier `/` ; une
/// clé sans `/` ne nomme rien.
pub fn empreintes_nommees<'a>(cles: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
    cles.into_iter()
        .filter_map(|cle| cle.split_once('/'))
        .map(|(empreinte, _)| empreinte.to_string())
        .collect()
}
