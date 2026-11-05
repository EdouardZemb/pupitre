//! Les copies d'un document qui s'échangent (§ 31 · 2.5, § 30 · 2) : la synchronisation les
//! compare avant tout échange, et deux copies étrangères ne s'échangent rien.

/// Ce qui situe une copie d'un document : l'identifiant de propriétaire, l'identifiant du
/// document et sa lignée, lus à la racine du document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copie {
    pub proprietaire: String,
    pub document: String,
    pub lignee: String,
}

/// La première différence entre deux copies, qui ferme l'échange.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difference {
    AutreProprietaire,
    AutreDocument,
    AutreLignee,
}

/// Deux copies s'échangent si elles ont le même propriétaire, le même document et la même
/// lignée ; sinon, la première différence, dans cet ordre.
pub fn comparer(locale: &Copie, distante: &Copie) -> Result<(), Difference> {
    if locale.proprietaire != distante.proprietaire {
        return Err(Difference::AutreProprietaire);
    }
    if locale.document != distante.document {
        return Err(Difference::AutreDocument);
    }
    if locale.lignee != distante.lignee {
        return Err(Difference::AutreLignee);
    }
    Ok(())
}
