# AI Gateway outputs
output "ai_gateway_app_name" {
  description = "Name of the AI Gateway application"
  value       = fly_app.ai_gateway.name
}

output "ai_gateway_app_id" {
  description = "ID of the AI Gateway application"
  value       = fly_app.ai_gateway.id
}

output "ai_gateway_hostname" {
  description = "Hostname of the AI Gateway application"
  value       = "${fly_app.ai_gateway.name}.fly.dev"
}

# Infrastructure applications outputs
output "infrastructure_apps" {
  description = "Infrastructure application information"
  value = {
    for app_name, app in fly_app.infrastructure_apps : app_name => {
      name     = app.name
      id       = app.id
      hostname = "${app.name}.fly.dev"
    }
  }
}

# Infrastructure machines are managed by deploy.sh using fly.toml files
# Machine information can be retrieved using: flyctl machines list --app <app-name>

# Volumes output removed - volumes need to be managed manually
# due to deprecated GraphQL API in Terraform provider

# Summary outputs
output "all_applications" {
  description = "Complete list of all applications managed by this module"
  value = merge(
    {
      "ai-gateway" = {
        name     = fly_app.ai_gateway.name
        id       = fly_app.ai_gateway.id
        hostname = "${fly_app.ai_gateway.name}.fly.dev"
        type     = "main"
        machines = {
          standard = var.ai_gateway_instances
          performance = 1
        }
      }
    },
    {
      for app_name, app in fly_app.infrastructure_apps : app_name => {
        name     = app.name
        id       = app.id
        hostname = "${app.name}.fly.dev"
        type     = "infrastructure"
        note     = "Machines managed by deploy.sh"
      }
    }
  )
}

output "application_urls" {
  description = "URLs for all applications with public access"
  value = {
    ai_gateway = "https://${fly_app.ai_gateway.name}.fly.dev"
    grafana    = "https://helicone-grafana.fly.dev"
  }
} 