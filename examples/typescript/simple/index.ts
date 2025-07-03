import OpenAI from "openai";
import path from "path";

async function main() {
  const envPath = path.resolve(__dirname, "../../../.env");

  require("dotenv").config({
    path: envPath,
  });
  // if the Gateway has authentication enabled, you must set the API key via an environment variable
  const apiKey = process.env.HELICONE_CONTROL_PLANE_API_KEY || "fake-api-key";
  const client = new OpenAI({
    baseURL: "http://localhost:8080/ai",
    // Required by OpenAI SDK, but gateway handles real auth
    apiKey,
  });

  const response = await client.chat.completions.create({
    // 100+ models available
    model: "openai/gpt-4o-mini",
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
  });

  // for await (const chunk of response) {
  //   console.log(chunk.choices[0].delta.content);
  // }
  console.log(response.choices[0].message.content);
}

main().catch(console.error);
