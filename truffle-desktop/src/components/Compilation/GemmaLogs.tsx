// SPEC v2.0 SECTION 5.2.2: Gemma 4 Logs Component
// Shows compilation logs from the AI model

import React, { useState, useRef, useEffect } from 'react';
import { 
  Terminal, 
  Download, 
  Trash2, 
  Filter,
  ChevronDown,
  Clock,
  CheckCircle2,
  AlertCircle,
  Info
} from 'lucide-react';

interface LogEntry {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warning' | 'error';
  message: string;
  metadata?: Record<string, unknown>;
}

interface GemmaLogsProps {
  artifactId: string | null;
  className?: string;
}

export const GemmaLogs: React.FC<GemmaLogsProps> = ({ 
  artifactId,
  className = '' 
}) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<'all' | 'info' | 'warning' | 'error'>('all');
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Mock logs - would come from API
  useEffect(() => {
    if (!artifactId) {
      setLogs([]);
      return;
    }

    // Simulate logs
    const mockLogs: LogEntry[] = [
      {
        id: '1',
        timestamp: new Date(Date.now() - 5000).toISOString(),
        level: 'info',
        message: 'Starting compilation for artifact: ' + artifactId,
      },
      {
        id: '2',
        timestamp: new Date(Date.now() - 4500).toISOString(),
        level: 'info',
        message: 'Loading Gemma 4 E2B model...',
      },
      {
        id: '3',
        timestamp: new Date(Date.now() - 4000).toISOString(),
        level: 'info',
        message: 'Model loaded successfully (2.1s)',
      },
      {
        id: '4',
        timestamp: new Date(Date.now() - 3500).toISOString(),
        level: 'info',
        message: 'Preprocessing image (resize to 896px)...',
      },
      {
        id: '5',
        timestamp: new Date(Date.now() - 3000).toISOString(),
        level: 'info',
        message: 'Running OCR...',
      },
      {
        id: '6',
        timestamp: new Date(Date.now() - 2500).toISOString(),
        level: 'info',
        message: 'OCR complete. Detected text: "Receipt from Coffee Shop"',
      },
      {
        id: '7',
        timestamp: new Date(Date.now() - 2000).toISOString(),
        level: 'info',
        message: 'Applying schema rules...',
      },
      {
        id: '8',
        timestamp: new Date(Date.now() - 1500).toISOString(),
        level: 'info',
        message: 'Matched rule: receipt detection',
      },
      {
        id: '9',
        timestamp: new Date(Date.now() - 1000).toISOString(),
        level: 'info',
        message: 'Extracting entities: merchant, total, date, items...',
      },
      {
        id: '10',
        timestamp: new Date(Date.now() - 500).toISOString(),
        level: 'info',
        message: 'Creating wiki nodes...',
      },
      {
        id: '11',
        timestamp: new Date().toISOString(),
        level: 'info',
        message: 'Compilation complete. Created 3 wiki nodes.',
      },
    ];

    setLogs(mockLogs);
  }, [artifactId]);

  // Auto-scroll to bottom
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  // Filter logs
  const filteredLogs = logs.filter(log => {
    if (filter === 'all') return true;
    return log.level === filter;
  });

  // Get level icon
  const getLevelIcon = (level: string) => {
    switch (level) {
      case 'error':
        return <AlertCircle className="w-3.5 h-3.5 text-error" />;
      case 'warning':
        return <AlertCircle className="w-3.5 h-3.5 text-warning" />;
      case 'info':
        return <Info className="w-3.5 h-3.5 text-secondary" />;
      default:
        return <CheckCircle2 className="w-3.5 h-3.5 text-text-muted" />;
    }
  };

  // Get level color
  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return 'text-error';
      case 'warning':
        return 'text-warning';
      case 'info':
        return 'text-secondary';
      default:
        return 'text-text-muted';
    }
  };

  // Handle export
  const handleExport = () => {
    const content = logs.map(log => 
      `[${log.timestamp}] [${log.level.toUpperCase()}] ${log.message}`
    ).join('\n');
    
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `compilation-logs-${artifactId || 'all'}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Handle clear
  const handleClear = () => {
    setLogs([]);
  };

  if (!artifactId) {
    return (
      <div className={`flex items-center justify-center h-full text-text-muted ${className}`}>
        <p>Select an artifact to view logs</p>
      </div>
    );
  }

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-surface/50">
        <div className="flex items-center gap-2">
          <Terminal className="w-4 h-4 text-text-muted" />
          <span className="text-xs font-medium text-text-secondary">
            Gemma 4 Logs
          </span>
          <span className="text-xs text-text-muted">
            ({filteredLogs.length} entries)
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Filter Dropdown */}
          <div className="relative group">
            <button className="flex items-center gap-1 px-2 py-1 rounded text-xs text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors">
              <Filter className="w-3.5 h-3.5" />
              <span className="capitalize">{filter}</span>
              <ChevronDown className="w-3 h-3" />
            </button>
            
            <div className="absolute top-full right-0 mt-1 w-24 bg-surface-elevated border border-border rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-50">
              {(['all', 'info', 'warning', 'error'] as const).map((f) => (
                <button
                  key={f}
                  onClick={() => setFilter(f)}
                  className={`
                    w-full px-3 py-1.5 text-left text-xs capitalize
                    ${filter === f 
                      ? 'text-primary bg-primary/10' 
                      : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
                    }
                  `}
                >
                  {f}
                </button>
              ))}
            </div>
          </div>

          {/* Auto-scroll Toggle */}
          <button
            onClick={() => setAutoScroll(!autoScroll)}
            className={`
              px-2 py-1 rounded text-xs transition-colors
              ${autoScroll 
                ? 'bg-primary/10 text-primary' 
                : 'text-text-secondary hover:text-text-primary'
              }
            `}
          >
            Auto-scroll
          </button>

          {/* Export */}
          <button
            onClick={handleExport}
            className="p-1.5 rounded text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
            title="Export logs"
          >
            <Download className="w-3.5 h-3.5" />
          </button>

          {/* Clear */}
          <button
            onClick={handleClear}
            className="p-1.5 rounded text-text-secondary hover:text-error hover:bg-error/10 transition-colors"
            title="Clear logs"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Logs */}
      <div 
        ref={containerRef}
        className="flex-1 overflow-auto p-2 font-mono text-xs"
      >
        {filteredLogs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-text-muted">
            <p>No logs available</p>
          </div>
        ) : (
          <div className="space-y-1">
            {filteredLogs.map((log) => (
              <div 
                key={log.id}
                className="flex items-start gap-2 p-1.5 rounded hover:bg-surface/50 transition-colors"
              >
                <div className="flex-shrink-0 mt-0.5">
                  {getLevelIcon(log.level)}
                </div>
                
                <div className="flex-shrink-0 text-text-muted">
                  <Clock className="w-3 h-3 inline mr-1" />
                  {new Date(log.timestamp).toLocaleTimeString()}
                </div>
                
                <div className={getLevelColor(log.level)}>
                  [{log.level.toUpperCase()}]
                </div>
                
                <div className="flex-1 text-text-primary break-all">
                  {log.message}
                </div>
              </div>
            ))}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>
    </div>
  );
};

export default GemmaLogs;
