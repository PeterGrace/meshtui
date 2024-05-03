## 2024-05-03, Version v0.12.0
### Commits
- [[`2db0f05556`](https://github.com/PeterGrace/meshtui/commit/2db0f05556d081a6c861bbcab9e362dbd8586498)] chore: Release meshtui version 0.12.0 (Peter Grace)
- [[`3ed2c2722d`](https://github.com/PeterGrace/meshtui/commit/3ed2c2722d3e7d2493bcd0c5885798ac136e863f)] initial work on flatpak support for JAYC on discord (Peter Grace)
- [[`95b20a7cea`](https://github.com/PeterGrace/meshtui/commit/95b20a7ceadc4dfedc98d8034857e156c3997155)] add charts to details;reduce tick speed to decrease CPU utilization (Peter Grace)
- [[`61a1844449`](https://github.com/PeterGrace/meshtui/commit/61a1844449ea4f80c354dfa9767747164e7cc901)] keep track of changelogging (Peter Grace)
- [[`6a6cad102a`](https://github.com/PeterGrace/meshtui/commit/6a6cad102ac7e5ceeb75ba6ef99df76e7678457d)] adjust how tab and backtab behavior work (Peter Grace)
- [[`f9775305b5`](https://github.com/PeterGrace/meshtui/commit/f9775305b5f4bec291b679d6c52ba46144fbc81d)] Box<> ComprehensiveNode to help balance struct size (Peter Grace)
- [[`13a1848203`](https://github.com/PeterGrace/meshtui/commit/13a18482031c93b9b342553adfa5827d11965f1d)] facilitate F12 to force comms restart (Peter Grace)
- [[`aab4e2023e`](https://github.com/PeterGrace/meshtui/commit/aab4e2023ef84d22482d92cb979d9a4f4146279c)] play with flatpak building (Peter Grace)

### Stats
```diff
 .github/workflows/flatpak.yaml |  17 ++-
 CHANGELOG.md                   |  29 ++++-
 Cargo.lock                     |   2 +-
 Cargo.toml                     |   3 +-
 Dockerfile.flatpak             |  13 ++-
 src/app.rs                     |  72 +++++----
 src/consts.rs                  |   2 +-
 src/packet_handler.rs          |  66 ++++++--
 src/tabs/nodes.rs              | 330 ++++++++++++++++++++++++++++++++++--------
 9 files changed, 431 insertions(+), 103 deletions(-)
```


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


