# Third-Party Notices

This project is licensed under the MIT License. See `LICENSE`.

This file provides a lightweight summary of third-party license situations for common distributions of this repository.

## How to regenerate the license inventory

From the repo root:

```bash
pnpm licenses:audit
```

This writes `license-report.json` (not committed by default). It summarizes:

- Node (pnpm) dependency licenses via `pnpm licenses list --json`
- Rust (cargo) crate licenses via `cargo metadata`

## Notes (high-signal)

### Node dependencies (pnpm)

- Most packages are permissive (MIT/ISC/Apache/BSD).
- `caniuse-lite` is `CC-BY-4.0` (commonly pulled in via build tooling such as Browserslist).
  - If you redistribute `caniuse-lite` itself, you must comply with CC-BY attribution requirements.

### Rust crates (cargo)

- Most crates are permissive (MIT and/or Apache-2.0).
- Some crates are under `MPL-2.0` (file-level copyleft) and `Unicode-3.0` (Unicode data/license).
  - If you modify MPL-2.0 licensed files and distribute them, you must comply with MPL-2.0 obligations for those files.
- `r-efi` is multi-licensed (`MIT OR Apache-2.0 OR LGPL-2.1-or-later`).
  - You can comply by selecting the MIT or Apache-2.0 option (LGPL is an alternative option, not mandatory).



