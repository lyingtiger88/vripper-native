// ignore_for_file: deprecated_member_use

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:vripper_native/main.dart';

void main() {
  testWidgets('renders the primary VRipper navigation', (tester) async {
    await tester.pumpWidget(const VripperApp());

    expect(find.text('Posts'), findsOneWidget);
    expect(find.text('Threads'), findsOneWidget);
    expect(find.text('Tasks'), findsOneWidget);
    expect(find.text('Logs'), findsOneWidget);
  });

  testWidgets('opens Settings and exposes Network and Viper sections', (tester) async {
    await tester.pumpWidget(const VripperApp());

    await tester.tap(find.byTooltip('Settings [Ctrl+P]'));
    await tester.pumpAndSettle();

    expect(find.text('Network'), findsOneWidget);
    await tester.tap(find.text('Network').last);
    await tester.pumpAndSettle();
    expect(find.text('Proxy mode'), findsOneWidget);

    await tester.tap(find.text('Viper').last);
    await tester.pumpAndSettle();
    expect(find.text('Viper account'), findsOneWidget);
    expect(find.text('Username'), findsOneWidget);

    await tester.drag(find.byType(ListView).first, const Offset(0, -500));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Database').last);
    await tester.pumpAndSettle();
    expect(find.text('Legacy Java database'), findsOneWidget);
    expect(find.text('Choose Java database'), findsOneWidget);
  });

  testWidgets('Posts list displays extracted records and supports selection/removal', (tester) async {
    final search = TextEditingController();
    var selected = false;
    var removed = false;
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 1200,
          height: 600,
          child: StatefulBuilder(
            builder: (context, setState) => PostsTab(
            search: search,
            summary: '1 post discovered',
            posts: [
              {
                'thread_id': 100,
                'thread_title': 'Fixture thread',
                'post_id': 200,
                'post_number': 3,
                'title': 'Fixture post title',
                'image_count': 2,
                'url': 'https://example.invalid/post/200',
                'hosts': [{'host': 'ImageBam', 'count': 2}],
              },
            ],
            selectedKeys: selected ? {'100:200'} : <String>{},
            onAddLinks: () {},
            onToggleSelection: (_, value) => setState(() => selected = value),
            onRemoveSelected: () => removed = true,
            onClear: () {},
            onCopyUrl: (_) {},
          )),
        ),
      ),
    ));

    expect(find.text('Fixture post title'), findsOneWidget);
    expect(find.textContaining('Fixture thread'), findsOneWidget);
    await tester.tap(find.byType(Checkbox));
    await tester.pump();
    expect(selected, isTrue);
    await tester.tap(find.text('Remove selected (1)'));
    await tester.pump();
    expect(removed, isTrue);
    search.dispose();
  });

  testWidgets('Threads list displays discovered threads and exposes removal', (tester) async {
    var removedId = 0;
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 800,
          height: 500,
          child: ThreadsTab(
            threads: [
              {'thread_id': 77, 'title': 'Stored thread', 'forum': 'Forum A', 'post_count': 4},
            ],
            onRemove: (id) => removedId = id,
            onClear: () {},
          ),
        ),
      ),
    ));
    expect(find.text('Stored thread'), findsOneWidget);
    await tester.tap(find.byTooltip('Remove thread'));
    expect(removedId, 77);
  });

  testWidgets('status strip remains within compact desktop width', (tester) async {
    tester.view.physicalSize = const Size(800, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(const VripperApp());
    await tester.pump();

    expect(tester.takeException(), isNull);
  });
  testWidgets('post progress includes completed and partially downloaded images', (tester) async {
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 900,
          height: 500,
          child: PostsTab(
            search: TextEditingController(),
            summary: 'Progress fixture',
            posts: [
              {
                'thread_id': 1,
                'thread_title': 'Thread',
                'post_id': 2,
                'post_number': 1,
                'title': 'Post progress',
                'image_count': 2,
                'queue_job_ids': [7, 8],
                'url': 'https://example.invalid/post/2',
                'hosts': [],
              },
            ],
            snapshot: {
              'items': [
                {'id': 7, 'status': 'FINISHED', 'bytes_written': 100, 'expected_bytes': 100},
                {'id': 8, 'status': 'RUNNING', 'bytes_written': 50, 'expected_bytes': 100},
              ],
            },
            selectedKeys: const {},
            onAddLinks: () {},
            onToggleSelection: (_, __) {},
            onRemoveSelected: () {},
            onClear: () {},
            onCopyUrl: (_) {},
          ),
        ),
      ),
    ));

    expect(find.text('1 / 2 images • 75%'), findsOneWidget);
    expect(find.byType(LinearProgressIndicator), findsOneWidget);
  });

  testWidgets('logs tab restores supplied persisted entries', (tester) async {
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: LogsTab(logs: const ['Task 1 • FINISHED', 'Task 2 • STOPPED']),
      ),
    ));

    expect(find.text('Task 1 • FINISHED'), findsOneWidget);
    expect(find.text('Task 2 • STOPPED'), findsOneWidget);
  });

  testWidgets('posts switch grid, filter live search and inspect image records', (tester) async {
    final search = TextEditingController();
    addTearDown(search.dispose);
    await tester.pumpWidget(MaterialApp(home: Scaffold(body: PostsTab(
      search: search, summary: '', selectedKeys: const {},
      posts: [
        {'thread_id': 1, 'post_id': 2, 'title': 'Alpha', 'image_count': 1, 'legacy_id': 9, 'done': 1},
        {'thread_id': 1, 'post_id': 3, 'title': 'Beta', 'image_count': 1},
      ],
      legacyImages: [{'post_legacy_id': 9, 'filename': 'alpha.jpg', 'status': 'FINISHED', 'url': 'https://example.invalid/a.jpg'}],
      onAddLinks: () {}, onToggleSelection: (_, __) {}, onRemoveSelected: () {}, onClear: () {}, onCopyUrl: (_) {},
    ))));
    expect(find.text('1 / 1 images • 100%'), findsOneWidget);
    await tester.tap(find.byTooltip('Grid view'));
    await tester.pumpAndSettle();
    expect(find.byType(GridView), findsOneWidget);
    search.text = 'Alpha';
    await tester.pumpAndSettle();
    expect(find.text('Beta'), findsNothing);
    await tester.tap(find.byTooltip('View images'));
    await tester.pumpAndSettle();
    expect(find.text('alpha.jpg'), findsOneWidget);
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });

  testWidgets('tasks filter and expose all pending movement actions', (tester) async {
    String? movement;
    await tester.pumpWidget(MaterialApp(home: Scaffold(body: TasksTab(
      snapshot: {'items': [
        {'id': 1, 'host': 'Test', 'status': 'PENDING', 'filename': 'pending.jpg'},
        {'id': 2, 'host': 'Test', 'status': 'ERROR', 'filename': 'failed.jpg'},
      ]},
      onClear: () {}, onStop: (_) {}, onRetry: (_) {}, onMove: (_, value) => movement = value,
    ))));
    await tester.tap(find.byTooltip('Move task'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Move to bottom'));
    await tester.pumpAndSettle();
    expect(movement, 'bottom');
    await tester.enterText(find.byType(TextField), 'failed');
    await tester.pump();
    expect(find.text('pending.jpg'), findsNothing);
    expect(find.text('failed.jpg'), findsOneWidget);
    expect(find.byTooltip('Retry task'), findsOneWidget);
  });

  testWidgets('logs can be filtered to errors', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: Scaffold(body: LogsTab(
      logs: ['Task 1 FINISHED', 'Task 2 ERROR'],
    ))));
    await tester.tap(find.text('Errors only'));
    await tester.pumpAndSettle();
    expect(find.text('Task 1 FINISHED'), findsNothing);
    expect(find.text('Task 2 ERROR'), findsOneWidget);
  });

}
