# Mock Server Flyio Configuration

This directory is used in the performance benchmarking documented 
in this [README](/benchmarks/README.md).


## Load Balancer Configuration

The following snippet is useful to set in the flyio toml in order
to test load balancing:

```toml

[experimental]
  cmd = ['/usr/local/bin/mock-server', '-c', '/app/config/config.yaml']

[[files]]
  guest_path = "/app/config/config.yaml"
  raw_value = '''

openai-latency: 90
anthropic-latency: 120
gemini-latency: 160
'''

```