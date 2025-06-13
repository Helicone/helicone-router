variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "helicone"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
}

variable "log_retention_days" {
  description = "Number of days to retain CloudWatch logs"
  type        = number
  default     = 30
}

variable "common_tags" {
  description = "Common tags to be applied to all resources"
  type        = map(string)
  default = {
    Project     = "helicone"
    Environment = "dev"
    ManagedBy   = "terraform"
  }
}

variable "create_knowledge_base" {
  description = "Whether to create a Bedrock Knowledge Base"
  type        = bool
  default     = false
}

variable "knowledge_base_name" {
  description = "Name of the Bedrock Knowledge Base"
  type        = string
  default     = "helicone-knowledge-base"
}

variable "knowledge_base_description" {
  description = "Description of the Bedrock Knowledge Base"
  type        = string
  default     = "Helicone Knowledge Base for document retrieval"
}

variable "create_s3_bucket" {
  description = "Whether to create an S3 bucket for Bedrock artifacts"
  type        = bool
  default     = false
}

variable "bedrock_models" {
  description = "List of Bedrock models available in your AWS account"
  type        = list(string)
  default = [
    # Latest Claude Models
    "anthropic.claude-3-7-sonnet-20250219-v1:0",      # Claude 3.7 Sonnet
    "anthropic.claude-sonnet-4-20250514-v1:0",        # Claude Sonnet 4
    "anthropic.claude-opus-4-20250514-v1:0",          # Claude Opus 4
    "anthropic.claude-3-5-sonnet-20241022-v2:0",      # Claude 3.5 Sonnet v2
    "anthropic.claude-3-5-sonnet-20240620-v1:0",      # Claude 3.5 Sonnet v1
    "anthropic.claude-3-5-haiku-20241022-v1:0",       # Claude 3.5 Haiku
    "anthropic.claude-3-haiku-20240307-v1:0",         # Claude 3 Haiku
    "anthropic.claude-3-opus-20240229-v1:0",          # Claude 3 Opus
    
    # Amazon Nova Models
    "amazon.nova-premier-v1:0",                       # Nova Premier
    "amazon.nova-pro-v1:0",                           # Nova Pro
    "amazon.nova-lite-v1:0",                          # Nova Lite
    "amazon.nova-micro-v1:0",                         # Nova Micro
    
    # Amazon Titan Models
    "amazon.titan-text-premier-v1:0",                 # Titan Text Premier
    "amazon.titan-embed-text-v2:0",                   # Latest embedding model
    "amazon.titan-embed-text-v1",                     # Classic embedding model
    "amazon.titan-image-generator-v2:0",              # Latest image generation
    
    # Meta Llama 4 Models
    "meta.llama4-scout-17b-instruct-v1:0",           # Llama 4 Scout
    "meta.llama4-maverick-17b-instruct-v1:0",        # Llama 4 Maverick
    "meta.llama3-3-70b-instruct-v1:0",               # Llama 3.3 70B
    "meta.llama3-2-90b-instruct-v1:0",               # Llama 3.2 90B
    
    # Other High-Performance Models
    "mistral.pixtral-large-2502-v1:0",               # Mistral multimodal
    "mistral.mistral-large-2402-v1:0",               # Mistral Large
    "cohere.command-r-plus-v1:0",                    # Cohere Command R+
    "deepseek.r1-v1:0"                               # DeepSeek R1
  ]
}