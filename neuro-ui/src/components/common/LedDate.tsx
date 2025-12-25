import { memo } from 'react';

interface LedDateProps {
  readonly date: Date | string;
  readonly format?: 'full' | 'date' | 'time' | 'relative';
  readonly className?: string;
  readonly showIcon?: boolean;
}

function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return `${diffSecs}s`;
  if (diffMins < 60) return `${diffMins}m`;
  if (diffHours < 24) return `${diffHours}h`;
  if (diffDays < 7) return `${diffDays}d`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo`;
  return `${Math.floor(diffDays / 365)}y`;
}

function formatDateTime(date: Date, format: LedDateProps['format']): string {
  const pad = (n: number) => n.toString().padStart(2, '0');
  
  const day = pad(date.getDate());
  const month = pad(date.getMonth() + 1);
  const year = date.getFullYear().toString().slice(-2);
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  const seconds = pad(date.getSeconds());

  switch (format) {
    case 'time':
      return `${hours}:${minutes}:${seconds}`;
    case 'date':
      return `${day}/${month}/${year}`;
    case 'relative':
      return formatRelativeTime(date);
    case 'full':
    default:
      return `${day}/${month}/${year} ${hours}:${minutes}`;
  }
}

function LedDate({
  date,
  format = 'full',
  className = '',
  showIcon = false,
}: LedDateProps) {
  const dateObj = typeof date === 'string' ? new Date(date) : date;
  const formattedDate = formatDateTime(dateObj, format);

  return (
    <span 
      className={`led-date inline-flex items-center gap-1 ${className}`}
      title={dateObj.toLocaleString()}
    >
      {showIcon && (
        <svg className="w-3 h-3 opacity-70" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      )}
      <span className="led-digits">{formattedDate}</span>
    </span>
  );
}

export default memo(LedDate);
