# UI Behavior Audit — Detailed Source Findings

Reference: `dev-claw/vripper-project`, branch `main`. Static source review only; pixel-perfect acceptance requires reference screenshots and interaction recordings.

## Main application shell

- `AppView` composition order: MenuBar → optional ActionBar → RemoteErrorBanner → MainView tabs → optional StatusBar.
- Main window dimensions and title are restored/derived from application settings and version.
- Root drag/drop accepts string payloads; URI-like links are sent to link scanning.
- Theme, dark mode, accent, font family and font size are applied through the theme manager.
- Main tabs are non-closable: Posts (`download-tab`), Threads (`thread-tab`), Tasks, Logs.

## Posts — three presentation modes

### Shared interaction contract
- Multi-selection is supported in Table, List and Grid.
- Double-click/activate opens the post overview workspace.
- Context menu is available per post.
- Delete key removes selected posts.
- Selection is synchronized across view modes and controller state.
- Hover preview popup can be enabled/disabled in settings.
- Empty/loading/connection-failure states are distinct.
- Search is shared from the action bar and matches title, ID, host, status or path.

### Table mode
Columns in source order:
1. # (unassigned rank renders `*`)
2. Preview (hover shows up to four preview images)
3. Title
4. Progress
5. Status
6. Automation
7. Path
8. Total
9. Hosts
10. Added On

- Column visibility and width are persisted independently.
- Added On defaults to descending and is included in sort order.
- Table supports native column menu and multiple selection.
- Progress cell has custom click/selection handling.

### List mode
- Row-based, expanded preview/metadata layout.
- Multiple selection, double-click open, right-click selects row and shows context menu.
- Delete key removes selected posts.
- Hover preview popup.
- Font size changes row height.

### Grid mode
- Virtualized grid to avoid materializing all tiles at once.
- Multiple selection, activation opens overview, per-tile context menu.
- Configurable tile size and hover preview popup.
- Selection state synchronizes with controller.
- Scrollable viewport and empty placeholder.

## Threads
- Header controls: Clear threads and quick filter pills for post count.
- Search and quick filter are applied together.
- Table columns: Title, URL, Count.
- Multi-selection.
- Double-click selects posts belonging to that thread.
- Context actions: Select posts, Open link, Delete.
- Delete key removes selected threads.
- Column visibility/width persisted.

## Tasks
- Header controls: Clear tasks, Group selector, Status quick filters.
- Search matches task name, description, category, status and result.
- Group modes: None, Category, Post.
- Status quick filters: All, Active (WAITING/RUNNING), Success, Issues (WARNING/FAILURE).
- Tasks render as cards/messages with status accent and Dismiss action.
- Category enum values: METADATA, THANKS, QUEUE, RENAME, SCAN, THREAD, AUTOMATION.
- Status enum values: WAITING, RUNNING, SUCCESS, FAILURE, WARNING.

## Logs
- Header controls: Clear logs and Level quick filters.
- Search and level filter are combined.
- Table columns: Time, Thread, Logger, Level, Message.
- Time defaults to descending.
- Double-click opens full log message.
- Column visibility/width persisted.
- Empty state says "No logs yet"; connection failure and loading have separate placeholders.

## Action bar
Source: `ActionBarView.kt`
- Brand/logo region.
- Search field prompt: "Search posts"; tooltip says it filters by title, ID, host, status or path.
- Add links button (Ctrl+L).
- Start All (Ctrl+S).
- Stop All (Ctrl+Q).
- Clear finished posts (Ctrl+Delete), with confirmation.
- Settings (Ctrl+P).
- Start/Stop enabled state depends on active download count.
- Search is debounced/connected to Posts controller; full exact delay and empty-state behavior should be verified in source/runtime.

## File menu and keyboard actions
Source: `MenuBarView.kt`
- File → Add links (Ctrl+L)
- File → Start All (Ctrl+S)
- File → Stop All (Ctrl+Q)
- File → Clear (Ctrl+Delete)
- File → Settings (Ctrl+P)
- File → Change session (Shift+S)
- File → Exit (Ctrl+X)
- View menu includes Toolbar (F6) and Info Panel (F7); remaining menu items require further inspection.
- Preserve exact accelerator behavior and conflict handling.

## Add Links
- Multiline input.
- Scan action disabled for blank input.
- Submit runs asynchronously; dialog closes after request completion.
- Scanner splits by newline, trims whitespace, ignores blank entries.
- Accepts configured Viper base/proxy prefixes and parses thread ID plus optional post ID.
- Invalid links are reported, not silently dropped.

## Settings window
- Nine original sections, exact control inventory in `SETTINGS_INVENTORY.md`.
- Sidebar preferred width 205px, min 185px, max 230px.
- Scrollable content area and footer Save.
- Save validates service settings; client-side Posts/Viewer/Overview preferences commit only after successful save.
- New Network section is additive; do not change existing Connection or Viper semantics.

