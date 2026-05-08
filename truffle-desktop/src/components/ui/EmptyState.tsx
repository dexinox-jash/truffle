import React from 'react';

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  description?: string;
  action?: React.ReactNode;
}

export const EmptyState: React.FC<EmptyStateProps> = ({
  icon,
  title,
  description,
  action,
}) => {
  return (
    <div className="flex flex-col items-center justify-center h-full min-h-[300px] p-8 text-center">
      {icon && (
        <div className="text-darkroom-text-tertiary mb-4">
          {icon}
        </div>
      )}
      <h3 className="text-lg font-semibold text-darkroom-text-primary mb-2">
        {title}
      </h3>
      {description && (
        <p className="text-sm text-darkroom-text-secondary max-w-sm mb-6">
          {description}
        </p>
      )}
      {action && <div>{action}</div>}
    </div>
  );
};

export default EmptyState;
