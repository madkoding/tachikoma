import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { useChatStore, Conversation } from '../stores/chatStore';
import { chatApi } from '../api/client';
import clsx from 'clsx';

interface SidebarProps {
  readonly isOpen: boolean;
  readonly onToggle: () => void;
}

export default function Sidebar({ isOpen, onToggle }: Readonly<SidebarProps>) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { conversations, currentConversationId, setCurrentConversation, deleteConversation } = useChatStore();

  const handleNewChat = () => {
    setCurrentConversation(null);
    navigate('/');
  };

  const handleSelectConversation = (id: string) => {
    setCurrentConversation(id);
    navigate(`/chat/${id}`);
  };

  const handleDeleteConversation = async (id: string) => {
    try {
      // Delete from backend
      await chatApi.deleteConversation(id);
      // Update local state
      deleteConversation(id);
      // If we deleted the current conversation, go to home
      if (currentConversationId === id) {
        navigate('/');
      }
    } catch (error) {
      console.error('Failed to delete conversation:', error);
    }
  };

  const groupConversations = (conversations: Conversation[]) => {
    const today: Conversation[] = [];
    const yesterday: Conversation[] = [];
    const older: Conversation[] = [];
    
    const now = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterdayStart = new Date(todayStart.getTime() - 24 * 60 * 60 * 1000);

    conversations.forEach((conv) => {
      const convDate = new Date(conv.createdAt);
      if (convDate >= todayStart) {
        today.push(conv);
      } else if (convDate >= yesterdayStart) {
        yesterday.push(conv);
      } else {
        older.push(conv);
      }
    });

    return { today, yesterday, older };
  };

  const { today, yesterday, older } = groupConversations(conversations);

  return (
    <>
      {/* Overlay for mobile */}
      {isOpen && (
        <div
          className="fixed inset-0 bg-black/70 z-40 lg:hidden backdrop-blur-sm"
          onClick={onToggle}
        />
      )}

      {/* Sidebar */}
      <aside
        className={clsx(
          'fixed lg:relative inset-y-0 left-0 z-50 w-64 bg-cyber-surface border-r border-cyber-cyan/20',
          'transform transition-transform duration-200 ease-in-out',
          isOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0 lg:w-0 lg:border-0'
        )}
      >
        {/* Glow effect */}
        <div className="absolute inset-0 bg-gradient-to-b from-cyber-cyan/5 to-transparent pointer-events-none"></div>
        
        <div className="flex flex-col h-full relative">
          {/* Header */}
          <div className="h-14 flex items-center justify-between px-4 border-b border-cyber-cyan/20">
            <span className="font-bold neon-cyan font-cyber tracking-wider text-sm">TACHIKOMA</span>
            <button
              onClick={onToggle}
              className="p-1 hover:bg-cyber-cyan/10 rounded lg:hidden text-cyber-cyan"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* New Chat Button */}
          <div className="p-4">
            <button
              onClick={handleNewChat}
              className="cyber-button w-full flex items-center justify-center gap-2"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              {t('sidebar.newChat')}
            </button>
          </div>

          {/* Conversations List */}
          <div className="flex-1 overflow-y-auto px-2">
            {today.length > 0 && (
              <ConversationGroup
                title={t('sidebar.today')}
                conversations={today}
                currentId={currentConversationId}
                onSelect={handleSelectConversation}
                onDelete={handleDeleteConversation}
              />
            )}
            {yesterday.length > 0 && (
              <ConversationGroup
                title={t('sidebar.yesterday')}
                conversations={yesterday}
                currentId={currentConversationId}
                onSelect={handleSelectConversation}
                onDelete={handleDeleteConversation}
              />
            )}
            {older.length > 0 && (
              <ConversationGroup
                title={t('sidebar.older')}
                conversations={older}
                currentId={currentConversationId}
                onSelect={handleSelectConversation}
                onDelete={handleDeleteConversation}
              />
            )}
          </div>
        </div>
      </aside>
    </>
  );
}

interface ConversationGroupProps {
  readonly title: string;
  readonly conversations: Conversation[];
  readonly currentId: string | null;
  readonly onSelect: (id: string) => void;
  readonly onDelete: (id: string) => void;
}

function ConversationGroup({ title, conversations, currentId, onSelect, onDelete }: Readonly<ConversationGroupProps>) {
  return (
    <div className="mb-4">
      <h3 className="px-2 py-1 text-xs font-medium text-cyber-cyan/50 uppercase tracking-wider font-mono">
        {title}
      </h3>
      <ul className="space-y-1">
        {conversations.map((conv) => (
          <li key={conv.id} className="group relative">
            <button
              onClick={() => onSelect(conv.id)}
              className={clsx(
                'w-full flex items-center gap-2 px-2 py-2 rounded text-left text-sm truncate font-mono',
                'hover:bg-cyber-cyan/10 transition-all border border-transparent',
                currentId === conv.id 
                  ? 'bg-cyber-cyan/10 border-cyber-cyan/30 text-cyber-cyan' 
                  : 'text-cyber-cyan/60 hover:text-cyber-cyan hover:border-cyber-cyan/20'
              )}
            >
              <svg className="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                      d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
              <span className="truncate flex-1">{conv.title}</span>
            </button>
            <button
              onClick={(e) => {
                e.stopPropagation();
                onDelete(conv.id);
              }}
              className="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-red-500/20 rounded"
              title="Eliminar conversación"
            >
              <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
