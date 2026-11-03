# Getting started

`rok` is one command for the rok toolchain: rok-data's commands (databases,
migrations, code generation), `rok tools`, and any `rok-<name>` program on
your `PATH`.

## Install

```sh
cargo install rok-cli
```

Or a prebuilt binary, with its checksum verified:

```sh
curl -fsSL https://raw.githubusercontent.com/ateeq1999/rok-v2/main/install.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/ateeq1999/rok-v2/main/install.ps1 | iex
```

`rok --version` says which version you have.

## A project in two commands

```sh
rok init my-api --schema schema.sql
cd my-api
rok gen
```

`rok init` writes the application skeleton, the settings file and the
schema directory, `migrations/`, seeded with the schema file you named.
Every file it writes is written once and never rewritten.

`rok gen` reads the schema directory and writes the data layer
(`src/db/**`) and the API features (`src/features/**`), printing every
file it wrote. The plain `.sql` files in the schema directory are the
schema as it is now; the migration pairs beside them are its history.

Whole projects come ready-made too:

```sh
rok init my-api --template axum
rok init my-shop --template monorepo --org com.acme
```

## Change the schema

Edit the `.sql` files, then stage the change as a migration pair and
regenerate:

```sh
rok gen --migration add_posts
```

That writes `migrations/0002_add_posts.up.sql` and its `.down.sql`: the
difference between the schema and the last staged migration. Apply it:

```sh
rok migrate up
```

In CI, `rok gen check` writes nothing and exits 1 when the generated code
is out of date.

## Next

- [Servers and databases](/guides/servers-and-databases)
- [Migrations](/guides/migrations)
- [Generating code](/guides/generating-code)
- Every command's page, from [the index](/).
