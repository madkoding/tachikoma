import { memo } from 'react';

interface AnimatedLedDigitsProps {
  readonly value: string;
  readonly duration?: number;
  readonly className?: string;
  readonly variant?: 'default' | 'subtle' | 'cyan' | 'large' | 'time';
  readonly animateOnChange?: boolean;
  readonly animate?: boolean;
}

// ponytail: full LED animation replaced with plain mono span; props kept for compatibility
function AnimatedLedDigits({ value, className = '' }: AnimatedLedDigitsProps) {
  return <span className={`font-mono ${className}`}>{value}</span>;
}

export default memo(AnimatedLedDigits);