# Host Resolver Audit — Source-Level Findings

Reference: `dev-claw/vripper-project`, branch `main`. All 16 classes below are registered in `vripper-core/.../Module.kt`. This is static inspection of the checked-in resolver code, not a claim that current live host pages still match the selectors. No host network probes were performed.

## Resolver matrix

| ID | Host | Domain match | Resolution behavior | Filename / request notes |
|---:|---|---|---|---|
| 0 | AcidimgHost | acidimg.cc | Fetch page; require `//input[@id='continuebutton']`; POST `imgContinue=Continue to your image`; parse `//img[@class='centred']`. | Prefer image alt; fallback to URL-derived name. POST uses page URL as Referer. |
| 1 | DPicMeHost | dpic.me | Parse `//img[@id='pic']`. | Optional alt and src; fallback filename from final URL segment, or UUID if no slash. |
| 2 | ImageBamHost | imagebam.com | Fetch page; if `//*[contains(text(), 'Continue')]` exists, add `nsfw_inter=1` and `sfw_inter=1` cookies (3-day expiry), refetch; parse `//img[contains(@class,'main-image')]`. | Prefer alt; fallback URL segment. |
| 3 | ImageTwistHost | imagetwist.com | Parse `//img[contains(@class, 'img')]`. | Uses alt and src; sets Referer to `https://imagetwist.com/`. |
| 4 | ImageVenueHost | imagevenue.com | If `//a[@title='Continue to ImageVenue']` exists, refetch page; parse `//a[@data-toggle='full']/img[@id='main-image']`. | Prefer alt; fallback URL segment. |
| 5 | ImageZillaHost | imagezilla.net | Parse `//img[@id='photo']`. | Title attribute or URL-derived fallback; direct image URL produced by literal `show` → `images` replacement. |
| 6 | ImgboxHost | imgbox.com | Parse `//img[@id='img']`. | Uses title attribute and src. |
| 7 | ImgSpiceHost | imgspice.com | Parse `//img[@id='imgpreview']`. | Prefer alt; fallback URL segment. |
| 8 | ImxHost | imx.to | Reconstruct direct image URL from thumbnail URL patterns; optionally fetch page and POST continuation to retrieve original title. | `TRY_TO_FETCH_ORIGINAL_FILENAME` host setting controls title lookup; recognizes five thumbnail URL prefixes plus already-original path. |
| 9 | PimpandhostHost | pimpandhost.com | Normalize http to https, remove `-medium(.html)?`, append `size=original` query, parse `//img[contains(@class, 'original')]`. | Prefixes src with `https:`; prefer alt, fallback URL segment. |
| 10 | PixhostHost | pixhost.to, pixhost.cc | Parse `//img[@id='image']`. | Uses alt and src. |
| 11 | PixRouteHost | pixroute.com | Parse `//img[@id='imgpreview']`. | Uses alt and src. |
| 12 | PixxxelsHost | pixxxels.cc | Parse `//*[@id='download']`; title from `/html/body/main/div/div[3]/div[1]/h6`. | Extracts href, not src; title fallback to URL segment. |
| 13 | PostImgHost | postimg.cc | Normalize `http:` to `https:`; parse `//img[contains(@class,'img-fluid')]`. | Uses alt and src. |
| 14 | TurboImageHost | turboimagehost.com | Title from `//div[contains(@class,'titleFullS')]/h1`; image from `//img[@id='imageid']`. | Empty title falls back to URL-derived name. |
| 15 | ViprImHost | vipr.im | Parse `//img[contains(@class, 'img')]`. | Uses alt and src; sets Referer to `https://vipr.im/`. |

## Cross-host behavior

- Base Host logic usually performs a HEAD request and inspects Content-Type.
- Direct image responses bypass HTML resolver parsing.
- HTML responses are passed to the selected host resolver.
- Image bodies stream to a temporary file using an 8192-byte buffer.
- Default Referer is `https://vipergirls.to/`, overridden by selected hosts above.
- Allowed image MIME types are BMP, GIF, JPEG, PNG, WEBP and SVG.
- Imx host ID 8 follows a special base-class path that skips the generic HEAD/direct-image branch; preserve and test this exact branch.
- The resolver's returned name and URL feed the common MIME-based final naming pipeline; see `DOWNLOAD_AND_OUTPUT_AUDIT.md`.

## Source provenance (Git blob SHA)

| Class | Blob SHA |
|---|---|
| AcidimgHost.kt | `858e9d4db6fdcaf04f87a24659e546ad344c86cb` |
| DPicMeHost.kt | `03a1caf552d91bea8555ea28e0f4b3e040934d41` |
| ImageBamHost.kt | `a666e9185d0aad58de99a831366833f89de7664f` |
| ImageTwistHost.kt | `0d89c8bc0c16198d55b62b373da1e950e2a667e3` |
| ImageVenueHost.kt | `afbab60e39cf668c744c5403cd193e1a1b74d328` |
| ImageZillaHost.kt | `93cecdaff4be7980e8d9f85bfc5e5323fc9bc9f8` |
| ImgSpiceHost.kt | `5dc23de007f13cb2811bee368de4232db603065c` |
| ImgboxHost.kt | `3cea37fa3d54defa9cdf6fc167b841abfd6584de` |
| ImxHost.kt | `e3ea6eee2100301a7f4347e42b67083e86c9a3df` |
| PimpandhostHost.kt | `6cc2cc2abd85edc8aad144dabc74f4517ea2c02d` |
| PixRouteHost.kt | `88c931f923961ae7422fa44765fa063c438643d0` |
| PixhostHost.kt | `5ff5abb6956555524aeba904e135b0b9b4e7b257` |
| PixxxelsHost.kt | `db99f170723c26212d61077ac7439e5395defb29` |
| PostImgHost.kt | `47885fbe91bfa81d52a18639b1e893444cae6e0a` |
| TurboImageHost.kt | `620157e3549698b0106fe76a3f583352803effd9` |
| ViprImHost.kt | `5a96a0ccd35b54c7ffc2775ef673268194246677` |

## Discrepancies and caution

- README host list omits DPicMe, but dependency injection registers DPicMeHost.
- README lists ImageZilla as `imagezilla.com`; code matches `imagezilla.net`.
- README lists PixRoute as `pixroute.to`; code matches `pixroute.com`.
- Pixhost code supports both `pixhost.to` and `pixhost.cc`.
- These are observed source differences, not resolved product decisions. The native compatibility baseline follows active code registration and URL matching unless a reproducible reference test or maintainer decision establishes otherwise.
- Selectors may no longer match live HTML. Do not attempt to evade CAPTCHA, anti-bot challenges, access controls or site restrictions.

## Required tests before port acceptance

- [ ] One sanitized, authorized HTML fixture per registered host.
- [ ] Exact resolver output (name, URL, Referer and cookies) asserted for each fixture.
- [ ] Missing selector and malformed attribute error paths tested.
- [ ] Direct-image and HTML-page base-host paths tested.
- [ ] Imx thumbnail-to-original transformations tested for every accepted prefix.
- [ ] ImageBam cookie continuation tested with a local fixture.
- [ ] Acidimg and Imx POST form fields and request headers tested.
- [ ] HTTP→HTTPS and host-specific URL rewriting tested exactly.
- [ ] Live-site validation only where authorized and permitted by site rules.
