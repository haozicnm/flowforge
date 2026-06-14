// FlowForge — Visual Workflow Automation Engine
//
// Architecture: left sidebar = workflow list, right = editor.
// Everything else (settings, create, delete) opens as dialogs.
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'api/flowforge_api.dart';
import 'generated/codegen_loader.g.dart';
import 'services/server_manager.dart';
import 'theme/flowforge_theme.dart';
import 'widgets/ff_widgets.dart';
import 'widgets/flowforge_icons.dart';
import 'pages/editor_page.dart';
import 'pages/settings/settings_shell.dart';
import 'widgets/command_palette.dart';
import 'widgets/ff_layout.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await EasyLocalization.ensureInitialized();

  final serverManager = ServerManager();
  final externalUrl = const String.fromEnvironment('SERVER_URL');

  await serverManager.start(
    externalServerUrl: externalUrl.isNotEmpty ? externalUrl : null,
  );

  runApp(
    EasyLocalization(
      supportedLocales: const [Locale('zh'), Locale('en')],
      path: 'assets/translations',
      assetLoader: const CodegenLoader(),
      fallbackLocale: const Locale('zh'),
      saveLocale: false,
      startLocale: const Locale('zh'),
      child: FlowForgeApp(serverManager: serverManager),
    ),
  );
}

/// Global keyboard shortcuts.
class FlowForgeShortcuts extends StatelessWidget {
  final Widget child;
  final VoidCallback? onSave;
  final VoidCallback? onExecute;
  final VoidCallback? onCommandPalette;

  const FlowForgeShortcuts({
    super.key,
    required this.child,
    this.onSave,
    this.onExecute,
    this.onCommandPalette,
  });

