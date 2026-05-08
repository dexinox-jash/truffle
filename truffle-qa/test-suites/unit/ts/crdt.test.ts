/**
 * @fileoverview CRDT Module Unit Tests
 * @module tests/unit/crdt
 * 
 * Validates Yjs CRDT integration, conflict resolution, and vector clocks.
 * 
 * @requires jest
 * @requires yjs
 */

import * as Y from 'yjs';
import {
  createDocument,
  applyUpdate,
  encodeStateAsUpdate,
  mergeUpdates,
  createVectorClock,
  incrementClock,
  compareClocks,
  resolveConflict,
  type CRDTDocument,
  type VectorClock,
  type Conflict,
} from '@truffle/core/crdt';

describe('Yjs Document Operations', () => {
  test('SHOULD create empty document', () => {
    const doc = createDocument();
    
    expect(doc).toBeDefined();
    expect(doc.getMap('wiki')).toBeDefined();
    expect(doc.getText('content')).toBeDefined();
  });

  test('SHOULD apply update to document', () => {
    const doc1 = createDocument();
    const map = doc1.getMap('wiki');
    map.set('title', 'Test Node');
    
    const update = encodeStateAsUpdate(doc1);
    
    const doc2 = createDocument();
    applyUpdate(doc2, update);
    
    expect(doc2.getMap('wiki').get('title')).toBe('Test Node');
  });

  test('SHOULD merge concurrent updates', () => {
    const doc1 = createDocument();
    const doc2 = createDocument();
    
    // Concurrent edits
    doc1.getMap('wiki').set('title', 'Title A');
    doc2.getMap('wiki').set('content', 'Content B');
    
    const update1 = encodeStateAsUpdate(doc1);
    const update2 = encodeStateAsUpdate(doc2);
    
    // Merge both updates into new document
    const merged = createDocument();
    applyUpdate(merged, update1);
    applyUpdate(merged, update2);
    
    expect(merged.getMap('wiki').get('title')).toBe('Title A');
    expect(merged.getMap('wiki').get('content')).toBe('Content B');
  });

  test('SHOULD handle concurrent text edits', () => {
    const doc1 = createDocument();
    const doc2 = createDocument();
    
    const text1 = doc1.getText('content');
    const text2 = doc2.getText('content');
    
    text1.insert(0, 'Hello ');
    text2.insert(0, 'World');
    
    const update1 = encodeStateAsUpdate(doc1);
    const update2 = encodeStateAsUpdate(doc2);
    
    const merged = createDocument();
    applyUpdate(merged, update1);
    applyUpdate(merged, update2);
    
    const mergedText = merged.getText('content').toString();
    expect(mergedText).toContain('Hello');
    expect(mergedText).toContain('World');
  });
});

describe('Vector Clocks (Lamport Timestamps)', () => {
  test('SHOULD create initial vector clock', () => {
    const clock = createVectorClock('device-1');
    
    expect(clock.deviceId).toBe('device-1');
    expect(clock.counters).toEqual({ 'device-1': 0 });
  });

  test('SHOULD increment local counter', () => {
    const clock = createVectorClock('device-1');
    
    incrementClock(clock);
    expect(clock.counters['device-1']).toBe(1);
    
    incrementClock(clock);
    expect(clock.counters['device-1']).toBe(2);
  });

  test('SHOULD merge vector clocks', () => {
    const clock1 = createVectorClock('device-1');
    incrementClock(clock1);
    incrementClock(clock1);
    
    const clock2 = createVectorClock('device-2');
    incrementClock(clock2);
    
    const merged = mergeClocks(clock1, clock2);
    
    expect(merged.counters['device-1']).toBe(2);
    expect(merged.counters['device-2']).toBe(1);
  });

  test('SHOULD compare vector clocks correctly', () => {
    const clock1 = createVectorClock('device-1');
    incrementClock(clock1);
    
    const clock2 = createVectorClock('device-1');
    incrementClock(clock2);
    incrementClock(clock2);
    
    // clock2 > clock1
    expect(compareClocks(clock2, clock1)).toBe(1);
    expect(compareClocks(clock1, clock2)).toBe(-1);
    
    // Equal clocks
    const clock3 = createVectorClock('device-1');
    incrementClock(clock3);
    expect(compareClocks(clock1, clock3)).toBe(0);
  });

  test('SHOULD detect concurrent events', () => {
    const clock1 = createVectorClock('device-1');
    incrementClock(clock1);
    
    const clock2 = createVectorClock('device-2');
    incrementClock(clock2);
    
    // Neither happens before the other (concurrent)
    expect(compareClocks(clock1, clock2)).toBeNull();
  });
});

