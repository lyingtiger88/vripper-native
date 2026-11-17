//! Deterministic host-page extraction logic transcribed from the reference Host classes.
//! This module performs no network requests. Callers own the HTTP session, cookies,
//! referer, redirect policy and follow-up request execution.

use regex::Regex;
use scraper::{Html, Selector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HostId {
    Acidimg = 0,
    DPicMe = 1,
    ImageBam = 2,
    ImageTwist = 3,
    ImageVenue = 4,
    ImageZilla = 5,
    Imgbox = 6,
    ImgSpice = 7,
    Imx = 8,
    Pimpandhost = 9,
    Pixhost = 10,
    PixRoute = 11,
    Pixxxels = 12,
    PostImg = 13,
    TurboImageHost = 14,
    ViprIm = 15,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostRegistration {
    pub id: HostId,
    pub name: &'static str,
    pub domains: &'static [&'static str],
}

pub const HOST_REGISTRY: [HostRegistration; 16] = [
    HostRegistration { id: HostId::Acidimg, name: "acidimg", domains: &["acidimg.cc"] },
    HostRegistration { id: HostId::DPicMe, name: "dpic", domains: &["dpic.me"] },
    HostRegistration { id: HostId::ImageBam, name: "imagebam", domains: &["imagebam.com"] },
    HostRegistration { id: HostId::ImageTwist, name: "imagetwist", domains: &["imagetwist.com"] },
    HostRegistration { id: HostId::ImageVenue, name: "imagevenue", domains: &["imagevenue.com"] },
    HostRegistration { id: HostId::ImageZilla, name: "imagezilla", domains: &["imagezilla.net"] },
    HostRegistration { id: HostId::Imgbox, name: "imgbox", domains: &["imgbox.com"] },
    HostRegistration { id: HostId::ImgSpice, name: "imgspice", domains: &["imgspice.com"] },
    HostRegistration { id: HostId::Imx, name: "imx", domains: &["imx.to"] },
    HostRegistration { id: HostId::Pimpandhost, name: "pimpandhost", domains: &["pimpandhost.com"] },
    HostRegistration { id: HostId::Pixhost, name: "pixhost", domains: &["pixhost.to", "pixhost.cc"] },
    HostRegistration { id: HostId::PixRoute, name: "pixroute", domains: &["pixroute.com"] },
    HostRegistration { id: HostId::Pixxxels, name: "pixxxels", domains: &["pixxxels.cc"] },
    HostRegistration { id: HostId::PostImg, name: "postimg", domains: &["postimg.cc"] },
    HostRegistration { id: HostId::TurboImageHost, name: "turboimagehost", domains: &["turboimagehost.com"] },
    HostRegistration { id: HostId::ViprIm, name: "vipr", domains: &["vipr.im"] },
];

pub fn host_for_url(url: &str) -> Option<HostId> {
    HOST_REGISTRY
        .iter()
        .find(|host| host.domains.iter().any(|domain| url.contains(domain)))
        .map(|host| host.id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowUpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieMutation {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub max_age_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowUpRequest {
    pub method: FollowUpMethod,
    pub url: String,
    pub referer: Option<String>,
    pub form_fields: Vec<(String, String)>,
    pub cookies: Vec<CookieMutation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImage {
    pub name: String,
    pub image_url: String,
    pub referer_override: Option<String>,
    pub cookies: Vec<CookieMutation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostResolution {
    FollowUp(FollowUpRequest),
    Resolved(ResolvedImage),
}

fn selector(value: &str) -> Selector {
    Selector::parse(value).expect("static host selector")
}

fn first_attr(document: &Html, css: &str, attribute: &str) -> Option<String> {
    document
        .select(&selector(css))
        .next()
        .and_then(|node| node.value().attr(attribute))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn first_attr_including_empty(document: &Html, css: &str, attribute: &str) -> Option<String> {
    document
        .select(&selector(css))
        .next()
        .and_then(|node| node.value().attr(attribute))
        .map(str::trim)
        .map(str::to_owned)
}

fn first_text(document: &Html, css: &str) -> Option<String> {
    document
        .select(&selector(css))
        .next()
        .map(|node| node.text().collect::<String>().trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn default_name(url: &str) -> String {
    let last = url.rsplit('/').next().unwrap_or(url);
    last.rsplit_once('.').map(|(name, _)| name).unwrap_or(last).to_owned()
}

fn required(value: Option<String>, label: &str) -> Result<String, String> {
    value.ok_or_else(|| format!("required {label} attribute or selector was not found"))
}

fn resolved(name: String, image_url: String) -> HostResolution {
    HostResolution::Resolved(ResolvedImage {
        name,
        image_url,
        referer_override: None,
        cookies: Vec::new(),
    })
}

/// Resolve a host page from a captured HTML response. If the reference would
/// perform a continuation request, this returns that request until follow-up HTML
/// is supplied. The caller must execute it through the centrally configured HTTP
/// transport; this function never accesses the network.
pub fn resolve_host_html(
    host: HostId,
    source_url: &str,
    thumbnail_url: &str,
    html: &str,
    follow_up_html: Option<&str>,
    fetch_original_imx_filename: bool,
) -> Result<HostResolution, String> {
    let document = Html::parse_document(html);

    match host {
        HostId::Acidimg => {
            let needs_continue = first_attr(&document, "input#continuebutton", "id").is_some();
            let page = if needs_continue {
                let Some(after) = follow_up_html else {
                    return Ok(HostResolution::FollowUp(FollowUpRequest {
                        method: FollowUpMethod::Post,
                        url: source_url.to_owned(),
                        referer: Some(source_url.to_owned()),
                        form_fields: vec![("imgContinue".into(), "Continue to your image".into())],
                        cookies: Vec::new(),
                    }));
                };
                Html::parse_document(after)
            } else {
                document
            };
            let image_url = required(first_attr_including_empty(&page, "img[class='centred']", "src"), "centred image src")?;
            let title = required(first_attr_including_empty(&page, "img[class='centred']", "alt"), "centred image alt")?;
            let name = if title.is_empty() { default_name(&image_url) } else { title };
            Ok(resolved(name, image_url))
        }
        HostId::DPicMe => {
            let image_url = required(first_attr_including_empty(&document, "img#pic", "src"), "pic image src")?;
            let name = first_attr(&document, "img#pic", "alt")
                .unwrap_or_else(|| image_url.rsplit('/').next().unwrap_or(&image_url).to_owned());
            Ok(resolved(name, image_url))
        }
        HostId::ImageBam => {
            let has_continue = document.root_element().text().any(|text| text.contains("Continue"));
            let (page, cookies) = if has_continue {
                let Some(after) = follow_up_html else {
                    return Ok(HostResolution::FollowUp(FollowUpRequest {
                        method: FollowUpMethod::Get,
                        url: source_url.to_owned(),
                        referer: None,
                        form_fields: Vec::new(),
                        cookies: vec![
                            CookieMutation {
                                name: "nsfw_inter".into(),
                                value: "1".into(),
                                domain: "www.imagebam.com".into(),
                                path: "/".into(),
                                max_age_seconds: Some(3 * 24 * 60 * 60),
                            },
                            CookieMutation {
                                name: "sfw_inter".into(),
                                value: "1".into(),
                                domain: "www.imagebam.com".into(),
                                path: "/".into(),
                                max_age_seconds: Some(3 * 24 * 60 * 60),
                            },
                        ],
                    }));
                };
                (
                    Html::parse_document(after),
                    vec![
                        CookieMutation {
                            name: "nsfw_inter".into(),
                            value: "1".into(),
                            domain: "www.imagebam.com".into(),
                            path: "/".into(),
                            max_age_seconds: Some(3 * 24 * 60 * 60),
                        },
                        CookieMutation {
                            name: "sfw_inter".into(),
                            value: "1".into(),
                            domain: "www.imagebam.com".into(),
                            path: "/".into(),
                            max_age_seconds: Some(3 * 24 * 60 * 60),
                        },
                    ],
                )
            } else {
                (document, Vec::new())
            };
            let image_url = required(first_attr_including_empty(&page, "img.main-image", "src"),
                "main-image src",
            )?;
            let name = first_attr(&page, "img.main-image", "alt")
                .unwrap_or_else(|| image_url.rsplit('/').next().unwrap_or(&image_url).to_owned());
            Ok(HostResolution::Resolved(ResolvedImage {
                name,
                image_url,
                referer_override: None,
                cookies,
            }))
        }
        HostId::ImageTwist => {
            let image_url = required(first_attr_including_empty(&document, "img[class*='img']", "src"), "image src")?;
            let name = required(first_attr_including_empty(&document, "img[class*='img']", "alt"), "image alt")?;
            Ok(HostResolution::Resolved(ResolvedImage {
                name,
                image_url,
                referer_override: Some("https://imagetwist.com/".into()),
                cookies: Vec::new(),
            }))
        }
        HostId::ImageVenue => {
            let has_continue = first_attr(&document, "a[title='Continue to ImageVenue']", "title").is_some();
            let page = if has_continue {
                let Some(after) = follow_up_html else {
                    return Ok(HostResolution::FollowUp(FollowUpRequest {
                        method: FollowUpMethod::Get,
                        url: source_url.to_owned(),
                        referer: None,
                        form_fields: Vec::new(),
                        cookies: Vec::new(),
                    }));
                };
                Html::parse_document(after)
            } else {
                document
            };
            let image_url = required(first_attr_including_empty(&page, "a[data-toggle='full'] > img#main-image", "src"),
                "main-image src",
            )?;
            let name = first_attr(&page, "a[data-toggle='full'] > img#main-image", "alt")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| image_url.rsplit('/').next().unwrap_or(&image_url).to_owned());
            Ok(resolved(name, image_url))
        }
        HostId::ImageZilla => {
            let title = first_attr_including_empty(&document, "img#photo", "title")
                .ok_or_else(|| "photo title attribute was not found".to_owned())?;
            let name = if title.is_empty() { default_name(source_url) } else { title };
            Ok(resolved(name, source_url.replace("show", "images")))
        }
        HostId::Imgbox => {
            let name = required(first_attr_including_empty(&document, "img#img", "title"), "img title")?;
            let image_url = required(first_attr_including_empty(&document, "img#img", "src"), "img src")?;
            Ok(resolved(name, image_url))
        }
        HostId::ImgSpice => {
            let image_url = required(first_attr_including_empty(&document, "img#imgpreview", "src"), "imgpreview src")?;
            let name = required(first_attr_including_empty(&document, "img#imgpreview", "alt"), "imgpreview alt")?;
            Ok(resolved(name, image_url))
        }
        HostId::Imx => {
            let direct_url = imx_direct_image_url(source_url, thumbnail_url)?;
            if !fetch_original_imx_filename {
                return Ok(resolved(default_name(thumbnail_url), direct_url));
            }
            let page_url = imx_page_url(source_url);
            let Some(continue_value) = first_attr(&document, "input[name='imgContinue']", "value") else {
                return Err("Imx continuation input imgContinue was not found".into());
            };
            let Some(after) = follow_up_html else {
                return Ok(HostResolution::FollowUp(FollowUpRequest {
                    method: FollowUpMethod::Post,
                    url: page_url,
                    referer: None,
                    form_fields: vec![("imgContinue".into(), continue_value)],
                    cookies: Vec::new(),
                }));
            };
            let after_document = Html::parse_document(after);
            let name = first_text(&after_document, "div[class*='img-title']")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| default_name(thumbnail_url));
            Ok(resolved(name, direct_url))
        }
        HostId::Pimpandhost => {
            let request_url = pimpandhost_original_page_url(source_url)?;
            let Some(after) = follow_up_html else {
                return Ok(HostResolution::FollowUp(FollowUpRequest {
                    method: FollowUpMethod::Get,
                    url: request_url,
                    referer: None,
                    form_fields: Vec::new(),
                    cookies: Vec::new(),
                }));
            };
            let page = Html::parse_document(after);
            let image_url = required(first_attr_including_empty(&page, "img[class*='original']", "src"),
                "original image src",
            )?;
            let image_url = format!("https:{image_url}");
            let title = required(first_attr_including_empty(&page, "img[class*='original']", "alt"), "original image alt")?;
            let name = if title.is_empty() {
                image_url.rsplit('/').next().unwrap_or(&image_url).to_owned()
            } else {
                title
            };
            Ok(resolved(name, image_url))
        }
        HostId::Pixhost => {
            let image_url = required(first_attr_including_empty(&document, "img#image", "src"), "image src")?;
            let name = required(first_attr_including_empty(&document, "img#image", "alt"), "image alt")?;
            Ok(resolved(name, image_url))
        }
        HostId::PixRoute => {
            let image_url = required(first_attr_including_empty(&document, "img#imgpreview", "src"), "imgpreview src")?;
            let name = required(first_attr_including_empty(&document, "img#imgpreview", "alt"), "imgpreview alt")?;
            Ok(resolved(name, image_url))
        }
        HostId::Pixxxels => {
            let image_url = required(first_attr_including_empty(&document, "a#download", "href"), "download href")?;
            let name = first_text(
                &document,
                "main > div > div:nth-of-type(3) > div:nth-of-type(1) > h6",
            )
            .unwrap_or_else(|| image_url.rsplit('/').next().unwrap_or(&image_url).to_owned());
            Ok(resolved(name, image_url))
        }
        HostId::PostImg => {
            let image_url = required(first_attr_including_empty(&document, "img.img-fluid", "src"), "img-fluid src")?;
            let name = required(first_attr_including_empty(&document, "img.img-fluid", "alt"), "img-fluid alt")?;
            Ok(resolved(name, image_url))
        }
        HostId::TurboImageHost => {
            let name = first_text(&document, "div.titleFullS > h1")
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| default_name(source_url));
            let image_url = required(first_attr_including_empty(&document, "img#imageid", "src"), "imageid src")?;
            Ok(resolved(name, image_url))
        }
        HostId::ViprIm => {
            let image_url = required(first_attr_including_empty(&document, "img[class*='img']", "src"), "image src")?;
            let name = required(first_attr_including_empty(&document, "img[class*='img']", "alt"), "image alt")?;
            Ok(HostResolution::Resolved(ResolvedImage {
                name,
                image_url,
                referer_override: Some("https://vipr.im/".into()),
                cookies: Vec::new(),
            }))
        }
    }
}

pub fn imx_page_url(source_url: &str) -> String {
    let normalized = source_url.replace("http:", "https:");
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);
    let id = filename
        .split('.')
        .next()
        .unwrap_or(filename)
        .replace("img-", "");
    format!("https://imx.to/i/{id}")
}

pub fn imx_direct_image_url(source_url: &str, thumbnail_url: &str) -> Result<String, String> {
    let _normalized_source = source_url.replace("http:", "https:");
    let thumb = thumbnail_url.replace("http:", "https:");
    let prefixes = [
        ("https://image.imx.to/u/t/", "https://image.imx.to/u/i/"),
        ("https://t.imx.to/t/", "https://image.imx.to/u/i/"),
        ("https://imx.to/upload/small/", "https://image.imx.to/u/i/"),
        ("https://i.imx.to/t/", "https://image.imx.to/u/i/"),
    ];
    for (prefix, replacement) in prefixes {
        if let Some(rest) = thumb.strip_prefix(prefix) {
            return Ok(format!("{replacement}{rest}"));
        }
    }
    if thumb.starts_with("https://imx.to/u/t") {
        return Ok(format!(
            "https://image.imx.to/u/i/{}",
            thumb.replace("https://imx.to/u/t/", "")
        ));
    }
    if thumb.starts_with("https://image.imx.to/u/i/") {
        return Ok(thumb);
    }
    Err(format!("no known Imx thumbnail pattern for {thumbnail_url}"))
}

pub fn pimpandhost_original_page_url(source_url: &str) -> Result<String, String> {
    let normalized = source_url
        .replace("http://", "https://");
    let medium = Regex::new(r"-medium(\.html)?").map_err(|error| error.to_string())?;
    let normalized = medium.replace_all(&normalized, "").into_owned();
    let (without_fragment, fragment) = match normalized.split_once('#') {
        Some((base, fragment)) => (base, Some(fragment)),
        None => (normalized.as_str(), None),
    };
    let (base, query) = match without_fragment.split_once('?') {
        Some((base, query)) => (base, Some(query)),
        None => (without_fragment, None),
    };
    let new_query = match query {
        Some(query) if !query.is_empty() => format!("{query}&size=original"),
        _ => "size=original".to_owned(),
    };
    let mut result = format!("{base}?{new_query}");
    if let Some(fragment) = fragment {
        result.push('#');
        result.push_str(fragment);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn fixture(name: &str) -> String {
        let path = format!("../../fixtures/hosts/{name}");
        fs::read_to_string(path).unwrap_or_else(|error| panic!("fixture {name}: {error}"))
    }

    fn resolved(result: HostResolution) -> ResolvedImage {
        match result {
            HostResolution::Resolved(image) => image,
            HostResolution::FollowUp(request) => panic!("expected resolved result, got {request:?}"),
        }
    }

    #[test]
    fn registry_has_all_sixteen_reference_hosts() {
        assert_eq!(HOST_REGISTRY.len(), 16);
        assert_eq!(host_for_url("https://pixhost.cc/show/1"), Some(HostId::Pixhost));
        assert_eq!(host_for_url("https://imagezilla.net/show/1"), Some(HostId::ImageZilla));
        assert_eq!(host_for_url("https://imagezilla.com/show/1"), None);
    }

    #[test]
    fn resolves_simple_host_fixtures() {
        let cases = [
            (HostId::DPicMe, "dpicme.html", "sample-dpic"),
            (HostId::ImageTwist, "imagetwist.html", "sample-twist"),
            (HostId::ImageZilla, "imagezilla.html", "sample-zilla"),
            (HostId::Imgbox, "imgbox.html", "sample-box"),
            (HostId::ImgSpice, "imgspice.html", "sample-spice"),
            (HostId::Pixhost, "pixhost.html", "sample-pixhost"),
            (HostId::PixRoute, "pixroute.html", "sample-route"),
            (HostId::Pixxxels, "pixxxels.html", "sample-pixxxels"),
            (HostId::PostImg, "postimg.html", "sample-postimg"),
            (HostId::TurboImageHost, "turboimagehost.html", "sample-turbo"),
            (HostId::ViprIm, "vipr.html", "sample-vipr"),
        ];
        for (host, file, expected_name) in cases {
            let html = fixture(file);
            let result = resolved(
                resolve_host_html(host, "https://example.test/show/fixture.jpg", "", &html, None, false)
                    .unwrap(),
            );
            assert_eq!(result.name, expected_name, "fixture {file}");
            assert!(!result.image_url.is_empty(), "fixture {file}");
        }
    }

    #[test]
    fn acidimg_returns_exact_continue_post_then_resolves_response() {
        let initial = fixture("acidimg-page.html");
        let request = resolve_host_html(
            HostId::Acidimg,
            "https://acidimg.cc/view/fixture",
            "",
            &initial,
            None,
            false,
        )
        .unwrap();
        match request {
            HostResolution::FollowUp(request) => {
                assert_eq!(request.method, FollowUpMethod::Post);
                assert_eq!(request.form_fields, vec![("imgContinue".into(), "Continue to your image".into())]);
                assert_eq!(request.referer.as_deref(), Some("https://acidimg.cc/view/fixture"));
            }
            other => panic!("expected follow-up, got {other:?}"),
        }

        let after = fixture("acidimg-after-continue.html");
        let image = resolved(
            resolve_host_html(
                HostId::Acidimg,
                "https://acidimg.cc/view/fixture",
                "",
                &initial,
                Some(&after),
                false,
            )
            .unwrap(),
        );
        assert_eq!(image.name, "sample-acid");
        assert_eq!(image.image_url, "https://acidimg.cc/images/sample-acid.jpg");
    }

    #[test]
    fn imagebam_returns_interstitial_cookies_and_resolves_followup() {
        let initial = fixture("imagebam-continue.html");
        let request = resolve_host_html(
            HostId::ImageBam,
            "https://imagebam.com/view/fixture",
            "",
            &initial,
            None,
            false,
        )
        .unwrap();
        match request {
            HostResolution::FollowUp(request) => {
                assert_eq!(request.method, FollowUpMethod::Get);
                assert_eq!(request.cookies.len(), 2);
                assert_eq!(request.cookies[0].name, "nsfw_inter");
                assert_eq!(request.cookies[1].name, "sfw_inter");
            }
            other => panic!("expected follow-up, got {other:?}"),
        }

        let after = fixture("imagebam-after-continue.html");
        let image = resolved(
            resolve_host_html(
                HostId::ImageBam,
                "https://imagebam.com/view/fixture",
                "",
                &initial,
                Some(&after),
                false,
            )
            .unwrap(),
        );
        assert_eq!(image.name, "sample-bam");
        assert_eq!(image.cookies.len(), 2);
    }

    #[test]
    fn imagevenue_refetches_continue_page_then_resolves_image() {
        let initial = fixture("imagevenue-page.html");
        let request = resolve_host_html(
            HostId::ImageVenue,
            "https://imagevenue.com/view/fixture",
            "",
            &initial,
            None,
            false,
        )
        .unwrap();
        assert!(matches!(request, HostResolution::FollowUp(_)));

        let after = fixture("imagevenue-after-continue.html");
        let image = resolved(
            resolve_host_html(
                HostId::ImageVenue,
                "https://imagevenue.com/view/fixture",
                "",
                &initial,
                Some(&after),
                false,
            )
            .unwrap(),
        );
        assert_eq!(image.name, "sample-venue");
    }

    #[test]
    fn imx_maps_thumbnail_patterns_and_optional_original_name() {
        assert_eq!(
            imx_direct_image_url(
                "https://imx.to/i/img-abc.jpg",
                "https://image.imx.to/u/t/abc.jpg"
            )
            .unwrap(),
            "https://image.imx.to/u/i/abc.jpg"
        );
        assert_eq!(
            imx_page_url("https://imx.to/i/img-abc.jpg"),
            "https://imx.to/i/abc"
        );

        let html = fixture("imx-title.html");
        let followup = "<html><body><div class='img-title'>Original title</div></body></html>";
        let request = resolve_host_html(
            HostId::Imx,
            "https://imx.to/i/img-abc.jpg",
            "https://image.imx.to/u/t/abc.jpg",
            &html,
            None,
            true,
        )
        .unwrap();
        assert!(matches!(request, HostResolution::FollowUp(_)));
        let image = resolved(
            resolve_host_html(
                HostId::Imx,
                "https://imx.to/i/img-abc.jpg",
                "https://image.imx.to/u/t/abc.jpg",
                &html,
                Some(followup),
                true,
            )
            .unwrap(),
        );
        assert_eq!(image.name, "Original title");
    }

    #[test]
    fn pimpandhost_preserves_query_and_fragment_when_adding_original_size() {
        assert_eq!(
            pimpandhost_original_page_url("http://pimpandhost.com/image-medium.html?x=1#view").unwrap(),
            "https://pimpandhost.com/image?x=1&size=original#view"
        );

        let initial = "<html><body>thumbnail page</body></html>";
        let request = resolve_host_html(
            HostId::Pimpandhost,
            "http://pimpandhost.com/image-medium.html?x=1",
            "",
            initial,
            None,
            false,
        )
        .unwrap();
        match request {
            HostResolution::FollowUp(request) => {
                assert_eq!(request.url, "https://pimpandhost.com/image?x=1&size=original");
                assert_eq!(request.method, FollowUpMethod::Get);
            }
            other => panic!("expected follow-up, got {other:?}"),
        }

        let after = fixture("pimpandhost.html");
        let image = resolved(
            resolve_host_html(
                HostId::Pimpandhost,
                "http://pimpandhost.com/image-medium.html?x=1",
                "",
                initial,
                Some(&after),
                false,
            )
            .unwrap(),
        );
        assert_eq!(image.name, "sample-pimp");
        assert_eq!(image.image_url, "https://cdn.pimpandhost.com/sample-pimp.jpg");
    }

    #[test]
    fn host_specific_referers_are_preserved() {
        let twist = resolved(
            resolve_host_html(
                HostId::ImageTwist,
                "https://imagetwist.com/view",
                "",
                &fixture("imagetwist.html"),
                None,
                false,
            )
            .unwrap(),
        );
        assert_eq!(twist.referer_override.as_deref(), Some("https://imagetwist.com/"));

        let vipr = resolved(
            resolve_host_html(
                HostId::ViprIm,
                "https://vipr.im/view",
                "",
                &fixture("vipr.html"),
                None,
                false,
            )
            .unwrap(),
        );
        assert_eq!(vipr.referer_override.as_deref(), Some("https://vipr.im/"));
    }
}
