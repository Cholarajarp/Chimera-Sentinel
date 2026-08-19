'use client';

import { ThemeProvider as NextThemesProvider } from 'next-themes';
import type { ThemeProviderProps } from 'next-themes/dist/types';

// Sentinel uses CSS custom properties scoped to the `.light` class on <html>.
// next-themes applies the chosen theme class to <html> via attribute="class".
// defaultTheme="dark" means no class is added in dark mode (the :root vars are dark),
// and the "light" class is added when the user selects light — matching our CSS.
export function ThemeProvider({ children, ...props }: ThemeProviderProps) {
  return (
    <NextThemesProvider
      attribute="class"
      defaultTheme="dark"
      enableSystem={false}
      disableTransitionOnChange
      {...props}
    >
      {children}
    </NextThemesProvider>
  );
}
