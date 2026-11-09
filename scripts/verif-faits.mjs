#!/usr/bin/env node
// Vérification des faits recopiés : une plage, un compte ou une liste écrits en plusieurs
// endroits disent la même chose que leur source.
//
// C'est la porte du motif « Un compte recopié en plusieurs endroits » (docs/registre.md :
// AM-1, LE-8, LE-9). Elle vérifie :
//
// 1. Les familles d'identifiants que le glossaire recense : B, D, N, S, P, R, IR-, INC-, H, I,
//    Q et A. Chacune est définie à un seul endroit, une fois par numéro, et ses numéros se
//    suivent depuis le premier. Toute plage écrite qui part du premier (« Q01–Q22 »,
//    « I01 à I27 », « P1 … P8 », « INC-0 à INC-6 ») finit au dernier.
// 2. Les ADR. Leurs numéros se suivent, et chacune a un titre et un statut lisibles. Dans un
//    paragraphe, tout « N ADR » écrit en chiffres vaut leur nombre ; tout « N acceptées »
//    (proposées, remplacées, abandonnées) d'un paragraphe qui parle d'ADR vaut le compte de ce
//    statut ; et des comptes par statut écrits à côté du total le ventilent en entier. Dans
//    README.md, CLAUDE.md et CONVENTIONS.md, ces comptes se lisent aussi en lettres, et
//    « décisions d'architecture » y vaut « ADR ». L'index
//    docs/adr/README.md recopie le titre, le statut et les principes de chaque ADR, et son
//    graphe leurs dépendances (« S'appuie sur »).
// 3. La SFD. La colonne « Fonctions » de docs/20-sfd/README.md compte les fonctions de chaque
//    module, numérotées sans trou ni doublon. Le tableau « Les fonctions » nomme chacune, avec
//    son titre, et la matrice de traçabilité la rattache à au moins un besoin, chaque besoin
//    ayant sa ligne.
// 4. Les portes de CI. Le tableau de CONVENTIONS.md (« Portes de CI ») et son compte en
//    lettres nomment les contrôles des workflows lancés sur une PR. CLAUDE.md nomme les jobs
//    de ci.yml. Le README ne les recopie pas : il renvoie à CONVENTIONS.md.
// 5. Les contrôles requis. Ils se disent à un seul endroit, « Protection de `main` » dans
//    CONVENTIONS.md : ailleurs, aucune phrase ne dit un contrôle requis, exigé, obligatoire,
//    imposé ou bloquant, qu'elle nomme une porte ou parle de la règle de protection, sauf pour
//    renvoyer à cette section.
//    Ceux qui y sont dits requis sont des portes, leur compte en lettres est juste, et les autres
//    portes y sont dites non requises. En CI, la liste est comparée à la règle de protection
//    réelle, lue sur GitHub ; hors CI, la comparaison est tentée, et son échec n'est qu'un
//    avertissement.
// 6. Les crates. CLAUDE.md nomme les membres de l'espace de travail ; le README ne les
//    recopie pas.
// 7. Les comptes écrits dans README.md, CLAUDE.md et CONVENTIONS.md, en chiffres ou en
//    lettres, devant « portes », « jobs », « modules », « crates » ou « situations » : chacun
//    vaut le total.
// 8. Les critères livrés. Les critères d'acceptation de chaque fonction de la SFD se numérotent
//    depuis 1, sans trou ni doublon, un critère retiré gardant son numéro. Le tableau « Les
//    critères livrés » du § 70 · 4.1 inscrit des critères que la SFD a, non retirés, une ligne
//    par fonction, et chacun a au moins un test Rust qui le nomme : une fonction
//    « fn mic12_1_… » (le critère 1 de MIC-12), marquée #[test], sans #[ignore]. Une fonction
//    Rust nommée ainsi d'après un critère que la SFD n'a pas, ou retiré, est refusée aussi, comme
//    un nom écrit « mic_12_1_… ». C'est la porte d'un second motif, « Une spec qui se contredit »
//    (AM-2, LE-6, LE-11). Elle vérifie qu'un test qui tourne nomme chaque critère inscrit ; qu'il
//    en calcule l'exemple, c'est la revue de la PR qui le vérifie.
//
// Trois conventions en découlent. Une plage qui part du premier numéro couvre toute la famille.
// Un compte partiel ne s'écrit pas en chiffres devant « ADR », ni, dans ces trois fichiers, en
// lettres devant « ADR », « décisions d'architecture » et les noms du point 7. Un fait qui
// n'apporte rien là où on le recopierait ne s'y écrit pas : on renvoie à sa source.
//
// Usage : node scripts/verif-faits.mjs
// Sortie : 0 si chaque fait recopié dit vrai, 1 sinon.
// Aucune dépendance : Node 22 ou plus suffit.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

// Chemins gérés par modele-projet : leurs textes sont l'affaire du modèle.
const GERES = /^(\.githooks|scripts\/modele|docs\/modele|\.github\/modele|\.claude\/skills\/consigner)\/|^\.github\/workflows\/modele-[^/]*\.yml$/;
// Ce qui ne dit pas l'état présent : le registre cite des états passés, un plan un état à venir.
const HORS_ETAT = /^(docs\/registre\.md|\.claude\/plans\/)/;
// Les trois fichiers qui disent l'état du dépôt : leurs comptes sont lus aussi en lettres.
const ETAT = ["README.md", "CLAUDE.md", "CONVENTIONS.md"];

