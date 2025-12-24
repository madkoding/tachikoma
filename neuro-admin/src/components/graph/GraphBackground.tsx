import StarField from './StarField';

interface GraphBackgroundProps {
  readonly graphRef?: React.RefObject<any>;
}

export default function GraphBackground({ graphRef }: GraphBackgroundProps) {
  return (
    <div className="absolute inset-0 pointer-events-none -z-10 overflow-hidden">
      {/* Deep space gradient base */}
      <div className="deep-space-base"></div>

      {/* Distant galaxy clusters */}
      <div className="galaxy-cluster galaxy-1"></div>
      <div className="galaxy-cluster galaxy-2"></div>
      <div className="galaxy-cluster galaxy-3"></div>

      {/* Nebula layers - more dramatic */}
      <div className="nebula-layer nebula-1"></div>
      <div className="nebula-layer nebula-2"></div>
      <div className="nebula-layer nebula-3"></div>

      {/* Cosmic dust cloud */}
      <div className="cosmic-dust"></div>

      {/* Canvas-based star field with thousands of stars */}
      <StarField starCount={5000} graphRef={graphRef} />

      {/* Subtle grid overlay - optional sci-fi feel */}
      <div
        className="absolute inset-0 opacity-[0.02] pointer-events-none"
        style={{
          backgroundImage:
            'linear-gradient(rgba(0,245,255,0.15) 1px, transparent 1px), linear-gradient(90deg, rgba(0,245,255,0.15) 1px, transparent 1px)',
          backgroundSize: '80px 80px',
        }}
      />
    </div>
  );
}
