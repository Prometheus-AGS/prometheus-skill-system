// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  guideSidebar: [
    {
      type: 'category',
      label: 'Getting Started',
      items: ['guide/introduction', 'guide/installation', 'guide/quick-start'],
    },
    {
      type: 'category',
      label: 'Architecture',
      items: [
        'guide/metaprompting-pmpo-kbd',
        'guide/loop-architecture',
        'guide/four-layer-pipeline',
        'guide/mcp-substrate',
        'guide/memory-and-learning',
        'guide/sycophancy-correction',
      ],
    },
    {
      type: 'category',
      label: 'Skills Reference',
      items: ['guide/skills-overview', 'guide/process-skills', 'guide/language-skills'],
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
