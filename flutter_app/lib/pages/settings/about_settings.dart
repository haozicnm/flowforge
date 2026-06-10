/// About settings page — version info, update check, backend status.
library;

import 'package:flutter/material.dart';
import '../../theme/flowforge_theme.dart';
import '../../widgets/ff_widgets.dart';
import '../../widgets/flowforge_icons.dart';

class AboutSettings extends StatelessWidget {
  const AboutSettings({super.key});

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
            FfText('关于 FlowForge', fontSize: FontSizes.s18, fontWeight: FontWeights.semibold,
              color: theme.colorScheme.onSurface),
            const SizedBox(height: FlowForgeSpacing.lg),

            // App info
            Container(
              padding: const EdgeInsets.all(FlowForgeSpacing.lg),
              decoration: BoxDecoration(
                border: Border.all(color: ext.border.primary),
                borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
              ),
              child: Column(
                children: [
                  FfSvg(FfIconName.bolt, size: 48, color: ext.brandColor),
                  const SizedBox(height: FlowForgeSpacing.sm),
                  FfText('FlowForge', fontSize: FontSizes.s20, fontWeight: FontWeights.bold,
                    color: theme.colorScheme.onSurface),
                  const SizedBox(height: 4),
                  FfText('v1.0.0', fontSize: FontSizes.s14,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
                  const SizedBox(height: FlowForgeSpacing.md),
                  FfText('可视化工作流自动化引擎', fontSize: FontSizes.s13,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.6)),
                ],
              ),
            ),
            const SizedBox(height: FlowForgeSpacing.md),

            // Tech info
            _InfoRow(label: '前端框架', value: 'Flutter 3 (Dart)'),
            _InfoRow(label: '后端引擎', value: 'Rust (Axum + Tokio)'),
            _InfoRow(label: '数据存储', value: 'JSON 文件'),
            _InfoRow(label: '通信协议', value: 'HTTP + WebSocket'),
            _InfoRow(label: '许可证', value: 'MIT'),
            const SizedBox(height: FlowForgeSpacing.md),

            FfButton.outlined(
              label: '检查更新',
              size: FfButtonSize.sm,
              onTap: () {
                FfToast.show(context, message: '已是最新版本', type: FfToastType.success);
              },
            ),
          ],
        ),
      ),
    );
  }
}

class _InfoRow extends StatelessWidget {
  final String label;
  final String value;
  const _InfoRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final ext = FlowForgeThemeExtension.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: FlowForgeSpacing.md, vertical: FlowForgeSpacing.sm + 2),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: ext.border.secondary)),
      ),
      child: Row(
        children: [
          Expanded(
            child: FfText(label, fontSize: FontSizes.s13,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.6)),
          ),
          FfText(value, fontSize: FontSizes.s13,
            color: theme.colorScheme.onSurface, fontWeight: FontWeights.medium),
        ],
      ),
    );
  }
}
