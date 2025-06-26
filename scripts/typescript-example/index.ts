import OpenAI from "openai";

async function main() {
  const client = new OpenAI({
    baseURL: "http://localhost:8080/ai",
    // Required by SDK, but gateway handles real auth
    apiKey: "fake-api-key",
  });

  const response = await client.chat.completions.create({
    // 100+ models available
    model: "anthropic/claude-sonnet-4-0",
    messages: [
      {
          role: "system",
          content: "You are a helpful assistant that can answer questions and help with tasks."
      },
      {
          role: "user",
          content: "Hello, world!"
      }
    ],
    max_tokens: 400,
    stream: true,
  });

  for await (const chunk of response) {
    console.log(chunk.choices[0].delta.content);
  }
  // console.log(response.choices[0].message.content);
}

main().catch(console.error);
