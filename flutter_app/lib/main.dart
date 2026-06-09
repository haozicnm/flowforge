/// FlowForge — Visual Workflow Automation Engine
///
/// Architecture: Flutter Desktop connects to Rust backend via HTTP.
/// Pattern: BLoC + GetIt DI + layered theme (inspired by AppFlowy).
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:get_it/get_it.dart';

import 'api/flowforge_api.dart';
import 'bloc/workspace_bloc.dart';
import 'pages/dashboard_page.dart';
import 'pages/editor_page.dart';
import 'pages/settings_page.dart';
import 'services/server_manager.dart';
import 'theme/flowforge_theme.dart';
import 'widgets/ff_widgets.dart';

final getIt = GetIt.instance;

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // ── DI Setup (like AppFlowy's deps_resolver) ──
  final serverManager = ServerManager();
  getIt.registerSingleton<ServerManager>(serverManager);

  // Try to connect to existing server, or start a new one
  final externalUrl = const String.fromEnvironment('SERVER_URL');
  try {
    if (externalUrl.isNotEmpty) {
      await serverManager.start(externalServerUrl: externalUrl);
    } else {
      await serverManager.start();
    }
  } catch (e) {
    debugPrint('Server start failed: $e');
  }

  final api = FlowForgeApi(baseUrl: serverManager.serverUrl);
  getIt.registerSingleton<FlowForgeApi>(api);

  runApp(FlowForgeApp(api: api));
}

class FlowForgeApp extends StatelessWidget {
  final FlowForgeApi api;

  const FlowForgeApp({super.key, required this.api});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (_) => WorkspaceBloc(api: api)..add(const WorkspaceEvent.loadWorkflows()),
      child: MaterialApp(
        title: 'FlowForge',
        debugShowCheckedModeBanner: false,
        theme: buildLightTheme(),
        darkTheme: buildDarkTheme(),
        themeMode: ThemeMode.system,
        home: const MainShell(),
      ),
    );
  }
}

/// Main app shell — Stack layout with sidebar (like AppFlowy's DesktopHomeScreen).
class MainShell extends StatelessWidget {
  const MainShell({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<WorkspaceBloc, WorkspaceState>(
      builder: (context, state) {
        return Scaffold(
          body: Row(
            children: [
              // ── Sidebar ──
              const _Sidebar(),
              const VerticalDivider(thickness: 1, width: 1),
              // ── Main content ──
              Expanded(
                child: IndexedStack(
                  index: state.selectedPageIndex,
                  children: const [
                    DashboardPage(),
                    EditorPage(),
                    SettingsPage(),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}

/// Sidebar navigation (like AppFlowy's HomeSideBar).
class _Sidebar extends StatelessWidget {
  const _Sidebar();

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return BlocBuilder<WorkspaceBloc, WorkspaceState>(
      builder: (context, state) {
        return Container(
          width: 220,
          color: ext.sidebarBg,
          child: Column(
            children: [
              // ── Logo ──
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
                child: Row(
                  children: [
                    Icon(Icons.bolt, color: theme.colorScheme.primary, size: 24),
                    const SizedBox(width: 8),
                    Text(
                      'FlowForge',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ],
                ),
              ),
              const FfDivider(),
              const SizedBox(height: 8),

              // ── Navigation items ──
              FfButton(
                text: '工作流',
                icon: Icons.dashboard_outlined,
                selected: state.selectedPageIndex == 0,
                onTap: () => context
                    .read<WorkspaceBloc>()
                    .add(const WorkspaceEvent.switchPage(0)),
              ),
              FfButton(
                text: '编辑器',
                icon: Icons.edit_outlined,
                selected: state.selectedPageIndex == 1,
                onTap: () => context
                    .read<WorkspaceBloc>()
                    .add(const WorkspaceEvent.switchPage(1)),
              ),

              const Spacer(),
              const FfDivider(),

              // ── Footer ──
              FfButton(
                text: '设置',
                icon: Icons.settings_outlined,
                selected: state.selectedPageIndex == 2,
                onTap: () => context
                    .read<WorkspaceBloc>()
                    .add(const WorkspaceEvent.switchPage(2)),
              ),
              const SizedBox(height: 8),
            ],
          ),
        );
      },
    );
  }
}
