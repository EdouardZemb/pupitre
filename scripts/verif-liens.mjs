#!/usr/bin/env node
// Verification des liens internes de la documentation.
//
// Chaque lien relatif d'un fichier Markdown — syntaxe Markdown, ou attribut
// href / src d'une balise a ou img — doit mener a un fichier qui existe, et
// chaque ancre (#...) a un titre du fichier vise. Les ancres sont calculees
// comme GitHub les calcule : minuscules, ponctuation et symboles retires,
// espaces changees en tirets, suffixe -1, -2... pour les doublons.
//
// Usage : node scripts/verif-liens.mjs
// Sortie : 0 si tous les liens menent quelque part, 1 sinon.
// Aucune dependance : Node 22 ou plus suffit.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, join, normalize } from "node:path";

// Chemins gérés par modele-projet : leurs liens et leurs diagrammes sont l'affaire du modèle.
const GERES = /^(\.githooks|scripts\/modele|docs\/modele|\.github\/modele|\.claude\/skills\/consigner)\/|^\.github\/workflows\/modele-[^/]*\.yml$/;

const fichiers = execFileSync(
  "git", ["ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
  { encoding: "utf8" },
).split("\n").filter((f) => f && existsSync(f) && !GERES.test(f));

// Retire les blocs de code delimites ; garde les numeros de ligne.
function lignesHorsCode(texte) {
  let dedans = false;
  return texte.split("\n").map((l) => {
    if (/^\s*(```|~~~)/.test(l)) { dedans = !dedans; return ""; }
    return dedans ? "" : l;
  });
}

// Texte rendu d'un titre : sans emphase, sans code, liens reduits a leur texte.
function texteDeTitre(brut) {
  return brut
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/<[^>]+>/g, "")
    .replace(/`([^`]*)`/g, "$1")
    .replace(/(\*\*|__)(.+?)\1/g, "$2")
    .replace(/(^|[^\p{L}\p{N}])[*_](.+?)[*_](?=$|[^\p{L}\p{N}])/gu, "$1$2")
    .trim();
}

function slug(texte) {
  return texte.toLowerCase().replace(/[^\p{L}\p{M}\p{N}\p{Pc} -]/gu, "").replace(/ /g, "-");
}

const ancresParFichier = new Map();
function ancres(fichier) {
  if (ancresParFichier.has(fichier)) return ancresParFichier.get(fichier);
  const vues = new Map();
  const res = new Set();
  for (const l of lignesHorsCode(readFileSync(fichier, "utf8"))) {
    const t = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(l);
    if (t) {
      const base = slug(texteDeTitre(t[2]));
      const n = vues.get(base) ?? 0;
      vues.set(base, n + 1);
      res.add(n === 0 ? base : `${base}-${n}`);
    }
    for (const m of l.matchAll(/<a\s+(?:id|name)="([^"]+)"/g)) res.add(m[1]);
  }
  ancresParFichier.set(fichier, res);
  return res;
}

const echecs = [];
let total = 0;
for (const fichier of fichiers) {
  lignesHorsCode(readFileSync(fichier, "utf8")).forEach((ligne, i) => {
    const sansCode = ligne.replace(/`[^`]*`/g, "");
    const liens = [
      ...sansCode.matchAll(/!?\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+"[^"]*")?\s*\)/g),
      ...sansCode.matchAll(/<(?:a|img)\b[^>]*?\s(?:href|src)="([^"]+)"/g),
    ];
    for (const m of liens) {
      const cible = m[1];
      if (/^[a-z][a-z0-9+.-]*:/i.test(cible)) continue; // http:, https:, mailto:...
      total++;
      const [chemin, ancre] = cible.split("#");
      const vise = chemin ? normalize(join(dirname(fichier), decodeURIComponent(chemin))) : fichier;
      const ou = `${fichier}:${i + 1}`;
      if (!existsSync(vise)) { echecs.push(`${ou} — fichier absent : ${cible}`); continue; }
      if (ancre === undefined || ancre === "") continue;
      if (statSync(vise).isDirectory() || !vise.endsWith(".md")) {
        echecs.push(`${ou} — ancre sur autre chose qu'un fichier Markdown : ${cible}`);
        continue;
      }
      if (!ancres(vise).has(decodeURIComponent(ancre))) {
        echecs.push(`${ou} — ancre introuvable : ${cible}`);
      }
    }
  });
}

for (const e of echecs) console.log(`REFUSE — ${e}`);
console.log(`${total} liens internes dans ${fichiers.length} fichiers, ${echecs.length} casses.`);
process.exit(echecs.length ? 1 : 0);
