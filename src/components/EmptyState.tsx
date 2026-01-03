interface EmptyStateProps {
  icon: string;
  message: string;
}

export const EmptyState = ({ icon, message }: EmptyStateProps) => {
  return (
    <div className="text-center text-text-secondary py-4">
      <div className="text-2xl mb-1.5">{icon}</div>
      <p className="text-[0.625rem]">{message}</p>
    </div>
  );
};
