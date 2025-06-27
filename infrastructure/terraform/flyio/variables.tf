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

# Infrastructure Applications Configuration
variable "infrastructure_apps" {
  description = "Configuration for infrastructure applications (grafana, loki, tempo, redis, otel-collector)"
  type = map(object({
    image = string
    services = optional(list(object({
      protocol      = string
      internal_port = number
      ports = list(object({
        port     = number
        handlers = list(string)
      }))
    })))
    volumes = optional(list(object({
      path    = string
      size_gb = optional(number)
    })))
    env = optional(map(string))
  }))
  default = {
    grafana = {
      image = "grafana/grafana:11.2.0"
      services = [
        {
          protocol      = "tcp"
          internal_port = 3010
          ports = [
            {
              port     = 443
              handlers = ["tls", "http"]
            }
          ]
        }
      ]
      volumes = [
        {
          path    = "/var/lib/grafana"
          size_gb = 10
        }
      ]
    }
    loki = {
      image = "grafana/loki:3.0.0"
      volumes = [
        {
          path    = "/var/lib/loki"
          size_gb = 10
        }
      ]
    }
    tempo = {
      image = "grafana/tempo:2.5.0"
      volumes = [
        {
          path    = "/var/lib/tempo"
          size_gb = 10
        }
      ]
    }
    redis = {
      image = "flyio/redis:6.2.6"
      volumes = [
        {
          path    = "/data"
          size_gb = 10
        }
      ]
    }
    otel-collector = {
      image = "otel/opentelemetry-collector:0.108.0"
    }
  }
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