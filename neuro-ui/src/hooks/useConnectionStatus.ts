import { useState, useEffect, useCallback } from 'react';
import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || '/api';
const HEALTH_CHECK_INTERVAL = 5000; // Check every 5 seconds
const INITIAL_CHECK_DELAY = 1000; // Wait 1 second before first check

export interface ConnectionStatus {
  isConnected: boolean;
  isChecking: boolean;
  lastCheck: Date | null;
  error: string | null;
  dbConnected: boolean;
}

export function useConnectionStatus() {
  const [status, setStatus] = useState<ConnectionStatus>({
    isConnected: true, // Assume connected initially to avoid flash
    isChecking: true,
    lastCheck: null,
    error: null,
    dbConnected: true,
  });

  const checkConnection = useCallback(async () => {
    setStatus(prev => ({ ...prev, isChecking: true }));
    
    try {
      const response = await axios.get(`${API_BASE_URL}/health`, {
        timeout: 5000,
      });
      
      // Check if database is connected from health response
      // Backend returns: { status: "healthy"|"degraded"|"unhealthy", services: { database: "healthy"|"unhealthy" } }
      const data = response.data;
      const dbConnected = data?.services?.database === 'healthy';
      const isHealthy = data?.status === 'healthy' || data?.status === 'degraded';
      
      setStatus({
        isConnected: isHealthy,
        isChecking: false,
        lastCheck: new Date(),
        error: isHealthy ? null : 'Servicios no disponibles',
        dbConnected,
      });
    } catch (error) {
      let errorMessage = 'No se puede conectar con el servidor';
      
      if (axios.isAxiosError(error)) {
        if (error.code === 'ECONNABORTED') {
          errorMessage = 'Tiempo de espera agotado';
        } else if (error.code === 'ERR_NETWORK') {
          errorMessage = 'Error de red - servidor no disponible';
        } else if (error.response) {
          errorMessage = `Error del servidor: ${error.response.status}`;
        }
      }
      
      setStatus({
        isConnected: false,
        isChecking: false,
        lastCheck: new Date(),
        error: errorMessage,
        dbConnected: false,
      });
    }
  }, []);

  const retry = useCallback(() => {
    checkConnection();
  }, [checkConnection]);

  useEffect(() => {
    // Initial check after a short delay
    const initialTimeout = setTimeout(() => {
      checkConnection();
    }, INITIAL_CHECK_DELAY);

    // Periodic health checks
    const interval = setInterval(() => {
      checkConnection();
    }, HEALTH_CHECK_INTERVAL);

    return () => {
      clearTimeout(initialTimeout);
      clearInterval(interval);
    };
  }, [checkConnection]);

  return { ...status, retry };
}
