# ECS Certificate Setup Guide

## Current Status
The ECS configuration is currently set up to use HTTP (port 80) because there is no ACM certificate for `helicone.ai` in the `us-east-1` region.

## To Enable HTTPS

You have several options to enable HTTPS for your ECS load balancer:

### Option 1: Create Certificate in us-east-1 (Recommended)
Deploy your route53-acm configuration in us-east-1:

```bash
cd /Users/devinat1/Desktop/helicone-projs/helicone-helm-v3/terraform/route53-acm
terraform apply -var="region=us-east-1"
```

Then update the ECS configuration to use the remote state (uncomment and modify the HTTPS listener in `ecs.tf`).

### Option 2: Create Certificate Locally in ECS Module
Add this to your `ecs.tf` file:

```terraform
# Create ACM certificate in us-east-1 for the load balancer
resource "aws_acm_certificate" "ecs_cert" {
  domain_name               = "helicone.ai"
  subject_alternative_names = ["*.helicone.ai"]
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = {
    Name = "helicone-ai-ecs-${var.environment}"
  }
}
```

Note: You'll need to validate this certificate by adding DNS records to your domain.

### Option 3: Import Existing Certificate
If you have an existing certificate, import it:

```bash
aws acm import-certificate \
  --certificate file://Certificate.pem \
  --certificate-chain file://CertificateChain.pem \
  --private-key file://PrivateKey.pem \
  --region us-east-1
```

## Enabling HTTPS Listener

Once you have a certificate ARN in us-east-1, uncomment and update the HTTPS listener in `ecs.tf`:

```terraform
resource "aws_lb_listener" "https_listener" {
  load_balancer_arn = aws_lb.fargate_lb.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-2016-08"
  certificate_arn   = "YOUR_CERTIFICATE_ARN_HERE"  # Update this

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.fargate_tg.arn
  }
}
```

Also update:
1. The ECS service dependency back to `depends_on = [aws_lb_listener.https_listener]`
2. The target group port back to 443 if needed

## Important Notes
- ACM certificates must be in the same region as the load balancer
- For CloudFront distributions, certificates must be in us-east-1
- DNS validation is recommended for ACM certificates managed by Terraform 