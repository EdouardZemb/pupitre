// Tests de scripts/verif-faits.mjs, sur des textes écrits ici : chaque règle refuse l'écart
// qu'elle vise et laisse passer ce qui est juste.
//
// Usage : node --test scripts/verif-faits.test.mjs

import assert from "node:assert/strict";
import { test } from "node:test";
import {
  comptesAdr,
  comptesNommes,
  controlesDesWorkflows,
  controlesRequisEcrits,
  cratesDeClaude,
  criteresDuModule,
  fonctionsDuModule,
  fonctionsNommeesCritere,
  jobsDeClaude,
  jobsDuWorkflow,
  lanceSurPr,
  lireAdr,
  lireCriteresLivres,
  lireGraphe,
  lireIndexAdr,
  lireIndexSfd,
  lireMatrice,
  lireModulesSfd,
  membresDuCargo,
  nomDuPaquet,
  nombreEnLettres,
  numeros,
  paragraphes,
  phrasesSurLesRequis,
  plages,
  portesDuTableau,
  section,
  trous,
  verifierAdr,
  verifierCriteres,
} from "./verif-faits.mjs";

test("numeros relève chaque numéro, et ceux qui reviennent", () => {
  assert.deepEqual(numeros("| A02 | b |\n| A01 | a |\n", /^\| A(\d\d) \|/gm), { nums: [1, 2], doubles: [] });
  assert.deepEqual(numeros("| A23 | a |\n| A23 | b |\n", /^\| A(\d\d) \|/gm), { nums: [23], doubles: [23] });
});

test("trous dit les numéros absents depuis le premier", () => {
  assert.deepEqual(trous([1, 2, 3]), []);
  assert.deepEqual(trous([1, 3, 5]), [2, 4]);
  assert.deepEqual(trous([]), []);
  assert.deepEqual(trous([0, 2], 0), [1]);
  assert.deepEqual(trous([1, 2], 0), [0]);
});

test("plages relève les plages qui partent du premier numéro, sous toutes leurs graphies", () => {
  const texte = [
    "Invariants I01–I27, bancs",
    "| I01 … I27 | **Invariant** |",
    "| 4 | Invariants I01 à I27 |",
    "de I01-26 ici",
    "Q07 à Q16 ne part pas de 01",
    "DG-I01 à I09 n'est pas un invariant",
  ].join("\n");
  assert.deepEqual(plages(texte, "I"), [
    { ligne: 1, fin: 27 }, { ligne: 2, fin: 27 }, { ligne: 3, fin: 27 }, { ligne: 4, fin: 26 },
  ]);
  assert.deepEqual(plages(texte, "Q"), []);
});

test("plages lit les familles à un chiffre, à tiret et à zéro, sans les confondre", () => {
  const texte = "P1–P8 ; IR-1 … IR-9 ; INC-0 à INC-6 ; R1 … R7 ; DG-P1 à P8 ; INC-1 à INC-3";
  assert.deepEqual(plages(texte, "P", "1"), [{ ligne: 1, fin: 8 }]);
  assert.deepEqual(plages(texte, "IR-", "1"), [{ ligne: 1, fin: 9 }]);
  assert.deepEqual(plages(texte, "INC-", "0"), [{ ligne: 1, fin: 6 }]);
  assert.deepEqual(plages(texte, "R", "1"), [{ ligne: 1, fin: 7 }]);
});

test("nombreEnLettres lit un nombre en toutes lettres, composé ou non", () => {
  assert.equal(nombreEnLettres("Sept"), 7);
  assert.equal(nombreEnLettres("vingt et une"), 21);
  assert.equal(nombreEnLettres("vingt-deux"), 22);
  assert.equal(nombreEnLettres("dix-huit"), 18);
  assert.equal(nombreEnLettres("beaucoup"), null);
  assert.equal(nombreEnLettres("et"), null);
});

test("paragraphes joint les lignes coupées, isole tableaux, titres et puces, et situe chaque ligne", () => {
  const p = paragraphes("Un texte\ncoupé en deux.\n\n| a | b |\n| c | d |\n- une puce\n  qui continue\n## Titre\n```\ncode\n```\nfin");
  assert.deepEqual(p.map((x) => [x.ligne, x.texte, x.tableau]), [
    [1, "Un texte coupé en deux.", false],
    [4, "| a | b |", true],
    [5, "| c | d |", true],
    [6, "- une puce qui continue", false],
    [8, "## Titre", false],
    [12, "fin", false],
  ]);
});

test("paragraphes prolonge une citation sur plusieurs lignes, marque ôtée", () => {
  const p = paragraphes("> La série compte 22\n> ADR, dont 10 acceptées.\nAprès la citation.");
  assert.deepEqual(p.map((x) => [x.ligne, x.texte]), [[1, "La série compte 22 ADR, dont 10 acceptées."], [3, "Après la citation."]]);
});

