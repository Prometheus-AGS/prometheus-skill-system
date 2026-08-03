// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  // guideSidebar moved to sidebars-guide.js — the guide is served from the
  // canonical ../docs/guide via a separate plugin instance (id: 'guide').

  memorySidebar: [
    {
      type: 'category',
      label: 'Memory',
      items: [
        'memory/overview',
        'memory/operation-api',
        'memory/executor-and-recovery',
      ],
    },
  ],

  knowledgeLearningSidebar: [
    {
      type: 'category',
      label: 'Knowledge & Learning',
      items: [
        'knowledge-learning/snapshots-and-context',
        'knowledge-learning/hooks-worker-and-receipts',
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
        'operations/doctors-and-mac-certification',
        'operations/logs-recovery-and-failures',
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
      items: [
        'kbd/control-plane',
        'kbd/tokens-and-authentication',
        'kbd/bash-mutation-guard',
      ],
    },
    {
      type: 'category',
      label: 'Operations',
      items: [
        'kbd/operator-controls',
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
