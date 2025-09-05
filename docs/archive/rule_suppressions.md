# Rule Suppressions

You can suppress specific diagnostics or rules in your code using suppression comments. This is useful when you want to ignore a particular rule for an entire file, a line or a block of code.

## How to Suppress a Rule

To suppress a rule, add a comment above the line causing the diagnostic with the following format:

```sql
-- pgt-ignore lint/safety/banDropTable
drop table users;
```

You can suppress single rules, groups of rules, or entire categories. The format of the rule to suppress is:

`category(/group(/specific-rule))`

Where group and specific rule are optional.

So, to suppress the `lint/safety/banDropTable` diagnostic, all of these would work:

```sql
-- pgt-ignore lint
-- pgt-ignore lint/safety
-- pgt-ignore lint/safety/banDropTable
```

You can also add an explanation to the suppression by adding a `:` and the explanation text:

```sql
-- pgt-ignore lint/safety/banDropTable: My startup never had any users.
drop table users;
```

### Suppressing Rules for Block of Code

You can suppress rules for blocks of code.

```sql
create table users (
  -- ...
);

-- pgt-ignore-start typecheck: The `users` table will be created with this migration.
alter table users drop constraint users_pkey;

alter table users add primary key (user_id);
-- pgt-ignore-end typecheck
```

Every `pgt-ignore-start` needs a `pgt-ignore-end` suppression comment, and the suppressed rules must match exactly.

This _won't_ work, because the start tag suppresses a different diagnostic:

```sql
-- pgt-ignore-start lint/safety/banDropColumn
-- pgt-ignore-end lint/safety
```

Nesting is allowed, so this works fine:

```sql
-- pgt-ignore-start typecheck: outer
-- pgt-ignore-start lint/safety: inner
-- pgt-ignore-end lint/safety: inner
-- pgt-ignore-end typecheck: outer
```

### Suppressing Rules for Entire Files

Instead of repeating the same suppression on multiple lines, you can suppress for an entire file.

```sql
-- pgt-ignore-all lint/safety/banDropTable

drop table tasks;
drop table projects;
drop table users;
```

## Suppressing Multiple Rules

You can suppress multiple rules by adding multiple suppression comments above a statement:

```sql
-- pgt-ignore lint/safety/banDropColumn
-- pgt-ignore typecheck
alter table tasks drop column created_at;
```

## Notes

- Trying to suppress diagnostics that have already been disabled in your [configuration file](/#configuration) will show a warning.
- Trying to suppress diagnostics that don't haven't been raised will also show a warning.
