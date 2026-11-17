# Synthetic Compatibility Fixtures

These are synthetic local HTML/XML fixtures created from selectors and response attributes observed in the reference source. They contain no real user data, session cookies, credentials, or downloaded images.

- `hosts/`: minimal markup for each registered host resolver; continuation flows have separate before/after files.
- `api/thread-response.xml`: synthetic response for the SAX thread parser.

The fixtures are structural test inputs only. They do not establish that a live host currently serves the same markup, and they do not authorize automated access to any site.

Planned tests:
- Parse each fixture with the same XPath/HTML semantics as the reference.
- Assert resolved image URL, title, request headers, cookies and continuation behavior.
- Assert API parser ordering, linked-image filtering, HTTP→HTTPS normalization and generated post URLs.
