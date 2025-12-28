import { useState, useRef, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { useChatStore, Message, Conversation } from '../stores/chatStore';
import { useMusicStore } from '../stores/musicStore';
import { useNotificationStore } from '../stores/notificationStore';
import { chatApi, StreamCompleteResponse } from '../api/client';
import ChatMessage from '../components/ChatMessage';
import ChatInput from '../components/ChatInput';
import Sidebar from '../components/Sidebar';
import TypingIndicator from '../components/TypingIndicator';
import WelcomeScreen from '../components/WelcomeScreen';
import { useVoiceStream } from '../hooks/useVoiceStream';

// Helper to detect complete sentences for progressive voice synthesis
const SENTENCE_ENDINGS = /[.!?。！？]+\s*/g;

// Fallback UUID generator for non-secure contexts (HTTP)
function generateUUID(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  // Fallback for non-secure contexts
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export default function ChatPage() {
  const { t } = useTranslation();
  const { conversationId } = useParams<{ conversationId?: string }>();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [initialLoadDone, setInitialLoadDone] = useState(false);
  
  // Voice synthesis hook
  const { state: voiceState, config: voiceConfig, speak, stop: stopVoice, updateConfig: setVoiceConfig } = useVoiceStream();
  const lastSpokenMessageRef = useRef<string | null>(null);
  
  // For progressive voice synthesis during streaming
  const pendingTextRef = useRef<string>('');
  const spokenSentencesRef = useRef<Set<string>>(new Set());
  
  const {
    conversations,
    currentConversationId,
    isLoading,
    setCurrentConversation,
    addMessage,
    addConversation,
    setConversations,
    setLoading,
    setError,
  } = useChatStore();

  // Start with sidebar closed on mobile (< 1024px)
  const [isSidebarOpen, setIsSidebarOpen] = useState(() => {
    if (typeof window !== 'undefined') {
      return window.innerWidth >= 1024;
    }
    return true;
  });

  // Check if music is playing (for mobile bottom padding)
  const currentSong = useMusicStore(state => state.player.currentSong);

  // Load conversations from server on mount
  const loadConversations = useCallback(async () => {
    try {
      const serverConversations = await chatApi.getConversations();
      const loadedConversations: Conversation[] = serverConversations.map(conv => ({
        id: conv.id,
        title: conv.title,
        messages: [],
        createdAt: new Date(conv.created_at),
        updatedAt: new Date(conv.updated_at),
      }));
      setConversations(loadedConversations);
    } catch (error) {
      console.error('Failed to load conversations:', error);
    }
  }, [setConversations]);

  // Load conversations on mount
  useEffect(() => {
    if (!initialLoadDone) {
      loadConversations();
      setInitialLoadDone(true);
    }
  }, [initialLoadDone, loadConversations]);

  // Get current conversation
  const currentConversation = conversations.find(
    (c) => c.id === currentConversationId
  );

  // Load conversation messages when selecting a conversation
  useEffect(() => {
    const loadMessages = async () => {
      if (currentConversationId && currentConversation?.messages.length === 0) {
        try {
          const conv = await chatApi.getConversation(currentConversationId);
          const messages: Message[] = conv.messages.map(m => ({
            id: m.id,
            role: m.role as 'user' | 'assistant' | 'system',
            content: m.content,
            createdAt: new Date(m.created_at),
            model: m.model,
            tokensPrompt: m.tokens_prompt,
            tokensCompletion: m.tokens_completion,
          }));
          // Update the conversation in store with messages
          useChatStore.getState().updateConversation(currentConversationId, { messages });
        } catch (error) {
          console.error('Failed to load conversation messages:', error);
        }
      }
    };
    loadMessages();
  }, [currentConversationId, currentConversation]);

  // Set current conversation from URL
  useEffect(() => {
    if (conversationId) {
      setCurrentConversation(conversationId);
    }
  }, [conversationId, setCurrentConversation]);

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [currentConversation?.messages]);

  const handleSendMessage = async (content: string) => {
    if (!content.trim() || isLoading) return;

    const convId = currentConversationId || generateUUID();
    
    // If new conversation, create it
    if (!currentConversationId) {
      addConversation({
        id: convId,
        title: content.slice(0, 50),
        messages: [],
        createdAt: new Date(),
        updatedAt: new Date(),
      });
      setCurrentConversation(convId);
    }

    // Add user message
    const userMessage: Message = {
      id: generateUUID(),
      role: 'user',
      content,
      createdAt: new Date(),
    };
    addMessage(convId, userMessage);
    setLoading(true);
    setError(null);

    // Reset progressive voice synthesis state
    pendingTextRef.current = '';
    spokenSentencesRef.current = new Set();

    // Create placeholder for assistant message that will be updated during streaming
    const assistantMessageId = generateUUID();
    const assistantMessage: Message = {
      id: assistantMessageId,
      role: 'assistant',
      content: '',
      createdAt: new Date(),
    };
    addMessage(convId, assistantMessage);

    // Use streaming API
    chatApi.sendMessageStream(
      {
        message: content,
        conversation_id: convId,
        stream: true,
      },
      // On chunk - update message content AND progressively synthesize voice
      (chunk: string) => {
        useChatStore.getState().updateMessage(convId, assistantMessageId, (msg) => ({
          ...msg,
          content: msg.content + chunk,
        }));
        
        // Progressive voice synthesis: speak complete sentences as they arrive
        if (voiceConfig.enabled && voiceConfig.autoPlay) {
          pendingTextRef.current += chunk;
          
          // Check for complete sentences
          const sentences = pendingTextRef.current.split(SENTENCE_ENDINGS);
          
          // If we have more than one part, we have complete sentence(s)
          if (sentences.length > 1) {
            // Speak all complete sentences (all except the last incomplete part)
            for (let i = 0; i < sentences.length - 1; i++) {
              const sentence = sentences[i].trim();
              if (sentence && !spokenSentencesRef.current.has(sentence)) {
                spokenSentencesRef.current.add(sentence);
                // Fire and forget - don't await, let it queue up
                speak(sentence);
              }
            }
            // Keep only the incomplete part
            pendingTextRef.current = sentences[sentences.length - 1];
          }
        }
      },
      // On complete - update with final metadata and speak any remaining text
      async (response: StreamCompleteResponse) => {
        useChatStore.getState().updateMessage(convId, assistantMessageId, (msg) => ({
          ...msg,
          id: response.message_id,
          model: response.model,
          tokensPrompt: response.tokens_prompt,
          tokensCompletion: response.tokens_completion,
          processingTimeMs: response.processing_time_ms,
        }));
        
        // Speak any remaining text that didn't end with punctuation
        if (voiceConfig.enabled && voiceConfig.autoPlay) {
          const remainingText = pendingTextRef.current.trim();
          if (remainingText && !spokenSentencesRef.current.has(remainingText)) {
            speak(remainingText);
          }
          // Reset refs
          pendingTextRef.current = '';
          spokenSentencesRef.current = new Set();
        }
        
        lastSpokenMessageRef.current = response.message_id;
        setLoading(false);
      },
      // On error
      (error: string) => {
        console.error('Streaming error:', error);
        setError(error);
        setLoading(false);
      },
      // On tool executed - handle tool notifications and specific actions
      async (tools: string[]) => {
        console.log('🔧 Tools executed:', tools);
        
        // Notify sidebar about tool execution (makes icon glow)
        useNotificationStore.getState().notifyFromTools(tools);
        
        if (tools.includes('create_playlist')) {
          console.log('🎵 Playlist tool detected!');
          try {
            // Wait a bit for the playlist to be created
            await new Promise(resolve => setTimeout(resolve, 500));
            
            // Fetch updated playlists
            await useMusicStore.getState().fetchPlaylists();
            
            // Find the newest playlist (first one after refresh)
            const playlists = useMusicStore.getState().playlists;
            if (playlists.length > 0) {
              const newestPlaylist = playlists[0]; // Sorted by created_at desc
              console.log('🎵 Viewing new playlist:', newestPlaylist.name);
              
              // Fetch full playlist detail - SSE will handle updates automatically
              await useMusicStore.getState().fetchPlaylistDetail(newestPlaylist.id);
              
              // Start watching this playlist for updates (uses SSE with polling fallback)
              useMusicStore.getState().startWatchingPlaylist(newestPlaylist.id);
            }
          } catch (error) {
            console.error('Failed to handle playlist creation:', error);
          }
        }
      }
    );
  };
  return (
    <div className="flex h-screen bg-cyber-bg">
      {/* Sidebar */}
      <Sidebar isOpen={isSidebarOpen} onToggle={() => setIsSidebarOpen(!isSidebarOpen)} />

      {/* Main chat area */}
      <main className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <header className="h-14 border-b border-cyber-cyan/20 flex items-center px-2 sm:px-4 gap-2 sm:gap-4 bg-cyber-surface">
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="p-2 hover:bg-cyber-cyan/10 rounded text-cyber-cyan flex-shrink-0"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          <h1 className="font-semibold truncate text-cyber-cyan font-mono text-xs sm:text-sm flex-1 min-w-0">
            {currentConversation?.title || t('header.defaultTitle')}
          </h1>
          <div className="flex items-center gap-2 sm:gap-3 flex-shrink-0">
            {/* Voice Controls */}
            {voiceState.isAvailable && (
              <div className="flex items-center gap-2">
                {/* Voice toggle */}
                <button
                  onClick={() => setVoiceConfig({ enabled: !voiceConfig.enabled })}
                  className={`p-1.5 rounded transition-colors ${
                    voiceConfig.enabled 
                      ? 'text-cyber-cyan bg-cyber-cyan/10 hover:bg-cyber-cyan/20' 
                      : 'text-cyber-text/40 hover:text-cyber-text/60 hover:bg-cyber-surface'
                  }`}
                  title={voiceConfig.enabled ? 'Disable voice' : 'Enable voice'}
                >
                  {voiceConfig.enabled ? (
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                    </svg>
                  ) : (
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" />
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2" />
                    </svg>
                  )}
                </button>
                
                {/* Stop button (when playing) */}
                {(voiceState.isPlaying || voiceState.isLoading) && (
                  <button
                    onClick={stopVoice}
                    className="p-1.5 rounded text-cyber-pink bg-cyber-pink/10 hover:bg-cyber-pink/20 transition-colors"
                    title="Stop voice"
                  >
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                      <rect x="6" y="6" width="12" height="12" rx="1" />
                    </svg>
                  </button>
                )}
                
                {/* Speaking indicator */}
                {voiceState.isPlaying && (
                  <div className="flex items-center gap-1">
                    <div className="w-1 h-3 bg-cyber-cyan rounded-full animate-pulse"></div>
                    <div className="w-1 h-4 bg-cyber-cyan rounded-full animate-pulse" style={{ animationDelay: '0.1s' }}></div>
                    <div className="w-1 h-2 bg-cyber-cyan rounded-full animate-pulse" style={{ animationDelay: '0.2s' }}></div>
                  </div>
                )}
              </div>
            )}
            
            {/* Connection status */}
            <span className="w-2 h-2 rounded-full bg-cyber-green animate-pulse shadow-[0_0_10px_#00ff88]"></span>
            <span className="text-xs text-cyber-green font-mono">{t('status.online')}</span>
          </div>
        </header>

        {/* Messages */}
        <div className="flex-1 overflow-y-auto p-2 sm:p-4 space-y-3 sm:space-y-4">
          {!currentConversation || currentConversation.messages.length === 0 ? (
            <WelcomeScreen />
          ) : (
            <>
              {currentConversation.messages.map((message, index) => {
                // The last assistant message during loading is streaming
                const isLastMessage = index === currentConversation.messages.length - 1;
                const isStreamingMessage = isLoading && isLastMessage && message.role === 'assistant';
                return (
                  <ChatMessage 
                    key={message.id} 
                    message={message} 
                    isStreaming={isStreamingMessage}
                  />
                );
              })}
              {isLoading && <TypingIndicator />}
            </>
          )}
          <div ref={messagesEndRef} />
        </div>

        {/* Input - sticky at bottom */}
        <div className={clsx(
          "p-2 sm:p-4 border-t border-cyber-cyan/20 bg-cyber-surface flex-shrink-0",
          currentSong && "mb-20 sm:mb-0"
        )}>
          <ChatInput onSend={handleSendMessage} disabled={isLoading} />
        </div>
      </main>
    </div>
  );
}
