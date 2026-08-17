import { useToastStore, Toast } from '../../stores/toastStore';
import clsx from 'clsx';

const TYPE_STYLES: Record<Toast['type'], string> = {
  info: 'border-cyber-cyan text-cyber-cyan',
  success: 'border-cyber-green text-cyber-green',
  warning: 'border-cyber-yellow text-cyber-yellow',
  error: 'border-cyber-red text-cyber-red',
};

export default function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2 w-80 max-w-[calc(100vw-2rem)]">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          role="status"
          className={clsx(
            'cyber-card px-3 py-2 border bg-cyber-bg shadow-lg',
            TYPE_STYLES[toast.type]
          )}
        >
          <div className="flex items-start justify-between gap-2">
            <span className="text-sm break-words">{toast.message}</span>
            <button
              onClick={() => dismiss(toast.id)}
              className="text-xs opacity-60 hover:opacity-100 shrink-0"
              aria-label="Dismiss"
            >
              ✕
            </button>
          </div>
          {toast.actionLabel && (
            <button className="text-xs underline mt-1 block">{toast.actionLabel}</button>
          )}
          {typeof toast.progress === 'number' && (
            <div className="mt-2 h-1 w-full bg-black/40 rounded overflow-hidden">
              <div
                className="h-full bg-cyber-cyan transition-all"
                style={{ width: `${Math.min(100, Math.max(0, toast.progress))}%` }}
              />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
