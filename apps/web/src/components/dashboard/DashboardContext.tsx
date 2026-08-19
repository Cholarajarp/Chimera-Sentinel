import React, { createContext, useContext } from 'react';

// Using 'any' to simply bypass deep typing requirements for the massive prop tree
// while we deconstruct the prototype.
export const DashboardContext = createContext<any>(null);

export function DashboardProvider({ children, value }: { children: React.ReactNode, value: any }) {
  return <DashboardContext.Provider value={value}>{children}</DashboardContext.Provider>;
}

export function useDashboard() {
  return useContext(DashboardContext);
}
