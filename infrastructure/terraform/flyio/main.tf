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

# Application-specific apps (API, Web, Web Admin)
resource "fly_app" "application_apps" {
  for_each = var.application_apps
  
  name = "helicone-${each.key}"
  org  = var.fly_org
}

resource "fly_machine" "application_machines" {
  for_each = var.application_apps
  
  app    = fly_app.application_apps[each.key].name
  region = var.primary_region
  name   = "${fly_app.application_apps[each.key].name}-1"
  image  = each.value.image

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
      internal_port = each.value.internal_port
    }
  ]

  env = try(each.value.env, {})
} 