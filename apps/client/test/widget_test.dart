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

  testWidgets('opens Settings and exposes the Network section', (tester) async {
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
}

  testWidgets('status strip remains within compact desktop width', (tester) async {
    tester.view.physicalSize = const Size(800, 600);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);

    await tester.pumpWidget(const VripperApp());
    await tester.pump();

    expect(tester.takeException(), isNull);
  });
