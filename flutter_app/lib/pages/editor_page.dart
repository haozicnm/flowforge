/// Editor page — YAML editor with execution.
import 'package:flutter/material.dart';
import '../api/flowforge_api.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';

class EditorPage extends StatefulWidget {
  final FlowForgeApi api;

  const EditorPage({super.key, required this.api});

  @override
  State<EditorPage> createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  String? _selectedWorkflowId;
  final _yamlController = TextEditingController();
  bool _isExecuting = false;
  String _output = '';

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
          const SizedBox(height: FlowForgeSpacing.md),

          // Editor area
          Expanded(
            child: Row(
              children: [
                // YAML editor
                Expanded(
                  flex: 3,
                  child: _buildYamlEditor(theme, ext),
                ),
                const SizedBox(width: FlowForgeSpacing.md),
                // Output panel
                Expanded(
                  flex: 2,
                  child: _buildOutputPanel(theme, ext),
                ),
              ],
            ),
          ),
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
            '工作流编辑器',
            fontSize: 22,
            fontWeight: FontWeight.w600,
            color: theme.colorScheme.onSurface,
          ),
          const Spacer(),
          // Execute button
          FfButton(
            onTap: _isExecuting ? null : _execute,
            builder: (context, isHovering) {
              return Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: FlowForgeSpacing.md,
                  vertical: FlowForgeSpacing.sm,
                ),
                decoration: BoxDecoration(
                  color: _isExecuting
                      ? Colors.grey
                      : isHovering
                          ? FlowForgeColors.brandDark
                          : FlowForgeColors.brand,
                  borderRadius: BorderRadius.circular(FlowForgeRadius.md),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      _isExecuting ? Icons.hourglass_empty : Icons.play_arrow,
                      size: 16,
                      color: Colors.white,
                    ),
                    const SizedBox(width: FlowForgeSpacing.xs),
                    FfText(
                      _isExecuting ? '执行中...' : '执行',
                      fontSize: 13,
                      color: Colors.white,
                      fontWeight: FontWeight.w500,
                    ),
                  ],
                ),
              );
            },
          ),
        ],
      ),
    );
  }

  Widget _buildYamlEditor(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: ext.borderColor),
        borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      ),
      child: Column(
        children: [
          // Editor header
          Container(
            padding: const EdgeInsets.all(FlowForgeSpacing.sm),
            decoration: BoxDecoration(
              color: ext.surfaceColor,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(FlowForgeRadius.lg),
                topRight: Radius.circular(FlowForgeRadius.lg),
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.code, size: 16),
                const SizedBox(width: FlowForgeSpacing.sm),
                const FfText('YAML', fontSize: 12, fontWeight: FontWeight.w600),
              ],
            ),
          ),
          const FfDivider(),
          // Editor body
          Expanded(
            child: TextField(
              controller: _yamlController,
              maxLines: null,
              expands: true,
              style: const TextStyle(
                fontFamily: 'monospace',
                fontSize: 13,
              ),
              decoration: const InputDecoration(
                hintText: '# 在此编辑工作流 YAML\nname: my-workflow\nnodes:\n  - id: http1\n    type: http\n    config:\n      url: "https://api.example.com"',
                border: InputBorder.none,
                contentPadding: EdgeInsets.all(FlowForgeSpacing.md),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildOutputPanel(ThemeData theme, FlowForgeThemeExtension ext) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: ext.borderColor),
        borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      ),
      child: Column(
        children: [
          // Output header
          Container(
            padding: const EdgeInsets.all(FlowForgeSpacing.sm),
            decoration: BoxDecoration(
              color: ext.surfaceColor,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(FlowForgeRadius.lg),
                topRight: Radius.circular(FlowForgeRadius.lg),
              ),
            ),
            child: const Row(
              children: [
                Icon(Icons.output, size: 16),
                SizedBox(width: FlowForgeSpacing.sm),
                FfText('输出', fontSize: 12, fontWeight: FontWeight.w600),
              ],
            ),
          ),
          const FfDivider(),
          // Output body
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(FlowForgeSpacing.md),
              child: SingleChildScrollView(
                child: FfText(
                  _output.isEmpty ? '执行结果将显示在这里' : _output,
                  fontSize: 12,
                  color: _output.isEmpty
                      ? theme.colorScheme.onSurface.withValues(alpha: 0.4)
                      : theme.colorScheme.onSurface,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _execute() async {
    if (_yamlController.text.isEmpty) return;

    setState(() {
      _isExecuting = true;
      _output = '执行中...';
    });

    // TODO: Parse YAML and execute via API
    await Future.delayed(const Duration(seconds: 1));

    setState(() {
      _isExecuting = false;
      _output = '执行完成\n\n节点输出:\n  http1.status: 200\n  http1.body: {"success": true}';
    });
  }

  @override
  void dispose() {
    _yamlController.dispose();
    super.dispose();
  }
}
