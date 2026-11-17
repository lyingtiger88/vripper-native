# Remote Client/Server Protocol Audit

Reference: `vripper-core/src/main/proto/*.proto`, `vripper-web/grpc` and the Kotlin Multiplatform remote client.

## Architectural finding

The reference is not only a local desktop downloader. It has a local endpoint and an optional remote gRPC endpoint. The mobile client implements the remote `EndpointApi` interface and delegates business logic to the server. The desktop UI can switch session mode as well.

The native migration must therefore preserve both execution modes:

1. **Local mode:** Rust engine runs in-process on the device; Flutter talks to it through the versioned FFI.
2. **Remote mode:** Flutter connects to a compatible remote VRipper engine/server through a native protocol adapter. The remote mode must retain session selection, connection state, authentication, event streams and remote file/image transfer semantics.

The exact remote server deployment target (desktop-only, Linux service, or additional platforms) remains an architecture decision to finalize after server service dependencies are audited. The initial mobile behavior must not silently switch to local mode when remote connection fails.

## gRPC service surface

The reference `EndpointService` exposes these operation groups.

### Posts and download control
- Scan links.
- Stream new/update/delete posts.
- Find all posts and one post.
- Restart/stop selected or all posts.
- Remove posts and clear completed.
- Rename post and rename-to-first.
- Move queued post up/down/top/bottom.
- Find images for a post and stream image updates.
- Stream image bytes as `ImageChunk` messages with image ID, byte offset, data, missing flag and final-chunk flag.

### Threads
- Stream new/update/delete/clear events.
- Find all threads.
- Remove/clear threads.
- Grab/select posts from a thread.
- Queue selected thread/post pairs for download.

### Tasks, logs and metadata
- Find all tasks and tasks by post.
- Stream new/update/dismiss/clear task events.
- Dismiss and clear tasks.
- Stream metadata updates.
- Stream logs.
- Initialize logger.

### Status and observability
- Stream download speed.
- Stream logged-in user changes.
- Stream queue state and retrieve current queue state.
- Stream error count.
- Stream task-running state.
- Get version and detailed server information.
- Run database migration endpoint.

### Settings and session
- Get/save settings.
- Get Viper base/proxy list.
- Stream settings updates.
- Get logged-in user.
- Change session is a desktop/mobile UI workflow around the remote/local endpoint selection.

## Data contracts observed

- Post carries IDs, title/thread/forum, URL, token, total/done, host list, download path, folder name, status, bytes, previews, metadata and post-action fields.
- Image carries IDs, source and thumbnail URLs, host ID, index, parent post reference, size/downloaded, status and filename.
- Thread carries local ID, title, link, external thread ID and total count.
- Task carries ID, name, category, description, status, result, timestamps and optional post entity ID.
- QueueState carries running count, remaining count and per-post rank.
- ImageChunk carries missing, image ID, byte offset, bytes and final marker.

## Transport security observed

- gRPC service is configured with a shared passphrase.
- Request and response protobuf streams are wrapped by AES utility marshallers.
- This message-level encryption does not replace TLS: transport metadata, endpoints and traffic patterns still need TLS protection.
- The old mobile client stores the passphrase in platform settings; native migration must use OS secure storage.
- New protocol design must include authenticated TLS, modern key derivation, key rotation and replay/nonce handling. Do not perpetuate weak or ambiguous legacy crypto as the new protocol.

## Native protocol strategy

- Define a transport-neutral domain API shared by local and remote sessions.
- Local adapter calls Rust directly via FFI.
- Remote adapter implements the same interface over a versioned protocol.
- Preserve streaming semantics for posts, images, tasks, logs, queue state and download chunks.
- Add explicit capability negotiation/version handshake.
- Add reconnect/backoff and cancellation semantics without duplicating or silently losing active download state.
- Keep proxy settings and remote connection settings distinct: a network proxy may be used to reach a remote server, but it does not replace the remote session's own host/port/authentication configuration.

## Acceptance tests
- [ ] Local session starts and runs without a server.
- [ ] Remote session connects to a compatible test server and reports version/user.
- [ ] Failed remote connection remains disconnected and shows actionable error.
- [ ] Session switching closes old streams and does not duplicate subscriptions.
- [ ] Posts, threads, tasks, logs and queue state streams match reference state transitions.
- [ ] Image chunk offsets, ordering, missing marker and final marker are correct.
- [ ] Remote proxy settings route the remote endpoint traffic through the configured proxy.
- [ ] No passphrases, tokens or cookies are logged or persisted in plaintext.
- [ ] Protocol version mismatch is detected before incompatible operations run.
