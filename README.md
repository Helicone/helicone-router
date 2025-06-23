![Helicone AI Gateway](https://marketing-assets-helicone.s3.us-west-2.amazonaws.com/github-w%3Alogo.png)

# Helicone AI Gateway

[![GitHub stars](https://img.shields.io/github/stars/Helicone/aia-gateway?style=for-the-badge)](https://github.com/helicone/aia-gateway/)
[![Downloads](https://img.shields.io/github/downloads/Helicone/aia-gateway/total?style=for-the-badge)](https://github.com/helicone/aia-gateway/releases)
[![Docker pulls](https://img.shields.io/docker/pulls/helicone/ai-gateway?style=for-the-badge)](https://hub.docker.com/r/helicone/ai-gateway)
[![License](https://img.shields.io/badge/license-APACHE-green?style=for-the-badge)](LICENSE)

**The fastest, lightest, and most powerful AI Gateway on the market.**

*Built by the team at [Helicone](https://helicone.ai), open-sourced for the community.*

[🚀 Quick Start](#-deploy-with-docker-in-seconds) • [📖 Docs](https://docs.helicone.ai/ai-gateway) • [💬 Discord](https://discord.gg/7aSCGCGUeu) • [🌐 Website](https://helicone.ai)

---

## 🚀 One-Click Deploy to AWS ECS

Deploy Helicone Helicone AI Gateway to AWS ECS with a single click:

[![Deploy to AWS ECS](https://img.shields.io/badge/Deploy%20to-AWS%20ECS-FF9900?style=for-the-badge&logo=amazon-aws)](https://github.com/Helicone/helicone-router/actions/workflows/deploy-to-ecs.yml)

**Prerequisites:**
- AWS Account with appropriate permissions
- AWS IAM role configured for GitHub Actions (see [setup guide](#aws-setup))

Click the button above → **Run workflow** → Select your environment → **Deploy!**

---

## 👩🏻‍💻 Deploy with Docker in seconds

```bash
docker run -d --name helix \
  -p 8080:8080 \
  -e OPENAI_API_KEY=your_openai_key \
  -e ANTHROPIC_API_KEY=your_anthropic_key \
  helicone/helix:latest
```

2. Run locally in your terminal
```bash
npx @helicone/ai-gateway start
```

3. Make your requests using any OpenAI SDK:

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/production"
)

# Route to any LLM provider through the same interface, we handle the rest.
response = client.chat.completions.create(
    model="anthropic/claude-3-5-sonnet",  # Or openai/gpt-4o, gemini/gemini-2.5-pro, etc.
    messages=[{"role": "user", "content": "Hello from Helicone AI Gateway!"}]
)
```

**That's it.** No new SDKs to learn, no integrations to maintain. Fully-featured and open-sourced.

*-- For advanced config, check out our [configuration guide](https://docs.helicone.ai/ai-gateway/config) and the [providers we support](https://docs.helicone.ai/ai-gateway/providers).*

---

## Why Helicone AI Gateway?

<!-- TODO: include launch video here -->

#### 🌐 **Unified interface**
Request **any LLM provider** using familiar OpenAI syntax. Stop rewriting integrations—use one API for OpenAI, Anthropic, Google, AWS Bedrock, and [20+ more providers](https://docs.helicone.ai/ai-gateway/providers).

#### ⚡ **Smart provider selection**
**Load balance** to always hit the fastest, cheapest, or most reliable option. Built-in strategies include latency-based P2C + PeakEWMA, weighted distribution, and cost optimization. Always aware of provider uptime and rate limits.

#### 💰 **Control your spending**
**Rate limit** to prevent runaway costs and usage abuse. Set limits per user, team, or globally with support for request counts, token usage, and dollar amounts.

#### 🚀 **Improve performance**
**Cache responses** to reduce costs and latency by up to 95%. Supports Redis and S3 backends with intelligent cache invalidation.

#### 📊 **Simplified tracing**
Monitor performance and debug issues with built-in Helicone integration, plus OpenTelemetry support for **logs, metrics, and traces**.

#### ☁️ **One-click deployment**
Deploy in seconds to your own infrastructure by using our **Docker** or **binary** download following our [deployment guides](https://docs.helicone.ai/gateway/deployment).

---

## 🎥 Demo

<!-- TODO: Add demo GIF/video showing Helicone AI Gateway routing between providers -->

![Helicone AI Gateway Demo](https://via.placeholder.com/800x400/0ea5e9/ffffff?text=Helicone+AI+Gateway+Demo+%28Coming+Soon%29)

*Coming soon: Interactive demo showing real-time load balancing across providers*

---

## ⚡ Scalable for production

<!-- TODO: include correct metrics -->

| Metric | Helicone AI Gateway | Typical Setup | Improvement |
|--------|-------|---------------|-------------|
| **P95 Latency** | ~1-5ms | ~60-100ms | **10-100x faster** |
| **Memory Usage** | ~64MB | ~512MB | **8x lower** |
| **Requests/sec** | ~10,000 | ~1,000 | **10x throughput** |
| **Binary Size** | ~15MB | ~200MB | **13x smaller** |
| **Cold Start** | ~100ms | ~2s | **20x faster** |

---

## 🏗️ How it works

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Your App      │───▶│ Helicone AI     │───▶│  LLM Providers  │
│                 │    │ Gateway         │    │                 │
│ OpenAI SDK      │    │                 │    │ • OpenAI        │
│ (any language)  │    │ • Load Balance  │    │ • Anthropic     │
│                 │    │ • Rate Limit    │    │ • AWS Bedrock   │
│                 │    │ • Cache         │    │ • Google Vertex │
│                 │    │ • Trace         │    │ • 20+ more      │
│                 │    │ • Fallbacks     │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                               │
                               ▼
                      ┌─────────────────┐
                      │ Helicone        │
                      │ Observability   │
                      │                 │
                      │ • Dashboard     │
                      │ • Observability │
                      │ • Monitoring    │
                      │ • Debugging     │
                      └─────────────────┘
```

---

## ⚙️ Custom configuration

### Environment variables
Include your `PROVIDER_API_KEY`s in your `.env` file.

```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
HELICONE_API_KEY=sk-...
REDIS_URL=redis://localhost:6379
```

### Sample config file

# Run directly
./helix
```

### Option 3: Cargo (From Source)
```bash
cargo install --git https://github.com/Helicone/helicone-router.git ai-gateway
ai-gateway
```

### Option 4: Local Deploy Script
```bash
# Clone and deploy to AWS ECS
git clone https://github.com/Helicone/helicone-router.git
cd helicone-router
./infrastructure/deploy.sh
```

---

## 🔧 AWS Setup for One-Click Deploy

To use the one-click deploy button, configure AWS IAM for GitHub Actions:

### 1. Create OIDC Provider (if not exists)
```bash
aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --thumbprint-list 6938fd4d98bab03faadb97b34396831e3780aea1 \
  --client-id-list sts.amazonaws.com
```

### 2. Create IAM Role
```bash
# Replace YOUR_ACCOUNT_ID and YOUR_GITHUB_USERNAME
cat > github-actions-trust-policy.json << EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::YOUR_ACCOUNT_ID:oidc-provider/token.actions.githubusercontent.com"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "token.actions.githubusercontent.com:aud": "sts.amazonaws.com",
          "token.actions.githubusercontent.com:sub": "repo:YOUR_GITHUB_USERNAME/helicone-router:ref:refs/heads/main"
        }
      }
    }
  ]
}
EOF

aws iam create-role \
  --role-name GitHubActions-ECS-Deploy \
  --assume-role-policy-document file://github-actions-trust-policy.json
```

### 3. Attach Policies
```bash
aws iam attach-role-policy \
  --role-name GitHubActions-ECS-Deploy \
  --policy-arn arn:aws:iam::aws:policy/AmazonECS_FullAccess

aws iam attach-role-policy \
  --role-name GitHubActions-ECS-Deploy \
  --policy-arn arn:aws:iam::aws:policy/AmazonEC2ContainerRegistryFullAccess

aws iam attach-role-policy \
  --role-name GitHubActions-ECS-Deploy \
  --policy-arn arn:aws:iam::aws:policy/IAMFullAccess

aws iam attach-role-policy \
  --role-name GitHubActions-ECS-Deploy \
  --policy-arn arn:aws:iam::aws:policy/AmazonVPCFullAccess
```

### 4. Configure GitHub Secrets
In your GitHub repo: **Settings** → **Secrets and variables** → **Actions**

Add these secrets:
- `AWS_ROLE_ARN`: `arn:aws:iam::YOUR_ACCOUNT_ID:role/GitHubActions-ECS-Deploy`
- `AWS_ACCOUNT_ID`: Your AWS account ID
- `TERRAFORM_CLOUD_TOKEN`: Your Terraform Cloud token (if using Terraform Cloud)

---

## ⚙️ Configuration

### Environment variables (Simplest)
```bash
export OPENAI_API_KEY=sk-...
export ANTHROPIC_API_KEY=sk-ant-...
export REDIS_URL=redis://localhost:6379
```

### Configuration file
```yaml
# config.yaml
providers:
  - name: openai
    type: openai
    api_key: ${OPENAI_API_KEY}
    models: [gpt-4o, gpt-4o-mini, gpt-3.5-turbo]

  - name: anthropic
    type: anthropic
    api_key: ${ANTHROPIC_API_KEY}
    models: [claude-3-5-sonnet, claude-3-5-haiku]

  - name: bedrock
    type: bedrock
    region: us-east-1
    models: [anthropic.claude-3-5-sonnet-20241022-v2:0]

load_balancing:
  strategy: latency_based  # or weighted, cost_based

rate_limits:
  global:
    requests_per_minute: 1000
  per_user:
    requests_per_minute: 60

caching:
  backend: redis
  ttl: 3600  # 1 hour
```

Run with config:
```bash
helix --config helix.yaml
```

---

## 🌍 Supported Providers & Models

<!-- TODO: revise the correct models & providers supported -->

### Cloud Providers
| Provider | Models | Auth Method |
|----------|--------|-------------|
| **OpenAI** | GPT-4o, GPT-4o-mini, o1, o3-mini, embeddings | API Key |
| **Anthropic** | Claude 3.5 Sonnet/Haiku, Claude 3 Opus | API Key |
| **AWS Bedrock** | Claude, Nova, Titan, Llama | AWS Credentials |
| **Google Vertex** | Gemini Pro/Flash, PaLM, Claude | Service Account |
| **Azure OpenAI** | GPT models via Azure | API Key |
| **Mistral** | Mistral Large/Medium/Small | API Key |
| **Cohere** | Command R+, Embed | API Key |
| **Perplexity** | Sonar models | API Key |
| **Together** | Llama, Mixtral, Qwen | API Key |
| **Groq** | Llama, Mixtral, Gemma | API Key |

### Self-Hosted
| Provider | Models | Notes |
|----------|--------|-------|
| **Ollama** | Llama, Mistral, CodeLlama, etc. | Local deployment |
| **vLLM** | Any HuggingFace model | OpenAI-compatible |
| **OpenAI-compatible** | Custom endpoints | Generic support |

<!-- TODO: update to the correct provider list link -->

*See our [full provider list](https://docs.helicone.ai/helix/providers) for the complete matrix*

---

## 🎯 Production examples

### Docker Compose
```yaml
version: '3.8'
services:
  helix:
    image: helicone/helix:latest
    ports:
      - "8080:8080"
    environment:
      OPENAI_API_KEY: ${OPENAI_API_KEY}
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
      REDIS_URL: redis://redis:6379
    volumes:
      - ./helix.yaml:/app/helix.yaml
    depends_on:
      - redis
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  redis_data:
```

### Kubernetes Deployment
```yaml
apiVersion: apps/
kind: Deployment
metadata:
  name: helix
spec:
  replicas: 3
  selector:
    matchLabels:
      app: helix
  template:
    metadata:
      labels:
        app: helix
    spec:
      containers:
      - name: helix
        image: helicone/helix:latest
        ports:
        - containerPort: 8080
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-secrets
              key: openai
        - name: REDIS_URL
          value: redis://redis-service:6379
        resources:
          requests:
            memory: "64Mi"
            cpu: "50m"
          limits:
            memory: "128Mi"
            cpu: "200m"
---
apiVersion: v1
kind: Service
metadata:
  name: helicone-ai-gateway-service
spec:
  selector:
    app: ai-gateway
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

### Sidecar Pattern
```dockerfile
# Add to your existing application
FROM your-app:latest

# Install Helicone AI Gateway
COPY --from=helicone/helix:latest /usr/local/bin/helix /usr/local/bin/helix

# Start both services
CMD ["sh", "-c", "helix & your-app"]
```

---

## 🔧 Advanced Features

### Load Balancing Strategies

```yaml
providers: # Include their PROVIDER_API_KEY in .env file
  openai:
    models:
      - gpt-4
      - gpt-4o
      - gpt-4o-mini

  anthropic:
    version: "2023-06-01"
    models:
      - claude-3-opus
      - claude-3-sonnet

global: # Global settings for all routers
  cache:
    enabled: true
    directive: "max-age=3600, max-stale=1800"
    buckets: 10
    seed: "unique-cache-seed"

routers:
  production: # Per router configuration
    load-balance:
      chat:
        strategy: latency
        targets:
          - openai
          - anthropic
    retries:
      enabled: true
        max-retries: 3
        strategy: exponential
        base: 1s
        max: 30s
    rate-limit:
      global:
        store: in-memory
        per-api-key:
          capacity: 500
          refill-frequency: 1s
        cleanup-interval: 5m
    helicone: # Include your HELICONE_API_KEY in your .env file
      enable: true
    telemetry:
      level: "info,ai_gateway=trace"
```
### Run with your custom config file

```bash
npx @helicone/ai-gateway start --config config.yaml
```
---

## 📚 Migration guide

### From OpenAI
```diff
from openai import OpenAI

client = OpenAI(
-   api_key=os.getenv("OPENAI_API_KEY")
+   base_url="http://localhost:8080/production"
)

# No other changes needed!
response = client.chat.completions.create(
    model="gpt-4o",
    messages=[{"role": "user", "content": "Hello!"}]
)
```

### From LangChain
```diff
from langchain_openai import ChatOpenAI

llm = ChatOpenAI(
    model="gpt-4o",
-   api_key=os.getenv("OPENAI_API_KEY")
+   base_url="http://localhost:8080/"
)
```

### From multiple providers
```python
# Before: Managing multiple clients
openai_client = OpenAI(api_key=openai_key)
anthropic_client = Anthropic(api_key=anthropic_key)

# After: One client for everything
client = OpenAI(
    base_url="http://localhost:8080/production"
)

# Use any model through the same interface
gpt_response = client.chat.completions.create(model="gpt-4o", ...)
claude_response = client.chat.completions.create(model="claude-3-5-sonnet", ...)
```

---

## 💗 What they say about The Helicone AI Gateway

> *"The Helicone AI Gateway reduced our LLM integration complexity from 15 different SDKs to just one. We're now spending time building features instead of maintaining integrations."*
>
> — **Senior Engineer, Fortune 500 Company**

> *"The cost optimization alone saved us $50K/month. The unified observability is just a bonus."*
>
> — **CTO, AI Startup**

> *"We went from 200ms P95 latency to 50ms with smart caching and load balancing. Our users immediately noticed."*
>
> — **Staff Engineer, SaaS Platform**

*Want to be featured? [Share your story!](https://github.com/Helicone/aia-gateway/discussions)*

---

## 📚 Resources

<!-- TODO: include correct resources -->

### Documentation
- 📖 **[Full Documentation](https://docs.helicone.ai/ai-gateway)** - Complete guides and API reference
- 🚀 **[Quickstart Guide](https://docs.helicone.ai/ai-gateway/quickstart)** - Get up and running in 1 minute
- 🔬 **[Advanced Configurations](https://docs.helicone.ai/ai-gateway/config)** - Configuration reference & examples

### Community
- 💬 **[Discord Server](https://discord.gg/7aSCGCGUeu)** - Our community of passionate AI engineers
- 🐙 **[GitHub Discussions](https://github.com/helicone/ai-gateway/discussions)** - Q&A and feature requests
- 🐦 **[Twitter](https://twitter.com/helicone_ai)** - Latest updates and announcements
- 📧 **[Newsletter](https://helicone.ai/email-signup)** - Tips and tricks to deploying AI applications

### Support
- 🎫 **[Report bugs](https://github.com/helicone/ai-gateway/issues)**: Github issues
- 💼 **[Enterprise Support](https://cal.com/team/helicone/helicone-discovery)**: Book a discovery call with our team

---

## 📄 License

The Helicone AI Gateway is licensed under the [Apache License](LICENSE) - see the file for details.

---

**Made with ❤️ by [Helicone](https://helicone.ai).**

[Website](https://helicone.ai) • [Docs](https://docs.helicone.ai) • [Discord](https://discord.gg/7aSCGCGUeu) • [Twitter](https://twitter.com/helicone_ai)
