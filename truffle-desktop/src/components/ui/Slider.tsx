import React from 'react';
import { cn } from '../../utils/cn';

interface SliderProps {
  min: number;
  max: number;
  value: number;
  onChange: (value: number) => void;
  className?: string;
}

export const Slider: React.FC<SliderProps> = ({
  min,
  max,
  value,
  onChange,
  className,
}) => {
  const percentage = ((value - min) / (max - min)) * 100;

  return (
    <div className={cn('relative w-full h-2 bg-darkroom-gray-700 rounded-full', className)}>
      {/* Track fill */}
      <div
        className="absolute h-full bg-accent rounded-full"
        style={{ width: `${percentage}%` }}
      />
      
      {/* Input */}
      <input
        type="range"
        min={min}
        max={max}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
      />
      
      {/* Thumb */}
      <div
        className="absolute top-1/2 -translate-y-1/2 w-4 h-4 bg-accent rounded-full shadow-lg pointer-events-none transition-transform"
        style={{ left: `calc(${percentage}% - 8px)` }}
      />
    </div>
  );
};

export default Slider;
