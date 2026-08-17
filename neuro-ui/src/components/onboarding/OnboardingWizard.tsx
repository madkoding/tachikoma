import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import clsx from 'clsx';
import { HardwareProfile } from '../../api/system';
import { ModelSize, useOnboardingStore } from '../../stores/onboardingStore';
import { systemApi } from '../../api/client';
import { MODEL_CATALOG, OPENAI_CATALOG, OLLAMA_CLOUD_CATALOG, useModelsStore } from '../../stores/modelsStore';
import TemplateGallery from '../agents/TemplateGallery';

const MODEL_SIZES: { id: ModelSize; nameKey: string; descKey: string }[] = [
  { id: 'small', nameKey: 'onboarding.model.small.name', descKey: 'onboarding.model.small.desc' },
  { id: 'medium', nameKey: 'onboarding.model.medium.name', descKey: 'onboarding.model.medium.desc' },
  { id: 'large', nameKey: 'onboarding.model.large.name', descKey: 'onboarding.model.large.desc' },
];

const SIZE_LABEL_KEY: Record<ModelSize, string> = {
  small: 'onboarding.model.small.name',
  medium: 'onboarding.model.medium.name',
  large: 'onboarding.model.large.name',
};

export default function OnboardingWizard() {
  const { t } = useTranslation();
  const complete = useOnboardingStore((s) => s.complete);

  const [step, setStep] = useState(0);
  const [hardware, setHardware] = useState<HardwareProfile | null>(null);
  const [hardwareLoading, setHardwareLoading] = useState(false);
  const [modelSize, setModelSize] = useState<ModelSize | null>(null);
  const [agentName, setAgentName] = useState('');

  // Detect hardware when reaching step 1
  const detectHardware = async () => {
    if (hardware || hardwareLoading) return;
    setHardwareLoading(true);
    try {
      const profile = await systemApi.hardware();
      setHardware(profile);
      // Pre-select recommended size
      if (!modelSize && profile.recommended_model_size) {
        setModelSize(profile.recommended_model_size as ModelSize);
      }
    } finally {
      setHardwareLoading(false);
    }
  };

  const goNext = () => {
    if (step === 1) void detectHardware();
    if (step < 3) setStep(step + 1);
  };

  const finish = () => {
    complete({
      modelSize: modelSize ?? undefined,
      agentName: agentName.trim() || t('onboarding.agent.defaultName'),
    });
    // Kick off the model download for the chosen size (fire-and-forget).
    if (modelSize) {
      const p = useModelsStore.getState().provider;
      const catalog =
        p === 'openai' ? OPENAI_CATALOG
        : p === 'ollama_cloud' ? OLLAMA_CLOUD_CATALOG
        : MODEL_CATALOG;
      const entry = catalog.find((m) => m.size_tier === modelSize && !m.is_embedding);
      if (entry) void useModelsStore.getState().install(entry);
    }
  };

  const total = 4;

  return (
    <div className="min-h-screen bg-cyber-bg flex flex-col items-center justify-center p-6">
      <div className="w-full max-w-lg">
        {/* Progress */}
        <div className="flex items-center justify-between mb-6">
          <span className="text-xs font-mono text-cyber-cyan/60">
            {t('onboarding.step', { current: step + 1, total })}
          </span>
          <button
            onClick={finish}
            className="text-xs font-mono text-cyber-cyan/50 hover:text-cyber-cyan underline"
          >
            {t('onboarding.skip')}
          </button>
        </div>

        <div className="h-1 w-full bg-black/40 rounded overflow-hidden mb-8">
          <div
            className="h-full bg-cyber-cyan transition-all duration-300"
            style={{ width: `${((step + 1) / total) * 100}%` }}
          />
        </div>

        <div className="cyber-card p-6 sm:p-8 border-cyber-cyan/30">
          {step === 0 && <WelcomeStep />}
          {step === 1 && <HardwareStep profile={hardware} loading={hardwareLoading} />}
          {step === 2 && (
            <ModelStep selected={modelSize} recommended={hardware?.recommended_model_size ?? null} onSelect={setModelSize} />
          )}
          {step === 3 && <AgentStep name={agentName} onChange={setAgentName} />}
        </div>

        {/* Nav */}
        <div className="flex justify-between mt-6">
          {step > 0 ? (
            <button
              onClick={() => setStep(step - 1)}
              className="cyber-button opacity-70"
            >
              {t('onboarding.back')}
            </button>
          ) : (
            <span />
          )}
          {step < total - 1 ? (
            <button onClick={goNext} className="cyber-button">
              {t('onboarding.next')}
            </button>
          ) : (
            <button
              onClick={finish}
              className="cyber-button border-cyber-green text-cyber-green"
            >
              {t('onboarding.finish')}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function WelcomeStep() {
  const { t } = useTranslation();
  return (
    <div className="text-center">
      <h1 className="text-2xl font-bold font-cyber tracking-wider neon-cyan mb-2">
        {t('onboarding.welcome.title')}
      </h1>
      <p className="text-cyber-cyan/60 text-sm mb-6">{t('onboarding.welcome.desc')}</p>
      <ul className="space-y-2 text-left font-mono text-sm text-cyber-cyan/80">
        <li>• {t('onboarding.welcome.feature1')}</li>
        <li>• {t('onboarding.welcome.feature2')}</li>
        <li>• {t('onboarding.welcome.feature3')}</li>
      </ul>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between py-2 border-b border-black/30 text-sm">
      <span className="text-cyber-cyan/50">{label}</span>
      <span className="font-mono text-cyber-cyan/90 text-right">{value}</span>
    </div>
  );
}

function HardwareStep({ profile, loading }: { profile: HardwareProfile | null; loading: boolean }) {
  const { t } = useTranslation();
  return (
    <div>
      <h2 className="text-xl font-bold mb-1">{t('onboarding.hardware.title')}</h2>
      <p className="text-cyber-cyan/60 text-sm mb-4">{t('onboarding.hardware.desc')}</p>
      {loading && <div className="text-sm text-cyber-cyan/70">{t('common.loading')}</div>}
      {!loading && !profile && <div className="text-sm text-cyber-red">{t('onboarding.hardware.detectFailed')}</div>}
      {!loading && profile && (
        <div>
          <Row label={t('onboarding.hardware.cpu')} value={t('onboarding.hardware.cores', { count: profile.cpu_cores })} />
          <Row label={t('onboarding.hardware.ram')} value={t('onboarding.hardware.ramValue', { gb: Math.round(profile.ram_gb) })} />
          <Row label={t('onboarding.hardware.disk')} value={t('onboarding.hardware.diskValue', { gb: Math.round(profile.disk_free_gb) })} />
          <Row
            label={t('onboarding.hardware.gpu')}
            value={
              profile.gpu_model
                ? t('onboarding.hardware.gpuValue', { model: profile.gpu_model, gb: Math.round(profile.vram_gb) })
                : t('onboarding.hardware.noGpu')
            }
          />
          <p className="mt-4 text-xs font-mono text-cyber-yellow">
            {t('onboarding.hardware.recommended', {
              size: t(SIZE_LABEL_KEY[profile.recommended_model_size as ModelSize] ?? SIZE_LABEL_KEY.small),
            })}
          </p>
        </div>
      )}
    </div>
  );
}

function ModelStep({
  selected,
  recommended,
  onSelect,
}: {
  selected: ModelSize | null;
  recommended: string | null;
  onSelect: (size: ModelSize) => void;
}) {
  const { t } = useTranslation();
  return (
    <div>
      <h2 className="text-xl font-bold mb-1">{t('onboarding.model.title')}</h2>
      <p className="text-cyber-cyan/60 text-sm mb-4">{t('onboarding.model.desc')}</p>
      <div className="space-y-2">
        {MODEL_SIZES.map((size) => {
          const isRecommended = recommended === size.id;
          const isSelected = selected === size.id;
          return (
            <button
              key={size.id}
              onClick={() => onSelect(size.id)}
              className={clsx(
                'w-full text-left p-3 rounded border transition-all',
                isSelected
                  ? 'border-cyber-cyan bg-cyber-cyan/10'
                  : 'border-black/40 hover:border-cyber-cyan/50'
              )}
            >
              <div className="flex items-center justify-between">
                <span className="font-medium">{t(size.nameKey)}</span>
                {isRecommended && (
                  <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-cyber-yellow/10 text-cyber-yellow border border-cyber-yellow/40">
                    {t('onboarding.model.recommended')}
                  </span>
                )}
                {isSelected && (
                  <span className="text-xs text-cyber-cyan">{t('onboarding.model.selected')}</span>
                )}
              </div>
              <p className="text-xs text-cyber-cyan/50 mt-1">{t(size.descKey)}</p>
            </button>
          );
        })}
      </div>
    </div>
  );
}

function AgentStep({
  name,
  onChange,
}: {
  name: string;
  onChange: (name: string) => void;
}) {
  const { t } = useTranslation();
  return (
    <div>
      <h2 className="text-xl font-bold mb-1">{t('onboarding.agent.title')}</h2>
      <p className="text-cyber-cyan/60 text-sm mb-4">{t('onboarding.agent.desc')}</p>
      <label className="block text-xs text-cyber-cyan/60 mb-1">{t('onboarding.agent.nameLabel')}</label>
      <input
        className="cyber-input w-full mb-4"
        value={name}
        onChange={(e) => onChange(e.target.value)}
        placeholder={t('onboarding.agent.namePlaceholder')}
      />
      <p className="text-xs text-cyber-cyan/60 mb-2">{t('templates.title')}</p>
      <TemplateGallery />
    </div>
  );
}
