# Project Truffle - DevOps & Infrastructure

This directory contains all CI/CD, infrastructure, and deployment automation for Project Truffle.

## 📁 Directory Structure

```
truffle/
├── .github/workflows/          # GitHub Actions CI/CD pipelines
│   ├── ci.yml                 # Continuous Integration
│   ├── build-desktop.yml      # Desktop builds (Tauri)
│   ├── build-mobile.yml       # Mobile builds (Fastlane)
│   ├── deploy-relay.yml       # Cloudflare Workers deployment
│   ├── release.yml            # Release automation
│   └── security-audit.yml     # Security scanning
│
├── truffle-infra/terraform/    # Infrastructure as Code
│   ├── main.tf                # Terraform backend & core resources
│   ├── cloudflare.tf          # Workers, R2, DNS configuration
│   ├── variables.tf           # Input variables
│   ├── outputs.tf             # Output values
│   └── terraform.tfvars.example  # Example configuration
│
├── scripts/                    # Automation scripts
│   ├── build.sh               # Cross-platform build script
│   ├── sign.sh                # Code signing script
│   ├── deploy.sh              # Deployment orchestration
│   └── release.sh             # Release management
│
├── fastlane/                   # Mobile deployment
│   ├── Fastfile               # iOS/Android lanes
│   └── Appfile                # App configuration
│
├── monitoring/                 # Observability
│   ├── grafana-dashboards/    # Grafana dashboards
│   └── prometheus-config/     # Prometheus & alerts
│
└── tauri.conf.json            # Tauri configuration with updater
```

## 🚀 Quick Start

### Prerequisites

- Node.js 20+
- Rust 1.75+
- Terraform 1.5+
- Wrangler CLI
- GitHub CLI (optional)

### Environment Setup

1. **Set required environment variables:**

```bash
export CLOUDFLARE_API_TOKEN="your_api_token"
export CLOUDFLARE_ACCOUNT_ID="your_account_id"
export CLOUDFLARE_ZONE_ID="your_zone_id"
```

2. **Configure Terraform:**

```bash
cd truffle-infra/terraform
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with your values
```

3. **Initialize infrastructure:**

```bash
terraform init
terraform workspace new staging
terraform apply
```

## 🔧 CI/CD Pipelines

### Continuous Integration (ci.yml)

Runs on every push and PR:
- Code quality checks (lint, format)
- Unit tests (Jest + Cargo test) - 80% coverage required
- Integration tests (SQLite + Gemma 4)
- E2E tests (Playwright desktop)
- Performance budget checks (<100MB desktop, <50MB mobile)
- Security scans (npm audit, cargo audit, TruffleHog)

### Build Pipelines

**Desktop (build-desktop.yml):**
- macOS (ARM64 + x86_64 universal binary)
- Windows (x64 MSI + EXE)
- Linux (deb, rpm, AppImage)
- Code signing (Apple Developer ID, EV Certificate)
- Notarization (macOS)

**Mobile (build-mobile.yml):**
- iOS (TestFlight via Fastlane)
- Android (Play Console via Fastlane)
- Performance budget enforcement

### Deployment (deploy-relay.yml)

Zero-downtime deployment to Cloudflare Workers:
1. Deploy to staging
2. Run smoke tests
3. Deploy to production
4. Verify health checks
5. Auto-rollback on failure

### Release (release.yml)

Release strategy:
- **Canary:** 5% opt-in users
- **Stable:** Auto-update via Tauri updater
- **Hotfix:** Emergency patches

## 🏗️ Infrastructure

### Cloudflare Resources

| Resource | Purpose |
|----------|---------|
| Workers | Relay server, updater endpoint |
| R2 Buckets | Releases, sync data, AI models |
| KV Namespaces | Device registry, rate limits, sessions |
| D1 Database | Metadata storage |
| Queues | Async sync processing |

### Security Features

- Zero-knowledge architecture
- AES-256-GCM encryption
- X3DH key exchange
- Rate limiting
- WAF rules
- Turnstile bot protection

## 📊 Monitoring

### Metrics Collected

