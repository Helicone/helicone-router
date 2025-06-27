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

# Infrastructure Applications - Apps only (machines managed by deploy.sh)
resource "fly_app" "infrastructure_apps" {
  for_each = var.infrastructure_apps
  
  name = "helicone-${each.value}"
  org  = var.fly_org
}

# Infrastructure apps only - machines are deployed via deploy.sh using fly.toml files
# This avoids duplication and ensures fly.toml files remain the source of truth

 