# Grammar Guidelines

First off, this is not a tree-sitter grammar as it's regularly used to syntax-highlight a finished SQL file.

Instead, it is designed to work tightly with LSP features, mainly autocompletion-suggestions and hover information.

Those features should be available _while_ the SQL is being typed, and it should allow us to provide the most specific intel possible.

Here are a couple of design choices that help with that goal.

## Use Specific Identifier Types

In the original grammar we forked this one from, there was only one kind of `identifier` node.

So a `select email from auth.users` statement was parsed as `keyword_select identifier keyword_from object_reference`, with `object_reference: identifier "." identifier`.

The problem was that we would have to infer the _kind_ of identifier from context: If we were in a `select` clause, it could be a column or a function; if we were in a `from` clause, it could be a `table` or a `function`, and so on.

Today, we try to be more specific. We have various kinds of identifiers and references:

identifiers:

- `schema_identifier`
- `function_identifier`
- `type_identifier`
- `column_identifier`
- `table_identifier`
- ...

references:

- `function_reference`
- `table_reference`
- `column_reference`
- ...

and we keep the ambiguous `object_reference` and `any_identifier` that can be used anywhere we can't be more specific.

We can now parse the above statement like this:

`keyword_select column_identifier keyword_from table_reference`, with `table_reference: schema_identifier "." table_identifier`.

This helps us to suggest only columns in the select clause, and we can only suggest tables of the matching schema on the `table_identifier`.

The references are used wherever an identifier can be qualified, such as `select public.users.email from ...` or `select auth.uid()`.
They all match 2-3 qualification variants (`public.users.email`, `users.email`, `email`, ...) to cover the various states when a user is typing them.

For example, when the user types `select pu|` and we're looking at a `column_reference`, `pu` might refer to a schema, an alias, a table, or the actual column name. In that case, it is parsed as `column_reference: any_identifier`, because we can't be more specific about it.

If the user types `select public.us|`, `us` can refer to a column or a table name, and `public` can be an alias, a schema, or a table. Again, it's parsed as `column_reference: any_identifier "." any_identifier`.

Finally, if the user types `select public.users.em|`, we know for sure that `public` is a schema, `users` is a table, and `em` is a column. It is parsed as `column_reference: schema_identifier "." table_identifier "." column_identifier`.

We then use TreeSitter fields to narrow the possibilities.

Looking again at the `column_reference`, depending on the specificity, we assign the following field names:

`select pu|`:

```md
column_reference
any_identifier (@column_reference_1of1)
```

`select public.us|`:

```md
column_reference
any_identifier (@column_reference_1of2)
"."
any_identifier (@column_reference_2of2)
```

`select public.users.em|`:

```md
column_reference:
schema_identifier (@column_reference_1of3)
"."
table_identifier (@column_reference_2of3)
"."
column_identifier (@column_reference_3of3)
```

This helps us in completions, because we know that an `1of1` can be a column, schema or table, even though it's parsed as an `any_identifier`. A `2of2` can be a column or a table, but never a schema—and so on.

## We Need Partial Grammars

Treesitter only parses reliably when not in an error state.

Let's take a look at a simplified version of the `insert` rule:

```js
insert: $ => seq(
  $.keyword_insert,
  $.keyword_into,
  $.table_reference,
  $.keyword_values,
  paren_list($._expression)
),
```

If the user types `insert into |`, we would like to suggest tables for autocompletion. But since `values (..)` is missing, the tree is in an error state, and suggestions aren't working reliably.

We therefore need a grammar that matches rules as early as possible, and treats subsequent tokens optional. We have the `partialSeq` function for this.

The `partialSeq` function requires the first token and makes everything else optional:

```js
insert: $ => partialSeq(
  $.keyword_insert,
  $.keyword_into,
  $.table_reference,
  $.keyword_values,
  paren_list($._expression)
),

// expands to

insert: $ => prec.right(seq(
  $.keyword_insert,
  optional(
    seq(
      $.keyword_into,
      optional(
        seq(
          $.table_reference,
          optional(
            seq(
              $.keyword_values,
              optional(
                paren_list($._expression)
              )
            )
          )
        )
      )
    )
  )
))
```

So, everything starting from `insert |` is matched as an insert rule, but the grammar knows what kind of optional token comes next.
We use a right precedence to parse the last two tokens of `select * from table left join` as a single `left_join` clause (`$.keyword_left $.keyword_join`) rather than a separate `left_join` (consisting of only a `$.keyword_left`) and a `join` (consisting of only a `$.keyword_join`).

