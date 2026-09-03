import React from 'react';

export default function DocsIndexPage() {
  return (
    <div className="space-y-8 py-6">
      <div>
        <h1 className="text-3xl font-extrabold text-white">API Reference & Overview</h1>
        <p className="text-slate-400 mt-2">Documentation and integration guide for Stellar Primitives smart contracts.</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <a href="/docs/vesting" className="glass-panel p-6 rounded-xl hover:border-indigo-500/50 transition-all">
          <h2 className="text-lg font-bold text-white mb-2">Vesting & Escrow Contract</h2>
          <p className="text-slate-400 text-sm">Linear cliff and schedule calculations for token vesting on Soroban.</p>
        </a>

        <a href="/docs/multisig" className="glass-panel p-6 rounded-xl hover:border-indigo-500/50 transition-all">
          <h2 className="text-lg font-bold text-white mb-2">Multisig Threshold Vault</h2>
          <p className="text-slate-400 text-sm">N-of-M multi-party approval and execution workflow.</p>
        </a>

        <a href="/docs/splitter" className="glass-panel p-6 rounded-xl hover:border-indigo-500/50 transition-all">
          <h2 className="text-lg font-bold text-white mb-2">Revenue Splitter</h2>
          <p className="text-slate-400 text-sm">Basis points payment distribution compatible with Drips streaming.</p>
        </a>

        <a href="/docs/access-control" className="glass-panel p-6 rounded-xl hover:border-indigo-500/50 transition-all">
          <h2 className="text-lg font-bold text-white mb-2">Access Control</h2>
          <p className="text-slate-400 text-sm">Role assignment (Admin, Minter, Operator) and authentication checks.</p>
        </a>
      </div>
    </div>
  );
}
