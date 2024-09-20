use tokio;
use std::{iter::once, sync::Arc};
use std::path::{Path, PathBuf};

use arrow::{
    buffer::Buffer,
    array::{
        Float32Builder,
        ArrayData,
        Array,
        ListArray,
    }
};
use arrow_array::{
    cast::{
        as_map_array,
        AsArray
    },
    types::Float32Type,
    FixedSizeListArray,
    PrimitiveArray,
    Float32Array,
    Int32Array,
    RecordBatch,
    RecordBatchIterator,
    StringArray,
    builder::{
        FixedSizeListBuilder,
        PrimitiveBuilder,
    },
};


use arrow_schema::{DataType, Field, Schema};

use futures::{StreamExt, TryStreamExt};

use async_openai::error::OpenAIError;

use lancedb::{
    connect, connection::CreateTableMode, 
    query::{ExecutableQuery, QueryBase, Select}, table::Table,
    Connection, Result
};

use crate::structs;

use crate::openai_utils;

use crate::env;

use crate::pooling;

const TABLE_NAME: &str = &"vectors";

const MAX_TOKENS: usize = 8192;

pub struct Embedding <'a> {
    client: &'a openai_utils::OpenAI,
    db: Connection,
    table: Table,
    dim: usize,
}

impl <'a> Embedding <'a> {
    pub async fn new(env: &env::Env, client: &'a openai_utils::OpenAI) -> Result<Self> {

        let path = env.work_dir().join(".readit").join("db");

        let db = connect(path.as_path().to_str().unwrap()).execute().await?;
        let table = Self::open_table(&db, env.config.dim() as i32).await?;

        Ok(Self {
            client,
            db,
            table,
            dim: env.config.dim() as usize,
        })
    }

