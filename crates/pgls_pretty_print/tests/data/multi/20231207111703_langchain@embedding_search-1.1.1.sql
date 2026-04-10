insert into app.package_versions(package_id, version_struct, sql, description_md)
values (
(select id from app.packages where package_alias= 'langchain@embedding_search'),
(1,1,1),
$pkg$
-- Enforce requirements
-- Workaround to https://github.com/aws/pg_tle/issues/183
do $$
    declare
        dependencies_exists boolean = exists(
            select 1
            from pg_available_extensions
            where
                name = 'vector'
                and installed_version is not null
        );
    begin

        if not dependencies_exists then
            raise
                exception '"langchain@embedding_search" requires "vector"'
                using hint = 'Run "create extension vector" and try again';
        end if;
    end
$$;

-- Create a table to store your documents
create table documents (
  id bigserial primary key,
  content text, -- corresponds to Document.pageContent
  metadata jsonb, -- corresponds to Document.metadata
  embedding vector(1536) -- 1536 works for OpenAI embeddings, change if needed
);

-- Create a function to search for documents
create function match_documents (
  query_embedding vector(1536),
  match_count int,
  filter jsonb DEFAULT '{}'
) returns table (
  id bigint,
  content text,
  metadata jsonb,
  similarity float
)
language plpgsql
as $$
#variable_conflict use_column
begin
  return query
  select
    id,
    content,
    metadata,
    1 - (documents.embedding <=> query_embedding) as similarity
  from documents
  where metadata @> filter
  order by documents.embedding <=> query_embedding
  limit match_count;
end;
$$;

$pkg$,

$description_md$
# embedding_search

[LangChain](https://js.langchain.com/docs/) is a framework for developing applications powered by language models with a plugable architecture.

`langchain@embedding_search` uses a Supabase Postgres database as its vector store.

## Installation

```sql
select dbdev.install('langchain@embedding_search');
create extension if not exists vector;
create extension "langchain@embedding_search"
    schema public
    version '1.1.0';
```
Note:

`vector` is a dependency of `langchain@embedding_search`.
Dependency resolution is currently under development.
In the near future it will not be necessary to manually create dependencies.


### Standard Usage

The below example shows how to perform a basic similarity search with Supabase:

Once created, you can access the vector store for search using langchain as shown below:

```js
import { SupabaseVectorStore } from "langchain/vectorstores/supabase";
import { OpenAIEmbeddings } from "langchain/embeddings/openai";
import { createClient } from "@supabase/supabase-js";

const privateKey = process.env.SUPABASE_PRIVATE_KEY;
if (!privateKey) throw new Error(`Expected env var SUPABASE_PRIVATE_KEY`);

const url = process.env.SUPABASE_URL;
if (!url) throw new Error(`Expected env var SUPABASE_URL`);

export const run = async () => {
  const client = createClient(url, privateKey);

  const vectorStore = await SupabaseVectorStore.fromTexts(
    ["Hello world", "Bye bye", "What's this?"],
    [{ id: 2 }, { id: 1 }, { id: 3 }],
    new OpenAIEmbeddings(),
    {
      client,
      tableName: "documents",
      queryName: "match_documents",
    }
  );

  const resultOne = await vectorStore.similaritySearch("Hello world", 1);

  console.log(resultOne);
};
```

### Metadata Filtering

Given the above `match_documents` Postgres function, you can also pass a filter parameter to only documents with a specific metadata field value.

```js
import { SupabaseVectorStore } from "langchain/vectorstores/supabase";
import { OpenAIEmbeddings } from "langchain/embeddings/openai";
import { createClient } from "@supabase/supabase-js";

// First, follow set-up instructions at
// https://js.langchain.com/docs/modules/indexes/vector_stores/integrations/supabase

const privateKey = process.env.SUPABASE_PRIVATE_KEY;
if (!privateKey) throw new Error(`Expected env var SUPABASE_PRIVATE_KEY`);

const url = process.env.SUPABASE_URL;
if (!url) throw new Error(`Expected env var SUPABASE_URL`);

export const run = async () => {
  const client = createClient(url, privateKey);

  const vectorStore = await SupabaseVectorStore.fromTexts(
    ["Hello world", "Hello world", "Hello world"],
    [{ user_id: 2 }, { user_id: 1 }, { user_id: 3 }],
    new OpenAIEmbeddings(),
    {
      client,
      tableName: "documents",
      queryName: "match_documents",
    }
  );

  const result = await vectorStore.similaritySearch("Hello world", 1, {
    user_id: 3,
  });

  console.log(result);
};
```

For more details, checkout the LangChain Supabase integration docs: https://js.langchain.com/docs/modules/indexes/vector_stores/integrations/supabase
$description_md$
);
