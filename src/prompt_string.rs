pub const SPLIT_SOURCE_FILE: &str = "I have the following code in {{ programming_language }}:

```
{{ code }}
```

Could you please tell me what classes or functions in this code, and the class's/function's source code?

Make sure the JSON output is structured as follows:

```
{
  \"classes\": [
    {
      \"name\": \"string\", // this class's name
      \"source_code\": \"string\", // this class's raw content
    }
  ],
  \"functions\": [
    {
      \"name\": \"string\", // this function's name
      \"source_code\": \"string\", // this function's raw content
    }
  ]
}
```

This JSON output will help us understand the structure and functionality of the source code file in a clear and concise manner, So PLEASE give me a JSON data follow above structure.
";

////////////////////////

pub const CHAT_WITH_RELATED_SOURCE_FILES: &str = "Here is the user's query:

```
{{ query }}
```

Understand the user's query and explain with the related description and source code:

{% for item in content_list %}
{{ item}}

{% endfor %}

No need to give the whole source code back.
";

////////////////////////

pub const ANALYSE_SOURCE_FILE: &str = "I have the following code in {{ programming_language }}:

```
{{ code }}
```

Could you please explain what this code does, including the purpose of class and key part of the code?

Make sure the JSON output is structured as follows:

```
{
  \"purpose\": \"string\", // what this code is doing
  \"classes\": [
    {
      \"name\": \"string\", // this class's name
      \"source_code\": \"string\", // this class's raw content
      \"purpose\": \"string\" // what this class is doing
    }
  ],
  \"functions\": [
    {
      \"name\": \"string\", // this function's name
      \"source_code\": \"string\", // this function's raw content
      \"purpose\": \"string\" // what this function is doing
    }
  ]
}
```

This JSON output will help us understand the structure and functionality of the source code file in a clear and concise manner, So PLEASE give me a JSON data follow above structure.
";


////////////////////////


pub const CHAT_WITH_MODEL: &str = "Here is the user's query:

```
{{ query }}
```

Understand the user's query and give a reasonable response.
";

////////////////////////

pub const GET_RELATED_SOURCE_FILES: &str = "Here is the user's query:

```
{{ query }}
```

Here are the source code files and their names and purposes:

[
{% for item in source_file_indexes %}
  {
    \"file\": {{ item.file }},
    \"name\": {{ item.name }},
    \"purpose\": {{ item.purpose }}
  },
{% endfor %}
]

Return at most {{ max_file_count }} file names and the reaons that are most relevant to the user's query. The output file should be the same complete file path as the following source code files.

Make sure the JSON output is structured as follows:

```
{
  \"related_files\": [
    {
      file: \"string\",
      reason: \"string\"
    }
  ]
}
```
";

////////////////////////