  @override
  Widget build(BuildContext context) {
    return Shortcuts(
      shortcuts: {
        SingleActivator(LogicalKeyboardKey.keyS, control: true):
            const _SaveIntent(),
        SingleActivator(LogicalKeyboardKey.enter, control: true):
            const _ExecuteIntent(),
        SingleActivator(LogicalKeyboardKey.keyK, control: true):
            const _CommandPaletteIntent(),
      },
      child: Actions(
        actions: {
          _SaveIntent: CallbackAction<_SaveIntent>(
            onInvoke: (_) => onSave?.call(),
          ),
          _ExecuteIntent: CallbackAction<_ExecuteIntent>(
            onInvoke: (_) => onExecute?.call(),
          ),
          _CommandPaletteIntent: CallbackAction<_CommandPaletteIntent>(
            onInvoke: (_) => onCommandPalette?.call(),
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

class _CommandPaletteIntent extends Intent {
  const _CommandPaletteIntent();
}

class FlowForgeApp extends StatefulWidget {
  final ServerManager serverManager;

  const FlowForgeApp({super.key, required this.serverManager});

  @override
  State<FlowForgeApp> createState() => _FlowForgeAppState();
}

class _FlowForgeAppState extends State<FlowForgeApp> {
  ThemeMode _themeMode = ThemeMode.system;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'FlowForge',
      debugShowCheckedModeBanner: false,
      localizationsDelegates: context.localizationDelegates,
      supportedLocales: context.supportedLocales,
      locale: context.locale,
      theme: buildLightTheme(),
      darkTheme: buildDarkTheme(),
      themeMode: _themeMode,
      home: MainShell(
        serverManager: widget.serverManager,
        themeMode: _themeMode,
        onThemeModeChanged: (m) => setState(() => _themeMode = m),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// MainShell — sidebar workflow list + editor
// ─────────────────────────────────────────────────────────────────

class MainShell extends StatefulWidget {
  final ServerManager serverManager;
  final ThemeMode themeMode;
  final ValueChanged<ThemeMode> onThemeModeChanged;

  const MainShell({
    super.key,
    required this.serverManager,
    required this.themeMode,
    required this.onThemeModeChanged,
  });

  @override
  State<MainShell> createState() => _MainShellState();
}

class _MainShellState extends State<MainShell> {
  String? _selectedWorkflowId;
  List<Workflow> _workflows = [];
  bool _loadingWorkflows = true;
  String? _workflowError;

  FlowForgeApi get _api => FlowForgeApi(baseUrl: widget.serverManager.serverUrl);

  @override
  void initState() {
    super.initState();
    _loadWorkflows();
  }

  Future<void> _loadWorkflows() async {
    try {
      final workflows = await _api.listWorkflows();
      if (!mounted) return;
      setState(() {
        _workflows = workflows;
        _loadingWorkflows = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _workflowError = e.toString();
        _loadingWorkflows = false;
      });
    }
  }

  Future<void> _createWorkflow() async {
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('dashboard.createDialog'.tr()),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: InputDecoration(
            hintText: 'dashboard.nameHint'.tr(),
            border: const OutlineInputBorder(),
          ),
          onSubmitted: (v) => Navigator.pop(ctx, v),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: Text('dashboard.cancel'.tr())),
          ElevatedButton(
            onPressed: () => Navigator.pop(ctx, controller.text),
            child: Text('dashboard.create'.tr()),
          ),
        ],
      ),
    );
    if (name == null || name.isEmpty) return;

    try {
      final wf = await _api.createWorkflow(name);
      setState(() {
        _workflows.insert(0, wf);
        _selectedWorkflowId = wf.id;
      });
    } catch (e) {
      if (mounted) {
        FfToast.show(context,
          message: 'dashboard.createFailed'.tr(args: [e.toString()]),
          type: FfToastType.error);
      }
    }
  }

  Future<void> _deleteWorkflow(Workflow wf) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('dashboard.confirmDelete'.tr()),
        content: Text('dashboard.deleteConfirm'.tr(args: [wf.name])),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx, false), child: Text('dashboard.cancel'.tr())),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: Text('dashboard.delete'.tr(), style: const TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
    if (confirm != true) return;

    try {
      await _api.deleteWorkflow(wf.id);
      setState(() {
        _workflows.removeWhere((w) => w.id == wf.id);
        if (_selectedWorkflowId == wf.id) _selectedWorkflowId = null;
      });
    } catch (e) {
      if (mounted) {
        FfToast.show(context,
          message: 'dashboard.deleteFailed'.tr(args: [e.toString()]),
          type: FfToastType.error);
      }
    }
  }

  void _openSettings() {
    showDialog(
      context: context,
      useSafeArea: false,
      builder: (_) => _SettingsDialog(
        api: _api,
        themeMode: widget.themeMode,
        onThemeModeChanged: widget.onThemeModeChanged,
      ),
    );
  }

  void _openCommandPalette() {
    CommandPalette.toggle(
      context,
      api: _api,
      onOpenWorkflow: (id) => setState(() => _selectedWorkflowId = id),
      onNavigateSettings: _openSettings,
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return FlowForgeShortcuts(
      onCommandPalette: _openCommandPalette,
      child: Scaffold(
        body: Column(
          children: [
            Expanded(
              child: Row(
                children: [
                  _buildSidebar(theme, ext),
                  const FfDivider(direction: Axis.vertical),
                  Expanded(
                    child: EditorPage(
                      api: _api,
                      workflowId: _selectedWorkflowId,
                    ),
                  ),
                ],
              ),
            ),
            _buildStatusBar(theme, ext),
          ],
        ),
      ),
    );
  }

  Widget _buildSidebar(ThemeData theme, FlowForgeThemeExtension ext) {
    final layout = context.ffLayout();
    return SizedBox(
      width: layout.sidebarWidth,
      child: Container(
        color: ext.bg.secondary,
        child: Column(
          children: [
            // Logo area
            SizedBox(
              height: ext.topBarHeight,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.lg),
                child: Row(
                  children: [
                    Container(
                      width: 28,
                      height: 28,
                      decoration: BoxDecoration(
                        color: ext.brandColor.withValues(alpha: 0.12),
                        borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
                      ),
                      child: Center(
                        child: FfSvg(FfIconName.bolt, color: ext.brandColor, size: 16),
                      ),
                    ),
                    const SizedBox(width: FlowForgeSpacing.sm),
                    FfText('FlowForge', fontSize: 15, fontWeight: FontWeight.w700,
                      color: theme.colorScheme.onSurface),
                  ],
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md),
              child: Divider(height: 1, color: ext.borderColor.withValues(alpha: 0.5)),
            ),
            // New workflow button
            Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: FlowForgeSpacing.md,
                vertical: FlowForgeSpacing.sm,
              ),
              child: FfButton(
                onTap: _createWorkflow,
                builder: (context, isHovering) {
                  return Container(
                    width: double.infinity,
                    padding: const EdgeInsets.symmetric(vertical: 6),
                    decoration: BoxDecoration(
                      color: isHovering
                          ? ext.brandColor.withValues(alpha: 0.8)
                          : ext.brandColor,
                      borderRadius: BorderRadius.circular(FlowForgeRadius.md),
                    ),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        FfSvg(FfIconName.add, size: 14, color: Colors.white),
                        const SizedBox(width: 6),
                        FfText('dashboard.newWorkflow'.tr(), fontSize: 12,
                          color: Colors.white, fontWeight: FontWeight.w500),
                      ],
                    ),
                  );
                },
              ),
            ),
            // Workflow list
            Expanded(
              child: _loadingWorkflows
                  ? const Center(child: SizedBox(
                      width: 20, height: 20,
                      child: CircularProgressIndicator(strokeWidth: 2)))
                  : _workflowError != null
                      ? _buildSidebarError(theme)
                      : _workflows.isEmpty
                          ? _buildSidebarEmpty(theme, ext)
                          : _buildWorkflowList(theme, ext),
            ),
            // Footer
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md),
              child: Divider(height: 1, color: ext.borderColor.withValues(alpha: 0.5)),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(
                horizontal: FlowForgeSpacing.md,
                vertical: FlowForgeSpacing.sm,
              ),
              child: Row(
                children: [
                  FfButton(
                    onTap: _openSettings,
                    builder: (ctx, hovering) => Container(
                      width: 28, height: 28,
                      decoration: BoxDecoration(
                        color: hovering
                            ? theme.colorScheme.onSurface.withValues(alpha: 0.06)
                            : Colors.transparent,
                        borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
                      ),
                      child: Center(
                        child: FfSvg(FfIconName.settings, size: 16,
                          color: theme.colorScheme.onSurface.withValues(alpha: 0.45)),
                      ),
                    ),
                  ),
                  const Spacer(),
                  FfText('v1.2.0', fontSize: 11,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.35)),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSidebarError(ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.all(FlowForgeSpacing.md),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          FfSvg(FfIconName.error, size: 24,
            color: theme.colorScheme.error.withValues(alpha: 0.6)),
          const SizedBox(height: FlowForgeSpacing.sm),
          Center(
            child: FfText('dashboard.connectionFailed'.tr(), fontSize: 12,
              color: theme.colorScheme.error),
          ),
          const SizedBox(height: FlowForgeSpacing.sm),
          FfButton(
            onTap: () {
              setState(() { _loadingWorkflows = true; _workflowError = null; });
              _loadWorkflows();
            },
            builder: (ctx, _) => Text('dashboard.retry'.tr(),
              style: TextStyle(fontSize: 12, color: theme.colorScheme.primary)),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebarEmpty(ThemeData theme, FlowForgeThemeExtension ext) {
    return Padding(
      padding: const EdgeInsets.all(FlowForgeSpacing.md),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          FfSvg(FfIconName.workspaces, size: 32,
            color: ext.brandColor.withValues(alpha: 0.35)),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText('dashboard.emptyTitle'.tr(), fontSize: 13, fontWeight: FontWeight.w600,
            color: theme.colorScheme.onSurface.withValues(alpha: 0.45)),
          const SizedBox(height: 4),
          Center(
            child: FfText('dashboard.emptySubtitle'.tr(), fontSize: 11,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.3)),
          ),
        ],
      ),
    );
  }

  Widget _buildWorkflowList(ThemeData theme, FlowForgeThemeExtension ext) {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.sm),
      itemCount: _workflows.length,
      itemBuilder: (context, index) {
        final wf = _workflows[index];
        final isSelected = wf.id == _selectedWorkflowId;
        return _WorkflowListItem(
          workflow: wf,
          isSelected: isSelected,
          onTap: () => setState(() => _selectedWorkflowId = wf.id),
          onDelete: () => _deleteWorkflow(wf),
          theme: theme,
          ext: ext,
        );
      },
    );
  }

