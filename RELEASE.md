## ✨

- Internal code changes have been done to provide same path and config/snapshot context to all commands of cutler. To elaborate, a new `AppContext` struct implementation has been introduced which is passed through `Runnable`.
- The `Snapshot` struct is now a builder for the new `LoadedSnapshot` struct - similar to `Config` and `LoadedConfig`.
- Overall stability has been uplifted.
