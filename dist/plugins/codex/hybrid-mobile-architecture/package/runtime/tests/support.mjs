import { copyFileSync, lstatSync, mkdirSync, readdirSync } from 'node:fs';
import { dirname, join } from 'node:path';

export function copyPathExact(source, destination) {
  const sourceStat = lstatSync(source);
  if (sourceStat.isSymbolicLink()) throw new Error(`Test fixture cannot copy symlink: ${source}`);
  if (sourceStat.isDirectory()) {
    mkdirSync(destination, { recursive: true });
    for (const entry of readdirSync(source)) copyPathExact(join(source, entry), join(destination, entry));
    return;
  }
  mkdirSync(dirname(destination), { recursive: true });
  copyFileSync(source, destination);
}
