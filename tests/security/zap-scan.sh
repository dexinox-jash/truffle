#!/bin/bash
# OWASP ZAP Security Scan

set -e

TARGET_URL=${1:-"https://api.staging.ruflo.ai"}
REPORT_DIR="security-reports"
mkdir -p $REPORT_DIR

echo "Starting OWASP ZAP security scan against $TARGET_URL"

# Run ZAP baseline scan
docker run -v $(pwd)/$REPORT_DIR:/zap/wrk/:rw \
    -t owasp/zap2docker-stable zap-baseline.py \
    -t $TARGET_URL \
    -g gen.conf \
    -r zap-baseline-report.html \
    -J zap-report.json \
    -w zap-md-report.md || true

# Run full scan for API endpoints
docker run -v $(pwd)/$REPORT_DIR:/zap/wrk/:rw \
    -t owasp/zap2docker-stable zap-full-scan.py \
    -t $TARGET_URL/api/v1 \
    -g api-scan.conf \
    -r zap-full-report.html \
    -J zap-full-report.json || true

echo "Security scan complete. Reports in $REPORT_DIR/"
