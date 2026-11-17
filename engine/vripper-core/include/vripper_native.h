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


/* Coordinator API. Configuration and input arguments are UTF-8 JSON strings.
 * Handles are process-local and must be destroyed before application shutdown. */
char *vripper_coordinator_create_json(const char *config_json);
char *vripper_coordinator_scan_and_queue_json(
    uint64_t handle,
    const char *input,
    const char *configured_bases_json
);
char *vripper_coordinator_snapshot_json(uint64_t handle);
char *vripper_coordinator_stop_json(uint64_t handle, uint64_t job_id);
char *vripper_coordinator_stop_all_json(uint64_t handle);
char *vripper_coordinator_move_json(uint64_t handle, uint64_t job_id, const char *movement);
char *vripper_coordinator_clear_finished_json(uint64_t handle);
char *vripper_coordinator_poll_event_json(uint64_t handle, uint64_t timeout_ms);
char *vripper_coordinator_destroy_json(uint64_t handle);

/* Legacy database inspection, snapshot import and read-only catalog. */
char *vripper_database_inspect_json(const char *path);
char *vripper_database_import_json(const char *source, const char *destination);
char *vripper_database_catalog_json(const char *path);
char *vripper_native_catalog_json(const char *path, const char *data);

/* Release any non-NULL string returned by this library exactly once. */
void vripper_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif /* VRIPPER_NATIVE_H */
