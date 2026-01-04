import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

function sh(cmd, opts = {}) {
  // cargo metadata can be very large; increase buffer to avoid ENOBUFS.
  return execSync(cmd, {
    stdio: ['ignore', 'pipe', 'pipe'],
    encoding: 'utf8',
    maxBuffer: 256 * 1024 * 1024,
    ...opts,
  });
}

function summarizePnpmLicenses(repoRoot) {
  const jsonText = sh('pnpm licenses list --json', { cwd: repoRoot });
  const byLicense = JSON.parse(jsonText);

  const counts = Object.entries(byLicense)
    .map(([lic, pkgs]) => [lic, Array.isArray(pkgs) ? pkgs.length : 0])
    .sort((a, b) => b[1] - a[1]);

  const nonMit = Object.keys(byLicense)
    .filter((l) => l !== 'MIT')
    .sort()
    .map((lic) => ({
      license: lic,
      packages: byLicense[lic]
        .map((p) => ({ name: p.name, versions: p.versions }))
        .sort((a, b) => a.name.localeCompare(b.name)),
    }));

  return {
    totalPackages: counts.reduce((acc, [, n]) => acc + n, 0),
    distinctLicenses: counts.length,
    topLicenses: counts.slice(0, 30),
    nonMit,
  };
}

function summarizeCargoLicenses(repoRoot) {
  const cargoDir = join(repoRoot, 'src-tauri');
  const metaText = sh('CARGO_NET_OFFLINE=true cargo metadata --format-version 1', {
    cwd: cargoDir,
  });
  const meta = JSON.parse(metaText);
  const pkgs = meta.packages ?? [];

  const counts = new Map();
  const byLicense = new Map();
  for (const p of pkgs) {
    const lic = (p.license ?? 'UNKNOWN').trim() || 'UNKNOWN';
    counts.set(lic, (counts.get(lic) ?? 0) + 1);
    if (!byLicense.has(lic)) byLicense.set(lic, []);
    byLicense.get(lic).push({ name: p.name, version: p.version });
  }

  const topLicenses = [...counts.entries()].sort((a, b) => b[1] - a[1]).slice(0, 30);
  const unknown = (byLicense.get('UNKNOWN') ?? []).sort((a, b) => a.name.localeCompare(b.name));

  // “MIT配布と相性が悪い可能性が高い”ものを機械的に抽出（ただし OR で回避可能なものは手動判断）
  const suspectRe = /(\bAGPL\b|\bGPL\b|\bLGPL\b|SSPL|BUSL|CPAL|EUPL)/i;
  const suspect = [];
  for (const [lic, arr] of byLicense.entries()) {
    if (lic !== 'UNKNOWN' && suspectRe.test(lic)) {
      for (const x of arr) suspect.push({ ...x, license: lic });
    }
  }

  return {
    totalCrates: pkgs.length,
    distinctLicenses: counts.size,
    topLicenses,
    unknown,
    suspect,
  };
}

function main() {
  const repoRoot = process.cwd();
  const nodeSummary = summarizePnpmLicenses(repoRoot);
  const rustSummary = summarizeCargoLicenses(repoRoot);

  const report = {
    generatedAt: new Date().toISOString(),
    node: nodeSummary,
    rust: rustSummary,
  };

  writeFileSync(
    join(repoRoot, 'license-report.json'),
    JSON.stringify(report, null, 2) + '\n',
    'utf8'
  );

  console.log('Wrote license-report.json');
}

main();
