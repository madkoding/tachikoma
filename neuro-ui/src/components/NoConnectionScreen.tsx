import { useTranslation } from 'react-i18next';
import { WifiOff, RefreshCw } from 'lucide-react';

interface NoConnectionScreenProps {
  error?: string | null;
  onRetry?: () => void;
  isChecking?: boolean;
}

// ponytail: CRT canvas replaced with a static error card; props kept for compatibility
export function NoConnectionScreen({ error, onRetry, isChecking }: NoConnectionScreenProps) {
  const { t } = useTranslation();
  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-gray-900/95 p-4">
      <div className="max-w-md w-full bg-gray-900 border-2 border-red-500/50 rounded-lg shadow-2xl overflow-hidden">
        <div className="bg-red-600/90 px-6 py-4 flex items-center gap-3">
          <WifiOff className="w-8 h-8 text-white" />
          <div>
            <h2 className="text-xl font-bold text-white tracking-wide">
              {t('connection.noSignal', 'SIN SEÑAL')}
            </h2>
            <p className="text-red-200 text-sm">
              {t('connection.checkConnection', 'Verificando conexión...')}
            </p>
          </div>
        </div>
        <div className="p-6 space-y-4">
          {error && (
            <div className="bg-red-900/30 border border-red-500/30 rounded-lg p-3">
              <p className="text-red-300 text-sm font-mono">{error}</p>
            </div>
          )}
          <p className="text-gray-400 text-sm">
            {t('connection.tryingToReconnect', 'El sistema está intentando reconectarse automáticamente. Si el problema persiste, verifica que los servicios estén ejecutándose.')}
          </p>
          <button
            onClick={onRetry}
            disabled={isChecking}
            className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-red-600 hover:bg-red-500 disabled:bg-red-800 disabled:cursor-not-allowed text-white rounded-lg transition-colors font-medium"
          >
            <RefreshCw className={`w-5 h-5 ${isChecking ? 'animate-spin' : ''}`} />
            {isChecking ? t('connection.checking', 'Verificando...') : t('connection.retry', 'Reintentar conexión')}
          </button>
        </div>
      </div>
    </div>
  );
}