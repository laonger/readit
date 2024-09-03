use tokio::{runtime::Handle, task};
use std::{iter::once, sync::Arc};

use arrow::array::{Array, Float32Builder, ArrayData};
use arrow_schema::{DataType, Field, Schema};
use arrow_array::{
    cast::AsArray,
    Float32Array,
};

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        CreateChatCompletionRequestArgs,
        CreateChatCompletionResponse,

        ChatCompletionResponseFormat,
        ChatCompletionResponseFormatType,

        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,

        CreateEmbeddingRequest,
        Embedding,
        EmbeddingInput,
        EncodingFormat
    },
    error::OpenAIError,
    
};

use serde_json;

use crate::{prompt_utils, structs};

use crate::env;

pub struct OpenAI {
    client: Client<OpenAIConfig>,
    dim: u32,
    chat_model: String,
    analyse_model: String,
    embedding_model: String,
}

impl OpenAI {
    pub fn new(env: &env::Env) -> Self {

        let config = OpenAIConfig::new()
            .with_api_key(env.config.openai_key.clone())
            .with_api_base(env.config.openai_base.clone())
        ;
        
        Self {
            client: Client::with_config(config),
            dim: env.config.dim as u32,
            chat_model: env.config.chat_model.clone(),
            analyse_model: env.config.analyse_model.clone(),
            embedding_model: env.config.embedding_model.clone(),
        }
    }
    

    pub async fn analyse_source(&self,
        code_string: String, programming_lang: String, language: String
    ) -> Result<structs::GPTResponse, OpenAIError> 
    {
        let prompt = prompt_utils::analyse_source_file_prompt(
            programming_lang, code_string, language
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.analyse_model)
            .stream(false)
            .response_format(
                ChatCompletionResponseFormat {
                    r#type: ChatCompletionResponseFormatType::JsonObject
                }
            )
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("As a professional programming expert, analyze the given source code file. Your goal is to thoroughly understand the content and purpose of the code. Your response should be in JSON format.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        //println!("{:?}", response);
        let _text = response.choices[0].clone().message.content.unwrap();
        let mut text = String::new();
        if _text.starts_with("```json\n"){
            text = _text.replace("```json\n", "").replace("```", "");
        } else {
            text = _text;
        }
        //println!("analyze: {:?}", text);
        let code_description: structs::GPTResponse = serde_json::from_str(&text).unwrap();
        Ok(code_description)
    }

    pub async fn split_source(&self,
        code_string: String, programming_lang: String, language: String
    ) -> Result<structs::GPTCodeSplitResponse, OpenAIError> 
    {
        let prompt = prompt_utils::split_source_file_prompt(
            programming_lang, code_string, language
        );
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.analyse_model)
            .stream(false)
            //.response_format(ChatCompletionResponseFormatType::JsonObject)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("As a professional programming expert, analyze the given source code file. Your response should be in JSON format.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        //println!("{:?}", response);
        let _text = response.choices[0].clone().message.content.unwrap();
        let mut text = String::new();
        if _text.starts_with("```json\n"){
            text = _text.replace("```json\n", "").replace("```", "");
        } else {
            text = _text;
        }
        println!("split: {:?}", text);
        let code_description: structs::GPTCodeSplitResponse = serde_json::from_str(&text).unwrap();
        Ok(code_description)
    }

    pub async fn ask(&self,
        query: String, code_list: Vec<String>, language: String
    ) -> Result<String, OpenAIError> 
    {
        let prompt = prompt_utils::ask_prompt(
            query, code_list, language
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.chat_model)
            .stream(false)
            .response_format(
                ChatCompletionResponseFormat {
                    r#type: ChatCompletionResponseFormatType::Text
                }
            )
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        //println!("{:?}", response);
        let _text = response.choices[0].clone().message.content.unwrap();
        Ok(_text)
    }

    pub async fn chat(&self, message: String) -> Result<String, OpenAIError> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.chat_model)
            .stream(false)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(message)
                    .build()?
                    .into(),
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;
        Ok(response.choices[0].clone().message.content.unwrap())
    }


    pub async fn embedding_compute(&self, source: Arc<dyn Array>) 
        -> Result<Float32Array, OpenAIError> 
    {

        let input = match source.data_type() {
            DataType::Utf8 => {
                let array = source
                    .as_string::<i32>()
                    .into_iter()
                    .map(|s| {
                        s.expect("we already asserted that the array is non-nullable")
                            .to_string()
                    })
                    .collect::<Vec<String>>();
                EmbeddingInput::StringArray(array)
            }
            DataType::LargeUtf8 => {
                let array = source
                    .as_string::<i64>()
                    .into_iter()
                    .map(|s| {
                        s.expect("we already asserted that the array is non-nullable")
                            .to_string()
                    })
                    .collect::<Vec<String>>();
                EmbeddingInput::StringArray(array)
            }
            _ => unreachable!("This should not happen. We already checked the data type."),
        };

        let embed = self.client.embeddings();
        let req = CreateEmbeddingRequest {
            model: self.embedding_model.clone(),
            input,
            encoding_format: Some(EncodingFormat::Float),
            user: None,
            dimensions: Some(self.dim),
        };

        // TODO: request batching and retry logic
        task::block_in_place(move || {
            Handle::current().block_on(async {
                let mut builder = Float32Builder::new();

                let res = embed.create(req).await?;

                for Embedding { embedding, .. } in res.data.iter() {
                    builder.append_slice(embedding);
                }

                Ok(builder.finish())
            })
        })
    }
}