const ADR_3 = [
  "# ADR-003 — Des documents CRDT pour travailler",
  "",
  "| | |",
  "|---|---|",
  "| **Statut** | 🟡 proposée |",
  "| **Date** | 2026-10-11 |",
  "| **Sert** | B13, B14 · principes P5, P8 · contre le risque R5 |",
  "| **S'appuie sur** | [ADR-002](adr-002-local-first.md) et [ADR-001](adr-001-un.md) |",
].join("\n");

test("lireAdr lit le numéro, le titre, le statut, les principes et les appuis", () => {
  assert.deepEqual(lireAdr("adr-003-crdt.md", ADR_3), {
    num: 3, titre: "Des documents CRDT pour travailler", statut: "proposée", remplacante: null,
    principes: [5, 8], appuis: [1, 2],
  });
  const remplacee = ADR_3.replace("🟡 proposée", "🔄 remplacée par [ADR-019](adr-019-x.md)");
  assert.equal(lireAdr("adr-003-crdt.md", remplacee).statut, "remplacée");
  assert.equal(lireAdr("adr-003-crdt.md", remplacee).remplacante, 19);
  const acceptee = ADR_3.replace("🟡 proposée", "✅ acceptée — confirmée le 2026-10-28");
  assert.equal(lireAdr("adr-003-crdt.md", acceptee).statut, "acceptée");
});

const sansParagraphe = (cs) => cs.map(({ paragraphe, ecrit, ...c }) => c);

test("comptesAdr relève les comptes en chiffres, et les statuts d'un paragraphe qui parle d'ADR", () => {
  const texte = [
    "et 21 ADR dans `docs/adr/` (10 acceptées, 8 proposées, 3 remplacées).",
    "",
    "| **Décisions** | 21 ADR, dont 10 acceptées, 8 proposées et 1 remplacée |",
    "",
    "103 notes proposées, sans rapport",
  ].join("\n");
  assert.deepEqual(sansParagraphe(comptesAdr(texte)), [
    { ligne: 1, quoi: "total", valeur: 21 },
    { ligne: 1, quoi: "acceptée", valeur: 10 },
    { ligne: 1, quoi: "proposée", valeur: 8 },
    { ligne: 1, quoi: "remplacée", valeur: 3 },
    { ligne: 3, quoi: "total", valeur: 21 },
    { ligne: 3, quoi: "acceptée", valeur: 10 },
    { ligne: 3, quoi: "proposée", valeur: 8 },
    { ligne: 3, quoi: "remplacée", valeur: 1 },
  ]);
});

test("comptesAdr ne prend pas le numéro d'un identifiant pour un compte", () => {
  assert.deepEqual(comptesAdr("La conception a vu les ADR-003 et ADR-006 acceptées, et l'ADR-016 remplacée."), []);
  assert.deepEqual(comptesAdr("voir ADR-021 et l'ADR-019"), []);
  assert.deepEqual(comptesAdr("Les ADR-019 et 020 acceptées.", { lettres: true }), []);
});

test("comptesAdr lit les comptes en lettres dans un fichier d'état, et « décisions d'architecture »", () => {
  const texte = "Vingt et une ADR, dont onze acceptées et huit proposées.\n\nvingt et une décisions d’architecture, dont 11 acceptées.";
  assert.deepEqual(sansParagraphe(comptesAdr(texte, { lettres: true })), [
    { ligne: 1, quoi: "total", valeur: 21 },
    { ligne: 1, quoi: "acceptée", valeur: 11 },
    { ligne: 1, quoi: "proposée", valeur: 8 },
    { ligne: 3, quoi: "total", valeur: 21 },
    { ligne: 3, quoi: "acceptée", valeur: 11 },
  ]);
  assert.deepEqual(comptesAdr(texte), []);
});

test("comptesAdr lit un compte coupé par le retour à la ligne, et ignore les blocs de code", () => {
  assert.deepEqual(sansParagraphe(comptesAdr("La conception compte 21 ADR, dont 10 acceptées,\n8 proposées et 3 remplacées.")), [
    { ligne: 1, quoi: "total", valeur: 21 },
    { ligne: 1, quoi: "acceptée", valeur: 10 },
    { ligne: 2, quoi: "proposée", valeur: 8 },
    { ligne: 2, quoi: "remplacée", valeur: 3 },
  ]);
  assert.deepEqual(sansParagraphe(comptesAdr("compte maintenant 21\nADR en tout.")), [{ ligne: 1, quoi: "total", valeur: 21 }]);
  assert.deepEqual(comptesAdr("```\n3 ADR\n```\n"), []);
});

const INDEX = [
  "## La série",
  "",
  "| N° | Décision | Statut | Principe |",
  "|---|---|---|---|",
  "| [ADR-001](adr-001-un.md) | Un seul matériau | ✅ acceptée | P1 |",
  "| [ADR-002](adr-002-deux.md) | Local-first | 🔄 remplacée par [ADR-003](adr-003-trois.md) | P5 |",
  "| [ADR-003](adr-003-trois.md) | Des documents CRDT | 🟡 proposée | P5, P8 |",
  "",
  "```mermaid",
  "flowchart LR",
  "  classDef remplacee stroke-dasharray:5 3",
  '  A1["ADR-001<br/>un seul"] --> A2["ADR-002 · remplacée<br/>local"]:::remplacee',
  '  A2 --> A3["ADR-003<br/>CRDT"]',
  "```",
].join("\n");

