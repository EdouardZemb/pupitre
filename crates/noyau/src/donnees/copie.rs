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

/// Ce que les lignées de deux copies disent de l'échange (§ 31 · 2.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rapport {
    /// Même lignée : les copies s'échangent.
    Meme,
    /// La distante nomme la locale parmi celles qu'elle remplace : la locale se met à part.
    LocaleRemplacee,
    /// La locale nomme la distante : la distante ne fusionne pas.
    DistanteRemplacee,
    /// Aucune ne remplace l'autre, ou chacune nomme l'autre : rien, et Pupitre le dit.
    Etrangeres,
}

/// La liste des lignées remplacées se veut complète, celles que remplaçait une lignée remplacée
/// comprises, que le lot 4 écrira : seule l'appartenance compte (§ 31 · 2.5).
pub fn rapport_des_lignees(
    locale: &str,
    remplacees_par_la_locale: &[&str],
    distante: &str,
    remplacees_par_la_distante: &[&str],
) -> Rapport {
    if locale == distante {
        return Rapport::Meme;
    }
    let locale_remplacee = remplacees_par_la_distante.contains(&locale);
    let distante_remplacee = remplacees_par_la_locale.contains(&distante);
    match (locale_remplacee, distante_remplacee) {
        (true, false) => Rapport::LocaleRemplacee,
        (false, true) => Rapport::DistanteRemplacee,
        _ => Rapport::Etrangeres,
    }
}
