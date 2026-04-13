import { useEffect, useRef, useState } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { getGroupedScanRules } from '@/lib/scanRules';
import { cn } from '@/lib/utils';

const groupedScanRules = getGroupedScanRules();
const defaultRulesTab = groupedScanRules[0]?.category ?? 'core';

export const ScanRulesContent = () => {
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const copyResetRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const copyRuleId = async (id: string) => {
    try {
      await navigator.clipboard.writeText(id);
      setCopiedId(id);
      if (copyResetRef.current) clearTimeout(copyResetRef.current);
      copyResetRef.current = setTimeout(() => {
        setCopiedId(null);
        copyResetRef.current = null;
      }, 2000);
    } catch {
      // Clipboard unavailable
    }
  };

  useEffect(() => {
    return () => {
      if (copyResetRef.current) clearTimeout(copyResetRef.current);
    };
  }, []);

  return (
    <>
      <p className="text-sm text-stone-500 leading-relaxed">
        Built-in rules from the schema. Configure severities under{' '}
        <span className="text-stone-300">rules</span> in <span className="text-stone-300">flick.json</span>.
      </p>
      <Tabs defaultValue={defaultRulesTab} className="w-full">
        <TabsList className="h-auto w-full flex-wrap justify-start gap-1 overflow-x-auto rounded-md border border-stone-800 bg-stone-950/30 p-1 text-stone-400">
          {groupedScanRules.map((group) => (
            <TabsTrigger
              key={group.category}
              value={group.category}
              className="shrink-0 text-stone-400 data-[state=active]:bg-stone-900 data-[state=active]:text-stone-200 data-[state=active]:shadow-none"
            >
              {group.label}
            </TabsTrigger>
          ))}
        </TabsList>
        {groupedScanRules.map((group) => (
          <TabsContent key={group.category} value={group.category} className="mt-3">
            <div className="flex max-h-[min(28rem,70vh)] flex-col gap-4 overflow-y-auto rounded border border-stone-800 p-4">
              {group.rules.map((rule) => (
                <div key={rule.id} className="flex flex-col gap-1">
                  <code className="font-mono text-sm">
                    <span
                      role="button"
                      tabIndex={0}
                      title="Click to copy rule id"
                      className={cn(
                        'inline-flex max-w-full cursor-pointer flex-wrap items-baseline gap-2 rounded px-0.5 -mx-0.5 text-left text-stone-300 outline-none transition-colors',
                        'hover:text-stone-100 hover:underline decoration-stone-600 underline-offset-2',
                        'focus-visible:ring-1 focus-visible:ring-stone-500'
                      )}
                      onClick={() => copyRuleId(rule.id)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' || e.key === ' ') {
                          e.preventDefault();
                          copyRuleId(rule.id);
                        }
                      }}
                    >
                      {rule.id}
                      {copiedId === rule.id ? (
                        <span className="font-sans text-xs font-normal text-emerald-500/90 no-underline">
                          Copied
                        </span>
                      ) : null}
                    </span>
                  </code>
                  <p className="text-sm leading-relaxed text-stone-500">{rule.description}</p>
                </div>
              ))}
            </div>
          </TabsContent>
        ))}
      </Tabs>
    </>
  );
};
