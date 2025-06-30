# 🎮 Demo Guide

Here are instructions for running a demo of Helicone AI Gateway locally.

## Basic Setup
1. Set up your environment as described in the [Development Setup](DEVELOPMENT.md) section
   Make sure you've set the `HELICONE_CONTROL_PLANE_API_KEY`.
2. Run the router locally with OpenAI/Anthropic:
   ```bash
   cargo run -- -c ./ai-gateway/config/sidecar.yaml
   ```
3. Send a test request:
   ```bash
   cargo run -p test test-request -a
   ```
   You should see the request logged in your Helicone dashboard!
