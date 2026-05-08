# Incident Response Runbook: INCIDENT-001
## Key Compromise / Cryptographic Failure

**Classification:** SEV-1 (Critical)  
**Response Time:** Immediate (within 1 hour)  
**Last Updated:** 2024  

---

## 1. Incident Overview

### 1.1 Description

A key compromise incident occurs when:
- Device private keys are exposed or stolen
- Cryptographic implementation has a vulnerability
- Key generation is found to be weak/predictable
- Side-channel attack reveals key material

### 1.2 Impact Assessment

| Impact Area | Severity | Description |
|-------------|----------|-------------|
| Confidentiality | CRITICAL | Sync data may be decryptable |
| Integrity | HIGH | Messages may be forgeable |
| Availability | MEDIUM | Re-pairing required |
| Compliance | HIGH | Zero-knowledge claim affected |

### 1.3 Triggers

- Security researcher report
- Automated anomaly detection
- User report of suspicious activity
- Internal code audit finding
- External penetration test result

---

## 2. Immediate Response (0-1 Hour)

### 2.1 Alert and Assemble

```bash
# 1. Page on-call security engineer
pagerduty trigger "SEV-1: Key compromise suspected"

# 2. Create incident channel
slack create-channel "incident-001-key-$(date +%Y%m%d)"

# 3. Notify stakeholders
slack notify "@channel SEV-1 Key Compromise - War Room: [Zoom Link]"
```

### 2.2 Initial Assessment

**War Room Checklist:**
- [ ] Incident commander assigned
- [ ] Technical lead assigned
- [ ] Communications lead assigned
- [ ] Legal notified (if user data potentially affected)
- [ ] Executive sponsor notified

### 2.3 Containment

```typescript
// Emergency key revocation
async function emergencyKeyRevocation(deviceId: string): Promise<void> {
  // 1. Revoke compromised keys
  await keyStore.revoke(deviceId);
  
  // 2. Generate new key pair
  const newKeys = await generateDeviceKeys();
  
  // 3. Update key directory
  await keyDirectory.update(deviceId, {
    status: 'compromised',
    revokedAt: Date.now(),
    newKeyFingerprint: newKeys.fingerprint,
  });
  
  // 4. Notify linked devices
  await notifyLinkedDevices(deviceId, {
    type: 'KEY_REVOCATION',
    oldFingerprint: compromisedFingerprint,
    newFingerprint: newKeys.fingerprint,
  });
}
```

### 2.4 User Notification

```typescript
// In-app notification (no email - no PII)
const notification = {
  title: 'Security Update Required',
  body: 'We detected a potential security issue. Please re-pair your devices.',
  actions: [
    { label: 'Re-pair Now', action: 'start_pairing' },
    { label: 'Learn More', action: 'open_security_doc' },
  ],
  priority: 'urgent',
};

// Push to all affected devices
await pushNotification.send(affectedDevices, notification);
```

---

## 3. Investigation (1-24 Hours)

### 3.1 Scope Determination

```bash
# Identify affected devices
./scripts/security/identify-affected-devices.sh \
  --key-fingerprint "$COMPROMISED_FINGERPRINT" \
  --time-range "$START_TIME,$END_TIME"

# Check for suspicious sync activity
./scripts/security/analyze-sync-logs.sh \
  --affected-devices "$AFFECTED_DEVICES" \
  --lookback-days 30

# Verify zero-knowledge status
./scripts/security/verify-zero-knowledge.sh \
  --compromised-keys "$COMPROMISED_KEYS"
```

### 3.2 Root Cause Analysis

**Investigation Checklist:**
- [ ] How were keys compromised?
  - [ ] Implementation vulnerability?
  - [ ] Side-channel attack?
  - [ ] User device compromise?
  - [ ] Supply chain attack?
- [ ] When did compromise occur?
- [ ] Which devices affected?
- [ ] Was any data decrypted?
- [ ] Were any messages forged?

### 3.3 Evidence Collection

