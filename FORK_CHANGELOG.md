# Fork changelog

This tree is [seefs001/grok-build](https://github.com/seefs001/grok-build), a
personal fork of [xai-org/grok-build](https://github.com/xai-org/grok-build).
Upstream remains the SpaceXAI public snapshot. This file lists **fork-only**
behavior relative to the last synced upstream commit.

The stock CLI changelog is still
[`crates/codegen/xai-grok-shell/CHANGELOG.md`](crates/codegen/xai-grok-shell/CHANGELOG.md)
and [x.ai/build/changelog](https://x.ai/build/changelog).

| | |
| --- | --- |
| Upstream | https://github.com/xai-org/grok-build |
| Fork | https://github.com/seefs001/grok-build |
| Based on | `48271133` (“Synced from monorepo”, 1.0.32) |
| `SOURCE_REV` | `be7ce6e8cffe46d20bef9834b211616082ee866b` |
| Date | 2026-09-15 |

## Remotes

```sh
git remote -v
# origin    https://github.com/seefs001/grok-build.git
# upstream  https://github.com/xai-org/grok-build.git
```

Sync from SpaceXAI:

```sh
git fetch upstream
git merge upstream/main
```

## Features

- **Grok 4.6 default reasoning effort** is `xhigh` (catalog default was `high`).
- **Session Fast** via `[models].fast`. On SuperGrok / cli-chat-proxy traffic
  (`X-XAI-Token-Auth: xai-grok-cli`), the sampler stamps Responses
  `service_tier: "priority"`. Grok 4.6 has no `*-fast` model id; Fast is this
  field. Stock default is off, and API-key traffic never gets the stamp.
  Explicit `service_tier` on a request is not overwritten.
- **Reasoning summary** global fallback via `[models].reasoning_summary`
  (`none` / `auto` / `concise` / `detailed`). Upstream 1.0.28 added the
  per-model `[model.<id>].reasoning_summary` and the `ReasoningSummary` type
  (with `none`); the fork now reuses that type and sampler plumbing, and only
  keeps the global key as a fallback that applies when the per-model value is
  unset. Stock default stays `concise`.
- **`reasoning` alias**: Chat Completions `reasoning` is accepted as an alias
  for `reasoning_content` on deltas and messages; serialization keeps the
  canonical wire name.
- **ACP session config** now comes from upstream (1.0.14–1.0.16): `new_session`
  / `load_session` advertise both a `model` select and a `thought_level`
  `reasoning_effort` select; `session/set_config_option` is handled by
  `handlers/config_option.rs`. The fork no longer carries a private
  effort-only implementation.
- **ACP `usage_update`**. Occupancy (used / window size) and optional USD cost
  are emitted as a live-only `session/update` with `sessionUpdate:
  "usage_update"`. It is independent of the status-row capability: clients
  that do not draw the row still get a meter without a git discovery. Attach
  always requests a fresh snapshot because these notifications are not in
  `updates.jsonl`.
- **`web_fetch` RFC 2544 Fake IPs**. `[toolset.web_fetch].allow_rfc2544_ips`
  (default off, trusted TOML layers only — not the `GROK_CONFIG` overlay)
  allows hosts that resolve into `198.18.0.0/15` (Surge-style Fake IP).
  Loopback, RFC1918, link-local, and cloud-metadata ranges stay blocked.

## Privacy and update policy

Hard-off in this fork; not configurable from `config.toml`:

- Telemetry mode is always `Disabled`.
- Sentry / error reporting is always off.
- Auth data-collection predicates fail closed (`is_data_collection_disabled`
  always true; `allows_data_collection` always false).
- Remote session registry config is never built (`build_registry_config`
  returns `None`).
- Auto-update checks are skipped at the pager gate. Background
  `ensure_latest_on_disk` / `check_update_background` are no-ops. Only an
  explicit user `grok update` (`CliUpdateTrigger::UserCommand`) may install.

## Config keys added

```toml
[models]
fast = false                         # SuperGrok Fast; service_tier = priority
reasoning_summary = "concise"        # none | auto | concise | detailed (fallback; [model.<id>] wins)

[toolset.web_fetch]
allow_rfc2544_ips = false            # 198.18.0.0/15 Fake IP opt-in
```

See also [05-configuration.md](crates/codegen/xai-grok-pager/docs/user-guide/05-configuration.md)
and [26-config-reference.md](crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md).
