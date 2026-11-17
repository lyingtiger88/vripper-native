//! VRipper Native compatibility engine.
//!
//! This first vertical slice captures deterministic behavior from the reference:
//! link scanning, post folder naming, image filename normalization and output paths.
//! Network access and live host resolution are deliberately not implemented until
//! authorized fixtures and behavior tests are available.

pub mod api;
pub mod hosts;
pub mod ffi;

use regex::Regex;
use serde::Serialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ThreadLink {
    pub base_url: String,
    pub thread_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostLink {
    pub base_url: String,
    pub thread_id: u64,
    pub post_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    let normalized = format!("{}.{}", sanitize_reference_name(base), mime.extension());

    if force_order {
        format!("{:03}_{normalized}", zero_based_index + 1)
    } else {
        normalized
    }
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
