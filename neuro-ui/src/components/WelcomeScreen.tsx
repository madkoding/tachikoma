import { useTranslation } from 'react-i18next';

export default function WelcomeScreen() {
  const { t } = useTranslation();

  return (
    <div className="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div className="w-16 h-16 mb-6 rounded-full bg-neuro-100 dark:bg-neuro-900/30 flex items-center justify-center">
        <svg className="w-10 h-10 text-neuro-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} 
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
      </div>
      <h2 className="text-2xl font-bold mb-2 text-gray-900 dark:text-gray-100">
        {t('chat.welcome')}
      </h2>
      <p className="text-gray-600 dark:text-gray-400 max-w-md">
        {t('chat.welcomeDesc')}
      </p>
      
      <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-4 max-w-2xl">
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          }
          title="Long-term Memory"
          description="I remember our conversations and learn your preferences"
        />
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          }
          title="Web Search"
          description="Search the web for current information when needed"
        />
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          }
          title="Command Execution"
          description="Run safe local commands to help with tasks"
        />
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          }
          title="Adaptive Model"
          description="Automatically selects the best model for each task"
        />
      </div>
    </div>
  );
}

interface FeatureCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
}

function FeatureCard({ icon, title, description }: FeatureCardProps) {
  return (
    <div className="p-4 rounded-lg border border-gray-200 dark:border-gray-700 text-left">
      <div className="text-neuro-500 mb-2">{icon}</div>
      <h3 className="font-medium mb-1">{title}</h3>
      <p className="text-sm text-gray-600 dark:text-gray-400">{description}</p>
    </div>
  );
}
