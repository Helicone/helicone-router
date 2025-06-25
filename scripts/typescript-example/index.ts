import OpenAI from "openai";

async function main() {
  const client = new OpenAI({
    baseURL: "http://localhost:8080/ai",
    // Required by SDK, but gateway handles real auth
    apiKey: "fake-api-key",
  });

  const response = await client.chat.completions.create({
    // 100+ models available
    model: "openai/gpt-4o-mini",
    messages: [{ role: "user", content: "Hello, world!" }],
  });

  console.log(response.choices[0].message.content);
}

main().catch(console.error);
