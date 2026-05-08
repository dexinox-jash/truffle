// SPEC v2.0 SECTION 5.2.2: Drag & Drop Zone Component
// File drop zone for importing screenshots

import React, { useCallback, useState } from 'react';
import { Upload, X, Image, FileImage, FolderOpen } from 'lucide-react';

interface DragDropZoneProps {
  isActive: boolean;
  onClose: () => void;
  className?: string;
}

export const DragDropZone: React.FC<DragDropZoneProps> = ({
  isActive,
  onClose,
  className = '',
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [droppedFiles, setDroppedFiles] = useState<File[]>([]);

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files).filter(
      file => file.type.startsWith('image/')
    );

    if (files.length > 0) {
      setDroppedFiles(prev => [...prev, ...files]);
    }
  }, []);

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || []).filter(
      file => file.type.startsWith('image/')
    );

    if (files.length > 0) {
      setDroppedFiles(prev => [...prev, ...files]);
    }
  }, []);

  const handleImport = useCallback(async () => {
    if (droppedFiles.length === 0) return;

    // TODO: Import files via Tauri API
    console.log('Importing files:', droppedFiles);
    
    // Reset and close
    setDroppedFiles([]);
    onClose();
  }, [droppedFiles, onClose]);

  const removeFile = useCallback((index: number) => {
    setDroppedFiles(prev => prev.filter((_, i) => i !== index));
  }, []);

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div 
      className={`
        absolute inset-0 z-50 flex items-center justify-center
        bg-background/90 backdrop-blur-sm
        ${className}
      `}
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      <div className="relative w-full max-w-lg mx-4">
        {/* Close Button */}
        <button
          onClick={onClose}
          className="absolute -top-10 right-0 p-2 text-text-muted hover:text-text-primary transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Drop Zone */}
        <div
          className={`
            border-2 border-dashed rounded-xl p-8
            transition-all duration-200
            ${isDragging 
              ? 'border-primary bg-primary/5 scale-[1.02]' 
              : 'border-border bg-surface'
            }
          `}
        >
          {droppedFiles.length === 0 ? (
            <div className="text-center">
              <div className={`
                w-20 h-20 mx-auto mb-4 rounded-full 
                flex items-center justify-center
                transition-all duration-200
                ${isDragging ? 'bg-primary/20' : 'bg-surface-elevated'}
              `}>
                <Upload className={`
                  w-10 h-10 transition-colors
                  ${isDragging ? 'text-primary' : 'text-text-muted'}
                `} />
              </div>
              
              <h3 className="text-lg font-medium text-text-primary mb-2">
                {isDragging ? 'Drop to import' : 'Drag screenshots here'}
              </h3>
              
              <p className="text-sm text-text-muted mb-4">
                or click to browse your files
              </p>
              
              <label className="inline-flex items-center gap-2 px-4 py-2 bg-primary text-background rounded-lg font-medium cursor-pointer hover:bg-primary-hover transition-colors">
                <FolderOpen className="w-4 h-4" />
                <span>Browse Files</span>
                <input
                  type="file"
                  multiple
                  accept="image/*"
                  className="hidden"
                  onChange={handleFileSelect}
                />
              </label>
              
              <p className="text-xs text-text-muted mt-4">
                Supports PNG, JPG, WebP, HEIC
              </p>
            </div>
          ) : (
            <div>
              <h3 className="text-lg font-medium text-text-primary mb-4">
                {droppedFiles.length} file{droppedFiles.length > 1 ? 's' : ''} ready to import
              </h3>
              
              {/* File List */}
              <div className="max-h-48 overflow-auto space-y-2 mb-4">
                {droppedFiles.map((file, index) => (
                  <div 
                    key={index}
                    className="flex items-center gap-3 p-2 bg-background rounded-lg"
                  >
                    <div className="w-10 h-10 rounded bg-surface-elevated flex items-center justify-center flex-shrink-0">
                      <Image className="w-5 h-5 text-text-muted" />
                    </div>
                    
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-text-primary truncate">
                        {file.name}
                      </p>
                      <p className="text-xs text-text-muted">
                        {formatFileSize(file.size)}
                      </p>
                    </div>
                    
                    <button
                      onClick={() => removeFile(index)}
                      className="p-1.5 text-text-muted hover:text-error transition-colors"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                ))}
              </div>
              
              {/* Actions */}
              <div className="flex gap-3">
                <button
                  onClick={() => setDroppedFiles([])}
                  className="flex-1 px-4 py-2 border border-border rounded-lg text-text-secondary hover:text-text-primary hover:border-border-hover transition-colors"
                >
                  Clear All
                </button>
                <button
                  onClick={handleImport}
                  className="flex-1 px-4 py-2 bg-primary text-background rounded-lg font-medium hover:bg-primary-hover transition-colors"
                >
                  Import {droppedFiles.length} File{droppedFiles.length > 1 ? 's' : ''}
                </button>
              </div>
            </div>
          )}
        </div>
        
        {/* Drag Active Indicator */}
        {isDragging && (
          <div className="absolute inset-0 pointer-events-none">
            <div className="absolute inset-0 border-2 border-primary rounded-xl animate-pulse" />
          </div>
        )}
      </div>
    </div>
  );
};

export default DragDropZone;
