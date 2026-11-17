# Reference UI Inventory — Initial Pass

This is a source-level inventory, not a claim of pixel-perfect visual capture. Reference screenshots and runtime interaction recordings are still required before UI parity can be signed off.

## Application shell
Source: vripper-gui/.../components/views/AppView.kt

Composition order:
1. MenuBarView
2. ActionBarView (optional; visibility preference)
3. RemoteErrorBanner
4. MainView
5. StatusBarView (optional; visibility preference)

Additional shell behavior:
- Window title includes VRipper version.
- Initial width/height are restored from current settings.
- Root accepts drag-over of string payloads and URI-like strings are scanned on drop.
- Theme manager attaches to the scene; theme, dark mode, accent color, font family and font size changes are applied live.

## Primary navigation
Source: vripper-gui/.../components/views/MainView.kt

Non-closable tabs:
| Tab | Source view | Stable UI id |
|---|---|---|
| Posts | PostsView | download-tab |
| Threads | ThreadTableView | thread-tab |
| Tasks | TaskTableView | task tab |
| Logs | LogTableView | log tab |

Icons use Feather icon glyphs: download, file-text, activity and database respectively.

## Posts workspace
Source files include:
- PostsView.kt
- PostsTableView.kt
- PostsGridView.kt
- PostsListView.kt
- PostsPlaceholder.kt
- PostsTabController.kt
- PostController.kt

Observed behaviors from inspected table/list source:
- Multi-selection.
- Double-click opens post overview/workspace.
- Context menu is built per post row/cell.
- Delete key removes selected posts.
- Search field in action toolbar has prompt "Search posts" and filters by title, ID, host, status or path.
- Add links is available from toolbar, File menu and posts context menu.
- Table view supports user-visible columns and column-width preferences.
- List/grid/table presentation modes exist and must be inventoried for exact selector/behavior.

## Menu and action bar
Sources:
- MenuBarView.kt
- ActionBarView.kt

Observed File menu actions include:
- Add links — Ctrl+L
- Start All — Ctrl+S
- Stop All — Ctrl+Q
- Clear finished posts — Ctrl+Delete
- Settings — Ctrl+P

Action bar includes brand area, Search posts, Add links, Start All, Stop All and Clear. It also displays or controls other action items that require full-source inspection.

## Add Links dialog
Source: AddLinksFragment.kt

- Multiline text area.
- Scan button.
- Empty input disables Scan.
- Input is submitted asynchronously.
- Dialog closes after scan request completes.

## Status bar
Source: StatusBarView.kt

Source-level state includes:
- Remote/local connection state and indicator.
- Logged-in user.
- Running/pending/error counts.
- Download speed.
- Running task name/detail, with a short linger before hiding.
- Server version tracking.

Exact visual ordering, labels, number formatting, tooltips and interaction details remain to be captured.

## Image workspace
Sources:
- PostWorkspaceFragment.kt
- PostDetailsPane.kt
- ImageCanvasPane.kt
- ImageLoadPipeline.kt
- DeepZoomController.kt
- ViewZoomController.kt
- WindowGeometryController.kt
- DecodedImageCache.kt

Observed feature responsibilities:
- Opens from post list; can start in overview mode.
- Maintains image index and source list.
- Overview/browse mode and single image mode.
- Asynchronous load and speculative preload.
- Encoded/decoded cache with memory bounds.
- Deep zoom and fit/zoom controls.
- Minimap viewport.
- Workspace window geometry persistence.
- Refreshes download state and status bar labels.

## Settings inventory — sections identified
The fragment tree contains:
- AppearanceSettingsFragment
- AutomationSettingsFragment
- ConnectionSettingsFragment
- DownloadSettingsFragment
- HostSettingsFragment
- PostsSettingsFragment
- SettingsFragment
- SystemSettingsFragment
- ViewersSettingsFragment
- ViperSettingsFragment

Each section's individual controls, defaults, validation, persistence and dependencies still require a line-by-line inventory.

## Required visual capture set
1. Main window in default state.
2. Posts table, grid and list modes.
3. Empty, loading, active, completed, stopped and error states.
4. Add Links dialog, invalid-link result and task progress.
5. Threads, Tasks and Logs tabs.
6. Every settings section, including validation/error states.
7. Remote connection failure banner and local/remote session states.
8. Image workspace overview and single-image view.
9. Zoom, deep zoom, minimap, next/previous image and refresh behavior.
10. Window resize/restore and optional toolbar/statusbar states.

## UI parity rule
Flutter implementation must not reinterpret the design. Use reference captures and scripted actions as the acceptance oracle. If a control cannot be represented identically on mobile, document a platform adaptation while preserving the same action and result.


## New requested section: Network
A separate **Network** item must be added to the Settings sidebar. It is distinct from the existing **Connection** section, which continues to control concurrency, timeout and retry settings. Network settings provide proxy mode selection (Disabled / System / Manual), manual HTTP/HTTPS/SOCKS5 endpoint and optional authentication, bypass list, Test connection feedback and secure credential handling. See `NETWORK_SETTINGS.md` for the complete behavior and acceptance criteria.
