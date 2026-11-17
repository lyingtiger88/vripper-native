# Host Compatibility Matrix — Static Source Inventory

Registry verified in vripper-core/.../Module.kt. All entries below are bound as Host implementations in the core dependency-injection module.

| ID | Host class | Host name | Matched address | Resolver selector / behavior observed |
|---:|---|---|---|---|
| 0 | AcidimgHost | acidimg | acidimg.cc | Continue button id=continuebutton; image class=centred |
| 1 | DPicMeHost | dpic | dpic.me | image id=pic |
| 2 | ImageBamHost | imagebam | imagebam.com | Continue text fallback; image class contains main-image |
| 3 | ImageTwistHost | imagetwist | imagetwist.com | image class contains img |
| 4 | ImageVenueHost | imagevenue | imagevenue.com | Continue link title=Continue to ImageVenue; image under data-toggle=full and id=main-image |
| 5 | ImageZillaHost | imagezilla | imagezilla.net | image id=photo |
| 6 | ImgboxHost | imgbox | imgbox.com | image id=img |
| 7 | ImgSpiceHost | imgspice | imgspice.com | image id=imgpreview |
| 8 | ImxHost | imx | imx.to | Special base-host path; optional original filename setting; continue name=imgContinue; title selector class contains img-title |
| 9 | PimpandhostHost | pimpandhost | pimpandhost.com | image class contains original; resolver may use a rewritten URL |
| 10 | PixhostHost | pixhost | pixhost.to, pixhost.cc | image id=image |
| 11 | PixRouteHost | pixroute | pixroute.com | image id=imgpreview |
| 12 | PixxxelsHost | pixxxels | pixxxels.cc | image id=download; title from main/div/div[3]/div[1]/h6 |
| 13 | PostImgHost | postimg | postimg.cc | image class contains img-fluid; resolver normalizes http: to https: |
| 14 | TurboImageHost | turboimagehost | turboimagehost.com | title from titleFullS/h1; image id=imageid |
| 15 | ViprImHost | vipr | vipr.im | image class contains img |

## Important distinctions

- The README's published host list omits DPicMe, but the core module actively registers DPicMeHost.
- The README lists ImageZilla as imagezilla.com, while the implementation matches imagezilla.net. This discrepancy must not be silently normalized; confirm actual current behavior against reference fixtures and document the accepted source-of-truth.
- The Imx host ID is 8 and the base Host implementation has a special branch for that ID that calls the resolver without the generic HEAD/direct-image check.
- The selectors above are source-code observations, not a guarantee that the live websites currently return those structures.
- This matrix intentionally does not provide instructions to evade anti-bot or access controls.

## Porting acceptance per host

For each registered host:
1. Capture the exact source implementation and blob SHA.
2. Identify URL match semantics and supported domains.
3. Identify redirect, continue-page, form, cookie and referer behavior.
4. Identify DOM/XPath selector(s), fallback order and title extraction.
5. Record direct-image and HTML response paths.
6. Create authorized, sanitized HTML fixtures and expected resolver output.
7. Test MIME validation, temporary file, final name and error behavior.
8. Verify host-specific settings and optional original-name behavior.