    fn get_schema(dim: i32) -> Arc<Schema>{
        let schema = Arc::new(Schema::new(vec![
            //Field::new("id", DataType::Int32, true),
            //Field::new("line_number" , DataType::Int32   , false)    ,
            //Field::new("lines"       , DataType::Int32   , false)    ,
            Field::new("file"        , DataType::Utf8    , false)    ,
            Field::new("md5"         , DataType::Utf8    , false)    ,
            Field::new("code_type"   , DataType::Utf8    , false)    , // "file" , "class" , "function"
            Field::new("lang"        , DataType::Utf8    , false)    ,
            Field::new("name"        , DataType::Utf8    , false)    ,
            Field::new("purpose"     , DataType::Utf8    , false)    ,
            Field::new("content"     , DataType::Utf8    , false)    ,  // name + purpose + code
            Field::new(
                "embedings", 
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    dim as i32,
                ),
                false
            ),  // name + purpose + code
        ]));
        schema
    }

    async fn init_table(db: &Connection, dim:i32) -> Result<Table>{
        let schema = Self::get_schema(dim);
        db.create_empty_table(TABLE_NAME, schema)
            .mode(CreateTableMode::Overwrite)
            .execute()
            .await
    }

    async fn open_table(db: &Connection, dim: i32) -> Result<Table> {

        let table = match db.open_table("vectors").execute().await {
            Err(_) => {
                println!("no table");
                let t = Self::init_table(db, dim).await?;
                t
            },
            Ok(t) => t
        };

        Ok(table)
    }

    fn split_string(&self, s: String) -> Vec<String> {
        let mut r: Vec<String> = Vec::new();
        let mut sum = 0;
        let mut rr: Vec<String> = Vec::new();
        for l in s.split("\n") {
            if l.len() + sum < MAX_TOKENS{
                rr.push(l.to_string());
                sum += l.len();
            } else {
                r.push(rr.join("\n").to_string());
                sum = 0;
                rr = Vec::new();
            }
            
        };
        r
    }

    async fn embedding_compute(&self, query: String)
        -> Result<(Float32Array, u32)> 
    {
        let (embedding, tokens) = match self.client.embedding_compute(
                Arc::new(StringArray::from_iter_values(once(query.clone())))
            ).await
        {
            Ok((e, t)) => (e, t),
            Err(e) => {
                match e {
                    OpenAIError::ApiError(ref ee) => {
                        if ee.message.contains(
                            "This model's maximum context length "
                        ) {
                            let string_list = self.split_string(query);
                            let mut _e_all_builder = PrimitiveBuilder::<Float32Type>::new();
                            let mut token = 0;
                            for s in string_list.iter() {
                                let (e, t) = self.client.embedding_compute(
                                    Arc::new(StringArray::from_iter_values(once(s.clone())))
                                ).await.unwrap();
                                token += t;
                                for e in e.iter() {
                                    _e_all_builder.append_value(e.unwrap());
                                }
                            };
                            let _e_all = _e_all_builder.finish();
                            (pooling::pooling(self.dim, &_e_all), token)
                        } else {
                            panic!("embedding error, {:?}", e);
                        }
                    },
                    _ => {
                        panic!("embedding error, {:?}", e);
                    }
                }
                //println!("embedding error: {:?}", e);
                //panic!("embedding error, {:?}", e);
            }
        };
        Ok((embedding, tokens))
    }
      
    pub async fn add_data(&self, data: structs::CodeDescription) -> Result<u32> {

        let schema = Self::get_schema(self.dim as i32);

        //let line_number = Int32Array::from(vec![data.line_number]);
        //let lines = Int32Array::from(vec![data.lines]);
        let file = StringArray::from_iter_values(vec![ data.file.clone().unwrap(),]);
        let md5 = StringArray::from_iter_values(vec![ data.md5.clone().unwrap(),]);
        let code_type = StringArray::from_iter_values(vec![ data.code_type.clone().unwrap(),]);
        let lang = StringArray::from_iter_values(vec![ data.lang.clone().unwrap(),]);
        let name = StringArray::from_iter_values(vec![ data.name.clone(),]);
        let purpose = StringArray::from_iter_values(vec![ data.purpose.clone(),]);
        
        let content_string = format!("//file {:} \n//{:} name: {:}\n\n// {:}\n{:}",
                data.file.clone().unwrap(),
                data.code_type.clone().unwrap(),
                data.name.clone(),
                data.purpose.clone(),
                data.source_code.clone()
        );

        let content = StringArray::from_iter_values(vec![content_string.to_string()]);

        let (embedding, tokens) = self.embedding_compute(content_string).await?;

        let float_builder = Float32Array::builder(self.dim);
        let mut fixed_size_list_builder = FixedSizeListBuilder::new(float_builder, self.dim as i32);  // 每个子数组长度为 3

        for e in embedding.iter() {
            fixed_size_list_builder.values().append_value(e.unwrap());
        }
        fixed_size_list_builder.append(true);

        let embedding_array = fixed_size_list_builder.finish();

        let rb = RecordBatch::try_new(
            schema.clone(),
            vec![
                //Arc::new(line_number),
                //Arc::new(lines      ),
                Arc::new(file       ),
                Arc::new(md5        ),
                Arc::new(code_type  ),
                Arc::new(lang       ),
                Arc::new(name       ),
                Arc::new(purpose    ),
                Arc::new(content    ),
                Arc::new(embedding_array  ),
            ],
        )?;
        self.table.add(Box::new(RecordBatchIterator::new(vec![Ok(rb)], schema)))
            .execute()
            .await?;
        Ok(tokens)
    }

    pub async fn delete_file(&self, data: structs::CodeDescription) -> Result<()> {
        self.table.delete(
            format!("file = '{}'", data.file.clone().unwrap()).as_str()
        ).await?;
        Ok(())
    }
    

    //pub async fn update(&self, data: structs::CodeDescription) -> Result<u32> {
    //    self.table.delete(
    //        format!("file = '{}'", data.file.clone().unwrap()).as_str()
    //    ).await?;
    //    self.add_data(data).await
    //}

    pub async fn clean_all(&self) -> Result<()> {
        self.table.delete("md5 like '%'").await
    }

    pub async fn search(&self, prompt: String) -> Result<(Vec<String>, u32)> {

        let query = Arc::new(StringArray::from_iter_values(once(prompt)));

        let (query_vector, tokens) = self.client.embedding_compute(query)
            .await
            .unwrap()
        ;
        let query_vector = query_vector
            .iter()
            .map(|x| x.unwrap())
            .collect::<Vec<f32>>()
        ;

        //println!("query's embedding: {:?}", query_vector);

        let results = self.table.query()
            .nearest_to(query_vector)
            .unwrap()
            //.refine_factor(2)
            //.nprobes(20)
            .limit(10)
            ;
        //let s = results.explain_plan(true).await.unwrap();
        //println!("explain: {}", s);
        //println!("query: {:?}", results);
        let results = results
            .execute()
            .await?
            .try_collect::<Vec<RecordBatch>>()
            .await?
        ;
        println!("find {} answers", results.iter().len());
        let r: Vec<String> = results.iter().map(|rb| {
            let out = rb.column_by_name("content")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                ;
            let text = out.iter().next().unwrap().unwrap();
            let name = rb.column_by_name("name")
                .unwrap()
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                ;
            //let name = name.iter().next().unwrap().unwrap();
            //println!("name: {}", name);
            //println!("text: {}", text);
            text.to_string()
        }).collect();
        Ok((r, tokens))
    }

    pub async fn is_file_change(&self, file_path: &String, md5: &String) -> Result<bool> {
        let query = format!("file == \"{}\" and md5 == \"{}\"", file_path, md5);
        let results = self.table.query()
            .select(Select::Columns(vec!["file".to_string(), "md5".to_string()]))
            .only_if(query)
            .execute()
            .await?
            .try_collect::<Vec<RecordBatch>>()
            .await?
        ;
        Ok(results.len() == 0)
    }


    pub async fn search_other(&self, column: String, value: String) -> Result<Vec<RecordBatch>> {
        let query = format!("{} == \"{}\"", column, value);
        let results = self.table.query()
            .select(Select::All)
            .only_if(query)
            .limit(9999)
            .execute()
            .await?
            .try_collect::<Vec<RecordBatch>>()
            .await?
        ;
        Ok(results)
    }

    pub async fn all(&self) -> Result<Vec<RecordBatch>> {
        let results = self.table.query()
            .select(Select::All)
            .execute()
            .await.unwrap();
        let results = results
            .try_collect::<Vec<RecordBatch>>()
            .await.unwrap()
        ;
        Ok(results)
    }

    pub async fn update_summary(&self, language: String) -> u32 {
        self.table.delete("file = 'whole project'").await.unwrap();
        let all = self.all().await.unwrap();

        //let all = self.search_other(
        //    "code_type".to_string(),
        //    "file".to_string(),
        //).await.unwrap();

        let mut all_file_des: Vec<(String, String)> = all
            .iter()
            .map(|rb| {
                let file = rb.column_by_name("file")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .iter()
                    .next()
                    .unwrap()
                    .unwrap()
                    .to_string()
                ;
                let purpose = rb.column_by_name("purpose")
                    .unwrap()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .iter()
                    .next()
                    .unwrap()
                    .unwrap()
                    .to_string()
                ;
                (file, purpose)
            })
            .collect()
        ;
        all_file_des.sort_by_key(|x| x.0.clone());

        let summary = all_file_des
            .iter()
            .map(|(f, p)| format!("{}: {}", f, p))
            .collect::<Vec<String>>()
            .join("\n")
        ;

        //println!("summary: {}", summary);
        let (summary, t) = self.client.summarize(summary, language).await.unwrap();
        println!("summarize token usege: {}", t);
        //println!("summary2: {}", summary);

        self.add_data(structs::CodeDescription {
            file: Some("whole project".to_string()),
            md5: Some("".to_string()),
            code_type: Some("file".to_string()),
            lang: Some("".to_string()),
            name: "whole project summary".to_string(),
            purpose: summary,
            source_code: "".to_string(),
        }).await.unwrap()
    }
}
