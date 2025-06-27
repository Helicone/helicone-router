variable "fly_api_token" {
  description = "Fly.io API token for authentication"
  type        = string
  sensitive   = true
}

variable "fly_org" {
  description = "Fly.io organization name"
  type        = string
  default     = "personal"
}

variable "primary_region" {
  description = "Primary region for all applications"
  type        = string
  default     = "sjc"
}

# AI Gateway Configuration
variable "ai_gateway_app_name" {
  description = "Name of the main AI Gateway application"
  type        = string
  default     = "helicone-ai-gateway"
}

variable "ai_gateway_instances" {
  description = "Number of instances for the AI Gateway"
  type        = number
  default     = 1
}

variable "ai_gateway_image" {
  description = "Docker image for the AI Gateway"
  type        = string
  default     = "helicone/ai-gateway:main"
}



variable "ai_gateway_env_vars" {
  description = "Environment variables for AI Gateway"
  type        = map(string)
  default     = {}
}

# Infrastructure Applications - App Names Only
# Machine configuration is handled by deploy.sh using fly.toml files
variable "infrastructure_apps" {
  description = "List of infrastructure application names to create (machines deployed via deploy.sh)"
  type        = set(string)
  default = [
    "grafana",
    "loki", 
    "tempo",
    "otel-collector",
    "prometheus"
  ]
}



# Common tags
variable "common_tags" {
  description = "Common tags to apply to all resources"
  type        = map(string)
  default = {
    Project     = "helicone-ai-gateway"
    Environment = "production"
    ManagedBy   = "terraform"
  }
} 