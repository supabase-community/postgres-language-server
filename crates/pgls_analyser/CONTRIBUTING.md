# Analyser

## Creating a rule

When creating or updating a lint rule, you need to be aware that there's a lot of generated code inside our toolchain.
Our CI ensures that this code is not out of sync and fails otherwise.
See the [code generation section](#code-generation) for more details.

To create a new rule, you have to create and update several files.
Because it is a bit tedious, we provide an easy way to create and test your rule using [Just](https://just.systems/man/en/).
_Just_ is not part of the rust toolchain, you have to install it with [a package manager](https://just.systems/man/en/chapter_4.html).

### Choose a name

We follow a naming convention according to what the rule does:

1. Forbid a concept

   ```block
   ban<Concept>
   ```

   When a rule's sole intention is to **forbid a single concept** the rule should be named using the `ban` prefix.

   Example: "banDropColumn"

2. Mandate a concept

   ```block
   use<Concept>
   ```

   When a rule's sole intention is to **mandate a single concept** the rule should be named using the `use` prefix.

### Explain a rule to the user

A rule should be informative to the user, and give as much explanation as possible.

When writing a rule, you must adhere to the following **pillars**:

1. Explain to the user the error. Generally, this is the message of the diagnostic.
2. Explain to the user **why** the error is triggered. Generally, this is implemented with an additional node.
3. Tell the user what they should do. Generally, this is implemented using a code action. If a code action is not applicable a note should tell the user what they should do to fix the error.

### Create and implement the rule

> [!TIP]
> As a developer, you aren't forced to make a rule perfect in one PR. Instead, you are encouraged to lay out a plan and to split the work into multiple PRs.
>
> If you aren't familiar with the APIs, this is an option that you have. If you decide to use this option, you should make sure to describe your plan in an issue.

Let's say we want to create a new **lint** rule called `useMyRuleName`, follow these steps:

1. Run the command

   ```shell
   just new-lintrule safety useMyRuleName (<severity>)
   ```

   Where severity is optional but can be "info", "warn", or "error" (default).

   The script will generate a bunch of files inside the `pgls_analyser` crate.
   Among the other files, you'll find a file called `use_my_new_rule_name.rs` inside the `pgls_analyser/lib/src/lint/safety` folder. You'll implement your rule in this file.

1. The `Options` type doesn't have to be used, so it can be considered optional. However, it has to be defined as `type Options = ()`.
1. Implement the `run` function: The function is called for every statement, and should return zero or more diagnostics. Follow the [pillars](#explain-a-rule-to-the-user) when writing the message of a diagnostic

Don't forget to format your code with `just f` and lint with `just l`.

That's it! Now, let's test the rule.

### Rule configuration

Some rules may allow customization using options.
We try to keep rule options to a minimum and only when needed.
Before adding an option, it's worth a discussion.

Let's assume that the rule we implement support the following options:

- `behavior`: a string among `"A"`, `"B"`, and `"C"`;
- `threshold`: an integer between 0 and 255;
- `behaviorExceptions`: an array of strings.

We would like to set the options in the `postgres-language-server.jsonc` configuration file:

```json
{
  "linter": {
    "rules": {
      "safety": {
        "myRule": {
          "level": "warn",
          "options": {
            "behavior": "A",
            "threshold": 20,
            "behaviorExceptions": ["one", "two"]
          }
        }
      }
    }
  }
}
```

The first step is to create the Rust data representation of the rule's options.

```rust
#[derive(Clone, Debug, Default)]
pub struct MyRuleOptions {
    behavior: Behavior,
    threshold: u8,
    behavior_exceptions: Box<[Box<str>]>
}

#[derive(Clone, Debug, Defaul)]
pub enum Behavior {
    #[default]
    A,
    B,
    C,
}
```

Note that we use a boxed slice `Box<[Box<str>]>` instead of `Vec<String>`.
This allows saving memory: [boxed slices and boxed str use two instead of three words](https://nnethercote.github.io/perf-book/type-sizes.html#boxed-slices).

With these types in place, you can set the associated type `Options` of the rule:

```rust
impl Rule for MyRule {
    type Options = MyRuleOptions;
}
```

A rule can retrieve its options with:

```rust
let options = ctx.options();
```

The compiler should warn you that `MyRuleOptions` does not implement some required types.
We currently require implementing _serde_'s traits `Deserialize`/`Serialize`.

Also, we use other `serde` macros to adjust the JSON configuration:

- `rename_all = "camelCase"`: it renames all fields in camel-case, so they are in line with the naming style of the `postgres-language-server.jsonc`.
- `deny_unknown_fields`: it raises an error if the configuration contains extraneous fields.
- `default`: it uses the `Default` value when the field is missing from `postgres-language-server.jsonc`. This macro makes the field optional.

You can simply use a derive macros:

```rust
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct MyRuleOptions {
    #[serde(default, skip_serializing_if = "is_default")]
    main_behavior: Behavior,

    #[serde(default, skip_serializing_if = "is_default")]
    extra_behaviors: Vec<Behavior>,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub enum Behavior {
    #[default]
    A,
    B,
    C,
}
```

### Coding the rule

Below, there are many tips and guidelines on how to create a lint rule using our infrastructure.

#### `declare_lint_rule`

This macro is used to declare an analyzer rule type, and implement the [RuleMeta] trait for it.

The macro itself expects the following syntax:

```rust
use pgls_analyse::declare_lint_rule;

declare_lint_rule! {
    /// Documentation
    pub(crate) ExampleRule {
        version: "next",
        name: "myRuleName",
        severity: Severity::Error,
        recommended: false,
    }
}
```

##### Lint rules inspired by other lint rules

If a **lint** rule is inspired by an existing rule from other ecosystems (Squawk etc.), you can add a new metadata to the macro called `source`. Its value is `&'static [RuleSource]`, which is a reference to a slice of `RuleSource` elements, each representing a different source.

If you're implementing a lint rule that matches the behaviour of the Squawk rule `ban-drop-column`, you'll use the variant `::Squawk` and pass the name of the rule:

```rust
use pgls_analyse::{declare_lint_rule, RuleSource};

declare_lint_rule! {
    /// Documentation
    pub(crate) ExampleRule {
        version: "next",
        name: "myRuleName",
        severity: Severity::Error,
        recommended: false,
        sources: &[RuleSource::Squawk("ban-drop-column")],
    }
}
```

#### Category Macro

Declaring a rule using `declare_lint_rule!` will cause a new `rule_category!`
macro to be declared in the surrounding module. This macro can be used to
refer to the corresponding diagnostic category for this lint rule, if it
has one. Using this macro instead of getting the category for a diagnostic
by dynamically parsing its string name has the advantage of statically
injecting the category at compile time and checking that it is correctly
registered to the `pgls_diagnostics` library.

```rust
declare_lint_rule! {
    /// Documentation
    pub(crate) ExampleRule {
        version: "next",
        name: "myRuleName",
        severity: Severity::Error,
        recommended: false,
    }
}

impl Rule for BanDropColumn {
    type Options = Options;

    fn run(ctx: &RuleContext<Self>) -> Vec<RuleDiagnostic> {
        vec![RuleDiagnostic::new(
            rule_category!(),
            None,
            "message",
        )]
    }
}
```

### Document the rule

The documentation needs to adhere to the following rules:

- The **first** paragraph of the documentation is used as brief description of the rule, and it **must** be written in one single line. Breaking the paragraph in multiple lines will break the table content of the rules page.
- The next paragraphs can be used to further document the rule with as many details as you see fit.
- The documentation must have a `## Examples` header, followed by two headers: `### Invalid` and `### Valid`. `### Invalid` must go first because we need to show when the rule is triggered.
- Rule options if any, must be documented in the `## Options` section.
- Each code block must have `sql` set as language defined.
- When adding _invalid_ snippets in the `### Invalid` section, you must use the `expect_diagnostic` code block property. We use this property to generate a diagnostic and attach it to the snippet. A snippet **must emit only ONE diagnostic**.
- When adding _valid_ snippets in the `### Valid` section, you can use one single snippet.
- You can use the code block property `ignore` to tell the code generation script to **not generate a diagnostic for an invalid snippet**.

Here's an example of how the documentation could look like:

````rust
declare_lint_rule! {
    /// Dropping a column may break existing clients.
    ///
    /// Update your application code to no longer read or write the column.
    ///
    /// You can leave the column as nullable or delete the column once queries no longer select or modify the column.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table test drop column id;
    /// ```
    ///
    pub BanDropColumn {
        version: "next",
        name: "banDropColumn",
        recommended: true,
        severity: Severity::Error,
        sources: &[RuleSource::Squawk("ban-drop-column")],
    }
}
````

This will cause the documentation generator to ensure the rule does emit
exactly one diagnostic for this code, and to include a snapshot for the
diagnostic in the resulting documentation page.

### Testing the Rule

#### Quick Test

To quickly test your rule, head to the `pgls_analyser/src/lib.rs` file and modify the `debug_test` function.

You should:

- remove the `#[ignore]` macro if present
- change the content of the `SQL` static `&str` to whatever you need
- pass your group and rule to the `RuleFilter::Rule(..)`

If you run the test, you'll see any diagnostics your rule created in your console.

#### Snapshot Tests

After implementing your rule, you must create snapshot tests in the `tests/specs/<group>/<ruleName>/` directory.

Each test file should:
1. Start with a comment describing what the test is checking
2. Include either `-- expect_lint/<group>/<ruleName>` or `-- expect_no_diagnostics` on the first line
3. Contain valid SQL that should either trigger or not trigger your rule

**Example test structure for `addSerialColumn` rule:**

```
tests/specs/safety/addSerialColumn/
├── basic.sql                      # Basic case that triggers the rule
├── basic.sql.snap                 # Auto-generated snapshot
├── bigserial.sql                  # Test bigserial type
├── bigserial.sql.snap
├── generated_stored.sql           # Test GENERATED ... STORED
├── generated_stored.sql.snap
├── valid_regular_column.sql       # Valid case - should NOT trigger
└── valid_regular_column.sql.snap
```

**Invalid test example (should trigger diagnostic):**
```sql
-- expect_lint/safety/addSerialColumn
-- Test adding serial column to existing table
ALTER TABLE prices ADD COLUMN id serial;
```

**Valid test example (should NOT trigger diagnostic):**
```sql
-- Test adding regular column (should be safe)
-- expect_no_diagnostics
ALTER TABLE prices ADD COLUMN name text;
```

**Running and updating tests:**

```shell
# Run tests and generate snapshots
cargo test -p pgls_analyser --test rules_tests

# Review and accept new/changed snapshots
cargo insta test --accept

# Or review snapshots interactively
cargo insta review
```

The snapshot files (`.sql.snap`) are auto-generated and should be committed to the repository. They capture the expected diagnostic output for each test case.

### Code generation

For simplicity, use `just` to run all the commands with:

```shell
just gen-lint
```

### Commit your work

Once the rule implemented, tested, and documented, you are ready to open a pull request!

Stage and commit your changes:

```shell
> git add -A
> git commit -m 'feat(pgls_analyser): myRuleName'
```

### Deprecate a rule

There are occasions when a rule must be deprecated, to avoid breaking changes. The reason
of deprecation can be multiple.

In order to do, the macro allows adding additional field to add the reason for deprecation

````rust
use pgls_analyse::declare_lint_rule;

declare_lint_rule! {
    /// Dropping a column may break existing clients.
    ///
    /// Update your application code to no longer read or write the column.
    ///
    /// You can leave the column as nullable or delete the column once queries no longer select or modify the column.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```sql,expect_diagnostic
    /// alter table test drop column id;
    /// ```
    ///
    pub BanDropColumn {
        version: "next",
        name: "banDropColumn",
        recommended: true,
        severity: Severity::Error,
        deprecated: true,
        sources: &[RuleSource::Squawk("ban-drop-column")],
    }
}
````
