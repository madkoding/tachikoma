import { memo, useEffect, useState, useRef } from 'react';

interface AnimatedLedDigitsProps {
  /** El valor a mostrar (puede contener números y separadores como : / - ) */
  readonly value: string;
  /** Duración de la animación en ms */
  readonly duration?: number;
  /** Clase CSS adicional */
  readonly className?: string;
  /** Variante del estilo LED */
  readonly variant?: 'default' | 'subtle' | 'cyan' | 'large' | 'time';
  /** Si debe animar cada vez que cambia el valor */
  readonly animateOnChange?: boolean;
  /** Si debe animar al aparecer (default: true) */
  readonly animate?: boolean;
}

/**
 * Extrae los números de un string y devuelve sus posiciones
 */
function extractNumbers(str: string): { numbers: number[]; positions: number[] } {
  const numbers: number[] = [];
  const positions: number[] = [];
  
  for (let i = 0; i < str.length; i++) {
    const char = str[i];
    if (/\d/.test(char)) {
      numbers.push(parseInt(char, 10));
      positions.push(i);
    }
  }
  
  return { numbers, positions };
}

/**
 * Reconstruye el string con los números animados
 */
function reconstructString(
  original: string,
  animatedNumbers: number[],
  positions: number[]
): string {
  const chars = original.split('');
  
  for (let i = 0; i < positions.length; i++) {
    const pos = positions[i];
    chars[pos] = animatedNumbers[i].toString();
  }
  
  return chars.join('');
}

/**
 * Genera el texto ghost para el efecto de segmentos apagados
 * Cada dígito se reemplaza por "8", los ":" se mantienen
 */
function generateGhostText(text: string): string {
  return text.replace(/\d/g, '8');
}

/**
 * Renderiza un dígito o grupo con efecto ghosting LED
 */
function renderWithGhosting(
  text: string,
  ledClass: string,
  isAnimating: boolean
): React.ReactNode {
  const ghostText = generateGhostText(text);
  
  return (
    <span className="relative inline-block">
      {/* Ghost layer - segmentos "apagados" */}
      <span 
        className={`${ledClass} opacity-[0.15] select-none`}
        aria-hidden="true"
      >
        {ghostText}
      </span>
      {/* Active layer - segmentos encendidos */}
      <span 
        className={`${ledClass} ${isAnimating ? 'led-digits-animated' : ''} absolute inset-0`}
      >
        {text}
      </span>
    </span>
  );
}

/**
 * Renderiza el texto separando dígitos (con estilo LED) de letras (estilo normal)
 */
function renderWithSeparatedStyles(
  text: string,
  ledClass: string,
  isAnimating: boolean
): React.ReactNode {
  // Dividir el texto en grupos de dígitos y separadores de tiempo vs texto normal
  const parts = text.match(/(\d+[:\d]*\d*|[^\d]+)/g) || [text];
  
  return parts.map((part, index) => {
    // Verificar si es un patrón de dígitos (posiblemente con :)
    const isDigitPattern = /^[\d:]+$/.test(part);
    if (isDigitPattern) {
      return (
        <span key={index}>
          {renderWithGhosting(part, ledClass, isAnimating)}
        </span>
      );
    }
    // Las letras usan un estilo más suave, similar al texto "canciones"
    return (
      <span
        key={index}
        className="text-gray-400 font-mono text-[0.85em] mx-0.5"
      >
        {part}
      </span>
    );
  });
}

/**
 * Componente que anima dígitos LED de 0 hasta el valor final
 */