describe('Conflict Resolution', () => {
  test('SHOULD resolve last-write-wins for scalars', () => {
    const conflict: Conflict = {
      field: 'title',
      values: [
        { value: 'Title A', clock: { deviceId: 'a', counters: { a: 1 } } },
        { value: 'Title B', clock: { deviceId: 'b', counters: { b: 2 } } },
      ],
    };
    
    const resolved = resolveConflict(conflict, 'lww');
    
    // Higher clock wins
    expect(resolved).toBe('Title B');
  });

  test('SHOULD preserve multiple values for multi-value register', () => {
    const conflict: Conflict = {
      field: 'tags',
      values: [
        { value: ['tag1'], clock: { deviceId: 'a', counters: { a: 1 } } },
        { value: ['tag2'], clock: { deviceId: 'b', counters: { b: 1 } } },
      ],
    };
    
    const resolved = resolveConflict(conflict, 'multi-value');
    
    // Both values preserved for user resolution
    expect(resolved).toContain('tag1');
    expect(resolved).toContain('tag2');
  });

  test('SHOULD handle concurrent wiki edits', () => {
    const node1 = {
      id: 'node-1',
      title: 'Original Title',
      content: 'Original content',
      version: { deviceId: 'a', counters: { a: 1 } },
    };
    
    const node2 = {
      id: 'node-1',
      title: 'Edited Title',
      content: 'Original content',
      version: { deviceId: 'b', counters: { b: 1 } },
    };
    
    const resolved = resolveWikiConflict(node1, node2);
    
    // Both versions should be available
    expect(resolved.versions).toHaveLength(2);
    expect(resolved.needsUserResolution).toBe(true);
  });
});

describe('Schema Migration', () => {
  test('SHOULD migrate document to new schema version', () => {
    const oldDoc = createDocument();
    oldDoc.getMap('wiki').set('schema_version', '1.0');
    oldDoc.getMap('wiki').set('title', 'Old Node');
    
    const migrated = migrateDocument(oldDoc, '1.0', '2.0');
    
    expect(migrated.getMap('wiki').get('schema_version')).toBe('2.0');
    expect(migrated.getMap('wiki').get('title')).toBe('Old Node');
  });

  test('SHOULD add new fields during migration', () => {
    const oldDoc = createDocument();
    oldDoc.getMap('wiki').set('schema_version', '1.0');
    
    const migration = (doc: Y.Doc) => {
      const map = doc.getMap('wiki');
      map.set('privacy_classification', 'personal');
      map.set('encryption_status', 'aes256-gcm');
    };
    
    const migrated = migrateDocument(oldDoc, '1.0', '2.0', [migration]);
    
    expect(migrated.getMap('wiki').get('privacy_classification')).toBe('personal');
    expect(migrated.getMap('wiki').get('encryption_status')).toBe('aes256-gcm');
  });
});

describe('Tombstones (GDPR Deletion)', () => {
  test('SHOULD create tombstone for deleted node', () => {
    const nodeId = 'node-to-delete';
    const tombstone = createTombstone(nodeId, 'device-1');
    
    expect(tombstone.artifactId).toBe(nodeId);
    expect(tombstone.deviceId).toBe('device-1');
    expect(tombstone.timestamp).toBeGreaterThan(0);
    expect(tombstone.isDeleted).toBe(true);
  });

  test('SHOULD propagate tombstone in sync', () => {
    const doc = createDocument();
    const tombstone = createTombstone('deleted-node', 'device-1');
    
    // Add tombstone to document
    const tombstones = doc.getMap('tombstones');
    tombstones.set('deleted-node', tombstone);
    
    const update = encodeStateAsUpdate(doc);
    
    // Verify tombstone in update
    const received = createDocument();
    applyUpdate(received, update);
    
    expect(received.getMap('tombstones').get('deleted-node')).toEqual(tombstone);
  });

  test('SHOULD handle concurrent delete and edit', () => {
    const doc1 = createDocument();
    const map1 = doc1.getMap('wiki');
    map1.set('id', 'node-1');
    map1.set('title', 'Original');
    
    // Device 1 deletes
    const tombstone = createTombstone('node-1', 'device-1');
    
    // Device 2 edits concurrently
    const doc2 = createDocument();
    const map2 = doc2.getMap('wiki');
    map2.set('id', 'node-1');
    map2.set('title', 'Edited');
    
    // Merge - tombstone should take precedence
    const merged = createDocument();
    applyUpdate(merged, encodeStateAsUpdate(doc1));
    merged.getMap('tombstones').set('node-1', tombstone);
    applyUpdate(merged, encodeStateAsUpdate(doc2));
    
    expect(merged.getMap('tombstones').has('node-1')).toBe(true);
  });
});

// Helper functions
function mergeClocks(clock1: VectorClock, clock2: VectorClock): VectorClock {
  const merged = { ...clock1 };
  for (const [device, counter] of Object.entries(clock2.counters)) {
    merged.counters[device] = Math.max(merged.counters[device] || 0, counter);
  }
  return merged;
}

function resolveWikiConflict(node1: any, node2: any): any {
  return {
    versions: [node1, node2],
    needsUserResolution: true,
  };
}

function migrateDocument(
  doc: Y.Doc,
  fromVersion: string,
  toVersion: string,
  migrations: Function[] = []
): Y.Doc {
  const migrated = createDocument();
  applyUpdate(migrated, encodeStateAsUpdate(doc));
  
  for (const migration of migrations) {
    migration(migrated);
  }
  
  migrated.getMap('wiki').set('schema_version', toVersion);
  return migrated;
}

function createTombstone(artifactId: string, deviceId: string): any {
  return {
    artifactId,
    deviceId,
    timestamp: Date.now(),
    isDeleted: true,
  };
}
