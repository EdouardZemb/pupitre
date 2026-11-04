# Décisions d'architecture

Les décisions de pupitre ne sont pas ici. Elles vivent dans [`docs/adr/`](../adr/README.md) : un fichier par décision, numérotées `adr-NNN`, avec leur gabarit et leur série dans [`docs/adr/README.md`](../adr/README.md). Elles sont citées partout dans la conception, et la porte `documentation` suit ces liens.

Les décisions héritées du modèle restent dans [`docs/modele/decisions/`](../modele/decisions/M-0001-ecosysteme-des-applications-personnelles.md), avec le préfixe `M-`, pour que leurs numéros ne se heurtent pas à ceux de pupitre.

**Une ADR acceptée ne se réécrit pas ; aucune ne se renumérote ni ne se supprime.** Si la décision change, une nouvelle ADR la remplace, et l'ancienne garde son fichier avec le statut `remplacée`. Si elle ne change qu'en partie, une nouvelle ADR l'amende, et l'ancienne reste acceptée, son texte intact ([les statuts](../adr/README.md#statuts)). Une ADR proposée se corrige et se complète tant qu'elle est proposée, au plus tard dans la PR qui l'accepte.
