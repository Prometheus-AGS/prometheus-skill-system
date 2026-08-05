import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import styles from './index.module.css';

const capabilities = [
  {
    label: 'Durable operations',
    title: 'Memory',
    description:
      'Submit idempotent operations, replay exact receipts, and recover interrupted work without guessing from retry counts.',
    to: '/docs/memory/overview',
  },
  {
    label: 'Bounded context',
    title: 'Knowledge & Learning',
    description:
      'Turn immutable evidence into project, shared, and global snapshots through an asynchronous supervised worker.',
    to: '/docs/knowledge-learning/snapshots-and-context',
  },
  {
    label: 'Trusted delivery',
    title: 'Plugin Distribution',
    description:
      'Activate signed, immutable generations across supported agent tools with atomic rollback and target receipts.',
    to: '/docs/plugin-distribution/immutable-generations',
  },
  {
    label: 'Local-first release',
    title: 'Operations',
    description:
      'Build, install, diagnose, and recover on the host while keeping hosted automation limited to documentation.',
    to: '/docs/operations/installation-and-upgrades',
  },
];

const exploreLinks = [
  {
    title: 'Learn with the Feynman loop',
    description: 'Move from a learning goal to retained, independently graded mastery.',
    to: '/docs/learn/overview',
  },
  {
    title: 'Pair trusted peers',
    description: 'Understand signed pushes, receipts, allow-lists, and private transport.',
    to: '/docs/sovereign-sync/overview',
  },
  {
    title: 'Browse the complete guide',
    description: 'Follow the architecture from loop design through installation and upgrades.',
    to: '/docs/guide/introduction',
  },
];

export default function Home() {
  return (
    <Layout
      title="Prometheus Skill Pack"
      description="Governed AI skills, dynamic operations, durable evidence, and compounding learning"
    >
      <main className={styles.main}>
        <section className={styles.hero} aria-labelledby="home-title">
          <div className={`container ${styles.heroGrid}`}>
            <div className={styles.heroCopy}>
              <p className={styles.eyebrow}>Prometheus 1.7.0</p>
              <h1 id="home-title">Build AI systems that learn without losing control.</h1>
              <p className={styles.lead}>
                Create capabilities, run bounded dynamic operations, preserve signed evidence, and
                turn verified outcomes into durable learning.
              </p>
              <div className={styles.actions}>
                <Link
                  className={`button button--primary button--lg ${styles.primaryAction}`}
                  to="/docs/guide/introduction"
                >
                  Get started
                </Link>
                <Link
                  className={`button button--secondary button--lg ${styles.secondaryAction}`}
                  to="/docs/catalog/"
                >
                  Browse skills
                </Link>
              </div>
            </div>

            <aside className={styles.releaseCard} aria-labelledby="release-card-title">
              <p className={styles.cardKicker}>Current release</p>
              <h2 id="release-card-title">One certified source, every supported tool.</h2>
              <dl className={styles.metrics}>
                <div>
                  <dt>145</dt>
                  <dd>loadable skills</dd>
                </div>
                <div>
                  <dt>14</dt>
                  <dd>signed targets</dd>
                </div>
                <div>
                  <dt>6</dt>
                  <dd>versioned binaries</dd>
                </div>
              </dl>
              <Link
                className={styles.inlineLink}
                to="/docs/operations/doctors-and-mac-certification"
              >
                See the evidence model <span aria-hidden="true">→</span>
              </Link>
            </aside>
          </div>
        </section>

        <section className={styles.dynamicOperations} aria-labelledby="dynamic-operations-title">
          <div className="container">
            <div className={styles.featuredCapability}>
              <div>
                <p className={styles.eyebrow}>Dynamic Operations</p>
                <h2 id="dynamic-operations-title">Turn generated code into verifiable work.</h2>
                <p>
                  Prometheus Exec runs bounded Python, Node, Bash, and authorized WebAssembly
                  components. It records ordered events, content-addressed artifacts, and a signed
                  receipt—without replacing ordinary agent tools or the native-agent generator.
                </p>
              </div>
              <nav className={styles.featuredLinks} aria-label="Dynamic Operations resources">
                <Link to="/docs/execution/overview-and-use-cases">
                  Understand the execution loop
                </Link>
                <Link to="/docs/execution/choosing-the-right-capability">
                  Choose Exec, a toolchain, or an agent
                </Link>
                <Link to="/docs/execution/local-api-cli-and-mcp">Run the first operation</Link>
              </nav>
            </div>
          </div>
        </section>

        <section className={styles.capabilities} aria-labelledby="capabilities-title">
          <div className="container">
            <div className={styles.sectionHeading}>
              <p className={styles.eyebrow}>Core capabilities</p>
              <h2 id="capabilities-title">A deterministic path from evidence to operation.</h2>
            </div>
            <div className={styles.cardGrid}>
              {capabilities.map(capability => (
                <Link className={styles.capabilityCard} key={capability.title} to={capability.to}>
                  <span className={styles.cardLabel}>{capability.label}</span>
                  <h3>{capability.title}</h3>
                  <p>{capability.description}</p>
                  <span className={styles.cardArrow} aria-hidden="true">
                    Explore →
                  </span>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.explore} aria-labelledby="explore-title">
          <div className="container">
            <div className={styles.sectionHeading}>
              <p className={styles.eyebrow}>Go deeper</p>
              <h2 id="explore-title">Start with the path that matches your work.</h2>
            </div>
            <div className={styles.exploreGrid}>
              {exploreLinks.map(item => (
                <Link className={styles.exploreLink} key={item.title} to={item.to}>
                  <strong>{item.title}</strong>
                  <span>{item.description}</span>
                </Link>
              ))}
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
