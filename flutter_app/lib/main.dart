// FlowForge — Visual Workflow Automation Engine
//
// Architecture: Flutter Desktop connects to Rust backend via HTTP.
// Backend is started separately — this app only connects.
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'api/flowforge_api.dart';
import 'services/server_manager.dart';
import 'theme/flowforge_theme.dart';
import 'widgets/ff_widgets.dart';
import 'pages/dashboard_page.dart';
import 'pages/editor_page.dart';
import 'pages/settings_page.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final serverManager = ServerManager();
  final externalUrl = const String.fromEnvironment('SERVER_URL');

  await serverManager.start(
    externalServerUrl: externalUrl.isNotEmpty ? externalUrl : null,
  );

  runApp(FlowForgeApp(serverManager: serverManager));
}

/// Global keyboard shortcuts.
class FlowForgeShortcuts extends StatelessWidget {
  final Widget child;
  final VoidCallback? onSave;
  final VoidCallback? onExecute;

  const FlowForgeShortcuts({
    super.key,
    required this.child,
    this.onSave,
    this.onExecute,
  });

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: {
        LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyS):
            const _SaveIntent(),
        LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.enter):
            const _ExecuteIntent(),
      },
      child: Actions(
        actions: {
          _SaveIntent: CallbackAction<_SaveIntent>(
            onInvoke: (_) => onSave?.call(),
          ),
          _ExecuteIntent: CallbackAction<_ExecuteIntent>(
            onInvoke: (_) => onExecute?.call(),
          ),
        },
        child: Focus(autofocus: true, child: child),
      ),
    );
  }
}

class _SaveIntent extends Intent {
  const _SaveIntent();
}

class _ExecuteIntent extends Intent {
  const _ExecuteIntent();
}

class FlowForgeApp extends StatelessWidget {
  final ServerManager serverManager;

  const FlowForgeApp({super.key, required this.serverManager});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'FlowForge',
      debugShowCheckedModeBanner: false,
      theme: buildLightTheme(),
      darkTheme: buildDarkTheme(),
      themeMode: ThemeMode.system,
      home: MainShell(serverManager: serverManager),
    );
  }
}

class MainShell extends StatefulWidget {
  final ServerManager serverManager;

  const MainShell({super.key, required this.serverManager});

  @override
  State<MainShell> createState() => _MainShellState();
}

class _MainShellState extends State<MainShell> {
  int _selectedIndex = 0;
  String? _selectedWorkflowId;

  void _openWorkflow(String id) {
    setState(() {
      _selectedWorkflowId = id;
      _selectedIndex = 1; // switch to editor
    });
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;
    final api = FlowForgeApi(baseUrl: widget.serverManager.serverUrl);

    return Scaffold(
      body: Column(
        children: [
          Expanded(
            child: Stack(
              children: [
                Positioned.fill(
                  left: ext.sidebarWidth + 1,
                  child: IndexedStack(
                    index: _selectedIndex,
                    children: [
                      DashboardPage(api: api, onOpenEditor: _openWorkflow),
                      EditorPage(api: api, workflowId: _selectedWorkflowId),
                      const SettingsPage(),
                    ],
                  ),
                ),
                Positioned(
                  left: 0, top: 0, bottom: 0,
                  width: ext.sidebarWidth,
                  child: _buildSidebar(theme, ext),
                ),
                Positioned(
                  left: ext.sidebarWidth, top: 0, bottom: 0,
                  child: const FfDivider(direction: Axis.vertical),
                ),
              ],
            ),
          ),
          // Status bar
          Container(
            height: 24,
            color: ext.surfaceColor,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Row(
                children: [
                  Icon(Icons.circle, size: 8,
                    color: widget.serverManager.isConnected ? Colors.green : Colors.red),
                  const SizedBox(width: 6),
                  FfText(
                    widget.serverManager.isConnected ? '已连接' : '未连接',
                    fontSize: 11,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
                  ),
                  const Spacer(),
                  FfText(
                    widget.serverManager.serverUrl,
                    fontSize: 11,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
                  ),
                  const SizedBox(width: 12),
                  FfText('Ctrl+S 保存  Ctrl+Enter 执行', fontSize: 10,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.3)),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebar(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      color: ext.surfaceColor,
      child: Column(
        children: [
          SizedBox(
            height: ext.topBarHeight,
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md),
              child: Row(
                children: [
                  Icon(Icons.bolt, color: ext.brandColor, size: 24),
                  const SizedBox(width: FlowForgeSpacing.sm),
                  FfText('FlowForge', fontSize: 16, fontWeight: FontWeight.w600,
                    color: theme.colorScheme.onSurface),
                ],
              ),
            ),
          ),
          const FfDivider(),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: FlowForgeSpacing.sm),
              child: Column(
                children: [
                  _buildNavItem(index: 0, icon: Icons.workspaces_outlined,
                    selectedIcon: Icons.workspaces, label: '工作流', ext: ext, theme: theme),
                  _buildNavItem(index: 1, icon: Icons.edit_outlined,
                    selectedIcon: Icons.edit, label: '编辑器', ext: ext, theme: theme),
                  _buildNavItem(index: 2, icon: Icons.settings_outlined,
                    selectedIcon: Icons.settings, label: '设置', ext: ext, theme: theme),
                ],
              ),
            ),
          ),
          const FfDivider(),
          Padding(
            padding: const EdgeInsets.all(FlowForgeSpacing.md),
            child: FfText('v0.1.0', fontSize: 11,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
          ),
        ],
      ),
    );
  }

  Widget _buildNavItem({
    required int index,
    required IconData icon,
    required IconData selectedIcon,
    required String label,
    required FlowForgeThemeExtension ext,
    required ThemeData theme,
  }) {
    final isSelected = _selectedIndex == index;
    return FfButton(
      isSelected: isSelected,
      onTap: () => setState(() => _selectedIndex = index),
      builder: (context, isHovering) {
        return Container(
          height: 32,
          margin: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.sm),
          child: Row(
            children: [
              const SizedBox(width: FlowForgeSpacing.sm),
              Icon(isSelected ? selectedIcon : icon, size: 20,
                color: isSelected ? ext.brandColor : theme.colorScheme.onSurface.withValues(alpha: 0.6)),
              const SizedBox(width: FlowForgeSpacing.md),
              FfText(label, fontSize: 13,
                fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
                color: isSelected ? ext.brandColor : theme.colorScheme.onSurface.withValues(alpha: 0.8)),
            ],
          ),
        );
      },
    );
  }
}
