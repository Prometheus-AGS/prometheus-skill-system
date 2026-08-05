import fs from 'node:fs';
import path from 'node:path';

const inputPath = path.join(process.env.PROMETHEUS_INPUT_DIR, 'records');
const outputPath = path.join(process.env.PROMETHEUS_OUTPUT_DIR, 'summary.json');
const records = JSON.parse(fs.readFileSync(inputPath, 'utf8')).records;
const summary = {
  count: records.length,
  ids: records.map(record => record.id).sort(),
  maxRisk: Math.max(...records.map(record => record.risk)),
  totalRisk: records.reduce((total, record) => total + record.risk, 0),
};
fs.writeFileSync(outputPath, `${JSON.stringify(summary, null, 2)}\n`, 'utf8');
process.stdout.write(`${JSON.stringify(summary)}\n`);
