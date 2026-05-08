import React, { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';
import { Button } from './ui/Button';

// Error Boundary Styles using Tailwind classes
const styles = {
  container: 'min-h-[300px] flex items-center justify-center p-8',
  card: 'max-w-md w-full bg-surface border border-error/30 rounded-xl p-8 text-center shadow-xl',
  iconContainer: 'w-16 h-16 mx-auto mb-6 rounded-full bg-error/10 flex items-center justify-center',
  icon: 'w-8 h-8 text-error',
  title: 'text-xl font-bold text-text-primary mb-2',
  description: 'text-text-secondary mb-6',
  errorDetails: 'mb-6',
  summary: 'text-xs text-text-muted cursor-pointer hover:text-text-secondary',
  errorBox: 'mt-2 p-3 bg-background rounded-lg border border-border',
  errorMessage: 'text-xs font-mono text-error break-all',
  buttonGroup: 'flex flex-col gap-3',
  buttonRow: 'flex gap-3',
  supportText: 'mt-6 text-xs text-text-muted',
};

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onReset?: () => void;
  componentName?: string;
}

interface State {
  hasError: boolean;
  error?: Error;
}

/**
 * Error Boundary Component
 * Catches JavaScript errors in child component tree and displays fallback UI
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    
    // Report to backend (optional - can be enabled when API is ready)
    this.reportError(error, errorInfo);
  }

  private async reportError(error: Error, errorInfo: ErrorInfo) {
    try {
      // Only report in production and if endpoint exists
      if (import.meta.env.PROD) {
        await fetch('/api/report-error', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            error: error.message,
            stack: error.stack,
            componentStack: errorInfo.componentStack,
            componentName: this.props.componentName || 'Unknown',
            timestamp: new Date().toISOString(),
          }),
        });
      }
    } catch (e) {
      // Silent fail for error reporting
      console.warn('Failed to report error:', e);
    }
  }

  handleReset = () => {
    this.setState({ hasError: false, error: undefined });
    this.props.onReset?.();
  };

  handleReload = () => {
    window.location.reload();
  };

  handleGoHome = () => {
    window.location.href = '/';
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Default error UI
      return (
        <div className={styles.container}>
          <div className={styles.card}>
            {/* Error Icon */}
            <div className={styles.iconContainer}>
              <AlertTriangle className={styles.icon} />
            </div>

            {/* Title */}
            <h2 className={styles.title}>
              Something went wrong
            </h2>

            {/* Description */}
            <p className={styles.description}>
              {this.props.componentName 
                ? `An error occurred in the ${this.props.componentName} component.`
                : 'An unexpected error occurred while rendering this page.'}
            </p>

            {/* Error Details (collapsed by default) */}
            {this.state.error && (
              <div className={styles.errorDetails}>
                <details className="text-left">
                  <summary className={styles.summary}>
                    Error details
                  </summary>
                  <div className={styles.errorBox}>
                    <p className={styles.errorMessage}>
                      {this.state.error.message}
                    </p>
                  </div>
                </details>
              </div>
            )}

            {/* Action Buttons */}
            <div className={styles.buttonGroup}>
              <Button
                onClick={this.handleReset}
                leftIcon={<RefreshCw className="w-4 h-4" />}
                className="w-full"
              >
                Try Again
              </Button>
              
              <div className={styles.buttonRow}>
                <Button
                  variant="secondary"
                  onClick={this.handleReload}
                  className="flex-1"
                >
                  Reload Page
                </Button>
                
                <Button
                  variant="ghost"
                  onClick={this.handleGoHome}
                  leftIcon={<Home className="w-4 h-4" />}
                  className="flex-1"
                >
                  Go Home
                </Button>
              </div>
            </div>

            {/* Support Link */}
            <p className={styles.supportText}>
              If this persists, please contact support with the error details above.
            </p>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Page-level Error Boundary wrapper
 * Simplifies usage for page components
 */
export const withErrorBoundary = <P extends object>(
  Component: React.ComponentType<P>,
  componentName: string
): React.FC<P> => {
  return (props: P) => (
    <ErrorBoundary componentName={componentName}>
      <Component {...props} />
    </ErrorBoundary>
  );
};

export default ErrorBoundary;
