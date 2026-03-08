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
  
  // Single ref to track if component is mounted (prevents state updates after unmount)
  const isMountedRef = useRef(true);
  const onCompleteRef = useRef(onComplete);
  
  // Keep onComplete ref updated
  useEffect(() => {
    onCompleteRef.current = onComplete;
  }, [onComplete]);

  // Single effect that handles the entire typewriter lifecycle
  useEffect(() => {
    // Mark as mounted
    isMountedRef.current = true;
    
    // Reset state for new text
    setDisplayedText('');
    setIsComplete(false);
    setHasStarted(false);
    
    let charIndex = 0;
    let intervalId: ReturnType<typeof setInterval> | null = null;
    let delayTimeoutId: ReturnType<typeof setTimeout> | null = null;
    
    // If text is empty, complete immediately
    if (text.length === 0) {
      setIsComplete(true);
      onCompleteRef.current?.();
      return;
    }

    // Start typing after delay
    delayTimeoutId = setTimeout(() => {
      if (!isMountedRef.current) return;
      
      setHasStarted(true);
      
      // Start the typing interval
      intervalId = setInterval(() => {
        if (!isMountedRef.current) {
          if (intervalId) clearInterval(intervalId);
          return;
        }
        
        if (charIndex < text.length) {
          charIndex += 1;
          setDisplayedText(text.substring(0, charIndex));
        } else {
          // Done typing
          if (intervalId) {
            clearInterval(intervalId);
            intervalId = null;
          }
          setIsComplete(true);
          onCompleteRef.current?.();
        }
      }, speed);
    }, delay);

    // Cleanup function - called on unmount or when dependencies change
    return () => {
      isMountedRef.current = false;
      
      if (delayTimeoutId) {
        clearTimeout(delayTimeoutId);
        delayTimeoutId = null;
      }
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    };
  }, [text, delay, speed]); // All dependencies in one effect

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
