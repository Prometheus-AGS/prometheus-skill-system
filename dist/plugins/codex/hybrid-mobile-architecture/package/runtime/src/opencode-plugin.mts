// TJ-ARCH-MOB-001 compliant
import { readFile } from 'node:fs/promises';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
interface Skill { name: string; terms: string[] }
interface Manifest { skills?: Skill[] }
const pluginDirectory = dirname(fileURLToPath(import.meta.url));
async function readManifest(directory: string): Promise<Manifest | null> {
  const candidates = [process.env.KNOWME_BUILDER_ACTIVATION_MANIFEST, join(directory, '.knowme-builder/activation-manifest.json'), resolve(pluginDirectory, '../../templates/activation-manifest.json'), join(process.env.XDG_CONFIG_HOME ?? join(homedir(), '.config'), 'opencode/knowme-builder/activation-manifest.json')];
  for (const candidate of candidates) if (candidate) try { return JSON.parse(await readFile(candidate, 'utf8')) as Manifest; } catch { /* try next */ }
  return null;
}
export const KnowMeBuilderPlugin = async ({ directory }: { directory: string }) => {
  const manifest = await readManifest(directory), sessionHints = new Map<string, string[]>();
  return {
    'chat.message': async (input: { sessionID: string }, output: { parts?: { type?: string; text?: string }[] }) => {
      const prompt = (output.parts ?? []).filter(part => part?.type === 'text' && typeof part.text === 'string').map(part => part.text).join('\n').toLowerCase();
      const matches = (Array.isArray(manifest?.skills) ? manifest.skills : []).filter(skill => typeof skill?.name === 'string' && Array.isArray(skill.terms) && skill.terms.some(term => typeof term === 'string' && prompt.includes(term.toLowerCase()))).map(skill => skill.name).slice(0, 8);
      if (matches.length) sessionHints.set(input.sessionID, matches); else sessionHints.delete(input.sessionID);
    },
    'experimental.chat.system.transform': async (input: { sessionID?: string }, output: { system: string[] }) => {
      const matches = input.sessionID ? sessionHints.get(input.sessionID) : undefined;
      if (matches?.length) output.system.push(`Relevant KnowMe Builder skills (advisory; load only when applicable): ${matches.join(', ')}. Prometheus remains the development lifecycle and KBD authority.`);
    },
    event: async ({ event }: { event?: { type?: string; properties?: { info?: { id?: string } } } }) => {
      if (event?.type === 'session.deleted' && event.properties?.info?.id) sessionHints.delete(event.properties.info.id);
    },
  };
};
