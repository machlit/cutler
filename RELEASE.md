## ✨

- **Bug fix:** The comparison for `no_deps` inside the `brew backup` command was previously invalid and resulted in improper dependency comparisons. This has been fixed in this release.
- Config-locks are now handled before command execution rather than within the command.
