#!/usr/bin/env node
// Frontend bundle budget for Copyfie (PLAN.md §8, §11).
//
// Sums gzip sizes of the production JS + CSS emitted to dist/. Fails the build
// when the total exceeds the budget so a stray library cannot silently ship.
//
// Budget: 50 KB gzipped for all JS + CSS. The plan targets <50 KB; raise this
// only with a recorded reason in progress.md.

import { gzipSync } from "node:zlib";
import { readFileSync, readdirSync } from "node:fs";
import { extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const DIST = fileURLToPath(new URL("../../dist/", import.meta.url));
const BUDGET_BYTES = 50 * 1024;
const COUNTED = new Set([".js", ".css"]);

function walk(dir) {
  const files = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(full));
    } else if (COUNTED.has(extname(entry.name))) {
      files.push(full);
    }
  }
  return files;
}

let total = 0;
let rows = [];

try {
  for (const file of walk(DIST)) {
    const raw = readFileSync(file);
    const gz = gzipSync(raw, { level: 9 }).length;
    total += gz;
    rows.push({ file: relative(DIST, file), raw: raw.length, gz });
  }
} catch (cause) {
  console.error(`Cannot read ${DIST}. Run \`npm run build\` first.`);
  console.error(cause instanceof Error ? cause.message : cause);
  process.exit(1);
}

rows.sort((a, b) => b.gz - a.gz);
for (const row of rows) {
  console.log(
    `${row.file.padEnd(36)} ${(row.raw / 1024).toFixed(1).padStart(7)} KB raw ${(row.gz / 1024).toFixed(1).padStart(6)} KB gz`,
  );
}

const budgetKb = (BUDGET_BYTES / 1024).toFixed(0);
const totalKb = (total / 1024).toFixed(1);
console.log(`\nTotal gzipped: ${totalKb} KB / budget ${budgetKb} KB`);

if (total > BUDGET_BYTES) {
  console.error(`Frontend bundle exceeds the ${budgetKb} KB budget by ${((total - BUDGET_BYTES) / 1024).toFixed(1)} KB.`);
  process.exit(1);
}
