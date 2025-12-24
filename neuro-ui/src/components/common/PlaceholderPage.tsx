import { useTranslation } from 'react-i18next';
import TypewriterText from './TypewriterText';

interface PlaceholderPageProps {
  readonly titleKey: string;
  readonly icon: React.ReactNode;
}

export default function PlaceholderPage({ titleKey, icon }: PlaceholderPageProps) {
  const { t } = useTranslation();

  return (
    <div className="flex-1 flex items-center justify-center bg-cyber-bg">
      <div className="text-center">
        <div className="w-20 h-20 mx-auto mb-6 rounded-2xl bg-cyber-cyan/10 border border-cyber-cyan/30 flex items-center justify-center text-cyber-cyan/50">
          {icon}
        </div>
        <h1 className="text-2xl font-cyber font-bold text-cyber-cyan mb-2">
          <TypewriterText text={t(titleKey)} speed={20} />
        </h1>
        <p className="text-cyber-cyan/50 font-mono text-sm">
          <TypewriterText text={t('common.comingSoon')} delay={500} speed={25} />
        </p>
        <div className="mt-8 flex justify-center gap-1">
          <span className="w-2 h-2 rounded-full bg-cyber-cyan/30 animate-pulse"></span>
          <span className="w-2 h-2 rounded-full bg-cyber-cyan/30 animate-pulse" style={{ animationDelay: '0.2s' }}></span>
          <span className="w-2 h-2 rounded-full bg-cyber-cyan/30 animate-pulse" style={{ animationDelay: '0.4s' }}></span>
        </div>
      </div>
    </div>
  );
}
