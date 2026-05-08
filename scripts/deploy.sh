#!/bin/bash
# Project Truffle - Deployment Script
# Section 6.2: CI/CD Pipeline - Deployment Automation
# Zero-downtime deployments with rollback capability

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
INFRA_DIR="${PROJECT_ROOT}/truffle-infra/terraform"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
ENVIRONMENT="staging"
COMPONENT="all"
ACTION="deploy"
SKIP_TESTS=false
FORCE=false

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_usage() {
    cat << EOF
Truffle Deployment Script

Usage: $0 [OPTIONS] <action>

Actions:
    deploy              Deploy infrastructure and applications
    destroy             Destroy infrastructure (DANGEROUS)
    plan                Show deployment plan without applying
    rollback            Rollback to previous version
    status              Show deployment status

Options:
    -e, --env <env>         Environment (staging|production)
                            Default: staging
    -c, --component <comp>  Component to deploy (all|relay|infrastructure|desktop|mobile)
                            Default: all
    --skip-tests            Skip pre-deployment tests
    -f, --force             Force deployment without confirmation
    -v, --verbose           Verbose output
    -h, --help              Show this help message

Examples:
    $0 deploy                           # Deploy all to staging
    $0 deploy -e production             # Deploy all to production
    $0 deploy -c relay -e production    # Deploy only relay to production
    $0 plan -e production               # Show production deployment plan
    $0 rollback -e production           # Rollback production
    $0 status -e production             # Show production status

EOF
}

# ============================================================================
# Validation Functions
# ============================================================================

validate_environment() {
    local env="$1"
    
    if [[ "$env" != "staging" && "$env" != "production" ]]; then
        log_error "Invalid environment: $env"
        log_error "Must be 'staging' or 'production'"
        exit 1
    fi
}

validate_prerequisites() {
    log_info "Validating prerequisites..."
    
    # Check required tools
    local tools=("terraform" "wrangler" "jq")
    
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "$tool is not installed"
            exit 1
        fi
    done
    
    # Check environment variables
    if [[ -z "${CLOUDFLARE_API_TOKEN:-}" ]]; then
        log_error "CLOUDFLARE_API_TOKEN is not set"
        exit 1
    fi
    
    if [[ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]]; then
        log_error "CLOUDFLARE_ACCOUNT_ID is not set"
        exit 1
    fi
    
    log_success "Prerequisites validated"
}

confirm_deployment() {
    local env="$1"
    
    if [[ "$env" == "production" && "$FORCE" != true ]]; then
        echo ""
        log_warn "You are about to deploy to PRODUCTION!"
        read -p "Are you sure? Type 'yes' to continue: " confirm
        
        if [[ "$confirm" != "yes" ]]; then
            log_info "Deployment cancelled"
            exit 0
        fi
    fi
}

# ============================================================================
# Infrastructure Functions
# ============================================================================

deploy_infrastructure() {
    local env="$1"
    
    log_info "Deploying infrastructure to $env..."
    
    cd "$INFRA_DIR"
    
    # Initialize Terraform
    log_info "Initializing Terraform..."
    terraform init -upgrade
    
    # Select workspace
    terraform workspace select "$env" 2>/dev/null || terraform workspace new "$env"
    
    # Plan deployment
    log_info "Planning infrastructure changes..."
    terraform plan \
        -var="environment=$env" \
        -var="cloudflare_api_token=$CLOUDFLARE_API_TOKEN" \
        -var="cloudflare_account_id=$CLOUDFLARE_ACCOUNT_ID" \
        -out=tfplan
    
    # Apply deployment
    log_info "Applying infrastructure changes..."
    terraform apply tfplan
    
    # Output results
    log_info "Infrastructure outputs:"
    terraform output
    
    log_success "Infrastructure deployment complete"
}

destroy_infrastructure() {
    local env="$1"
    
    log_warn "DESTROYING infrastructure in $env!"
    
    if [[ "$FORCE" != true ]]; then
        read -p "Are you sure? Type 'destroy' to continue: " confirm
        
        if [[ "$confirm" != "destroy" ]]; then
            log_info "Destruction cancelled"
            exit 0
        fi
    fi
    
    cd "$INFRA_DIR"
    
    terraform workspace select "$env"
    
    terraform destroy \
        -var="environment=$env" \
        -var="cloudflare_api_token=$CLOUDFLARE_API_TOKEN" \
        -var="cloudflare_account_id=$CLOUDFLARE_ACCOUNT_ID" \
        -auto-approve
    
    log_success "Infrastructure destroyed"
}