// Les familles d'identifiants : leur premier numéro, où elles sont définies, et par quels
// motifs. Chaque motif relève chaque numéro au plus une fois.
export const FAMILLES = [
  { prefixe: "B", premier: "01", fichier: "docs/11-persona-et-situations.md", motifs: [/^\| \*\*B(\d\d)\*\* \|/gm] },
  { prefixe: "D", premier: "1", fichier: "docs/11-persona-et-situations.md", motifs: [/^\| D(\d+) \|/gm] },
  { prefixe: "N", premier: "1", fichier: "docs/11-persona-et-situations.md", motifs: [/^\| N(\d+) \|/gm] },
  { prefixe: "S", premier: "1", fichier: "docs/11-persona-et-situations.md", motifs: [/^### S(\d+) ·/gm] },
  { prefixe: "P", premier: "1", fichier: "docs/10-vision.md", motifs: [/^\| P(\d+) \|/gm] },
  { prefixe: "R", premier: "1", fichier: "docs/10-vision.md", motifs: [/^\| R(\d+) \|/gm] },
  { prefixe: "IR-", premier: "1", fichier: "docs/10-vision.md", motifs: [/^\| IR-(\d+) \|/gm] },
  { prefixe: "INC-", premier: "0", fichier: "docs/40-architecture.md", motifs: [/^\| \*\*INC-(\d+)\*\* \|/gm] },
  { prefixe: "H", premier: "01", fichier: "docs/40-architecture.md", motifs: [/^\| (?:\*\*)?H(\d\d)(?:\*\*)? \|/gm] },
  { prefixe: "I", premier: "01", fichier: "docs/70-qualite-et-verification.md", motifs: [/^\| (?:\*\*)?I(\d\d)(?:\*\*)? \|/gm] },
  { prefixe: "Q", premier: "01", fichier: "docs/90-questions-ouvertes.md", motifs: [/^#{3,4} Q(\d\d) —/gm, /^\| (?:\*\*)?Q(\d\d)(?:\*\*)? \|/gm] },
  { prefixe: "A", premier: "01", fichier: "docs/90-questions-ouvertes.md", motifs: [/^\| (?:\*\*)?A(\d\d)(?:\*\*)? \|/gm] },
];

const STATUTS = ["acceptée", "proposée", "remplacée", "abandonnée"];
const STATUT = /(?<!\p{L})(acceptée|proposée|remplacée|abandonnée)(?!\p{L})/u;
// Une phrase qui dit un contrôle requis, exigé, obligatoire, imposé ou bloquant, à tous les temps
// (« requiert », « requis », « requière », « requérir », « requerra »), mais pas « impossible ».
const REQUIS = /(?<!\p{L})(exig|requ(?:i|é|err)|bloqu|oblig|impos(?!s))/iu;
// Une phrase qui parle de la règle de protection sans nommer de porte.
const REGLE = /règle de protection|branche-principale|contrôles? du socle/iu;
// Un renvoi à la section qui fait foi.
const RENVOI = /Protection de `main`/;

const VALEURS = {
  un: 1, une: 1, deux: 2, trois: 3, quatre: 4, cinq: 5, six: 6, sept: 7, huit: 8, neuf: 9,
  dix: 10, onze: 11, douze: 12, treize: 13, quatorze: 14, quinze: 15, seize: 16,
  vingt: 20, trente: 30, quarante: 40, cinquante: 50, soixante: 60,
};

// Un nombre écrit en toutes lettres (« Sept », « vingt et une », « dix-huit »), ou null.
export function nombreEnLettres(texte) {
  const mots = texte.toLowerCase().split(/[\s-]+/).filter(Boolean);
  if (!mots.length) return null;
  let total = 0;
  for (const [i, m] of mots.entries()) {
    if (m === "et") { if (i === 0 || i === mots.length - 1) return null; continue; }
    if (!(m in VALEURS)) return null;
    total += VALEURS[m];
  }
  return total;
}

const pluriel = (n, un, plusieurs) => `${n} ${n > 1 ? plusieurs : un}`;
const liste = (xs) => xs.join(", ");
const enCode = (texte) => [...texte.matchAll(/`([^`]+)`/g)].map((m) => m[1]);
const memes = (a, b) => {
  const x = [...a].sort();
  const y = [...b].sort();
  return x.length === y.length && x.every((v, i) => v === y[i]);
};
const num = (prefixe, n, largeur) => `${prefixe}${String(n).padStart(largeur, "0")}`;

// Les lignes d'un texte Markdown, blocs de code délimités vidés ; les numéros de ligne restent.
function lignesHorsCode(texte) {
  let dedans = false;
  return texte.split("\n").map((l) => {
    if (/^\s*(```|~~~)/.test(l)) { dedans = !dedans; return ""; }
    return dedans ? "" : l;
  });
}

