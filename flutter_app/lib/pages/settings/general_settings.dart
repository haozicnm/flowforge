/// General settings page — language, theme, data directory, server address.
library;

import 'package:flutter/material.dart';
import '../../theme/flowforge_theme.dart';
import '../../widgets/ff_widgets.dart';

class GeneralSettings extends StatelessWidget {
  const GeneralSettings({super.key});

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
            FfText('通用设置', fontSize: FontSizes.s18, fontWeight: FontWeights.semibold,
              color: theme.colorScheme.onSurface),
            const SizedBox(height: FlowForgeSpacing.lg),

            _SettingCategory(
              title: '语言',
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: FfDropdown<String>(
                  value: '简体中文',
                  items: const [
                    FfDropdownItem(value: '简体中文', label: '简体中文'),
                    FfDropdownItem(value: 'English', label: 'English'),
                  ],
                  onChanged: (_) {},
                  width: 200,
                ),
              ),
            ),
            const SizedBox(height: FlowForgeSpacing.md),

            _SettingCategory(
              title: '主题',
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: FfDropdown<String>(
                  value: '跟随系统',
                  items: const [
                    FfDropdownItem(value: '跟随系统', label: '跟随系统'),
                    FfDropdownItem(value: '浅色', label: '浅色'),
                    FfDropdownItem(value: '深色', label: '深色'),
                  ],
                  onChanged: (_) {},
                  width: 200,
                ),
              ),
            ),
            const SizedBox(height: FlowForgeSpacing.md),

            _SettingCategory(
              title: '数据目录',
              subtitle: '工作流文件存储位置',
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: FfTextField(
                  hintText: 'data/',
                  readOnly: true,
                  suffixIcon: FfButton.text(
                    label: '浏览',
                    size: FfButtonSize.sm,
                    onTap: () {
                      FfToast.show(context, message: '文件浏览器功能将在后续版本开放', type: FfToastType.info);
                    },
                  ),
                ),
              ),
            ),
            const SizedBox(height: FlowForgeSpacing.md),

            _SettingCategory(
              title: '服务器地址',
              subtitle: 'Rust 后端监听地址',
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: FfTextField(
                  hintText: '127.0.0.1:19529',
                  readOnly: true,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SettingCategory extends StatelessWidget {
  final String title;
  final String? subtitle;
  final Widget child;

  const _SettingCategory({required this.title, this.subtitle, required this.child});

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.all(FlowForgeSpacing.md),
      decoration: BoxDecoration(
        border: Border.all(color: ext.border.primary),
        borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FfText(title, fontSize: FontSizes.s14, fontWeight: FontWeights.semibold,
            color: theme.colorScheme.onSurface),
          if (subtitle != null) ...[
            const SizedBox(height: 2),
            FfText(subtitle!, fontSize: FontSizes.s11,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
          ],
          const SizedBox(height: 6),
          child,
        ],
      ),
    );
  }
}
