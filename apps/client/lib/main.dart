// ignore: deprecated_member_use
import 'package:flutter/material.dart';
import 'native_engine.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  NativeEngine.instance.load();
  runApp(const VripperApp());
}

const _ink = Color(0xFF263238);
const _muted = Color(0xFF607D8B);
const _line = Color(0xFFE1E6EA);
const _surface = Color(0xFFF7F8FA);
const _accent = Color(0xFF2878C8);

class VripperApp extends StatelessWidget {
  const VripperApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
        title: 'VRipper',
        debugShowCheckedModeBanner: false,
        theme: ThemeData(
          useMaterial3: false,
          brightness: Brightness.light,
          scaffoldBackgroundColor: Colors.white,
          colorScheme: const ColorScheme.light(
            primary: _accent,
            secondary: _accent,
            surface: Colors.white,
            onSurface: _ink,
          ),
          dividerColor: _line,
          fontFamily: 'Arial',
          textTheme: const TextTheme(
            bodyMedium: TextStyle(fontSize: 13, color: _ink),
            bodySmall: TextStyle(fontSize: 12, color: _muted),
            titleMedium: TextStyle(fontSize: 15, fontWeight: FontWeight.w600),
          ),
        ),
        home: const HomePage(),
      );
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});
  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  final _search = TextEditingController();
  bool _showToolbar = true;
  bool _showStatus = true;
  String _scanSummary = '';
  String _session = 'Local';

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  Future<void> _addLinks() async {
    final text = await showDialog<String>(
      context: context,
      builder: (_) => const AddLinksDialog(),
    );
    if (!mounted || text == null || text.trim().isEmpty) return;
    try {
      final result = NativeEngine.instance.scanLinks(
        text,
        const ['https://vipergirls.to', 'https://www.vipergirls.to'],
      );
      final threads = (result['threads'] as List? ?? []).length;
      final posts = (result['posts'] as List? ?? []).length;
      final invalid = (result['invalid'] as List? ?? []).length;
      setState(() => _scanSummary =
          'Scanned: $threads thread(s), $posts post(s), $invalid invalid link(s).');
    } catch (error) {
      _showMessage('Native scan unavailable: $error');
    }
  }

  Future<void> _openSettings() async {
    await showDialog<void>(context: context, builder: (_) => const SettingsDialog());
  }

  Future<void> _changeSession() async {
    final result = await showDialog<String>(
      context: context,
      builder: (_) => SessionDialog(initialSession: _session),
    );
    if (!mounted) return;
    if (result != null) {
      setState(() => _session = result);
      _showMessage('Session selected: $result. Session transport is not connected yet.');
    }
  }

  void _showMessage(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), behavior: SnackBarBehavior.floating),
    );
  }

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 4,
      child: Scaffold(
        body: Column(
          children: [
            _MenuStrip(
              onAddLinks: _addLinks,
              onSettings: _openSettings,
              onChangeSession: _changeSession,
              onToggleToolbar: () => setState(() => _showToolbar = !_showToolbar),
              onToggleStatus: () => setState(() => _showStatus = !_showStatus),
            ),
            if (_showToolbar)
              _ActionToolbar(
                search: _search,
                onAddLinks: _addLinks,
                onStart: () => _showMessage('Download queue is not connected yet.'),
                onStop: () => _showMessage('Download queue is not connected yet.'),
                onClear: () => _showMessage('No completed posts to clear.'),
                onSettings: _openSettings,
              ),
            Container(
              alignment: Alignment.centerLeft,
              decoration: const BoxDecoration(
                color: _surface,
                border: Border(bottom: BorderSide(color: _line)),
              ),
              child: const TabBar(
                isScrollable: true,
                labelColor: _accent,
                unselectedLabelColor: _muted,
                indicatorColor: _accent,
                indicatorWeight: 2,
                tabs: [
                  Tab(icon: Icon(Icons.download_outlined, size: 17), text: 'Posts'),
                  Tab(icon: Icon(Icons.description_outlined, size: 17), text: 'Threads'),
                  Tab(icon: Icon(Icons.bolt_outlined, size: 17), text: 'Tasks'),
                  Tab(icon: Icon(Icons.storage_outlined, size: 17), text: 'Logs'),
                ],
              ),
            ),
            Expanded(
              child: TabBarView(
                children: [
                  PostsTab(search: _search, summary: _scanSummary, onAddLinks: _addLinks),
                  const ThreadsTab(),
                  const TasksTab(),
                  const LogsTab(),
                ],
              ),
            ),
            if (_showStatus)
              _StatusStrip(
                session: _session,
                engineLoaded: NativeEngine.instance.isLoaded,
                engineError: NativeEngine.instance.loadError,
                onChangeSession: _changeSession,
              ),
          ],
        ),
      ),
    );
  }
}

