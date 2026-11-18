//! VRipper Native compatibility engine.
//!
//! Native compatibility engine: deterministic parsing, host resolution, shared
//! proxy-aware HTTP transport, streamed image downloads and bounded queue execution.
//! Automated tests use synthetic fixtures and local mock servers; no live site
//! scraping or access-control bypass is performed by the test suite.

pub mod api;
pub mod hosts;
pub mod network;
pub mod transport;
pub mod pipeline;
pub mod queue;
pub mod viper_api;
pub mod viper_auth;
pub mod coordinator;
pub mod ffi;
pub mod database;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadLink {
    pub base_url: String,
    pub thread_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostLink {
    pub base_url: String,
    pub thread_id: u64,
    pub post_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScanResult {
    pub threads: Vec<ThreadLink>,
    pub posts: Vec<PostLink>,
    pub invalid: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CompatibilityError {
    #[error("invalid numeric thread or post identifier")]
    InvalidIdentifier,
    #[error("unsupported image MIME type")]
    UnsupportedImageMime,
}

/// Mirrors the reference scanner's newline splitting, trimming, prefix selection,
/// regex search semantics and thread/post classification.
pub fn scan_links(input: &str, configured_bases: &[String]) -> ScanResult {
    let mut result = ScanResult::default();

    for raw in input.split('\n') {
        let link = raw.trim().trim_end_matches('\r');
        if link.is_empty() {
            continue;
        }

        let Some(base) = configured_bases
            .iter()
            .find(|base| link.starts_with(base.as_str()))
        else {
            result.invalid.push(link.to_owned());
            continue;
        };

        let pattern = format!(
            "{}(/threads/(\\d+)((.*p=)(\\d+))?)",
            regex::escape(base)
        );
        let regex = match Regex::new(&pattern) {
            Ok(regex) => regex,
            Err(_) => {
                result.invalid.push(link.to_owned());
                continue;
            }
        };

        let Some(captures) = regex.captures(link) else {
            result.invalid.push(link.to_owned());
            continue;
        };

        let thread_id = match captures
            .get(2)
            .and_then(|capture| capture.as_str().parse::<u64>().ok())
        {
            Some(id) => id,
            None => {
                result.invalid.push(link.to_owned());
                continue;
            }
        };

        if let Some(post_match) = captures.get(5) {
            match post_match.as_str().parse::<u64>() {
                Ok(post_id) => result.posts.push(PostLink {
                    base_url: base.clone(),
                    thread_id,
                    post_id,
                }),
                Err(_) => result.invalid.push(link.to_owned()),
            }
        } else {
            result.threads.push(ThreadLink {
                base_url: base.clone(),
                thread_id,
            });
        }
    }

    result
}

/// Reference PathUtils sanitizer: replace dot, slash, backslash, pipe, colon,
/// question mark, asterisk, quote, angle brackets and control characters with '_'.
pub fn sanitize_reference_name(value: &str) -> String {
    let invalid = Regex::new(r#"[.\\/|:?*"<>\p{Cc}]"#).expect("static sanitizer regex");
    invalid.replace_all(value, "_").into_owned()
}

/// DataAccessService.buildPosts folderName rule.
pub fn post_folder_name(post_title: &str, post_id: u64, append_post_id: bool) -> String {
    let title = sanitize_reference_name(post_title);
    if append_post_id {
        format!("{title}_{post_id}")
    } else {
        title
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageMime {
    Bmp,
    Gif,
    Jpeg,
    Png,
    Webp,
    Svg,
}

impl ImageMime {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Bmp => "BMP",
            Self::Gif => "GIF",
            Self::Jpeg => "JPG",
            Self::Png => "PNG",
            Self::Webp => "WEBP",
            Self::Svg => "SVG",
        }
    }
}

/// Mirrors ImageDownloadRunnable.checkImageTypeAndRename. Note SVG is accepted
/// as a response MIME but is not in the reference's recognized input-extension list.
pub fn final_image_filename(
    supplied_name: &str,
    mime: ImageMime,
    zero_based_index: usize,
    force_order: bool,
) -> String {
    let extension = supplied_name
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())
        .filter(|extension| {
            ["bmp", "gif", "jpg", "jpeg", "png", "webp"].contains(&extension.as_str())
        });

    let base = if extension.is_some() {
        supplied_name
            .rsplit_once('.')
            .map(|(base, _)| base)
            .unwrap_or(supplied_name)
    } else {
        supplied_name
    };
    let sanitized_base = sanitize_reference_name(base);
    let reserved_stem = sanitized_base
        .split('.')
        .next()
        .unwrap_or(&sanitized_base)
        .to_ascii_uppercase();
    let safe_base = if matches!(reserved_stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (reserved_stem.len() == 4
            && (reserved_stem.starts_with("COM") || reserved_stem.starts_with("LPT"))
            && reserved_stem.as_bytes()[3].is_ascii_digit()
            && reserved_stem.as_bytes()[3] != b'0')
    {
        format!("_{sanitized_base}")
    } else {
        sanitized_base
    };
    let normalized = format!("{safe_base}.{}", mime.extension());

    if force_order {
        format!("{:03}_{normalized}", zero_based_index + 1)
    } else {
        normalized
    }
}

/// Reject path components that could escape the configured download root or
/// behave inconsistently across Windows, Linux and Android filesystems.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum UnsafeDownloadPath {
    #[error("unsafe download path component")]
    UnsafeComponent,
}

/// Validate one remote-controlled directory component without rewriting its
/// visible name. This intentionally fails closed instead of silently changing
/// a reference title that contains path syntax.
pub fn validate_download_component(value: &str) -> Result<(), UnsafeDownloadPath> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.ends_with('.')
        || value.ends_with(' ')
        || value.chars().any(|ch| ch == '/' || ch == '\\' || ch == ':' || ch.is_control())
    {
        return Err(UnsafeDownloadPath::UnsafeComponent);
    }

    let stem = value.split('.').next().unwrap_or(value).trim_end_matches([' ', '.']);
    let upper = stem.to_ascii_uppercase();
    if matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.as_bytes()[3].is_ascii_digit()
            && upper.as_bytes()[3] != b'0')
    {
        return Err(UnsafeDownloadPath::UnsafeComponent);
    }

    let mut components = Path::new(value).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(UnsafeDownloadPath::UnsafeComponent);
    }
    Ok(())
}

