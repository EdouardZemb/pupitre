//! Le cœur de la couche de synchronisation de pupitre (§ 40 · 3) : comparer deux copies d'un
//! document, puis porter entre elles le protocole de synchronisation d'Automerge. Au lot 3, il
//! lit aussi les conflits et les relève, selon la règle du noyau (§ 70 · 1). Il n'a aucun
//! transport : le réseau se branche dessus, ou à côté.