test("lireIndexAdr lit chaque ligne de la série", () => {
  const index = lireIndexAdr(INDEX);
  assert.equal(index.size, 3);
  assert.deepEqual(index.get(2), { ligne: 6, titre: "Local-first", statut: "remplacée", remplacante: 3, principes: [5] });
  assert.deepEqual(index.get(3).principes, [5, 8]);
});

test("lireGraphe lit les nœuds, leur classe et les arêtes", () => {
  const g = lireGraphe(INDEX);
  assert.deepEqual([...g.noeuds.keys()].sort(), [1, 2, 3]);
  assert.equal(g.noeuds.get(2).classe, "remplacee");
  assert.equal(g.noeuds.get(1).classe, null);
  assert.deepEqual([...g.aretes].sort(), ["1>2", "2>3"]);
  assert.deepEqual(g.illisibles, []);
});

test("lireGraphe lit les flèches chaînées, allongées, étiquetées et groupées", () => {
  const texte = "```mermaid\nflowchart LR\n  A19 --> A22 --> A20\n  A2 --->|amendée par| A23[\"ADR-023\"]\n  A4 & A5 --> A6\n```";
  const g = lireGraphe(texte);
  assert.deepEqual([...g.aretes].sort(), ["19>22", "22>20", "2>23", "4>6", "5>6"]);
  assert.deepEqual(g.illisibles, []);
  for (const lien of ["A7 -.-> A8", "A9 ==> A20", "A9 --- A20", "A9 --o A20"]) {
    assert.equal(lireGraphe("```mermaid\n  " + lien + "\n```").illisibles.length, 1, lien);
  }
  assert.deepEqual(lireGraphe("```mermaid\n  A1[sans guillemets] --> A2\n```").illisibles.length, 2);
});

function adr(num, titre, statut, principes, appuis, remplacante = null) {
  return { num, titre, statut, remplacante, principes, appuis };
}
const TROIS = [
  adr(1, "Un seul matériau", "acceptée", [1], []),
  adr(2, "Local-first", "remplacée", [5], [1], 3),
  adr(3, "Des documents CRDT", "proposée", [5, 8], [2]),
];

test("verifierAdr accepte un index et un graphe fidèles", () => {
  assert.deepEqual(verifierAdr(TROIS, INDEX), []);
});

test("verifierAdr refuse un statut, un titre, des principes ou une dépendance qui divergent", () => {
  const adrs = [TROIS[0], TROIS[1], adr(3, "Des documents CRDT, autrement", "acceptée", [5], [1])];
  const ecarts = verifierAdr(adrs, INDEX).map((e) => e.message);
  for (const attendu of [/ADR-003.*statut/, /ADR-003.*titre/, /ADR-003.*principes/, /arête/]) {
    assert.ok(ecarts.some((m) => attendu.test(m)), ecarts.join("\n"));
  }
});

test("verifierAdr refuse une ADR absente de l'index ou du graphe", () => {
  const ecarts = verifierAdr([...TROIS, adr(4, "Nouvelle", "proposée", [2], [])], INDEX).map((e) => e.message);
  assert.ok(ecarts.some((m) => /ADR-004.*index/.test(m)), ecarts.join("\n"));
  assert.ok(ecarts.some((m) => /ADR-004.*graphe/.test(m)), ecarts.join("\n"));
});

const SFD = [
  "| Module | Fichier | Fonctions |",
  "|---|---|---|",
  "| **CAP** · Capter et organiser | [21-capter.md](21-capter.md) | 2 |",
  "",
  "## Les fonctions",
  "",
  "| Fonction | Échelle · moment |",
  "|---|---|",
  "| [CAP-01](21-capter.md#cap-01) Capter en un geste | É0 · M1 |",
  "| [CAP-02](21-capter.md#cap-02) Trier la boîte | É0 · M1 |",
  "",
  "## Matrice de traçabilité",
  "",
  "| Besoin | Fonctions |",
  "|---|---|",
  "| **B01** Capter partout | CAP-01 · CAP-02 |",
  "| **B02** Retrouver | CAP-02 |",
].join("\n");

test("lireModulesSfd, lireIndexSfd et lireMatrice lisent les tableaux de la SFD", () => {
  assert.deepEqual(lireModulesSfd(SFD), [{ ligne: 3, code: "CAP", fichier: "21-capter.md", compte: 2 }]);
  assert.deepEqual(lireIndexSfd(SFD), [
    { ligne: 9, id: "CAP-01", titre: "Capter en un geste" },
    { ligne: 10, id: "CAP-02", titre: "Trier la boîte" },
  ]);
  assert.deepEqual(lireMatrice(SFD), [
    { ligne: 16, besoin: 1, fonctions: ["CAP-01", "CAP-02"] },
    { ligne: 17, besoin: 2, fonctions: ["CAP-02"] },
  ]);
});

