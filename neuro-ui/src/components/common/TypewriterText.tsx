import { memo, useEffect, useState, useRef } from 'react';

interface TypewriterTextProps {
  readonly text: string;
  readonly className?: string;
  readonly speed?: number;
  readonly delay?: number;
  readonly onComplete?: () => void;
}

function TypewriterText({
  text,
  className = '',
  speed = 15,
  delay = 0,
  onComplete,
}: TypewriterTextProps) {
  const [displayedText, setDisplayedText] = useState('');
  const [isComplete, setIsComplete] = useState(false);
  const [hasStarted, setHasStarted] = useState(false);
  const containerRef = useRef<HTMLSpanElement>(null);
  
  // Refs to track state without causing re-renders
  const charIndexRef = useRef(0);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const delayTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const charsArrayRef = useRef<string[]>([]);
  const onCompleteRef = useRef(onComplete);
  
  // Keep onComplete ref updated
  useEffect(() => {
    onCompleteRef.current = onComplete;
  }, [onComplete]);

  // Reset and start effect
  useEffect(() => {
    // Clear any existing timers
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    if (delayTimeoutRef.current) {
      clearTimeout(delayTimeoutRef.current);
      delayTimeoutRef.current = null;
    }
    
    // Reset state
    setDisplayedText('');
    setIsComplete(false);
    setHasStarted(false);
    charIndexRef.current = 0;
    charsArrayRef.current = Array.from(text);

    // Start after delay
    delayTimeoutRef.current = setTimeout(() => {
      setHasStarted(true);
    }, delay);

    return () => {
      if (delayTimeoutRef.current) {
        clearTimeout(delayTimeoutRef.current);
      }
    };
  }, [text, delay]);

  // Typing effect using interval (more efficient than recursive setTimeout)
  useEffect(() => {
    if (!hasStarted) return;

    const charsArray = charsArrayRef.current;
    
    // If text is empty or already complete, finish immediately
    if (charsArray.length === 0) {
      setIsComplete(true);
      onCompleteRef.current?.();
      return;
    }

    intervalRef.current = setInterval(() => {
      if (charIndexRef.current < charsArray.length) {
        charIndexRef.current += 1;
        setDisplayedText(charsArray.slice(0, charIndexRef.current).join(''));
      } else {
        // Done typing
        if (intervalRef.current) {
          clearInterval(intervalRef.current);
          intervalRef.current = null;
        }
        setIsComplete(true);
        onCompleteRef.current?.();
      }
    }, speed);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [hasStarted, speed]);

  return (
    <span 
      ref={containerRef}
      className={`${className} ${isComplete ? '' : 'streaming-content'}`}
    >
      {displayedText}
      {hasStarted && !isComplete && (
        <span className="typewriter-cursor" />
      )}
    </span>
  );
}

export default memo(TypewriterText);