Of course, since using `partialSeq` makes it such that only a single keyword is required to identify a grammar rule, we have more conflicts in the grammar: `alter table something rename |` can now be a `$.rename_object` or a `$.rename_column` rule. We can handle this with either treesitter conflicts (adding too many conflicts makes treesitter slow) or by using precedence.

## We Need to Know When a Rule Finishes

We want to suggest keywords for autocompletion where they make sense. If a user types `select * from users order |`, the only suggested keyword should be `by`.

But because of `partialSeq`, this isn't so easy anymore. The keyword `order` is enough to parse as the `$.order` grammar rule; the grammar doesn't require a `by` or column list.

Any keyword that starts a new clause produces an error-free tree:

- `select * from users order where` has a valid order and a valid where clause at the end
- `select * from users order join` has a valid order and a valid join clause at the end
- `select * from users order group` has a valid order and a valid group clause at the end
- `select * from users order limit` has a valid order and a valid limit clause at the end

To filter out keywords that are valid in our grammar but not valid in actual SQL, we use field names to mark the _actual_ end of clauses.

The order by clause looks like this:

```js
order_by: partialSeq(
  $.keyword_order,
  $.keyword_by,
  field("end", comma_list($.order_target, true))
),
```

That way, we can identify `order|` as an `order_by` clause, but we also know it hasn't finished, since it does not have a child with an `end` field name.
In completions, we then filter out those keywords that open a new clause, even though the previous one isn't finished.

This requirement introduces a couple of rules for our grammar.

1. Every branch in a clause can only ever have _one_ node with an `"end"` field name.

Multiple possible branches should be separated with a `choice` function, where the last node of each branch gets the `"end"` field name.

The `order_target` rule from the `order_by` clause is a good example:

```js
order_target: ($) =>
  choice(
    field("end", $._expression),
    seq(
      $._expression,
      seq(
        choice(
          field("end", $.direction),
          seq($.keyword_using, field("end", choice("<", ">", "<=", ">=")))
        ),
        optional($.order_target_nulls)
      )
    )
  ),

order_target_nulls: ($) =>
  seq(
    $.keyword_nulls,
    field("end", choice($.keyword_first, $.keyword_last))
  ),
```

You can see how the first branch assigns an `"end"` to the `$._expression`, while the second branch does not.
The second branch does the same on a nested level, for the `$.direction` and `"<", ...` nodes.

2. Optional clauses at the end of a rule should be public.

You can see this too in the `order_target` clause.
The keyword `nulls` might appear or it might not. If it doesn't, the clause is finished at the `$.direction` or `"<", ...` nodes. If it does, we should finish on `$.keyword_first` or `$.keyword_last`.
To disambiguate this, we must open a new clause: `order_target_nulls`. When our parse sees `$.keyword_nulls`, it enters the `order_target_nulls` clause. `$.order_target` is finished, but we stay on `$.order_target_nulls` before we open e.g. a `$.limit` clause.

3. Each public rule should have an `"end"` field name.

That's the only way our parser can determine that a subclause has ended.

Take a look at the alias clause:

```js
alias: ($) =>
  choice(
    partialSeq($.keyword_as, field("end", $.any_identifier)),
    field("end", $.any_identifier)
  ),
```

Without the `end` tokens, a user might type `select * from auth.users u |`, and we would never suggest a completable keyword, since we haven't marked the `alias` clause as finished.

4. Be careful with `"end`" tokens in [hidden clauses](https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html#hiding-rules).

Hidden clauses are "spread" into their parent clauses. Suppose the `_alias` was hidden, and we have a `$.select` rule like this:

```js
select: ($) =>
  partialSeq(
    $.keyword_select,
    $.column_identifier,
    optional($._alias),
    $.keyword_from,
    field("end", $.table_reference),
  );
```

Now, if the user types `select email as e|`, the resulting looks like `keyword_select column_identifier keyword_as any_identifier(@end)`, and the select statement is prematurely considered completed.

However, we could hypothetically make the `$.table_reference` a hidden `$._table_reference` and put an `"end"` node in there. The clause would still complete at the right spot.
So, a hidden clause should only ever contain an `"end"` field if that makes sense in all possible parent statement positions.

5. Single-Token rules don't need an `"end"` field.

We have a couple of clauses that are ever a single (whitespace-separated) token, so they don't need a `partialSeq` and an `"end"` field — they are inherently completed once matched. Examples include `$.literal`, `$.bang`, `$.any_identifier`, and so on.
