//! Les hauteurs : des intervalles orthographiés (ADR-008).

use noyau::hauteur::{Intervalle, Lettre, NomDeNote};
use proptest::prelude::*;

fn n(lettre: Lettre, alteration: i8) -> NomDeNote {
    NomDeNote { lettre, alteration }
}

fn i(degres: i8, demi_tons: i8) -> Intervalle {
    Intervalle::new(degres, demi_tons)
}

#[test]
fn reduire() {
    assert_eq!(i(9, 16).reduit(), i(2, 4));
    assert_eq!(i(-5, -9).reduit(), i(2, 3));
}

#[test]
fn alterations() {
    assert_eq!(i(1, 1).alteration(), -1);
    assert_eq!(i(3, 6).alteration(), 1);
    assert_eq!(i(5, 8).alteration(), -1);
    assert_eq!(i(6, 11).alteration(), 0);
}

#[test]
fn additionner_et_soustraire_des_intervalles() {
    assert_eq!(i(2, 4).plus(i(2, 3)), i(4, 7));
    assert_eq!(i(4, 7).moins(i(2, 4)), i(2, 3));
}

#[test]
fn intervalles_hors_de_l_octave() {
    assert_eq!(i(2, 3).moins(i(4, 7)), i(-2, -4));
    assert_eq!(n(Lettre::C, 0).plus(i(9, 16)), n(Lettre::E, 0));
}

#[test]
fn triple_alteration() {
    use Lettre::*;
    assert_eq!(n(F, -1).plus(i(3, 4)), n(B, -3));
    assert_eq!(n(B, -3).intervalle_vers(n(F, -1)), i(4, 8));
    assert_eq!(n(B, -3).plus(i(4, 8)), n(F, -1));
}

#[test]
fn franchir_l_octave() {
    use Lettre::*;
    assert_eq!(n(B, -1).meme_son(1), n(C, -2));
    assert_eq!(n(C, -1).meme_son(-1), n(B, 0));
    assert_eq!(n(C, -2).enharmonique_simple(), n(B, -1));
    assert_eq!(n(B, 2).enharmonique_simple(), n(C, 1));
}

#[test]
fn nommer() {
    use Lettre::*;
    assert_eq!(n(C, 0).plus(i(2, 4)), n(E, 0));
    assert_eq!(n(D, 0).plus(i(4, 7)), n(A, 0));
    assert_eq!(n(F, 1).plus(i(2, 4)), n(A, 1));
    assert_eq!(n(D, -1).plus(i(5, 8)), n(B, -2));
    assert_eq!(n(G, 1).plus(i(6, 11)), n(F, 2));
}

#[test]
fn intervalle_vers() {
    use Lettre::*;
    assert_eq!(n(G, 1).intervalle_vers(n(G, 0)), i(0, -1));
    assert_eq!(n(D, -1).intervalle_vers(n(E, 0)), i(1, 3));
    assert_eq!(n(E, -1).intervalle_vers(n(B, 0)), i(4, 8));
    assert_eq!(n(A, 0).intervalle_vers(n(C, 0)), i(2, 3));
}

#[test]
fn classes() {
    use Lettre::*;
    assert_eq!(n(C, 0).classe(), 0);
    assert_eq!(n(B, 1).classe(), 0);
    assert_eq!(n(C, -1).classe(), 11);
    assert_eq!(n(F, 2).classe(), 7);
}

#[test]
fn enharmoniques() {
    use Lettre::*;
    assert_eq!(n(B, -2).enharmonique_simple(), n(A, 0));
    assert_eq!(n(F, 2).enharmonique_simple(), n(G, 0));
    assert_eq!(n(C, -2).enharmonique_simple(), n(B, -1));
    assert_eq!(n(E, 1).enharmonique_simple(), n(E, 1));
    assert_eq!(n(D, -1).meme_son(-1), n(C, 1));
    assert_eq!(n(F, -1).meme_son(-1), n(E, 0));
    assert_eq!(n(A, 1).meme_son(1), n(B, -1));
}

#[test]
fn noms() {
    use Lettre::*;
    assert_eq!(n(E, -1).anglais(), "E♭");
    assert_eq!(n(E, -1).solfege(), "mi♭");
    assert_eq!(n(F, 1).anglais(), "F♯");
    assert_eq!(n(F, 1).solfege(), "fa♯");
    assert_eq!(n(C, 0).anglais(), "C");
    assert_eq!(n(C, 0).solfege(), "do");
    assert_eq!(n(B, -2).anglais(), "B𝄫");
    assert_eq!(n(B, -2).solfege(), "si𝄫");
    assert_eq!(n(F, 2).anglais(), "F𝄪");
    assert_eq!(n(D, 0).solfege(), "ré");
    assert_eq!(n(B, -3).anglais(), "B𝄫♭");
    assert_eq!(n(B, -3).solfege(), "si𝄫♭");
    assert_eq!(n(F, 3).anglais(), "F𝄪♯");
    assert_eq!(n(F, 3).solfege(), "fa𝄪♯");
}

fn note_simple() -> impl Strategy<Value = NomDeNote> {
    (0usize..7, -1i8..=1).prop_map(|(l, alteration)| NomDeNote {
        lettre: [
            Lettre::C,
            Lettre::D,
            Lettre::E,
            Lettre::F,
            Lettre::G,
            Lettre::A,
            Lettre::B,
        ][l],
        alteration,
    })
}

fn intervalle_simple() -> impl Strategy<Value = Intervalle> {
    (0i8..7, -1i8..=1).prop_map(|(degres, alteration)| {
        let naturel = [0, 2, 4, 5, 7, 9, 11][degres as usize];
        Intervalle::new(degres, naturel + alteration)
    })
}

proptest! {
    #[test]
    fn transposer_puis_revenir(n in note_simple(), i in intervalle_simple()) {
        prop_assert_eq!(n.plus(i).plus(Intervalle::new(0, 0).moins(i)), n);
        prop_assert_eq!(n.intervalle_vers(n.plus(i)), i.reduit());
        let arrivee = n.plus(i);
        let simple = arrivee.enharmonique_simple();
        prop_assert_eq!(simple.classe(), arrivee.classe());
        prop_assert!(simple.alteration.abs() <= 1);
    }
}
