'use client';

import { useTheme } from 'next-themes';
import { useEffect, useRef, useState } from 'react';
import { Sun, Moon, Monitor } from 'lucide-react';

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => setMounted(true), []);

  useEffect(() => {
    if (!open) return;

    const closeOutside = (event: MouseEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) setOpen(false);
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };

    document.addEventListener('mousedown', closeOutside);
    document.addEventListener('keydown', closeOnEscape);
    return () => {
      document.removeEventListener('mousedown', closeOutside);
      document.removeEventListener('keydown', closeOnEscape);
    };
  }, [open]);

  if (!mounted) {
    return (
      <button
        className="theme-toggle-btn"
        aria-label="Toggle theme"
      />
    );
  }

  const options: { value: string; icon: React.ReactNode; label: string }[] = [
    { value: 'light', icon: <Sun size={14} />, label: 'Light' },
    { value: 'dark', icon: <Moon size={14} />, label: 'Dark' },
    { value: 'system', icon: <Monitor size={14} />, label: 'System' },
  ];

  const current = options.find(o => o.value === theme) ?? options[1];

  return (
    <div className="relative theme-toggle-wrapper" ref={menuRef}>
      <button
        type="button"
        className="theme-toggle-btn"
        aria-label="Choose color theme"
        aria-expanded={open}
        aria-haspopup="menu"
        onClick={() => setOpen(currentOpen => !currentOpen)}
      >
        {current.icon}
        <span className="theme-toggle-label">{current.label}</span>
      </button>
      <div
        className={`theme-toggle-menu ${open ? 'is-open' : ''}`}
        role="menu"
      >
        {options.map(o => (
          <button
            key={o.value}
            type="button"
            role="menuitemradio"
            aria-checked={theme === o.value}
            onClick={() => {
              setTheme(o.value);
              setOpen(false);
            }}
            className={`theme-toggle-item ${theme === o.value ? 'is-active' : ''}`}
          >
            {o.icon}
            {o.label}
          </button>
        ))}
      </div>
    </div>
  );
}
