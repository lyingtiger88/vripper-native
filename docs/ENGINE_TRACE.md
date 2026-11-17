# Engine Behavior Trace — Initial Findings

## Host registry (verified in core module)
The Koin core module registers these Host implementations:
- AcidimgHost
- DPicMeHost
- ImageBamHost
- ImageTwistHost
- ImageVenueHost
- ImageZillaHost
- ImgboxHost
- ImgSpiceHost
- ImxHost
- PimpandhostHost
- PixhostHost
- PixRouteHost
- PixxxelsHost
- PostImgHost
- TurboImageHost
- ViprImHost

This is the active DI registration list observed in Module.kt. The README's published list does not mention DPicMe; the discrepancy should be preserved as a documented audit item until confirmed with maintainer intent.

## Thread lookup response mapping
Source: ThreadLookupAPIResponseHandler.kt

The SAX handler recognizes:
- error: reads details attribute.
- thread: reads id and title.
- forum: reads title.
- user: reads hash as security token.
- post: reads id, number and title; blank post title falls back to thread title.
- image: reads main_url, thumb_url and type.

Observed image mapping:
- Rewrites a literal http:// prefix to https:// for main_url and thumb_url.
- Only image elements whose type attribute equals linked are added.
- The main URL must match a registered Host.
- Per-post host counts are accumulated.
- Post URL is constructed as siteProxy/threads/{threadId}?p={postId}&viewfull=1#post{postId}.
- A post is added only when the image list is non-empty.
- The handler clears image and host accumulators after each post.
- Thread result includes thread ID/title, security token, forum, ordered posts and error text.

Critical parity implications:
- Preserve SAX element handling and attribute defaults.
- Preserve image filtering to type=linked.
- Preserve host matching semantics and per-post image ordering.
- Preserve empty-post omission.
- Preserve HTTP-to-HTTPS rewrite behavior exactly unless a tested compatibility decision changes it.

## Retry policy
Source: RetryPolicyService.kt

- Default maximum attempts: 3.
- Retry delay range: 2 to 5 seconds.
- Failed attempts are logged.
- Exact retryable exception classification and call sites still need inspection.

## Settings and configuration
Source: SettingsService.kt

- Main configuration file: config.json in the VRipper data directory.
- Custom proxy/base list: proxies.json in the same directory.
- Built-in proxies are loaded from packaged resource proxies.json.
- Custom proxy list is merged with defaults.
- https://vipergirls.to is explicitly added.
- JSON encoding writes defaults, pretty prints and ignores unknown keys on read.
- If configured Viper host is no longer in proxy list, it falls back to https://vipergirls.to.
- Invalid settings cause fallback to default Settings.
- When login is disabled, credentials and thanks preference are cleared from settings.
- When password changes, SettingsService hashes it with MD5 before saving. This is a security weakness to address in the native migration; preserve login compatibility only through an explicit, secure migration plan, not by perpetuating MD5 as a new password-storage design.

## Authentication
Source: VGAuthService.kt

- Optional ViperGirls login controlled by settings.
- Login POST is sent to configured host + /login.php?do=login.
- Form includes username, cookieuser=1, do=login and vb_login_md5password.
- Authentication expects vg_userid and vg_password cookies.
- Click lookup uses a dedicated HTTP context and session cookie state.
- Remaining cookie/token refresh, leave-thanks and failure behavior needs complete inspection.

## Persistence
Source: DatabaseManager.kt and db.changelog-master.xml

- SQLite path is {VRIPPER_DIR}/vripper.db.
- Liquibase applies schema changelog at startup.
- Schema has relational entities for posts, images, metadata, tasks, threads and post actions; exact columns and indexes must be mapped before migration.

## Download transfer and finalization
Source: Host.kt and ImageDownloadRunnable.kt

- Host HEAD request adds Referer: https://vipergirls.to/.
- For HTML responses, host-specific resolver obtains image name and direct image URL.
- Image body is streamed to a temp file, buffer size 8192 bytes.
- Downloaded byte count and speed are updated while streaming.
- Image MIME is checked against BMP/GIF/JPEG/PNG/WEBP/SVG.
- Final extension is selected from MIME, not trusted solely from URL suffix.
- Destination copy uses replace-existing behavior.
- Temporary file is removed after final copy attempt.

## Known security/compatibility issues to resolve
- MD5 password storage is weak; new credential storage must use platform secure storage and modern password/session handling where supported.
- Existing HTTP/cleartext gRPC settings require a separate transport-security audit.
- Cookies and site credentials are sensitive; logs and diagnostics must redact them.
- Do not implement anti-bot bypass or access-control circumvention.
