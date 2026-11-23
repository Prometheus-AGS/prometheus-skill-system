import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath} from 'node:url';
import {definitions} from './catalog.mjs';

const home = process.env.HOME || os.homedir();
const receipt = JSON.parse(fs.readFileSync(path.join(home,'.prometheus/service-providers/full.json'),'utf8'));
const ids = Object.values(receipt.services).filter(s => s.provider === 'native').map(s => s.id);
if (process.argv[2] === 'services') console.log(ids.join(','));
else {
  const catalog = JSON.parse(fs.readFileSync(fileURLToPath(new URL('../../config/release-binaries.json',import.meta.url)),'utf8'));
  const platform = 'windows-' + (process.arch === 'arm64' ? 'arm64' : 'amd64');
  const skipped = new Set(Object.entries(definitions).filter(([id]) => !ids.includes(id)).map(([,d]) => d.binary));
  for (const component of catalog.components) {
    if (component.platforms && !component.platforms.includes(platform)) continue;
    for (const name of component.binaries) if (!skipped.has(name)) console.log(name);
  }
}
