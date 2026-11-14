//! Les règles d'une idée et de ses captations : les empreintes que nomment les clés
//! `<empreinte>/<fait>` des conteneurs des captations (§ 31 · 6.1).

use std::collections::BTreeSet;

use noyau::donnees::idee::empreintes_nommees;

#[test]
fn les_empreintes_nommees_se_coupent_au_premier_trait() {
    let cles = [
        "e1/idee",
        "e1/format",
        "e2/idee",
        "sans-trait",
        "e3/nom/a/b.wav",
    ];

    assert_eq!(
        empreintes_nommees(cles),
        BTreeSet::from(["e1".to_string(), "e2".to_string(), "e3".to_string()])
    );
}
