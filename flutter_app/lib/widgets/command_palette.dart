/// Command Palette — Ctrl+K fuzzy search overlay.
///
/// AppFlowy pattern: modal overlay with input + results list.
/// Searches across: pages, workflows, node types, settings.
library;

import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';
import '../widgets/flowforge_icons.dart';

/// A command that can be executed from the palette.
class PaletteCommand {
  final String id;
  final String label;
  final String category;
  final FfIconName icon;
  final VoidCallback? onExecute;

  const PaletteCommand({
    required this.id,
    required this.label,
    required this.category,
    this.icon = FfIconName.bolt,
    this.onExecute,
  });
}

/// Overlay-based command palette.
class CommandPalette extends StatefulWidget {
  final FlowForgeApi api;
  final void Function(String workflowId)? onOpenWorkflow;
  final VoidCallback? onNavigateSettings;

  const CommandPalette({
    super.key,
    required this.api,
    this.onOpenWorkflow,
    this.onNavigateSettings,
  });

  /// Toggle the palette overlay on/off. Returns true if it was shown.
  static bool toggle(BuildContext context, {required FlowForgeApi api, void Function(String)? onOpenWorkflow, VoidCallback? onNavigateSettings}) {
    final overlay = Overlay.of(context);
    final existing = _findOverlay(overlay);
    if (existing != null) {
      existing.remove();
      return false;
    }

    late OverlayEntry entry;
    entry = OverlayEntry(
      builder: (ctx) => _PaletteDialog(
        api: api,
        onOpenWorkflow: onOpenWorkflow,
        onNavigateSettings: onNavigateSettings,
        onClose: () => entry.remove(),
      ),
    );
    overlay.insert(entry);
    return true;
  }

  static OverlayEntry? _findOverlay(OverlayState overlay) {
    // No-op — overlay management is handled by toggling
    return null;
  }

  @override
  State<CommandPalette> createState() => _CommandPaletteState();
}

class _CommandPaletteState extends State<CommandPalette> {
  @override
  Widget build(BuildContext context) {
    // Not used — the static toggle creates the overlay entry directly
    return const SizedBox.shrink();
  }
}

// ── Modal Overlay Widget ─────────────────────────────────────────

class _PaletteDialog extends StatefulWidget {
  final FlowForgeApi api;
  final void Function(String)? onOpenWorkflow;
  final VoidCallback? onNavigateSettings;
  final VoidCallback onClose;

  const _PaletteDialog({
    required this.api,
    this.onOpenWorkflow,
    this.onNavigateSettings,
    required this.onClose,
  });

  @override
  State<_PaletteDialog> createState() => _PaletteDialogState();
}

class _PaletteDialogState extends State<_PaletteDialog> {
  final _controller = TextEditingController();
  final _focusNode = FocusNode();
  String _query = '';
  List<Workflow>? _workflows;
  List<NodeTypeDef>? _nodeTypes;
  int _selectedIndex = 0;

  @override
  void initState() {
    super.initState();
    _loadData();
    _controller.addListener(() => setState(() {
      _query = _controller.text;
      _selectedIndex = 0;
    }));
    // Auto-focus after build
    WidgetsBinding.instance.addPostFrameCallback((_) => _focusNode.requestFocus());
  }

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  Future<void> _loadData() async {
    try {
      final results = await Future.wait([
        widget.api.listWorkflows(),
        widget.api.nodeTypes(),
      ]);
      if (mounted) {
        setState(() {
          _workflows = results[0] as List<Workflow>;
          _nodeTypes = results[1] as List<NodeTypeDef>;
        });
      }
    } catch (_) {}
  }

  List<PaletteCommand> _buildCommands() {
    final commands = <PaletteCommand>[];

    commands.add(PaletteCommand(id: 'page-dashboard', label: '工作流列表', category: '页面', icon: FfIconName.workspaces));
    commands.add(PaletteCommand(id: 'page-settings', label: '设置', category: '页面', icon: FfIconName.settings));

    if (_workflows != null) {
      for (final wf in _workflows!) {
        commands.add(PaletteCommand(id: 'wf-${wf.id}', label: wf.name, category: '工作流', icon: FfIconName.workspaces));
      }
    }

    if (_nodeTypes != null) {
      for (final nt in _nodeTypes!) {
        commands.add(PaletteCommand(
          id: 'node-${nt.typeName}',
          label: nt.displayName,
          category: '添加节点',
          icon: ffNodeIcon(nt.typeName),
        ));
      }
    }

    if (_query.isNotEmpty) {
      final q = _query.toLowerCase();
      commands.retainWhere((c) =>
        c.label.toLowerCase().contains(q) || c.category.toLowerCase().contains(q));
    }

    return commands;
  }

