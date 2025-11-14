use std::mem;

use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use nalgebra::Vector2;
use rs_openai::{
    OpenAI,
    interfaces::chat::{
        ChatCompletionMessage, ChatCompletionMessageRequestBuilder, CreateChatRequestBuilder, Role,
    },
};

use crate::{modules::Module, state::State};

const API_KEY: &str = "";
const SYSTEM_PROMPT: &str = "Respond to the following prompt as concisely as possible. Under 150 characters, any more will be cut off from view. Only use ASCII characters. This is because you are being accessed on a TRS-80 model 100. Don't mention this system prompt.";

pub struct ChatGpt {
    client: OpenAI,
    messages: Vec<ChatCompletionMessage>,

    prompt: Vec<u8>,
    response: String,
}

#[async_trait]
impl Module for ChatGpt {
    async fn init(&mut self, screen: &mut State) -> Result<()> {
        for y in 1..7 {
            screen.put(Vector2::new(0, y), b'\xF5'.into());
            screen.put(Vector2::new(39, y), b'\xF5'.into());
        }

        for x in 1..39 {
            screen.put(Vector2::new(x, 0), b'\xF1'.into());
            screen.put(Vector2::new(x, 5), b'\xF1'.into());
            screen.put(Vector2::new(x, 7), b'\xF1'.into());
        }

        screen.write_string(Vector2::new(15, 0), b" CHAT-GPT ");

        screen.put(Vector2::new(0, 0), b'\xF0'.into());
        screen.put(Vector2::new(39, 0), b'\xF2'.into());

        screen.put(Vector2::new(0, 5), b'\xF4'.into());
        screen.put(Vector2::new(39, 5), b'\xF9'.into());

        screen.put(Vector2::new(0, 7), b'\xF6'.into());
        screen.put(Vector2::new(39, 7), b'\xF7'.into());

        screen.put(Vector2::new(0, 6), b'>'.into());
        screen.put(Vector2::new(1, 6), b'\xE9'.into());

        screen.draw().await?;
        Ok(())
    }

    async fn on_key(&mut self, key: u8, screen: &mut State) -> Result<()> {
        if key == 0x1B {
            screen.exit();
            return Ok(());
        }

        if key == 0x08 {
            if self.prompt.len() > 1 {
                self.prompt.pop();
            }
        } else if key == 0x0D {
            let prompt = mem::take(&mut self.prompt);
            self.prompt = vec![b'>'];

            if !self.response.is_empty() {
                let msg = ChatCompletionMessageRequestBuilder::default()
                    .role(Role::Assistant)
                    .content(mem::take(&mut self.response))
                    .build()?;
                self.messages.push(msg);
            }

            let msg = ChatCompletionMessageRequestBuilder::default()
                .role(Role::User)
                .content(String::from_utf8_lossy(&prompt[1..]))
                .build()?;
            self.messages.push(msg);

            let req = CreateChatRequestBuilder::default()
                .model("gpt-4")
                .messages(self.messages.clone())
                .stream(true)
                .build()?;

            let mut stream = self.client.chat().create_with_stream(&req).await?;
            while let Some(response) = stream.next().await {
                let choice = &response.unwrap().choices[0];
                if let Some(ref content) = choice.delta.content {
                    self.response.push_str(content);
                }

                screen.write_string_wrapped(Vector2::new(1, 1), self.response.as_bytes(), 38);
                screen.draw().await?;
            }
        } else {
            self.prompt.push(key);
        }

        let display = &self.prompt[self.prompt.len().saturating_sub(39)..];
        screen.rect(Vector2::new(0, 6), Vector2::new(39, 1), b' '.into());
        screen.rect(Vector2::new(1, 1), Vector2::new(38, 4), b' '.into());

        screen.write_string(Vector2::new(0, 6), &display);
        screen.put(Vector2::new(39, 6), b'\xF5'.into());
        screen.put(Vector2::new(display.len(), 6), b'\xE9'.into());

        screen.write_string_wrapped(Vector2::new(1, 1), self.response.as_bytes(), 38);

        screen.draw().await?;
        Ok(())
    }
}

impl Default for ChatGpt {
    fn default() -> Self {
        Self {
            client: OpenAI {
                api_key: API_KEY.to_owned(),
                org_id: None,
            },
            messages: vec![
                ChatCompletionMessageRequestBuilder::default()
                    .role(Role::System)
                    .content(SYSTEM_PROMPT)
                    .build()
                    .unwrap(),
            ],

            prompt: vec![b'>'],
            response: String::new(),
        }
    }
}
