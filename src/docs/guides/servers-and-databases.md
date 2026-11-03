# Servers and databases

A **server** is a named PostgreSQL connection you register once and reach
from any project. Register it, then create, drop and query databases on
it by name.

## Register a server

```sh
rok server add local --url postgres://postgres:secret@localhost:5432/postgres
```

The settings go to `servers.toml` in your config directory; the password
goes to the OS keychain (service `rok`, account `local`), never to a
file. Without a password in the URL, type it at a prompt instead:

```sh
rok server add local --host localhost --user postgres --password-prompt
```

## When a password is missing

`rok db create … --server local` fails with "no password for server
'local'" when neither the keychain nor `ROK_SERVER_LOCAL_PASSWORD` has
one. Store one:

```sh
rok server update local --password-prompt
```

Or, for one shell, set the variable: the name upper-cased, `-` as `_`.
Where there is no keychain (headless Linux, CI), the variable is the only
source; `ROK_KEYCHAIN=off` skips the keychain on purpose.

`rok server list` shows where each server's password comes from.

## Change or forget a server

```sh
rok server update local --host db.internal --port 6543   # only these change
rok server remove local                                   # asks you to type its name
```

## Databases on a server

```sh
rok db create shop_dev --server local
rok db query 'select count(*) from users' --server local
rok db drop shop_dev --server local
```

`db drop` asks you to type the database's name (`--yes` skips that, for
scripts), and refuses the database your project's `DATABASE_URL` names
unless you pass `--force`. `db query` runs in one transaction that
commits only when every statement succeeds.

Without `--server`, these commands use `DATABASE_URL`, and so do
`rok db check` and `rok db inspect`.
