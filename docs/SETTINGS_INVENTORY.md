# Reference Settings Inventory

Reference source: `dev-claw/vripper-project`, branch `main`. This document records the existing settings UI and model behavior. It is not a redesign proposal.

## Settings navigation

`SettingsFragment.kt` builds a left sidebar with these sections, in this order:

1. Appearance
2. Downloads
3. Connection
4. System
5. Automation
6. Viper
7. Hosts
8. Posts
9. Viewers

The settings window uses a 205px preferred sidebar (185–230px range), a scrollable content region, and a footer Save action. Save validates and persists service settings, then commits staged Posts, Image Viewer and Overview Window preferences.

The new **Network** section requested for VRipper Native is additive. Do not rename or merge the existing Connection section.

## Existing controls by section

### Appearance
- Color theme selector.
- Accent color picker, preset swatches and reset.
- Font family selector (system default plus installed families).
- Font size spinner constrained by AppThemeManager min/max.

### Downloads
- Download path; editable only for local session, with Browse in local mode.
- Auto start downloads.
- Auto queue thread threshold.
- Organize by category (forum subdirectory).
- Organize by thread (thread sub-location).
- Order images (force order prefix).
- Append post ID to download folder.
- Clear Finished.

### Connection
- Concurrent downloads per host: 1–4.
- Global concurrent downloads: 0–24; 0 means scheduler's fallback max pool (24).
- Connection timeout: 1–300 seconds.
- Maximum attempts: 1–10.
- This section is not proxy configuration. Preserve it unchanged.

### System
- Temporary path; local-session Browse.
- Maximum log entries: UI spinner 10–10,000 (service validation message says [100,10000], an inconsistency to verify).
- Clipboard monitoring toggle.
- Clipboard polling rate: minimum 500ms, shown only when monitoring is enabled.

### Automation
- Enable post automation.
- Mode: AUTO / REVIEW.
- Required custom fields.
- Compress post folder.
- Trigger action: NONE / MOVE / WEBHOOK / SCRIPT.
- MOVE: destination picker and overwrite option.
- WEBHOOK: URL, GET/POST method and payload.
- SCRIPT: script path picker and arguments.
- Exact visibility/validation for all conditional controls must be confirmed from remaining source lines.

### Viper
- Viper domain selected from built-in plus custom proxy/base list.
- Request limit: 1–5 requests/second.
- Fetch Metadata.
- Authentication toggle.
- When enabled: username, password and Leave likes.
- This is site/account configuration and is distinct from the new general network proxy mode.

### Hosts
- Host-specific settings generated from typed HostName/HostSettingKey map.
- Current registered host-specific setting: Imx → TRY_TO_FETCH_ORIGINAL_FILENAME (Boolean).

### Posts
- Default layout: TABLE / LIST / GRID.
- Grid tile size.
- Preview popups.

### Viewers
- Overview window default layout: GRID / TABLE.
- Overview tile size.
- Details panel visibility.
- Reset remembered position/size and divider.
- Image viewer fade transition.
- Decoded frame cache MB.
- Disk cache MB.
- Byte preload radius.
- Byte preload concurrency.
- Decode preload concurrency.

## Settings persistence and validation observations

- Settings are serialized to `config.json` under the VRipper data directory.
- Custom Viper base/proxy entries are stored separately in `proxies.json`; packaged defaults are merged with custom entries.
- Unknown JSON keys are ignored; defaults are encoded.
- Invalid settings restore defaults.
- The service validates download directory existence/type, queue threshold, concurrency, timeout, retry count, event log size, clipboard polling rate, Viper request limit and automation requirements.
- Viper password is MD5-hashed before config persistence when changed. This is legacy behavior and a security weakness; do not reproduce MD5 as the new credential-storage design. A compatibility migration must preserve the ability to authenticate only through a secure, explicitly tested flow.
- The current settings Save flow stages some UI preferences and commits them after successful service settings save; preserve this transactional user experience.

## Network section requirements

See `NETWORK_SETTINGS.md`. In brief:
- Add Network as a separate sidebar item.
- Proxy modes: Disabled, System, Manual.
- Manual protocol choices: HTTP, HTTPS, SOCKS5.
- Endpoint, optional credentials, bypass list and Test connection.
- Apply proxy consistently to all app-owned network calls.
- Store secrets in OS secure storage, not plaintext JSON/database/logs.
