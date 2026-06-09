/// FlowForge — Visual Workflow Automation Engine
///
/// Architecture: Flutter Desktop connects to Rust backend via HTTP.
import 'package:flutter/material.dart';
import 'api/flowforge_api.dart';
import 'services/server_manager.dart';
import 'pages/dashboard_page.dart';
import 'pages/editor_page.dart';
import 'pages/settings_page.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final serverManager = ServerManager();

  // Try to connect to existing server, or start a new one
  final externalUrl = const String.fromEnvironment('SERVER_URL');
  try {
    if (externalUrl.isNotEmpty) {
      await serverManager.start(externalServerUrl: externalUrl);
    } else {
      await serverManager.start();
    }
  } catch (e) {
    print('Server start failed: $e');
    // Continue anyway — the UI will show connection error
  }

  runApp(FlowForgeApp(serverManager: serverManager));
}

class FlowForgeApp extends StatelessWidget {
  final ServerManager serverManager;

  const FlowForgeApp({super.key, required this.serverManager});

  @override
  Widget build(BuildContext context) {
    final api = FlowForgeApi(baseUrl: serverManager.serverUrl);

    return MaterialApp(
      title: 'FlowForge',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF00B4D8),
          brightness: Brightness.light,
        ),
        useMaterial3: true,
        fontFamily: 'Inter',
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF00B4D8),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
        fontFamily: 'Inter',
      ),
      themeMode: ThemeMode.system,
      home: MainShell(api: api, serverManager: serverManager),
    );
  }
}

/// Main app shell with sidebar navigation.
class MainShell extends StatefulWidget {
  final FlowForgeApi api;
  final ServerManager serverManager;

  const MainShell({
    super.key,
    required this.api,
    required this.serverManager,
  });

  @override
  State<MainShell> createState() => _MainShellState();
}

class _MainShellState extends State<MainShell> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final pages = [
      DashboardPage(api: widget.api),
      const EditorPage(),
      const SettingsPage(),
    ];

    return Scaffold(
      body: Row(
        children: [
          // Sidebar
          NavigationRail(
            selectedIndex: _selectedIndex,
            onDestinationSelected: (index) {
              setState(() => _selectedIndex = index);
            },
            labelType: NavigationRailLabelType.all,
            destinations: const [
              NavigationRailDestination(
                icon: Icon(Icons.dashboard_outlined),
                selectedIcon: Icon(Icons.dashboard),
                label: Text('工作流'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.edit_outlined),
                selectedIcon: Icon(Icons.edit),
                label: Text('编辑器'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.settings_outlined),
                selectedIcon: Icon(Icons.settings),
                label: Text('设置'),
              ),
            ],
          ),
          // Divider
          const VerticalDivider(thickness: 1, width: 1),
          // Main content
          Expanded(
            child: pages[_selectedIndex],
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    widget.api.dispose();
    widget.serverManager.stop();
    super.dispose();
  }
}