**Infrastructure:**
- Sync availability (SLO: 99.9%)
- Request latency (P50, P95, P99)
- Error rates
- R2 storage & bandwidth
- Rate limit usage

**Application:**
- Active devices
- Crash rates
- Compilation accuracy (target: >95%)
- Update success rates
- User satisfaction

### Alerts

Critical alerts:
- Sync availability < 95%
- Error rate > 5%
- R2 storage > 95%
- Rate limit exceeded

## 🔐 Code Signing

### macOS

Requires:
- Apple Developer ID certificate
- Apple ID for notarization
- Team ID

### Windows

Requires:
- EV Code Signing Certificate
- Certificate thumbprint

### Linux

Uses GPG signing for packages.

## 📝 Scripts Usage

### Build Script

```bash
# Build all platforms
./scripts/build.sh

# Build specific platform
./scripts/build.sh -t macos

# Debug build
./scripts/build.sh -b debug

# Clean build
./scripts/build.sh -c
```

### Deploy Script

```bash
# Deploy to staging
./scripts/deploy.sh deploy -e staging

# Deploy to production
./scripts/deploy.sh deploy -e production

# Show deployment plan
./scripts/deploy.sh plan -e production

# Rollback
./scripts/deploy.sh rollback -e production

# Show status
./scripts/deploy.sh status -e production
```

### Release Script

```bash
# Create canary release
./scripts/release.sh -t canary

# Create stable release
./scripts/release.sh -t stable -v 1.2.0

# Dry run
./scripts/release.sh -t stable --dry-run
```

## 🧪 Testing

### Test Coverage Requirements

- Unit tests: 80% minimum
- Integration tests: SQLite + Gemma 4
- E2E tests: Playwright (desktop), Detox (mobile)

### Running Tests

```bash
# Unit tests
npm run test:unit

# Rust tests
cd src-tauri && cargo test

# E2E tests
npm run test:e2e

# Integration tests
npm run test:integration
```

## 📦 Release Process

1. **Canary Release:**
   ```bash
   ./scripts/release.sh -t canary
   ```
   - 5% opt-in users
   - Monitor for 24-48 hours

2. **Promote to Stable:**
   ```bash
   ./scripts/release.sh -t stable -v X.Y.Z
   ```
   - Full rollout
   - Auto-update enabled

3. **Emergency Hotfix:**
   ```bash
   ./scripts/release.sh -t hotfix -v X.Y.Z+1 --skip-tests
   ```

## 🔒 Security

### Secrets Management

All secrets stored in GitHub Secrets:
- `CLOUDFLARE_API_TOKEN`
- `APPLE_CERTIFICATE`
- `WINDOWS_CERTIFICATE`
- `TAURI_PRIVATE_KEY`
- `SENTRY_DSN`
- `SLACK_WEBHOOK_URL`

### Security Scanning

Automated scans run daily:
- npm audit
- cargo audit
- TruffleHog (secrets)
- Snyk (dependencies)
- SBOM generation

## 📈 SLOs

| Metric | Target |
|--------|--------|
| Sync availability | 99.9% |
| Sync latency (P95) | <500ms |
| Model download speed | >10Mbps |
| Compilation accuracy | >95% |
| Update success rate | >99% |

## 🆘 Troubleshooting

### Common Issues

**Build failures:**
- Check Node.js and Rust versions
- Verify dependencies: `npm ci`
- Clear cache: `./scripts/build.sh -c`

**Deployment failures:**
- Verify Cloudflare credentials
- Check Terraform state: `terraform show`
- Review Wrangler logs

**Code signing issues:**
- Verify certificates are valid
- Check keychain access (macOS)
- Ensure correct thumbprint (Windows)

### Getting Help

- Check runbooks: https://wiki.truffle.io/runbooks
- Slack: #devops-support
- On-call: pagerduty.truffle.io

## 📚 References

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Cloudflare Workers](https://developers.cloudflare.com/workers/)
- [Terraform Cloudflare Provider](https://registry.terraform.io/providers/cloudflare/cloudflare/latest/docs)
- [Fastlane Documentation](https://docs.fastlane.tools/)

## 📄 License

Copyright © 2024 Truffle Inc. All rights reserved.
