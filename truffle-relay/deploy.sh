#!/bin/bash
# Project Truffle - Relay Server Deployment Script
# SPEC v2.0 SECTION 6.1 - Cloud Architecture

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-staging}
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(development|staging|production)$ ]]; then
    echo -e "${RED}Error: Environment must be one of: development, staging, production${NC}"
    exit 1
fi

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}Project Truffle - Relay Server Deployment${NC}"
echo -e "${BLUE}Environment: $ENVIRONMENT${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# Step 1: Install dependencies
echo -e "${YELLOW}Step 1: Installing dependencies...${NC}"
npm install

# Step 2: Run type checking
echo -e "${YELLOW}Step 2: Running TypeScript type check...${NC}"
npm run typecheck

# Step 3: Run linter
echo -e "${YELLOW}Step 3: Running linter...${NC}"
npm run lint

# Step 4: Run tests
echo -e "${YELLOW}Step 4: Running tests...${NC}"
npm run test

# Step 5: Build the project
echo -e "${YELLOW}Step 5: Building project...${NC}"
npm run cf-typegen

# Step 6: Deploy to Cloudflare
echo -e "${YELLOW}Step 6: Deploying to Cloudflare ($ENVIRONMENT)...${NC}"

if [ "$ENVIRONMENT" == "production" ]; then
    npm run deploy:production
elif [ "$ENVIRONMENT" == "staging" ]; then
    npm run deploy:staging
else
    npm run deploy
fi

# Step 7: Set secrets (if in production)
if [ "$ENVIRONMENT" == "production" ]; then
    echo -e "${YELLOW}Step 7: Checking secrets...${NC}"
    echo -e "${YELLOW}Note: Ensure secrets are set using:${NC}"
    echo -e "${YELLOW}  wrangler secret put JWT_SECRET --env production${NC}"
    echo -e "${YELLOW}  wrangler secret put CLERK_SECRET_KEY --env production${NC}"
fi

# Step 8: Verify deployment
echo -e "${YELLOW}Step 8: Verifying deployment...${NC}"

# Get the worker URL
if [ "$ENVIRONMENT" == "production" ]; then
    WORKER_URL="https://truffle-relay-production.YOUR_ZONE.com"
elif [ "$ENVIRONMENT" == "staging" ]; then
    WORKER_URL="https://truffle-relay-staging.YOUR_ZONE.com"
else
    WORKER_URL="http://localhost:8787"
fi

# Health check
HEALTH_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$WORKER_URL/health" || echo "000")

if [ "$HEALTH_STATUS" == "200" ]; then
    echo -e "${GREEN}Health check passed!${NC}"
    curl -s "$WORKER_URL/health" | jq .
else
    echo -e "${RED}Health check failed (status: $HEALTH_STATUS)${NC}"
    exit 1
fi

# Zero-access verification
echo -e "${YELLOW}Step 9: Running zero-access verification...${NC}"
ZERO_ACCESS_STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$WORKER_URL/verify-zero-access" || echo "000")

if [ "$ZERO_ACCESS_STATUS" == "200" ]; then
    echo -e "${GREEN}Zero-access verification passed!${NC}"
    curl -s "$WORKER_URL/verify-zero-access" | jq .
else
    echo -e "${RED}Zero-access verification failed (status: $ZERO_ACCESS_STATUS)${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}Deployment completed successfully!${NC}"
echo -e "${GREEN}Environment: $ENVIRONMENT${NC}"
echo -e "${GREEN}Worker URL: $WORKER_URL${NC}"
echo -e "${GREEN}============================================${NC}"