/// Safe counterpart to calculate_download_directory. Forum names follow the
/// reference sanitizer; remote thread titles must be a single portable path
/// component or the operation is rejected.
pub fn safe_calculate_download_directory(
    root: &Path,
    forum: &str,
    thread_title: &str,
    forum_subdirectory: bool,
    thread_sublocation: bool,
) -> Result<PathBuf, UnsafeDownloadPath> {
    let mut output = root.to_path_buf();
    if forum_subdirectory {
        let forum_component = sanitize_reference_name(forum);
        validate_download_component(&forum_component)?;
        output.push(forum_component);
    }
    if thread_sublocation {
        validate_download_component(thread_title)?;
        output.push(thread_title);
    }
    Ok(output)
}

/// Mirrors PathUtils.calculateDownloadPath for the observed rules.
/// Forum names are sanitized; thread titles are appended as-is in the source.
pub fn calculate_download_directory(
    root: &Path,
    forum: &str,
    thread_title: &str,
    forum_subdirectory: bool,
    thread_sublocation: bool,
) -> PathBuf {
    let mut output = root.to_path_buf();
    if forum_subdirectory {
        output.push(sanitize_reference_name(forum));
    }
    if thread_sublocation {
        output.push(thread_title);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bases() -> Vec<String> {
        vec![
            "https://vipergirls.to".to_owned(),
            "https://www.vipergirls.to".to_owned(),
        ]
    }

    #[test]
    fn scans_thread_and_post_links_and_ignores_blank_lines() {
        let result = scan_links(
            "  https://vipergirls.to/threads/12345\nhttps://vipergirls.to/threads/12345?p=67890\n\n",
            &bases(),
        );

        assert_eq!(
            result.threads,
            vec![ThreadLink {
                base_url: "https://vipergirls.to".into(),
                thread_id: 12345,
            }]
        );
        assert_eq!(
            result.posts,
            vec![PostLink {
                base_url: "https://vipergirls.to".into(),
                thread_id: 12345,
                post_id: 67890,
            }]
        );
        assert!(result.invalid.is_empty());
    }

    #[test]
    fn scanner_preserves_invalid_link_text_after_trimming() {
        let result = scan_links(
            "https://not-configured.example/threads/1\nhttps://vipergirls.to/not-a-thread\n",
            &bases(),
        );

        assert_eq!(
            result.invalid,
            vec![
                "https://not-configured.example/threads/1",
                "https://vipergirls.to/not-a-thread"
            ]
        );
    }


    #[test]
    fn scanner_finds_post_id_after_other_query_parameters_and_fragment() {
        let result = scan_links(
            "https://vipergirls.to/threads/12345?view=unread&p=67890#post67890",
            &bases(),
        );
        assert_eq!(result.posts.len(), 1);
        assert_eq!(result.posts[0].thread_id, 12345);
        assert_eq!(result.posts[0].post_id, 67890);
        assert!(result.threads.is_empty());
        assert!(result.invalid.is_empty());
    }

    #[test]
    fn scanner_rejects_overflowing_thread_and_post_identifiers() {
        let result = scan_links(
            "https://vipergirls.to/threads/999999999999999999999999\nhttps://vipergirls.to/threads/7?p=999999999999999999999999",
            &bases(),
        );
        assert_eq!(result.invalid.len(), 2);
        assert!(result.threads.is_empty());
        assert!(result.posts.is_empty());
    }

    #[test]
    fn scanner_uses_first_matching_configured_base() {
        let bases = vec![
            "https://vipergirls.to".to_owned(),
            "https://vipergirls.to/threads".to_owned(),
        ];
        let result = scan_links("https://vipergirls.to/threads/7", &bases);

        assert_eq!(result.threads.len(), 1);
        assert_eq!(result.threads[0].base_url, "https://vipergirls.to");
    }

    #[test]
    fn post_folder_name_matches_reference_option() {
        assert_eq!(
            post_folder_name("A: title / one", 42, false),
            "A_ title _ one"
        );
        assert_eq!(
            post_folder_name("A: title / one", 42, true),
            "A_ title _ one_42"
        );
    }

    #[test]
    fn final_filename_uses_mime_extension_and_optional_order_prefix() {
        assert_eq!(
            final_image_filename("my.image.jpeg", ImageMime::Png, 0, true),
            "001_my_image.PNG"
        );
        assert_eq!(
            final_image_filename("no-extension", ImageMime::Jpeg, 8, false),
            "no-extension.JPG"
        );
    }

    #[test]
    fn svg_input_extension_is_not_recognized_by_reference_extension_list() {
        assert_eq!(
            final_image_filename("vector.svg", ImageMime::Svg, 0, false),
            "vector_svg.SVG"
        );
    }


    #[test]
    fn sanitizer_replaces_every_reference_forbidden_character() {
        let input = "a.b/c\\d|e:f?g*h\"i<j>k\\n";
        assert_eq!(sanitize_reference_name(input), "a_b_c_d_e_f_g_h_i_j_k_n");
        assert_eq!(sanitize_reference_name("control\u{0001}tail"), "control_tail");
    }

    #[test]
    fn folder_name_preserves_unicode_and_only_appends_post_id_when_enabled() {
        assert_eq!(post_folder_name("عنوان فارسی — 東京", 987654, false), "عنوان فارسی — 東京");
        assert_eq!(post_folder_name("عنوان فارسی — 東京", 987654, true), "عنوان فارسی — 東京_987654");
    }

    #[test]
    fn filename_extension_is_derived_from_detected_mime_type() {
        let cases = [
            ("photo.bmp", ImageMime::Jpeg, "photo.JPG"),
            ("photo.gif", ImageMime::Png, "photo.PNG"),
            ("photo.jpeg", ImageMime::Webp, "photo.WEBP"),
            ("photo.JPG", ImageMime::Gif, "photo.GIF"),
            ("photo.webp", ImageMime::Bmp, "photo.BMP"),
            ("photo", ImageMime::Svg, "photo.SVG"),
        ];
        for (supplied, mime, expected) in cases {
            assert_eq!(final_image_filename(supplied, mime, 0, false), expected);
        }
    }

    #[test]
    fn filename_avoids_windows_reserved_device_names() {
        assert_eq!(final_image_filename("CON.jpg", ImageMime::Jpeg, 0, false), "_CON.JPG");
        assert_eq!(final_image_filename("nul.png", ImageMime::Png, 0, false), "_nul.PNG");
        assert_eq!(final_image_filename("COM1.webp", ImageMime::Webp, 0, false), "_COM1.WEBP");
    }

    #[test]
    fn force_order_uses_one_based_three_digit_prefix() {
        assert_eq!(final_image_filename("a.jpg", ImageMime::Jpeg, 0, true), "001_a.JPG");
        assert_eq!(final_image_filename("a.jpg", ImageMime::Jpeg, 8, true), "009_a.JPG");
        assert_eq!(final_image_filename("a.jpg", ImageMime::Jpeg, 99, true), "100_a.JPG");
        assert_eq!(final_image_filename("a.jpg", ImageMime::Jpeg, 999, true), "1000_a.JPG");
    }

    #[test]
    fn directory_options_are_independent_and_keep_root_when_disabled() {
        let root = Path::new("/downloads");
        assert_eq!(
            calculate_download_directory(root, "Forum: A", "Thread: B", false, false),
            PathBuf::from("/downloads")
        );
        assert_eq!(
            calculate_download_directory(root, "Forum: A", "Thread: B", true, false),
            PathBuf::from("/downloads/Forum_ A")
        );
        assert_eq!(
            calculate_download_directory(root, "Forum: A", "Thread: B", false, true),
            PathBuf::from("/downloads/Thread: B")
        );
    }

    #[test]
    fn safe_download_path_rejects_traversal_and_windows_special_components() {
        for unsafe_name in ["..", "../escape", "..\\\\escape", "/absolute", "C:\\\\escape", "CON", "NUL.txt", "name.", "name "] {
            assert_eq!(validate_download_component(unsafe_name), Err(UnsafeDownloadPath::UnsafeComponent), "{unsafe_name}");
        }
    }

    #[test]
    fn safe_download_component_rejects_reserved_post_folder_names() {
        for name in ["CON", "PRN", "AUX", "NUL", "COM1", "LPT9", "trailing.", "trailing "] {
            assert_eq!(
                validate_download_component(name),
                Err(UnsafeDownloadPath::UnsafeComponent),
                "{name}"
            );
        }
        assert!(validate_download_component("normal folder — فارسی").is_ok());
    }

    #[test]
    fn safe_download_path_accepts_unicode_and_keeps_root_contained() {
        let path = safe_calculate_download_directory(
            Path::new("/downloads"),
            "Forum: A",
            "عنوان فارسی — 東京",
            true,
            true,
        ).unwrap();
        assert_eq!(path, PathBuf::from("/downloads/Forum_ A/عنوان فارسی — 東京"));
    }

    #[test]
    fn safe_download_path_rejects_unsafe_remote_thread_title() {
        let result = safe_calculate_download_directory(
            Path::new("/downloads"),
            "Forum",
            "../../outside",
            true,
            true,
        );
        assert_eq!(result, Err(UnsafeDownloadPath::UnsafeComponent));
    }

    #[test]
    fn directory_sanitizes_forum_but_preserves_thread_title() {
        let path = calculate_download_directory(
            Path::new("/downloads"),
            "Forum: A",
            "Thread: B",
            true,
            true,
        );

        assert_eq!(
            path,
            PathBuf::from("/downloads/Forum_ A/Thread: B")
        );
    }
}
