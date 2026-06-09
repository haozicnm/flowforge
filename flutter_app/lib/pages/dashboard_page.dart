/// Dashboard page — the "home" screen showing all workflows.
///
/// Rule: This is the entry point, NOT a blank canvas.
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';

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

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Row(
            children: [
              Icon(Icons.dashboard, color: theme.colorScheme.primary),
              const SizedBox(width: 12),
              Text(
                '我的工作流',
                style: theme.textTheme.headlineMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
              const Spacer(),
              FilledButton.icon(
                onPressed: () {
                  // TODO: create new workflow
                },
                icon: const Icon(Icons.add),
                label: const Text('新建工作流'),
              ),
            ],
          ),
          const SizedBox(height: 24),

          // Content
          Expanded(
            child: _buildContent(theme),
          ),
        ],
      ),
    );
  }

  Widget _buildContent(ThemeData theme) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.error_outline, size: 48, color: theme.colorScheme.error),
            const SizedBox(height: 16),
            Text('连接服务器失败', style: theme.textTheme.titleLarge),
            const SizedBox(height: 8),
            Text(_error!, style: theme.textTheme.bodyMedium),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: () {
                setState(() {
                  _loading = true;
                  _error = null;
                });
                _loadWorkflows();
              },
              child: const Text('重试'),
            ),
          ],
        ),
      );
    }

    if (_workflows.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.auto_awesome_outlined,
                size: 64, color: theme.colorScheme.primary.withOpacity(0.5)),
            const SizedBox(height: 16),
            Text('还没有工作流', style: theme.textTheme.titleLarge),
            const SizedBox(height: 8),
            Text(
              '点击"新建工作流"开始自动化之旅',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurface.withOpacity(0.6),
              ),
            ),
          ],
        ),
      );
    }

    // Workflow grid
    return GridView.builder(
      gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: 320,
        mainAxisSpacing: 16,
        crossAxisSpacing: 16,
        childAspectRatio: 1.6,
      ),
      itemCount: _workflows.length,
      itemBuilder: (context, index) {
        final wf = _workflows[index];
        return _WorkflowCard(workflow: wf);
      },
    );
  }
}

/// A single workflow card in the dashboard grid.
class _WorkflowCard extends StatelessWidget {
  final Workflow workflow;

  const _WorkflowCard({required this.workflow});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      elevation: 1,
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: () {
          // TODO: navigate to editor
        },
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(Icons.play_circle_outline,
                      color: theme.colorScheme.primary, size: 20),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      workflow.name,
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  PopupMenuButton<String>(
                    itemBuilder: (context) => [
                      const PopupMenuItem(
                          value: 'edit', child: Text('编辑')),
                      const PopupMenuItem(
                          value: 'duplicate', child: Text('复制')),
                      const PopupMenuItem(
                          value: 'delete', child: Text('删除')),
                    ],
                    onSelected: (value) {
                      // TODO: handle menu actions
                    },
                    icon: const Icon(Icons.more_vert, size: 18),
                  ),
                ],
              ),
              const Spacer(),
              if (workflow.description != null)
                Text(
                  workflow.description!,
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: theme.colorScheme.onSurface.withOpacity(0.6),
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              const SizedBox(height: 8),
              Text(
                _formatDate(workflow.createdAt),
                style: theme.textTheme.labelSmall?.copyWith(
                  color: theme.colorScheme.onSurface.withOpacity(0.4),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatDate(DateTime date) {
    return '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')}';
  }
}
