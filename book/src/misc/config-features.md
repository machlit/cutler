# Configuration Features

There are some nifty features built into the software for your convenience. These configuration features have been documented below so that you can have a quick look:

## Config-Locking

> **WARNING:** This feature is **still under development** and changes will be made to alter parts of its functionality in later releases, so be sure to stay alert before you use it in your everyday configuration.

If you want to prevent yourself/others from repeatedly applying your configuration, you can run the following command:

```bash
$ cutler lock
```

This will append the following line to the config:

```toml
lock = true
```

This is a **soft-lock** of cutler's configuration, which prevents you from using the commands which alter your system preferences.

Some commands which are disabled by locking the config include:
- `apply`
- `unapply`
- `reset`
- `config` (prevents edits)
- ... and so on.

To unlock it:

```bash
$ cutler unlock
```