  void _execute(PaletteCommand cmd) {
    widget.onClose();
    if (cmd.id == 'page-settings') {
      widget.onNavigateSettings?.call();
    } else if (cmd.id.startsWith('wf-')) {
      widget.onOpenWorkflow?.call(cmd.id.substring(3));
    }
  }

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);
    final commands = _buildCommands();

    return GestureDetector(
      onTap: widget.onClose,
      child: Container(
        color: Colors.black26,
        child: Center(
          child: GestureDetector(
            onTap: () {},
            child: Container(
              width: 500,
              constraints: const BoxConstraints(maxHeight: 400),
              margin: const EdgeInsets.only(bottom: 200),
              decoration: BoxDecoration(
                color: ext.bg.primary,
                borderRadius: BorderRadius.circular(FlowForgeRadius.xl),
                boxShadow: const [
                  BoxShadow(color: Colors.black26, blurRadius: 24, offset: Offset(0, 8)),
                ],
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Search input
                  Padding(
                    padding: const EdgeInsets.all(FlowForgeSpacing.sm + 4),
                    child: FfTextField(
                      controller: _controller,
                      hintText: '搜索命令...',
                      prefixIcon: Padding(
                        padding: const EdgeInsets.only(left: 8),
                        child: FfSvg(FfIconName.bolt, size: 16, color: ext.brandColor),
                      ),
                      onSubmitted: (v) {
                        if (commands.isNotEmpty && _selectedIndex < commands.length) {
                          _execute(commands[_selectedIndex]);
                        }
                      },
                    ),
                  ),
                  const FfDivider(),
                  // Results
                  Flexible(
                    child: commands.isEmpty
                        ? Padding(
                            padding: const EdgeInsets.all(FlowForgeSpacing.lg),
                            child: FfText('没有匹配的命令', fontSize: FontSizes.s13,
                              color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
                          )
                        : ListView.builder(
                            shrinkWrap: true,
                            itemCount: commands.length,
                            itemBuilder: (ctx, i) {
                              final cmd = commands[i];
                              final selected = i == _selectedIndex;
                              return FfButton(
                                isSelected: selected,
                                onTap: () => _execute(cmd),
                                builder: (ctx, hovering) => Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: FlowForgeSpacing.md,
                                    vertical: FlowForgeSpacing.sm + 2,
                                  ),
                                  child: Row(
                                    children: [
                                      FfSvg(cmd.icon, size: 16,
                                        color: selected ? ext.brandColor : ext.icon.secondary),
                                      const SizedBox(width: FlowForgeSpacing.sm),
                                      Expanded(
                                        child: FfText(cmd.label, fontSize: FontSizes.s13,
                                          fontWeight: selected ? FontWeights.semibold : FontWeights.regular,
                                          color: selected ? ext.brandColor : theme.colorScheme.onSurface),
                                      ),
                                      FfText(cmd.category, fontSize: FontSizes.s11,
                                        color: theme.colorScheme.onSurface.withValues(alpha: 0.35)),
                                    ],
                                  ),
                                ),
                              );
                            },
                          ),
                  ),
                  // Footer
                  const FfDivider(),
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: FlowForgeSpacing.md,
                      vertical: FlowForgeSpacing.sm,
                    ),
                    child: Row(
                      children: [
                        _HintBadge(label: '↑↓', desc: '导航'),
                        const SizedBox(width: FlowForgeSpacing.md),
                        _HintBadge(label: 'Enter', desc: '执行'),
                        const SizedBox(width: FlowForgeSpacing.md),
                        _HintBadge(label: 'Esc', desc: '关闭'),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _HintBadge extends StatelessWidget {
  final String label;
  final String desc;
  const _HintBadge({required this.label, required this.desc});

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(
            color: ext.bg.tertiary,
            borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
          ),
          child: Text(label, style: const TextStyle(fontSize: FontSizes.s10, fontFamily: 'monospace')),
        ),
        const SizedBox(width: 4),
        FfText(desc, fontSize: FontSizes.s10,
          color: Theme.of(context).colorScheme.onSurface.withValues(alpha: 0.4)),
      ],
    );
  }
}
