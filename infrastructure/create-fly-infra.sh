#!/bin/bash

# create-fly-infra.sh - Apply Terraform configuration and deploy all infrastructure services to Fly.io
# Usage: ./create-fly-infra.sh [service_name] or ./create-fly-infra.sh all

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if flyctl is installed
check_flyctl() {
    if ! command -v flyctl &> /dev/null; then
        log_error "flyctl is not installed. Please install it first: https://fly.io/docs/getting-started/installing-flyctl/"
        exit 1
    fi
}

# Check if terraform is installed
check_terraform() {
    if ! command -v terraform &> /dev/null; then
        log_error "terraform is not installed. Please install it first: https://developer.hashicorp.com/terraform/downloads"
        exit 1
    fi
}

# Check if we're authenticated with Fly.io
check_auth() {
    if ! flyctl auth whoami &> /dev/null; then
        log_error "Not authenticated with Fly.io. Please run 'flyctl auth login' first."
        exit 1
    fi
}

# Apply Terraform configuration
apply_terraform() {
    local terraform_dir="./terraform/flyio"
    
    log_info "Applying Terraform configuration..."
    
    if [ ! -d "$terraform_dir" ]; then
        log_error "Terraform directory $terraform_dir does not exist"
        return 1
    fi
    
    # Change to terraform directory
    cd "$terraform_dir"
    
    # Check if main.tf exists
    if [ ! -f "main.tf" ]; then
        log_error "No main.tf found in $terraform_dir"
        cd - > /dev/null
        return 1
    fi
    
    # Initialize terraform if .terraform directory doesn't exist
    if [ ! -d ".terraform" ]; then
        log_info "Initializing Terraform..."
        if ! terraform init; then
            log_error "Failed to initialize Terraform"
            cd - > /dev/null
            return 1
        fi
    fi
    
    # Plan and apply terraform
    log_info "Planning Terraform changes..."
    if ! terraform plan -out=tfplan; then
        log_error "Terraform plan failed"
        cd - > /dev/null
        return 1
    fi
    
    log_info "Applying Terraform configuration..."
    if terraform apply tfplan; then
        log_success "Terraform configuration applied successfully"
        # Clean up plan file
        rm -f tfplan
        cd - > /dev/null
        return 0
    else
        log_error "Failed to apply Terraform configuration"
        cd - > /dev/null
        return 1
    fi
}

# Deploy a specific service
deploy_service() {
    local service_dir=$1
    local service_name=$2
    local app_name=$3
    
    log_info "Deploying $service_name..."
    
    if [ ! -d "$service_dir" ]; then
        log_error "Service directory $service_dir does not exist"
        return 1
    fi
    
    if [ ! -f "$service_dir/fly.toml" ]; then
        log_error "No fly.toml found in $service_dir"
        return 1
    fi
    
    # Change to service directory
    cd "$service_dir"
    
    # Deploy the service
    if flyctl deploy --remote-only; then
        log_success "$service_name deployed successfully"
        cd - > /dev/null
        return 0
    else
        log_error "Failed to deploy $service_name"
        cd - > /dev/null
        return 1
    fi
}

# List of services to deploy (directory:service_name:app_name)
SERVICES=(
    "grafana:grafana:helicone-grafana"
    "loki:loki:helicone-loki"
    "tempo:tempo:helicone-tempo"
    "opentelemetry-collector:otel-collector:helicone-otel-collector"
    "prometheus:prometheus:helicone-prometheus"
    # "redis:redis:helicone-redis-cache"  # Removed - no longer supported in Terraform config
)

# Deploy service in background and track status
deploy_service_parallel() {
    local service_dir=$1
    local service_name=$2
    local app_name=$3
    local status_file=$4
    
    # Deploy the service and capture result
    if deploy_service "$service_dir" "$service_name" "$app_name" 2>&1; then
        echo "SUCCESS:$service_name" > "$status_file"
    else
        echo "FAILED:$service_name" > "$status_file"
    fi
}

