import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Translation resources
const resources = {
  en: {
    translation: {
      // Common
      'app.title': 'NEURO-OS',
      'app.subtitle': 'AI Assistant with Memory',
      'common.loading': 'Loading...',
      'common.error': 'An error occurred',
      'common.retry': 'Retry',
      'common.cancel': 'Cancel',
      'common.save': 'Save',
      'common.delete': 'Delete',
      'common.search': 'Search',
      'common.settings': 'Settings',
      
      // Chat
      'chat.placeholder': 'Type your message...',
      'chat.send': 'Send',
      'chat.thinking': 'Thinking...',
      'chat.newConversation': 'New Conversation',
      'chat.noMessages': 'Start a conversation',
      'chat.noMessagesDesc': 'Send a message to begin chatting with NEURO-OS',
      'chat.welcome': 'Hello! I\'m NEURO-OS, your AI assistant with memory.',
      'chat.welcomeDesc': 'I remember our previous conversations and can help you with various tasks.',
      
      // Features
      'feature.memory': 'LONG_TERM_MEMORY',
      'feature.memoryDesc': 'I remember our conversations and learn your preferences',
      'feature.search': 'WEB_SEARCH',
      'feature.searchDesc': 'Search the web for current information when needed',
      'feature.cmd': 'CMD_EXECUTE',
      'feature.cmdDesc': 'Run safe local commands to help with tasks',
      'feature.model': 'ADAPTIVE_MODEL',
      'feature.modelDesc': 'Automatically selects the best model for each task',
      
      // Message labels
      'message.user': 'USER',
      'message.assistant': 'TACHIKOMA',
      'message.model': 'MODEL',
      'message.tokens': 'TOKENS',
      'message.time': 'TIME',
      'message.speed': 'SPEED',
      
      // Status
      'status.online': 'ONLINE',
      'status.offline': 'OFFLINE',
      'header.defaultTitle': 'TACHIKOMA // NEURAL INTERFACE',
      
      // Sidebar
      'sidebar.conversations': 'Conversations',
      'sidebar.newChat': 'New Chat',
      'sidebar.today': 'Today',
      'sidebar.yesterday': 'Yesterday',
      'sidebar.older': 'Older',
      
      // Settings
      'settings.title': 'Settings',
      'settings.language': 'Language',
      'settings.theme': 'Theme',
      'settings.theme.light': 'Light',
      'settings.theme.dark': 'Dark',
      'settings.theme.system': 'System',
      
      // Errors
      'error.network': 'Network error. Please check your connection.',
      'error.server': 'Server error. Please try again later.',
      'error.unknown': 'An unknown error occurred.',
    },
  },
  es: {
    translation: {
      // Common
      'app.title': 'NEURO-OS',
      'app.subtitle': 'Asistente IA con Memoria',
      'common.loading': 'Cargando...',
      'common.error': 'Ha ocurrido un error',
      'common.retry': 'Reintentar',
      'common.cancel': 'Cancelar',
      'common.save': 'Guardar',
      'common.delete': 'Eliminar',
      'common.search': 'Buscar',
      'common.settings': 'Configuración',
      
      // Chat
      'chat.placeholder': 'Escribe tu mensaje...',
      'chat.send': 'Enviar',
      'chat.thinking': 'Pensando...',
      'chat.newConversation': 'Nueva Conversación',
      'chat.noMessages': 'Inicia una conversación',
      'chat.noMessagesDesc': 'Envía un mensaje para comenzar a chatear con NEURO-OS',
      'chat.welcome': '¡Hola! Soy NEURO-OS, tu asistente IA con memoria.',
      'chat.welcomeDesc': 'Recuerdo nuestras conversaciones anteriores y puedo ayudarte con diversas tareas.',
      
      // Features
      'feature.memory': 'MEMORIA_LARGO_PLAZO',
      'feature.memoryDesc': 'Recuerdo nuestras conversaciones y aprendo tus preferencias',
      'feature.search': 'BÚSQUEDA_WEB',
      'feature.searchDesc': 'Busco en la web información actualizada cuando es necesario',
      'feature.cmd': 'EJECUTAR_COMANDOS',
      'feature.cmdDesc': 'Ejecuto comandos locales seguros para ayudarte con tareas',
      'feature.model': 'MODELO_ADAPTATIVO',
      'feature.modelDesc': 'Selecciona automáticamente el mejor modelo para cada tarea',
      
      // Message labels
      'message.user': 'USUARIO',
      'message.assistant': 'TACHIKOMA',
      'message.model': 'MODELO',
      'message.tokens': 'TOKENS',
      'message.time': 'TIEMPO',
      'message.speed': 'VELOCIDAD',
      
      // Status
      'status.online': 'EN LÍNEA',
      'status.offline': 'DESCONECTADO',
      'header.defaultTitle': 'TACHIKOMA // INTERFAZ NEURAL',
      
      // Sidebar
      'sidebar.conversations': 'Conversaciones',
      'sidebar.newChat': 'Nuevo Chat',
      'sidebar.today': 'Hoy',
      'sidebar.yesterday': 'Ayer',
      'sidebar.older': 'Más antiguo',
      
      // Settings
      'settings.title': 'Configuración',
      'settings.language': 'Idioma',
      'settings.theme': 'Tema',
      'settings.theme.light': 'Claro',
      'settings.theme.dark': 'Oscuro',
      'settings.theme.system': 'Sistema',
      
      // Errors
      'error.network': 'Error de red. Por favor verifica tu conexión.',
      'error.server': 'Error del servidor. Por favor intenta más tarde.',
      'error.unknown': 'Ha ocurrido un error desconocido.',
    },
  },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    supportedLngs: ['en', 'es'],
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator', 'htmlTag'],
      caches: ['localStorage'],
    },
  });

export default i18n;
