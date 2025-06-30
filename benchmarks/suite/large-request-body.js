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
      "content": "Hi! Can you tell me who Ferris the crab is?"
    },
    {
      "role": "assistant",
      "content": "Of course! Ferris the crab is the official, friendly mascot of the Rust programming language. Ferris is often depicted as a cheerful, red, cartoon crab."
    },
    {
      "role": "user",
      "content": "Why was a crab chosen as Rust's mascot?"
    },
    {
      "role": "assistant",
      "content": "The crab was chosen as Rust's mascot because the word 'ferric' means 'containing iron,' and Rust is a language named after iron oxide. Ferris is a play on 'ferric,' and crabs have a hard shell, symbolizing safety and protection, much like Rust's focus on safety."
    },
    {
      "role": "user",
      "content": "What kind of personality does Ferris have?"
    },
    {
      "role": "assistant",
      "content": "Ferris is known for being friendly, curious, and always eager to help others learn Rust. Ferris loves to encourage newcomers and make programming fun!"
    },
    {
      "role": "user",
      "content": "What are Ferris's favorite activities?"
    },
    {
      "role": "assistant",
      "content": "Ferris enjoys exploring new code, helping people debug their programs, and attending Rust conferences and meetups. Ferris also loves collecting sea shells and learning new things."
    },
    {
      "role": "user",
      "content": "Can you share a fun fact about Ferris?"
    },
    {
      "role": "assistant",
      "content": "Absolutely! Ferris is a beloved symbol in the Rust community, and you can find Ferris plushies, stickers, and even Ferris-themed code examples! Ferris often appears in Rust learning materials to make them more approachable."
    },
    {
      "role": "user",
      "content": "Does Ferris know anything about the Rust Nomicon?"
    },
    {
      "role": "assistant",
      "content": "Yes! The Rust Nomicon is an infamous book in the Rust community, delving into the dark arts of unsafe Rust. While Ferris usually helps with safe Rust, they sometimes peek into the Nomicon to understand the mysterious and powerful features of the language. The Nomicon is full of warnings and arcane knowledge, and Ferris always reminds users to be careful when venturing there!"
    },
    {
      "role": "user",
      "content": "Has Ferris appeared in any interesting places?"
    },
    {
      "role": "assistant",
      "content": "Yes! Ferris has appeared at many Rust conferences, in online tutorials, and even in some creative art projects by the community. Ferris is always ready for a new adventure!"
    },
    {
      "role": "user",
      "content": "What is Ferris's favorite feature in Rust?"
    },
    {
      "role": "assistant",
      "content": "Ferris loves Rust's ownership system! It helps keep programs safe and efficient by making sure memory is managed correctly without needing a garbage collector. Ferris thinks it's one of the coolest things about Rust."
    },
    {
      "role": "user",
      "content": "Do you have any advice for someone new to Rust?"
    },
    {
      "role": "assistant",
      "content": "Absolutely! Ferris always says: don't be afraid of the compiler. The Rust compiler is strict, but it's like a helpful friend guiding you to write better code. Take your time, read the error messages, and ask the community for help!"
    },
    {
      "role": "user",
      "content": "Can Ferris tell a Rust-themed joke?"
    },
    {
      "role": "assistant",
      "content": "Sure! Why do Rustaceans never get lost? Because they always borrow the right path!"
    },
    {
      "role": "user",
      "content": "What does Ferris think about Rust's borrow checker?"
    },
    {
      "role": "assistant",
      "content": "Ferris thinks the borrow checker is a trusty sidekick! It can be tricky at first, but once you get used to it, it helps you write safe and reliable code. Ferris encourages everyone to embrace it!"
    },
    {
      "role": "user",
      "content": "Thanks, Ferris! Any final words?"
    },
    {
      "role": "assistant",
      "content": "You're welcome! Ferris wishes you happy coding and lots of fun exploring Rust. Remember, the Rustacean community is always here to help!"
    }
  ],
  max_tokens: 10000,
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