# Main deployment function - parallel execution
deploy_all() {
    local failed_services=()
    local deployed_count=0
    local pids=()
    local temp_dir
    temp_dir=$(mktemp -d)
    
    log_info "Starting infrastructure deployment..."
    
    # First apply Terraform configuration
    log_info "Step 1: Applying Terraform configuration..."
    if ! apply_terraform; then
        log_error "Terraform apply failed. Aborting deployment."
        return 1
    fi
    
    log_info "Step 2: Starting parallel deployment of all infrastructure services..."
    
    # Start all deployments in parallel
    for i in "${!SERVICES[@]}"; do
        local service_config="${SERVICES[$i]}"
        IFS=':' read -r service_dir service_name app_name <<< "$service_config"
        
        local status_file="$temp_dir/status_$i"
        log_info "Starting deployment of $service_name in background..."
        
        # Start deployment in background
        deploy_service_parallel "$service_dir" "$service_name" "$app_name" "$status_file" &
        local pid=$!
        pids+=($pid)
        
        # Store service info for this PID
        echo "$service_name:$pid" >> "$temp_dir/service_pids"
    done
    
    log_info "All deployments started. Waiting for completion..."
    echo "Active deployments:"
    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service_dir service_name app_name <<< "$service_config"
        echo "  - $service_name → $app_name"
    done
    echo ""
    
    # Wait for all background processes to complete
    local completed=0
    local total=${#SERVICES[@]}
    
    while [ $completed -lt $total ]; do
        sleep 2
        completed=0
        
        # Count completed deployments
        for i in "${!SERVICES[@]}"; do
            local status_file="$temp_dir/status_$i"
            if [ -f "$status_file" ]; then
                ((completed++))
            fi
        done
        
        if [ $completed -lt $total ]; then
            log_info "Progress: $completed/$total deployments completed..."
        fi
    done
    
    # Wait for all PIDs to ensure clean completion
    for pid in "${pids[@]}"; do
        wait "$pid" 2>/dev/null || true
    done
    
    echo ""
    log_info "All deployments completed. Collecting results..."
    
    # Collect results
    for i in "${!SERVICES[@]}"; do
        local service_config="${SERVICES[$i]}"
        IFS=':' read -r service_dir service_name app_name <<< "$service_config"
        local status_file="$temp_dir/status_$i"
        
        if [ -f "$status_file" ]; then
            local status
            status=$(cat "$status_file")
            if [[ "$status" == "SUCCESS:"* ]]; then
                ((deployed_count++))
                log_success "$service_name deployed successfully"
            else
                failed_services+=("$service_name")
                log_error "$service_name deployment failed"
            fi
        else
            failed_services+=("$service_name")
            log_error "$service_name deployment status unknown"
        fi
    done
    
    # Cleanup
    rm -rf "$temp_dir"
    
    # Summary
    echo ""
    echo "=================================================="
    log_info "Parallel Deployment Summary:"
    log_success "Successfully deployed: $deployed_count/$total services"
    
    if [ ${#failed_services[@]} -gt 0 ]; then
        log_error "Failed deployments: ${failed_services[*]}"
        return 1
    else
        log_success "Infrastructure deployment completed successfully! 🚀"
        log_success "✅ Terraform configuration applied"
        log_success "✅ All $deployed_count services deployed in parallel"
        return 0
    fi
}

# Deploy specific service
deploy_specific() {
    local target_service=$1
    local found=false
    
    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service_dir service_name app_name <<< "$service_config"
        
        if [ "$service_name" = "$target_service" ]; then
            found=true
            deploy_service "$service_dir" "$service_name" "$app_name"
            return $?
        fi
    done
    
    if [ "$found" = false ]; then
        log_error "Service '$target_service' not found. Available services:"
        for service_config in "${SERVICES[@]}"; do
            IFS=':' read -r service_dir service_name app_name <<< "$service_config"
            echo "  - $service_name"
        done
        return 1
    fi
}

# Show usage
show_usage() {
    echo "Usage: $0 [service_name|all]"
    echo ""
    echo "Available services:"
    for service_config in "${SERVICES[@]}"; do
        IFS=':' read -r service_dir service_name app_name <<< "$service_config"
        echo "  - $service_name (app: $app_name)"
    done
    echo ""
    echo "Examples:"
    echo "  $0 all           # Apply Terraform config, then deploy all services in parallel"
    echo "  $0 grafana       # Deploy only grafana (skips Terraform)"
    echo "  $0 prometheus    # Deploy only prometheus (skips Terraform)"
    echo "  $0 otel-collector # Deploy only otel-collector (skips Terraform)"
    echo ""
    echo "Features:"
    echo "  - Terraform integration: Automatically applies Terraform configuration before service deployment"
    echo "  - Parallel deployment: All services deploy simultaneously for faster execution"
    echo "  - Progress tracking: Real-time updates on deployment status"
    echo "  - Error handling: Individual service failures won't stop other deployments"
    echo ""
    echo "Prerequisites:"
    echo "  - terraform: Required for infrastructure provisioning"
    echo "  - flyctl: Required for service deployment"
    echo "  - Authenticated with Fly.io (flyctl auth login)"
    echo ""
    echo "Note: When deploying 'all', Terraform configuration is applied first from terraform/flyio/"
}

# Main script execution
main() {
    # Ensure we're in the infrastructure directory
    if [ ! -f "compose.yaml" ] || [ ! -d "grafana" ]; then
        log_error "Please run this script from the infrastructure directory"
        exit 1
    fi
    
    # Pre-flight checks
    check_flyctl
    check_terraform
    check_auth
    
    local current_user
    current_user=$(flyctl auth whoami)
    log_info "Deploying as: $current_user"
    
    # Handle command line arguments
    case "${1:-all}" in
        "all")
            deploy_all
            ;;
        "help"|"-h"|"--help")
            show_usage
            ;;
        "")
            deploy_all
            ;;
        *)
            deploy_specific "$1"
            ;;
    esac
}

# Run main function
main "$@" 