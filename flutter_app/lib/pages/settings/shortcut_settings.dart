/// Shortcut settings page — view current shortcuts.
library;

import 'package:flutter/material.dart';
import '../../theme/flowforge_theme.dart';
import '../../widgets/ff_widgets.dart';

class ShortcutSettings extends StatelessWidget {
  const ShortcutSettings({super.key});

  static const _shortcuts = [
    ('Ctrl + S', '保存工作流'),
    ('Ctrl + Enter', '执行工作流'),
    ('Ctrl + K', '命令面板'),
    ('Ctrl + \\', '切换侧栏'),
  ];

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.only(left: FlowForgeSpacing.lg),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            FfText('快捷键', fontSize: FontSizes.s18, fontWeight: FontWeights.semibold,
              color: theme.colorScheme.onSurface),
            const SizedBox(height: FlowForgeSpacing.lg),
            Container(
              decoration: BoxDecoration(
                border: Border.all(color: ext.border.primary),
                borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
              ),
              child: Column(
                children: _shortcuts.map((s) => _ShortcutRow(keys: s.$1, action: s.$2)).toList(),
              ),
            ),
            const SizedBox(height: FlowForgeSpacing.md),
            FfText('更多自定义快捷键功能将在后续版本开放', fontSize: FontSizes.s12,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
          ],
        ),
      ),
    );
  }
}

class _ShortcutRow extends StatelessWidget {
  final String keys;
  final String action;
  const _ShortcutRow({required this.keys, required this.action});

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md, vertical: FlowForgeSpacing.sm + 2),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
            decoration: BoxDecoration(
              color: ext.bg.secondary,
              borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
              border: Border.all(color: ext.border.secondary),
            ),
            child: Text(keys,
              style: const TextStyle(
                fontSize: FontSizes.s12,
                fontWeight: FontWeights.semibold,
                fontFamily: 'monospace',
              )),
          ),
          const SizedBox(width: FlowForgeSpacing.md),
          FfText(action, fontSize: FontSizes.s13, color: theme.colorScheme.onSurface),
        ],
      ),
    );
  }
}
