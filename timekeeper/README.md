
Start timekeeper with default configuration:

```
cargo run --bin timekeeper -- --db-user=<DB_USER> --db-password=<DB_PASSWORD>
```

Use `--regatta-id=<ID>` to select the regatta for loading and persisting timestamps.
Without this option, the latest regatta is selected. The selected ID is logged when
the time strip loads. When running alongside the infoportal, select the same ID as
its `ACTIVE_REGATTA_ID` override.
