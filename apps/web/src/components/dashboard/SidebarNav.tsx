import React from 'react';
import Image from 'next/image';
import {
  Download,
  Layers,
  GitBranch,
  FlaskConical,
  CheckSquare,
  FileText,
  KeyRound,
  Activity,
  Settings2,
  ScanSearch,
  LayoutDashboard,
  type LucideProps,
} from 'lucide-react';
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

export const TABS: Array<{ value: TabType; label: string; icon: React.ForwardRefExoticComponent<Omit<LucideProps, 'ref'> & React.RefAttributes<SVGSVGElement>> }> = [
  { value: 'fleet',         label: 'Fleet Posture',          icon: LayoutDashboard },
  { value: 'candidate',     label: 'Candidate ABOM',         icon: Layers },
  { value: 'certification', label: 'Certification Timeline', icon: GitBranch },
  { value: 'corpus',        label: '80-Case Matrix',         icon: FlaskConical },
  { value: 'approval',      label: 'Reviewer Approval',      icon: CheckSquare },
  { value: 'evidence',      label: 'Evidence Manifest',      icon: FileText },
  { value: 'attestation',   label: 'KMS Attestation',        icon: KeyRound },
  { value: 'trace',         label: 'Cloud Trace',            icon: Activity },
  { value: 'policy',        label: 'Policy Config',          icon: Settings2 },
  { value: 'vulnerability', label: 'Vulnerability Scan',     icon: ScanSearch },
];

interface SidebarNavProps {
  activeTab: string;
  setActiveTab: (tab: any) => void;
  handleExportReport: () => void;
}

export function SidebarNav({ activeTab, setActiveTab, handleExportReport }: SidebarNavProps) {
  return (
    <aside className="w-[260px] flex-shrink-0 flex flex-col border-r border-[var(--border-subtle)] bg-[#06070a] z-50">
      {/* Brand */}
      <div className="h-[72px] flex items-center gap-3 px-5 border-b border-[var(--border-subtle)]">
        <div className="relative w-9 h-9 flex items-center justify-center rounded-lg bg-[#0d0f14] border border-[rgba(0,228,155,0.2)] shadow-[0_0_12px_rgba(0,228,155,0.12)]">
          <Image src="/chimera-sentinel.png" alt="" width={22} height={22} priority className="drop-shadow-lg" />
        </div>
        <div className="flex flex-col min-w-0">
          <span className="font-semibold tracking-tight text-[13px] text-white leading-tight">Chimera Sentinel</span>
          <span className="text-[9px] font-mono font-bold text-[#00e49b] uppercase tracking-[0.14em] leading-tight mt-0.5">Admission Control</span>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 py-4 px-3 space-y-0.5 overflow-y-auto custom-scrollbar">
        {/* Section label */}
        <div className="px-3 pb-2 pt-1">
          <span className="text-[9px] font-mono font-bold text-[#4e596b] uppercase tracking-[0.12em]">Navigation</span>
        </div>

        {TABS.map(tab => {
          const Icon = tab.icon;
          const isActive = activeTab === tab.value;
          return (
            <button
              key={tab.value}
              id={`tab-${tab.value}`}
              onClick={() => setActiveTab(tab.value)}
              className={cn(
                "w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg text-[12.5px] font-medium transition-all duration-200 relative group text-left",
                isActive
                  ? "text-[#00e49b] bg-[rgba(0,228,155,0.08)] border border-[rgba(0,228,155,0.18)]"
                  : "text-[#8b9cb8] hover:text-[#f1f5f9] hover:bg-[rgba(255,255,255,0.04)] border border-transparent"
              )}
            >
              {isActive && (
                <motion.div
                  layoutId="activeTabIndicator"
                  className="absolute left-0 top-1/2 -translate-y-1/2 w-[2px] h-5 bg-[#00e49b] rounded-r-full"
                  style={{ boxShadow: '0 0 6px rgba(0,228,155,0.35)' }}
                />
              )}
              <Icon
                size={14}
                className={cn(
                  "flex-shrink-0 transition-colors duration-200",
                  isActive ? "text-[#00e49b]" : "text-[#4e596b] group-hover:text-[#8b9cb8]"
                )}
              />
              <span className="truncate">{tab.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="p-3 border-t border-[var(--border-subtle)] flex flex-col gap-2">
        <button
          onClick={handleExportReport}
          className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-[12px] font-medium text-[#8b9cb8] hover:text-[#f1f5f9] hover:bg-[rgba(255,255,255,0.04)] border border-transparent hover:border-[var(--border-subtle)] transition-all duration-200"
        >
          <Download size={13} className="flex-shrink-0" />
          <span>Export Report</span>
        </button>
        <button
          onClick={async () => {
            document.cookie = 'sentinel-session=; path=/; max-age=0; SameSite=Strict';
            window.sessionStorage.removeItem('sentinel-experience');
            window.location.assign('/signin');
          }}
          className="w-full py-2 rounded-lg border border-[var(--border-subtle)] bg-transparent text-[#4e596b] text-[11px] font-medium uppercase tracking-wider hover:bg-[rgba(251,75,110,0.08)] hover:text-[#fda4af] hover:border-[rgba(251,75,110,0.2)] transition-all duration-200"
        >
          Sign Out
        </button>
      </div>
    </aside>
  );
}
