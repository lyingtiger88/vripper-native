# Network Settings — Feature Specification

**Status:** Approved requirement for implementation  
**UI location:** Settings sidebar → Network (new independent section)  
**Existing Connection section:** Retained unchanged for concurrency, timeout and retry controls.

## User-facing controls

### Proxy mode
A required mode selector with these choices:
1. **Disabled** — connect directly; this is the default.
2. **Use system proxy** — use the operating system proxy configuration where the platform exposes it.
3. **Manual proxy** — use the endpoint configured below.

### Manual proxy fields
Visible and editable only when Manual proxy is selected:
- Protocol: HTTP, HTTPS, SOCKS5.
- Server / hostname (required).
- Port (required; valid range 1–65535).
- Username (optional).
- Password (optional; masked).
- Bypass list (optional; newline or comma-separated hostnames/domains).

### Actions and feedback
- **Test connection** action uses the configured proxy against a small, safe connectivity check; it must not start a download or expose credentials in logs.
- Show a clear result state: testing, success, failure, timeout, authentication required.
- **Save** persists the configuration. Invalid values block save with field-level validation.
- Proxy credentials are stored only in OS secure storage (Windows Credential Manager/DPAPI-backed secure storage, macOS Keychain, Linux Secret Service where available, Android Keystore-backed storage, iOS Keychain). They must not be written in plaintext to config JSON or SQLite.
- When switching away from Manual proxy, retain the manual endpoint values but do not use them until selected again; credentials remain in secure storage and can be cleared explicitly.

## Traffic scope
The selected proxy mode applies consistently to all application-owned outbound HTTP(S) requests:
- Forum/site login and session requests.
- Thread and post lookup API calls.
- Host-page resolution and image downloads.
- Any future network metadata/update checks, if enabled.

No network call may silently bypass the selected proxy. Exceptions must be explicitly documented and surfaced (for example, an OS-level connectivity check used only by the proxy test itself).

## Security and correctness
- Never log proxy passwords, authorization headers, cookies, or full credential-bearing proxy URLs.
- Support proxy authentication without embedding credentials in persisted endpoint strings.
- Validate host, protocol and port before saving.
- Do not add proxy rotation, public proxy discovery, proxy scraping, or anti-bot/access-control circumvention.
- TLS certificate validation remains enabled; no insecure TLS mode is introduced.
- Direct mode is the default and must behave like the reference's current networking path.
- System proxy support is platform-dependent. If unavailable, show a clear unsupported/unavailable state rather than silently falling back to direct mode.
- SOCKS5 DNS behavior must be explicit and tested; prefer remote DNS resolution when supported to avoid leaking host lookups outside the proxy.

## Architecture requirements
- Add a single NetworkConfiguration domain model shared by Flutter and Rust.
- Rust HTTP client/session construction must consume this configuration centrally; individual host resolvers must not construct independent clients that bypass it.
- Proxy configuration changes apply to newly created requests and new sessions. Active requests should either finish with their original client or be cancelled and recreated according to an explicit, tested policy; do not mutate a live connection pool unsafely.
- Proxy test uses the same transport configuration as production requests, with a dedicated short timeout.
- Use typed errors for unsupported system proxy, DNS failure, proxy connection failure, proxy authentication failure and destination TLS failure.
- FFI must never return secret values to Flutter after save. Flutter may submit credentials to secure storage through a one-time write operation; reads should return only whether a credential is configured.

## Acceptance tests
1. Default installation starts in Disabled/direct mode.
2. Disabled mode preserves direct-connection behavior.
3. System mode uses OS proxy settings where supported.
4. System mode reports unsupported configuration without direct fallback.
5. Manual HTTP, HTTPS and SOCKS5 proxy connections work.
6. Invalid/missing host and port are rejected; ports outside 1–65535 are rejected.
7. Proxy authentication success and failure are handled correctly.
8. Bypass list applies consistently and predictably.
9. DNS lookup behavior for SOCKS5 is tested for leaks.
10. All app HTTP traffic uses the selected mode (login, lookup, host resolution, image download).
11. Test connection returns actionable status without creating download records.
12. Proxy credentials never appear in logs, config files, database, crash reports or FFI read responses.
13. Changing proxy mode does not corrupt active downloads or leak requests through direct connections.
14. Configuration survives restart and migrates safely from installations with no network section.
15. Each desktop/mobile platform has a smoke test for supported proxy modes.

## UI parity and layout
The new Network item is an additive sidebar section. Existing Appearance, Downloads, Connection, System, Automation, Viper, Hosts, Posts and Viewers sections remain present and retain their behavior. Use the same navigation button style, spacing, section title and scrollable content region as the existing settings sections.
