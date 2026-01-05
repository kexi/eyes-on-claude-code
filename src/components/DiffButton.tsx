import { FileIcon } from './icons';

interface DiffButtonProps {
  onClick: () => void;
  small?: boolean;
  className?: string;
}

export const DiffButton = ({ onClick, small, className = '' }: DiffButtonProps) => (
  <button
    onClick={onClick}
    className={`flex items-center gap-1 rounded-md border border-text-secondary/30 text-text-secondary hover:bg-bg-card transition-colors ${
      small ? 'px-1.5 py-0.5 text-[0.5rem]' : 'px-3 py-1 text-xs gap-1.5'
    } ${className}`}
  >
    <FileIcon size={small ? 10 : 14} />
    Diff
  </button>
);
