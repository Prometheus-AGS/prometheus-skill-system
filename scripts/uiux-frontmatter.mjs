// Distribution metadata only. Preserve the upstream body, identities, versions
// and invocation flags byte-for-byte; do not serialize the entire YAML header.
export const frontmatterAdaptation = 'Filled missing strict distribution license, version and metadata.tags without replacing upstream values or invocation flags. When upstream supplies no version, 1.0.0-prometheus.1 is a Prometheus packaging version, not an upstream release claim.';

export function normalizeUiuxFrontmatter(content, license, id) {
  const match = /^(---\r?\n)([\s\S]*?)(\r?\n---(?:\r?\n|$))/.exec(content);
  if (!match) throw new Error(`Missing skill frontmatter: ${id}`);
  const newline = match[1].includes('\r\n') ? '\r\n' : '\n';
  let header = match[2];
  if (!/^license:\s*\S/m.test(header)) header += `${newline}license: ${license}`;
  const hasVersion = /^(?: {2})?version:\s*\S/m.test(header);
  if (!hasVersion) header += `${newline}version: 1.0.0-prometheus.1`;
  if (!/^metadata:\s*$/m.test(header)) header += `${newline}metadata:`;
  const additions = [];
  if (!/^ {2}tags:\s*\S/m.test(header)) additions.push(`  tags: ui, ux, ${id}`);
  if (!hasVersion) additions.push('  version-origin: Prometheus packaging version; upstream did not declare a version');
  if (additions.length) header = header.replace(/^metadata:\s*$/m, `metadata:${newline}${additions.join(newline)}`);
  return match[1] + header + match[3] + content.slice(match[0].length);
}
