import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { assert, files, main, npm, text } from './common.mjs';
await main(() => {
    const site = resolve(process.argv[2] ?? 'site');
    assert(existsSync(join(site, 'package-lock.json')), `missing package-lock.json: ${site}`);
    npm(['ci'], site);
    npm(['run', 'sanitize'], site);
    npm(['run', 'build'], site);
    for (const file of files(join(site, 'build')))
        assert(!/\/Users\/|\.prometheus\/|BEGIN .*PRIVATE KEY/.test(text(file)), `private material found in site output: ${file}`);
    console.log('branded Docusaurus verification passed');
});
