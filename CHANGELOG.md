## 2024-04-30, Version v0.11.0
### Commits
- [[`61d51aff7e`](https://github.com/PeterGrace/meshtui/commit/61d51aff7e3815ca54cba021d3061f9631478110)] chore: Release meshtui version 0.11.0 (Peter Grace)
- [[`468a5782cd`](https://github.com/PeterGrace/meshtui/commit/468a5782cdbb158565cfd7a522abe38ced582975)] clippified (Peter Grace)
- [[`eb5f536860`](https://github.com/PeterGrace/meshtui/commit/eb5f536860a9d14b165a38ffcc4423427f5b2bc6)] Clippy stuff (Peter Grace)
- [[`cb2134478c`](https://github.com/PeterGrace/meshtui/commit/cb2134478c7cdc60e7ccd32e61de64bcdcbe1877)] channels list (Peter Grace)

### Stats
```diff
 Cargo.lock                    |   2 +-
 Cargo.toml                    |   2 +-
 src/app.rs                    | 124 ++++++++++++++-----------------
 src/consts.rs                 |   2 +-
 src/ipc.rs                    |   1 +-
 src/main.rs                   |  17 ++--
 src/meshtastic_interaction.rs |  75 ++++++++-----------
 src/packet_handler.rs         | 149 +++++++++++++++++++++++---------------
 src/tabs.rs                   |   6 +-
 src/tabs/channels.rs          |  86 ++++++++++++++++++++++-
 src/tabs/device_config.rs     |  37 ++-------
 src/tabs/messages.rs          |  25 +++---
 src/tabs/modules_config.rs    |  90 ++++++++++++-----------
 src/tabs/nodes.rs             | 170 +++++++++++++++++++------------------------
 src/theme.rs                  |  17 +---
 src/util.rs                   |  27 +++++--
 16 files changed, 458 insertions(+), 372 deletions(-)
```