plan_infrastructure() {
    local env="$1"
    
    log_info "Planning infrastructure changes for $env..."
    
    cd "$INFRA_DIR"
    
    terraform init
    terraform workspace select "$env" 2>/dev/null || terraform workspace new "$env"
    
    terraform plan \
        -var="environment=$env" \
        -var="cloudflare_api_token=$CLOUDFLARE_API_TOKEN" \
        -var="cloudflare_account_id=$CLOUDFLARE_ACCOUNT_ID"
}

# ============================================================================
# Relay Deployment Functions
# ============================================================================

deploy_relay() {
    local env="$1"
    
    log_info "Deploying relay to $env..."
    
    cd "${PROJECT_ROOT}/relay"
    
    # Install dependencies
    npm ci
    
    # Run tests if not skipped
    if [[ "$SKIP_TESTS" != true ]]; then
        log_info "Running tests..."
        npm test
    fi
    
    # Build
    log_info "Building relay..."
    npm run build
    
    # Deploy with Wrangler
    log_info "Deploying to Cloudflare Workers..."
    npx wrangler deploy --env "$env"
    
    # Run smoke tests
    log_info "Running smoke tests..."
    sleep 5
    
    local health_url
    if [[ "$env" == "production" ]]; then
        health_url="https://relay.truffle.io/health"
    else
        health_url="https://relay-staging.truffle.io/health"
    fi
    
    local health_status
    health_status=$(curl -s -o /dev/null -w "%{http_code}" "$health_url" || echo "000")
    
    if [[ "$health_status" != "200" ]]; then
        log_error "Health check failed with status $health_status"
        
        # Rollback on failure
        log_warn "Initiating rollback..."
        rollback_relay "$env"
        exit 1
    fi
    
    log_success "Relay deployment complete"
}

rollback_relay() {
    local env="$1"
    
    log_info "Rolling back relay in $env..."
    
    cd "${PROJECT_ROOT}/relay"
    
    # Get previous version
    local previous_version
    previous_version=$(npx wrangler versions list --env "$env" --json | jq -r '.[1].id')
    
    if [[ -n "$previous_version" && "$previous_version" != "null" ]]; then
        log_info "Rolling back to version: $previous_version"
        npx wrangler versions deploy "$previous_version" --env "$env"
        log_success "Rollback complete"
    else
        log_error "No previous version found for rollback"
        exit 1
    fi
}

# ============================================================================
# Desktop Deployment Functions
# ============================================================================

deploy_desktop() {
    local env="$1"
    
    log_info "Deploying desktop builds to $env..."
    
    # Check for release artifacts
    local release_dir="${PROJECT_ROOT}/dist"
    
    if [[ ! -d "$release_dir" ]]; then
        log_error "No release artifacts found in $release_dir"
        log_error "Run build.sh first"
        exit 1
    fi
    
    # Upload to R2
    log_info "Uploading to R2..."
    
    for file in "$release_dir"/**/*; do
        if [[ -f "$file" ]]; then
            local filename
            filename=$(basename "$file")
            
            log_info "Uploading: $filename"
            
            wrangler r2 object put \
                "truffle-releases/$env/$filename" \
                --file "$file" \
                --content-type "$(file -b --mime-type "$file" 2>/dev/null || echo 'application/octet-stream')"
        fi
    done
    
    # Update latest.json
    if [[ -f "$release_dir/latest.json" ]]; then
        wrangler r2 object put \
            "truffle-releases/$env/latest.json" \
            --file "$release_dir/latest.json" \
            --content-type "application/json"
    fi
    
    # Purge CDN cache
    log_info "Purging CDN cache..."
    curl -X POST "https://api.cloudflare.com/client/v4/zones/${CLOUDFLARE_ZONE_ID}/purge_cache" \
        -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
        -H "Content-Type: application/json" \
        --data '{"files":["https://releases.truffle.io/'$env'/*"]}'
    
    log_success "Desktop deployment complete"
}

