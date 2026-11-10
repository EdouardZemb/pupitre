//! Le point d'injection d'une panne, à chaque étape d'une écriture (§ 31 · 2.1).
//!
//! Chaque étape d'une écriture passe par `point` juste avant son geste sur le disque, sauf `Save`
//! et `Reception`, qui y passent juste après leur geste en mémoire, et chaque lecture de
//! `fichier::lire` rend ses octets par `relue`. Sans la fonction `pannes`, le point
//! laisse tout passer et la relecture rend les octets tels quels : la configuration livrée n'a
//! rien d'autre. Sous la fonction, que seuls les tests du stockage demandent, une panne s'arme
//! par `couper` ou `alterer_la_relecture`, dans le fil du test, et se désarme à sa chute.

#[cfg(feature = "pannes")]
use std::cell::{Cell, RefCell};
use std::io;
#[cfg(feature = "pannes")]
use std::marker::PhantomData;

/// Une étape d'une écriture, où une panne peut s'injecter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etape {
    /// Après le `save()` d'une compaction, avant de relire ses octets en mémoire
    /// (`Automerge::load`) : en échec, comme des octets qui ne se relisent pas, la compaction est
    /// infidèle, et rien ne s'écrit.
    Save,
    /// Après la création du voisin, avant d'y écrire ses octets (`write_all`).
    VoisinEcrit,
    /// Avant la synchronisation du voisin (`sync_all`).
    SyncAll,
    /// Avant la lecture d'un fichier (`fs::read`), dont la relecture du voisin.
    Relecture,
    /// Avant le renommage du voisin (`fs::rename`).
    Renommage,
    /// Avant de rendre durable le renommage.
    RendreDurable,
    /// Avant le retrait d'un fichier (`fs::remove_file`).
    Retrait,
    /// Après une réception qui a changé les heads du document, avant d'écrire ce qu'elle a
    /// apporté : en échec, la réception est refusée (`synchro::Erreur::Automerge`), ce qu'elle a
    /// apporté écrit. L'étape ne passe, et une panne `Reception` armée ne compte ce passage, que
    /// si les heads ont changé, y compris sur une réception que `synchro` a déjà refusée : une
    /// panne qui tombe sur celle-ci ne change pas son issue, qui reste le refus de `synchro`.
    Reception,
}

/// La panne armée : elle se désarme à sa chute.
#[cfg(feature = "pannes")]
#[must_use]
#[derive(Debug)]
pub struct Panne {
    numero: u64,
    /// Armée dans le fil d'un test, elle ne vaut que pour lui, et ne le quitte pas.
    _fil: PhantomData<*const ()>,
}

/// La `rang`-ième fois (dès 1) que `etape` survient dans ce fil depuis cet appel, elle échoue, une
/// fois.
#[cfg(feature = "pannes")]
pub fn couper(etape: Etape, rang: usize) -> Panne {
    armer(Sorte::Coupure(etape), rang)
}

/// La `rang`-ième lecture réussie de `fichier::lire` (dès 1) dans ce fil depuis cet appel réussit,
/// mais rend ses octets altérés : le premier inversé (`!`), ou un octet ajouté si rien n'est lu.
/// Une fois. Une lecture qui échoue, coupée par une panne `Relecture` ou refusée par le disque,
/// ne compte pas.
#[cfg(feature = "pannes")]
pub fn alterer_la_relecture(rang: usize) -> Panne {
    armer(Sorte::Alteration, rang)
}

/// Le passage par `etape` : `Ok(())` sans la fonction `pannes`.
#[cfg(not(feature = "pannes"))]
pub(crate) fn point(_etape: Etape) -> io::Result<()> {
    Ok(())
}

/// Le passage par `etape` : une erreur si une panne armée sur elle tombe à ce passage.
#[cfg(feature = "pannes")]
pub(crate) fn point(etape: Etape) -> io::Result<()> {
    if passer(Sorte::Coupure(etape)) {
        return Err(io::Error::other(format!("panne injectée : {etape:?}")));
    }
    Ok(())
}

/// Les octets lus par `fichier::lire`, tels quels sans la fonction `pannes`.
#[cfg(not(feature = "pannes"))]
pub(crate) fn relue(octets: Vec<u8>) -> Vec<u8> {
    octets
}

/// Les octets lus par `fichier::lire`, altérés si une panne de relecture tombe à cette lecture.
#[cfg(feature = "pannes")]
pub(crate) fn relue(mut octets: Vec<u8>) -> Vec<u8> {
    if passer(Sorte::Alteration) {
        match octets.first_mut() {
            Some(premier) => *premier = !*premier,
            None => octets.push(0),
        }
    }
    octets
}

/// Ce que fait tomber une panne armée.
#[cfg(feature = "pannes")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sorte {
    /// Le passage par une étape, qui échoue.
    Coupure(Etape),
    /// Une lecture de `fichier::lire`, qui réussit mais rend des octets altérés.
    Alteration,
}

/// Une panne armée dans ce fil, avec son compte.
#[cfg(feature = "pannes")]
struct Armee {
    numero: u64,
    sorte: Sorte,
    /// Les passages qui restent jusqu'à sa chute, celui de la chute compris : à zéro, elle est
    /// tombée et ne tombe plus.
    reste: usize,
}

#[cfg(feature = "pannes")]
thread_local! {
    /// Les pannes armées dans ce fil, et pas encore désarmées.
    static ARMEES: RefCell<Vec<Armee>> = const { RefCell::new(Vec::new()) };
    /// Le numéro de la prochaine panne armée dans ce fil.
    static PROCHAINE: Cell<u64> = const { Cell::new(0) };
}

/// Arme une panne de `sorte`, qui tombe au `rang`-ième passage (dès 1).
#[cfg(feature = "pannes")]
fn armer(sorte: Sorte, rang: usize) -> Panne {
    assert!(rang >= 1, "le rang d'une panne compte dès 1");
    let numero = PROCHAINE.get();
    PROCHAINE.set(numero + 1);
    ARMEES.with_borrow_mut(|armees| {
        armees.push(Armee {
            numero,
            sorte,
            reste: rang,
        })
    });
    Panne {
        numero,
        _fil: PhantomData,
    }
}

/// Compte un passage pour chaque panne de `sorte` armée dans ce fil, qui n'est pas encore
/// tombée : vrai si l'une tombe à ce passage.
#[cfg(feature = "pannes")]
fn passer(sorte: Sorte) -> bool {
    ARMEES.with_borrow_mut(|armees| {
        let mut tombee = false;
        for armee in armees
            .iter_mut()
            .filter(|armee| armee.sorte == sorte && armee.reste > 0)
        {
            armee.reste -= 1;
            tombee |= armee.reste == 0;
        }
        tombee
    })
}

#[cfg(feature = "pannes")]
impl Drop for Panne {
    fn drop(&mut self) {
        // Un fil qui s'achève a pu détruire ses pannes armées avant celle-ci : il n'y a alors plus
        // rien à désarmer.
        let _ = ARMEES.try_with(|armees| {
            armees
                .borrow_mut()
                .retain(|armee| armee.numero != self.numero)
        });
    }
}
