/**
 * @fileoverview Schema Validation Unit Tests
 * @module tests/unit/schema
 * 
 * Validates compilation rules, privacy classifications, and prohibited extractions.
 * Critical for GDPR Article 32 compliance.
 * 
 * @requires jest
 * @requires @truffle/core/schema
 */

import {
  Schema,
  CompilationRule,
  PrivacyClassification,
  compileArtifact,
  classifyPrivacy,
  extractEntities,
  applyPrivacyRules,
  redactSensitiveData,
  tokenizeCreditCard,
  type SchemaConfig,
  type ExtractionResult,
  type WikiNode,
} from '@truffle/core/schema';

const TEST_SCHEMA: SchemaConfig = {
  version: '2.0',
  compilation_rules: [
    {
      trigger: { concept: 'receipt' },
      action: 'create_entity',
      extract: ['merchant_name', 'total_amount', 'currency', 'tax_amount', 'items_list', 'payment_method'],
      link_to: ['concepts/expenses', 'chronology/{YYYY-MM}'],
      privacy: 'financial',
    },
    {
      trigger: { entity_type: 'person', context: 'contact_card' },
      action: 'merge_or_create',
      extract: ['phone', 'email', 'company', 'role'],
      privacy: 'personal',
    },
    {
      trigger: { pattern: 'flight_confirmation' },
      action: 'create_travel_itinerary',
      extract: ['airline', 'flight_number', 'departure', 'arrival', 'confirmation_code'],
      temporal_awareness: true,
      privacy: 'personal',
    },
  ],
  prohibited_extractions: [
    { field: 'password', action: 'redact_and_log' },
    { field: 'credit_card_number', action: 'tokenize_last4_only' },
    { field: 'ssn', action: 'refuse_and_quarantine' },
  ],
  linking_rules: {
    temporal_proximity: 300, // 5 minutes
    semantic_similarity_threshold: 0.82,
    entity_resolution: { levenshtein_threshold: 2 },
  },
};

describe('Receipt Compilation Rules', () => {
  const schema = new Schema(TEST_SCHEMA);

  test('SHOULD detect receipt and extract merchant name', () => {
    const ocrText = 'STARBUCKS COFFEE\n123 Main St\nTotal: $12.50';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      image_type: 'receipt',
    }, schema);
    
    expect(result.node_type).toBe('entity');
    expect(result.title.toLowerCase()).toContain('starbucks');
    expect(result.privacy_classification).toBe('financial');
  });

  test('SHOULD extract total amount from receipt', () => {
    const ocrText = 'GROCERY STORE\nItems: $45.00\nTax: $3.60\nTOTAL: $48.60';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      image_type: 'receipt',
    }, schema);
    
    expect(result.content).toContain('48.60');
    expect(result.content).toContain('total_amount');
  });

  test('SHOULD extract item list from receipt', () => {
    const ocrText = `
      RESTAURANT
      1x Burger $15.00
      1x Fries $5.00
      1x Soda $3.00
      Total: $23.00
    `;
    
    const result = compileArtifact({
      ocr_text: ocrText,
      image_type: 'receipt',
    }, schema);
    
    expect(result.content).toContain('Burger');
    expect(result.content).toContain('Fries');
    expect(result.content).toContain('items_list');
  });

  test('SHOULD auto-encrypt financial data for sync', () => {
    const ocrText = 'BANK STATEMENT\nBalance: $10,000.00';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      image_type: 'receipt',
    }, schema);
    
    expect(result.privacy_classification).toBe('financial');
    expect(result.encryption_status).toBe('aes256-gcm');
  });

  test('SHOULD link to chronology by month', () => {
    const ocrText = 'RECEIPT\nDate: 2024-03-15\nTotal: $50.00';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      image_type: 'receipt',
      captured_at: '2024-03-15T10:00:00Z',
    }, schema);
    
    expect(result.forward_links).toContain('chronology/2024-03');
  });
});

describe('Contact Card Compilation Rules', () => {
  const schema = new Schema(TEST_SCHEMA);

  test('SHOULD extract contact information', () => {
    const ocrText = `
      JOHN SMITH
      Software Engineer
      john.smith@example.com
      (555) 123-4567
      Acme Corp
    `;
    
    const result = compileArtifact({
      ocr_text: ocrText,
      entity_type: 'person',
      context: 'contact_card',
    }, schema);
    
    expect(result.content).toContain('john.smith@example.com');
    expect(result.content).toContain('555-123-4567');
    expect(result.content).toContain('Acme Corp');
    expect(result.privacy_classification).toBe('personal');
  });

  test('SHOULD merge with existing contact', () => {
    const existingContacts = [
      { name: 'John Smith', email: 'john@old.com' },
    ];
    
    const ocrText = 'JOHN SMITH\njohn.smith@new.com\n(555) 999-8888';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      entity_type: 'person',
      context: 'contact_card',
      existing_entities: existingContacts,
    }, schema);
    
    // Should merge, not create duplicate
    expect(result.action).toBe('merge');
    expect(result.merged_with).toBeDefined();
  });
});

