import { FileIcon } from './icons';

interface DiffButtonProps {
  onClick: () => void;
}

export const DiffButton = ({ onClick }: DiffButtonProps) => (
  <button
    onClick={onClick}
    className="flex items-center gap-1.5 px-3 py-1 rounded-md border border-text-secondary/30 text-text-secondary text-xs hover:bg-bg-card transition-colors"
  >
    <FileIcon size={14} />
    Diff
  </button>
);
