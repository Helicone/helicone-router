provider "aws" {
  region = var.aws_region
}

# Data source to get current AWS account ID and region
data "aws_caller_identity" "current" {}
data "aws_region" "current" {}

# Enable Bedrock model invocation logging
resource "aws_bedrock_model_invocation_logging_configuration" "main" {
  logging_config {
    cloudwatch_config {
      log_group_name = aws_cloudwatch_log_group.bedrock_logs.name
      role_arn      = aws_iam_role.bedrock_logging_role.arn
    }
    text_data_delivery_enabled = true
    image_data_delivery_enabled = true
    embedding_data_delivery_enabled = true
  }
}

# CloudWatch Log Group for Bedrock logs
resource "aws_cloudwatch_log_group" "bedrock_logs" {
  name              = "/aws/bedrock/model-invocation-logs"
  retention_in_days = var.log_retention_days
  
  tags = merge(var.common_tags, {
    Name = "bedrock-model-invocation-logs"
  })
}

# IAM Role for Bedrock logging
resource "aws_iam_role" "bedrock_logging_role" {
  name = "bedrock-logging-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "bedrock.amazonaws.com"
        }
      }
    ]
  })

  tags = var.common_tags
}

# IAM Policy for Bedrock logging
resource "aws_iam_role_policy" "bedrock_logging_policy" {
  name = "bedrock-logging-policy"
  role = aws_iam_role.bedrock_logging_role.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "${aws_cloudwatch_log_group.bedrock_logs.arn}:*"
      }
    ]
  })
}

# Note: Bedrock Knowledge Base resources are not yet supported in the current AWS provider version
# These resources will be added in a future update when the provider supports them
# For now, you can create Knowledge Bases manually through the AWS Console if needed

# Placeholder for future Knowledge Base support
locals {
  knowledge_base_enabled = var.create_knowledge_base
  knowledge_base_message = var.create_knowledge_base ? "Knowledge Base creation is not yet supported in Terraform. Please create manually in AWS Console." : ""
}

# IAM Role for potential future Knowledge Base use
resource "aws_iam_role" "bedrock_kb_role" {
  count = var.create_knowledge_base ? 1 : 0
  
  name = "bedrock-kb-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "bedrock.amazonaws.com"
        }
      }
    ]
  })

  tags = merge(var.common_tags, {
    Name = "bedrock-kb-role"
    Purpose = "Future Knowledge Base support"
  })
}

# IAM Policy for potential future Knowledge Base use
resource "aws_iam_role_policy" "bedrock_kb_policy" {
  count = var.create_knowledge_base ? 1 : 0
  
  name = "bedrock-kb-policy"
  role = aws_iam_role.bedrock_kb_role[0].id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "bedrock:InvokeModel",
          "bedrock:RetrieveAndGenerate",
          "bedrock:Retrieve"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "aoss:APIAccessAll"
        ]
        Resource = "*"
      }
    ]
  })
}

# S3 Bucket for Bedrock artifacts (optional)
resource "aws_s3_bucket" "bedrock_artifacts" {
  count = var.create_s3_bucket ? 1 : 0
  
  bucket = "${var.project_name}-bedrock-artifacts-${random_id.bucket_suffix[0].hex}"

  tags = merge(var.common_tags, {
    Name = "${var.project_name}-bedrock-artifacts"
  })
}

resource "random_id" "bucket_suffix" {
  count = var.create_s3_bucket ? 1 : 0
  byte_length = 4
}

# S3 Bucket versioning
resource "aws_s3_bucket_versioning" "bedrock_artifacts" {
  count = var.create_s3_bucket ? 1 : 0
  
  bucket = aws_s3_bucket.bedrock_artifacts[0].id
  versioning_configuration {
    status = "Enabled"
  }
}

# S3 Bucket encryption
resource "aws_s3_bucket_server_side_encryption_configuration" "bedrock_artifacts" {
  count = var.create_s3_bucket ? 1 : 0
  
  bucket = aws_s3_bucket.bedrock_artifacts[0].id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

# S3 Bucket public access block
resource "aws_s3_bucket_public_access_block" "bedrock_artifacts" {
  count = var.create_s3_bucket ? 1 : 0
  
  bucket = aws_s3_bucket.bedrock_artifacts[0].id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
} 