"""Réglages du projet, posés au rendu du modèle."""

BRANCHE_PRINCIPALE = "main"
# « code » : la porte de confidentialité refuse tout constat ; « donnees » : elle avertit sur ce qui identifie et refuse les secrets (M-0006).
NATURE = "code"
# Clés que la mécanique du modèle exige dans chaque `.cles`, indexées par le chemin du `.cles`.
SECRETS_EXIGES: dict[str, list[str]] = {}
