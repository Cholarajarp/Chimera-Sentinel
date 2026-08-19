import React from 'react';
import Image from 'next/image';
import { Download } from 'lucide-react';
import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';

export type TabType =
  | 'fleet'
  | 'candidate'
  | 'certification'
  | 'corpus'
  | 'approval'
  | 'evidence'
  | 'attestation'
  | 'trace'
  | 'policy'
  | 'vulnerability';

export const TABS: Array<{ value: TabType; label: string }> = [
  { value: 'fleet', label: 'Fleet Posture' },
  { value: 'candidate', label: 'Candidate ABOM' },
  { value: 'certification', label: 'Certification Timeline' },
  { value: 'corpus', label: '80-Case Matrix' },
  { value: 'approval', label: 'Reviewer Approval' },
  { value: 'evidence', label: 'Evidence Manifest' },
  { value: 'attestation', label: 'KMS Attestation Verifier' },
  { value: 'trace', label: 'Cloud Trace Correlation' },
  { value: 'policy', label: 'Dynamic Policy Config' },
  { value: 'vulnerability', label: 'Vulnerability Scanner' },
];

interface SidebarNavProps {
  activeTab: string;
  setActiveTab: (tab: any) => void;
  handleExportReport: () => void;
}

export function SidebarNav({ activeTab, setActiveTab, handleExportReport }: SidebarNavProps) {
  return (
    <aside className="w-[280px] flex-shrink-0 flex flex-col border-r border-slate-800 bg-[#09090b] z-50">
      <div className="h-20 flex items-center gap-4 px-6 border-b border-slate-800">
        <div className="relative w-10 h-10 flex items-center justify-center rounded-xl bg-gradient-to-br from-emerald-500/20 to-emerald-500/0 border border-emerald-500/20 shadow-[0_0_15px_rgba(16,185,129,0.2)]">
          <Image src="/chimera-sentinel.png" alt="" width={24} height={24} priority className="drop-shadow-lg" />
        </div>
        <div className="flex flex-col">
          <span className="font-display font-semibold tracking-wide text-sm text-white drop-shadow-md">Chimera Sentinel</span>
          <span className="text-[10px] font-mono text-emerald-400 uppercase tracking-widest">Admission Control</span>
        </div>
      </div>

      <nav className="flex-1 py-6 px-4 space-y-1 overflow-y-auto custom-scrollbar">
        {TABS.map(tab => (
          <button
            key={tab.value}
            id={`tab-${tab.value}`}
            onClick={() => setActiveTab(tab.value)}
            className={cn(
              "w-full flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-all duration-200 relative group",
              activeTab === tab.value 
                ? "text-emerald-300 bg-emerald-500/10 border border-emerald-500/20 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]" 
                : "text-slate-400 hover:text-slate-200 hover:bg-slate-800/50"
            )}
          >
            {activeTab === tab.value && (
              <motion.div layoutId="activeTabIndicator" className="absolute left-0 w-1 h-6 bg-emerald-400 rounded-r-full shadow-[0_0_10px_rgba(52,211,153,0.8)]" />
            )}
            {tab.label}
          </button>
        ))}
      </nav>

      <div className="p-4 border-t border-slate-800 flex flex-col gap-3">
        <div className="flex items-center justify-between px-2">
           <button onClick={handleExportReport} className="p-2 rounded-lg text-slate-400 hover:text-emerald-400 hover:bg-emerald-500/10 transition-colors" title="Export Report"><Download size={16} /></button>
        </div>
        <button
           onClick={async () => {
              document.cookie = 'sentinel-session=; path=/; max-age=0; SameSite=Strict';
              window.sessionStorage.removeItem('sentinel-experience');
              window.location.assign('/signin');
           }}
           className="w-full py-2.5 rounded-lg border border-slate-800 bg-[#111113] text-slate-400 text-xs font-medium uppercase tracking-wider hover:bg-rose-500/10 hover:text-rose-400 hover:border-rose-500/20 transition-all duration-200"
        >
          Sign Out
        </button>
      </div>
    </aside>
  );
}