class _MenuAction {
  const _MenuAction(this.label, this.action);
  final String label;
  final VoidCallback action;
}

class _MenuStrip extends StatelessWidget {
  const _MenuStrip({
    required this.onAddLinks,
    required this.onSettings,
    required this.onChangeSession,
    required this.onToggleToolbar,
    required this.onToggleStatus,
  });

  final VoidCallback onAddLinks;
  final VoidCallback onSettings;
  final VoidCallback onChangeSession;
  final VoidCallback onToggleToolbar;
  final VoidCallback onToggleStatus;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 34,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(bottom: BorderSide(color: _line)),
      ),
      child: Row(
        children: [
          _MenuButton(label: 'File', items: [
            _MenuAction('Add links   Ctrl+L', onAddLinks),
            _MenuAction('Start All   Ctrl+S', () {}),
            _MenuAction('Stop All   Ctrl+Q', () {}),
            _MenuAction('Clear   Ctrl+Delete', () {}),
            _MenuAction('Settings   Ctrl+P', onSettings),
            _MenuAction('Change session   Shift+S', onChangeSession),
            _MenuAction('Exit   Ctrl+X', () {}),
          ]),
          _MenuButton(label: 'View', items: [
            _MenuAction('Toolbar   F6', onToggleToolbar),
            _MenuAction('Status Bar', onToggleStatus),
          ]),
          _MenuButton(label: 'Help', items: [
            _MenuAction('About VRipper', () {
              showAboutDialog(context: context, applicationName: 'VRipper Native');
            }),
          ]),
          const Spacer(),
          const Text('VRipper Native', style: TextStyle(fontSize: 11, color: _muted)),
        ],
      ),
    );
  }
}

class _MenuButton extends StatelessWidget {
  const _MenuButton({required this.label, required this.items});
  final String label;
  final List<_MenuAction> items;

  @override
  Widget build(BuildContext context) => PopupMenuButton<_MenuAction>(
        tooltip: '',
        padding: EdgeInsets.zero,
        onSelected: (item) => item.action(),
        itemBuilder: (_) => items
            .map((item) => PopupMenuItem<_MenuAction>(
                  value: item,
                  height: 34,
                  child: Text(item.label, style: const TextStyle(fontSize: 12)),
                ))
            .toList(),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
          child: Text(label, style: const TextStyle(fontSize: 12, color: _ink)),
        ),
      );
}

class _ActionToolbar extends StatelessWidget {
  const _ActionToolbar({
    required this.search,
    required this.onAddLinks,
    required this.onStart,
    required this.onStop,
    required this.onClear,
    required this.onSettings,
  });

  final TextEditingController search;
  final VoidCallback onAddLinks;
  final VoidCallback onStart;
  final VoidCallback onStop;
  final VoidCallback onClear;
  final VoidCallback onSettings;

