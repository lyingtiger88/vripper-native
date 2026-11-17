// ignore_for_file: deprecated_member_use
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:path_provider/path_provider.dart';
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
  int? _coordinatorHandle;
  String? _coordinatorError;
  Map<String, dynamic> _queueSnapshot = const {};
  Map<String, dynamic> _networkSettings = {
    'mode': 'disabled', 'manual_proxy': null, 'bypass_hosts': <String>[],
  };
  Map<String, dynamic>? _proxyCredentials;
  Map<String, dynamic>? _viperCredentials;
  static const _secureStorage = FlutterSecureStorage();

  @override
  void initState() {
    super.initState();
    _loadSettingsAndInitialize();
  }

  @override
  void dispose() {
    final handle = _coordinatorHandle;
    if (handle != null && NativeEngine.instance.isLoaded) {
      NativeEngine.instance.destroyCoordinator(handle);
    }
    _search.dispose();
    super.dispose();
  }

  Future<void> _loadSettingsAndInitialize() async {
    try {
      final preferences = await SharedPreferences.getInstance();
      final encoded = preferences.getString('vripper.network.settings');
      if (encoded != null) {
        final decoded = jsonDecode(encoded);
        if (decoded is Map) _networkSettings = Map<String, dynamic>.from(decoded);
      }
      final proxyUser = await _secureStorage.read(key: 'vripper.proxy.username');
      final proxyPassword = await _secureStorage.read(key: 'vripper.proxy.password');
      if (proxyUser != null && proxyPassword != null) {
        _proxyCredentials = {'username': proxyUser, 'password': proxyPassword};
      }
      final viperUser = await _secureStorage.read(key: 'vripper.viper.username');
      final viperPassword = await _secureStorage.read(key: 'vripper.viper.password');
      if (viperUser != null && viperPassword != null) {
        _viperCredentials = {'username': viperUser, 'password': viperPassword};
      }
    } catch (error) {
      if (mounted) setState(() => _coordinatorError = 'Could not load saved settings: $error');
    }
    await _initializeCoordinator();
  }

  Map<String, dynamic>? _environmentProxyEndpoint() {
    final environment = Platform.environment;
    final raw = environment['HTTPS_PROXY'] ??
        environment['https_proxy'] ??
        environment['HTTP_PROXY'] ??
        environment['http_proxy'] ??
        environment['ALL_PROXY'] ??
        environment['all_proxy'];
    if (raw == null || raw.trim().isEmpty) return null;
    final value = raw.contains('://') ? raw : 'http://$raw';
    final uri = Uri.tryParse(value);
    if (uri == null || uri.host.isEmpty) return null;
    final scheme = uri.scheme.toLowerCase();
    final protocol = switch (scheme) {
      'https' => 'https',
      'socks5' || 'socks5h' => 'socks5',
      'http' => 'http',
      _ => null,
    };
    if (protocol == null) return null;
    final defaultPort = protocol == 'https' ? 443 : (protocol == 'socks5' ? 1080 : 80);
    return {'protocol': protocol, 'host': uri.host, 'port': uri.hasPort ? uri.port : defaultPort};
  }

  Map<String, dynamic>? _environmentProxyCredentials() {
    final environment = Platform.environment;
    final raw = environment['HTTPS_PROXY'] ??
        environment['https_proxy'] ??
        environment['HTTP_PROXY'] ??
        environment['http_proxy'] ??
        environment['ALL_PROXY'] ??
        environment['all_proxy'];
    if (raw == null || raw.trim().isEmpty) return null;
    final value = raw.contains('://') ? raw : 'http://$raw';
    final uri = Uri.tryParse(value);
    if (uri == null || uri.userInfo.isEmpty) return null;
    final parts = uri.userInfo.split(':');
    if (parts.length < 2) return null;
    final username = Uri.decodeComponent(parts.first);
    final password = Uri.decodeComponent(parts.sublist(1).join(':'));
    if (username.isEmpty || password.isEmpty) return null;
    return {'username': username, 'password': password};
  }

  Future<void> _initializeCoordinator() async {
    try {
      NativeEngine.instance.load();
      if (!NativeEngine.instance.isLoaded) {
        throw StateError(NativeEngine.instance.loadError ?? 'Native engine is unavailable');
      }
      final downloads = await getDownloadsDirectory() ?? await getApplicationDocumentsDirectory();
      final temporary = await getTemporaryDirectory();
      await Directory(downloads.path).create(recursive: true);
      await Directory(temporary.path).create(recursive: true);
      final config = <String, dynamic>{
        ..._networkSettings,
        'system_proxy': (_networkSettings['network'] as Map?)?['mode'] == 'system'
            ? _environmentProxyEndpoint()
            : null,
        'proxy_credentials': (_networkSettings['mode'] == 'system'
            ? _environmentProxyCredentials()
            : _proxyCredentials),
        'viper_host': 'https://vipergirls.to',
        'site_proxy': 'https://vipergirls.to',
        'viper_credentials': _viperCredentials,
        'request_limit_per_second': 2,
        'max_attempts': 3,
        'connection_timeout_secs': 30,
        'queue_settings': {'max_global_concurrent': 0, 'max_per_host_concurrent': 2},
        'download_path': downloads.path,
        'temp_path': temporary.path,
        'forum_subdirectory': false,
        'thread_sublocation': false,
        'append_post_id': false,
        'force_order': false,
        'fetch_original_imx_filename': false,
      };
      final handle = await NativeEngine.instance.createCoordinatorInBackground(config);
      if (!mounted) {
        NativeEngine.instance.destroyCoordinator(handle);
        return;
      }
      setState(() {
        _coordinatorHandle = handle;
        _coordinatorError = null;
      });
      _pollQueueEvents(handle);
    } catch (error) {
      if (mounted) setState(() => _coordinatorError = error.toString());
    }
  }

  Future<void> _pollQueueEvents(int handle) async {
    while (mounted && _coordinatorHandle == handle) {
      try {
        final event = await NativeEngine.instance.pollEventInBackground(handle, 900);
        if (event.containsKey('timeout')) continue;
        final snapshot = event['snapshot'];
        if (snapshot is Map && mounted) {
          setState(() => _queueSnapshot = Map<String, dynamic>.from(snapshot));
        }
      } catch (error) {
        if (mounted) setState(() => _coordinatorError = error.toString());
        break;
      }
    }
  }

  Future<void> _stopAllDownloads() async {
    final handle = _coordinatorHandle;
    if (handle == null) return;
    try {
      NativeEngine.instance.stopAll(handle);
      setState(() => _queueSnapshot = NativeEngine.instance.queueSnapshot(handle));
    } catch (error) {
      _showMessage('Could not stop downloads: $error');
    }
  }

  Future<void> _stopJob(int jobId) async {
    final handle = _coordinatorHandle;
    if (handle == null) return;
    try {
      NativeEngine.instance.stopJob(handle, jobId);
      if (mounted) setState(() => _queueSnapshot = NativeEngine.instance.queueSnapshot(handle));
    } catch (error) {
      _showMessage('Could not stop task $jobId: $error');
    }
  }

  Future<void> _moveJob(int jobId, String direction) async {
    final handle = _coordinatorHandle;
    if (handle == null) return;
    try {
      NativeEngine.instance.moveJob(handle, jobId, direction);
      if (mounted) setState(() => _queueSnapshot = NativeEngine.instance.queueSnapshot(handle));
    } catch (error) {
      _showMessage('Could not reorder task $jobId: $error');
    }
  }

  Future<void> _clearFinished() async {
    final handle = _coordinatorHandle;
    if (handle == null) return;
    try {
      NativeEngine.instance.clearFinished(handle);
      setState(() => _queueSnapshot = NativeEngine.instance.queueSnapshot(handle));
    } catch (error) {
      _showMessage('Could not clear completed downloads: $error');
    }
  }

  Future<void> _addLinks() async {
    final text = await showDialog<String>(
      context: context,
      builder: (_) => const AddLinksDialog(),
    );
    if (!mounted || text == null || text.trim().isEmpty) return;
    final handle = _coordinatorHandle;
    if (handle == null) {
      _showMessage('Native download engine is not ready: ${_coordinatorError ?? "initializing"}');
      return;
    }
    try {
      final result = await NativeEngine.instance.scanAndQueueInBackground(
        handle,
        text,
        const ['https://vipergirls.to', 'https://www.vipergirls.to'],
      );
      if (!mounted) return;
      final threads = result['thread_lookups'] ?? 0;
      final posts = result['posts_discovered'] ?? 0;
      final images = result['images_queued'] ?? 0;
      final invalid = (result['invalid_links'] as List? ?? []).length;
      final errors = (result['errors'] as List? ?? []).length;
      setState(() {
        _scanSummary = 'Lookups: $threads thread(s); $posts post(s); $images image(s) queued; '
            '$invalid invalid link(s); $errors error(s).';
        _queueSnapshot = NativeEngine.instance.queueSnapshot(handle);
      });
    } catch (error) {
      _showMessage('Link scan or extraction failed: $error');
    }
  }

  Future<void> _storeCredentialPair(
    String prefix,
    Map<String, dynamic>? credentials,
  ) async {
    final username = credentials?['username']?.toString();
    final password = credentials?['password']?.toString();
    if (username == null || username.isEmpty || password == null || password.isEmpty) {
      await _secureStorage.delete(key: '$prefix.username');
      await _secureStorage.delete(key: '$prefix.password');
      return;
    }
    await _secureStorage.write(key: '$prefix.username', value: username);
    await _secureStorage.write(key: '$prefix.password', value: password);
  }

  Future<void> _openSettings() async {
    final result = await showDialog<Map<String, dynamic>>(
      context: context,
      builder: (_) => SettingsDialog(
        initialNetworkSettings: _networkSettings,
        initialProxyCredentials: _proxyCredentials,
        initialViperCredentials: _viperCredentials,
      ),
    );
    if (!mounted || result == null) return;
    final network = Map<String, dynamic>.from(result['network'] as Map);
    final proxyCredentials = result['proxy_credentials'] as Map<String, dynamic>?;
    final viperCredentials = result['viper_credentials'] as Map<String, dynamic>?;
    final preferences = await SharedPreferences.getInstance();
    await preferences.setString('vripper.network.settings', jsonEncode(network));
    await _storeCredentialPair('vripper.proxy', proxyCredentials);
    await _storeCredentialPair('vripper.viper', viperCredentials);
    final oldHandle = _coordinatorHandle;
    setState(() {
      _networkSettings = network;
      _proxyCredentials = proxyCredentials;
      _viperCredentials = viperCredentials;
      _coordinatorHandle = null;
      _queueSnapshot = const {};
    });
    if (oldHandle != null) NativeEngine.instance.destroyCoordinator(oldHandle);
    await _initializeCoordinator();
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
                onStart: () => _showMessage('Downloads start automatically when links are added.'),
                onStop: _stopAllDownloads,
                onClear: _clearFinished,
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
                  TasksTab(snapshot: _queueSnapshot, onClear: _clearFinished, onStop: _stopJob, onMove: _moveJob),
                  const LogsTab(),
                ],
              ),
            ),
            if (_showStatus)
              _StatusStrip(
                session: _session,
                engineLoaded: NativeEngine.instance.isLoaded,
                engineError: _coordinatorError ?? NativeEngine.instance.loadError,
                queueSnapshot: _queueSnapshot,
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
  const TasksTab({super.key, required this.snapshot, required this.onClear, required this.onStop, required this.onMove});
  final Map<String, dynamic> snapshot;
  final VoidCallback onClear;
  final ValueChanged<int> onStop;
  final void Function(int, String) onMove;

  @override
  Widget build(BuildContext context) {
    final items = (snapshot['items'] as List? ?? const []).whereType<Map>().toList();
    if (items.isEmpty) {
      return Column(
        children: [
          Container(
            height: 42,
            alignment: Alignment.centerRight,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: TextButton.icon(
              onPressed: onClear,
              icon: const Icon(Icons.delete_outline, size: 16),
              label: const Text('Clear finished tasks'),
            ),
          ),
          const Expanded(child: Center(child: Text('No tasks yet', style: TextStyle(color: _muted)))),
        ],
      );
    }
    return Column(
      children: [
        Container(
          height: 42,
          alignment: Alignment.centerRight,
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: TextButton.icon(
            onPressed: onClear,
            icon: const Icon(Icons.delete_outline, size: 16),
            label: const Text('Clear finished tasks'),
          ),
        ),
        const Divider(height: 1),
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(
            children: [
              SizedBox(width: 55, child: Text('ID', style: TextStyle(fontSize: 11, color: _muted))),
              SizedBox(width: 115, child: Text('Host', style: TextStyle(fontSize: 11, color: _muted))),
              SizedBox(width: 100, child: Text('Status', style: TextStyle(fontSize: 11, color: _muted))),
              Expanded(child: Text('Filename / Destination', style: TextStyle(fontSize: 11, color: _muted))),
              SizedBox(width: 120, child: Text('Bytes', style: TextStyle(fontSize: 11, color: _muted))),
              SizedBox(width: 92, child: Text('Actions', style: TextStyle(fontSize: 11, color: _muted))),
            ],
          ),
        ),
        const Divider(height: 1),
        Expanded(
          child: ListView.separated(
            itemCount: items.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, index) {
              final item = items[index];
              final id = item['id'] ?? '';
              final host = item['host'] ?? '';
              final status = item['status'] ?? '';
              final filename = item['filename'] ?? 'Resolving image…';
              final destination = item['destination_path'] ?? '';
              final bytes = item['bytes_written'] ?? 0;
              final expected = item['expected_bytes'];
              return Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 9),
                child: Row(
                  children: [
                    SizedBox(width: 55, child: Text(id.toString(), style: const TextStyle(fontSize: 11))),
                    SizedBox(width: 115, child: Text(host.toString(), style: const TextStyle(fontSize: 11))),
                    SizedBox(width: 100, child: Text(status.toString(), style: const TextStyle(fontSize: 11))),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(filename.toString(), overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 12)),
                          if (destination.toString().isNotEmpty)
                            Text(destination.toString(), overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, color: _muted)),
                          if (item['error'] != null)
                            Text(item['error'].toString(), overflow: TextOverflow.ellipsis, style: const TextStyle(fontSize: 10, color: Colors.red)),
                        ],
                      ),
                    ),
                    SizedBox(
                      width: 120,
                      child: Text(expected == null ? bytes.toString() : '$bytes / $expected',
                          style: const TextStyle(fontSize: 10, color: _muted)),
                    ),
                    SizedBox(
                      width: 92,
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          IconButton(
                            tooltip: 'Move up',
                            visualDensity: VisualDensity.compact,
                            iconSize: 16,
                            onPressed: status.toString() == 'PENDING' ? () => onMove(id as int, 'up') : null,
                            icon: const Icon(Icons.arrow_upward),
                          ),
                          IconButton(
                            tooltip: 'Stop task',
                            visualDensity: VisualDensity.compact,
                            iconSize: 16,
                            onPressed: status.toString() == 'PENDING' || status.toString() == 'RUNNING'
                                ? () => onStop(id as int)
                                : null,
                            icon: const Icon(Icons.stop_circle_outlined),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      ],
    );
  }
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
    required this.queueSnapshot,
    required this.onChangeSession,
  });

  final String session;
  final bool engineLoaded;
  final String? engineError;
  final Map<String, dynamic> queueSnapshot;
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
            Flexible(
              child: Text(
                engineLoaded ? 'Engine ' + NativeEngine.instance.version : 'Native engine unavailable',
                overflow: TextOverflow.ellipsis,
                maxLines: 1,
                style: const TextStyle(fontSize: 11, color: _muted),
              ),
            ),
            if (MediaQuery.sizeOf(context).width >= 900) ...[
              const SizedBox(width: 14),
              Text(
                'Running: ' + (queueSnapshot['running']?.toString() ?? '0') +
                    '  Pending: ' + (queueSnapshot['pending']?.toString() ?? '0') +
                    '  Done: ' + (queueSnapshot['completed']?.toString() ?? '0'),
                overflow: TextOverflow.ellipsis,
                style: const TextStyle(fontSize: 11, color: _muted),
              ),
            ],
            if (engineError != null && MediaQuery.sizeOf(context).width >= 1050) ...[
              const SizedBox(width: 8),
              Flexible(
                child: Text(engineError!, overflow: TextOverflow.ellipsis, maxLines: 1,
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
  const SettingsDialog({super.key, required this.initialNetworkSettings});
  final Map<String, dynamic> initialNetworkSettings;

  @override
  State<SettingsDialog> createState() => _SettingsDialogState();
}

class _SettingsDialogState extends State<SettingsDialog> {
  final _sections = const [
    'Appearance', 'Downloads', 'Connection', 'Network', 'System',
    'Automation', 'Viper', 'Hosts', 'Posts', 'Viewers',
  ];
  int _selected = 0;
  late String _proxyMode;
  late String _proxyProtocol;
  final _proxyHost = TextEditingController();
  final _proxyPort = TextEditingController();
  final _proxyUser = TextEditingController();
  final _proxyPassword = TextEditingController();
  final _bypass = TextEditingController();

  @override
  void initState() {
    super.initState();
    final network = widget.initialNetworkSettings['network'] as Map? ?? const {};
    final mode = network['mode']?.toString() ?? 'disabled';
    _proxyMode = switch (mode) {
      'system' => 'System',
      'manual' => 'Manual',
      _ => 'Disabled',
    };
    final manual = network['manual_proxy'] as Map? ?? const {};
    final protocol = manual['protocol']?.toString() ?? 'http';
    _proxyProtocol = protocol.toUpperCase();
    _proxyHost.text = manual['host']?.toString() ?? '';
    _proxyPort.text = manual['port']?.toString() ?? '';
    _proxyUser.text = (widget.initialNetworkSettings['proxy_credentials'] as Map?)?['username']?.toString() ?? '';
    _proxyPassword.text = (widget.initialNetworkSettings['proxy_credentials'] as Map?)?['password']?.toString() ?? '';
    _bypass.text = (network['bypass_hosts'] as List? ?? const []).join('\n');
  }

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
                    FilledButton(onPressed: _saveSettings, child: const Text('Save')),
                  ],
                ),
              ),
            ],
          ),
        ),
      );


  void _saveSettings() {
    final host = _proxyHost.text.trim();
    final port = int.tryParse(_proxyPort.text.trim());
    final username = _proxyUser.text.trim();
    final password = _proxyPassword.text;
    if (_proxyMode == 'Manual') {
      if (host.isEmpty || port == null || port < 1 || port > 65535) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Enter a valid proxy host and port (1–65535).')),
        );
        return;
      }
      if (username.isEmpty != password.isEmpty) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Proxy username and password must be provided together.')),
        );
        return;
      }
    }
    final mode = switch (_proxyMode) {
      'System' => 'system',
      'Manual' => 'manual',
      _ => 'disabled',
    };
    final bypass = _bypass.text
        .split(RegExp(r'[,\n]'))
        .map((value) => value.trim())
        .where((value) => value.isNotEmpty)
        .toList();
    final manualProxy = host.isEmpty || port == null
        ? null
        : {
            'protocol': _proxyProtocol.toLowerCase(),
            'host': host,
            'port': port,
          };
    final credentials = username.isNotEmpty && password.isNotEmpty
        ? {'username': username, 'password': password}
        : null;
    Navigator.pop(context, {
      'network': {
        'mode': mode,
        'manual_proxy': manualProxy,
        'bypass_hosts': bypass,
      },
      'proxy_credentials': credentials,
    });
  }

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
          if (_proxyMode == 'System') ...[
            const SizedBox(height: 10),
            const Text(
              'System mode currently reads HTTPS_PROXY / HTTP_PROXY / ALL_PROXY environment variables. Native OS proxy discovery is still pending; if none are configured, the connection fails rather than switching to direct mode.',
              style: TextStyle(fontSize: 11, color: _muted),
            ),
          ],
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
                const SnackBar(content: Text('Connection test is not yet wired to the coordinator.')),
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
