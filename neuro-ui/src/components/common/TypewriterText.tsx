import { memo, useEffect, useState, useRef, useCallback } from 'react';

interface TypewriterTextProps {
  readonly text: string;
  readonly className?: string;
  readonly speed?: number;
  readonly delay?: number;
  readonly onComplete?: () => void;
}

interface CharState {
  char: string;
  id: number;
  isNew: boolean;
}

function TypewriterText({
  text,
  className = '',
  speed = 15,
  delay = 0,
  onComplete,
}: TypewriterTextProps) {
  const [displayedChars, setDisplayedChars] = useState<CharState[]>([]);
  const [isComplete, setIsComplete] = useState(false);
  const [hasStarted, setHasStarted] = useState(false);
  const containerRef = useRef<HTMLSpanElement>(null);
  const charIndexRef = useRef(0);
  const charIdRef = useRef(0);
  // Use Array.from to properly handle Unicode characters (emojis, etc.)
  const charsArrayRef = useRef<string[]>([]);

  // Reset effect
  useEffect(() => {
    setDisplayedChars([]);
    setIsComplete(false);
    setHasStarted(false);
    charIndexRef.current = 0;
    charIdRef.current = 0;
    // Split text properly handling Unicode (emojis are surrogate pairs)
    charsArrayRef.current = Array.from(text);

    const startTimer = setTimeout(() => {
      setHasStarted(true);
    }, delay);

    return () => clearTimeout(startTimer);
  }, [text, delay]);

  // Clear new flag after animation
  const clearNewFlag = useCallback((charId: number) => {
    setTimeout(() => {
      setDisplayedChars(prev => 
        prev.map(c => c.id === charId ? { ...c, isNew: false } : c)
      );
    }, 400);
  }, []);

  // Typing effect
  useEffect(() => {
    if (!hasStarted) return;

    const charsArray = charsArrayRef.current;
    if (charIndexRef.current < charsArray.length) {
      const timer = setTimeout(() => {
        const newChar = charsArray[charIndexRef.current];
        const newId = charIdRef.current;
        charIndexRef.current += 1;
        charIdRef.current += 1;
        
        setDisplayedChars(prev => [
          ...prev.map(c => ({ ...c, isNew: false })),
          { char: newChar, id: newId, isNew: true }
        ]);

        clearNewFlag(newId);
      }, speed);

      return () => clearTimeout(timer);
    } else if (!isComplete && displayedChars.length === charsArray.length) {
      setIsComplete(true);
      onComplete?.();
    }
  }, [displayedChars, text, speed, hasStarted, isComplete, onComplete, clearNewFlag]);

  return (
    <span 
      ref={containerRef}
      className={`${className} ${isComplete ? '' : 'streaming-content'}`}
    >
      {displayedChars.map((item) => (
        <span
          key={item.id}
          className={item.isNew ? 'typewriter-char-new' : ''}
        >
          {item.char}
        </span>
      ))}
      {hasStarted && !isComplete && (
        <span className="typewriter-cursor" />
      )}
    </span>
  );
}

export default memo(TypewriterText);