  @override
  Widget build(BuildContext context) => Container(
        height: 54,
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
        decoration: const BoxDecoration(
          color: Colors.white,
          border: Border(bottom: BorderSide(color: _line)),
        ),
        child: Row(
          children: [
            const Icon(Icons.downloading, size: 22, color: _accent),
            const SizedBox(width: 8),
            const Text('VRipper', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 14)),
            const SizedBox(width: 18),
            Expanded(
              child: SizedBox(
                height: 34,
                child: TextField(
                  controller: search,
                  decoration: const InputDecoration(
                    hintText: 'Search posts',
                    hintStyle: TextStyle(fontSize: 12),
                    prefixIcon: Icon(Icons.search, size: 17),
                    contentPadding: EdgeInsets.symmetric(vertical: 6),
                    border: OutlineInputBorder(),
                    isDense: true,
                  ),
                  style: const TextStyle(fontSize: 12),
                ),
              ),
            ),
            const SizedBox(width: 10),
            _ToolbarButton(icon: Icons.add, tooltip: 'Add links [Ctrl+L]', onPressed: onAddLinks),
            _ToolbarButton(icon: Icons.play_arrow, tooltip: 'Start downloads [Ctrl+S]', onPressed: onStart),
            _ToolbarButton(icon: Icons.stop, tooltip: 'Stop downloads [Ctrl+Q]', onPressed: onStop),
            _ToolbarButton(icon: Icons.delete_outline, tooltip: 'Clear finished [Ctrl+Del]', onPressed: onClear),
            _ToolbarButton(icon: Icons.settings_outlined, tooltip: 'Settings [Ctrl+P]', onPressed: onSettings),
          ],
        ),
      );
}

class _ToolbarButton extends StatelessWidget {
  const _ToolbarButton({required this.icon, required this.tooltip, required this.onPressed});
  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) => IconButton(
        visualDensity: VisualDensity.compact,
        tooltip: tooltip,
        icon: Icon(icon, size: 19),
        onPressed: onPressed,
      );
}

class PostsTab extends StatelessWidget {
  const PostsTab({required this.search, required this.summary, required this.onAddLinks});
  final TextEditingController search;
  final String summary;
  final VoidCallback onAddLinks;

  @override
  Widget build(BuildContext context) => Column(
        children: [
          Container(
            height: 42,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: _line))),
            child: Row(
              children: [
                const Text('All', style: TextStyle(color: _accent, fontSize: 12)),
                const SizedBox(width: 14),
                for (final label in ['Active', 'Done', 'Automation', 'Paused', 'Error'])
                  Padding(
                    padding: const EdgeInsets.only(right: 14),
                    child: Text(label, style: const TextStyle(color: _muted, fontSize: 12)),
                  ),
                const Spacer(),
                OutlinedButton.icon(
                  onPressed: onAddLinks,
                  icon: const Icon(Icons.add, size: 16),
                  label: const Text('Add links'),
                ),
              ],
            ),
          ),
          if (summary.isNotEmpty)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(10),
              color: const Color(0xFFEAF3FC),
              child: Text(summary, style: const TextStyle(fontSize: 12)),
            ),
          const Expanded(
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.download_for_offline_outlined, size: 42, color: Color(0xFF9AA8B2)),
                  SizedBox(height: 10),
                  Text('No posts yet', style: TextStyle(color: _muted, fontSize: 13)),
                ],
              ),
            ),
          ),
          Container(
            height: 28,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: const BoxDecoration(border: Border(top: BorderSide(color: _line))),
            child: const Row(
              children: [
                _PostColumnLabel('#', 38),
                _PostColumnLabel('Preview', 90),
                _PostColumnLabel('Title', 1),
                _PostColumnLabel('Progress', 90),
                _PostColumnLabel('Status', 80),
                _PostColumnLabel('Automation', 100),
                _PostColumnLabel('Path', 1),
                _PostColumnLabel('Total', 80),
                _PostColumnLabel('Hosts', 90),
                _PostColumnLabel('Added On', 110),
              ],
            ),
          ),
        ],
      );
}

class _PostColumnLabel extends StatelessWidget {
  const _PostColumnLabel(this.label, this.width);
  final String label;
  final double width;

  @override
  Widget build(BuildContext context) {
    final child = Text(label, overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 11, color: _muted));
    return width == 1 ? Expanded(child: child) : SizedBox(width: width, child: child);
  }
}

class ThreadsTab extends StatelessWidget {
  const ThreadsTab({super.key});
  @override
  Widget build(BuildContext context) => const _EmptyTable(
        title: 'No threads yet',
        columns: ['Title', 'URL', 'Count'],
        action: 'Clear threads',
      );
}

