/**
 * OpenCode tool definition for kbd-process-orchestrator commands.
 * Provides typed entry point for KBD lifecycle operations from OpenCode.
 */
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

export default {
  name: 'kbd',
  description:
    'Run KBD process orchestrator commands — init, assess, plan, execute, reflect, status. ' +
    'Coordinates multi-tool execution with file-based state in .kbd-orchestrator/.',
  parameters: {
    type: 'object' as const,
    properties: {
      command: {
        type: 'string',
        enum: ['init', 'assess', 'plan', 'execute', 'reflect', 'status', 'new-phase'],
        description: 'KBD sub-command to run',
      },
      phase: {
        type: 'string',
        description: 'Phase name (optional — defaults to active waypoint phase)',
      },
      goals: {
        type: 'array',
        items: { type: 'string' },
        description: 'Phase goals (required for new-phase command)',
      },
      backend: {
        type: 'string',
        enum: ['openspec', 'native-tool', 'hybrid', 'manual'],
        description: 'Execution backend (for execute command)',
      },
    },
    required: ['command'],
  },
  execute: async (args: Record<string, unknown>, context: { directory: string }) => {
    const { command, phase, goals, backend } = args as {
      command: string;
      phase?: string;
      goals?: string[];
      backend?: string;
    };

    const kbdDir = join(context.directory, '.kbd-orchestrator');
    const hasKbd = existsSync(kbdDir);
    const waypointPath = join(kbdDir, 'current-waypoint.json');

    let waypoint = null;
    if (existsSync(waypointPath)) {
      try {
        waypoint = JSON.parse(readFileSync(waypointPath, 'utf-8'));
      } catch { /* proceed without waypoint */ }
    }

    const skillPath = 'skills/process/kbd-process-orchestrator/SKILL.md';
    const subSkill = command === 'new-phase' ? 'kbd-init' : `kbd-${command}`;

    return {
      action: 'invoke_skill',
      skill: skillPath,
      command: `/${subSkill}${phase ? ` ${phase}` : ''}`,
      context: {
        command,
        phase: phase ?? waypoint?.active_phase,
        goals,
        backend,
        kbd_initialized: hasKbd,
        active_waypoint: waypoint,
        openspec_available: existsSync(join(context.directory, 'openspec')),
        has_constraints: existsSync(join(kbdDir, 'constraints.md')),
      },
    };
  },
};
