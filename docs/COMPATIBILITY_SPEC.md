# Compatibility Specification

This document is the acceptance contract for the native migration. A behavior is not preserved until it has a reference observation, a deterministic test where possible, and a passing native implementation.

## Compatibility levels
- **Exact:** byte-for-byte or value-for-value parity.
- **Behavioral:** same user-visible result and state transitions.
- **Platform-adapted:** unavoidable OS conventions only; exception documented and tested.

## UI and navigation
Initial observations:
- Shell order: menu bar → optional action toolbar → remote error banner → main tabs → optional status bar.
- Tabs: Posts, Threads, Tasks and Logs; tabs are non-closable.
- Root accepts dropped text URLs and starts scanning.
- Add Links dialog accepts multiline input and has a Scan action.
- Post workspace supports overview/browse and image view with zoom, deep zoom and minimap.

Acceptance:
- Match screen geometry, hierarchy, labels, icon meaning, order, control states and interaction sequence.
- Preserve menu/toolbar actions, filters, table/grid/list modes, sorting, keyboard shortcuts, context menus, drag/drop and dialogs.
- Preserve window geometry, visibility preferences, theme and viewer settings.
- Mobile layouts may adapt to screen dimensions but may not remove workflows or change data semantics.

## Site navigation and extraction
- Preserve URL patterns, normalization and invalid-link handling.
- Preserve thread/post classification and lookup request construction.
- Preserve response parsing and order of posts/images.
- Preserve host matching, host-specific resolution, referer/header behavior and MIME types.
- Preserve cookie/session behavior and retry/rate-limit semantics within authorized access.
- Never bypass access controls, CAPTCHA, anti-bot challenges or site restrictions.

## Download lifecycle
- Preserve queue order and move operations.
- Preserve global and per-host concurrency settings.
- Preserve pending/running/completed/stopped/error semantics.
- Preserve start, stop, restart, clear and retry behavior.
- Preserve progress/speed reporting.
- Preserve cancellation cleanup and temporary-file lifecycle.

## Output compatibility
Golden tests required:
1. Forum subdirectory enabled/disabled.
2. Thread sub-location enabled/disabled.
3. Forum names with reserved characters.
4. Thread titles with reserved/control characters.
5. Host filename with and without recognized extension.
6. MIME extension normalization for BMP/GIF/JPEG/PNG/WEBP/SVG.
7. Force-order enabled/disabled and one-based three-digit prefix.
8. Existing destination replacement.
9. Interrupted/failed downloads.
10. Temporary-file cleanup.
11. Folder rename and empty old-folder cleanup.
12. Unicode, emoji, trailing dots/spaces, long names and case-sensitive filesystems.

Do not improve output naming until reference vectors exist. Document and test any OS-forced divergence.

## Data compatibility
- Preserve SQLite records and relationships, IDs, statuses, metadata, tasks, settings and paths.
- Migration must be transactional, backed up, idempotent where possible and rollback-tested.
- Never automatically move or rename existing downloaded files.

## Release acceptance
- No separately installed Java/JVM required to run shipped apps.
- Native engine compiled for target architecture.
- Desktop: Windows/Linux x86-64; macOS x86-64 and ARM64.
- Mobile: Android ARM64 and iOS ARM64.
- Install/update/launch/download/pause/stop/resume/view/settings/uninstall smoke tests on each platform.


## Network and proxy compatibility
- Add a distinct Network section to Settings; preserve the existing Connection section and its concurrency/timeout/retry controls.
- Proxy modes: Disabled (default), System, Manual (HTTP/HTTPS/SOCKS5).
- Manual mode supports endpoint, optional authentication and bypass list.
- Apply the selected mode consistently to login, lookup, host resolution and image download requests.
- Never silently fall back from System/Manual proxy to direct connections.
- Store credentials only in platform secure storage; never plaintext in config/database/logs/FFI reads.
- See `NETWORK_SETTINGS.md` for full requirements and test matrix.
