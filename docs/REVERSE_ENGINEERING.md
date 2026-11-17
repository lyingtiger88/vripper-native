# Reference Reverse Engineering Notes

**Reference:** https://github.com/dev-claw/vripper-project  
**Branch inspected:** main  
**Method:** static Git tree and source inspection. No reference build or runtime test has been executed yet.

## 1. Repository structure
The Maven parent declares:
- vripper-core: domain, persistence, site parsing, host resolution, download scheduling and services.
- vripper-gui: JavaFX/TornadoFX desktop UI.
- vripper-web and vripper-web-ui: server/API and web interface.
- vripper-mobile-client: separate Kotlin Multiplatform / Compose Multiplatform client.

The desktop UI and engine are separate modules. The mobile client is a separate implementation; it does not make the desktop application a native Flutter application.

## 2. Desktop shell and navigation
Sources:
- vripper-gui/.../components/views/AppView.kt
- vripper-gui/.../components/views/MainView.kt

AppView composition order:
1. Menu bar
2. Optional action toolbar
3. Remote error banner
4. Main tab view
5. Optional status bar

The root accepts dropped text URLs and forwards a URI to the scan controller.

MainView has four non-closable tabs:
- Posts view — download-tab
- Threads table — thread-tab
- Tasks table
- Logs table

Full screen inventory remains open: menus, toolbar actions, columns, filters, context menus, keyboard shortcuts, dialogs, preferences and all empty/loading/error states.

## 3. Link scanning and site navigation
Sources:
- vripper-core/.../tasks/ScanLinksTask.kt
- vripper-core/.../vgapi/ThreadLookupAPIParser.kt
- vripper-core/.../vgapi/PostLookupAPIParser.kt
- vripper-core/.../vgapi/ThreadLookupAPIResponseHandler.kt

Observed scanner behavior:
- Split input by newline, trim, discard blanks.
- A link must start with one of the configured proxy/base prefixes.
- Match a thread path with numeric thread ID and optional post query parameter.
- Thread-only URLs schedule thread lookup; post-specific URLs schedule post queueing.
- Invalid URLs are collected and reported as ignored.
- Thread and post operations are submitted as separate tasks.

Lookup behavior:
- GET requests target https://viper.click/vr.php.
- Thread lookup uses query parameter t; post lookup uses p.
- Response is parsed by SAX via ThreadLookupAPIResponseHandler.
- Non-2xx responses become failures.
- Retry policy and request limiting are applied.

The exact SAX event mapping, post/image ordering and URL rewriting still need inspection before porting.

## 4. Host extraction
Base source: vripper-core/.../host/Host.kt. Individual implementations are under vripper-core/.../host/.

README host list:
acidimg.cc, imagetwist.com, imagezilla.com, imgspice.com, imagebam.com, imgbox.com, imx.to, pimpandhost.com, pixhost.to, pixxxels.cc, turboimagehost.com, postimg.cc, imagevenue.com, pixroute.to and vipr.im.

The tree also contains DPicMeHost; active registration and support status must be confirmed.

Observed base Host behavior:
- Host selection matches the URL against configured host address strings.
- For most hosts, perform HEAD and inspect Content-Type.
- Direct image responses are downloaded directly.
- HTML responses are passed to the host-specific resolver.
- Image data streams to a temporary file using an 8192-byte buffer.
- Referer defaults to https://vipergirls.to/ unless overridden in context.
- Accepted MIME types: BMP, GIF, JPEG, PNG, WEBP and SVG.
- A special host ID bypasses the initial HEAD branch; its active mapping and rationale need follow-up.

Do not implement access-control bypasses. Respect authorization, terms, anti-bot controls and server responses.

## 5. Download lifecycle
Sources:
- vripper-core/.../services/download/DownloadService.kt
- vripper-core/.../services/download/QueueManager.kt
- vripper-core/.../services/download/ImageDownloadRunnable.kt

Observed:
- Scheduler fallback maximum pool size is 24.
- Pending work is grouped by host and constrained by global and per-host settings.
- Queue tracks pending groups and active image runnables.
- Posts can be stopped/restarted and queue positions moved.
- Download context retains HTTP requests, cookies, headers, child jobs, stop and completion flags.
- Image state changes to DOWNLOADING; post state is updated; host is selected; image is resolved/downloaded; then MIME/name validation and finalization occur.
- Progress is periodically persisted during streaming.
- Stop cancels active requests and coroutine work; incomplete stopped images become STOPPED.
- Retry policy is a separate service and must be traced with Failsafe use.

