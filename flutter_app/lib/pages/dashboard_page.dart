// Dashboard page — workflow list with create/delete.
import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';
import '../widgets/flowforge_icons.dart';

class DashboardPage extends StatefulWidget {
  final FlowForgeApi api;
  final ValueChanged<String>? onOpenEditor;

  const DashboardPage({super.key, required this.api, this.onOpenEditor});

  @override
  State<DashboardPage> createState() => _DashboardPageState();
}

class _DashboardPageState extends State<DashboardPage> {
  List<Workflow> _workflows = [];
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadWorkflows();
  }

  Future<void> _loadWorkflows() async {
    try {
      final workflows = await widget.api.listWorkflows();
      setState(() {
        _workflows = workflows;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  Future<void> _createWorkflow() async {
    final name = await _showCreateDialog();
    if (name == null || name.isEmpty) return;

    try {
      final wf = await widget.api.createWorkflow(name);
      setState(() => _workflows.insert(0, wf));
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('dashboard.createFailed'.tr(args: [e.toString()]))),
        );
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
      await widget.api.deleteWorkflow(wf.id);
      setState(() => _workflows.removeWhere((w) => w.id == wf.id));
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('dashboard.deleteFailed'.tr(args: [e.toString()]))),
        );
      }
    }
  }

  Future<String?> _showCreateDialog() async {
    final controller = TextEditingController();
    return showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('dashboard.createDialog'.tr()),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: InputDecoration(
            hintText: 'dashboard.nameHint'.tr(),
            border: OutlineInputBorder(),
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
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return Padding(
      padding: const EdgeInsets.all(FlowForgeSpacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildTopBar(theme, ext),
          const SizedBox(height: FlowForgeSpacing.lg),
          Expanded(child: _buildContent(theme, ext)),
        ],
      ),
    );
  }

  Widget _buildTopBar(ThemeData theme, FlowForgeThemeExtension ext) {
    return SizedBox(
      height: ext.topBarHeight,
      child: Row(
        children: [
          FfText('dashboard.title'.tr(), fontSize: 22, fontWeight: FontWeight.w600),
          const Spacer(),
          FfButton(
            onTap: _createWorkflow,
            builder: (context, isHovering) {
              return Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: FlowForgeSpacing.md,
                  vertical: FlowForgeSpacing.sm,
                ),
                decoration: BoxDecoration(
                  color: isHovering
                      ? ext.brandColor.withValues(alpha: 0.8)
                      : ext.brandColor,
                  borderRadius: BorderRadius.circular(FlowForgeRadius.md),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    FfSvg(FfIconName.add, size: 16, color: Colors.white),
                    SizedBox(width: FlowForgeSpacing.xs),
                    FfText('dashboard.newWorkflow'.tr(), fontSize: 13, color: Colors.white, fontWeight: FontWeight.w500),
                  ],
                ),
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildContent(ThemeData theme, FlowForgeThemeExtension ext) {
    if (_loading) return const Center(child: CircularProgressIndicator());
    if (_error != null) return _buildError(theme);
    if (_workflows.isEmpty) return _buildEmpty(theme, ext);
    return _buildWorkflowGrid(theme);
  }

  Widget _buildError(ThemeData theme) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          FfSvg(FfIconName.error, size: 48, color: theme.colorScheme.error),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText('dashboard.connectionFailed'.tr(), fontSize: 18, fontWeight: FontWeight.w600),
          const SizedBox(height: FlowForgeSpacing.sm),
          FfText(_error!, fontSize: 13),
          const SizedBox(height: FlowForgeSpacing.md),
          FfButton(
            onTap: () {
              setState(() { _loading = true; _error = null; });
              _loadWorkflows();
            },
            builder: (context, _) => FfText('dashboard.retry'.tr()),
          ),
        ],
      ),
    );
  }

  Widget _buildEmpty(ThemeData theme, FlowForgeThemeExtension ext) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          FfSvg(FfIconName.workspaces, size: 64, color: ext.brandColor.withValues(alpha: 0.5)),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText('dashboard.emptyTitle'.tr(), fontSize: 18, fontWeight: FontWeight.w600),
          const SizedBox(height: FlowForgeSpacing.sm),
          FfText(
            'dashboard.emptySubtitle'.tr(),
            fontSize: 13,
            color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
          ),
        ],
      ),
    );
  }

  Widget _buildWorkflowGrid(ThemeData theme) {
    return GridView.builder(
      gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: 300,
        mainAxisSpacing: FlowForgeSpacing.md,
        crossAxisSpacing: FlowForgeSpacing.md,
        childAspectRatio: 1.6,
      ),
      itemCount: _workflows.length,
      itemBuilder: (context, index) {
        return _WorkflowCard(
          workflow: _workflows[index],
          onTap: () => widget.onOpenEditor?.call(_workflows[index].id),
          onDelete: () => _deleteWorkflow(_workflows[index]),
        );
      },
    );
  }
}

class _WorkflowCard extends StatelessWidget {
  final Workflow workflow;
  final VoidCallback? onTap;
  final VoidCallback? onDelete;

  const _WorkflowCard({required this.workflow, this.onTap, this.onDelete});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return FfHover(
      onTap: onTap,
      borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      child: Container(
        padding: const EdgeInsets.all(FlowForgeSpacing.md),
        decoration: BoxDecoration(
          border: Border.all(color: ext.borderColor),
          borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                FfSvg(FfIconName.play, color: ext.brandColor, size: 18),
                const SizedBox(width: FlowForgeSpacing.sm),
                Expanded(
                  child: FfText(workflow.name, fontSize: 14, fontWeight: FontWeight.w600),
                ),
                // Delete button
                FfButton(
                  onTap: onDelete,
                  builder: (ctx, hovering) => FfSvg(
                    FfIconName.delete,
                    size: 16,
                    color: hovering ? Colors.red : theme.colorScheme.onSurface.withValues(alpha: 0.3),
                  ),
                ),
              ],
            ),
            const Spacer(),
            if (workflow.description.isNotEmpty)
              FfText(workflow.description, fontSize: 12, maxLines: 2,
                color: theme.colorScheme.onSurface.withValues(alpha: 0.6)),
            const SizedBox(height: FlowForgeSpacing.sm),
            Row(
              children: [
                FfText(
                  'dashboard.nodes'.tr(args: ['${workflow.nodeCount}']),
                  fontSize: 11,
                  color: theme.colorScheme.onSurface.withValues(alpha: 0.4),
                ),
                const SizedBox(width: FlowForgeSpacing.md),
                FfText(
                  _formatDate(workflow.createdAt),
                  fontSize: 11,
                  color: theme.colorScheme.onSurface.withValues(alpha: 0.4),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  String _formatDate(DateTime date) {
    return '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')}';
  }
}
