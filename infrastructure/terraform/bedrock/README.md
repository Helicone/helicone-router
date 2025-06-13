# AWS Bedrock Terraform Configuration

This Terraform configuration sets up AWS Bedrock resources for the Helicone project.

## Resources Created

### Core Resources

- **Model Invocation Logging**: Captures logs from Bedrock model invocations
- **CloudWatch Log Group**: Stores Bedrock logs with configurable retention
- **IAM Roles and Policies**: Proper permissions for Bedrock services

### Optional Resources (configurable via variables)

- **Knowledge Base**: Vector database for document retrieval (RAG)
- **OpenSearch Serverless Collection**: Backend for the Knowledge Base
- **S3 Bucket**: Storage for Bedrock artifacts and documents

## Prerequisites

1. **AWS CLI configured** with appropriate permissions
2. **Terraform >= 1.0** installed
3. **Bedrock Model Access**: You must manually request access to Bedrock models in the AWS Console before using them

### Required AWS Permissions

Your AWS credentials need the following permissions:

- `bedrock:*`
- `logs:*`
- `iam:*`
- `s3:*` (if creating S3 bucket)
- `aoss:*` (if creating Knowledge Base)

## Usage

### 1. Configure Variables

Copy the example variables file:

```bash
cp terraform.tfvars.example terraform.tfvars
```

Edit `terraform.tfvars` with your specific configuration.

### 2. Initialize Terraform

```bash
terraform init
```

### 3. Plan the Deployment

```bash
terraform plan
```

### 4. Apply the Configuration

```bash
terraform apply
```

## Important Notes

### Bedrock Model Access

Before using any Bedrock models, you must:

1. Go to AWS Console → Bedrock → Model access
2. Request access to the models you want to use
3. Wait for approval (usually immediate for most models)

### Knowledge Base Setup

If you enable the Knowledge Base (`create_knowledge_base = true`):

- An OpenSearch Serverless collection will be created
- You'll need to separately upload documents and create data sources
- Additional costs will apply for OpenSearch Serverless

### Cost Considerations

- CloudWatch Logs: Minimal cost for log storage
- Bedrock Model Usage: Pay-per-use pricing
- OpenSearch Serverless: Hourly charges when Knowledge Base is enabled
- S3 Storage: Standard S3 pricing for artifacts

## Configuration Examples

### Basic Setup (Logging Only)

```hcl
aws_region = "us-east-1"
project_name = "helicone"
create_knowledge_base = false
create_s3_bucket = false
```

### Full Setup with Knowledge Base

```hcl
aws_region = "us-east-1"
project_name = "helicone"
create_knowledge_base = true
create_s3_bucket = true
knowledge_base_name = "helicone-docs"
```

## Outputs

After deployment, the following outputs are available:

- `bedrock_logging_role_arn`: IAM role for Bedrock logging
- `bedrock_log_group_name`: CloudWatch log group name
- `knowledge_base_id`: Knowledge Base ID (if created)
- `s3_bucket_name`: S3 bucket name (if created)

## Cleanup

To destroy all resources:

```bash
terraform destroy
```

**Warning**: This will delete all Bedrock resources including any uploaded documents in the Knowledge Base.

## Troubleshooting

### Model Access Denied

If you get permission errors when using models:

1. Check AWS Console → Bedrock → Model access
2. Ensure the model is enabled for your account
3. Verify your AWS credentials have Bedrock permissions

### OpenSearch Collection Issues

If Knowledge Base creation fails:

- Ensure your AWS account has OpenSearch Serverless enabled
- Check that you have sufficient service quotas
- Verify IAM permissions for OpenSearch Serverless

## Security Best Practices

- Enable CloudTrail logging for Bedrock API calls
- Use least-privilege IAM policies
- Encrypt sensitive data in S3 buckets
- Monitor CloudWatch logs for unusual activity
- Regularly rotate AWS access keys
