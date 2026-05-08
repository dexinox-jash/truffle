// Main entry point for Truffle Desktop

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

// Add class to body when app loads
const markAppLoaded = () => {
  document.body.classList.add('app-loaded');
};

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

// Mark app as loaded after initial render
markAppLoaded();

// Register service worker for offline support (PWA)
if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker
      .register('/sw.js')
      .then((registration) => {
        console.log('SW registered:', registration);
      })
      .catch((error) => {
        console.log('SW registration failed:', error);
      });
  });
}

// Handle Tauri-specific initialization
if (window.__TAURI__) {
  console.log('Running in Tauri environment');
  
  // Set up Tauri event listeners
  // This will be used for native notifications, file drops, etc.
}

// Expose app version for debugging
console.log(
  '%c Truffle Desktop ',
  'background: #FFB800; color: #0A0A0A; font-weight: bold; padding: 4px 8px; border-radius: 4px;'
);
console.log('Version:', import.meta.env.VITE_APP_VERSION || 'dev');

// Add global error handler
window.addEventListener('error', (event) => {
  console.error('Global error:', event.error);
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('Unhandled promise rejection:', event.reason);
});

// Type augmentation for Tauri
declare global {
  interface Window {
    __TAURI__?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      event: {
        listen: (event: string, handler: (event: any) => void) => Promise<() => void>;
      };
    };
  }
}
