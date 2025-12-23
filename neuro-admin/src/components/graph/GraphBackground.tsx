export default function GraphBackground() {
  return (
    <>
      {/* Nebula layers */}
      <div className="nebula-layer nebula-1"></div>
      <div className="nebula-layer nebula-2"></div>
      <div className="nebula-layer nebula-3"></div>

      {/* Stars */}
      <div className="stars-layer"></div>

      {/* Grid overlay effect */}
      <div
        className="absolute inset-0 opacity-5 pointer-events-none"
        style={{
          backgroundImage:
            'linear-gradient(rgba(0,245,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(0,245,255,0.1) 1px, transparent 1px)',
          backgroundSize: '50px 50px',
        }}
      />
    </>
  );
}
