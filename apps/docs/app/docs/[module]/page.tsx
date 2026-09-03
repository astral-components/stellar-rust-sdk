import React from 'react';

const moduleDocs: Record<string, { title: string; crate: string; description: string; functions: { name: string; args: string; returns: string; desc: string }[] }> = {
  vesting: {
    title: 'Linear & Cliff Token Vesting',
    crate: 'contracts/vesting',
    description: 'Manages token vesting schedules with optional cliff periods and proportional unlocking calculations.',
    functions: [
      { name: 'create_schedule', args: 'sender: Address, beneficiary: Address, token: Address, total_amount: i128, start_time: u64, cliff_time: u64, end_time: u64', returns: 'u64', desc: 'Creates a new linear vesting schedule and transfers tokens into escrow.' },
      { name: 'get_vested_amount', args: 'schedule_id: u64', returns: 'i128', desc: 'Calculates the current unlocked vested token amount based on current ledger timestamp.' },
      { name: 'claim', args: 'beneficiary: Address, schedule_id: u64', returns: 'i128', desc: 'Transfers unlocked tokens from contract escrow to beneficiary.' },
    ],
  },
  multisig: {
    title: 'Multi-Party Threshold Vault',
    crate: 'contracts/multisig',
    description: 'Requires N out of M signature approvals from configured vault owners before executing transaction proposals.',
    functions: [
      { name: 'init', args: 'owners: Vec<Address>, threshold: u32', returns: 'void', desc: 'Initializes the vault with owners list and required approval threshold.' },
      { name: 'create_proposal', args: 'proposer: Address, target: Address, amount: i128', returns: 'u64', desc: 'Creates a proposal for multi-sig vault approval.' },
      { name: 'approve_proposal', args: 'owner: Address, proposal_id: u64', returns: 'void', desc: 'Registers approval from an owner for a pending proposal.' },
      { name: 'execute_proposal', args: 'caller: Address, proposal_id: u64', returns: 'void', desc: 'Executes proposal once approval threshold is met.' },
    ],
  },
  splitter: {
    title: 'Revenue & Fee Splitter (Drips Compatible)',
    crate: 'contracts/splitter',
    description: 'Distributes incoming Stellar assets directly to multiple recipients according to basis points (BPS) allocations.',
    functions: [
      { name: 'init', args: 'admin: Address, shares: Vec<Share>', returns: 'void', desc: 'Sets up recipient allocations (shares must sum to 10,000 BPS).' },
      { name: 'distribute', args: 'sender: Address, token: Address, amount: i128', returns: 'void', desc: 'Transfers and splits amount directly among configured recipients.' },
      { name: 'get_shares', args: '', returns: 'Vec<Share>', desc: 'Returns current recipient share configuration.' },
    ],
  },
  'access-control': {
    title: 'Granular Role-Based Access Control',
    crate: 'contracts/access-control',
    description: 'Provides standardized role management (Admin, Minter, Operator) for Soroban applications.',
    functions: [
      { name: 'init', args: 'admin: Address', returns: 'void', desc: 'Initializes contract and grants Admin role.' },
      { name: 'grant_role', args: 'caller: Address, account: Address, role: Symbol', returns: 'void', desc: 'Grants role to an account (requires admin auth).' },
      { name: 'revoke_role', args: 'caller: Address, account: Address, role: Symbol', returns: 'void', desc: 'Revokes role from an account (requires admin auth).' },
      { name: 'has_role', args: 'account: Address, role: Symbol', returns: 'bool', desc: 'Queries if account possesses specified role.' },
    ],
  },
};

export default function ModuleDocPage({ params }: { params: { module: string } }) {
  const doc = moduleDocs[params.module] || {
    title: `Module: ${params.module}`,
    crate: `contracts/${params.module}`,
    description: 'Soroban smart contract primitive specification.',
    functions: [],
  };

  return (
    <div className="space-y-8 py-6 max-w-4xl">
      <div className="space-y-3 border-b border-slate-800 pb-6">
        <div className="flex items-center space-x-3">
          <span className="text-xs font-mono text-indigo-400 bg-indigo-500/10 px-2.5 py-1 rounded">
            {doc.crate}
          </span>
          <span className="text-xs text-slate-500">• soroban-sdk = "20.0.0"</span>
        </div>
        <h1 className="text-4xl font-bold text-white">{doc.title}</h1>
        <p className="text-slate-400 leading-relaxed text-base">{doc.description}</p>
      </div>

      <div className="space-y-6">
        <h2 className="text-2xl font-bold text-white">Smart Contract API Methods</h2>

        <div className="space-y-4">
          {doc.functions.map((fn) => (
            <div key={fn.name} className="glass-panel p-5 rounded-xl space-y-2 border border-slate-800">
              <div className="flex items-center justify-between font-mono text-sm">
                <span className="text-indigo-300 font-semibold">{fn.name}</span>
                <span className="text-slate-500 text-xs">→ {fn.returns}</span>
              </div>
              <p className="text-xs font-mono text-slate-400 bg-slate-950/60 p-2.5 rounded border border-slate-800/80 overflow-x-auto">
                fn {fn.name}({fn.args}) -&gt; {fn.returns};
              </p>
              <p className="text-xs text-slate-400 pt-1">{fn.desc}</p>
            </div>
          ))}
        </div>
      </div>

      <div className="glass-panel p-6 rounded-xl space-y-4 border border-indigo-500/20">
        <h3 className="text-lg font-bold text-white">TypeScript SDK Integration</h3>
        <pre className="text-xs font-mono text-indigo-200 bg-slate-950/90 p-4 rounded-xl overflow-x-auto border border-slate-800">
{`import { ${params.module.charAt(0).toUpperCase() + params.module.slice(1).replace('-c', 'C')}Client } from '@stellar-primitives/sdk';

const client = new ${params.module.charAt(0).toUpperCase() + params.module.slice(1).replace('-c', 'C')}Client({
  contractId: 'C...',
  rpcUrl: 'https://soroban-testnet.stellar.org',
});`}
        </pre>
      </div>
    </div>
  );
}
