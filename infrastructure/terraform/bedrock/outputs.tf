output "bedrock_logging_role_arn" {
  description = "ARN of the Bedrock logging IAM role"
  value       = aws_iam_role.bedrock_logging_role.arn
}

output "bedrock_log_group_name" {
  description = "Name of the CloudWatch log group for Bedrock"
  value       = aws_cloudwatch_log_group.bedrock_logs.name
}

output "bedrock_log_group_arn" {
  description = "ARN of the CloudWatch log group for Bedrock"
  value       = aws_cloudwatch_log_group.bedrock_logs.arn
}

output "knowledge_base_message" {
  description = "Message about Knowledge Base creation status"
  value       = local.knowledge_base_message
}

output "bedrock_kb_role_arn" {
  description = "ARN of the IAM role for future Knowledge Base use"
  value       = var.create_knowledge_base ? aws_iam_role.bedrock_kb_role[0].arn : null
}

output "s3_bucket_name" {
  description = "Name of the S3 bucket for Bedrock artifacts"
  value       = var.create_s3_bucket ? aws_s3_bucket.bedrock_artifacts[0].bucket : null
}

output "s3_bucket_arn" {
  description = "ARN of the S3 bucket for Bedrock artifacts"
  value       = var.create_s3_bucket ? aws_s3_bucket.bedrock_artifacts[0].arn : null
}

output "aws_region" {
  description = "AWS region where resources are created"
  value       = data.aws_region.current.name
}

output "aws_account_id" {
  description = "AWS account ID"
  value       = data.aws_caller_identity.current.account_id
} 