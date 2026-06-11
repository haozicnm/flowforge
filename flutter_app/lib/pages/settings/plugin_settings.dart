/// Plugin settings page — view installed node types & plugin directory.
library;

import 'package:flutter/material.dart';
import '../../api/flowforge_api.dart';
import '../../theme/flowforge_theme.dart';
import '../../widgets/ff_widgets.dart';
import '../../widgets/flowforge_icons.dart';

class PluginSettings extends StatefulWidget {
  final FlowForgeApi api;
  const PluginSettings({super.key, required this.api});

  @override
  State<PluginSettings> createState() => _PluginSettingsState();
}

class _PluginSettingsState extends State<PluginSettings> {
  List<NodeTypeDef>? _types;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final types = await widget.api.nodeTypes();
      if (mounted) setState(() { _types = types; _loading = false; });
    } catch (_) {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.only(left: FlowForgeSpacing.lg),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            FfText('插件管理', fontSize: FontSizes.s18, fontWeight: FontWeights.semibold,
              color: theme.colorScheme.onSurface),
            const SizedBox(height: FlowForgeSpacing.sm),
            FfText('已安装的节点类型 (${_types?.length ?? 0})', fontSize: FontSizes.s12,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
            const SizedBox(height: FlowForgeSpacing.lg),

            if (_loading)
              const Center(child: Padding(
                padding: EdgeInsets.all(32),
                child: SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2)),
              ))
            else if (_types == null || _types!.isEmpty)
              Center(child: FfText('暂无节点类型', fontSize: FontSizes.s13,
                color: theme.colorScheme.onSurface.withValues(alpha: 0.4)))
            else
              ..._types!.map((t) => _PluginCard(typeDef: t)),

            const SizedBox(height: FlowForgeSpacing.lg),
            FfText('插件目录', fontSize: FontSizes.s14, fontWeight: FontWeights.semibold,
              color: theme.colorScheme.onSurface),
            const SizedBox(height: FlowForgeSpacing.sm),
            FfText('将 .so/.dll/.dylib 插件文件放入 plugins/ 目录，重启后自动加载。',
              fontSize: FontSizes.s12,
              color: theme.colorScheme.onSurface.withValues(alpha: 0.5)),
          ],
        ),
      ),
    );
  }
}

class _PluginCard extends StatelessWidget {
  final NodeTypeDef typeDef;
  const _PluginCard({required this.typeDef});

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);
    final color = ffNodeColor(typeDef.typeName);

    return Container(
      margin: const EdgeInsets.only(bottom: FlowForgeSpacing.sm),
      padding: const EdgeInsets.all(FlowForgeSpacing.md),
      decoration: BoxDecoration(
        border: Border.all(color: ext.border.primary),
        borderRadius: BorderRadius.circular(FlowForgeRadius.lg),
      ),
      child: Row(
        children: [
          FfSvg(ffNodeIcon(typeDef.typeName), size: 20, color: color),
          const SizedBox(width: FlowForgeSpacing.sm),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                FfText(typeDef.displayName, fontSize: FontSizes.s13, fontWeight: FontWeights.medium,
                  color: theme.colorScheme.onSurface),
                FfText(typeDef.typeName, fontSize: FontSizes.s11,
                  color: theme.colorScheme.onSurface.withValues(alpha: 0.4)),
              ],
            ),
          ),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
            decoration: BoxDecoration(
              color: color.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
            ),
            child: FfText(typeDef.category, fontSize: FontSizes.s10, color: color),
          ),
          const SizedBox(width: FlowForgeSpacing.sm),
          Container(
            width: 8, height: 8,
            decoration: BoxDecoration(
              color: ext.brandColor,
              shape: BoxShape.circle,
            ),
          ),
        ],
      ),
    );
  }
}
