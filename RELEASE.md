## ✨

- `cutler brew install` now doesn't pre-fetch the formulas and instead directly passes in the list of formulae/casks to install, taking advantage of Homebrew's parallel downloads.
- Three new flags have been added to `cutler brew install`:
  1. `--force`: Passes the `--force` flag to Homebrew when installing.
  2. `--skip-formula`: Skips formula installs.
  3. `--skip-cask`: Skips cask installs.
- Following the three flags above, identical flags have been added with a `brew_` prefix for `cutler apply` in order to pass them to `cutler brew install`.
