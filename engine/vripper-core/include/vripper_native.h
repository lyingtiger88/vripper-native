#ifndef VRIPPER_NATIVE_H
#define VRIPPER_NATIVE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ABI version 1. Returned strings are owned by Rust. */
uint32_t vripper_abi_version(void);
char *vripper_engine_version(void);

/* input and configured_bases_json are UTF-8 NUL-terminated strings.
 * configured_bases_json must be a JSON array of strings. */
char *vripper_scan_links_json(const char *input, const char *configured_bases_json);

/* Parse a captured thread lookup XML response into a JSON model. */
char *vripper_parse_thread_response_json(const char *xml, const char *site_proxy);

/* Resolve captured host HTML. follow_up_html may be NULL. host_id is the
 * reference numeric host ID (0..15); fetch_original_imx_filename is 0 or 1. */
char *vripper_resolve_host_html_json(
    uint8_t host_id,
    const char *source_url,
    const char *thumbnail_url,
    const char *html,
    const char *follow_up_html,
    uint8_t fetch_original_imx_filename
);

/* Release any non-NULL string returned by this library exactly once. */
void vripper_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif /* VRIPPER_NATIVE_H */