test("fonctionsDuModule compte chaque titre, doublon compris", () => {
  const texte = "### CAP-01 · Capter\n\n### CAP-02 · Trier\n### CAP-02 · Trier encore\n#### CAP-03 · pas un titre de fonction";
  assert.deepEqual(fonctionsDuModule(texte, "CAP").map((f) => f.id), ["CAP-01", "CAP-02", "CAP-02"]);
});

const MODULE = [
  "### CAP-01 · Capter",
  "",
  "**Règles**",
  "1. Une règle numérotée n'est pas un critère.",
  "",
  "**Critères d'acceptation**",
  "1. *Quand* Sacha capte, *alors* une idée naît.",
  "   Une ligne de suite, en retrait.",
  "   1. Une liste en retrait n'est pas un critère.",
  "2. ~~*Étant donné* un fonds vide, *alors* rien.~~ Retiré : remplacé par le critère 3.",
  "```",
  "3. dans un bloc de code",
  "```",
  "3. *Étant donné* un fonds vide, *quand* Sacha capte, *alors* il compte une idée.",
  "",
  "---",
  "",
  "4. Après le filet, plus rien n'est un critère.",
  "### CAP-02 · Trier",
  "**Critères d’acceptation**",
  "1. Un critère.",
  "1. Le même numéro.",
  "3. Un trou avant lui.",
  "**Cas limites**",
  "4. Après le gras, plus rien.",
  "### CAP-03 · Chercher",
  "**Critères d'acceptation**",
  "1. Un critère.",
  "#### Un titre plus bas",
  "2. Après un titre, plus rien.",
  "## Une autre section",
  "5. Hors de toute fonction.",
].join("\n");

test("criteresDuModule lit les critères de chaque fonction, leurs retraits, et rien d'autre", () => {
  const r = criteresDuModule(MODULE, "CAP");
  assert.deepEqual([...r.keys()], ["CAP-01", "CAP-02", "CAP-03"]);
  assert.deepEqual(r.get("CAP-01"), [
    { n: 1, ligne: 7, retire: false }, { n: 2, ligne: 10, retire: true }, { n: 3, ligne: 14, retire: false },
  ]);
  assert.deepEqual(r.get("CAP-02").map((c) => c.n), [1, 1, 3]);
  assert.deepEqual(r.get("CAP-03").map((c) => c.n), [1]);
  assert.deepEqual(criteresDuModule("### CAP-04 · Sans critères\n\nTexte.", "CAP").get("CAP-04"), []);
});

const LIVRES = [
  "### 4.1 Les critères livrés",
  "",
  "Un texte, puis le tableau.",
  "",
  "| Fonction | Critères | Livrés par |",
  "|:---|:---:|---:|",
  "| CAP-06 | 1, 4, 7 à 10 | INC-1, lot 1 |",
  "| [SYN-07](20-sfd/29-socle.md#syn-07) | 6 | INC-1, lot 1 |",
  "| CAP06 | 1 | coquille |",
  "| CAP-01 | 1-3 | tiret |",
  "| CAP-02 | 4 à 2 | à l'envers |",
  "| CAP-03 | 0 | zéro |",
  "| CAP-05 | 7 à 7 | plage d'un seul |",
  "CAP-07 | 2 | sans barre initiale, la ligne compte |",
  "",
  "| CAP-04 | 1 | après la ligne vide, hors du tableau |",
  "## 5. La suite",
].join("\n");

test("lireCriteresLivres lit chaque ligne du tableau jusqu'à la ligne vide, et dit pourquoi une ligne ne se lit pas", () => {
  const r = lireCriteresLivres(section(LIVRES, "### 4.1 Les critères livrés"));
  assert.deepEqual(r.filter((l) => !l.illisible), [
    { ligne: 7, id: "CAP-06", criteres: [1, 4, 7, 8, 9, 10] },
    { ligne: 8, id: "SYN-07", criteres: [6] },
    { ligne: 14, id: "CAP-07", criteres: [2] },
  ]);
  assert.deepEqual(r.filter((l) => l.illisible).map((l) => [l.ligne, l.id]), [
    [9, undefined], [10, "CAP-01"], [11, "CAP-02"], [12, "CAP-03"], [13, "CAP-05"],
  ]);
  assert.equal(r.length, 8);
});

test("lireCriteresLivres rend une liste vide sous un tableau vide, et null sans tableau", () => {
  const vide = "### 4.1 Les critères livrés\n\n| Fonction | Critères | Livrés par |\n|---|---|---|\n\nAucun encore.";
  assert.deepEqual(lireCriteresLivres(section(vide, "### 4.1 Les critères livrés")), []);
  assert.equal(lireCriteresLivres(section("### 4.1 Les critères livrés\n\nRien.", "### 4.1 Les critères livrés")), null);
});

