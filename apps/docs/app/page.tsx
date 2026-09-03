import React from 'react';

export default function HomePage() {
  const modules = [
    {
      name: 'Vesting & Escrow',
      slug: 'vesting',
      description: 'Linear and cliff token vesting schedule engine with automated escrow for Stellar tokens.',
      badge: 'DeFi Primitives',
    },
    {
      name: 'Multi-Party Vault',
      slug: 'multisig',
      description: 'Threshold multi-signature vault contract with proposal creation, approval tracking, and execution.',
      badge: 'Security & Multisig',
    },
    {
      name: 'Revenue Splitter',
      slug: 'splitter',
      description: 'Continuous fee and revenue distribution contract compatible with Drips streaming rules.',
      badge: 'Drips Compatible',
    },
    {
      name: 'Access Control',
      slug: 'access-control',
      description: 'Fine-grained Role-Based Access Control (Admin, Minter, Operator) for Soroban applications.',
      badge: 'Governance',
    },
  ];

  return (
    <div className="space-y-16 py-8">
      {/* Hero Section */}
      <section className="text-center space-y-6 max-w-3xl mx-auto py-12">
        <div className="inline-flex items-center space-x-2 bg-indigo-500/10 border border-indigo-500/20 text-indigo-400 text-xs px-3 py-1.5 rounded-full font-medium">
          <span>Stellar Primitives Core Library</span>
          <span>•</span>
          <span>Soroban SDK v20.0.0</span>
        </div>

        <h1 className="text-5xl font-extrabold tracking-tight sm:text-6xl text-white leading-tight">
          Production-grade, modular Rust building blocks for{' '}
          <span className="bg-gradient-to-r from-indigo-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
            Soroban smart contracts
          </span>
        </h1>

        <p className="text-lg text-slate-400 leading-relaxed">
          Open-source, secure, <code className="text-indigo-300 font-mono bg-slate-800/80 px-1.5 py-0.5 rounded">#![no_std]</code> Rust smart contracts on Stellar. Complete with TypeScript SDK, deployment tools, and contributor bounties on Grantfox & Drips.
        </p>

        <div className="flex items-center justify-center space-x-4 pt-4">
          <a
            href="/docs"
            className="bg-indigo-600 hover:bg-indigo-500 text-white font-semibold px-6 py-3 rounded-xl shadow-lg shadow-indigo-500/25 transition-all text-sm"
          >
            Explore API Docs
          </a>
          <a
            href="https://github.com/stellar-primitives/soroban-contracts"
            target="_blank"
            rel="noreferrer"
            className="glass-panel text-slate-300 hover:text-white font-medium px-6 py-3 rounded-xl transition-all text-sm"
          >
            View on GitHub
          </a>
        </div>
      </section>

      {/* Module Grid */}
      <section className="space-y-6">
        <div className="text-center">
          <h2 className="text-2xl font-bold text-white">Modular Smart Contracts</h2>
          <p className="text-slate-400 text-sm mt-1">Audited, reusable Soroban building blocks ready for production deployment.</p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {modules.map((mod) => (
            <a
              key={mod.slug}
              href={`/docs/${mod.slug}`}
              className="glass-panel p-6 rounded-2xl border border-slate-800/80 hover:border-indigo-500/50 transition-all group block space-y-4"
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-semibold uppercase tracking-wider text-indigo-400 bg-indigo-500/10 px-2.5 py-1 rounded-md">
                  {mod.badge}
                </span>
                <span className="text-xs text-slate-500 group-hover:text-indigo-400 transition-colors">
                  Read API →
                </span>
              </div>
              <h3 className="text-xl font-semibold text-white group-hover:text-indigo-300 transition-colors">
                {mod.name}
              </h3>
              <p className="text-slate-400 text-sm leading-relaxed">
                {mod.description}
              </p>
            </a>
          ))}
        </div>
      </section>

      {/* Ecosystem Bounties & Funding */}
      <section className="glass-panel p-8 rounded-3xl border border-indigo-500/20 bg-gradient-to-br from-indigo-950/20 via-slate-900/40 to-slate-950/60 flex flex-col md:flex-row items-center justify-between gap-8">
        <div className="space-y-3 max-w-xl">
          <h3 className="text-2xl font-bold text-white">Contribute & Earn via Grantfox & Drips</h3>
          <p className="text-slate-400 text-sm leading-relaxed">
            All primitives are community-driven. Submit code improvements or complete open bounty issues to earn Stellar XLM/USDC directly to your wallet.
          </p>
        </div>
        <div className="flex items-center space-x-4 shrink-0">
          <span className="bg-purple-500/10 border border-purple-500/30 text-purple-300 text-xs font-semibold px-4 py-2 rounded-xl">
            Grantfox Bounties Active
          </span>
          <span className="bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 text-xs font-semibold px-4 py-2 rounded-xl">
            Drips Funded
          </span>
        </div>
      </section>
    </div>
  );
}