  Widget _buildStatusBar(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      height: 28,
      decoration: BoxDecoration(
        color: ext.bg.secondary,
        border: Border(top: BorderSide(color: ext.borderColor.withValues(alpha: 0.5))),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md),
        child: Row(
          children: [
            Container(
              width: 7,
              height: 7,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                color: widget.serverManager.isConnected
                    ? const Color(0xFF28A745)
                    : const Color(0xFFDC3545),
              ),
            ),
            const SizedBox(width: FlowForgeSpacing.sm),
            FfText(
              widget.serverManager.isConnected ? 'statusBar.connected'.tr() : 'statusBar.disconnected'.tr(),
              fontSize: 11,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
            ),
            const Spacer(),
            FfText(
              widget.serverManager.serverUrl,
              fontSize: 11,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
          ],
        ),
      ),
    );
  }
}

// --- Appended classes ---

// ─────────────────────────────────────────────────────────────────
// Workflow list item (compact, sidebar-optimized)
// ─────────────────────────────────────────────────────────────────

class _WorkflowListItem extends StatefulWidget {
  final Workflow workflow;
  final bool isSelected;
  final VoidCallback onTap;
  final VoidCallback onDelete;
  final ThemeData theme;
  final FlowForgeThemeExtension ext;

  const _WorkflowListItem({
    required this.workflow,
    required this.isSelected,
    required this.onTap,
    required this.onDelete,
    required this.theme,
    required this.ext,
  });