class TasksTab extends StatelessWidget {
  const TasksTab({super.key});
  @override
  Widget build(BuildContext context) => const _EmptyTable(
        title: 'No tasks yet',
        columns: [],
        action: 'Clear tasks',
      );
}

class LogsTab extends StatelessWidget {
  const LogsTab({super.key});
  @override
  Widget build(BuildContext context) => const _EmptyTable(
        title: 'No logs yet',
        columns: ['Time', 'Thread', 'Logger', 'Level', 'Message'],
        action: 'Clear logs',
      );
}

class _EmptyTable extends StatelessWidget {
  const _EmptyTable({required this.title, required this.columns, required this.action});
  final String title;
  final List<String> columns;
  final String action;

  @override
  Widget build(BuildContext context) => Column(
        children: [
          Container(
            height: 42,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            alignment: Alignment.centerRight,
            decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: _line))),
            child: TextButton.icon(
              onPressed: () {},
              icon: const Icon(Icons.delete_outline, size: 16),
              label: Text(action, style: const TextStyle(fontSize: 12)),
            ),
          ),
          if (columns.isNotEmpty)
            Container(
              height: 30,
              padding: const EdgeInsets.symmetric(horizontal: 12),
              alignment: Alignment.centerLeft,
              decoration: const BoxDecoration(border: Border(bottom: BorderSide(color: _line))),
              child: Row(
                children: columns
                    .map((column) => Expanded(
                          child: Text(column, style: const TextStyle(fontSize: 11, color: _muted)),
                        ))
                    .toList(),
              ),
            ),
          Expanded(
            child: Center(
              child: Text(title, style: const TextStyle(color: _muted, fontSize: 13)),
            ),
          ),
        ],
      );
}

class _StatusStrip extends StatelessWidget {
  const _StatusStrip({
    required this.session,
    required this.engineLoaded,
    required this.engineError,
    required this.onChangeSession,
  });

  final String session;
  final bool engineLoaded;
  final String? engineError;
  final VoidCallback onChangeSession;

  @override
  Widget build(BuildContext context) => Container(
        height: 27,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: const BoxDecoration(
          color: _surface,
          border: Border(top: BorderSide(color: _line)),
        ),
        child: Row(
          children: [
            Icon(engineLoaded ? Icons.circle : Icons.circle_outlined,
                size: 9, color: engineLoaded ? Colors.green : Colors.orange),
            const SizedBox(width: 6),
            Text('$session session', style: const TextStyle(fontSize: 11, color: _muted)),
            const SizedBox(width: 14),
            Text(
              engineLoaded ? 'Engine ' + NativeEngine.instance.version : 'Native engine not packaged',
              style: const TextStyle(fontSize: 11, color: _muted),
            ),
            if (engineError != null) ...[
              const SizedBox(width: 8),
              Expanded(
                child: Text(engineError!, overflow: TextOverflow.ellipsis,
                    style: const TextStyle(fontSize: 10, color: Colors.deepOrange)),
              ),
            ],
            const Spacer(),
            InkWell(
              onTap: onChangeSession,
              child: const Text('Change session', style: TextStyle(fontSize: 11, color: _accent)),
            ),
          ],
        ),
      );
}

class AddLinksDialog extends StatefulWidget {
  const AddLinksDialog({super.key});
  @override
  State<AddLinksDialog> createState() => _AddLinksDialogState();
}

class _AddLinksDialogState extends State<AddLinksDialog> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
        title: const Text('Add links'),
        content: SizedBox(
          width: 520,
          child: TextField(
            controller: _controller,
            onChanged: (_) => setState(() {}),
            minLines: 6,
            maxLines: 12,
            decoration: const InputDecoration(
              hintText: 'Paste one thread or post URL per line',
              border: OutlineInputBorder(),
            ),
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
          FilledButton(
            onPressed: _controller.text.trim().isEmpty ? null : () => Navigator.pop(context, _controller.text),
            child: const Text('Scan'),
          ),
        ],
      );
}

