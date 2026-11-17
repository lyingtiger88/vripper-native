# Download Lifecycle and Output Audit

Reference sources: `DownloadService.kt`, `QueueManager.kt`, `ImageDownloadRunnable.kt`, `Host.kt`, `PathUtils.kt`. Static inspection only; behavior still requires deterministic fixtures and runtime tests.

## Scheduling and queue semantics

- Queue is grouped as lists of image queue elements, with pending groups and active runnable collection.
- Pending work is grouped by host for scheduler capacity calculation.
- Global max pool fallback is 24.
- `maxGlobalConcurrent=0` means use fallback 24; otherwise use configured value (validated 0–24).
- Per-host concurrency is configured 1–4.
- Scheduler computes remaining global capacity and per-host capacity, then accepts the first eligible items from each host group.
- Queue supports moving post groups UP, DOWN, TOP and BOTTOM.
- Queue state reports running count, remaining image count and per-post 1-based ranks.
- Each accepted image becomes an ImageDownloadRunnable with a copy of current settings.
- Failsafe retries failed jobs with a delay range of 2–5 seconds and maximum attempts default 3.
- On terminal failure, image status becomes ERROR.
- After completion, running entry is removed; a post is finished only when no pending or running items remain and the context was not stopped.
- Queue and error-count events are published after state changes.

## Per-image lifecycle

1. Initialize context and load persisted cookie files from the VRipper data directory.
2. Set image status DOWNLOADING and reset downloaded bytes.
3. Set parent post DOWNLOADING if not already; optionally schedule the Thanks action through authentication service.
4. Select registered Host based on source URL.
5. Resolve page/direct-image URL and download to a temporary file.
6. Validate actual response MIME against BMP/GIF/JPEG/PNG/WEBP/SVG.
7. Build destination directory from post downloadDirectory + folderName.
8. Sanitize image basename and select uppercase extension based on MIME:
   - BMP → BMP
   - GIF → GIF
   - JPEG → JPG
   - PNG → PNG
   - WEBP → WEBP
   - SVG → SVG
9. If forceOrder is enabled, prepend a one-based, three-digit index and underscore (example `001_`).
10. Copy to final destination with REPLACE_EXISTING.
11. Delete temporary file in finally block.
12. If downloaded byte count equals expected image size and size > 0, mark FINISHED and increment post done/downloaded; otherwise ERROR.
13. On stop, abort HTTP requests, cancel coroutine jobs and mark incomplete image STOPPED.

## Output directory rules

- Begin with configured download path.
- If `forumSubDirectory` is enabled, append `sanitize(forum)`.
- If `threadSubLocation` is enabled, append thread title as-is in the inspected `calculateDownloadPath` method; unlike forum name it is not sanitized there. Verify downstream folder naming and platform behavior before porting.
- Image destination is `post.downloadDirectory / post.folderName`.
- If `appendPostId` is enabled, post folderName is `sanitize(postTitle) + "_" + postId`; otherwise it is `sanitize(postTitle)`. This is applied in DataAccessService.buildPosts. Preserve the underscore and decimal external post ID exactly.
- Rename operation creates sanitized new folder, moves known non-empty image filenames with ATOMIC_MOVE, and deletes old folder only if empty.

## Sanitization and extension rules

- Sanitizer replaces dot, slash, backslash, pipe, colon, question mark, asterisk, quote, angle brackets and control characters with underscore.
- Recognized pre-existing image extensions: bmp, gif, jpg, jpeg, png, webp. SVG is not in the recognized input extension list, although SVG is accepted as a MIME type and generated final extension.
- If supplied name has a recognized extension, remove it and use MIME-derived uppercase extension.
- If no recognized extension exists, sanitize full supplied name and append MIME-derived extension.
- Destination copy replaces an existing file with the same final name.

## Risks / compatibility questions

- Thread title is not sanitized in calculateDownloadPath; may cause path differences or invalid paths on some platforms.
- `appendPostId` behavior must be traced from post folder name construction.
- Sanitizer implementation and Kotlin regex escaping need exact test vectors.
- Atomic folder rename may fail across filesystem boundaries; preserve error semantics or document platform adapter behavior.
- File replacement semantics must be tested on Windows, Linux, macOS, Android and iOS sandbox storage.
- A download is only marked FINISHED if expected size is positive and downloaded byte count matches exactly.
- Stop/cancel behavior must not leave orphan temp files or stale DOWNLOADING states.

## Required golden tests

- [ ] forum subdirectory enabled/disabled
- [ ] thread sub-location enabled/disabled
- [ ] appendPostId enabled/disabled
- [ ] forceOrder enabled/disabled and index formatting
- [ ] all six accepted MIME types and exact extension mapping
- [ ] pre-existing filename extension recognized / unrecognized / absent
- [ ] Unicode, emoji, reserved characters, control characters, trailing dot/space and long names
- [ ] same-name replacement behavior
- [ ] interrupted stream and temp-file cleanup
- [ ] zero/unknown expected size and byte-count mismatch
- [ ] folder rename, atomic move failure and old-folder cleanup
- [ ] queue order movement, per-host/global capacity and retry attempts
