// SPEC v2.0 SECTION 5.2: Main Application Component
// Truffle Desktop - Local-First Knowledge Compiler

import React, { useEffect } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AppLayout } from './components/Layout/AppLayout';
import { ErrorBoundary } from './components/ErrorBoundary';
import { useAppStore } from './store/appStore';
import { useSearchStore } from './store/searchStore';
import { useSyncStore, createYDoc } from './store/syncStore';

// Page Components
import { EntitiesPage } from './pages/EntitiesPage';
import { GraphPage } from './pages/GraphPage';
import { ValidationPage } from './pages/ValidationPage';
import { MeetingsPage } from './pages/MeetingsPage';

// Import styles
import './styles/globals.css';
import './styles/animations.css';

// Create Query Client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 30, // 30 minutes
      retry: 3,
      refetchOnWindowFocus: false,
    },
  },
});

// ==================== APP INITIALIZATION ====================

const AppInitializer: React.FC = () => {
  const { 
    setArtifacts, 
    setWikiNodes, 
    setCompilationJobs,
    setSyncStatus,
    setLastSyncAt,
  } = useAppStore();
  
  const { initializeIndexes } = useSearchStore();
  const { setYDoc } = useSyncStore();

  useEffect(() => {
    // Initialize search indexes
    initializeIndexes();

    // Initialize Yjs document
    const ydoc = createYDoc();
    setYDoc(ydoc);

    // Load initial data (mock for now)
    loadInitialData();

    // Set up Tauri event listeners
    setupTauriListeners();

    console.log('Truffle Desktop initialized');
  }, []);

  const loadInitialData = () => {
    // Mock artifacts
    const mockArtifacts = [
      {
        id: '1',
        filename: 'receipt_coffee_shop.png',
        capturedAt: new Date(Date.now() - 86400000).toISOString(),
        status: 'compiled' as const,
        appContext: { appName: 'Photos', windowTitle: 'Screenshot' },
      },
      {
        id: '2',
        filename: 'flight_confirmation.png',
        capturedAt: new Date(Date.now() - 172800000).toISOString(),
        status: 'pending' as const,
        appContext: { appName: 'Mail', windowTitle: 'Inbox' },
      },
      {
        id: '3',
        filename: 'meeting_notes.png',
        capturedAt: new Date(Date.now() - 259200000).toISOString(),
        status: 'pending' as const,
        appContext: { appName: 'Notes', windowTitle: 'Meeting Notes' },
      },
    ];

    // Mock wiki nodes
    const mockWikiNodes = [
      {
        id: '1',
        title: 'Coffee Shop Receipt',
        content: `---
date: 2024-01-15
type: receipt
merchant: "Coffee Shop"
total: 12.50
---

# Coffee Shop Receipt

**Merchant:** Coffee Shop  
**Date:** January 15, 2024  
**Total:** $12.50

## Items

- Coffee - $4.50
- Pastry - $5.00
- Tip - $3.00

## Related

- [[Expenses/2024-01]]
- [[Coffee Shop]]`,
        nodeType: 'entity' as const,
        updatedAt: new Date(Date.now() - 86400000).toISOString(),
        backlinkCount: 2,
        forwardLinkCount: 2,
        excerpt: 'Receipt from Coffee Shop dated January 15, 2024',
      },
      {
        id: '2',
        title: 'Expenses/2024-01',
        content: `---
type: index
month: 2024-01
---

# January 2024 Expenses

## Receipts

- [[Coffee Shop Receipt]] - $12.50
- [[Grocery Store]] - $87.32

## Total: $99.82`,
        nodeType: 'index' as const,
        updatedAt: new Date(Date.now() - 172800000).toISOString(),
        backlinkCount: 1,
        forwardLinkCount: 2,
        excerpt: 'Monthly expense tracking for January 2024',
      },
      {
        id: '3',
        title: 'Coffee Shop',
        content: `---
type: entity
location: "123 Main St"
---

# Coffee Shop

Favorite local coffee shop.

## Visits

- [[Coffee Shop Receipt|2024-01-15]]

## Notes

Great espresso, friendly staff.`,
        nodeType: 'entity' as const,
        updatedAt: new Date(Date.now() - 259200000).toISOString(),
        backlinkCount: 1,
        forwardLinkCount: 1,
        excerpt: 'Favorite local coffee shop on Main Street',
      },
    ];

    // Mock compilation jobs
    const mockJobs = [
      {
        id: '1',
        artifactId: '1',
        status: 'completed' as const,
        progress: 100,
        startedAt: new Date(Date.now() - 86400000).toISOString(),
        completedAt: new Date(Date.now() - 86400000 + 5000).toISOString(),
      },
    ];

    setArtifacts(mockArtifacts);
    setWikiNodes(mockWikiNodes);
    setCompilationJobs(mockJobs);

    // Index content for search
    const { indexArtifact, indexWikiNode } = useSearchStore.getState();
    mockArtifacts.forEach(artifact => {
      indexArtifact({
        id: artifact.id,
        filename: artifact.filename,
        ocrText: '',
      });
    });
    mockWikiNodes.forEach(node => {
      indexWikiNode({
        id: node.id,
        title: node.title,
        content: node.content,
      });
    });
  };

  const setupTauriListeners = () => {
    // Listen for Tauri events
    // This would be implemented when Tauri APIs are available
    
    // Example:
    // listen('compilation:progress', (event) => {
    //   updateCompilationJob(event.payload.id, { progress: event.payload.progress });
    // });
    
    // listen('sync:update', (event) => {
    //   setPendingChanges(event.payload.pendingChanges);
    // });
  };

  return null;
};

