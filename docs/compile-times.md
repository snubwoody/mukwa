# Measuring compile times

Install `summarize` and `crox`

```bash
RUSTFLAGS='-Aunused_unsafe' cargo install --git https://github.com/rust-lang/measureme --branch stable summarize croz
```

Compile `mukwa` with the `-Zself-profile` flag to emit compiler profile files.

```bash
RUSTFLAGS='-Zself-profile' cargo build -p mukwa
```

Use `summarize` to read the profile data.

```bash
summarize summarize mukwa-<id>.mm_profdata
```
