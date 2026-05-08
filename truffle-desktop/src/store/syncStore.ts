// SPEC v2.0 SECTION 2.3: Yjs CRDT Sync Layer
// Sync state management for Truffle Desktop

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import * as Y from 'yjs';

// ==================== TYPES ====================

export interface SyncDevice {
  id: string;
  name: string;
  type: 'desktop' | 'mobile' | 'web';
  lastSeen: string;
  isOnline: boolean;
  isPrimary: boolean;
}

export interface SyncChange {
  id: string;
  type: 'create' | 'update' | 'delete';
  entityType: 'artifact' | 'wiki-node' | 'schema';
  entityId: string;
  timestamp: string;
  deviceId: string;
  isLocal: boolean;
}

export interface ConflictResolution {
  id: string;
  entityType: string;
  entityId: string;
  localValue: unknown;
  remoteValue: unknown;
  resolvedValue: unknown;
  resolution: 'local' | 'remote' | 'merge' | 'pending';
  timestamp: string;
}

export interface SyncState {
  // Connection state
  isConnected: boolean;
  isConnecting: boolean;
  connectionError: string | null;
  
  // Sync state
  isSyncing: boolean;
  lastSyncAt: string | null;
  pendingChanges: number;
  syncHealth: 'healthy' | 'degraded' | 'offline' | 'error';
  
  // Devices
  devices: SyncDevice[];
  primaryDevice: SyncDevice | null;
  thisDevice: SyncDevice | null;
  
  // Changes
  recentChanges: SyncChange[];
  conflicts: ConflictResolution[];
  
  // Yjs document
  ydoc: Y.Doc | null;
  
  // Pairing
  isPairing: boolean;
  pairingCode: string | null;
  pairingQrData: string | null;
  pairingExpiresAt: string | null;
  
  // Actions
  setConnected: (connected: boolean) => void;
  setConnecting: (connecting: boolean) => void;
  setConnectionError: (error: string | null) => void;
  
  setSyncing: (syncing: boolean) => void;
  setLastSyncAt: (date: string | null) => void;
  setPendingChanges: (count: number) => void;
  setSyncHealth: (health: SyncState['syncHealth']) => void;
  
  setDevices: (devices: SyncDevice[]) => void;
  addDevice: (device: SyncDevice) => void;
  removeDevice: (id: string) => void;
  updateDevice: (id: string, updates: Partial<SyncDevice>) => void;
  setPrimaryDevice: (device: SyncDevice | null) => void;
  setThisDevice: (device: SyncDevice | null) => void;
  
  addChange: (change: SyncChange) => void;
  clearOldChanges: (keepCount: number) => void;
  
  addConflict: (conflict: ConflictResolution) => void;
  resolveConflict: (id: string, resolution: ConflictResolution['resolution'], value: unknown) => void;
  
  setYDoc: (ydoc: Y.Doc | null) => void;
  
  startPairing: (code: string, qrData: string, expiresAt: string) => void;
  endPairing: () => void;
  
  // Computed
  getOnlineDevices: () => SyncDevice[];
  getPendingConflicts: () => ConflictResolution[];
}

// ==================== STORE ====================

