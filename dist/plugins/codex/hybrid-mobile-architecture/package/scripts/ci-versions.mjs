// TJ-ARCH-MOB-001 compliant
import { appendFileSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { main, packageRoot } from './portable/platform.mjs';
await main(() => {
    if (process.argv.length !== 2)
        throw new Error('Usage: node scripts/ci-versions.mjs');
    const source = readFileSync(join(packageRoot, 'versions.toml'), 'utf8');
    const section = (name) => source.match(new RegExp(`^\\[${name}\\][^\\n]*\\n([\\s\\S]*?)(?=^\\[|$(?![\\s\\S]))`, 'm'))?.[1];
    const values = {};
    for (const [outputKey, sectionName, sourceKey] of [
        ['rust', 'toolchain', 'rust'],
        ['node', 'toolchain', 'node'],
        ['flutter', 'toolchain', 'flutter'],
        ['flutter_rust_bridge', 'frameworks', 'flutter_rust_bridge'],
        ['tauri_driver', 'frameworks', 'tauri_driver'],
    ]) {
        const value = section(sectionName)?.match(new RegExp(`^${sourceKey}\\s*=\\s*"([^"\\r\\n]+)"`, 'm'))?.[1];
        if (!value || !/^\d+\.\d+\.\d+(?:[-+][\w.-]+)?$/.test(value)) {
            throw new Error(`Invalid ${sectionName}.${sourceKey} version`);
        }
        values[outputKey] = value;
    }
    const edgeRevision = section('frameworks')?.match(/^msedgedriver_tool_rev\s*=\s*"([0-9a-f]{40})"/m)?.[1];
    if (!edgeRevision)
        throw new Error('Invalid frameworks.msedgedriver_tool_rev revision');
    values.msedgedriver_tool_rev = edgeRevision;
    const output = Object.entries(values).map(([key, value]) => `${key}=${value}\n`).join('');
    if (process.env.GITHUB_OUTPUT)
        appendFileSync(process.env.GITHUB_OUTPUT, output);
    process.stdout.write(output);
});
