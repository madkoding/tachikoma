import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { useChatStore, Conversation } from '../stores/chatStore';
import clsx from 'clsx';

interface SidebarProps {
  isOpen: boolean;
  onToggle: () => void;
}

export default function Sidebar({ isOpen, onToggle }: SidebarProps) {
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
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={onToggle}
        />
      )}

      {/* Sidebar */}
      <aside
        className={clsx(
          'fixed lg:relative inset-y-0 left-0 z-50 w-64 bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700',
          'transform transition-transform duration-200 ease-in-out',
          isOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0 lg:w-0 lg:border-0'
        )}
      >
        <div className="flex flex-col h-full">
          {/* Header */}
          <div className="h-14 flex items-center justify-between px-4 border-b border-gray-200 dark:border-gray-700">
            <span className="font-semibold text-neuro-500">{t('app.title')}</span>
            <button
              onClick={onToggle}
              className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded lg:hidden"
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
              className="w-full flex items-center gap-2 px-4 py-2 bg-neuro-500 hover:bg-neuro-600 text-white rounded-lg transition-colors"
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
                onDelete={deleteConversation}
              />
            )}
            {yesterday.length > 0 && (
              <ConversationGroup
                title={t('sidebar.yesterday')}
                conversations={yesterday}
                currentId={currentConversationId}
                onSelect={handleSelectConversation}
                onDelete={deleteConversation}
              />
            )}
            {older.length > 0 && (
              <ConversationGroup
                title={t('sidebar.older')}
                conversations={older}
                currentId={currentConversationId}
                onSelect={handleSelectConversation}
                onDelete={deleteConversation}
              />
            )}
          </div>
        </div>
      </aside>
    </>
  );
}

interface ConversationGroupProps {
  title: string;
  conversations: Conversation[];
  currentId: string | null;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
}

function ConversationGroup({ title, conversations, currentId, onSelect, onDelete }: ConversationGroupProps) {
  return (
    <div className="mb-4">
      <h3 className="px-2 py-1 text-xs font-medium text-gray-500 uppercase tracking-wider">
        {title}
      </h3>
      <ul className="space-y-1">
        {conversations.map((conv) => (
          <li key={conv.id}>
            <button
              onClick={() => onSelect(conv.id)}
              className={clsx(
                'w-full flex items-center gap-2 px-2 py-2 rounded-lg text-left text-sm truncate',
                'hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors',
                currentId === conv.id && 'bg-neuro-50 dark:bg-neuro-900/20'
              )}
            >
              <svg className="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                      d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
              <span className="truncate">{conv.title}</span>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
