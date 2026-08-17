import { useState, useEffect } from 'react';
import clsx from 'clsx';
import { healthApi, HealthResponse } from '../../api/health';

const POLL_MS = 5000;

export default function StatusBar() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [online, setOnline] = useState(true);

  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;

    const poll = async () => {
      try {
        const h = await healthApi.check();
        if (!cancelled) {
          setHealth(h);
          setOnline(true);
        }
      } catch {
        if (!cancelled) setOnline(false);
      } finally {
        if (!cancelled) timer = setTimeout(poll, POLL_MS);
      }
    };

    poll();
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, []);

  const dbHealthy = health?.services.database === 'healthy';
  const llmHealthy = health?.services.llm === 'healthy';

  return (
    <div className="flex items-center gap-3 px-4 py-1.5 text-[10px] font-mono border-t border-black/40 bg-cyber-surface/80 text-cyber-cyan/60">
      <Dot on={online} label={online ? 'ONLINE' : 'OFFLINE'} />
      <Dot on={dbHealthy} label="DB" />
      <Dot on={llmHealthy} label="LLM" />
      <span className="ml-auto">
        v{health?.version ?? '—'}
      </span>
    </div>
  );
}

function Dot({ on, label }: { on: boolean; label: string }) {
  return (
    <span className="flex items-center gap-1.5">
      <span
        className={clsx(
          'w-1.5 h-1.5 rounded-full',
          on ? 'bg-cyber-green shadow-[0_0_6px_rgba(0,255,0,0.8)]' : 'bg-cyber-red'
        )}
      />
      {label}
    </span>
  );
}
