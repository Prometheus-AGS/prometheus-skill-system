#!/usr/bin/env node

import path from 'node:path';
import { writeReviewedSkillClosures } from './lib/reviewed-skill-closures.js';

function parseArgs(argv) {
  const args = { payload: null, sharedRoots: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === '--payload') args.payload = argv[++index];
    else if (value === '--shared-root') args.sharedRoots.push(argv[++index]);
    else throw new Error(`unknown argument: ${value}`);
  }
  if (!args.payload) throw new Error('missing --payload');
  if (args.sharedRoots.some(entry => !entry)) throw new Error('missing value for --shared-root');
  return { payload: path.resolve(args.payload), sharedRoots: args.sharedRoots };
}

try {
  const args = parseArgs(process.argv.slice(2));
  const inventory = writeReviewedSkillClosures(args.payload, { sharedRoots: args.sharedRoots });
  process.stdout.write(`${JSON.stringify({ inventoryDigest: inventory.inventoryDigest })}\n`);
} catch (error) {
  process.stderr.write(`generate-reviewed-skill-closures: ${error.message}\n`);
  process.exitCode = 1;
}
