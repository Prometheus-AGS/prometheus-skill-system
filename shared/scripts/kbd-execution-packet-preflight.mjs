#!/usr/bin/env node
// Inspect packet documents without running any command named in them.
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { preflightExecutionPacket } from './lib/kbd-execution-packet.mjs';

function read(file) {
  return fs.readFileSync(file, 'utf8');
}

try {
  const argv = process.argv.slice(2);
  if (argv.length === 1 && ['--help', '-h'].includes(argv[0])) {
    console.log('Usage: node kbd-execution-packet-preflight.mjs PARENT.json BINDING.json PACKET.json');
  } else {
    if (argv.length !== 3) throw new Error('expected parent, binding, and packet JSON paths');
    const [parent, binding, raw] = argv.map(read);
    const digest = createHash('sha256').update(raw).digest('hex');
    const result = preflightExecutionPacket(JSON.parse(parent), JSON.parse(binding), JSON.parse(raw));
    console.log(JSON.stringify({ ...result, exactPacketBytesSha256: digest }, null, 2));
  }
} catch (error) {
  console.error(`REFUSED: ${error.message}`);
  process.exitCode = 2;
}
