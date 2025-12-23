interface TooltipData {
  text: string;
  x: number;
  y: number;
  alignment: 'left' | 'center' | 'right';
}

interface GraphTooltipProps {
  readonly tooltip: TooltipData | null;
  readonly displayedText: string;
  readonly fading: boolean;
}

export default function GraphTooltip({ tooltip, displayedText, fading }: GraphTooltipProps) {
  if (!tooltip || !displayedText) return null;

  const getTransform = () => {
    if (tooltip.alignment === 'left') return 'translateY(-100%)';
    if (tooltip.alignment === 'right') return 'translate(-100%, -100%)';
    return 'translate(-50%, -100%)';
  };

  return (
    <div
      className={`absolute pointer-events-none z-20 transition-opacity duration-300 ${
        fading ? 'opacity-0' : 'opacity-100'
      }`}
      style={{
        left: tooltip.x,
        top: tooltip.y,
        transform: getTransform(),
        maxWidth: '280px',
      }}
    >
      <span className="text-cyber-cyan font-cyber text-sm font-bold tracking-wide drop-shadow-[0_0_10px_rgba(0,245,255,0.8)]">
        {displayedText}
      </span>
    </div>
  );
}
