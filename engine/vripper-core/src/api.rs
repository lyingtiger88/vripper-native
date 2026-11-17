//! Parser for the XML response consumed by the reference ThreadLookupAPIResponseHandler.

use crate::hosts::{host_for_url, HostId};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedImage {
    pub main_url: String,
    pub thumbnail_url: String,
    pub host: HostId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedHostCount {
    pub host: HostId,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedPost {
    pub thread_id: i64,
    pub thread_title: String,
    pub post_id: i64,
    pub post_number: i32,
    pub title: String,
    pub image_count: usize,
    pub url: String,
    pub hosts: Vec<ParsedHostCount>,
    pub security_token: String,
    pub forum: String,
    pub images: Vec<ParsedImage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedThread {
    pub thread_id: i64,
    pub title: String,
    pub security_token: String,
    pub forum: String,
    pub posts: Vec<ParsedPost>,
    pub error: String,
}

#[derive(Debug, Default)]
struct PostBuilder {
    id: i64,
    number: i32,
    title: String,
    images: Vec<ParsedImage>,
    hosts: Vec<ParsedHostCount>,
}

#[derive(Debug)]
struct ParserState {
    thread_id: i64,
    thread_title: String,
    forum: String,
    security_token: String,
    error: String,
    posts: Vec<ParsedPost>,
    current_post: Option<PostBuilder>,
}

impl Default for ParserState {
    fn default() -> Self {
        Self {
            thread_id: -1,
            thread_title: String::new(),
            forum: String::new(),
            security_token: String::new(),
            error: String::new(),
            posts: Vec::new(),
            current_post: None,
        }
    }
}

impl ParserState {
    fn start_element(&mut self, name: &str, attributes: &HashMap<String, String>) {
        match name {
            "error" => self.error = attributes.get("details").cloned().unwrap_or_default(),
            "thread" => {
                self.thread_id = parse_i64(attributes.get("id"), -1);
                self.thread_title = attributes.get("title").cloned().unwrap_or_default();
            }
            "forum" => self.forum = attributes.get("title").cloned().unwrap_or_default(),
            "user" => {
                self.security_token = attributes.get("hash").cloned().unwrap_or_default();
            }
            "post" => {
                let title = attributes.get("title").cloned().unwrap_or_default();
                self.current_post = Some(PostBuilder {
                    id: parse_i64(attributes.get("id"), -1),
                    number: parse_i32(attributes.get("number"), 0),
                    title: if title.trim().is_empty() {
                        self.thread_title.clone()
                    } else {
                        title
                    },
                    ..PostBuilder::default()
                });
            }
            "image" => self.add_image(attributes),
            _ => {}
        }
    }

    fn add_image(&mut self, attributes: &HashMap<String, String>) {
        let image_type = attributes.get("type").map(String::as_str).unwrap_or_default();
        if image_type != "linked" {
            return;
        }

        let main_url = attributes
            .get("main_url")
            .map(|url| url.replace("http://", "https://"))
            .unwrap_or_default();
        let thumbnail_url = attributes
            .get("thumb_url")
            .map(|url| url.replace("http://", "https://"))
            .unwrap_or_default();

        let Some(host) = host_for_url(&main_url) else {
            return;
        };
        let Some(post) = self.current_post.as_mut() else {
            return;
        };

        if let Some(count) = post.hosts.iter_mut().find(|entry| entry.host == host) {
            count.count += 1;
        } else {
            post.hosts.push(ParsedHostCount { host, count: 1 });
        }
        post.images.push(ParsedImage {
            main_url,
            thumbnail_url,
            host,
        });
    }

    fn end_element(&mut self, name: &str, site_proxy: &str) {
        if name != "post" {
            return;
        }

        let Some(post) = self.current_post.take() else {
            return;
        };
        let url = format!(
            "{site_proxy}/threads/{}?p={}&viewfull=1#post{}",
            self.thread_id, post.id, post.id
        );
        self.posts.push(ParsedPost {
            thread_id: self.thread_id,
            thread_title: self.thread_title.clone(),
            post_id: post.id,
            post_number: post.number,
            title: post.title,
            image_count: post.images.len(),
            url,
            hosts: post.hosts,
            security_token: self.security_token.clone(),
            forum: self.forum.clone(),
            images: post.images,
        });
    }

    fn finish(self) -> ParsedThread {
        ParsedThread {
            thread_id: self.thread_id,
            title: self.thread_title,
            security_token: self.security_token,
            forum: self.forum,
            posts: self.posts,
            error: self.error,
        }
    }
}

fn parse_i64(value: Option<&String>, fallback: i64) -> i64 {
    value
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(fallback)
}

fn parse_i32(value: Option<&String>, fallback: i32) -> i32 {
    value
        .and_then(|value| value.trim().parse::<i32>().ok())
        .unwrap_or(fallback)
}

fn read_attributes(
    event: &BytesStart<'_>,
    reader: &Reader<&[u8]>,
) -> Result<HashMap<String, String>, String> {
    let mut attributes = HashMap::new();
    for attribute in event.attributes().with_checks(false) {
        let attribute = attribute.map_err(|error| error.to_string())?;
        let key = String::from_utf8_lossy(attribute.key.as_ref()).to_ascii_lowercase();
        let value = attribute
            .decode_and_unescape_value(reader.decoder())
            .map_err(|error| error.to_string())?
            .into_owned();
        attributes.insert(key, value.trim().to_owned());
    }
    Ok(attributes)
}

/// Parse the thread/post/image XML using the reference SAX handler's semantics:
/// only type="linked" images with a registered host are included; HTTP URLs are
/// upgraded to HTTPS; source order and per-post host counts are retained.
pub fn parse_thread_response(xml: &str, site_proxy: &str) -> Result<ParsedThread, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut state = ParserState::default();

    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Start(event) => {
                let name = String::from_utf8_lossy(event.name().as_ref()).to_ascii_lowercase();
                let attributes = read_attributes(&event, &reader)?;
                state.start_element(&name, &attributes);
            }
            Event::Empty(event) => {
                let name = String::from_utf8_lossy(event.name().as_ref()).to_ascii_lowercase();
                let attributes = read_attributes(&event, &reader)?;
                state.start_element(&name, &attributes);
                state.end_element(&name, site_proxy);
            }
            Event::End(event) => {
                let name = String::from_utf8_lossy(event.name().as_ref()).to_ascii_lowercase();
                state.end_element(&name, site_proxy);
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(state.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_thread_fixture_with_reference_filtering_and_order() {
        let xml = fs::read_to_string("../../fixtures/api/thread-response.xml").unwrap();
        let thread = parse_thread_response(&xml, "https://vipergirls.to").unwrap();

        assert_eq!(thread.thread_id, 10001);
        assert_eq!(thread.title, "Compatibility Fixture Thread");
        assert_eq!(thread.security_token, "fixture-token-not-valid");
        assert_eq!(thread.forum, "Fixture Forum");
        assert_eq!(thread.posts.len(), 2);

        let first = &thread.posts[0];
        assert_eq!(first.post_id, 20001);
        assert_eq!(first.post_number, 1);
        assert_eq!(first.title, "First fixture post");
        assert_eq!(first.image_count, 2);
        assert_eq!(
            first.url,
            "https://vipergirls.to/threads/10001?p=20001&viewfull=1#post20001"
        );
        assert_eq!(first.images[0].main_url, "https://imagebam.com/view/sample01");
        assert_eq!(first.images[0].thumbnail_url, "https://thumb.imagebam.com/sample01");
        assert_eq!(first.images[0].host, HostId::ImageBam);
        assert_eq!(first.images[1].host, HostId::Pixhost);
        assert_eq!(first.hosts.len(), 2);

        let second = &thread.posts[1];
        assert_eq!(second.post_id, 20002);
        assert_eq!(second.title, "Compatibility Fixture Thread");
        assert_eq!(second.image_count, 1);
        assert_eq!(second.images[0].host, HostId::Imgbox);
    }

    #[test]
    fn retains_post_metadata_even_without_supported_linked_images() {
        let xml = r#"<thread id="1" title="T"><post id="2" number="1" title="Metadata only"><image type="unlinked" main_url="https://imgbox.com/x"/><image type="linked" main_url="https://unknown.invalid/x"/></post></thread>"#;
        let thread = parse_thread_response(xml, "https://vipergirls.to").unwrap();
        assert_eq!(thread.posts.len(), 1);
        assert_eq!(thread.posts[0].post_id, 2);
        assert_eq!(thread.posts[0].title, "Metadata only");
        assert_eq!(thread.posts[0].image_count, 0);
        assert!(thread.posts[0].images.is_empty());
    }

    #[test]
    fn preserves_empty_response_error() {
        let xml = r#"<thread id="1" title="T"><error details="fixture error"/></thread>"#;
        let thread = parse_thread_response(xml, "https://vipergirls.to").unwrap();
        assert_eq!(thread.error, "fixture error");
        assert!(thread.posts.is_empty());
    }
}