The Rust implementation must preserve observable scheduling behavior while replacing shared mutable state and mixed Java thread/coroutine mechanisms with explicit ownership, cancellation and bounded async concurrency.

## 6. Output paths and filenames
Sources:
- vripper-core/.../utilities/PathUtils.kt
- vripper-core/.../services/download/ImageDownloadRunnable.kt
- vripper-core/.../host/Host.kt

Observed directory calculation:
- Begin with configured download path.
- If forum subdirectories are enabled, append sanitized forum name.
- If thread sub-location is enabled, append thread title.
- Final destination is post.downloadDirectory joined with post.folderName.

Observed image naming:
- Use host-resolved image name; base Host can derive a name from the URL's final path segment.
- MIME determines uppercase extension: BMP, GIF, JPG, PNG, WEBP or SVG.
- If the supplied name has a recognized image extension, retain its basename and replace extension with MIME-derived extension.
- Otherwise sanitize the supplied name and append the MIME-derived extension.
- If forceOrder is enabled, prepend a three-digit, one-based index and underscore (example: 001_).
- Copy to destination with replace-existing semantics.
- Temporary file uses configured temp path, prefix vripper_ and suffix .tmp; it is deleted after final copy attempt.
- Sanitization replaces a defined set of reserved/control characters with underscores. Exact regex behavior must be locked down with test vectors, not paraphrased.

Potential edge cases: thread title sanitization, Windows reserved names, trailing dots/spaces, Unicode, long paths and case-sensitive filesystems. Preserve reference outputs first; document any OS-forced exception.

## 7. Persistence
DatabaseManager connects to SQLite at vripper.db inside the VRipper application data directory and runs Liquibase changelog updates. The migration must preserve IDs, relations, statuses, metadata, settings and existing file references.

## 8. Image viewer
PostWorkspaceFragment and related components:
- ImageCanvasPane
- ImageLoadPipeline
- DeepZoomController
- ViewZoomController
- WindowGeometryController
- DecodedImageCache

Observed responsibilities include overview/browse and single-image modes, current index, asynchronous image loading, speculative preloading, encoded/decoded caching, memory bounds, deep zoom, fit/zoom controls, minimap viewport, geometry persistence and refresh of download status.

Visual and gesture parity requires reference screenshots and scripted interaction recordings.

## 9. Immutable source blob snapshots
| Source file | Git blob SHA |
|---|---|
| PathUtils.kt | 5892a5f08df9c9e33d2ce6b1e29ccba77315bdf0 |
| DownloadService.kt | d56aa256d679a9ab84ad84003b15839f58232453 |
| ImageDownloadRunnable.kt | 154e0249cb23be59e8a49326ccd6faedd6cf651e |
| QueueManager.kt | 65e4cfbd8fa2c8b28b88d2ffd541c45b8f37c452 |
| PostLookupAPIParser.kt | 5f9253097db27dc607a9c3dce6c0f25185a02332 |
| ThreadLookupAPIParser.kt | 9dd3d25dd5d4731ecd86ffaf4d23fca18e4264ff |
| ScanLinksTask.kt | 6192c9cb664d292f11fe8d5431e7dc5e9fdbbb14 |
| Host.kt | 40dc35b6cd1bc26124850ad0b19208c717ed7228 |
| MainView.kt | ed4f7c0e921abecea33e45701dcab78852bd5232 |
| AppView.kt | 4635d0e07912dc8f725437791da07de2b36f8c73 |
| PostWorkspaceFragment.kt | bbd246c53ec2bdc6625bc1e4394871ec85fa0f55 |
| AddLinksFragment.kt | 29e62d2cd8ee770f7cc697ab5955af16962a18ee |
| DatabaseManager.kt | 3ed4d426a564d88cb37d39c4731923729f2db46f |

Re-check the source branch before porting; hashes identify the exact inspected blobs.

## 10. Remaining audit tasks
- Inspect all host resolver implementations and verify active registration.
- Inspect ThreadLookupAPIResponseHandler and model mapping.
- Inspect full QueueManager, retry policy, authentication, cookie handling, tasks and state transitions.
- Inventory all GUI views/controllers/settings and decide which server/web/mobile screens are in migration scope.
- Extract DB changelog and settings defaults.
- Capture reference screenshots and authorized local fixtures.
- Build golden vectors for URL parsing, host resolution, sanitization, filename extension, folder layout, ordering, overwrite and interrupted downloads.
- Audit GPL-3.0 and all third-party asset/dependency licenses before copying or adapting code.