// Les paragraphes d'un texte Markdown, hors blocs de code : des lignes qui se suivent, jointes.
// Une ligne vide, un titre, une ligne de tableau, une puce ou le début d'une citation en ouvre
// un nouveau ; une ligne de tableau est un paragraphe à elle seule, une citation se prolonge tant
// que ses lignes commencent par « > », marque ôtée. L'apostrophe typographique devient droite.
// Chacun garde le début de chaque ligne, pour situer.
export function paragraphes(texte) {
  const res = [];
  let courant = null;
  const fermer = () => { if (courant) res.push(courant); courant = null; };
  lignesHorsCode(texte).forEach((l, i) => {
    const tableau = /^\s*\|/.test(l);
    const citation = /^\s*>/.test(l);
    if (!l.trim()) { fermer(); return; }
    if (citation ? !courant?.citation : tableau || courant?.citation || /^\s*(#{1,6}\s|[-*+]\s|\d+\.\s)/.test(l)) fermer();
    if (!courant) courant = { ligne: i + 1, texte: "", debuts: [], tableau, citation };
    if (courant.texte) courant.texte += " ";
    courant.debuts.push({ offset: courant.texte.length, ligne: i + 1 });
    courant.texte += (citation ? l.replace(/^\s*>\s?/, "") : l).trim().replace(/’/g, "'");
    if (tableau) fermer();
  });
  fermer();
  return res;
}

// Le compte écrit juste avant une position : des chiffres (« 21 ADR », « (10 acceptées »), ou,
// si lettres est vrai, des mots (« vingt et une ADR », « **six** crates »). Le gras, les
// parenthèses et les guillemets autour sont ôtés. « un » et « une » seuls sont des articles, et
// un nombre qui commence par 0 est le numéro d'un identifiant (« ADR-019 et 020 »), pas un compte.
export function compteDevant(texte, offset, lettres = true) {
  const mots = texte.slice(0, offset).trimEnd().split(/\s+/)
    .map((m) => m.replace(/^[*_(«“"]+|[*_)»”",;:.]+$/g, ""));
  // Où le compte commence, pour le situer : la dernière occurrence de son premier mot.
  const debut = (premier) => texte.lastIndexOf(premier, offset - 1);
  const dernier = mots[mots.length - 1] ?? "";
  if (/^[1-9]\d*$/.test(dernier)) return { valeur: Number(dernier), ecrit: dernier, chiffres: true, debut: debut(dernier) };
  if (!lettres) return null;
  let k = mots.length;
  while (k > 0 && (mots[k - 1].toLowerCase() === "et" || nombreEnLettres(mots[k - 1]) !== null)) k--;
  const suite = mots.slice(k);
  while (suite[0]?.toLowerCase() === "et") suite.shift();
  if (!suite.length) return null;
  const valeur = nombreEnLettres(suite.join(" "));
  if (valeur === null || (valeur === 1 && suite.length === 1)) return null;
  const ecrit = suite.join(" ");
  return { valeur, ecrit, chiffres: false, debut: texte.lastIndexOf(suite[0], offset - ecrit.length) };
}

// La ligne d'une position dans le texte joint d'un paragraphe.
function ligneDe(p, offset) {
  let ligne = p.ligne;
  for (const d of p.debuts) if (d.offset <= offset) ligne = d.ligne;
  return ligne;
}

// Les numéros qu'un motif relève (son premier groupe), triés, avec ceux qu'il relève deux fois.
export function numeros(texte, motif) {
  const vus = new Set();
  const doubles = new Set();
  for (const m of texte.matchAll(motif)) {
    const n = Number(m[1]);
    if (vus.has(n)) doubles.add(n);
    vus.add(n);
  }
  return { nums: [...vus].sort((a, b) => a - b), doubles: [...doubles].sort((a, b) => a - b) };
}

// Les numéros qui manquent entre le premier et le plus grand.
export function trous(nums, premier = 1) {
  const presents = new Set(nums);
  const res = [];
  for (let n = premier; n <= Math.max(premier - 1, ...nums); n++) if (!presents.has(n)) res.push(n);
  return res;
}

// Les plages d'une famille qui partent de son premier numéro, et le numéro où chacune finit.
export function plages(texte, prefixe, premier = "01") {
  const motif = new RegExp(
    `(?<![\\p{L}\\p{N}-])${prefixe}${premier}\\s*(?:–|—|-|…|\\.\\.\\.|à)\\s*(?:${prefixe})?(\\d+)(?![\\p{L}\\p{N}])`,
    "gu",
  );
  const res = [];
  lignesHorsCode(texte).forEach((l, i) => {
    for (const m of l.matchAll(motif)) res.push({ ligne: i + 1, fin: Number(m[1]) });
  });
  return res;
}

// Une ADR, lue dans son fichier.
export function lireAdr(nom, texte) {
  const champ = (cle) => new RegExp(`^\\| \\*\\*${cle}\\*\\* \\| (.+?) \\|\\s*$`, "m").exec(texte)?.[1] ?? "";
  const statut = champ("Statut");
  const principes = champ("Sert").split("·").find((s) => /principe/.test(s)) ?? "";
  return {
    num: Number(/^adr-(\d{3})-/.exec(nom)[1]),
    titre: /^# ADR-\d{3} — (.+?)\s*$/m.exec(texte)?.[1] ?? null,
    statut: STATUT.exec(statut)?.[1] ?? null,
    remplacante: Number(/remplacée par \[?ADR-(\d{3})/.exec(statut)?.[1]) || null,
    principes: numeros(principes, /P(\d+)/g).nums,
    appuis: numeros(champ("S'appuie sur"), /ADR-(\d{3})/g).nums,
  };
}

// Les comptes d'ADR, paragraphe par paragraphe : le total, et les statuts d'un paragraphe qui
// parle d'ADR. En chiffres partout ; si lettres est vrai (les trois fichiers d'état), aussi en
// lettres, et « décisions d'architecture » vaut « ADR ». Le numéro d'un identifiant
// (« ADR-006 acceptées ») n'est pas un compte.
export function comptesAdr(texte, { lettres = false } = {}) {
  const res = [];
  const noms = lettres ? "ADR|décisions d'architecture" : "ADR";
  const total = new RegExp(`(?<![\\p{L}\\p{N}-])(${noms})(?![\\p{L}\\p{N}-])`, "gu");
  paragraphes(texte).forEach((p, k) => {
    for (const m of p.texte.matchAll(total)) {
      const c = compteDevant(p.texte, m.index, lettres);
      if (c) res.push({ ligne: ligneDe(p, c?.debut ?? m.index), paragraphe: k, quoi: "total", valeur: c.valeur, ecrit: c.ecrit });
    }
    if (!new RegExp(noms).test(p.texte)) return;
    for (const m of p.texte.matchAll(/(?<![\p{L}\p{N}])(acceptée|proposée|remplacée|abandonnée)s?(?!\p{L})/gu)) {
      const c = compteDevant(p.texte, m.index, lettres);
      if (c) res.push({ ligne: ligneDe(p, c?.debut ?? m.index), paragraphe: k, quoi: m[1], valeur: c.valeur, ecrit: c.ecrit });
    }
  });
  return res;
}

// Les lignes de « La série » de l'index : | [ADR-001](…) | Titre | ✅ acceptée | P1 |
export function lireIndexAdr(texte) {
  const res = new Map();
  lignesHorsCode(texte).forEach((l, i) => {
    const m = /^\| \[ADR-(\d{3})\]\([^)]*\) \| (.+?) \| (.+?) \| (.+?) \|\s*$/.exec(l);
    if (!m) return;
    res.set(Number(m[1]), {
      ligne: i + 1,
      titre: m[2].trim(),
      statut: STATUT.exec(m[3])?.[1] ?? null,
      remplacante: Number(/remplacée par \[?ADR-(\d{3})/.exec(m[3])?.[1]) || null,
      principes: numeros(m[4], /P(\d+)/g).nums,
    });
  });
  return res;
}

// Le graphe Mermaid de l'index : ses nœuds A<n>, ses arêtes « x>y » (y s'appuie sur x) et les
// lignes à flèche qu'il ne sait pas lire. Une ligne peut chaîner des flèches (A1 --> A2 --> A3),
// les allonger (--->), les étiqueter (-->|texte|) et grouper des nœuds (A1 & A2 --> A3).
export function lireGraphe(texte) {
  const debut = texte.split("\n").findIndex((l) => /^```mermaid\s*$/.test(l));
  const lignes = debut < 0 ? [] : texte.split("\n").slice(debut + 1);
  const noeuds = new Map();
  const aretes = new Set();
  const illisibles = [];
  for (const [k, l] of lignes.entries()) {
    if (/^```/.test(l)) break;
    const ligne = debut + 2 + k;
    for (const m of l.matchAll(/\bA(\d+)\["([^"]*)"\](?::::(\w+))?/g)) {
      noeuds.set(Number(m[1]), { libelle: m[2], classe: m[3] ?? null, ligne });
    }
    for (const m of l.matchAll(/\bA(\d+)\[(?!")/g)) illisibles.push({ ligne, texte: `nœud A${m[1]} sans libellé entre guillemets` });
    const nue = l.replace(/\["[^"]*"\]/g, "").replace(/:::\w+/g, "");
    // Tout autre lien (==>, ---, --o, -.->) serait une dépendance que la porte ne lit pas.
    if (!/--|==|-\./.test(nue)) continue;
    const segments = nue.split(/\s*-{2,}>\s*(?:\|[^|]*\|\s*)?/);
    const groupes = segments.map((s) => s.split("&").map((x) => /^\s*A(\d+)\s*$/.exec(x)?.[1]));
    if (segments.length < 2 || groupes.some((g) => g.some((x) => x === undefined))) { illisibles.push({ ligne, texte: l.trim() }); continue; }
    for (let i = 0; i + 1 < groupes.length; i++) {
      for (const x of groupes[i]) for (const y of groupes[i + 1]) aretes.add(`${Number(x)}>${Number(y)}`);
    }
  }
  return { noeuds, aretes, illisibles };
}

// L'index et le graphe d'ADR face aux fichiers : un écart par divergence.
export function verifierAdr(adrs, texteIndex, fichierIndex = "docs/adr/README.md") {
  const ecarts = [];
  const ecart = (ligne, message) => ecarts.push({ fichier: fichierIndex, ligne, message });
  const index = lireIndexAdr(texteIndex);
  const { noeuds, aretes, illisibles } = lireGraphe(texteIndex);
  const nom = (n) => num("ADR-", n, 3);
  const statutDe = (s, r) => `${s}${r ? ` par ${nom(r)}` : ""}`;
  for (const i of illisibles) ecart(i.ligne, `ligne du graphe illisible : ${i.texte} ; écrire « Ax --> Ay », « Ax["libellé"] »`);
  for (const a of adrs) {
    const e = index.get(a.num);
    if (!e) ecart(1, `${nom(a.num)} absente de l'index`);
    else {
      if (e.titre !== a.titre) ecart(e.ligne, `${nom(a.num)} : titre « ${e.titre} » dans l'index, « ${a.titre} » dans son fichier`);
      if (e.statut !== a.statut || e.remplacante !== a.remplacante) {
        ecart(e.ligne, `${nom(a.num)} : statut « ${statutDe(e.statut, e.remplacante)} » dans l'index, « ${statutDe(a.statut, a.remplacante)} » dans son fichier`);
      }
      if (!memes(e.principes, a.principes)) {
        ecart(e.ligne, `${nom(a.num)} : principes P${e.principes.join(", P")} dans l'index, P${a.principes.join(", P")} dans son fichier (« Sert »)`);
      }
    }
    const n = noeuds.get(a.num);
    if (!n) { ecart(1, `${nom(a.num)} absente du graphe`); continue; }
    if (!n.libelle.startsWith(nom(a.num))) ecart(n.ligne, `${nom(a.num)} : le nœud A${a.num} du graphe s'intitule « ${n.libelle} »`);
    const remplacee = a.statut === "remplacée";
    if (remplacee !== (n.classe === "remplacee") || remplacee !== n.libelle.includes("remplacée")) {
      ecart(n.ligne, `${nom(a.num)} : le nœud du graphe porte « remplacée » et la classe remplacee si, et seulement si, l'ADR est remplacée`);
    }
  }
  for (const [k, e] of index) if (!adrs.some((a) => a.num === k)) ecart(e.ligne, `${nom(k)} dans l'index, sans fichier`);
  const attendues = new Set(adrs.flatMap((a) => a.appuis.map((p) => `${p}>${a.num}`)));
  for (const k of attendues) if (!aretes.has(k)) {
    const [x, y] = k.split(">");
    ecart(1, `arête A${x} --> A${y} absente du graphe : ${nom(y)} s'appuie sur ${nom(x)}`);
  }
  for (const k of aretes) if (!attendues.has(k)) {
    const [x, y] = k.split(">");
    ecart(1, `arête A${x} --> A${y} du graphe, absente de la ligne « S'appuie sur » de ${nom(y)}`);
  }
  return ecarts;
}

// Le tableau des modules de la SFD : | **CAP** · … | [21-….md](21-….md) | 7 |
export function lireModulesSfd(texte) {
  const res = [];
  lignesHorsCode(texte).forEach((l, i) => {
    const m = /^\| \*\*([A-Z]{3})\*\* · .+? \| \[[^\]]+\]\(([^)]+)\) \| (\d+) \|\s*$/.exec(l);
    if (m) res.push({ ligne: i + 1, code: m[1], fichier: m[2], compte: Number(m[3]) });
  });
  return res;
}

// Les fonctions d'un module, d'après leurs titres : ### CAP-01 · Capter en un geste
export function fonctionsDuModule(texte, code) {
  return [...texte.matchAll(new RegExp(`^### (${code}-\\d{2}) · (.+?)\\s*$`, "gm"))].map((m) => ({ id: m[1], titre: m[2] }));
}

// Le tableau « Les fonctions » de l'index de la SFD : | [CAP-01](…) Capter en un geste | …
export function lireIndexSfd(texte) {
  const s = section(texte, "## Les fonctions");
  if (!s) return null;
  return s.lignes.flatMap((l, i) => {
    const m = /^\| \[([A-Z]{3}-\d{2})\]\([^)]*\) (.+?) \|/.exec(l);
    return m ? [{ ligne: s.debut + 1 + i, id: m[1], titre: m[2].trim() }] : [];
  });
}

// La matrice de traçabilité : | **B01** Capter partout | CAP-01 · ECO-06 |
export function lireMatrice(texte) {
  const s = section(texte, "## Matrice de traçabilité");
  if (!s) return null;
  return s.lignes.flatMap((l, i) => {
    const m = /^\| \*\*B(\d\d)\*\* .+? \| (.+?) \|\s*$/.exec(l);
    return m ? [{ ligne: s.debut + 1 + i, besoin: Number(m[1]), fonctions: [...m[2].matchAll(/[A-Z]{3}-\d{2}/g)].map((x) => x[0]) }] : [];
  });
}

// Les jobs d'un workflow : leur clé, leur nom (« name: »), le workflow qu'ils appellent, et
// s'ils portent une matrice.
export function jobsDuWorkflow(yaml) {
  const lignes = yaml.split("\n");
  const debut = lignes.findIndex((l) => /^jobs:\s*(#.*)?$/.test(l));
  if (debut < 0) return [];
  const jobs = [];
  let courant = null;
  for (const l of lignes.slice(debut + 1)) {
    if (/^[^\s#]/.test(l)) break;
    const j = /^ {2}([A-Za-z0-9_-]+):\s*(#.*)?$/.exec(l);
    if (j) { courant = { id: j[1], nom: null, appelle: null, matrice: false }; jobs.push(courant); continue; }
    if (!courant) continue;
    const n = /^ {4}name:\s*["']?(.+?)["']?\s*$/.exec(l);
    if (n) courant.nom = n[1];
    const u = /^ {4}uses:\s*\.\/(\S+)\s*$/.exec(l);
    if (u) courant.appelle = u[1];
    if (/^ {4}strategy:/.test(l)) courant.matrice = true;
  }
  return jobs;
}

// Le workflow se lance-t-il sur une PR ?
export function lanceSurPr(yaml) {
  const on = /^on:(.*(?:\n(?:[ \t].*)?)*)/m.exec(yaml)?.[1] ?? "";
  return /(?<![\w-])pull_request(?![\w-])/.test(on);
}

// Les contrôles que GitHub pose sur une PR : « job », ou « job / job appelé ». Un nom que la
// porte ne sait pas calculer (une matrice, une expression) lève une erreur qui le dit.
export function controlesDesWorkflows(fichiers) {
  const res = [];
  const nomme = (chemin, job) => {
    if (job.matrice || /\$\{\{/.test(job.nom ?? "")) {
      throw new Error(`${chemin} : le job ${job.id} porte une matrice ou un nom calculé ; la porte ne sait pas nommer ses contrôles (CONVENTIONS.md : « Pas de matrice »)`);
    }
    return job.nom ?? job.id;
  };
  for (const [chemin, yaml] of fichiers) {
    if (!lanceSurPr(yaml)) continue;
    const jobs = jobsDuWorkflow(yaml);
    if (!jobs.length) throw new Error(`${chemin} : lancé sur une PR, mais aucun job n'y est lisible`);
    for (const job of jobs) {
      const nom = nomme(chemin, job);
      if (!job.appelle) { res.push(nom); continue; }
      const appele = fichiers.get(job.appelle);
      if (appele === undefined) throw new Error(`${chemin} : le job ${job.id} appelle ${job.appelle}, introuvable`);
      const sous = jobsDuWorkflow(appele);
      if (!sous.length) throw new Error(`${job.appelle} : aucun job n'y est lisible`);
      for (const s of sous) res.push(`${nom} / ${nomme(job.appelle, s)}`);
    }
  }
  return res;
}

// Une section Markdown, de son titre au titre suivant de même niveau ou plus haut.
export function section(texte, titre) {
  const lignes = texte.split("\n");
  const i = lignes.indexOf(titre);
  if (i < 0) return null;
  const niveau = /^#+/.exec(titre)[0].length;
  let fin = lignes.length;
  for (let k = i + 1; k < lignes.length; k++) {
    const t = /^(#+)\s/.exec(lignes[k]);
    if (t && t[1].length <= niveau) { fin = k; break; }
  }
  return { debut: i + 1, fin, lignes: lignes.slice(i + 1, fin) };
}

// « Portes de CI » : le compte en lettres (« **Sept, aujourd'hui.** ») et les portes du tableau.
export function portesDuTableau(s) {
  let compte = null;
  const portes = [];
  for (const l of s.lignes) {
    const c = /^\*\*([\p{L} -]+), aujourd'hui\.\*\*/u.exec(l);
    if (c) compte = nombreEnLettres(c[1]);
    const p = /^\| `([^`]+)` \|/.exec(l);
    if (p) portes.push(p[1]);
  }
  return { compte, portes };
}

// « Protection de `main` » : « requiert N contrôles : `a`, `b` et `c`. », puis, s'il en reste,
// « `d` et `e` n'y sont pas requis. »
export function controlesRequisEcrits(s) {
  const texte = s.lignes.join(" ").replace(/\s+/g, " ");
  const r = /requiert ([\p{L} -]+?) contrôles : ([^.]+)\./u.exec(texte);
  const n = /((?:`[^`]+`(?:, | et )?)+) n'y (?:est|sont) pas requis/u.exec(texte);
  return {
    compte: r ? nombreEnLettres(r[1]) : null,
    requis: r ? enCode(r[2]) : null,
    nonRequis: n ? enCode(n[1]) : null,
  };
}

// Les phrases qui disent des contrôles requis, exigés, obligatoires ou bloquants : elles n'ont
// leur place qu'à « Protection de `main` ». Une phrase est retenue si elle dit l'un de ces mots
// et nomme une porte en code, ou parle de la règle de protection ; une phrase qui renvoie à
// « Protection de `main` » ne l'est pas. Dans une ligne de tableau, la porte en code est la clé
// de la ligne : seule la règle de protection y compte.
export function phrasesSurLesRequis(texte, portes) {
  const res = [];
  for (const p of paragraphes(texte)) {
    let offset = 0;
    for (const phrase of p.texte.split(/(?<=[.;!?])\s+/)) {
      const ou = p.texte.indexOf(phrase, offset);
      offset = ou + phrase.length;
      if (!REQUIS.test(phrase) || RENVOI.test(phrase)) continue;
      const nommees = p.tableau ? [] : enCode(phrase).filter((c) => portes.includes(c));
      if (nommees.length || REGLE.test(phrase)) res.push({ ligne: ligneDe(p, ou), portes: nommees });
    }
  }
  return res;
}

// CLAUDE.md : « La CI (…) a pour jobs `a`, `b` et `c`. »
export function jobsDeClaude(texte) {
  const lignes = texte.split("\n");
  const i = lignes.findIndex((l) => /a pour jobs /.test(l));
  if (i < 0) return null;
  return { ligne: i + 1, jobs: enCode(/a pour jobs (.+?)\./.exec(lignes[i])[1]) };
}

// Les membres de l'espace de travail, dans le Cargo.toml de la racine.
export function membresDuCargo(toml) {
  const m = /^members\s*=\s*\[([^\]]*)\]/m.exec(toml.replace(/#.*$/gm, ""));
  return m ? [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : [];
}

// Le nom du paquet d'un Cargo.toml.
export function nomDuPaquet(toml) {
  return /^\[package\][^[]*?^name\s*=\s*"([^"]+)"/m.exec(toml)?.[1] ?? null;
}

// CLAUDE.md : « - Les crates : », puis une puce par crate : « - `noyau` (`crates/noyau`) : … ».
export function cratesDeClaude(texte) {
  const lignes = texte.split("\n");
  const i = lignes.findIndex((l) => /^- Les crates :/.test(l));
  if (i < 0) return null;
  const crates = [];
  for (const l of lignes.slice(i + 1)) {
    const m = /^ {2}- `([^`]+)` \(`([^`]+)`\)/.exec(l);
    if (!m) break;
    crates.push({ nom: m[1], chemin: m[2] });
  }
  return { ligne: i + 1, crates };
}

// Les comptes écrits devant un nom : « sept portes », « 6 jobs », « **six** crates ».
export function comptesNommes(texte, noms) {
  const motif = new RegExp(`(?<![\\p{L}\\p{N}])(${noms.join("|")})(?![\\p{L}\\p{N}-])`, "gu");
  const res = [];
  for (const p of paragraphes(texte)) {
    for (const m of p.texte.matchAll(motif)) {
      const c = compteDevant(p.texte, m.index);
      if (c) res.push({ nom: m[1], ...c, ligne: ligneDe(p, c.debut) });
    }
  }
  return res;
}

// Les critères d'acceptation des fonctions d'un module : pour chaque fonction, les numéros lus
// sous « **Critères d'acceptation** », jusqu'au gras, au filet ou au titre suivants, chacun avec
// sa ligne, et s'il est retiré (« n. ~~texte~~ Retiré : raison. »). Un critère est une ligne qui
// commence par « n. » ; ses lignes de suite, en retrait, n'en sont pas, ni les listes numérotées
// des règles.
export function criteresDuModule(texte, code) {
  const res = new Map();
  const titre = new RegExp(`^### (${code}-\\d{2}) · `);
  let id = null;
  let dedans = false;
  lignesHorsCode(texte).forEach((l, i) => {
    const t = titre.exec(l);
    if (t) {
      id = t[1];
      dedans = false;
      if (!res.has(id)) res.set(id, []);
      return;
    }
    if (/^#{1,6}\s/.test(l)) { id = null; dedans = false; return; }
    if (!id) return;
    if (/^\*\*Critères d['’]acceptation\*\*\s*$/.test(l)) { dedans = true; return; }
    if (!dedans) return;
    if (/^\*\*/.test(l) || /^(-{3,}|_{3,})\s*$/.test(l)) { dedans = false; return; }
    const m = /^(\d+)\.\s+(~~)?/.exec(l);
    if (m) res.get(id).push({ n: Number(m[1]), ligne: i + 1, retire: Boolean(m[2]) });
  });
  return res;
}

// Le tableau des critères livrés (§ 70 · 4.1) : | CAP-06 | 1, 4, 7 à 10 | INC-1, lot 1 |
// L'en-tête précède le filet ; les lignes le suivent jusqu'à la première ligne vide, la barre
// initiale facultative, comme sur GitHub. La fonction se nomme seule ou par un lien ; ses
// critères, par numéros et par plages « n à m », séparés par des virgules. Une ligne qui ne se
// lit pas ainsi est rendue avec sa raison. Sans tableau, null.
export function lireCriteresLivres(s) {
  const filet = (l) => /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)+\|?\s*$/.test(l);
  const k = s.lignes.findIndex(filet);
  if (k < 0) return null;
  const res = [];
  for (let i = k + 1; i < s.lignes.length && s.lignes[i].trim(); i++) {
    const l = s.lignes[i];
    const ligne = s.debut + 1 + i;
    const m = /^\s*\|?\s*(?:\[([A-Z]{3}-\d{2})\]\([^)]*\)|([A-Z]{3}-\d{2}))\s*\|([^|]*)(?:\||$)/.exec(l);
    if (!m) {
      res.push({ ligne, illisible: "la première colonne nomme une fonction, « CAP-06 » ou « [CAP-06](…) »" });
      continue;
    }
    const id = m[1] ?? m[2];
    const criteres = [];
    let illisible = null;
    for (const morceau of m[3].split(",").map((x) => x.trim())) {
      const r = /^([1-9]\d*)(?:\s+à\s+([1-9]\d*))?$/.exec(morceau);
      if (!r || (r[2] && Number(r[2]) <= Number(r[1]))) {
        illisible = `« ${morceau} » : des numéros et des plages « n à m », n plus petit que m, séparés par des virgules`;
        break;
      }
      for (let n = Number(r[1]); n <= Number(r[2] ?? r[1]); n++) criteres.push(n);
    }
    res.push(illisible ? { ligne, id, illisible } : { ligne, id, criteres });
  }
  return res;
}

// Les attributs d'une fonction Rust, de la ligne `fn` vers le haut : ceux de sa ligne, puis
// les lignes d'attributs, un attribut écrit sur plusieurs lignes compris (rustfmt coupe ainsi
// un attribut de plus de 100 colonnes), les commentaires et les lignes vides ; un commentaire
// en fin de ligne est ôté. La remontée
// s'arrête à toute autre ligne : la fin de l'élément précédent. Chaque attribut est rendu en
// une seule chaîne, « #[should_panic(expected = "…")] ».
function attributsAuDessus(lignes, i, avant) {
  const attributs = [...avant.matchAll(/#!?\[[^\]]*\]/g)].map((a) => a[0]);
  const bloc = [];
  for (let k = i - 1; k >= 0; k--) {
    const t = lignes[k].trim().replace(/\s*\/\/[^"]*$/, "").replace(/^\/\*.*\*\/$/, "");
    if (bloc.length) {
      bloc.unshift(t);
      if (/^#!?\[/.test(t)) { attributs.push(bloc.join(" ")); bloc.length = 0; }
      else if (/[{};]$/.test(t)) break;
      continue;
    }
    if (!t || t.startsWith("//")) continue;
    if (/^#!?\[/.test(t)) {
      if (/\]$/.test(t)) attributs.push(...[...t.matchAll(/#!?\[[^\]]*\]/g)].map((a) => a[0]));
      else break;
      continue;
    }
    if (/\]$/.test(t)) { bloc.push(t); continue; }
    break;
  }
  return attributs;
}

// Les fonctions Rust nommées d'après un critère, à la façon des tests d'INC-0 : « fn
// mic12_1_do_majeur » nomme le critère 1 de MIC-12 (le code de la fonction en minuscules, son
// numéro collé, puis le numéro du critère). Chacune dit si ses attributs en font un test
// (#[test], #[tokio::test]…) et l'ignorent (#[ignore], ou un #[cfg_attr(…)] qui ignore). Une
// fonction écrite « mic_12_1_… » est rendue aussi, marquée : ce n'est pas la forme, et la porte
// le dit. Une ligne commentée par « // » n'en déclare aucune ; un commentaire de bloc ou le corps
// d'une macro ne sont pas distingués : la revue de la PR les voit.
export function fonctionsNommeesCritere(fichier, texte, codes) {
  const nom = new RegExp(`\\bfn\\s+((${codes.map((c) => c.toLowerCase()).join("|")})(_?)(\\d{2})_(\\d+)(?:_[\\p{L}\\p{N}_]*)?)\\s*[(<]`, "u");
  const lignes = texte.replace(/\r\n/g, "\n").split("\n");
  const res = [];
  lignes.forEach((l, i) => {
    if (l.trim().startsWith("//")) return;
    const m = nom.exec(l);
    if (!m) return;
    const noms = attributsAuDessus(lignes, i, l.slice(0, m.index))
      .map((a) => ({ a, nom: /^#!?\[\s*((?:\w+::)*\w+)/.exec(a)?.[1] ?? "" }));
    res.push({
      fichier, ligne: i + 1, nom: m[1], id: `${m[2].toUpperCase()}-${m[4]}`, n: Number(m[5]),
      forme: m[3] === "",
      test: noms.some((x) => /(?:^|::)\w*test$/.test(x.nom)),
      ignore: noms.some((x) => x.nom === "ignore" || (x.nom === "cfg_attr" && /\bignore\b/.test(x.a))),
    });
  });
  return res;
}

// Le point 8. criteres : les critères de chaque fonction de la SFD, id → [{ n, ligne, retire }] ;
// fichierDe : le module de chaque fonction ; livres : le tableau lu par lireCriteresLivres, ou
// null ; tests : les fonctions lues par fonctionsNommeesCritere.
export function verifierCriteres({ criteres, fichierDe, livres, fichierListe, tests }) {
  const ecarts = [];
  const ecart = (fichier, ligne, message) => ecarts.push({ fichier, ligne, message });
  const nomDe = (id, n) => `${id.toLowerCase().replace("-", "")}_${n}`;
  for (const [id, cs] of criteres) {
    if (!cs.length) ecart(fichierDe.get(id), 1, `${id} : aucun critère lu sous « **Critères d'acceptation** », que chaque fonction porte (§ 70 · 4)`);
    const vus = new Set();
    for (const c of cs) {
      if (vus.has(c.n)) ecart(fichierDe.get(id), c.ligne, `${id} · ${c.n} écrit deux fois : un critère garde son numéro, un nouveau prend le suivant`);
      vus.add(c.n);
    }
    const manquent = trous([...vus]);
    if (manquent.length) ecart(fichierDe.get(id), cs[0]?.ligne ?? 1, `${id} : critères absents, ${liste(manquent)} ; un critère retiré garde son numéro (« n. ~~texte~~ Retiré : raison. »)`);
  }
  const critere = (id, n) => criteres.get(id)?.find((c) => c.n === n);
  if (!livres) ecart(fichierListe, 1, "tableau « | Fonction | Critères | Livrés par | » introuvable sous « ### 4.1 Les critères livrés »");
  const inscrits = new Set();
  for (const l of livres ?? []) {
    if (l.illisible) { ecart(fichierListe, l.ligne, `ligne du tableau illisible : ${l.illisible}`); continue; }
    if (inscrits.has(l.id)) ecart(fichierListe, l.ligne, `${l.id} : plusieurs lignes ; une ligne par fonction`);
    inscrits.add(l.id);
    if (!criteres.has(l.id)) { ecart(fichierListe, l.ligne, `${l.id} n'est pas une fonction de la SFD`); continue; }
    const dits = new Set();
    for (const n of l.criteres) {
      if (dits.has(n)) { ecart(fichierListe, l.ligne, `${l.id} · ${n} inscrit deux fois`); continue; }
      dits.add(n);
      const c = critere(l.id, n);
      if (!c) { ecart(fichierListe, l.ligne, `${l.id} · ${n} : la SFD n'a pas ce critère (${fichierDe.get(l.id)})`); continue; }
      if (c.retire) { ecart(fichierListe, l.ligne, `${l.id} · ${n} est retiré (${fichierDe.get(l.id)}:${c.ligne}) : il ne s'inscrit pas`); continue; }
      const nomment = tests.filter((t) => t.forme && t.id === l.id && t.n === n);
      if (nomment.some((t) => t.test && !t.ignore)) continue;
      if (!nomment.length) ecart(fichierListe, l.ligne, `${l.id} · ${n} est inscrit, sans test qui le nomme : une fonction « fn ${nomDe(l.id, n)}_… », marquée #[test]`);
      else ecart(fichierListe, l.ligne, `${l.id} · ${n} est inscrit, mais ${liste(nomment.map((t) => `${t.fichier}:${t.ligne}`))} le nomme sans être un test qui tourne : #[test], sans #[ignore]`);
    }
  }
  for (const t of tests) {
    const c = critere(t.id, t.n);
    if (!t.forme) ecart(t.fichier, t.ligne, `fn ${t.nom} : un nom de test d'après un critère s'écrit « ${nomDe(t.id, t.n)}_… », le code collé à son numéro (§ 70 · 4.1)`);
    else if (!criteres.has(t.id)) ecart(t.fichier, t.ligne, `fn ${t.nom} nomme ${t.id}, qui n'est pas une fonction de la SFD`);
    else if (!c) ecart(t.fichier, t.ligne, `fn ${t.nom} nomme ${t.id} · ${t.n}, que la SFD n'a pas (${fichierDe.get(t.id)})`);
    else if (c.retire) ecart(t.fichier, t.ligne, `fn ${t.nom} nomme ${t.id} · ${t.n}, qui est retiré (${fichierDe.get(t.id)}:${c.ligne})`);
  }
  return ecarts;
}

// La règle de protection réelle de la branche principale, lue sur GitHub.
async function controlesRequisSurGitHub(depot, branche) {
  const jeton = process.env.GITHUB_TOKEN;
  const rep = await fetch(`https://api.github.com/repos/${depot}/rules/branches/${branche}`, {
    headers: {
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...(jeton ? { Authorization: `Bearer ${jeton}` } : {}),
    },
    signal: AbortSignal.timeout(20000),
  });
  if (!rep.ok) {
    const corps = await rep.json().catch(() => ({}));
    // Le message de GitHub peut citer l'adresse IP de l'appelant : elle n'a rien à faire dans un journal.
    const message = String(corps.message ?? "").split(". ")[0].replace(/\b\d{1,3}(?:\.\d{1,3}){3}\b/g, "…");
    throw new Error(`HTTP ${rep.status}${message ? `, « ${message} »` : ""}`);
  }
  const regles = await rep.json();
  return regles
    .filter((r) => r.type === "required_status_checks")
    .flatMap((r) => r.parameters.required_status_checks.map((c) => c.context));
}

async function principal() {
  const lire = (f) => readFileSync(f, "utf8").replace(/\r\n/g, "\n");
  const ecarts = [];
  const ecart = (fichier, ligne, message) => ecarts.push({ fichier, ligne, message });
  const avertissements = [];

  // Les fichiers suivis ou à suivre ; -z garde un nom accentué tel quel (sans -z, git le cite).
  const fichiers = (motif) => execFileSync(
    "git", ["ls-files", "-z", "--cached", "--others", "--exclude-standard", motif],
    { encoding: "utf8" },
  ).split("\0").filter((f) => f && existsSync(f) && !GERES.test(f));
  const markdown = fichiers("*.md").filter((f) => !HORS_ETAT.test(f));
  const textes = new Map(markdown.map((f) => [f, lire(f)]));

  // 1. Les familles d'identifiants.
  const derniers = {};
  for (const { prefixe, premier, fichier, motifs } of FAMILLES) {
    const texte = lire(fichier);
    const largeur = premier.length;
    const nums = new Set();
    for (const motif of motifs) {
      const r = numeros(texte, motif);
      r.nums.forEach((n) => nums.add(n));
      if (r.doubles.length) ecart(fichier, 1, `${liste(r.doubles.map((n) => num(prefixe, n, largeur)))} ${r.doubles.length > 1 ? "définis" : "défini"} deux fois : un identifiant n'a qu'une définition`);
    }
    const tries = [...nums].sort((a, b) => a - b);
    const dernier = tries.length ? tries[tries.length - 1] : null;
    derniers[prefixe] = dernier;
    if (dernier === null) { ecart(fichier, 1, `aucun ${prefixe}… défini : le motif de la famille ne trouve plus rien`); continue; }
    const manquent = trous(tries, Number(premier));
    if (manquent.length) ecart(fichier, 1, `numéros absents : ${liste(manquent.map((n) => num(prefixe, n, largeur)))} ; un identifiant retiré garde sa ligne`);
    for (const [f, t] of textes) {
      for (const p of plages(t, prefixe, premier)) {
        if (p.fin !== dernier) ecart(f, p.ligne, `plage ${prefixe}${premier}–${num(prefixe, p.fin, largeur)} : la famille va jusqu'à ${num(prefixe, dernier, largeur)} (${fichier})`);
      }
    }
  }

  // 2. Les ADR.
  const dossierAdr = "docs/adr";
  const adrs = readdirSync(dossierAdr).filter((f) => /^adr-\d{3}-.*\.md$/.test(f)).sort()
    .map((f) => ({ fichier: `${dossierAdr}/${f}`, ...lireAdr(f, lire(join(dossierAdr, f))) }));
  const manquent = trous(adrs.map((a) => a.num));
  if (manquent.length) ecart(dossierAdr, 1, `ADR absentes : ${liste(manquent.map((n) => num("ADR-", n, 3)))} ; une ADR n'est jamais supprimée`);
  for (const a of adrs) {
    if (!a.titre) ecart(a.fichier, 1, "titre illisible : « # ADR-NNN — Titre » attendu");
    if (!a.statut) ecart(a.fichier, 1, `statut illisible : « | **Statut** | … | » attendu, avec ${liste(STATUTS)}`);
  }
  const comptes = { total: adrs.length };
  for (const s of STATUTS) comptes[s] = adrs.filter((a) => a.statut === s).length;
  for (const [f, t] of textes) {
    const lus = comptesAdr(t, { lettres: ETAT.includes(f) });
    for (const c of lus) {
      if (c.valeur === comptes[c.quoi]) continue;
      ecart(f, c.ligne, c.quoi === "total"
        ? `« ${c.ecrit} » ADR : il y en a ${comptes.total} dans ${dossierAdr}/`
        : `« ${c.ecrit} ${c.quoi}${c.valeur > 1 ? "s" : ""} » : ${pluriel(comptes[c.quoi], "ADR a", "ADR ont")} ce statut`);
    }
    for (const k of new Set(lus.map((c) => c.paragraphe))) {
      const ici = lus.filter((c) => c.paragraphe === k);
      const total = ici.find((c) => c.quoi === "total");
      const parStatut = ici.filter((c) => c.quoi !== "total");
      if (!total || !parStatut.length) continue;
      const somme = parStatut.reduce((s, c) => s + c.valeur, 0);
      if (somme !== total.valeur) {
        const oublies = STATUTS.filter((s) => comptes[s] && !parStatut.some((c) => c.quoi === s));
        ecart(f, total.ligne, `les comptes par statut font ${somme} pour ${total.valeur} ADR${oublies.length ? ` : il manque ${liste(oublies.map((s) => `${comptes[s]} ${s}${comptes[s] > 1 ? "s" : ""}`))}` : ""}`);
      }
    }
  }
  ecarts.push(...verifierAdr(adrs, lire(`${dossierAdr}/README.md`), `${dossierAdr}/README.md`));

  // 3. La SFD.
  const indexSfd = "docs/20-sfd/README.md";
  const texteSfd = lire(indexSfd);
  const modules = lireModulesSfd(texteSfd);
  if (!modules.length) ecart(indexSfd, 1, "tableau des modules illisible");
  const fonctions = [];
  const textesModules = new Map();
  for (const m of modules) {
    textesModules.set(m, lire(join("docs/20-sfd", m.fichier)));
    const ici = fonctionsDuModule(textesModules.get(m), m.code);
    fonctions.push(...ici);
    if (ici.length !== m.compte) ecart(indexSfd, m.ligne, `${m.code} : ${pluriel(m.compte, "fonction écrite", "fonctions écrites")}, ${pluriel(ici.length, "titre", "titres")} « ### ${m.code}-nn · » dans ${m.fichier}`);
    const r = numeros(ici.map((f) => f.id).join("\n"), new RegExp(`^${m.code}-(\\d{2})$`, "gm"));
    if (r.doubles.length) ecart(`docs/20-sfd/${m.fichier}`, 1, `${liste(r.doubles.map((n) => num(`${m.code}-`, n, 2)))} ${r.doubles.length > 1 ? "définies" : "définie"} deux fois`);
    const t = trous(r.nums);
    if (t.length) ecart(`docs/20-sfd/${m.fichier}`, 1, `numéros absents : ${liste(t.map((n) => num(`${m.code}-`, n, 2)))}`);
  }
  const indexFonctions = lireIndexSfd(texteSfd);
  if (!indexFonctions) ecart(indexSfd, 1, "section « ## Les fonctions » introuvable");
  else {
    for (const f of fonctions) {
      const lignes = indexFonctions.filter((e) => e.id === f.id);
      if (lignes.length !== 1) { ecart(indexSfd, 1, `${f.id} : ${lignes.length ? "plusieurs lignes" : "aucune ligne"} dans « Les fonctions »`); continue; }
      if (lignes[0].titre !== f.titre) ecart(indexSfd, lignes[0].ligne, `${f.id} : « ${lignes[0].titre} » dans l'index, « ${f.titre} » dans son module`);
    }
    for (const e of indexFonctions) if (!fonctions.some((f) => f.id === e.id)) ecart(indexSfd, e.ligne, `${e.id} dans « Les fonctions », sans titre dans son module`);
  }
  const matrice = lireMatrice(texteSfd);
  if (!matrice) ecart(indexSfd, 1, "section « ## Matrice de traçabilité » introuvable");
  else {
    for (const f of fonctions) if (!matrice.some((r) => r.fonctions.includes(f.id))) ecart(indexSfd, 1, `${f.id} ne sert aucun besoin dans la matrice de traçabilité`);
    for (const r of matrice) for (const id of r.fonctions) if (!fonctions.some((f) => f.id === id)) ecart(indexSfd, r.ligne, `${id} dans la matrice, sans titre dans son module`);
    for (let b = 1; b <= (derniers.B ?? 0); b++) {
      const n = matrice.filter((r) => r.besoin === b).length;
      if (n !== 1) ecart(indexSfd, 1, `${num("B", b, 2)} : ${n ? "plusieurs lignes" : "aucune ligne"} dans la matrice de traçabilité`);
    }
  }

  // 4. Les portes de CI.
  const dossierWf = ".github/workflows";
  const workflows = new Map(readdirSync(dossierWf).filter((f) => /\.ya?ml$/.test(f))
    .map((f) => [`${dossierWf}/${f}`, lire(join(dossierWf, f))]));
  const controles = controlesDesWorkflows(workflows);
  const conventions = lire("CONVENTIONS.md");
  const sPortes = section(conventions, "## Portes de CI");
  if (!sPortes) ecart("CONVENTIONS.md", 1, "section « ## Portes de CI » introuvable");
  else {
    const { compte, portes } = portesDuTableau(sPortes);
    if (!memes(portes, controles)) ecart("CONVENTIONS.md", sPortes.debut, `le tableau des portes nomme ${liste(portes)} ; les workflows lancés sur une PR posent ${liste(controles)}`);
    if (compte !== controles.length) ecart("CONVENTIONS.md", sPortes.debut, `« **N, aujourd'hui.** » doit compter ${pluriel(controles.length, "porte", "portes")}, en lettres (lu : ${compte ?? "rien"})`);
  }
  const claude = lire("CLAUDE.md");
  const jobsCi = jobsDuWorkflow(workflows.get(`${dossierWf}/ci.yml`)).map((j) => j.id);
  const jc = jobsDeClaude(claude);
  if (!jc) ecart("CLAUDE.md", 1, "phrase « a pour jobs … » introuvable");
  else if (!memes(jc.jobs, jobsCi)) ecart("CLAUDE.md", jc.ligne, `jobs ${liste(jc.jobs)} ; ci.yml a ${liste(jobsCi)}`);

  // 5. Les contrôles requis : à un seul endroit, puis justes.
  const sProtection = section(conventions, "## Protection de `main`");
  for (const [f, t] of textes) {
    const lignes = t.split("\n");
    const sansSource = f === "CONVENTIONS.md" && sProtection
      ? lignes.map((l, i) => (i >= sProtection.debut && i < sProtection.fin ? "" : l)).join("\n")
      : t;
    for (const p of phrasesSurLesRequis(sansSource, controles)) {
      const quoi = p.portes.length ? liste(p.portes) : "des contrôles de la règle de protection";
      ecart(f, p.ligne, `une phrase dit ${quoi} requis, exigés ou bloquants : les contrôles requis se disent à un seul endroit, CONVENTIONS.md « Protection de \`main\` » ; ici, y renvoyer`);
    }
  }
  let requis = null;
  if (!sProtection) ecart("CONVENTIONS.md", 1, "section « ## Protection de `main` » introuvable");
  else {
    const r = controlesRequisEcrits(sProtection);
    const ou = sProtection.debut;
    if (!r.requis) ecart("CONVENTIONS.md", ou, "phrase « requiert N contrôles : `a`, `b` et `c`. » introuvable");
    else {
      requis = r.requis;
      if (r.compte !== r.requis.length) ecart("CONVENTIONS.md", ou, `« requiert N contrôles » : ${pluriel(r.requis.length, "est nommé", "sont nommés")} (lu : ${r.compte ?? "rien"})`);
      const inconnus = r.requis.filter((c) => !controles.includes(c));
      if (inconnus.length) ecart("CONVENTIONS.md", ou, `contrôles requis qui ne sont pas des portes : ${liste(inconnus)}`);
      const autres = controles.filter((c) => !r.requis.includes(c));
      if (autres.length && (!r.nonRequis || !memes(r.nonRequis, autres))) {
        ecart("CONVENTIONS.md", ou, `« … n'y sont pas requis » doit nommer ${liste(autres)} (lu : ${r.nonRequis ? liste(r.nonRequis) : "rien"})`);
      }
      if (!autres.length && r.nonRequis?.length) ecart("CONVENTIONS.md", ou, "toutes les portes sont requises : la phrase « … n'y sont pas requis » n'a plus lieu d'être");
    }
  }

  // 6. Les crates.
  const membres = membresDuCargo(lire("Cargo.toml"));
  const crates = membres.map((chemin) => {
    const toml = join(chemin, "Cargo.toml");
    if (!existsSync(toml)) throw new Error(`Cargo.toml : le membre « ${chemin} » n'a pas de Cargo.toml (un motif comme crates/* n'est pas lu : nommer chaque membre)`);
    return { chemin, nom: nomDuPaquet(lire(toml)) };
  });
  const nomsCrates = crates.map((c) => c.nom);
  const cc = cratesDeClaude(claude);
  if (!cc) ecart("CLAUDE.md", 1, "liste « - Les crates : » introuvable");
  else {
    const ecrit = cc.crates.map((c) => `${c.nom} (${c.chemin})`);
    const reel = crates.map((c) => `${c.nom} (${c.chemin})`);
    if (!memes(ecrit, reel)) ecart("CLAUDE.md", cc.ligne, `crates ${liste(ecrit)} ; l'espace de travail a ${liste(reel)}`);
  }

  // 7. Les comptes écrits devant un nom, dans les trois fichiers d'état ; ceux des ADR et des
  // décisions d'architecture sont lus au point 2.
  const totaux = {
    portes: controles.length, jobs: jobsCi.length, modules: modules.length, crates: crates.length,
    situations: derniers.S,
  };
  for (const f of ETAT) {
    for (const c of comptesNommes(lire(f), Object.keys(totaux))) {
      if (c.valeur !== totaux[c.nom]) ecart(f, c.ligne, `« ${c.ecrit} ${c.nom} » : il y en a ${totaux[c.nom]} ; un compte partiel ne s'écrit pas ainsi`);
    }
  }

  // 8. Les critères livrés : la SFD, le tableau du § 70 · 4.1 et les tests Rust qui les nomment.
  const criteres = new Map();
  const fichierDe = new Map();
  for (const [m, texte] of textesModules) {
    for (const [id, cs] of criteresDuModule(texte, m.code)) {
      criteres.set(id, cs);
      fichierDe.set(id, `docs/20-sfd/${m.fichier}`);
    }
  }
  const qualite = "docs/70-qualite-et-verification.md";
  const sLivres = section(lire(qualite), "### 4.1 Les critères livrés");
  const livres = sLivres ? lireCriteresLivres(sLivres) : null;
  const sources = fichiers("*.rs");
  const codes = modules.map((m) => m.code);
  const tests = sources.flatMap((f) => fonctionsNommeesCritere(f, lire(f), codes));
  ecarts.push(...verifierCriteres({ criteres, fichierDe, livres, fichierListe: qualite, tests }));
  const nLivres = (livres ?? []).reduce((n, l) => n + (l.criteres?.length ?? 0), 0);

  // En dernier, l'appel réseau : la règle de protection réelle.
  if (requis) {
    const depot = process.env.GITHUB_REPOSITORY ?? "EdouardZemb/pupitre";
    try {
      const reels = await controlesRequisSurGitHub(depot, "main");
      if (!memes(requis, reels)) ecart("CONVENTIONS.md", sProtection.debut, `la règle de protection de ${depot} requiert ${liste(reels) || "rien"} ; le paragraphe dit ${liste(requis)}`);
    } catch (err) {
      const message = `règle de protection de ${depot} illisible sur GitHub (${err.message})`;
      if (process.env.GITHUB_ACTIONS === "true") ecart("GitHub", 0, `${message} : ce n'est pas un écart du dépôt, relancer le job`);
      else avertissements.push(`${message} : contrôles requis non comparés, hors CI`);
    }
  }

  for (const a of avertissements) console.log(`AVERTIT — ${a}`);
  for (const e of ecarts) console.log(`REFUSE — ${e.fichier}:${e.ligne} — ${e.message}`);
  console.log(`Faits recopiés : ${textes.size} fichiers Markdown, ${adrs.length} ADR, ${fonctions.length} fonctions, ${controles.length} portes, ${crates.length} crates, ${pluriel(nLivres, "critère livré", "critères livrés")} ; ${pluriel(ecarts.length, "écart", "écarts")}.`);
  process.exitCode = ecarts.length ? 1 : 0;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await principal();
  } catch (err) {
    console.log(`REFUSE — ${err.message}`);
    process.exitCode = 1;
  }
}