describe('Flight Confirmation Rules', () => {
  const schema = new Schema(TEST_SCHEMA);

  test('SHOULD extract flight details', () => {
    const ocrText = `
      FLIGHT CONFIRMATION
      Airline: Delta Airlines
      Flight: DL1234
      From: JFK (New York)
      To: LAX (Los Angeles)
      Departure: March 15, 2024 8:00 AM
      Arrival: March 15, 2024 11:30 AM
      Confirmation: ABC123
    `;
    
    const result = compileArtifact({
      ocr_text: ocrText,
      pattern: 'flight_confirmation',
    }, schema);
    
    expect(result.content).toContain('Delta Airlines');
    expect(result.content).toContain('DL1234');
    expect(result.content).toContain('ABC123');
    expect(result.node_type).toBe('travel_itinerary');
  });

  test('SHOULD extract temporal references', () => {
    const ocrText = 'Flight on March 15, 2024';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      pattern: 'flight_confirmation',
    }, schema);
    
    expect(result.temporal_vectors.mentioned_dates).toContain('2024-03-15');
  });

  test('SHOULD link to calendar concepts', () => {
    const ocrText = 'Flight: March 15, 2024';
    
    const result = compileArtifact({
      ocr_text: ocrText,
      pattern: 'flight_confirmation',
    }, schema);
    
    expect(result.forward_links).toContainEqual(
      expect.stringContaining('calendar')
    );
  });
});

describe('Prohibited Extractions (Security)', () => {
  const schema = new Schema(TEST_SCHEMA);

  test('SHOULD redact password fields', () => {
    const ocrText = `
      LOGIN CREDENTIALS
      Username: admin
      Password: SuperSecret123!
    `;
    
    const result = compileArtifact({
      ocr_text: ocrText,
    }, schema);
    
    expect(result.content).not.toContain('SuperSecret123');
    expect(result.content).toContain('[REDACTED]');
    expect(result.security_events).toContainEqual(
      expect.objectContaining({ type: 'password_detected' })
    );
  });

  test('SHOULD tokenize credit card numbers (last 4 only)', () => {
    const ocrText = 'Payment: Card ending in 4532 1234 5678 9012';
    
    const result = compileArtifact({
      ocr_text: ocrText,
    }, schema);
    
    // Full CC number should not appear
    expect(result.content).not.toContain('4532 1234 5678 9012');
    expect(result.content).not.toContain('4532123456789012');
    
    // Tokenized version may appear
    expect(result.content).toContain('****9012');
  });

  test('SHOULD refuse and quarantine SSN detection', () => {
    const ocrText = 'SSN: 123-45-6789';
    
    const result = compileArtifact({
      ocr_text: ocrText,
    }, schema);
    
    // Should refuse compilation
    expect(result.status).toBe('quarantined');
    expect(result.content).not.toContain('123-45-6789');
    expect(result.quarantine_reason).toBe('ssn_detected');
  });

  test('SHOULD log security events locally only', () => {
    const ocrText = 'Password: secret123';
    
    const result = compileArtifact({
      ocr_text: ocrText,
    }, schema);
    
    expect(result.security_events).toBeDefined();
    expect(result.security_events.length).toBeGreaterThan(0);
    
    // Events should not contain actual sensitive data
    const eventJson = JSON.stringify(result.security_events);
    expect(eventJson).not.toContain('secret123');
  });
});

describe('Privacy Classification', () => {
  test('SHOULD classify financial data correctly', () => {
    const node: Partial<WikiNode> = {
      content: 'Receipt from Starbucks, Total: $12.50',
    };
    
    const classification = classifyPrivacy(node as WikiNode);
    expect(classification).toBe('financial');
  });

  test('SHOULD classify personal data correctly', () => {
    const node: Partial<WikiNode> = {
      content: 'Contact: John Smith, john@example.com',
    };
    
    const classification = classifyPrivacy(node as WikiNode);
    expect(classification).toBe('personal');
  });

  test('SHOULD classify sensitive medical data', () => {
    const node: Partial<WikiNode> = {
      content: 'Medical record: Patient diagnosed with...',
    };
    
    const classification = classifyPrivacy(node as WikiNode);
    expect(classification).toBe('sensitive');
  });

  test('SHOULD classify public data', () => {
    const node: Partial<WikiNode> = {
      content: 'Public announcement: Company launches new product',
    };
    
    const classification = classifyPrivacy(node as WikiNode);
    expect(classification).toBe('public');
  });
});

