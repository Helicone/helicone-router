# BENCHMARKS

## Overview

This document contains performance benchmarks for the Helicone AI Gateway using k6 load testing. These benchmarks measure the gateway's ability to handle high-throughput AI inference requests.

Additional benchmarks regarding caching latency with Redis are available in
[cache.md](/benchmarks/cache.md)

**Test Date**: June 30, 2025

## System Specifications

**Mock Server:**

- Used to mock the LLM providers to allow for sustained load testing while
  avoiding costly bills
- Each provider is configured to have a median latency of `60ms`
- Hardware: Flyio's `performance-2x w/4GB RAM` server with AMD EYPC CPU
  - [Flyio app configuration](/infrastructure/mock-server/fly.toml)

**AI Gateway:**

- The service under test
- Hardware: Flyio's `performance-2x w/4GB RAM` server with AMD EYPC CPU
  - [Flyio app configuration](/infrastructure/ai-gateway/fly.toml)
  - **NOTE**: Be sure to update the `raw_values` in the `[[files]]`
    section that contains the config. The updated value should be copy+
    pasted from [load-test.yaml](/ai-gateway/config/load-test.yaml)
  - Also be sure to update the `[[vm]]` section to use the right
    machine as specified above.

**OpenTelemetry Stack:**
Each of the following were deployed to `shared-1x w/512MB RAM` servers
in order to view metrics using the provided [Grafana dashboard](/infrastructure/grafana/dashboards/ai-gateway.json):
- [grafana](/infrastructure/grafana/fly.toml)
- [loki](/infrastructure/loki/fly.toml)
- [opentelemetry-collector](/infrastructure/opentelemetry-collector/fly.toml)
- [prometheus](/infrastructure/prometheus/fly.toml)
- [tempo](/infrastructure/tempo/fly.toml)
   - Required `2GB` of memory to avoid OOM errors

## Running Instructions

### Prerequisites

