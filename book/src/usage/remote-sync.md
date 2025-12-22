# Remote Sync & URLs

cutler features a small logic which interacts with the cloud for your syncing needs. Completely optional - cutler works just fine if you even use it without it. This is primarily for cloud backups.

## Applying a configuration through a URL

If you want to host your configuration in the cloud and apply it remotely, you can do so using this command:

```sh
cutler apply --url https://example.com/config.toml
```

Once run, cutler will download, validate and eventually apply it to your machine.

## Remote auto-sync

To automatically download revisions of your configuration, include this section in your config along with your config URL:

```toml
[remote]
url = "https://example.com/config.toml"
autosync = true
```

The `autosync` flag ensures that the configuration will be automatically downloaded and applied the next time you use cutler.

Or, you can simply fetch from the config URL written in `[remote]` manually using the `fetch` command:

```sh
cutler fetch
```

In order to disable remote sync behavior while running any command, use the `--no-sync` global flag:

```sh
cutler status --no-sync
```

## Ignored commands

Since auto-sync is a cloud functionality and might not always be safe for certain commands, it is only allowed *permissively* for commands which directly intervene with status/apply operations.

The list of commands which do not auto-sync the configuration is given below:

- `brew backup`
- `check-update`
- `completion`
- `config`
- `cookbook`
- `fetch`
- `init`
- `lock`
- `reset`
- `self-update`
- `unapply`
