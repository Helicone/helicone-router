#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
AWS_REGION=${AWS_REGION:-us-east-1}
ECR_REGION=${ECR_REGION:-us-east-2}
ENVIRONMENT=${ENVIRONMENT:-dev}
AWS_ACCOUNT_ID=${AWS_ACCOUNT_ID:-849596434884}
ECR_REPOSITORY="helicone/aigateway"
IMAGE_TAG=${IMAGE_TAG:-latest}

echo -e "${GREEN}🚀 Starting Helicone Helix deployment to AWS ECS${NC}"

# Function to check if required tools are installed
check_dependencies() {
    echo -e "${YELLOW}📋 Checking dependencies...${NC}"
    
    if ! command -v aws &> /dev/null; then
        echo -e "${RED}❌ AWS CLI is not installed. Please install it first.${NC}"
        echo "Installation: https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}❌ Docker is not installed. Please install it first.${NC}"
        echo "Installation: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    if ! command -v terraform &> /dev/null; then
        echo -e "${RED}❌ Terraform is not installed. Please install it first.${NC}"
        echo "Installation: https://learn.hashicorp.com/tutorials/terraform/install-cli"
        exit 1
    fi
    
    echo -e "${GREEN}✅ All dependencies found${NC}"
}

# Function to check AWS authentication
check_aws_auth() {
    echo -e "${YELLOW}🔐 Checking AWS authentication...${NC}"
    
    if ! aws sts get-caller-identity &> /dev/null; then
        echo -e "${RED}❌ AWS authentication failed. Please configure your AWS credentials.${NC}"
        echo "Run: aws configure"
        exit 1
    fi
    
    ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
    echo -e "${GREEN}✅ Authenticated as account: ${ACCOUNT_ID}${NC}"
    
    # Update AWS_ACCOUNT_ID if not provided
    if [[ "$AWS_ACCOUNT_ID" == "849596434884" ]]; then
        AWS_ACCOUNT_ID=$ACCOUNT_ID
        echo -e "${YELLOW}📝 Using detected AWS Account ID: ${AWS_ACCOUNT_ID}${NC}"
    fi
}

# Function to create ECR repository if it doesn't exist
setup_ecr() {
    echo -e "${YELLOW}🐳 Setting up ECR repository...${NC}"
    
    # Check if repository exists
    if ! aws ecr describe-repositories --region $ECR_REGION --repository-names $ECR_REPOSITORY &> /dev/null; then
        echo -e "${YELLOW}📝 Creating ECR repository: ${ECR_REPOSITORY}${NC}"
        aws ecr create-repository \
            --region $ECR_REGION \
            --repository-name $ECR_REPOSITORY \
            --image-scanning-configuration scanOnPush=true
        echo -e "${GREEN}✅ ECR repository created${NC}"
    else
        echo -e "${GREEN}✅ ECR repository already exists${NC}"
    fi
}

# Function to build and push Docker image
build_and_push() {
    echo -e "${YELLOW}🔨 Building Docker image...${NC}"
    
    # Ensure we're in the project root directory for Docker build
    PROJECT_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
    
    # Check if Dockerfile exists
    if [[ ! -f "$PROJECT_ROOT/Dockerfile" ]]; then
        echo -e "${RED}❌ Dockerfile not found at $PROJECT_ROOT/Dockerfile${NC}"
        echo "Please run this script from the project root or ensure Dockerfile exists."
        exit 1
    fi
    
    # Build the image from project root
    echo -e "${YELLOW}📝 Building from: $PROJECT_ROOT${NC}"
    docker build -t $ECR_REPOSITORY:$IMAGE_TAG "$PROJECT_ROOT"
    
    # Tag for ECR
    ECR_URI="${AWS_ACCOUNT_ID}.dkr.ecr.${ECR_REGION}.amazonaws.com/${ECR_REPOSITORY}:${IMAGE_TAG}"
    docker tag $ECR_REPOSITORY:$IMAGE_TAG $ECR_URI
    
    echo -e "${YELLOW}📤 Pushing to ECR...${NC}"
    
    # Login to ECR
    aws ecr get-login-password --region $ECR_REGION | docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.${ECR_REGION}.amazonaws.com
    
    # Push the image
    docker push $ECR_URI
    
    echo -e "${GREEN}✅ Image pushed to ECR: ${ECR_URI}${NC}"
}

