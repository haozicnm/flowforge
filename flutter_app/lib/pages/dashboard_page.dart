/// Dashboard page — AppFlowy pattern with reusable widgets.
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';

class DashboardPage extends StatefulWidget {
  final FlowForgeApi api;

  const DashboardPage({super.key, required this.api});

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

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return Padding(
      padding: const EdgeInsets.all(FlowForgeSpacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Top bar
          _buildTopBar(theme, ext),
          const SizedBox(height: FlowForgeSpacing.lg),

          // Content
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
          FfText(
            '我的工作流',
            fontSize: 22,
            fontWeight: FontWeight.w600,
            color: theme.colorScheme.onSurface,
          ),
          const Spacer(),
          _buildNewWorkflowButton(ext),
        ],
      ),
    );
  }

  Widget _buildNewWorkflowButton(FlowForgeThemeExtension ext) {
    return FfButton(
      onTap: () {
        // TODO: create new workflow
      },
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
          child: const Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.add, size: 16, color: Colors.white),
              SizedBox(width: FlowForgeSpacing.xs),
              FfText(
                '新建工作流',
                fontSize: 13,
                color: Colors.white,
                fontWeight: FontWeight.w500,
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildContent(ThemeData theme, FlowForgeThemeExtension ext) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return _buildError(theme);
    }

    if (_workflows.isEmpty) {
      return _buildEmpty(theme, ext);
    }

    return _buildWorkflowGrid(theme);
  }

  Widget _buildError(ThemeData theme) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline,
              size: 48, color: theme.colorScheme.error),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText('连接服务器失败',
              fontSize: 18, fontWeight: FontWeight.w600),
          const SizedBox(height: FlowForgeSpacing.sm),
          FfText(_error!, fontSize: 13),
          const SizedBox(height: FlowForgeSpacing.md),
          FfButton(
            onTap: () {
              setState(() {
                _loading = true;
                _error = null;
              });
              _loadWorkflows();
            },
            builder: (context, _) => const FfText('重试'),
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
          Icon(Icons.workspaces_outlined,
              size: 64,
              color: ext.brandColor.withValues(alpha: 0.5)),
          const SizedBox(height: FlowForgeSpacing.md),
          FfText('还没有工作流',
              fontSize: 18, fontWeight: FontWeight.w600),
          const SizedBox(height: FlowForgeSpacing.sm),
          FfText(
            '点击"新建工作流"开始自动化之旅',
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
        return _WorkflowCard(workflow: _workflows[index]);
      },
    );
  }
}

/// Workflow card with FfHover.
class _WorkflowCard extends StatelessWidget {
  final Workflow workflow;

  const _WorkflowCard({required this.workflow});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return FfHover(
      onTap: () {
        // TODO: navigate to editor
      },
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
                Icon(Icons.play_circle_outline,
                    color: ext.brandColor, size: 18),
                const SizedBox(width: FlowForgeSpacing.sm),
                Expanded(
                  child: FfText(
                    workflow.name,
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ],
            ),
            const Spacer(),
            if (workflow.description != null)
              FfText(
                workflow.description!,
                fontSize: 12,
                color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
                maxLines: 2,
              ),
            const SizedBox(height: FlowForgeSpacing.sm),
            FfText(
              _formatDate(workflow.createdAt),
              fontSize: 11,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.4),
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
