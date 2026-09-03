import './globals.css';
import React from 'react';

export const metadata = {
  title: 'Stellar Primitives — Soroban Smart Contracts Documentation',
  description: 'Production-grade, modular Rust building blocks for Soroban smart contracts on Stellar.',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="flex flex-col min-h-screen">
        <header className="sticky top-0 z-50 glass-panel border-b border-slate-800/80 px-6 py-4">
          <div className="max-w-7xl mx-auto flex items-center justify-between">
            <a href="/" className="flex items-center space-x-3">
              <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-500 to-purple-500 flex items-center justify-center font-bold text-white shadow-lg shadow-indigo-500/30">
                SP
              </div>
              <span className="font-semibold text-lg tracking-tight text-white">
                stellar-primitives <span className="text-xs font-normal text-indigo-400 border border-indigo-500/30 bg-indigo-500/10 px-2 py-0.5 rounded-full ml-1">v0.1.0</span>
              </span>
            </a>
            <nav className="flex items-center space-x-6 text-sm font-medium text-slate-300">
              <a href="/docs" className="hover:text-indigo-400 transition-colors">Documentation</a>
              <a href="/docs/vesting" className="hover:text-indigo-400 transition-colors">Vesting</a>
              <a href="/docs/multisig" className="hover:text-indigo-400 transition-colors">Multisig</a>
              <a href="/docs/splitter" className="hover:text-indigo-400 transition-colors">Splitter</a>
              <a href="/docs/access-control" className="hover:text-indigo-400 transition-colors">Access Control</a>
              <a
                href="https://github.com/stellar-primitives/soroban-contracts"
                target="_blank"
                rel="noreferrer"
                className="bg-slate-800 hover:bg-slate-700 text-white px-4 py-2 rounded-lg border border-slate-700 transition-all text-xs font-mono"
              >
                GitHub Org
              </a>
            </nav>
          </div>
        </header>

        <main className="flex-1 max-w-7xl w-full mx-auto px-6 py-8">
          {children}
        </main>

        <footer className="border-t border-slate-800/80 py-8 px-6 text-center text-xs text-slate-500">
          <p>© 2026 stellar-primitives. Funded via Grantfox & Drips Network. Built for Stellar Soroban.</p>
        </footer>
      </body>
    </html>
  );
}
