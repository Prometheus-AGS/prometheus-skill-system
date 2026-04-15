/**
 * OpenCode tool definition for GitOps skill commands.
 * Provides typed entry point for GitOps bootstrap, transform, ArgoCD, and Kustomize operations.
 */
import { existsSync, readdirSync } from 'fs';
import { join } from 'path';

export default {
  name: 'gitops',
  description:
    'Run GitOps commands — bootstrap a new pipeline, transform existing CI/CD, ' +
    'set up ArgoCD multi-cloud, or generate Kustomize overlays. ' +
    'Follows TJ-CICD-001 standard.',
  parameters: {
    type: 'object' as const,
    properties: {
      command: {
        type: 'string',
        enum: ['bootstrap', 'transform', 'argocd', 'kustomize'],
        description: 'GitOps sub-command',
      },
      service: {
        type: 'string',
        description: 'Service name (kebab-case) for kustomize overlays',
      },
      clouds: {
        type: 'array',
        items: { type: 'string', enum: ['gke', 'aks', 'eks'] },
        description: 'Target clouds (auto-detected if omitted)',
      },
    },
    required: ['command'],
  },
  execute: async (args: Record<string, unknown>, context: { directory: string }) => {
    const { command, service, clouds } = args as {
      command: string;
      service?: string;
      clouds?: string[];
    };

    // Detect existing GitOps state
    const hasGitops = existsSync(join(context.directory, 'gitops'));
    const hasWorkflows = existsSync(join(context.directory, '.github', 'workflows'));
    const hasTerraform = readdirSync(context.directory).some((f) => f.endsWith('.tf'));
    const hasKubeconfig = existsSync(join(process.env.HOME ?? '', '.kube', 'config'));

    const skillMap: Record<string, string> = {
      bootstrap: 'skills/devops/gitops-bootstrap/SKILL.md',
      transform: 'skills/devops/gitops-transform/SKILL.md',
      argocd: 'skills/devops/argocd-multicloud/SKILL.md',
      kustomize: 'skills/devops/kustomize-overlay/SKILL.md',
    };

    return {
      action: 'invoke_skill',
      skill: skillMap[command],
      command: `/${command}${service ? ` ${service}` : ''}`,
      context: {
        command,
        service,
        clouds: clouds ?? [],
        has_gitops_dir: hasGitops,
        has_workflows: hasWorkflows,
        has_terraform: hasTerraform,
        has_kubeconfig: hasKubeconfig,
      },
    };
  },
};