## Image overview / viewer
- Post workspace opened from post item.
- Overview supports Grid/Table layout, tile size and details panel.
- Single-image mode has image index/navigation, zoom and deep zoom, minimap.
- Image loading asynchronous with byte/decode preloading and encoded/decoded caches.
- Disk/decoded cache sizes and preload concurrency are configurable.
- Workspace window dimensions, position, maximized state and divider position persist.
- Exact mouse gestures, keyboard shortcuts, zoom increments, image fit modes and window actions still need runtime recording.

## Source snapshots

| File | Blob SHA |
|---|---|
| PostsTableView.kt | `71e54fda8779200bbb61b2c5d12bc6324158e798` |
| PostsGridView.kt | `277eadad5ff25c127893bc89256df8310be008e1` |
| PostsListView.kt | `1b84dcf37d532fe05cd6f930c2aa5592a139c57c` |
| ThreadTableView.kt | fetched in audit; verify current SHA before port |
| TaskTableView.kt | fetched in audit; verify current SHA before port |
| LogTableView.kt | fetched in audit; verify current SHA before port |
| ActionBarView.kt | `603b3df86ecf0fc88cc273770d9e5ebfb866003d` |
| MenuBarView.kt | `0a41bda4b09ce176717c72a9e0453a3cc0c47070` |
| SettingsFragment.kt | `b6b2c527ec3b86e196f3df0602df58d5db5fa046` |

## Outstanding visual and behavioral capture checklist

- [ ] Capture main window at reference default size and theme.
- [ ] Capture Posts table/list/grid and all column visibility combinations relevant to defaults.
- [ ] Capture selected, multi-selected, hover, context-menu, empty, loading and error states.
- [ ] Record exact context menu items and ordering for Posts.
- [ ] Capture Threads, Tasks and Logs states and quick filters.
- [ ] Record all menu entries and accelerators.
- [ ] Capture every Settings section at default values.
- [ ] Capture Add Links validation and result states.
- [ ] Capture overview workspace, details panel, image viewer and minimap.
- [ ] Record mouse and keyboard interactions for image navigation/zoom.
- [ ] Verify window geometry restore and optional toolbar/status panel behavior.


## Posts quick filters and context menu

Posts quick filters:
- All
- Active (PENDING or DOWNLOADING)
- Done (FINISHED)
- Automation (AWAITING_REVIEW, READY, RUNNING or FAILED)
- Paused (STOPPED)
- Error (ERROR)

Search matches title, external post ID, thread ID, host text, status, automation action state and path. Search and quick filter are combined.

Post context menu order:
1. Start
2. Stop
3. Rename
4. Auto rename (bulk rename selected to first title)
5. Delete
6. Separator
7. Open containing folder (visible only in local session)
8. Open link
9. Open overview
10. Separator
11. Move to top
12. Move up
13. Move down
14. Move to bottom

Selection behavior: right-clicking an unselected row/cell selects that item before showing its context menu. Multi-selection is preserved when right-clicking an already selected item.

## Post details panel

The overview details panel has General and Automation sections.

General fields:
- Post Link
- Posted By
- Title
- More titles
- Status
- Path
- Total/progress
- Hosts
- Added On
- Downloads controls: Start and Stop

Automation fields:
- Automation state
- Missing fields
- Action error
- Custom fields editor (one `name=value` per line)
- Save fields
- Run action
- Retry action
- Skip action

## Image viewer interaction contract

Toolbar order:
1. Overview toggle
2. View toggle
3. Previous image
4. Next image
5. Fit
6. Zoom out
7. Zoom slider
8. Zoom in
9. 100% fill

- Left/Right arrows: previous/next image; in overview they move the highlighted thumbnail.
- Home/End: first/last image (or thumbnail selection).
- Plus/Add/Equals: zoom in.
- Minus/Subtract: zoom out.
- Digit 0: fit-to-window in viewer; reset thumbnail size in overview.
- Scroll up/down over the image: previous/next image; toolbar scroll is not intercepted.
- Dragging/panning suspends speculative preloading until mouse release.
- View mode displays the downloaded file from local disk or the server stream; it does not display the web-hosted original. Overview thumbnails may use remote thumbnail URLs.
- Status line shows image name, dimensions, format and byte size; center label shows current index/total or photo count; source badge distinguishes local, server, not-downloaded and missing/moved states.
- Loading bar is delayed 300ms to avoid flashing on cached loads.
- Resize debounce is 250ms; decode target grows with device-pixel scale and cache is rebuilt after resize settles.

These are source-level findings. Mouse wheel direction, zoom step value, gestures and exact minimap behavior still need runtime capture.
