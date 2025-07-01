# Redis Caching Performance

## Setup

The hardware and infrastructure setup for this test is the same as described in the 
[System Specifications](/benchmarks/README.md#system-specifications)
section of the `README`. The only difference is the configuration for the AI
gateway, which should now include the following:

```
cache-store:
  type: "redis"
  host-url: "redis://helicone-redis-patient-feather-2439.internal:6379"

global:
  cache:
    buckets: 1
    directive: "max-age=3200"
```

## Results

#### Cache Miss

The following shows the tracing for a cache miss:

![cache miss](/benchmarks/imgs/redis-cache/cache-miss.png)

Even with Redis enabled, the latency of the Gateway during a cache
miss is: 

`86.33ms (total request time) - 83.5ms (provider time) = 2.83ms`

#### Cache Hit

The following shows the tracing for a cache hit:

![cache miss](/benchmarks/imgs/redis-cache/cache-hit.png)

On a cache hit, the response time is now `1.62ms`!
