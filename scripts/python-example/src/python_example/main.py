from openai import OpenAI

client = OpenAI(
    # Required by SDK, but AI gateway handles real auth
    base_url="http://localhost:8080/ai",
    api_key="fake-api-key"
)


def main():
    print("Hello, World!")

    completion = client.chat.completions.create(
        model="openai/gpt-4o-mini",  # 100+ models available
        messages=[
            {
                "role": "system",
                "content": "You are a helpful assistant that can answer questions and help with tasks."
            },
            {
                "role": "user",
                "content": "Hello, world!"
            }
        ],
        max_tokens=400,
        stream=True,
    )

    # print(response.choices[0].message.content)
    for chunk in completion:
        print(chunk)


if __name__ == "__main__":
    main()