terraform {
  required_version = ">= 1.0"
    cloud { 
    
    organization = "helicone" 

    workspaces { 
      name = "helicone-flyio" 
    } 
  }

  required_providers {
    fly = {
      source  = "fly-apps/fly"
      version = "~> 0.0.23"
    }
  }
}

provider "fly" {
  fly_api_token = var.fly_api_token
}

# Main AI Gateway Application
resource "fly_app" "ai_gateway" {
  name = var.ai_gateway_app_name
  org  = var.fly_org
}

resource "fly_machine" "ai_gateway" {
  count  = var.ai_gateway_instances
  app    = fly_app.ai_gateway.name
  region = var.primary_region
  name   = "${var.ai_gateway_app_name}-${count.index + 1}"
  image  = var.ai_gateway_image

  services = [
    {
      ports = [
        {
          port     = 443
          handlers = ["tls", "http"]
        },
        {
          port     = 80
          handlers = ["http"]
        }
      ]
      protocol      = "tcp"
      internal_port = 8080
    }
  ]

  env = var.ai_gateway_env_vars
}

# High-performance AI Gateway machine with Performance 4x and 8GB memory
resource "fly_machine" "ai_gateway_performance" {
  app      = fly_app.ai_gateway.name
  region   = var.primary_region
  name     = "${var.ai_gateway_app_name}-performance"
  image    = var.ai_gateway_image
  cpus     = 4
  memorymb = 8192
  cputype  = "performance"

  services = [
    {
      ports = [
        {
          port     = 443
          handlers = ["tls", "http"]
        },
        {
          port     = 80
          handlers = ["http"]
        }
      ]
      protocol      = "tcp"
      internal_port = 8080
    }
  ]

  env = var.ai_gateway_env_vars
}

# Infrastructure Applications
resource "fly_app" "infrastructure_apps" {
  for_each = var.infrastructure_apps
  
  name = "helicone-${each.key}"
  org  = var.fly_org
}

# Note: Volumes are created separately via flyctl or Fly.io console
# The GraphQL API for volumes is deprecated and not working with Terraform

resource "fly_machine" "infrastructure_machines" {
  for_each = var.infrastructure_apps
  
  app    = fly_app.infrastructure_apps[each.key].name
  region = var.primary_region
  name   = "${fly_app.infrastructure_apps[each.key].name}-1"
  image  = each.value.image

  services = try(each.value.services, [])
  
  # Mounts removed - volumes need to be created manually via flyctl
  # due to deprecated GraphQL API in Terraform provider

  # Environment variables
  env = try(each.value.env, {})
}

 