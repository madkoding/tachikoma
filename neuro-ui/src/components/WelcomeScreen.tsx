import { useTranslation } from 'react-i18next';

export default function WelcomeScreen() {
  const { t } = useTranslation();

  return (
    <div className="flex-1 flex flex-col items-center justify-center text-center p-8">
      <div className="w-20 h-20 mb-6 rounded border border-cyber-cyan/50 bg-cyber-cyan/10 flex items-center justify-center shadow-[0_0_30px_rgba(0,245,255,0.3)]">
        <svg className="w-12 h-12 text-cyber-cyan" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} 
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
        </svg>
      </div>
      <h2 className="text-3xl font-bold mb-2 neon-cyan font-cyber tracking-wider glitch" data-text="TACHIKOMA">
        TACHIKOMA
      </h2>
      <p className="text-cyber-cyan/60 max-w-md font-mono text-sm">
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
          title={t('feature.memory')}
          description={t('feature.memoryDesc')}
        />
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          }
          title={t('feature.search')}
          description={t('feature.searchDesc')}
        />
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          }
          title={t('feature.cmd')}
          description={t('feature.cmdDesc')}
        />
        <FeatureCard
          icon={
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                    d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          }
          title={t('feature.model')}
          description={t('feature.modelDesc')}
        />
      </div>
    </div>
  );
}

interface FeatureCardProps {
  readonly icon: React.ReactNode;
  readonly title: string;
  readonly description: string;
}

function FeatureCard({ icon, title, description }: Readonly<FeatureCardProps>) {
  return (
    <div className="cyber-card text-left hover:border-cyber-cyan/50 transition-all duration-300 hover:shadow-[0_0_20px_rgba(0,245,255,0.2)]">
      <div className="text-cyber-cyan mb-2">{icon}</div>
      <h3 className="font-medium mb-1 text-cyber-cyan font-mono text-sm">{title}</h3>
      <p className="text-sm text-cyber-cyan/50">{description}</p>
    </div>
  );
}
