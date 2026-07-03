import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

export default function Home() {
  return (
    <Layout
      title="Prometheus Skill Pack"
      description="Enterprise-grade AI skills, P2P sync, and the Feynman learning engine"
    >
      <main>
        <section style={{ padding: '4rem 0', textAlign: 'center' }}>
          <h1>Prometheus Skill Pack</h1>
          <p>Enterprise-grade AI skills, P2P sync, and the Feynman learning engine</p>
          <div
            style={{ display: 'flex', gap: '1rem', justifyContent: 'center', marginTop: '2rem' }}
          >
            <Link className="button button--primary button--lg" to="/docs/guide/introduction">
              Get Started
            </Link>
            <Link className="button button--secondary button--lg" to="/docs/learn/overview">
              Learn Domain
            </Link>
            <Link
              className="button button--secondary button--lg"
              to="/docs/sovereign-sync/overview"
            >
              Sovereign Sync
            </Link>
          </div>
        </section>
      </main>
    </Layout>
  );
}
