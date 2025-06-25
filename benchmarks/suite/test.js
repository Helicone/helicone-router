import http from 'k6/http';

export const options = {
  scenarios: {
    constant_rate: {
      executor: 'constant-arrival-rate',
      rate: 5000, // 1000 RPS
      timeUnit: '1s',
      duration: '3s',
      preAllocatedVUs: 300,
      maxVUs: 1000,
    },
  },
};

const payload = JSON.stringify({
  model: "openai/gpt-4o-mini",
  messages: [
    {
      role: "system",
      content: "hi"
    },
  ],
  max_tokens: 2,

});

const params = {
  headers: {
    'Content-Type': 'application/json',
  },
};

export default function () {
  http.post('http://localhost:8080/ai/chat/completions', payload, params);
}
