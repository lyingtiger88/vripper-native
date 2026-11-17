import 'dart:ui';

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
          child: PostsTab(
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
            onToggleSelection: (_, value) => selected = value,
            onRemoveSelected: () => removed = true,
            onClear: () {},
            onCopyUrl: (_) {},
          ),
        ),
      ),
    ));

    expect(find.text('Fixture post title'), findsOneWidget);
    expect(find.textContaining('Fixture thread'), findsOneWidget);
    await tester.tap(find.byType(Checkbox));
    await tester.pump();
    expect(selected, isTrue);
    await tester.tap(find.text('Remove selected (0)'));
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
}
