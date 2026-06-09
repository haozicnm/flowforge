/// Settings page — AppFlowy pattern.
import 'package:flutter/material.dart';
import '../theme/flowforge_theme.dart';
import '../widgets/ff_widgets.dart';

class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

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
          SizedBox(
            height: ext.topBarHeight,
            child: FfText(
              '设置',
              fontSize: 22,
              fontWeight: FontWeight.w600,
              color: theme.colorScheme.onSurface,
            ),
          ),
          const SizedBox(height: FlowForgeSpacing.md),

          // Settings items
          _buildSettingItem(
            context: context,
            icon: Icons.info_outline,
            title: '版本',
            subtitle: 'FlowForge v0.1.0',
          ),
          const SizedBox(height: FlowForgeSpacing.sm),
          _buildSettingItem(
            context: context,
            icon: Icons.dns_outlined,
            title: '服务器地址',
            subtitle: 'http://127.0.0.1:19529',
          ),
          const SizedBox(height: FlowForgeSpacing.sm),
          _buildSettingItem(
            context: context,
            icon: Icons.palette_outlined,
            title: '主题',
            subtitle: '跟随系统',
          ),
        ],
      ),
    );
  }

  Widget _buildSettingItem({
    required BuildContext context,
    required IconData icon,
    required String title,
    required String subtitle,
  }) {
    final theme = Theme.of(context);
    final ext = theme.extension<FlowForgeThemeExtension>()!;

    return FfHover(
      onTap: () {},
      child: Container(
        padding: const EdgeInsets.all(FlowForgeSpacing.md),
        decoration: BoxDecoration(
          border: Border.all(color: ext.borderColor),
          borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
        ),
        child: Row(
          children: [
            Icon(icon, size: 20, color: ext.brandColor),
            const SizedBox(width: FlowForgeSpacing.md),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  FfText(title, fontSize: 14, fontWeight: FontWeight.w600),
                  const SizedBox(height: 2),
                  FfText(
                    subtitle,
                    fontSize: 12,
                    color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
                  ),
                ],
              ),
            ),
            Icon(Icons.chevron_right,
                size: 16,
                color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
          ],
        ),
      ),
    );
  }
}