```bash
# Preserve logs
tar czf "evidence-$(date +%Y%m%d-%H%M%S).tar.gz" \
  /var/log/truffle/ \
  /var/log/relay/ \
  ./audit-logs/

# Secure evidence
aws s3 cp "evidence-*.tar.gz" s3://truffle-security-evidence/ \
  --server-side-encryption AES256

# Hash for integrity
evidence_hash=$(sha256sum "evidence-*.tar.gz" | cut -d' ' -f1)
echo "$evidence_hash" > "evidence-hash.txt"
```

---

## 4. Remediation (24-72 Hours)

### 4.1 Key Rotation

```rust
// Implement emergency key rotation
pub async fn emergency_key_rotation(
    device_id: &str,
    reason: RotationReason,
) -> Result<RotationResult, RotationError> {
    // 1. Generate new identity key
    let new_identity = IdentityKey::generate()?;
    
    // 2. Generate new ephemeral keys
    let new_ephemeral = generate_ephemeral_keys(10)?;
    
    // 3. Generate new pre-keys
    let new_prekeys = generate_pre_keys(100)?;
    
    // 4. Create rotation package
    let rotation = KeyRotation {
        device_id: device_id.to_string(),
        old_keys: get_current_keys(device_id).await?,
        new_keys: DeviceKeys {
            identity_key: new_identity,
            ephemeral_keys: new_ephemeral,
            pre_keys: new_prekeys,
        },
        reason,
        timestamp: Utc::now(),
    };
    
    // 5. Apply rotation
    apply_rotation(rotation).await?;
    
    // 6. Notify linked devices
    notify_linked_devices(device_id, &rotation).await?;
    
    Ok(RotationResult {
        new_fingerprint: rotation.new_keys.fingerprint(),
        devices_notified: count_linked_devices(device_id).await?,
    })
}
```

### 4.2 User Actions Required

| Action | Required By | Impact |
|--------|-------------|--------|
| Re-pair devices | All affected users | Sync resumes |
| Verify fingerprints | All affected users | MITM prevention |
| Review recent syncs | Recommended | Detect anomalies |
| Export backup | Recommended | Data preservation |

### 4.3 Infrastructure Updates

```bash
# Rotate relay server certificates (precautionary)
./scripts/rotate-relay-certs.sh

# Update CRL (Certificate Revocation List)
./scripts/update-crl.sh --add "$COMPROMISED_FINGERPRINTS"

# Deploy patched version if implementation bug
./scripts/deploy-hotfix.sh --version "$PATCH_VERSION"
```

---

## 5. Communication

### 5.1 Internal Communication

**Timeline:**
- **0-1 hour:** War room assembled, initial assessment
- **1-4 hours:** Scope determined, stakeholders notified
- **4-24 hours:** Root cause identified, remediation planned
- **24-72 hours:** Remediation executed, monitoring enhanced
- **72+ hours:** Post-incident review, lessons learned

### 5.2 External Communication

**Transparency Report Template:**

```markdown
# Security Incident Report: [DATE]

## Summary
On [DATE], we became aware of a potential cryptographic issue affecting [SCOPE].

## Impact
- Affected users: [NUMBER]
- Data accessed: [NONE / UNKNOWN / CONFIRMED]
- Zero-knowledge status: [MAINTAINED / UNDER REVIEW]

## Response
- Issue detected: [TIME]
- Response initiated: [TIME]
- Users notified: [TIME]
- Remediation completed: [TIME]

## Actions Taken
1. [ACTION 1]
2. [ACTION 2]
3. [ACTION 3]

## User Actions Required
- [ACTION 1]
- [ACTION 2]

## Lessons Learned
[LESSONS]

## Prevention Measures
[MEASURES]
```

### 5.3 User Communication Channels

| Channel | Timing | Content |
|---------|--------|---------|
| In-app push | Immediate | Alert + action required |
| Status page | Within 1 hour | Incident posted |
| Blog post | Within 24 hours | Detailed explanation |
| Email (opt-in) | Within 24 hours | Summary + guidance |