// ==================== ROUTE ERROR WRAPPER ====================

interface RouteErrorWrapperProps {
  children: React.ReactNode;
  componentName: string;
}

const RouteErrorWrapper: React.FC<RouteErrorWrapperProps> = ({ children, componentName }) => (
  <ErrorBoundary componentName={componentName}>
    {children}
  </ErrorBoundary>
);

// ==================== MAIN APP ====================

const App: React.FC = () => {
  return (
    <ErrorBoundary componentName="App Root">
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AppInitializer />
          <AppLayout>
            <Routes>
              {/* Default route - Entities */}
              <Route 
                path="/" 
                element={
                  <RouteErrorWrapper componentName="EntitiesPage">
                    <EntitiesPage />
                  </RouteErrorWrapper>
                } 
              />
              
              {/* Entities routes */}
              <Route 
                path="/entities" 
                element={
                  <RouteErrorWrapper componentName="EntitiesPage">
                    <EntitiesPage />
                  </RouteErrorWrapper>
                } 
              />
              <Route 
                path="/entities/:id" 
                element={
                  <RouteErrorWrapper componentName="EntitiesPage">
                    <EntitiesPage />
                  </RouteErrorWrapper>
                } 
              />
              
              {/* Graph routes */}
              <Route 
                path="/graph" 
                element={
                  <RouteErrorWrapper componentName="GraphPage">
                    <GraphPage />
                  </RouteErrorWrapper>
                } 
              />
              <Route 
                path="/graph/:rootId" 
                element={
                  <RouteErrorWrapper componentName="GraphPage">
                    <GraphPage />
                  </RouteErrorWrapper>
                } 
              />
              
              {/* Validation route */}
              <Route 
                path="/validation" 
                element={
                  <RouteErrorWrapper componentName="ValidationPage">
                    <ValidationPage />
                  </RouteErrorWrapper>
                } 
              />
              
              {/* Meetings routes */}
              <Route 
                path="/meetings" 
                element={
                  <RouteErrorWrapper componentName="MeetingsPage">
                    <MeetingsPage />
                  </RouteErrorWrapper>
                } 
              />
              <Route 
                path="/meetings/:id" 
                element={
                  <RouteErrorWrapper componentName="MeetingsPage">
                    <MeetingsPage />
                  </RouteErrorWrapper>
                } 
              />
              
              {/* 404 Fallback */}
              <Route 
                path="*" 
                element={
                  <div className="flex items-center justify-center h-full">
                    <div className="text-center">
                      <h1 className="text-4xl font-bold text-darkroom-text-primary mb-4">404</h1>
                      <p className="text-darkroom-text-secondary">Page not found</p>
                    </div>
                  </div>
                } 
              />
            </Routes>
          </AppLayout>
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  );
};

export default App;
