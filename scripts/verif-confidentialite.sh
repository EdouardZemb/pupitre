#!/usr/bin/env bash
# Verification de confidentialite — porte de CI de ce depot.
#
# Refuse si une donnee personnelle entre dans le depot. Ce depot est public :
# ni paroles, ni audio, ni identite, ni secret.
#
# Usage : bash scripts/verif-confidentialite.sh
# Sortie : 0 si rien trouve, 1 sinon. Chaque constat est imprime avec son fichier.

set -uo pipefail

# Fichiers exclus du balayage : ceux qui citent les motifs pour les interdire, et les
# chemins gérés par modele-projet (docs/modele/methode.md : exclus des vérifications du
# projet ; scripts/modele/confidentialite.py cite les motifs de secrets pour les chercher).
EXCLUS='^(scripts/verif-confidentialite\.sh|\.gitignore)$|^(\.githooks|scripts/modele|docs/modele|\.github/modele|\.claude/skills/consigner)/|^\.github/workflows/modele-[^/]*\.yml$'

fichiers=$(git ls-files | grep -Ev "$EXCLUS" || true)
echec=0

signale() {
  echo "REFUSE — $1"
  echo "$2"
  echec=1
}

# 1. Adresses e-mail. Une seule exception : noreply@github.com.
trouve=$(printf '%s\n' "$fichiers" | xargs -r grep -nIE \
  '[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}' 2>/dev/null \
  | grep -v 'noreply@github\.com' || true)
[ -n "$trouve" ] && signale "adresse e-mail" "$trouve"

# 2. Memoire d'assistant exportee.
trouve=$(printf '%s\n' "$fichiers" | xargs -r grep -nIE \
  '\.claude/memory|memory_read|memory_list|\[stated\]' 2>/dev/null || true)
[ -n "$trouve" ] && signale "export de memoire d'assistant" "$trouve"

# 3. Secrets.
trouve=$(printf '%s\n' "$fichiers" | xargs -r grep -nIE \
  'BEGIN [A-Z ]*PRIVATE KEY|gh[pousr]_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|sk-[A-Za-z0-9]{20,}' \
  2>/dev/null || true)
[ -n "$trouve" ] && signale "secret ou jeton" "$trouve"

# 4. Fichiers de matiere musicale versionnes par erreur.
trouve=$(git ls-files \
  | grep -iE '\.(mp3|m4a|wav|aif|aiff|flac|ogg|mid|midi|musicxml|mxl)$' || true)
[ -n "$trouve" ] && signale "fichier audio, MIDI ou partition versionne" "$trouve"

# 5. Chemins absolus d'une machine personnelle : ils deviennent faux des que la
#    machine change, et ils disent a qui elle appartient.
trouve=$(printf '%s\n' "$fichiers" | xargs -r grep -nIE \
  'C:\\+Users\\+[A-Za-z0-9]|/home/[a-z0-9]+/|/Users/[a-z0-9]+/' 2>/dev/null || true)
[ -n "$trouve" ] && signale "chemin absolu d'une machine personnelle" "$trouve"

# 6. Un document ou une archive d'un fonds versionnes : un fonds ne se versionne jamais
#    (§ 40 · 9). Aucun dossier n'est exempte : la PR qui suivrait un document de test
#    ajouterait l'exemption, et le dirait.
trouve=$(git ls-files | grep -E '\.automerge$|(^|/)pupitre-archive-[^/]*/' || true)
[ -n "$trouve" ] && signale "document ou archive d'un fonds versionne" "$trouve"

if [ "$echec" -eq 0 ]; then
  echo "OK — aucun motif interdit dans $(printf '%s\n' "$fichiers" | wc -l) fichiers suivis."
fi
exit "$echec"
