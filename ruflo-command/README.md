# Ruflo Command Service

Enterprise-grade compliance, security, audit, and SSO module for the Ruflo AI Meeting Platform.

## Features

### 🔒 Security
- **Field-Level Encryption**: Transparent AES-256-GCM encryption with HashiCorp Vault integration
- **Key Rotation**: Zero-downtime key rotation with versioned keys
- **Searchable Encryption**: Deterministic encryption for encrypted search

### 📋 Compliance
- **GDPR**: Article 17 (Right to Erasure), Article 18 (Restriction), Article 20 (Portability)
- **SOC 2**: Trust Service Criteria (Security, Availability, Confidentiality)
- **Data Retention**: Automated policy enforcement with legal hold support
- **Audit Trails**: Tamper-evident logging with chain-of-custody

### 🌍 Data Residency
- **Regional Storage**: EU, US, UK, CA, AU, SG, JP, IN, BR regions
- **Geo-Fencing**: IP-based access control with VPN requirements
- **Cross-Border Transfer**: Automated compliance checks for data movement

### 🔐 Enterprise SSO
- **SAML 2.0**: Okta, Azure AD, OneLogin, Ping Identity support
- **SCIM 2.0**: Automated user provisioning/deprovisioning
- **Attribute Mapping**: Flexible identity attribute transformation

## Module Structure

```
ruflo-command/
├── src/
│   ├── lib.rs           # Service exports & initialization
│   ├── main.rs          # Standalone entry point
│   ├── audit/           # Tamper-evident audit logging
│   │   ├── mod.rs
│   │   └── logger.rs    # 727 lines
│   ├── compliance/      # GDPR, SOC 2, retention
│   │   ├── mod.rs
│   │   ├── gdpr.rs      # GDPR automation
│   │   ├── soc2.rs      # SOC 2 controls
│   │   └── retention.rs # 712 lines
│   ├── encryption/      # Field-level encryption
│   │   ├── mod.rs
│   │   └── field_level.rs # 474 lines
│   ├── geo/             # Data residency & geo-fencing
│   │   ├── mod.rs
│   │   └── residency.rs # 595 lines
│   └── sso/             # SAML & SCIM
│       ├── mod.rs
│       └── saml.rs      # 558 lines
├── Cargo.toml
└── README.md
```

## Usage

```rust
use ruflo_command::*;

// Initialize service
let ctx = ruflo_command::initialize(pool, "https://vault.ruflo.ai").await?;

// GDPR erasure
let gdpr = ctx.gdpr_controller();
let report = gdpr.process_erasure_request(user_id).await?;

// Audit logging
ctx.audit.log(AuditEvent {
    action: "user.login",
    user_id: user_id.to_string(),
    resource: "session",
    ..Default::default()
}).await?;

// Field encryption
let encrypted = ctx.encryption.encrypt_field("sensitive data", true).await?;
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/gdpr/erasure` | Submit erasure request |
| GET | `/api/v1/gdpr/export` | Data portability export |
| POST | `/api/v1/audit/query` | Query audit logs |
| POST | `/api/v1/compliance/scan` | Run compliance scan |
| POST | `/api/v1/saml/login` | SAML SSO endpoint |
| POST | `/api/v1/scim/v2/Users` | SCIM provisioning |

## Compliance Standards

| Standard | Status | Coverage |
|----------|--------|----------|
| GDPR | ✅ Complete | Articles 17, 18, 20, DPO, DPA |
| SOC 2 | ✅ Complete | Type II ready |
| ISO 27001 | 🔄 In Progress | Controls framework |
| HIPAA | 🔄 Planned | Healthcare module |
| CCPA | ✅ Complete | California privacy |
| LGPD | ✅ Complete | Brazil privacy |

## License

MIT License - See LICENSE file for details.

---
**Ruflo AI Meeting Platform** | Series A Ready | AAA Commercial Grade
