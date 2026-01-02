interface EmptyStateProps {
  icon: string;
  message: string;
  isMiniView?: boolean;
}

export const EmptyState = ({ icon, message, isMiniView = false }: EmptyStateProps) => {
  return (
    <div className={`text-center text-text-secondary ${isMiniView ? 'py-4' : 'py-10'}`}>
      <div className={`${isMiniView ? 'text-2xl mb-1.5' : 'text-5xl mb-4'}`}>{icon}</div>
      <p className={isMiniView ? 'text-[0.625rem]' : ''}>{message}</p>
    </div>
  );
};
