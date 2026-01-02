interface StatCardProps {
  value: number;
  label: string;
}

export const StatCard = ({ value, label }: StatCardProps) => {
  return (
    <div className="bg-bg-secondary p-5 rounded-xl text-center">
      <div className="text-3xl font-bold text-accent">{value}</div>
      <div className="text-sm text-text-secondary mt-1">{label}</div>
    </div>
  );
};