# ============================================================================
# Mobile Deployment Functions
# ============================================================================

deploy_mobile() {
    local env="$1"
    
    log_info "Deploying mobile builds to $env..."
    
    # iOS deployment via Fastlane
    if [[ -d "${PROJECT_ROOT}/mobile/ios" ]]; then
        log_info "Deploying iOS..."
        cd "${PROJECT_ROOT}/mobile/ios"
        
        if [[ "$env" == "production" ]]; then
            fastlane release
        else
            fastlane beta
        fi
    fi
    
    # Android deployment via Fastlane
    if [[ -d "${PROJECT_ROOT}/mobile/android" ]]; then
        log_info "Deploying Android..."
        cd "${PROJECT_ROOT}/mobile/android"
        
        if [[ "$env" == "production" ]]; then
            fastlane deploy_production
        else
            fastlane deploy_beta
        fi
    fi
    
    log_success "Mobile deployment complete"
}

# ============================================================================
# Status Functions
# ============================================================================

show_status() {
    local env="$1"
    
    log_info "Deployment status for $env:"
    
    echo ""
    echo "=== Infrastructure ==="
    cd "$INFRA_DIR"
    terraform workspace select "$env" 2>/dev/null || echo "Workspace not found"
    terraform output 2>/dev/null || echo "No outputs available"
    
    echo ""
    echo "=== Relay Health ==="
    local health_url
    if [[ "$env" == "production" ]]; then
        health_url="https://relay.truffle.io/health"
    else
        health_url="https://relay-staging.truffle.io/health"
    fi
    
    curl -s "$health_url" | jq . 2>/dev/null || echo "Health check failed"
    
    echo ""
    echo "=== Workers Versions ==="
    cd "${PROJECT_ROOT}/relay"
    npx wrangler versions list --env "$env" 2>/dev/null | head -10 || echo "No versions found"
}

# ============================================================================
# Main
# ============================================================================

main() {
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -e|--env)
                ENVIRONMENT="$2"
                shift 2
                ;;
            -c|--component)
                COMPONENT="$2"
                shift 2
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -v|--verbose)
                set -x
                shift
                ;;
            -h|--help)
                print_usage
                exit 0
                ;;
            deploy|destroy|plan|rollback|status)
                ACTION="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done
    
    # Validate environment
    validate_environment "$ENVIRONMENT"
    
    # Validate prerequisites
    validate_prerequisites
    
    # Confirm production deployments
    if [[ "$ACTION" == "deploy" || "$ACTION" == "destroy" ]]; then
        confirm_deployment "$ENVIRONMENT"
    fi
    
    # Execute action
    case "$ACTION" in
        deploy)
            log_info "Starting deployment to $ENVIRONMENT..."
            
            case "$COMPONENT" in
                all)
                    deploy_infrastructure "$ENVIRONMENT"
                    deploy_relay "$ENVIRONMENT"
                    ;;
                infrastructure)
                    deploy_infrastructure "$ENVIRONMENT"
                    ;;
                relay)
                    deploy_relay "$ENVIRONMENT"
                    ;;
                desktop)
                    deploy_desktop "$ENVIRONMENT"
                    ;;
                mobile)
                    deploy_mobile "$ENVIRONMENT"
                    ;;
                *)
                    log_error "Unknown component: $COMPONENT"
                    exit 1
                    ;;
            esac
            
            log_success "Deployment complete!"
            ;;
        
        destroy)
            destroy_infrastructure "$ENVIRONMENT"
            ;;
        
        plan)
            plan_infrastructure "$ENVIRONMENT"
            ;;
        
        rollback)
            log_info "Rolling back $COMPONENT in $ENVIRONMENT..."
            
            case "$COMPONENT" in
                all|relay)
                    rollback_relay "$ENVIRONMENT"
                    ;;
                *)
                    log_error "Rollback not implemented for component: $COMPONENT"
                    exit 1
                    ;;
            esac
            
            log_success "Rollback complete!"
            ;;
        
        status)
            show_status "$ENVIRONMENT"
            ;;
        
        *)
            log_error "Unknown action: $ACTION"
            print_usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
