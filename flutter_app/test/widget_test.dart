import 'package:flutter_test/flutter_test.dart';
import 'package:flowforge/main.dart';
import 'package:flowforge/services/server_manager.dart';

void main() {
  testWidgets('App renders smoke test', (WidgetTester tester) async {
    final serverManager = ServerManager();
    await tester.pumpWidget(FlowForgeApp(serverManager: serverManager));
    expect(find.byType(FlowForgeApp), findsOneWidget);
  });
}