---

## 6. Recovery Validation

### 6.1 Post-Remediation Testing

```typescript
// Verify key rotation success
describe('Post-Incident Validation', () => {
  test('new keys are valid', async () => {
    const newKeys = await getDeviceKeys(deviceId);
    expect(newKeys.isValid()).toBe(true);
    expect(newKeys.fingerprint).not.toBe(compromisedFingerprint);
  });

  test('sync works with new keys', async () => {
    const result = await testSync(deviceId, linkedDeviceId);
    expect(result.success).toBe(true);
  });

  test('old keys are rejected', async () => {
    const result = await attemptSyncWithOldKeys();
    expect(result.success).toBe(false);
    expect(result.error).toContain('revoked');
  });

  test('zero-knowledge maintained', async () => {
    const proof = await verifyZeroKnowledge();
    expect(proof.infrastructureCanDecrypt).toBe(false);
  });
});
```

### 6.2 Monitoring Enhancement

```typescript
// Enhanced monitoring for key-related anomalies
const keyMonitoring = {
  // Monitor for unusual key operations
  watchKeyOperations: () => {
    metrics.histogram('key.generation.time');
    metrics.counter('key.rotation.count');
    metrics.gauge('key.active.count');
  },

  // Alert on suspicious patterns
  detectAnomalies: () => {
    alerts.when('key.rotation.rate > 10/hour')
      .trigger('investigate-key-activity');
    
    alerts.when('key.generation.time > 5s')
      .trigger('slow-key-generation');
  },
};
```

---

## 7. Post-Incident Review

### 7.1 Timeline Documentation

```
Incident Timeline: INCIDENT-001

[TIME] - Issue detected by [SOURCE]
[TIME] - War room assembled
[TIME] - Initial assessment completed
[TIME] - Containment measures applied
[TIME] - Root cause identified
[TIME] - Remediation started
[TIME] - Remediation completed
[TIME] - Validation completed
[TIME] - Monitoring enhanced
[TIME] - Post-incident review scheduled
```

### 7.2 Lessons Learned Template

| Question | Answer |
|----------|--------|
| What went well? | [ANSWER] |
| What could have gone better? | [ANSWER] |
| What will we do differently? | [ANSWER] |
| What detection gaps exist? | [ANSWER] |
| What prevention measures needed? | [ANSWER] |

### 7.3 Action Items

| Action | Owner | Due Date | Status |
|--------|-------|----------|--------|
| [ACTION] | [OWNER] | [DATE] | [STATUS] |

---

## 8. Appendices

### Appendix A: Emergency Contacts

| Role | Name | Contact | Backup |
|------|------|---------|--------|
| Incident Commander | [TBD] | [TBD] | [TBD] |
| Security Lead | [TBD] | [TBD] | [TBD] |
| Engineering Lead | [TBD] | [TBD] | [TBD] |
| Legal | [TBD] | [TBD] | [TBD] |
| Communications | [TBD] | [TBD] | [TBD] |

### Appendix B: Quick Reference Commands

```bash
# Revoke device keys
./scripts/revoke-device-keys.sh --device-id "DEVICE_ID"

# Generate new keys
./scripts/generate-device-keys.sh --device-id "DEVICE_ID"

# Notify linked devices
./scripts/notify-linked-devices.sh --device-id "DEVICE_ID" --message "MESSAGE"

# Verify zero-knowledge
./scripts/verify-zero-knowledge.sh --device-id "DEVICE_ID"

# Generate transparency report
./scripts/generate-transparency-report.sh --incident-id "001"
```

### Appendix C: Related Documentation

- [Cryptographic Architecture](../../docs/crypto/architecture.md)
- [Key Management Procedures](../../docs/crypto/key-management.md)
- [Zero-Knowledge Proofs](../../docs/crypto/zero-knowledge.md)
- [Security Audit Report](../../SECURITY_AUDIT.md)

---

*Classification: Confidential*  
*Review Cycle: Quarterly*  
*Owner: Security Team*