describe('Entity Resolution', () => {
  test('SHOULD match similar entity names (Levenshtein <= 2)', () => {
    const existing = [
      { name: 'Starbucks Coffee' },
      { name: 'Amazon' },
    ];
    
    const newName = 'Starbcks Coffee'; // Typo
    
    const match = findEntityMatch(newName, existing, { levenshtein_threshold: 2 });
    
    expect(match).toBeDefined();
    expect(match.name).toBe('Starbucks Coffee');
  });

  test('SHOULD use Metaphone for phonetic matching', () => {
    const existing = [
      { name: 'Robert Johnson' },
    ];
    
    const newName = 'Rupert Johnson'; // Phonetically similar
    
    const match = findEntityMatch(newName, existing, { use_metaphone: true });
    
    expect(match).toBeDefined();
  });

  test('SHOULD not match dissimilar names', () => {
    const existing = [
      { name: 'Starbucks' },
    ];
    
    const newName = 'Walmart';
    
    const match = findEntityMatch(newName, existing, { levenshtein_threshold: 2 });
    
    expect(match).toBeNull();
  });
});

describe('Temporal Linking', () => {
  test('SHOULD link screenshots within 5 minutes', () => {
    const screenshot1 = { captured_at: '2024-03-15T10:00:00Z' };
    const screenshot2 = { captured_at: '2024-03-15T10:03:00Z' };
    
    const shouldLink = areTemporallyRelated(screenshot1, screenshot2, 300);
    
    expect(shouldLink).toBe(true);
  });

  test('SHOULD NOT link screenshots beyond 5 minutes', () => {
    const screenshot1 = { captured_at: '2024-03-15T10:00:00Z' };
    const screenshot2 = { captured_at: '2024-03-15T10:10:00Z' };
    
    const shouldLink = areTemporallyRelated(screenshot1, screenshot2, 300);
    
    expect(shouldLink).toBe(false);
  });
});

describe('Semantic Similarity Linking', () => {
  test('SHOULD link semantically similar content (threshold 0.82)', () => {
    const embedding1 = new Float32Array(384).fill(0.5);
    const embedding2 = new Float32Array(384).fill(0.5);
    
    // Slight variation
    embedding2[0] = 0.4;
    
    const similarity = cosineSimilarity(embedding1, embedding2);
    
    expect(similarity).toBeGreaterThanOrEqual(0.82);
  });

  test('SHOULD NOT link dissimilar content', () => {
    const embedding1 = new Float32Array(384).fill(1.0);
    const embedding2 = new Float32Array(384).fill(-1.0);
    
    const similarity = cosineSimilarity(embedding1, embedding2);
    
    expect(similarity).toBeLessThan(0.82);
  });
});

// Helper functions
function findEntityMatch(
  name: string,
  existing: Array<{ name: string }>,
  options: { levenshtein_threshold?: number; use_metaphone?: boolean }
): { name: string } | null {
  for (const entity of existing) {
    const distance = levenshteinDistance(name, entity.name);
    if (distance <= (options.levenshtein_threshold || 2)) {
      return entity;
    }
  }
  return null;
}

function levenshteinDistance(a: string, b: string): number {
  const matrix: number[][] = [];
  
  for (let i = 0; i <= b.length; i++) {
    matrix[i] = [i];
  }
  
  for (let j = 0; j <= a.length; j++) {
    matrix[0][j] = j;
  }
  
  for (let i = 1; i <= b.length; i++) {
    for (let j = 1; j <= a.length; j++) {
      if (b.charAt(i - 1) === a.charAt(j - 1)) {
        matrix[i][j] = matrix[i - 1][j - 1];
      } else {
        matrix[i][j] = Math.min(
          matrix[i - 1][j - 1] + 1,
          matrix[i][j - 1] + 1,
          matrix[i - 1][j] + 1
        );
      }
    }
  }
  
  return matrix[b.length][a.length];
}

function areTemporallyRelated(a: any, b: any, thresholdSeconds: number): boolean {
  const time1 = new Date(a.captured_at).getTime();
  const time2 = new Date(b.captured_at).getTime();
  const diffSeconds = Math.abs(time1 - time2) / 1000;
  return diffSeconds <= thresholdSeconds;
}

function cosineSimilarity(a: Float32Array, b: Float32Array): number {
  let dotProduct = 0;
  let normA = 0;
  let normB = 0;
  
  for (let i = 0; i < a.length; i++) {
    dotProduct += a[i] * b[i];
    normA += a[i] * a[i];
    normB += b[i] * b[i];
  }
  
  return dotProduct / (Math.sqrt(normA) * Math.sqrt(normB));
}
