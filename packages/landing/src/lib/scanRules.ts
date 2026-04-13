import scanSchema from '../../public/scan/schema.json';

export type ScanRuleCategory = 'core' | 'react' | 'nextjs' | 'react-native' | 'server';

export interface ScanRuleEntry {
  id: string;
  description: string;
}

export interface ScanRuleGroup {
  category: ScanRuleCategory;
  label: string;
  rules: ScanRuleEntry[];
}

const TAB_ORDER: { id: ScanRuleCategory; label: string }[] = [
  { id: 'core', label: 'Core' },
  { id: 'react', label: 'React' },
  { id: 'nextjs', label: 'Next.js' },
  { id: 'react-native', label: 'React Native' },
  { id: 'server', label: 'Server' }
];

export function categorizeRuleId(id: string): ScanRuleCategory {
  if (id.startsWith('react-native/')) return 'react-native';
  if (id.startsWith('react/')) return 'react';
  if (id.startsWith('nextjs/')) return 'nextjs';
  if (id.startsWith('server/')) return 'server';
  return 'core';
}

/** Built-in rules from public/scan/schema.json, grouped for the Scan docs page. */
export function getGroupedScanRules(): ScanRuleGroup[] {
  const rulesProps = (
    scanSchema as {
      properties?: { rules?: { properties?: Record<string, { description?: string }> } };
    }
  ).properties?.rules?.properties;

  const byCategory: Record<ScanRuleCategory, ScanRuleEntry[]> = {
    core: [],
    react: [],
    nextjs: [],
    'react-native': [],
    server: []
  };

  if (!rulesProps) return [];

  for (const id of Object.keys(rulesProps).sort()) {
    const description = rulesProps[id]?.description ?? '';
    byCategory[categorizeRuleId(id)].push({ id, description });
  }

  return TAB_ORDER.filter((t) => byCategory[t.id].length > 0).map((t) => ({
    category: t.id,
    label: t.label,
    rules: byCategory[t.id]
  }));
}