# Function to deploy with Terraform
deploy_terraform() {
    echo -e "${YELLOW}🏗️  Deploying with Terraform...${NC}"
    
    # Get the script directory and navigate to terraform directory
    SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
    TERRAFORM_DIR="$SCRIPT_DIR/terraform/ecs"
    
    # Check if terraform directory exists
    if [[ ! -d "$TERRAFORM_DIR" ]]; then
        echo -e "${RED}❌ Terraform directory not found at: ${TERRAFORM_DIR}${NC}"
        echo "Please ensure the terraform/ecs directory exists in the infrastructure folder."
        exit 1
    fi
    
    cd "$TERRAFORM_DIR"
    
    # Initialize Terraform
    echo -e "${YELLOW}📝 Initializing Terraform...${NC}"
    terraform init
    
    # Plan the deployment
    echo -e "${YELLOW}📋 Planning Terraform deployment...${NC}"
    terraform plan \
        -var="environment=${ENVIRONMENT}" \
        -var="region=${AWS_REGION}" \
        -out=tfplan
    
    # Apply the deployment
    echo -e "${YELLOW}🚀 Applying Terraform deployment...${NC}"
    terraform apply tfplan
    
    # Get outputs
    echo -e "${GREEN}📋 Deployment outputs:${NC}"
    terraform output
    
    # Return to original directory
    cd - > /dev/null
}

# Function to wait for deployment to be ready
wait_for_deployment() {
    echo -e "${YELLOW}⏳ Waiting for ECS service to be stable...${NC}"
    
    aws ecs wait services-stable \
        --region $AWS_REGION \
        --cluster "aigateway-cluster-${ENVIRONMENT}" \
        --services "aigateway-service-${ENVIRONMENT}"
    
    echo -e "${GREEN}✅ ECS service is stable and ready${NC}"
}

# Function to get the load balancer URL
get_endpoint() {
    echo -e "${YELLOW}🌐 Getting deployment endpoint...${NC}"
    
    # Get the script directory and navigate to terraform directory
    SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
    TERRAFORM_DIR="$SCRIPT_DIR/terraform/ecs"
    
    cd "$TERRAFORM_DIR"
    LB_DNS=$(terraform output -raw load_balancer_dns_name 2>/dev/null || echo "")
    cd - > /dev/null
    
    if [[ -n "$LB_DNS" ]]; then
        echo -e "${GREEN}🎉 Your Helicone Helix deployment is ready!${NC}"
        echo -e "${GREEN}📍 Endpoint: http://${LB_DNS}${NC}"
        echo -e "${GREEN}🧪 Test it: curl http://${LB_DNS}/health${NC}"
    else
        echo -e "${YELLOW}⚠️  Could not retrieve load balancer DNS. Check AWS console for the endpoint.${NC}"
    fi
}

# Main deployment flow
main() {
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                   Helicone Helix Deployer                    ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    check_dependencies
    check_aws_auth
    setup_ecr
    build_and_push
    deploy_terraform
    wait_for_deployment
    get_endpoint
    
    echo ""
    echo -e "${GREEN}🎉 Deployment completed successfully!${NC}"
    echo -e "${GREEN}💡 Your Helicone Helix router is now running on AWS ECS${NC}"
    echo ""
    echo -e "${YELLOW}📚 Next steps:${NC}"
    echo "   • Configure your API keys in the ECS task definition"
    echo "   • Set up custom domain with Route53 (optional)"
    echo "   • Configure HTTPS with ACM certificate"
    echo "   • Monitor with CloudWatch logs"
    echo ""
}

# Run main function
main "$@"
