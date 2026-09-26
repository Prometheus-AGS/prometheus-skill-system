// TJ-ARCH-MOB-001 compliant
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { packageRoot } from './platform.mjs';
export function versions(file = process.env.VERSIONS_TOML ?? join(packageRoot, 'versions.toml')): Record<string, Record<string, string>> {
  const result: Record<string, Record<string, string>> = {}; let section = '';
  for (const line of readFileSync(file, 'utf8').split(/\r?\n/)) {
    const heading = line.match(/^\s*\[([^\]]+)\]\s*(?:#.*)?$/); if (heading) { section = heading[1]; result[section] ??= {}; continue; }
    const pair = line.match(/^\s*([\w-]+)\s*=\s*"([^"\n]*)"\s*(?:#.*)?$/); if (pair && section) result[section][pair[1]] = pair[2];
  }
  return result;
}
export function requireVersion(section: string, key: string): string { const value = versions()[section]?.[key]; if (!value) throw new Error(`Missing version pin: ${section}.${key}`); return value; }
