# vripper-core

First native-engine vertical slice.

Implemented and unit-tested source-derived compatibility primitives:
- Link scanner and thread/post classification.
- Forum/post folder sanitization and optional post ID suffix.
- MIME-derived image filename normalization and optional three-digit order prefix.
- Download directory composition as observed in the reference.

Not implemented yet:
- HTTP client/session and proxy transport.
- XML/SAX thread response parser.
- Host-specific HTML resolvers.
- Download queue and streaming.
- Persistence and FFI.

Run tests from repository root:

```sh
cargo test --workspace
```

All behavior must be checked against reference fixtures before being treated as release-compatible.
