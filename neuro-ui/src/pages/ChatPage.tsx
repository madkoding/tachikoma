import { useState, useRef, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useChatStore, Message } from '../stores/chatStore';
import { chatApi } from '../api/client';
import ChatMessage from '../components/ChatMessage';
import ChatInput from '../components/ChatInput';
import Sidebar from '../components/Sidebar';
import TypingIndicator from '../components/TypingIndicator';
import WelcomeScreen from '../components/WelcomeScreen';

export default function ChatPage() {
  const { t } = useTranslation();
  const { conversationId } = useParams<{ conversationId?: string }>();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  
  const {
    conversations,
    currentConversationId,
    isLoading,
    setCurrentConversation,
    addMessage,
    addConversation,
    setLoading,
    setError,
  } = useChatStore();

  const [isSidebarOpen, setIsSidebarOpen] = useState(true);

  // Get current conversation
  const currentConversation = conversations.find(
    (c) => c.id === currentConversationId
  );

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

    try {
      const response = await chatApi.sendMessage({
        message: content,
        conversation_id: convId,
      });

      // Add assistant message
      const assistantMessage: Message = {
        id: response.message_id,
        role: 'assistant',
        content: response.content,
        createdAt: new Date(),
        model: response.model,
        tokensPrompt: response.tokens_prompt,
        tokensCompletion: response.tokens_completion,
      };
      addMessage(convId, assistantMessage);
    } catch (error: unknown) {
      console.error('Failed to send message:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to send message';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
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
          <div className="ml-auto flex items-center gap-2">
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
