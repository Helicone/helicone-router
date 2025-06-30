import http from 'k6/http';

export const options = {
  scenarios: {
    constant_rate: {
      executor: 'constant-arrival-rate',
      rate: 1500,
      timeUnit: '1s',
      duration: '3m',
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
};

const payload = JSON.stringify({
  model: "openai/gpt-4o-mini",
  messages: [
    {
        "role": "system",
        "content": "You are a helpful assistant that can answer questions and help with tasks."
    },
    {
        "role": "user",
        "content": "Hello, world!"
    }
  ],
  max_tokens: 1000,
});

const params = {
  headers: {
    'Content-Type': 'application/json',
    'Authorization': 'sk-helicone-test-key',
  },
};

export default function () {
  http.post('https://helicone-ai-gateway.fly.dev/ai/chat/completions', payload, params);
}
