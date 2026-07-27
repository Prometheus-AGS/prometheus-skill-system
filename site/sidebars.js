// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  // guideSidebar moved to sidebars-guide.js — the guide is served from the
  // canonical ../docs/guide via a separate plugin instance (id: 'guide').

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
      label: 'KBD Lifecycle',
      items: ['kbd/overview', 'kbd/stages', 'kbd/hooks-and-waypoints', 'kbd/quality-gates'],
    },
  ],

  substrateSidebar: [
    {
      type: 'category',
      label: 'Substrate Crates',
      items: [
        'substrate/index',
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
      label: 'Sovereign Sync',
      items: [
        'sovereign-sync/overview',
        'sovereign-sync/architecture',
        'sovereign-sync/installation',
        'sovereign-sync/privacy-model',
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
};

module.exports = sidebars;
