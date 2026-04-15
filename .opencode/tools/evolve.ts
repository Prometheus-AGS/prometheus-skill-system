/**
 * OpenCode tool definition for iterative-evolver /evolve command.
 * Provides typed entry point for running evolution cycles from OpenCode.
 */
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

export default {
  name: 'evolve',
  description:
    'Run an iterative evolution cycle — assess, analyze, plan, execute, reflect. ' +
    'For software projects, delegates execution to kbd-process-orchestrator with ' +
    'OpenSpec detection and artifact-refiner QA.',
  parameters: {
    type: 'object' as const,
    properties: {
      evolution_name: {
        type: 'string',
        description: 'Human-friendly name for cross-session retrieval (e.g., "api-improvement")',
      },
      domain: {
        type: 'string',
        enum: ['software', 'business', 'product', 'research', 'content', 'operations', 'compliance', 'generic'],
        description: 'Evolution domain — determines adapter and execution strategy',
      },
      goals: {
        type: 'array',
        items: { type: 'string' },
        description: 'What are we trying to achieve?',
      },
      phase: {
        type: 'string',
        enum: ['assess', 'analyze', 'plan', 'execute', 'reflect', 'full'],
        description: 'Run a specific phase or full cycle (default: full)',
      },
    },
    required: ['evolution_name'],
  },
  execute: async (args: Record<string, unknown>, context: { directory: string }) => {
    const { evolution_name, domain, goals, phase } = args as {
      evolution_name: string;
      domain?: string;
      goals?: string[];
      phase?: string;
    };

    // Check for existing evolution state
    const stateDir = join(context.directory, '.evolver', 'evolutions', evolution_name);
    const hasState = existsSync(join(stateDir, 'state.json'));

    let currentState = null;
    if (hasState) {
      try {
        currentState = JSON.parse(readFileSync(join(stateDir, 'state.json'), 'utf-8'));
      } catch { /* proceed without state */ }
    }

    // Build the skill invocation instruction
    const skillPath = 'skills/process/iterative-evolver/SKILL.md';
    const subCommand = phase && phase !== 'full' ? `evolve-${phase}` : 'evolve';

    return {
      action: 'invoke_skill',
      skill: skillPath,
      command: `/${subCommand} "${evolution_name}"`,
      context: {
        evolution_name,
        domain: domain ?? currentState?.domain ?? 'software',
        goals: goals ?? currentState?.goals ?? [],
        has_existing_state: hasState,
        current_iteration: currentState?.iteration ?? 0,
        kbd_available: existsSync(join(context.directory, '.kbd-orchestrator')),
        openspec_available: existsSync(join(context.directory, 'openspec')),
      },
    };
  },
};
