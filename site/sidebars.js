// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  // guideSidebar moved to sidebars-guide.js — the guide is served from the
  // canonical ../docs/guide via a separate plugin instance (id: 'guide').

  agentContextSidebar: [
    {
      type: 'category',
      label: 'Agent Context',
      items: [
        'agent-context/overview',
        'agent-context/quick-start',
        'agent-context/use-cases',
        'agent-context/model-profiles',
        'agent-context/harness-support',
        'agent-context/skill-budget',
        'agent-context/theory-and-sources',
      ],
    },
  ],

  memorySidebar: [
    {
      type: 'category',
      label: 'Memory',
      items: ['memory/overview', 'memory/operation-api', 'memory/executor-and-recovery'],
    },
  ],

  knowledgeLearningSidebar: [
    {
      type: 'category',
      label: 'Knowledge & Learning',
      items: [
        'knowledge-learning/snapshots-and-context',
        'knowledge-learning/hooks-worker-and-receipts',
        'knowledge-learning/loro-evidence-and-migration',
        'knowledge-learning/migration-and-troubleshooting',
      ],
    },
  ],

  pluginDistributionSidebar: [
    {
      type: 'category',
      label: 'Plugin Distribution',
      items: [
        'plugin-distribution/immutable-generations',
        'plugin-distribution/signing-index-and-receipts',
        'plugin-distribution/targets-and-dispatchers',
        'plugin-distribution/activation-rollback-uninstall',
      ],
    },
  ],

  operationsSidebar: [
    {
      type: 'category',
      label: 'Operations',
      items: [
        'operations/installation-and-upgrades',
        'operations/local-validation-and-docs-automation',
        'operations/generated-reference',
        'operations/doctors-and-mac-certification',
        'operations/logs-recovery-and-failures',
      ],
    },
  ],

  executionSidebar: [
    {
      type: 'category',
      label: 'Dynamic Operations',
      items: [
        'execution/overview-and-use-cases',
        'execution/choosing-the-right-capability',
        'execution/closed-loop-architecture',
        'execution/generating-programs',
        'execution/architecture-and-tiers',
        'execution/tier-p-native-processes',
        'execution/tier-w-portable-components',
        'execution/local-api-cli-and-mcp',
        'execution/remote-dispatch-and-reconciliation',
        'execution/receipts-verification-and-certification',
        'execution/use-case-cookbook',
        'execution/security-and-trust',
        'execution/installation-doctor-and-recovery',
        'execution/platform-and-evidence-status',
      ],
    },
  ],

  learnSidebar: [
    {
      type: 'category',
      label: 'Learn Domain',
      items: [
        'learn/overview',
        'learn/feynman-loop',
        'learn/mastery-criterion',
        'learn/kb-adapters',
        'learn/anti-sycophancy',
      ],
    },
    {
      type: 'category',
      label: 'Skills',
      items: [
        'learn/skills/learn-goal',
        'learn/skills/learn-survey',
        'learn/skills/learn-plan',
        'learn/skills/learn-grade',
        'learn/skills/learn-retain',
        'learn/skills/learn-practice',
        'learn/skills/learn-certify',
        'learn/skills/learn-kb',
        'learn/skills/learn-harness',
        'learn/skills/learn-about-system',
      ],
    },
  ],

  kbdSidebar: [
    {
      type: 'category',
      label: 'Lifecycle',
      items: ['kbd/overview', 'kbd/stages'],
    },
    {
      type: 'category',
      label: 'Control Plane',
      items: ['kbd/control-plane', 'kbd/tokens-and-authentication', 'kbd/bash-mutation-guard'],
    },
    {
      type: 'category',
      label: 'Operations',
      items: [
        'kbd/operator-controls',
        'kbd/checkpoints-compaction-recovery',
        'kbd/migration-and-rollout',
        'kbd/hooks-and-waypoints',
        'kbd/troubleshooting',
      ],
    },
    {
      type: 'category',
      label: 'Quality',
      items: ['kbd/quality-gates'],
    },
  ],

  substrateSidebar: [
    {
      type: 'category',
      label: 'Substrate Crates',
      items: [
        'substrate/index',
        'substrate/kbd-runtime',
        'substrate/storage-provider',
        'substrate/learner-model',
        'substrate/surface-bridge',
        'substrate/sovereign-client',
        'substrate/prometheus-research',
      ],
    },
  ],

  sovereignSidebar: [
    {
      type: 'category',
      label: 'Understand Sync',
      items: [
        'sovereign-sync/overview',
        'sovereign-sync/architecture',
        'sovereign-sync/data-scope',
        'sovereign-sync/privacy-model',
        'sovereign-sync/use-cases',
      ],
    },
    {
      type: 'category',
      label: 'Configure and Operate',
      items: [
        'sovereign-sync/installation',
        'sovereign-sync/pair-two-machines',
        'sovereign-sync/signed-pushes-and-receipts',
        'sovereign-sync/p2p-network',
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        'sovereign-sync/rest-api',
        'sovereign-sync/mcp-tools',
        'sovereign-sync/ag-ui-sse',
        'sovereign-sync/rust-sdk',
      ],
    },
    {
      type: 'category',
      label: 'Skills',
      items: [
        'sovereign-sync/sync-status-skill',
        'sovereign-sync/sync-peers-skill',
        'sovereign-sync/sync-push-skill',
      ],
    },
  ],

  mobileSidebar: [
    {
      type: 'category',
      label: 'Mobile Portability',
      items: [
        'mobile/overview',
        'mobile/execution-classes',
        'mobile/wasm-components',
        'mobile/native-ffi',
      ],
    },
    {
      type: 'category',
      label: 'Runtime Integration',
      items: ['mobile/uar-skill-database'],
    },
  ],
};

module.exports = sidebars;