Install [k6](https://k6.io):

```bash
# Install k6 (macOS)
brew install k6

# Install k6 (Ubuntu)
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install k6
```

### Deploy the Stack

As mentioned in the [reproducibility section](#reproducibility), the system under test should
be deployed to their own isolated machines. 

You may use the provided flyio app configurations to easily deploy the entire stack using
the [flyctl](https://fly.io/docs/flyctl/) cli.

After you've deployed, you may want to import the provided [Grafana dashboard](/infrastructure/grafana/dashboards/ai-gateway.json).

### Execute Tests

**Note:** Make sure the URL for the AI Gateway in `test.js` matches
where you've deployed the service!

```bash
# Run the benchmark
k6 run suite/test.js

# Monitor system resources (optional)
htop  # During test execution
```

----

## Sustained Load Test

- Gateway configuration:
  - Authenticated enabled, logging disabled

Two clients, both running `test.js` with a `rate` of `1500`:

**Target**: 3000 RPS | **Duration**: 2min

| Metric | Value |
|--------|-------|
| **Achieved RPS** | `3,000` |
| **Total Requests** | `539,630` |
| **Success Rate** | `100%` (`3` failures out of `539,630` requests) |
| **AI Gateway Overhead** | `<1ms` |
| **Avg Response Time** | `77.58ms` |
| **90th Percentile Total Request Time** | `85.49ms` |
| **95th Percentile Total Request Time** | `89.16ms` |
| **Max Response Time** | `504.8ms` |

Full `k6` output is available in [sustained-load-results.md](/benchmarks/0.2.0-beta.14/sustained-load-results.md)


### Requests Per Second

A chart showing the sustained 3k req/s request rate can be seen here:

![req-rate](/benchmarks/imgs/sustained-load/req-rate.png)


### Latency added by AI Gateway

In order to find the latency added by the gatency, we will inspect the timing information
provided by our traces:

1. Trace 1:

![trace-1](/benchmarks/imgs/sustained-load/trace-1.png)

2. Trace 2:

![trace-2](/benchmarks/imgs/sustained-load/trace-2.png)

By subtracting the time waiting for the provider from the total request time,
we can calculate the overhead in latency added by the AI Gateway itself:


| trace # | total request time | provider time | gateway overhead |
|-------|-------|--------|-------|
| 1 | `64.81ms` | `64.27ms` | `64.81ms - 64.27ms = 0.54ms` |
| 2 | `75.8ms` | `75.49ms` | `75.8ms - 75.49ms = 0.31ms` |


These are not cherry-picked results, just a random sampling. If you have suggestions
on how to make more robust queries in Tempo, please file an issue!


### Total Latency

A graph of the P95, P99, and P99.9 latency can be seen below:


![latency](/benchmarks/imgs/sustained-load/latency.png)

Note that this is using a mock API server for the LLM provider,
real world usage will see much higher latency due to provider inference.


### CPU Usage


You can see that this test was able to nearly saturate the CPU, getting to sustained ~80% usage:

![cpu-usage](/benchmarks/imgs/sustained-load/cpu.png)


### Memory Usage

Memory usage stayed below 100MB throughout the test despite the heavy traffic load:

![memory-usage](/benchmarks/imgs/sustained-load/memory.png)

Combined view:

![cpu-and-mem](/benchmarks/imgs/sustained-load/cpu-and-mem.png)


### Performance Analysis

1. **Optimal Throughput**: ~3,000 RPS sustained over 3 minutes
2. **Zero Error Rate**: Gateway handles overload gracefully without failures
3. **Stable memory usage**: Memory usage stays under `100MB` during entire sustained load test.
3. **Response Time Consistency**: 95th percentile typically under 80ms
4. **Graceful Degradation**: High load results in queuing rather than failures

-----

## Large Request Body Size Test Results


This `k6` suite can be found [here](/benchmarks/suite/large-request-body.js), and is 
designed to measure performance with large request body sizes for a more real world
environment.

- Gateway configuration:
  - Authenticated enabled, logging disabled

This test is executed with one client running `large-request-body.js` with a `rate` of `1500`:

**Target**: 1500 RPS | **Duration**: 2min

| Metric | Value |
|--------|-------|
| **Achieved RPS** | `1,500` |
| **Total Requests** | `269,777` |
| **Success Rate** | `100%` (`0` failures out of `269,777` requests) |
| **AI Gateway Overhead** | `<1ms` |
| **Avg Response Time** | `79.68ms` |
| **90th Percentile Total Request Time** | `87.66ms` |
| **95th Percentile Total Request Time** | `89.15ms` |
| **Max Response Time** | `902.44ms` |

Full `k6` output is available in [large-body-results.md](/benchmarks/0.2.0-beta.14/large-body-results.md)


### Requests Per Second

A chart showing the sustained 3k req/s request rate can be seen here:

![req-rate](/benchmarks/imgs/large-body/req-rate.png)


### Latency added by AI Gateway

In order to find the latency added by the gatency, we will inspect the timing information
provided by our traces:

1. Trace 1:

![trace-1](/benchmarks/imgs/large-body/trace-1.png)

2. Trace 2:

![trace-2](/benchmarks/imgs/large-body/trace-2.png)

By subtracting the time waiting for the provider from the total request time,
we can calculate the overhead in latency added by the AI Gateway itself:


| trace # | total request time | provider time | gateway overhead |
|-------|-------|--------|-------|
| 1 | `73.75ms` | `73.17ms` | `73.75ms - 73.17ms = 0.58ms` |
| 2 | `66.04ms` | `65.83ms` | `66.04ms - 65.83ms = 0.21ms` |


These are not cherry-picked results, just a random sampling. If you have suggestions
on how to make more robust queries in Tempo, please file an issue!


### Total Latency

A graph of the P95, P99, and P99.9 latency can be seen below:


![latency](/benchmarks/imgs/large-body/latency.png)

Note that this is using a mock API server for the LLM provider,
real world usage will see much higher latency due to provider inference.


### CPU Usage


You can see that the AI Gateway had some headroom as it only reached ~60% CPU usage.

![cpu-usage](/benchmarks/imgs/large-body/cpu.png)


### Memory Usage

Memory usage stayed below 65MB throughout the test despite large request body sizes,
thanks to the Gateway's intelligent stream processing:

![memory-usage](/benchmarks/imgs/large-body/memory.png)


### Performance Analysis

1. **Optimal Throughput**: ~1,500 RPS sustained over 3 minutes for large request bodies
2. **Zero Error Rate**: Gateway handles overload gracefully without failures
3. **Stable memory usage**: Memory usage stays under `100MB` during entire sustained load test.
3. **Response Time Consistency**: 95th percentile typically under 80ms
4. **Graceful Degradation**: High load results in queuing rather than failures


## Methodology Notes

### Reproducibility
All tests are executed using:
- The provided `k6` test suite: `test.js`
- The provided `fly.toml` files with the [aforementioned](#system-specifications) machine selection
- The mock server, AI Gateway, and OpenTelemetry stack should all run on their own isolated machines