class SessionDialog extends StatefulWidget {
  const SessionDialog({required this.initialSession});
  final String initialSession;
  @override
  State<SessionDialog> createState() => _SessionDialogState();
}

class _SessionDialogState extends State<SessionDialog> {
  late String _mode = widget.initialSession;
  final _host = TextEditingController();
  final _port = TextEditingController(text: '30000');
  final _passphrase = TextEditingController();

  @override
  void dispose() {
    _host.dispose();
    _port.dispose();
    _passphrase.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
        title: const Text('Change session'),
        content: SizedBox(
          width: 420,
          child: RadioGroup<String>(
            groupValue: _mode,
            onChanged: (value) {
              if (value != null) setState(() => _mode = value);
            },
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const RadioListTile<String>(
                  value: 'Local',
                  title: Text('Local'),
                ),
                const RadioListTile<String>(
                  value: 'Remote',
                  title: Text('Remote'),
                ),
              if (_mode == 'Remote') ...[
                TextField(controller: _host, decoration: const InputDecoration(labelText: 'Server host')),
                TextField(controller: _port, decoration: const InputDecoration(labelText: 'Port')),
                TextField(controller: _passphrase, obscureText: true,
                    decoration: const InputDecoration(labelText: 'Passphrase')),
              ],
            ],
          ),
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
          FilledButton(onPressed: () => Navigator.pop(context, _mode), child: const Text('Connect')),
        ],
      );
}

class SettingsDialog extends StatefulWidget {
  const SettingsDialog({super.key});
  @override
  State<SettingsDialog> createState() => _SettingsDialogState();
}

class _SettingsDialogState extends State<SettingsDialog> {
  final _sections = const [
    'Appearance', 'Downloads', 'Connection', 'Network', 'System',
    'Automation', 'Viper', 'Hosts', 'Posts', 'Viewers',
  ];
  int _selected = 0;
  String _proxyMode = 'Disabled';
  String _proxyProtocol = 'HTTP';
  final _proxyHost = TextEditingController();
  final _proxyPort = TextEditingController();
  final _proxyUser = TextEditingController();
  final _proxyPassword = TextEditingController();
  final _bypass = TextEditingController();