export const useSyncStore = create<SyncState>()(
  devtools(
    immer((set, get) => ({
      // Initial state
      isConnected: false,
      isConnecting: false,
      connectionError: null,
      
      isSyncing: false,
      lastSyncAt: null,
      pendingChanges: 0,
      syncHealth: 'offline',
      
      devices: [],
      primaryDevice: null,
      thisDevice: null,
      
      recentChanges: [],
      conflicts: [],
      
      ydoc: null,
      
      isPairing: false,
      pairingCode: null,
      pairingQrData: null,
      pairingExpiresAt: null,
      
      // Actions
      setConnected: (connected) => set({ isConnected: connected }),
      setConnecting: (connecting) => set({ isConnecting: connecting }),
      setConnectionError: (error) => set({ connectionError: error }),
      
      setSyncing: (syncing) => set({ isSyncing: syncing }),
      setLastSyncAt: (date) => set({ lastSyncAt: date }),
      setPendingChanges: (count) => set({ pendingChanges: count }),
      setSyncHealth: (health) => set({ syncHealth: health }),
      
      setDevices: (devices) => set({ devices }),
      
      addDevice: (device) => set((state) => {
        const existing = state.devices.find(d => d.id === device.id);
        if (existing) {
          Object.assign(existing, device);
        } else {
          state.devices.push(device);
        }
      }),
      
      removeDevice: (id) => set((state) => {
        state.devices = state.devices.filter(d => d.id !== id);
      }),
      
      updateDevice: (id, updates) => set((state) => {
        const device = state.devices.find(d => d.id === id);
        if (device) {
          Object.assign(device, updates);
        }
      }),
      
      setPrimaryDevice: (device) => set({ primaryDevice: device }),
      setThisDevice: (device) => set({ thisDevice: device }),
      
      addChange: (change) => set((state) => {
        state.recentChanges.unshift(change);
        // Keep only last 100 changes
        if (state.recentChanges.length > 100) {
          state.recentChanges = state.recentChanges.slice(0, 100);
        }
      }),
      
      clearOldChanges: (keepCount) => set((state) => {
        state.recentChanges = state.recentChanges.slice(0, keepCount);
      }),
      
      addConflict: (conflict) => set((state) => {
        state.conflicts.push(conflict);
      }),
      
      resolveConflict: (id, resolution, value) => set((state) => {
        const conflict = state.conflicts.find(c => c.id === id);
        if (conflict) {
          conflict.resolution = resolution;
          conflict.resolvedValue = value;
        }
      }),
      
      setYDoc: (ydoc) => set({ ydoc }),
      
      startPairing: (code, qrData, expiresAt) => set({
        isPairing: true,
        pairingCode: code,
        pairingQrData: qrData,
        pairingExpiresAt: expiresAt,
      }),
      
      endPairing: () => set({
        isPairing: false,
        pairingCode: null,
        pairingQrData: null,
        pairingExpiresAt: null,
      }),
      
      // Computed getters
      getOnlineDevices: () => {
        return get().devices.filter(d => d.isOnline);
      },
      
      getPendingConflicts: () => {
        return get().conflicts.filter(c => c.resolution === 'pending');
      },
    })),
    { name: 'truffle-sync-store' }
  )
);

// ==================== Yjs HELPERS ====================

export function createYDoc(): Y.Doc {
  const ydoc = new Y.Doc();
  
  // Create shared types
  ydoc.getMap('artifacts');
  ydoc.getMap('wiki-nodes');
  ydoc.getMap('metadata');
  
  return ydoc;
}

export function encodeYDoc(ydoc: Y.Doc): Uint8Array {
  return Y.encodeStateAsUpdate(ydoc);
}

export function decodeYDoc(update: Uint8Array, ydoc?: Y.Doc): Y.Doc {
  const doc = ydoc || new Y.Doc();
  Y.applyUpdate(doc, update);
  return doc;
}

export function getYDocStateVector(ydoc: Y.Doc): Uint8Array {
  return Y.encodeStateVector(ydoc);
}

export function getYDocDiff(ydoc: Y.Doc, stateVector: Uint8Array): Uint8Array {
  return Y.encodeStateAsUpdate(ydoc, stateVector);
}

// ==================== HOOKS ====================

export function useSync() {
  const store = useSyncStore();
  
  return {
    isConnected: store.isConnected,
    isSyncing: store.isSyncing,
    lastSyncAt: store.lastSyncAt,
    pendingChanges: store.pendingChanges,
    syncHealth: store.syncHealth,
    devices: store.devices,
    conflicts: store.conflicts,
    isPairing: store.isPairing,
    pairingCode: store.pairingCode,
    pairingQrData: store.pairingQrData,
    
    onlineDevices: store.getOnlineDevices(),
    pendingConflicts: store.getPendingConflicts(),
    
    setConnected: store.setConnected,
    setSyncing: store.setSyncing,
    addDevice: store.addDevice,
    addConflict: store.addConflict,
    resolveConflict: store.resolveConflict,
    startPairing: store.startPairing,
    endPairing: store.endPairing,
  };
}
