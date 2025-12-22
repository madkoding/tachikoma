import { useState, useRef, useEffect, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useChatStore, Message, Conversation } from '../stores/chatStore';
import { chatApi, StreamCompleteResponse } from '../api/client';
import ChatMessage from '../components/ChatMessage';
import ChatInput from '../components/ChatInput';
import Sidebar from '../components/Sidebar';
import TypingIndicator from '../components/TypingIndicator';
import WelcomeScreen from '../components/WelcomeScreen';
import { useVoiceStream } from '../hooks/useVoiceStream';

export default function ChatPage() {
  const { t } = useTranslation();
  const { conversationId } = useParams<{ conversationId?: string }>();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [initialLoadDone, setInitialLoadDone] = useState(false);
  
  // Voice synthesis hook
  const { state: voiceState, config: voiceConfig, speak, stop: stopVoice, setConfig: setVoiceConfig } = useVoiceStream();
  const lastSpokenMessageRef = useRef<string | null>(null);
  
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

  const [isSidebarOpen, setIsSidebarOpen] = useState(true);

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

    const convId = currentConversationId || crypto.randomUUID();
    
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
      id: crypto.randomUUID(),
      role: 'user',
      content,
      createdAt: new Date(),
    };
    addMessage(convId, userMessage);
    setLoading(true);
    setError(null);

    // Create placeholder for assistant message that will be updated during streaming
    const assistantMessageId = crypto.randomUUID();
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
      // On chunk - update message content
      (chunk: string) => {
        useChatStore.getState().updateMessage(convId, assistantMessageId, (msg) => ({
          ...msg,
          content: msg.content + chunk,
        }));
      },
      // On complete - update with final metadata
      (response: StreamCompleteResponse) => {
        const finalMessage = useChatStore.getState().conversations
          .find(c => c.id === convId)?.messages
          .find(m => m.id === assistantMessageId);
        
        useChatStore.getState().updateMessage(convId, assistantMessageId, (msg) => ({
          ...msg,
          id: response.message_id,
          model: response.model,
          tokensPrompt: response.tokens_prompt,
          tokensCompletion: response.tokens_completion,
          processingTimeMs: response.processing_time_ms,
        }));
        
        // Auto-speak the response if voice is enabled
        if (voiceConfig.enabled && voiceConfig.autoPlay && finalMessage?.content) {
          // Only speak if we haven't spoken this message yet
          if (lastSpokenMessageRef.current !== finalMessage.id) {
            lastSpokenMessageRef.current = finalMessage.id;
            speak(finalMessage.content);
          }
        }
        
        setLoading(false);
      },
      // On error
      (error: string) => {
        console.error('Streaming error:', error);
        setError(error);
        setLoading(false);
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
        <header className="h-14 border-b border-cyber-cyan/20 flex items-center px-4 gap-4 bg-cyber-surface">
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="p-2 hover:bg-cyber-cyan/10 rounded text-cyber-cyan lg:hidden"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
          <h1 className="font-semibold truncate text-cyber-cyan font-mono text-sm">
            {currentConversation?.title || t('header.defaultTitle')}
          </h1>
          <div className="ml-auto flex items-center gap-3">
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
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {!currentConversation || currentConversation.messages.length === 0 ? (
            <WelcomeScreen />
          ) : (
            <>
              {currentConversation.messages.map((message) => (
                <ChatMessage key={message.id} message={message} />
              ))}
              {isLoading && <TypingIndicator />}
            </>
          )}
          <div ref={messagesEndRef} />
        </div>

        {/* Input */}
        <div className="p-4 border-t border-cyber-cyan/20 bg-cyber-surface">
          <ChatInput onSend={handleSendMessage} disabled={isLoading} />
        </div>
      </main>
    </div>
  );
}