const SOURCE_RUST = `#[test]
fn cap06_7_la_reponse_declare_son_assiette() {}

/// Un commentaire de documentation.
#[test]
#[ignore = "lent"]
fn cap06_8_ignore() {}

fn cap06_9_une_aide() {}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]
    #[test]
    fn syn07_6_par_proprietes(a in 0..3u8) {}
}

#[tokio::test] async fn eco01_8() {}

// fn cap06_10_en_commentaire() {}
fn cap06_7x() {}
fn autre_cap06_7() {}
#[test]
pub fn cap06_12_generique<T>() {}
#[cfg(test)]
fn cap06_11_aide_de_test() {}
#[test]
#[should_panic(
    expected = "la racine des données est tenue par un autre processus : fermer l'autre Pupitre d'abord"
)]
fn cap06_13_attribut_sur_plusieurs_lignes() {}
#[test]
// Un commentaire entre l'attribut et la fonction.

fn cap06_14_commentaire_et_ligne_vide() {}
#[test]
#[cfg_attr(windows, ignore = "lent sous Windows")]
fn cap06_15_ignore_sous_windows() {}
#[ignore]
#[cfg_attr(
    windows,
    allow(unused_variables)
)]
#[test]
fn cap06_16_ignore_au_dessus_d_un_attribut_coupe() {}
#[test]
fn cap06_17_la_réponse_accentuée() {}
#[test]
fn cap_06_7_mauvaise_forme() {}
#[ignore] // lent : à lancer à la main
#[test]
fn cap06_18_ignore_commente() {}
#[test] // H07 : sous Windows aussi
fn cap06_19_test_commente() {}
#[test]
/* un commentaire de bloc */
fn cap06_20_bloc() {}
#[should_panic(expected = "voir http://exemple")]
#[test]
fn cap06_21_double_barre_dans_une_chaine() {}
`;

test("fonctionsNommeesCritere relève les fonctions nommées d'après un critère, et leurs attributs", () => {
  const r = fonctionsNommeesCritere("t.rs", SOURCE_RUST, ["CAP", "SYN", "ECO"]);
  assert.deepEqual(r.map((f) => [f.ligne, f.nom, f.id, f.n, f.forme, f.test, f.ignore]), [
    [2, "cap06_7_la_reponse_declare_son_assiette", "CAP-06", 7, true, true, false],
    [7, "cap06_8_ignore", "CAP-06", 8, true, true, true],
    [9, "cap06_9_une_aide", "CAP-06", 9, true, false, false],
    [14, "syn07_6_par_proprietes", "SYN-07", 6, true, true, false],
    [17, "eco01_8", "ECO-01", 8, true, true, false],
    [23, "cap06_12_generique", "CAP-06", 12, true, true, false],
    [25, "cap06_11_aide_de_test", "CAP-06", 11, true, false, false],
    [30, "cap06_13_attribut_sur_plusieurs_lignes", "CAP-06", 13, true, true, false],
    [34, "cap06_14_commentaire_et_ligne_vide", "CAP-06", 14, true, true, false],
    [37, "cap06_15_ignore_sous_windows", "CAP-06", 15, true, true, true],
    [44, "cap06_16_ignore_au_dessus_d_un_attribut_coupe", "CAP-06", 16, true, true, true],
    [46, "cap06_17_la_réponse_accentuée", "CAP-06", 17, true, true, false],
    [48, "cap_06_7_mauvaise_forme", "CAP-06", 7, false, true, false],
    [51, "cap06_18_ignore_commente", "CAP-06", 18, true, true, true],
    [53, "cap06_19_test_commente", "CAP-06", 19, true, true, false],
    [56, "cap06_20_bloc", "CAP-06", 20, true, true, false],
    [59, "cap06_21_double_barre_dans_une_chaine", "CAP-06", 21, true, true, false],
  ]);
  assert.ok(r.every((f) => f.fichier === "t.rs"));
  assert.deepEqual(fonctionsNommeesCritere("t.rs", "#[test]\r\nfn cap06_7_crlf() {}\r\n", ["CAP"]).map((f) => [f.ligne, f.test]), [[2, true]]);
});

// Une SFD de deux fonctions : CAP-06 a les critères 1 à 3, SYN-07 les critères 1 et 2, le 2 retiré.
const sfd = () => ({
  criteres: new Map([
    ["CAP-06", [{ n: 1, ligne: 10, retire: false }, { n: 2, ligne: 11, retire: false }, { n: 3, ligne: 12, retire: false }]],
    ["SYN-07", [{ n: 1, ligne: 40, retire: false }, { n: 2, ligne: 41, retire: true }]],
  ]),
  fichierDe: new Map([["CAP-06", "docs/20-sfd/21.md"], ["SYN-07", "docs/20-sfd/29.md"]]),
  fichierListe: "docs/70.md",
});
const nomme = (id, n, autres = {}) => ({
  fichier: "t.rs", ligne: 1, nom: `${id.toLowerCase().replace("-", "")}_${n}_x`, id, n, forme: true, test: true, ignore: false, ...autres,
});

