## Iterative compile times
All done on Windows.

Command run: `cargo build -p mukwa`

Changing a simple string inside a @tr("").

Nightly

 nightly-1.101.0-2026-10-13

|Build |Duration|
|---|---|
|Default|1m 16s|
|Nightly|1m 37s|
|-Zthreads=12|1m 15s|
|-Zshare-generics=y|46.72s|
