# Migrations

A migration is a numbered pair of files: `NNNN_name.up.sql` and
`NNNN_name.down.sql`. rok also reads a single file per migration with up
and down sections. Names must be versioned: `0002_add_posts`.

## Write one

```sh
rok migrate create create_posts_table
```

That writes the next numbered pair in `./migrations` (`--dir` for
another directory). With rok-data's generator, `rok gen --migration NAME`
writes the pair for you from the schema's changes.

## Apply and revert

```sh
rok migrate up            # every pending migration
rok migrate down          # the latest one (also `rok migrate rollback`)
rok migrate down --steps 2
```

They act on `DATABASE_URL`'s database, or the one `--database-url` names.

## Where things stand

```sh
rok migrate status
```

Each migration is **applied**, **pending**, **edited** (its file changed
after it was applied) or **unknown** (applied, but its file is gone). The
journal lives in the table `_rok_data_migrations`. An edited migration is
an error; `--json` reports it under `drift` instead.

`rok schema diff` compares the journal with the files in a directory, and
`rok schema dump` prints the live schema as approximate DDL.