test("verifierCriteres accepte des critères inscrits, chacun nommé par un test qui tourne", () => {
  const livres = [{ ligne: 7, id: "CAP-06", criteres: [1, 3] }, { ligne: 8, id: "SYN-07", criteres: [1] }];
  const tests = [
    nomme("CAP-06", 1), nomme("CAP-06", 3), nomme("CAP-06", 3, { ignore: true }), nomme("SYN-07", 1),
    nomme("CAP-06", 2, { test: false }),
  ];
  assert.deepEqual(verifierCriteres({ ...sfd(), livres, tests }), []);
  assert.deepEqual(verifierCriteres({ ...sfd(), livres: [], tests: [] }), []);
});

test("verifierCriteres refuse un critère livré sans test qui tourne, ou que la SFD n'a pas", () => {
  const livres = [
    { ligne: 7, id: "CAP-06", criteres: [1, 2, 3, 4, 2] },
    { ligne: 8, id: "MIC-99", criteres: [1] },
    { ligne: 9, id: "CAP-06", criteres: [] },
    { ligne: 10, illisible: "la première colonne nomme une fonction" },
    { ligne: 11, id: "SYN-07", criteres: [1, 2] },
  ];
  const tests = [
    nomme("CAP-06", 1), nomme("CAP-06", 2, { ignore: true }), nomme("CAP-06", 4),
    nomme("CAP-06", 3, { forme: false, nom: "cap_06_3_x" }), nomme("SYN-07", 1, { test: false }),
  ];
  const e = verifierCriteres({ ...sfd(), livres, tests });
  const dit = (motif) => e.filter((x) => motif.test(x.message)).length;
  assert.equal(dit(/^CAP-06 · 2 est inscrit, mais t\.rs:1 le nomme sans être un test qui tourne/), 1);
  assert.equal(dit(/^CAP-06 · 3 est inscrit, sans test qui le nomme : une fonction « fn cap06_3_… »/), 1);
  assert.equal(dit(/^SYN-07 · 1 est inscrit, mais t\.rs:1 le nomme sans être un test qui tourne/), 1);
  assert.equal(dit(/^SYN-07 · 2 est retiré \(docs\/20-sfd\/29\.md:41\) : il ne s'inscrit pas$/), 1);
  assert.equal(dit(/^CAP-06 · 4 : la SFD n'a pas ce critère/), 1);
  assert.equal(dit(/^CAP-06 · 2 inscrit deux fois/), 1);
  assert.equal(dit(/^MIC-99 n'est pas une fonction de la SFD/), 1);
  assert.equal(dit(/^CAP-06 : plusieurs lignes ; une ligne par fonction/), 1);
  assert.equal(dit(/^ligne du tableau illisible/), 1);
  assert.equal(dit(/^fn cap06_4_x nomme CAP-06 · 4, que la SFD n'a pas/), 1);
  assert.equal(dit(/^fn cap_06_3_x : un nom de test d'après un critère s'écrit « cap06_3_… »/), 1);
  assert.equal(e.length, 11);
  assert.ok(e.every((x) => x.fichier === (x.message.startsWith("fn ") ? "t.rs" : "docs/70.md")));
});

test("verifierCriteres refuse un tableau introuvable, des critères mal numérotés, et un test qui nomme ce qui n'existe pas", () => {
  const base = sfd();
  base.criteres.set("CAP-06", [{ n: 1, ligne: 10 }, { n: 1, ligne: 11 }, { n: 4, ligne: 12 }]);
  base.criteres.set("ECO-01", [{ n: 2, ligne: 70 }]);
  base.fichierDe.set("ECO-01", "docs/20-sfd/27.md");
  base.criteres.set("ECO-02", []);
  base.fichierDe.set("ECO-02", "docs/20-sfd/27.md");
  const tests = [nomme("MIC-99", 1), nomme("SYN-07", 3, { test: false }), nomme("SYN-07", 2)];
  const e = verifierCriteres({ ...base, livres: null, tests });
  assert.deepEqual(e.map((x) => [x.fichier, x.ligne]), [
    ["docs/20-sfd/21.md", 11], ["docs/20-sfd/21.md", 10], ["docs/20-sfd/27.md", 70], ["docs/20-sfd/27.md", 1],
    ["docs/70.md", 1], ["t.rs", 1], ["t.rs", 1], ["t.rs", 1],
  ]);
  assert.match(e[0].message, /^CAP-06 · 1 écrit deux fois/);
  assert.match(e[1].message, /^CAP-06 : critères absents, 2, 3 ; un critère retiré garde son numéro/);
  assert.match(e[2].message, /^ECO-01 : critères absents, 1 ;/);
  assert.match(e[3].message, /^ECO-02 : aucun critère lu sous « \*\*Critères d'acceptation\*\* »/);
  assert.match(e[4].message, /^tableau « \| Fonction \| Critères \| Livrés par \| » introuvable/);
  assert.match(e[5].message, /^fn mic99_1_x nomme MIC-99, qui n'est pas une fonction de la SFD$/);
  assert.match(e[6].message, /^fn syn07_3_x nomme SYN-07 · 3, que la SFD n'a pas/);
  assert.match(e[7].message, /^fn syn07_2_x nomme SYN-07 · 2, qui est retiré \(docs\/20-sfd\/29\.md:41\)$/);
});

const CI = `name: ci

on:
  pull_request:
    types: [opened]
  push:
    branches: [main]

jobs:
  socle:
    uses: ./.github/workflows/modele-socle.yml
    secrets: inherit

  # Un commentaire entre deux jobs.
  contenu:
    runs-on: ubuntu-latest
    steps:
      - name: Vérification
        run: bash x.sh
  verifier:
    name: verifier
    runs-on: ubuntu-latest
`;

const SOCLE = `on:
  workflow_call:

jobs:
  secrets:
    name: secrets
    runs-on: ubuntu-latest
  confidentialite:
    runs-on: ubuntu-latest
`;

test("jobsDuWorkflow lit les jobs, leur nom, le workflow qu'ils appellent et leur matrice", () => {
  assert.deepEqual(jobsDuWorkflow(CI), [
    { id: "socle", nom: null, appelle: ".github/workflows/modele-socle.yml", matrice: false },
    { id: "contenu", nom: null, appelle: null, matrice: false },
    { id: "verifier", nom: "verifier", appelle: null, matrice: false },
  ]);
  assert.equal(jobsDuWorkflow("jobs: # un seul\n  seul:\n    runs-on: x\n").length, 1);
  assert.equal(jobsDuWorkflow("jobs:\n  m:\n    strategy:\n      matrix:\n        os: [a, b]\n")[0].matrice, true);
});

test("lanceSurPr reconnaît un workflow lancé sur une PR", () => {
  assert.equal(lanceSurPr(CI), true);
  assert.equal(lanceSurPr(SOCLE), false);
  assert.equal(lanceSurPr("on: [push, pull_request]\njobs:\n"), true);
  assert.equal(lanceSurPr("on:\n  schedule:\n    - cron: '0 5 * * 1'\njobs:\n"), false);
});

test("controlesDesWorkflows nomme les contrôles comme GitHub, appels compris", () => {
  const fichiers = new Map([
    [".github/workflows/ci.yml", CI],
    [".github/workflows/modele-socle.yml", SOCLE],
    [".github/workflows/hebdo.yml", "on:\n  schedule:\n    - cron: x\njobs:\n  long:\n    runs-on: x\n"],
  ]);
  assert.deepEqual(controlesDesWorkflows(fichiers).sort(), ["contenu", "socle / confidentialite", "socle / secrets", "verifier"]);
});

test("controlesDesWorkflows refuse ce qu'il ne sait pas nommer", () => {
  const matrice = new Map([["w.yml", "on: pull_request\njobs:\n  m:\n    strategy:\n      matrix:\n        os: [a]\n"]]);
  assert.throws(() => controlesDesWorkflows(matrice), /matrice/);
  const vide = new Map([["w.yml", "on: pull_request\njobs:\n"]]);
  assert.throws(() => controlesDesWorkflows(vide), /aucun job/);
});

const CONVENTIONS = `# Conventions

## Portes de CI

**Quatre, aujourd'hui.** Chacune marque la PR en échec.

| Porte | Ce qu'elle refuse |
|---|---|
| \`socle / secrets\` | Un secret |
| \`contenu\` | Une donnée personnelle |
| \`verifier\` | Un test rouge |

Pas de matrice.

## Protection de \`main\`

La règle du dépôt, \`branche-principale\`, ne vit que sur GitHub. Depuis le 2026-11-03, elle
requiert deux contrôles : \`socle / secrets\` et
\`verifier\`. \`contenu\` n'y est pas requis.

## Ensuite
`;

test("section rend les lignes d'une section, et ses bornes", () => {
  const s = section(CONVENTIONS, "## Portes de CI");
  assert.equal(s.debut, 3);
  assert.ok(s.lignes.some((l) => l.includes("Pas de matrice")));
  assert.ok(!s.lignes.some((l) => l.includes("branche-principale")));
  assert.equal(section(CONVENTIONS, "## Absente"), null);
});

test("portesDuTableau lit le compte en lettres et les portes du tableau", () => {
  const p = portesDuTableau(section(CONVENTIONS, "## Portes de CI"));
  assert.equal(p.compte, 4);
  assert.deepEqual(p.portes, ["socle / secrets", "contenu", "verifier"]);
});

test("controlesRequisEcrits lit le compte, les requis et les non-requis", () => {
  const r = controlesRequisEcrits(section(CONVENTIONS, "## Protection de `main`"));
  assert.deepEqual(r, { compte: 2, requis: ["socle / secrets", "verifier"], nonRequis: ["contenu"] });
  const tous = controlesRequisEcrits({ lignes: ["Elle requiert deux contrôles : `a` et `b`, soit toutes les portes."] });
  assert.deepEqual(tous, { compte: 2, requis: ["a", "b"], nonRequis: null });
});

test("phrasesSurLesRequis refuse la phrase même de LE-9, et rien de ce qui ne dit pas une porte requise", () => {
  const portes = ["socle / secrets", "socle / confidentialite", "contenu", "documentation", "verifier", "windows", "mutants"];
  const le9 = "**Sept, aujourd'hui.** La règle de protection (voir plus bas) n'exige que les deux contrôles du\nsocle ; `contenu`, `documentation`, `verifier`, `windows` et `mutants` ne la bloquent pas.";
  // La phrase se coupe au point-virgule : sa seconde moitié, qui nomme les portes, est ligne 2.
  assert.deepEqual(phrasesSurLesRequis(le9, portes).map((p) => p.ligne), [1, 2]);
  const juste = [
    "Celles que la règle de protection exige sont dites à un seul endroit, plus bas, à « Protection de `main` ».",
    "Requise ou non, une porte rouge arrête la fusion.",
    "",
    "| `verifier` | Un test rouge |",
    "",
    "Le job `documentation` vérifie les faits recopiés. Il compare les contrôles requis à GitHub.",
    "",
    "Il est impossible de lire `contenu` ici. Elle impose trois graphies.",
  ].join("\n");
  assert.deepEqual(phrasesSurLesRequis(juste, portes), []);
  const faux = [
    "La règle de protection les exige.",
    "",
    "La règle de protection requerra `windows` et `verifier` jusqu'à INC-2.",
    "",
    "Seuls `socle / secrets` et `socle / confidentialite` sont obligatoires pour fusionner.",
    "",
    "La règle de protection n'impose rien d'autre.",
    "",
    "| `verifier` | Requise par la règle de protection. |",
  ].join("\n");
  assert.deepEqual(phrasesSurLesRequis(faux, portes).map((p) => p.ligne), [1, 3, 5, 7, 9]);
});

test("jobsDeClaude lit la liste des jobs de CLAUDE.md", () => {
  const texte = "- La CI (`.github/workflows/ci.yml`) a pour jobs `socle`, `contenu` et `mutants`. Le passage…\n";
  assert.deepEqual(jobsDeClaude(texte), { ligne: 1, jobs: ["socle", "contenu", "mutants"] });
});

test("membresDuCargo et nomDuPaquet lisent l'espace de travail", () => {
  assert.deepEqual(membresDuCargo('[workspace]\nmembers = ["crates/noyau", "app/ui"]\nresolver = "3"\n'), ["crates/noyau", "app/ui"]);
  assert.deepEqual(membresDuCargo('[workspace]\nmembers = [\n  "crates/noyau", # la théorie\n  "app/ui",\n]\n'), ["crates/noyau", "app/ui"]);
  assert.equal(nomDuPaquet('[package]\nname = "interface"\nversion.workspace = true\n'), "interface");
  assert.equal(nomDuPaquet('[lib]\nname = "etabli_lib"\n\n[package]\nname = "etabli"\n'), "etabli");
});

test("cratesDeClaude lit la liste des crates et leurs chemins", () => {
  const texte = "- Les crates :\n  - `noyau` (`crates/noyau`) : la théorie ;\n  - `interface` (`app/ui`) : l'interface.\n- La CI…\n";
  assert.deepEqual(cratesDeClaude(texte), {
    ligne: 1, crates: [{ nom: "noyau", chemin: "crates/noyau" }, { nom: "interface", chemin: "app/ui" }],
  });
});

test("comptesNommes relève les comptes en lettres et en chiffres, et laisse les articles", () => {
  const noms = ["portes", "ADR", "décisions d'architecture", "modules"];
  const texte = [
    "Vingt et une décisions d'architecture, une par fichier.",
    "",
    "Le dépôt a quatre portes de CI, et 21 ADR.",
    "",
    "Une ADR par décision ; les portes de CI ; Neuf modules.",
  ].join("\n");
  assert.deepEqual(comptesNommes(texte, noms).map(({ ligne, nom, valeur, chiffres, ecrit }) => [ligne, nom, valeur, chiffres, ecrit]), [
    [1, "décisions d'architecture", 21, false, "Vingt et une"],
    [3, "portes", 4, false, "quatre"],
    [3, "ADR", 21, true, "21"],
    [5, "modules", 9, false, "Neuf"],
  ]);
  const graphies = "L'espace de travail aura **six** crates. La CI a (huit jobs). Deux décisions d’architecture.";
  assert.deepEqual(comptesNommes(graphies, ["crates", "jobs", "décisions d'architecture"]).map((c) => [c.nom, c.valeur]), [
    ["crates", 6], ["jobs", 8], ["décisions d'architecture", 2],
  ]);
});
