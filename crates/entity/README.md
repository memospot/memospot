# Memos Entities

This crate contains all entities used by the Memos server v0.18.x.

Entities are generated using the [sea-orm](https://www.sea-ql.org/SeaORM) CLI.
To regenerate the entities, run the following command:

```sh
sea-orm-cli generate entity -l -u sqlite://memos_prod.db -o crates/entity/src
```
