#!/usr/bin/env node
// Verification des diagrammes de la documentation.
//
// Chaque bloc ```mermaid d'un fichier Markdown doit se rendre sans erreur : un
// diagramme qui ne se rend pas s'affiche sur GitHub comme un message d'erreur.
// Un seul navigateur pour tous les blocs.
//
// Prerequis (non versionnes) :
//   npm install --no-save --no-package-lock @mermaid-js/mermaid-cli@12.0.0
// CHROME_PATH, s'il est defini, designe le navigateur a employer ; sinon celui
// que Puppeteer a telecharge.
//
// Usage : node scripts/verif-diagrammes.mjs
// Sortie : 0 si tous les diagrammes se rendent, 1 sinon.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import puppeteer from "puppeteer";
import { renderMermaid } from "@mermaid-js/mermaid-cli";

// Chemins gérés par modele-projet : leurs liens et leurs diagrammes sont l'affaire du modèle.
const GERES = /^(\.githooks|scripts\/modele|docs\/modele|\.github\/modele|\.claude\/skills\/consigner)\/|^\.github\/workflows\/modele-[^/]*\.yml$/;

const fichiers = execFileSync(
  "git", ["ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
  { encoding: "utf8" },
).split("\n").filter((f) => f && existsSync(f) && !GERES.test(f));

const blocs = [];
for (const fichier of fichiers) {
  let debut = 0;
  let tampon = null;
  readFileSync(fichier, "utf8").split("\n").forEach((l, i) => {
    if (tampon === null && l.trim() === "```mermaid") { tampon = []; debut = i + 1; return; }
    if (tampon !== null && l.trim() === "```") {
      blocs.push({ fichier, ligne: debut, code: tampon.join("\n") });
      tampon = null;
      return;
    }
    if (tampon !== null) tampon.push(l);
  });
}

const navigateur = await puppeteer.launch({
  headless: true,
  executablePath: process.env.CHROME_PATH || undefined,
  args: ["--no-sandbox", "--disable-gpu"],
});
const echecs = [];
try {
  for (const b of blocs) {
    try {
      await renderMermaid(navigateur, b.code, "svg", {
        mermaidConfig: { securityLevel: "strict" },
        fontEmbed: false,
      });
    } catch (e) {
      const message = String(e?.message ?? e).split("\n").filter(Boolean).slice(0, 3).join(" / ").slice(0, 300);
      echecs.push(`${b.fichier}:${b.ligne} — ${message}`);
    }
  }
} finally {
  await navigateur.close();
}

for (const e of echecs) console.log(`REFUSE — ${e}`);
console.log(`${blocs.length} diagrammes dans ${fichiers.length} fichiers, ${echecs.length} en echec.`);
process.exit(echecs.length ? 1 : 0);