function AnimatedLedDigits({
  value,
  duration = 1000,
  className = '',
  variant = 'default',
  animateOnChange = false,
  animate = true,
}: AnimatedLedDigitsProps) {
  const [displayValue, setDisplayValue] = useState(animate ? value : value);
  const [isAnimating, setIsAnimating] = useState(false);
  const animationRef = useRef<number | null>(null);
  const hasAnimatedRef = useRef(!animate); // Si no anima, marcar como ya animado
  const previousValueRef = useRef(value);
  const elementRef = useRef<HTMLSpanElement>(null);

  // Obtener la clase CSS según la variante
  const getVariantClass = () => {
    switch (variant) {
      case 'subtle':
        return 'led-digits-subtle';
      case 'cyan':
        return 'led-digits-cyan';
      case 'large':
        return 'led-digits-large';
      case 'time':
        return 'led-time';
      default:
        return 'led-digits';
    }
  };

  useEffect(() => {
    // Si el valor no ha cambiado y ya animamos, no hacer nada
    if (!animateOnChange && hasAnimatedRef.current && value === previousValueRef.current) {
      return;
    }

    // Si animateOnChange es false y ya animamos una vez, solo actualizar el valor
    if (!animateOnChange && hasAnimatedRef.current) {
      setDisplayValue(value);
      previousValueRef.current = value;
      return;
    }

    // Si animateOnChange es true y el valor no cambió, no animar
    if (animateOnChange && value === previousValueRef.current && hasAnimatedRef.current) {
      return;
    }

    const { numbers: targetNumbers, positions } = extractNumbers(value);
    
    // Si no hay números que animar, mostrar directamente
    if (targetNumbers.length === 0) {
      setDisplayValue(value);
      hasAnimatedRef.current = true;
      previousValueRef.current = value;
      return;
    }

    // Iniciar animación
    setIsAnimating(true);
    const startTime = performance.now();
    const startNumbers = targetNumbers.map(() => 0);

    const animate = (currentTime: number) => {
      const elapsed = currentTime - startTime;
      const progress = Math.min(elapsed / duration, 1);
      
      // Easing function: easeOutExpo para un efecto más natural
      const eased = progress === 1 ? 1 : 1 - Math.pow(2, -10 * progress);
      
      // Calcular números actuales
      const currentNumbers = targetNumbers.map((target, i) => {
        const start = startNumbers[i];
        return Math.round(start + (target - start) * eased);
      });
      
      // Reconstruir string con números animados
      const newDisplayValue = reconstructString(value, currentNumbers, positions);
      setDisplayValue(newDisplayValue);
      
      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate);
      } else {
        setIsAnimating(false);
        hasAnimatedRef.current = true;
        previousValueRef.current = value;
      }
    };

    // Cancelar animación anterior si existe
    if (animationRef.current) {
      cancelAnimationFrame(animationRef.current);
    }

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [value, duration, animateOnChange]);

  // Intersection Observer para animar cuando el elemento entra en viewport
  useEffect(() => {
    if (hasAnimatedRef.current) return;

    const element = elementRef.current;
    if (!element) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && !hasAnimatedRef.current) {
            // Forzar re-render para iniciar animación
            const { numbers: targetNumbers, positions } = extractNumbers(value);
            
            if (targetNumbers.length === 0) {
              setDisplayValue(value);
              hasAnimatedRef.current = true;
              return;
            }

            setIsAnimating(true);
            const startTime = performance.now();

            const animate = (currentTime: number) => {
              const elapsed = currentTime - startTime;
              const progress = Math.min(elapsed / duration, 1);
              const eased = progress === 1 ? 1 : 1 - Math.pow(2, -10 * progress);
              
              const currentNumbers = targetNumbers.map((target) => {
                return Math.round(target * eased);
              });
              
              const newDisplayValue = reconstructString(value, currentNumbers, positions);
              setDisplayValue(newDisplayValue);
              
              if (progress < 1) {
                animationRef.current = requestAnimationFrame(animate);
              } else {
                setIsAnimating(false);
                hasAnimatedRef.current = true;
                previousValueRef.current = value;
              }
            };

            if (animationRef.current) {
              cancelAnimationFrame(animationRef.current);
            }
            animationRef.current = requestAnimationFrame(animate);
          }
        });
      },
      { threshold: 0.1 }
    );

    observer.observe(element);

    return () => {
      observer.disconnect();
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [value, duration]);

  const ledClass = getVariantClass();

  return (
    <span
      ref={elementRef}
      className={`inline-flex items-baseline ${className}`}
    >
      {renderWithSeparatedStyles(displayValue, ledClass, isAnimating)}
    </span>
  );
}

export default memo(AnimatedLedDigits);
