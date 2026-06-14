/// Settings shell — AppFlowy SettingsDialog pattern.
///
/// Layout: left SettingsMenu (204px) + FfDivider(vertical) + right content area.
/// Use  when embedding inside a dialog that already has a title.
library;

import 'package:easy_localization/easy_localization.dart';
import 'package:flutter/material.dart';
import '../../api/flowforge_api.dart';
import '../../theme/flowforge_theme.dart';
import '../../widgets/ff_widgets.dart';
import '../../widgets/flowforge_icons.dart';
import 'general_settings.dart';
import 'shortcut_settings.dart';
import 'about_settings.dart';
import 'plugin_settings.dart';

/// Available settings pages.
enum SettingsPageKind { general, shortcuts, about, plugins }

class SettingsShell extends StatefulWidget {
  final FlowForgeApi api;
  final ThemeMode themeMode;
  final ValueChanged<ThemeMode> onThemeModeChanged;
  final bool showTitleBar;

  const SettingsShell({
    super.key,
    required this.api,
    required this.themeMode,
    required this.onThemeModeChanged,
    this.showTitleBar = true,
  });

  @override
  State<SettingsShell> createState() => _SettingsShellState();
}

class _SettingsShellState extends State<SettingsShell> {
  SettingsPageKind _page = SettingsPageKind.general;

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(FlowForgeSpacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (widget.showTitleBar) ...[
            SizedBox(
              height: ext.topBarHeight,
              child: FfText('sidebar.settings'.tr(), fontSize: FontSizes.s22, fontWeight: FontWeights.semibold,
                color: theme.colorScheme.onSurface),
            ),
            const SizedBox(height: FlowForgeSpacing.md),
          ],
          Expanded(
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(
                  width: 204,
                  child: _SettingsMenu(selected: _page, onChanged: (p) => setState(() => _page = p)),
                ),
                const FfDivider(direction: Axis.vertical),
                Expanded(child: _buildPage()),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildPage() {
    switch (_page) {
      case SettingsPageKind.general: return GeneralSettings(
        themeMode: widget.themeMode,
        onThemeModeChanged: widget.onThemeModeChanged,
      );
      case SettingsPageKind.shortcuts: return const ShortcutSettings();
      case SettingsPageKind.about: return const AboutSettings();
      case SettingsPageKind.plugins: return PluginSettings(api: widget.api);
    }
  }
}

class _SettingsMenu extends StatelessWidget {
  final SettingsPageKind selected;
  final ValueChanged<SettingsPageKind> onChanged;
  const _SettingsMenu({required this.selected, required this.onChanged});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(right: FlowForgeSpacing.sm),
      child: Column(
        children: [
          _MenuItem(icon: FfIconName.settings, label: '通用', selected: selected == SettingsPageKind.general, onTap: () => onChanged(SettingsPageKind.general)),
          const SizedBox(height: 2),
          _MenuItem(icon: FfIconName.bolt, label: '快捷键', selected: selected == SettingsPageKind.shortcuts, onTap: () => onChanged(SettingsPageKind.shortcuts)),
          const SizedBox(height: 2),
          _MenuItem(icon: FfIconName.info, label: '关于', selected: selected == SettingsPageKind.about, onTap: () => onChanged(SettingsPageKind.about)),
          const SizedBox(height: 2),
          _MenuItem(icon: FfIconName.add, label: '插件', selected: selected == SettingsPageKind.plugins, onTap: () => onChanged(SettingsPageKind.plugins)),
        ],
      ),
    );
  }
}

class _MenuItem extends StatelessWidget {
  final FfIconName icon;
  final String label;
  final bool selected;
  final VoidCallback onTap;
  const _MenuItem({required this.icon, required this.label, required this.selected, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return FfButton(
      isSelected: selected,
      onTap: onTap,
      builder: (ctx, hovering) => AnimatedContainer(
        duration: const Duration(milliseconds: 150),
        height: 32,
        padding: const EdgeInsets.only(left: FlowForgeSpacing.sm + 4),
        decoration: BoxDecoration(
          border: Border(
            left: BorderSide(
              width: 2,
              color: selected ? ext.brandColor : Colors.transparent,
            ),
          ),
        ),
        child: Row(
          children: [
            FfSvg(icon, size: 18, color: selected ? ext.brandColor : theme.colorScheme.onSurface.withValues(alpha: 0.6)),
            const SizedBox(width: FlowForgeSpacing.sm),
            FfText(label, fontSize: FontSizes.s13, fontWeight: selected ? FontWeights.semibold : FontWeights.regular,
              color: selected ? ext.brandColor : theme.colorScheme.onSurface.withValues(alpha: 0.8)),
          ],
        ),
      ),
    );
  }
}
