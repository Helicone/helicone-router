# Fly.io Terraform Module

This Terraform module creates and manages Fly.io applications for the Helicone AI Gateway project, including the main gateway application and supporting infrastructure services.

## Applications Created

This module creates the following applications:

### Main Application
- **helicone-ai-gateway**: The main AI gateway application

### Infrastructure Applications
- **helicone-grafana**: Monitoring and visualization dashboard
- **helicone-loki**: Log aggregation system
- **helicone-tempo**: Distributed tracing backend
- **helicone-redis**: In-memory data store and cache
- **helicone-otel-collector**: OpenTelemetry collector for observability

### Application Services
- **helicone-api**: API backend service
- **helicone-web**: Frontend web application
- **helicone-web-admin**: Administrative web interface

## Prerequisites

1. **Fly.io Account**: You need a Fly.io account and organization
2. **Fly.io API Token**: Generate an API token from your Fly.io dashboard
3. **Terraform**: Version >= 1.0
4. **Docker Images**: Ensure all referenced Docker images are available

## Quick Start

1. **Copy the example configuration:**
   ```bash
   cp terraform.tfvars.example terraform.tfvars
   ```

2. **Edit the configuration:**
   ```bash
   # Edit terraform.tfvars with your specific values
   vim terraform.tfvars
   ```

3. **Initialize Terraform:**
   ```bash
   terraform init
   ```

4. **Plan the deployment:**
   ```bash
   terraform plan
   ```

5. **Apply the configuration:**
   ```bash
   terraform apply
   ```

## Configuration

### Required Variables

- `fly_api_token`: Your Fly.io API token (sensitive)

### Optional Variables

- `fly_org`: Fly.io organization name (default: "personal")
- `primary_region`: Primary region for applications (default: "sjc")
- `ai_gateway_app_name`: Name of the main AI Gateway app (default: "helicone-ai-gateway")
- `ai_gateway_instances`: Number of AI Gateway instances (default: 1)
- `ai_gateway_image`: Docker image for AI Gateway (default: "helicone/ai-gateway:main")

### Customizing Applications

You can customize infrastructure and application services by overriding the default configurations:

```hcl
# Example: Customize Grafana configuration
infrastructure_apps = {
  grafana = {
    image = "grafana/grafana:11.2.0"
    vm = {
      cpus     = 2
      memory   = "1024mb"
      cpu_kind = "shared"
    }
    # Add custom environment variables
    env = {
      GF_SECURITY_ADMIN_PASSWORD = "your-password"
    }
  }
}

# Example: Customize API service
application_apps = {
  api = {
    image             = "your-registry/helicone-api:v1.2.3"
    internal_port     = 3000
    health_check_path = "/health"
    vm = {
      cpus     = 2
      memory   = "1024mb"
      cpu_kind = "shared"
    }
    env = {
      NODE_ENV = "production"
      DATABASE_URL = "postgresql://..."
    }
  }
}
```

## Output Values

The module provides several output values:

- `ai_gateway_app_name`: Name of the AI Gateway application
- `ai_gateway_hostname`: Hostname of the AI Gateway application
- `infrastructure_apps`: Information about infrastructure applications
- `application_apps`: Information about application services
- `all_applications`: Complete list of all managed applications
- `application_urls`: URLs for accessing applications

## Architecture

### Infrastructure Applications
All infrastructure applications use the "helicone-" prefix and are configured for:
- Persistent storage (where applicable)
- Health checks
- Internal networking
- Observability metrics

### Application Services
Application services (API, Web, Web Admin) are configured for:
- HTTP/HTTPS access
- Health checks
- Auto-scaling capabilities
- Environment variable management

### Networking
- All applications are deployed in the same region for low latency
- Infrastructure services use internal networking
- Public services are accessible via HTTPS

## Managing Resources

### Scaling Applications
To scale the AI Gateway:
```hcl
ai_gateway_instances = 3
```

### Updating Images
Update the image version in your terraform.tfvars:
```hcl
ai_gateway_image = "helicone/ai-gateway:v2.0.0"
```

### Adding New Applications
Extend the `application_apps` variable to add new services.

## Security Considerations

1. **API Token**: Store your Fly.io API token securely (use environment variables or secret management)
2. **Environment Variables**: Sensitive environment variables should be managed through Fly.io secrets
3. **Network Access**: Infrastructure services are internal-only by default
4. **Volume Encryption**: All persistent volumes are encrypted

## Troubleshooting

### Common Issues

1. **Authentication Error**:
   - Verify your Fly.io API token is correct
   - Check that your organization has sufficient permissions

2. **Image Pull Issues**:
   - Ensure Docker images exist and are accessible
   - Check image tags are correct

3. **Resource Limits**:
   - Verify your Fly.io account has sufficient resources
   - Check pricing plan limits

### Useful Commands

```bash
# Check application status
fly apps list

# View application logs
fly logs -a <app-name>

# SSH into a machine
fly ssh console -a <app-name>

# Check machine status
fly status -a <app-name>
```

## Cleanup

To destroy all resources:
```bash
terraform destroy
```

**Warning**: This will permanently delete all applications and their data.

## Support

For issues specific to this Terraform module, please check:
1. Terraform plan output for configuration errors
2. Fly.io documentation for platform-specific issues
3. Application logs for runtime issues

## Contributing

When modifying this module:
1. Update variable descriptions and defaults
2. Add new outputs for new resources
3. Update this README
4. Test changes in a development environment 