# Generating code

`rok gen` turns the schema directory into Rust: the data layer
(`src/db/**`) and the API features (`src/features/**`). The schema is the
only source of truth; nothing generated writes DDL back.

## Run it

```sh
rok gen               # write what changed, printing every file
rok gen --dry-run     # plan only; write nothing
rok gen --json        # the same report, as JSON
```

`--force` overwrites files the generator did not write. `--strict` fails
on warnings (a table or query it had to skip).

## Change the schema

```sh
rok gen --migration add_orders
rok gen --rename users=accounts --migration rename_users
```

`--migration NAME` stages a migration pair for the changes since the last
one. A rename would otherwise read as a drop and a create; `--rename`
(repeatable, `table=new` or `table.column=new`) says what it is.
`--allow-destructive` allows the steps that lose data or can fail on
existing rows.

## In CI

```sh
rok gen check
```

It writes nothing, reports the missing, changed and stale files, and exits
1 when the generated code is out of date.

## Older generators

`rok generate` (a whole model crate from `.sql` files) and `rok sqlc …`
(SQLx resource modules) still work as they did; `rok gen` supersedes both.
