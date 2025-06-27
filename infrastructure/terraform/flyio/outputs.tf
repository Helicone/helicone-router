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

output "ai_gateway_machines" {
  description = "AI Gateway machine information"
  value = {
    for idx, machine in fly_machine.ai_gateway : idx => {
      id     = machine.id
      name   = machine.name
      region = machine.region
    }
  }
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

output "infrastructure_machines" {
  description = "Infrastructure machine information"
  value = {
    for app_name, machine in fly_machine.infrastructure_machines : app_name => {
      id     = machine.id
      name   = machine.name
      region = machine.region
    }
  }
}

# Volumes output removed - volumes need to be managed manually
# due to deprecated GraphQL API in Terraform provider

# Application apps outputs
output "application_apps" {
  description = "Application service information"
  value = {
    for app_name, app in fly_app.application_apps : app_name => {
      name     = app.name
      id       = app.id
      hostname = "${app.name}.fly.dev"
    }
  }
}

output "application_machines" {
  description = "Application machine information"
  value = {
    for app_name, machine in fly_machine.application_machines : app_name => {
      id     = machine.id
      name   = machine.name
      region = machine.region
    }
  }
}

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
      }
    },
    {
      for app_name, app in fly_app.infrastructure_apps : app_name => {
        name     = app.name
        id       = app.id
        hostname = "${app.name}.fly.dev"
        type     = "infrastructure"
      }
    },
    {
      for app_name, app in fly_app.application_apps : app_name => {
        name     = app.name
        id       = app.id
        hostname = "${app.name}.fly.dev"
        type     = "application"
      }
    }
  )
}

output "application_urls" {
  description = "URLs for all applications with public access"
  value = {
    ai_gateway = "https://${fly_app.ai_gateway.name}.fly.dev"
    grafana    = "https://helicone-grafana.fly.dev"
    applications = {
      for app_name, app in fly_app.application_apps : app_name => "https://${app.name}.fly.dev"
    }
  }
} 