  @override
  State<_WorkflowListItem> createState() => _WorkflowListItemState();
}

class _WorkflowListItemState extends State<_WorkflowListItem> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final ext = widget.ext;
    final theme = widget.theme;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          height: 36,
          margin: const EdgeInsets.symmetric(vertical: 1),
          padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.sm),
          decoration: BoxDecoration(
            color: widget.isSelected
                ? ext.brandColor.withValues(alpha: 0.1)
                : _hovering
                    ? theme.colorScheme.onSurface.withValues(alpha: 0.04)
                    : Colors.transparent,
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
          ),
          child: Row(
            children: [
              Container(
                width: 20, height: 20,
                decoration: BoxDecoration(
                  color: widget.isSelected
                      ? ext.brandColor.withValues(alpha: 0.15)
                      : theme.colorScheme.onSurface.withValues(alpha: 0.06),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Center(
                  child: FfText(
                    widget.workflow.nodeCount.toString(),
                    fontSize: 10,
                    fontWeight: FontWeight.w600,
                    color: widget.isSelected
                        ? ext.brandColor
                        : theme.colorScheme.onSurface.withValues(alpha: 0.45)),
                ),
              ),
              const SizedBox(width: FlowForgeSpacing.sm),
              Expanded(
                child: FfText(
                  widget.workflow.name,
                  fontSize: 13,
                  fontWeight: widget.isSelected ? FontWeight.w600 : FontWeight.w400,
                  maxLines: 1,
                  color: widget.isSelected
                      ? ext.brandColor
                      : theme.colorScheme.onSurface.withValues(alpha: 0.8),
                ),
              ),
              AnimatedOpacity(
                opacity: _hovering ? 1.0 : 0.0,
                duration: const Duration(milliseconds: 120),
                child: FfButton(
                  onTap: widget.onDelete,
                  builder: (ctx, hovering) => FfSvg(
                    FfIconName.delete, size: 14,
                    color: hovering ? Colors.red : theme.colorScheme.onSurface.withValues(alpha: 0.35),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// Settings dialog — wraps SettingsShell in a modal dialog
// ─────────────────────────────────────────────────────────────────

class _SettingsDialog extends StatelessWidget {
  final FlowForgeApi api;
  final ThemeMode themeMode;
  final ValueChanged<ThemeMode> onThemeModeChanged;

  const _SettingsDialog({
    required this.api,
    required this.themeMode,
    required this.onThemeModeChanged,
  });

  @override
  Widget build(BuildContext context) {
    final screenSize = MediaQuery.of(context).size;
    final ext = FlowForgeThemeExtension.of(context);

    return Dialog(
      insetPadding: const EdgeInsets.all(40),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(FlowForgeRadius.lg)),
      child: SizedBox(
        width: screenSize.width * 0.72,
        height: screenSize.height * 0.72,
        child: ClipRRect(
          borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
          child: Scaffold(
            backgroundColor: Theme.of(context).scaffoldBackgroundColor,
            body: Column(
              children: [
                // Title bar
                Container(
                  height: 44,
                  decoration: BoxDecoration(
                    color: ext.bg.secondary,
                    border: Border(
                      bottom: BorderSide(
                        color: ext.borderColor.withValues(alpha: 0.5)),
                    ),
                  ),
                  child: Row(
                    children: [
                      const SizedBox(width: FlowForgeSpacing.md),
                      FfSvg(FfIconName.settings, size: 16, color: ext.brandColor),
                      const SizedBox(width: FlowForgeSpacing.sm),
                      FfText('sidebar.settings'.tr(), fontSize: 14, fontWeight: FontWeight.w600),
                      const Spacer(),
                      FfButton(
                        onTap: () => Navigator.pop(context),
                        builder: (ctx, hovering) => Container(
                          width: 28, height: 28,
                          decoration: BoxDecoration(
                            color: hovering
                                ? Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.06)
                                : Colors.transparent,
                            borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
                          ),
                          child: Center(
                            child: FfSvg(FfIconName.close, size: 14,
                              color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.5)),
                          ),
                        ),
                      ),
                      const SizedBox(width: FlowForgeSpacing.sm),
                    ],
                  ),
                ),
                // Content
                Expanded(
                  child: SettingsShell(
                    api: api,
                    themeMode: themeMode,
                    onThemeModeChanged: onThemeModeChanged,
                    showTitleBar: false,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
