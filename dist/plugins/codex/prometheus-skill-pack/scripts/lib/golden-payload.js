/**
 * The fixed payload whose identity every supported host must agree on.
 *
 * Cross-host identity equality cannot be observed from one host, so it is
 * asserted as a CONSTANT: every leg materializes this same description and must
 * compute the same digest. A host that computes anything else has introduced
 * host dependence, which is the exact defect this change removes.
 *
 * The description lives here rather than inside the fixture because two things
 * need it and they must not drift: the fixture asserts the digest, and the
 * host-leg verifier records it into a receipt that the matrix comparator reads.
 * One definition, one digest.
 *
 * The payload deliberately covers every entry shape identity can express: a
 * non-executable file, an executable file, an empty directory, a link to a
 * directory, and a link to a file. Content is fixed literal text with explicit
 * `\n`, so no host's line-ending conventions can reach it.
 */

import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import { materializeLink } from './capabilities.js';
import { jcs } from './jcs.js';
import { applyManifestMode, collectPayloadEntries } from './payload-manifest.js';

export const GOLDEN_PAYLOAD = Object.freeze({
  directories: ['bin', 'data', 'empty'],
  files: [
    { path: 'bin/run.sh', content: '#!/bin/sh\necho hi\n', executable: true },
    { path: 'data/plain.txt', content: 'plain\n', executable: false },
    { path: 'top.json', content: '{"a":1}\n', executable: false },
  ],
  links: [
    { path: 'alias', target: 'bin' },
    { path: 'note-link', target: 'data/plain.txt' },
  ],
});

/**
 * Golden digest for the payload above.
 *
 * Regenerating this value is legitimate only when the payload description or
 * the entry schema changes -- never to make a failing host pass.
 */
export const GOLDEN_DIGEST = 'eb5271efd4c8fe85ca4eb7574189191a16b2898d9b2b8ce312f4466cea484591';

const INTENTS = new Map([
  ...GOLDEN_PAYLOAD.directories.map(entry => [entry, { type: 'directory' }]),
  ...GOLDEN_PAYLOAD.files.map(entry => [
    entry.path,
    { type: 'file', executable: entry.executable },
  ]),
  ...GOLDEN_PAYLOAD.links.map(entry => [entry.path, { type: 'symlink', target: entry.target }]),
]);

/** Intent lookup for the golden payload. Unknown paths are not part of it. */
export function goldenIntentOf(relative) {
  return INTENTS.get(relative) ?? null;
}

/**
 * Materialize the golden payload under `root` and compute its identity.
 *
 * Returns the entries, their digest, and any degradations the host had to make.
 * The degradations are reported, not hashed: a host that wrote a link as a copy
 * must still land on the same digest, which is the property under test.
 */
export function buildGoldenPayload(root, capabilities) {
  fs.mkdirSync(root, { recursive: true });
  for (const directory of GOLDEN_PAYLOAD.directories) {
    const absolute = path.join(root, directory);
    fs.mkdirSync(absolute, { recursive: true });
    applyManifestMode(absolute, { type: 'directory' }, capabilities);
  }
  for (const entry of GOLDEN_PAYLOAD.files) {
    const absolute = path.join(root, ...entry.path.split('/'));
    fs.writeFileSync(absolute, entry.content, 'utf8');
    applyManifestMode(absolute, { type: 'file', executable: entry.executable }, capabilities);
  }
  const degradations = [];
  for (const entry of GOLDEN_PAYLOAD.links) {
    const outcome = materializeLink({
      linkPath: path.join(root, ...entry.path.split('/')),
      target: entry.target,
      capabilities,
      // Payload links never take the junction rung: a junction's substitute
      // name is absolute and would not survive the rename that settles a
      // staged generation.
      allowJunction: false,
    });
    if (outcome.realized !== 'symlink') {
      degradations.push({ path: entry.path, intended: 'symlink', realized: outcome.realized });
    }
  }
  const entries = collectPayloadEntries(root, goldenIntentOf, capabilities);
  const digest = crypto.createHash('sha256').update(jcs(entries)).digest('hex');
  return { entries, digest, degradations };
}
