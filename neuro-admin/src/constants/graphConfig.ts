// Configuración centralizada del grafo 3D

export const GRAPH_CONFIG = {
  // Distribución de nodos
  nodes: {
    radius: { min: 300, max: 500 }, // Radio de dispersión de nodos
  },

  // Cámara
  camera: {
    initialDistance: 1500, // Distancia inicial de la cámara (debe ser ~3x el radio max)
    initialY: 0, // Altura inicial de la cámara (0 = centrado verticalmente)
    rotationSpeed: 0.001, // Velocidad de rotación automática
    resumeDelay: 5000, // Tiempo para reanudar rotación después de interacción (ms)
  },

  // Fuerzas de simulación D3
  forces: {
    link: {
      distance: 150,
      strength: 0.01,
    },
    charge: {
      strength: -800,
      distanceMax: 400,
    },
    center: {
      strength: 0.02,
    },
  },

  // Simulación
  simulation: {
    alphaDecay: 0.1,
    velocityDecay: 0.5,
    warmupTicks: 10,
    cooldownTicks: 10,
    cooldownTime: 0,
  },
};
