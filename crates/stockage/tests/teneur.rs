//! La teneur d'un document (§ 31 · 2.6), tirée de l'arbre que présente Automerge : la valeur
//! d'une clé en conflit est celle qu'il présente, `Map` et `Table` sont une seule sorte, et un
//! objet `Text` ne se confond pas avec une chaîne.

use automerge::transaction::Transactable;
use automerge::{Automerge, ObjType, ROOT, ReadDoc, Value};
use stockage::teneur;

#[test]
fn la_teneur_prend_la_valeur_que_presente_automerge() {
    let mut base = Automerge::new();
    let mut a = base.fork();
    let mut b = base.fork();
    let mut ta = a.transaction();
    ta.put(ROOT, "k", "a").unwrap();
    ta.commit();
    let mut tb = b.transaction();
    tb.put(ROOT, "k", "b").unwrap();
    tb.commit();
    a.merge(&mut b).unwrap();
    base.merge(&mut a).unwrap();

    let presentee = match base.get(ROOT, "k").unwrap() {
        Some((Value::Scalar(s), _)) => s.as_str().unwrap().to_string(),
        autre => panic!("la valeur de k : {autre:?}"),
    };
    assert!(presentee == "a" || presentee == "b");
    assert_eq!(base.get_all(ROOT, "k").unwrap().len(), 2, "un conflit");
    assert_eq!(teneur(&base).unwrap(), format!(r#"{{"k":s"{presentee}"}}"#));
}

#[test]
fn map_et_table_sont_une_seule_sorte() {
    let teneur_de = |sorte: ObjType| {
        let mut doc = Automerge::new();
        let mut tx = doc.transaction();
        let objet = tx.put_object(ROOT, "m", sorte).unwrap();
        tx.put(&objet, "x", 1_i64).unwrap();
        tx.commit();
        teneur(&doc).unwrap()
    };
    assert_eq!(teneur_de(ObjType::Map), r#"{"m":{"x":i1}}"#);
    assert_eq!(teneur_de(ObjType::Map), teneur_de(ObjType::Table));
}

#[test]
fn un_objet_text_n_est_pas_une_valeur_str() {
    let mut doc = Automerge::new();
    let mut tx = doc.transaction();
    let texte = tx.put_object(ROOT, "v", ObjType::Text).unwrap();
    tx.splice_text(&texte, 0, 0, "abc").unwrap();
    tx.commit();
    let mut autre = Automerge::new();
    let mut tx = autre.transaction();
    tx.put(ROOT, "v", "abc").unwrap();
    tx.commit();
    assert_eq!(teneur(&doc).unwrap(), r#"{"v":t"abc"}"#);
    assert_eq!(teneur(&autre).unwrap(), r#"{"v":s"abc"}"#);
}
