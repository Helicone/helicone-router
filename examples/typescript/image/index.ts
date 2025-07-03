import OpenAI from "openai";
import * as fs from "fs";
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

  // Read the image file
  const imagePath = "./image.png";
  const imageBuffer = fs.readFileSync(imagePath);
  const base64Image = imageBuffer.toString("base64");

  const response = await client.chat.completions.create({
    // model: "openai/gpt-4o-mini",
    model: "anthropic/claude-sonnet-4-0",
    messages: [
      {
        role: "user",
        content: [
          { type: "text", text: "What's in this image?" },
          {
            type: "image_url",
            image_url: {
              url: `data:image/png;base64,${base64Image}`,
            },
          },
        ],
      },
    ],
    max_tokens: 300,
  });

  console.log("Image description:", response.choices[0].message.content);
}

main().catch(console.error);