  @override
  void dispose() {
    _proxyHost.dispose();
    _proxyPort.dispose();
    _proxyUser.dispose();
    _proxyPassword.dispose();
    _bypass.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Dialog(
        child: SizedBox(
          width: 940,
          height: 620,
          child: Column(
            children: [
              Expanded(
                child: Row(
                  children: [
                    Container(
                      width: 205,
                      color: _surface,
                      padding: const EdgeInsets.all(10),
                      child: ListView.builder(
                        itemCount: _sections.length,
                        itemBuilder: (context, index) => Padding(
                          padding: const EdgeInsets.only(bottom: 4),
                          child: Material(
                            color: _surface,
                            child: ListTile(
                              dense: true,
                              selected: index == _selected,
                              selectedTileColor: const Color(0xFFE5F0FA),
                              title: Text(_sections[index], style: const TextStyle(fontSize: 12)),
                              onTap: () => setState(() => _selected = index),
                            ),
                          ),
                        ),
                      ),
                    ),
                    const VerticalDivider(width: 1),
                    Expanded(
                      child: Padding(
                        padding: const EdgeInsets.fromLTRB(20, 18, 20, 10),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(_sections[_selected],
                                style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600)),
                            const SizedBox(height: 14),
                            Expanded(
                              child: SingleChildScrollView(
                                child: _selected == 3 ? _networkSettings() : _placeholderSettings(_sections[_selected]),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              const Divider(height: 1),
              Padding(
                padding: const EdgeInsets.all(12),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    TextButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
                    const SizedBox(width: 8),
                    FilledButton(onPressed: () => Navigator.pop(context), child: const Text('Save')),
                  ],
                ),
              ),
            ],
          ),
        ),
      );

  Widget _networkSettings() => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('Proxy mode', style: TextStyle(fontWeight: FontWeight.w600)),
          const SizedBox(height: 8),
          DropdownButtonFormField<String>(
            initialValue: _proxyMode,
            decoration: const InputDecoration(border: OutlineInputBorder(), isDense: true),
            items: const [
              DropdownMenuItem(value: 'Disabled', child: Text('Disabled — direct connection')),
              DropdownMenuItem(value: 'System', child: Text('Use system proxy')),
              DropdownMenuItem(value: 'Manual', child: Text('Manual proxy')),
            ],
            onChanged: (value) => setState(() => _proxyMode = value ?? 'Disabled'),
          ),
          if (_proxyMode == 'Manual') ...[
            const SizedBox(height: 18),
            const Text('Manual proxy', style: TextStyle(fontWeight: FontWeight.w600)),
            const SizedBox(height: 8),
            DropdownButtonFormField<String>(
              initialValue: _proxyProtocol,
              decoration: const InputDecoration(labelText: 'Protocol', border: OutlineInputBorder()),
              items: const ['HTTP', 'HTTPS', 'SOCKS5']
                  .map((value) => DropdownMenuItem(value: value, child: Text(value)))
                  .toList(),
              onChanged: (value) => setState(() => _proxyProtocol = value ?? 'HTTP'),
            ),
            const SizedBox(height: 10),
            TextField(controller: _proxyHost, decoration: const InputDecoration(labelText: 'Server / Hostname', border: OutlineInputBorder())),
            const SizedBox(height: 10),
            TextField(controller: _proxyPort, keyboardType: TextInputType.number, decoration: const InputDecoration(labelText: 'Port', border: OutlineInputBorder())),
            const SizedBox(height: 10),
            TextField(controller: _proxyUser, decoration: const InputDecoration(labelText: 'Username (optional)', border: OutlineInputBorder())),
            const SizedBox(height: 10),
            TextField(controller: _proxyPassword, obscureText: true, decoration: const InputDecoration(labelText: 'Password (optional)', border: OutlineInputBorder())),
            const SizedBox(height: 10),
            TextField(controller: _bypass, minLines: 2, maxLines: 4, decoration: const InputDecoration(labelText: 'Bypass list', hintText: 'One host or domain per line', border: OutlineInputBorder())),
            const SizedBox(height: 12),
            OutlinedButton.icon(
              onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('Proxy transport test will be enabled after the Rust network layer is integrated.')),
              ),
              icon: const Icon(Icons.network_check),
              label: const Text('Test connection'),
            ),
          ],
          const SizedBox(height: 14),
          const Text(
            'Secure credential storage is a requirement; proxy secrets are not yet persisted by this prototype.',
            style: TextStyle(fontSize: 11, color: _muted),
          ),
        ],
      );

  Widget _placeholderSettings(String section) {
    final items = switch (section) {
      'Appearance' => ['Theme', 'Dark mode', 'Accent color', 'Font family', 'Font size'],
      'Downloads' => ['Download path', 'Auto start downloads', 'Auto queue threshold', 'Organize by category', 'Organize by thread', 'Order images', 'Append post ID to download folder', 'Clear finished'],
      'Connection' => ['Concurrent downloads per host', 'Global concurrent downloads', 'Connection timeout', 'Maximum attempts'],
      'System' => ['Temporary path', 'Maximum log entries', 'Clipboard monitoring', 'Clipboard polling rate'],
      'Automation' => ['Enable post automation', 'Mode', 'Required custom fields', 'Compress post folder', 'Trigger action'],
      'Viper' => ['Viper domain', 'Request limit', 'Fetch metadata', 'Authentication'],
      'Hosts' => ['Host-specific settings'],
      'Posts' => ['Default layout', 'Grid tile size', 'Preview popups'],
      'Viewers' => ['Overview layout', 'Tile size', 'Details panel', 'Image cache', 'Preload settings'],
      _ => <String>[],
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: items
          .map((item) => Padding(
                padding: const EdgeInsets.only(bottom: 14),
                child: TextFormField(
                  enabled: false,
                  decoration: InputDecoration(labelText: item, border: const OutlineInputBorder(), isDense: true),
                ),
              ))
          .toList(),
    );
  }
}
