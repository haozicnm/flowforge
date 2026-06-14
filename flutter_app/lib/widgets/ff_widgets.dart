/// FlowForge Reusable Widget Library (ff_widgets)
///
/// AppFlowy pattern: every visual component is a reusable widget
/// with theme-awareness built in.
library;

import 'dart:async';
import 'dart:math';

import 'package:flutter/material.dart';
import '../theme/flowforge_theme.dart';

// ─────────────────────────────────────────────────────────────────
// Enums
// ─────────────────────────────────────────────────────────────────

enum FfButtonSize { sm, md, lg }

extension _FfButtonSizeExt on FfButtonSize {
  EdgeInsets get padding {
    switch (this) {
      case FfButtonSize.sm:
        return const EdgeInsets.symmetric(horizontal: 10, vertical: 4);
      case FfButtonSize.md:
        return const EdgeInsets.symmetric(horizontal: 14, vertical: 7);
      case FfButtonSize.lg:
        return const EdgeInsets.symmetric(horizontal: 18, vertical: 10);
    }
  }

  double get fontSize {
    switch (this) {
      case FfButtonSize.sm:  return FontSizes.s11;
      case FfButtonSize.md:  return FontSizes.s13;
      case FfButtonSize.lg:  return FontSizes.s14;
    }
  }
}

// ─────────────────────────────────────────────────────────────────
// FfButton — core builder-based button + convenience factories
// ─────────────────────────────────────────────────────────────────

class FfButton extends StatefulWidget {
  // Core builder (full flexibility — existing API, unchanged)
  final Widget Function(BuildContext context, bool isHovering) builder;
  final VoidCallback? onTap;
  final bool isSelected;
  final BorderRadius? radius;
  final Color? hoverColor;

  const FfButton({
    super.key,
    required this.builder,
    this.onTap,
    this.isSelected = false,
    this.radius,
    this.hoverColor,
  });

  /// Colored button (brand fill, white text).
  factory FfButton.primary({
    Key? key,
    required String label,
    IconData? icon,
    VoidCallback? onTap,
    FfButtonSize size = FfButtonSize.md,
  }) {
    return FfButton(
      key: key,
      onTap: onTap,
      builder: (context, hovering) {
        final ext = FlowForgeThemeExtension.of(context);
        return Container(
          padding: size.padding,
          decoration: BoxDecoration(
            color: hovering
                ? ext.brandColor.withValues(alpha: 0.85)
                : ext.brandColor,
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (icon != null) ...[
                Icon(icon, size: size.fontSize + 2, color: Colors.white),
                const SizedBox(width: 6),
              ],
              Text(label,
                style: TextStyle(
                  fontSize: size.fontSize,
                  fontWeight: FontWeights.medium,
                  color: Colors.white,
                )),
            ],
          ),
        );
      },
    );
  }

  /// Outlined button (border, no fill).
  factory FfButton.outlined({
    Key? key,
    required String label,
    IconData? icon,
    VoidCallback? onTap,
    FfButtonSize size = FfButtonSize.md,
  }) {
    return FfButton(
      key: key,
      onTap: onTap,
      builder: (context, hovering) {
        final ext = FlowForgeThemeExtension.of(context);
        final theme = Theme.of(context);
        return Container(
          padding: size.padding,
          decoration: BoxDecoration(
            border: Border.all(color: ext.border.primary),
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
            color: hovering ? ext.bg.secondary : Colors.transparent,
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (icon != null) ...[
                Icon(icon, size: size.fontSize + 2, color: ext.icon.secondary),
                const SizedBox(width: 6),
              ],
              Text(label,
                style: TextStyle(
                  fontSize: size.fontSize,
                  fontWeight: FontWeights.medium,
                  color: theme.colorScheme.onSurface,
                )),
            ],
          ),
        );
      },
    );
  }

  /// Text-only button (no border, no fill).
  factory FfButton.text({
    Key? key,
    required String label,
    IconData? icon,
    VoidCallback? onTap,
    Color? textColor,
    FfButtonSize size = FfButtonSize.md,
  }) {
    return FfButton(
      key: key,
      onTap: onTap,
      builder: (context, hovering) {
        final ext = FlowForgeThemeExtension.of(context);
        final color = textColor ?? ext.icon.primary;
        return Container(
          padding: size.padding,
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (icon != null) ...[
                Icon(icon, size: size.fontSize + 2, color: color.withValues(alpha: hovering ? 0.75 : 1)),
                const SizedBox(width: 6),
              ],
              Text(label,
                style: TextStyle(
                  fontSize: size.fontSize,
                  fontWeight: FontWeights.medium,
                  color: color.withValues(alpha: hovering ? 0.75 : 1),
                )),
            ],
          ),
        );
      },
    );
  }

  @override
  State<FfButton> createState() => _FfButtonState();
}

class _FfButtonState extends State<FfButton> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final radius = widget.radius ?? BorderRadius.circular(FlowForgeRadius.md);
    final hoverColor =
        widget.hoverColor ?? ext.brandColor.withValues(alpha: 0.05);

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          decoration: BoxDecoration(
            color: widget.isSelected
                ? ext.brandColor.withValues(alpha: 0.1)
                : _isHovering
                    ? hoverColor
                    : Colors.transparent,
            borderRadius: radius,
          ),
          child: widget.builder(context, _isHovering),
        ),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfText
// ─────────────────────────────────────────────────────────────────

class FfText extends StatelessWidget {
  final String text;
  final TextStyle? style;
  final Color? color;
  final double? fontSize;
  final FontWeight? fontWeight;
  final int? maxLines;
  final TextOverflow? overflow;

  const FfText(
    this.text, {
    super.key,
    this.style,
    this.color,
    this.fontSize,
    this.fontWeight,
    this.maxLines,
    this.overflow,
  });

  @override
  Widget build(BuildContext context) {
    return Text(
      text,
      style: (style ?? const TextStyle()).copyWith(
        color: color,
        fontSize: fontSize,
        fontWeight: fontWeight,
      ),
      maxLines: maxLines,
      overflow: overflow ?? TextOverflow.ellipsis,
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfDivider
// ─────────────────────────────────────────────────────────────────

class FfDivider extends StatelessWidget {
  final Axis direction;
  final double? length;
  final Color? color;

  const FfDivider({
    super.key,
    this.direction = Axis.horizontal,
    this.length,
    this.color,
  });

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final dividerColor = color ?? ext.border.primary;

    if (direction == Axis.horizontal) {
      return Container(
        height: 1,
        width: length ?? double.infinity,
        color: dividerColor,
      );
    }
    return Container(
      width: 1,
      height: length ?? double.infinity,
      color: dividerColor,
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfHover
// ─────────────────────────────────────────────────────────────────

class FfHover extends StatefulWidget {
  final Widget child;
  final VoidCallback? onTap;
  final Color? hoverColor;
  final BorderRadius? borderRadius;

  const FfHover({
    super.key,
    required this.child,
    this.onTap,
    this.hoverColor,
    this.borderRadius,
  });

  @override
  State<FfHover> createState() => _FfHoverState();
}

class _FfHoverState extends State<FfHover> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final hoverColor =
        widget.hoverColor ?? ext.brandColor.withValues(alpha: 0.05);

    return MouseRegion(
      cursor:
          widget.onTap != null ? SystemMouseCursors.click : MouseCursor.defer,
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 120),
          decoration: BoxDecoration(
            color: _isHovering ? hoverColor : Colors.transparent,
            borderRadius:
                widget.borderRadius ?? BorderRadius.circular(FlowForgeRadius.md),
          ),
          child: widget.child,
        ),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfTextField — themed text input
// ─────────────────────────────────────────────────────────────────

class FfTextField extends StatefulWidget {
  final TextEditingController? controller;
  final String? hintText;
  final String? labelText;
  final bool obscureText;
  final Widget? prefixIcon;
  final Widget? suffixIcon;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final String? errorText;
  final int? maxLines;
  final bool autofocus;
  final bool readOnly;
  final double? height;
  final BorderRadius? borderRadius;
  final EdgeInsets? contentPadding;
  final TextStyle? style;

  const FfTextField({
    super.key,
    this.controller,
    this.hintText,
    this.labelText,
    this.obscureText = false,
    this.prefixIcon,
    this.suffixIcon,
    this.onChanged,
    this.onSubmitted,
    this.errorText,
    this.maxLines = 1,
    this.autofocus = false,
    this.readOnly = false,
    this.height,
    this.borderRadius,
    this.contentPadding,
    this.style,
  });

  @override
  State<FfTextField> createState() => _FfTextFieldState();
}

class _FfTextFieldState extends State<FfTextField> {
  final FocusNode _focusNode = FocusNode();
  bool _focused = false;

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);
    final br = widget.borderRadius ?? BorderRadius.circular(FlowForgeRadius.md);

    final borderColor = widget.errorText != null
        ? FlowForgeColors.error
        : _focused
            ? ext.brandColor
            : ext.border.primary;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        if (widget.labelText != null) ...[
          Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Text(widget.labelText!,
              style: TextStyle(
                fontSize: FontSizes.s12,
                fontWeight: FontWeights.medium,
                color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
              )),
          ),
        ],
        Focus(
          onFocusChange: (f) => setState(() => _focused = f),
          child: Container(
            height: widget.height ?? 36,
            decoration: BoxDecoration(
              borderRadius: br,
              border: Border.all(color: borderColor, width: 1),
              color: ext.bg.primary,
            ),
            child: Row(
              children: [
                if (widget.prefixIcon != null) ...[
                  widget.prefixIcon!,
                  const SizedBox(width: 4),
                ],
                Expanded(
                  child: TextField(
                    controller: widget.controller,
                    focusNode: _focusNode,
                    obscureText: widget.obscureText,
                    readOnly: widget.readOnly,
                    autofocus: widget.autofocus,
                    maxLines: widget.maxLines,
                    onChanged: widget.onChanged,
                    onSubmitted: widget.onSubmitted,
                    style: (widget.style ?? const TextStyle()).copyWith(
                      fontSize: FontSizes.s13,
                      color: theme.colorScheme.onSurface,
                    ),
                    decoration: InputDecoration(
                      hintText: widget.hintText,
                      hintStyle: TextStyle(
                        fontSize: FontSizes.s13,
                        color: ext.icon.disabled,
                      ),
                      border: InputBorder.none,
                      contentPadding: widget.contentPadding ??
                          const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                    ),
                  ),
                ),
                if (widget.suffixIcon != null) ...[
                  widget.suffixIcon!,
                  const SizedBox(width: 4),
                ],
              ],
            ),
          ),
        ),
        if (widget.errorText != null)
          Padding(
            padding: const EdgeInsets.only(top: 4),
            child: Text(widget.errorText!,
              style: const TextStyle(
                fontSize: FontSizes.s11,
                color: FlowForgeColors.error,
              )),
          ),
      ],
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfDropdown — themed dropdown selector
// ─────────────────────────────────────────────────────────────────

class FfDropdown<T> extends StatelessWidget {
  final T? value;
  final List<FfDropdownItem<T>> items;
  final ValueChanged<T?>? onChanged;
  final String? hintText;
  final double? width;

  const FfDropdown({
    super.key,
    required this.value,
    required this.items,
    this.onChanged,
    this.hintText,
    this.width,
  });

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return SizedBox(
      width: width,
      child: DropdownButtonFormField<T>(
        value: value,
        isExpanded: true,
        decoration: InputDecoration(
          hintText: hintText,
          hintStyle: TextStyle(fontSize: FontSizes.s13, color: ext.icon.disabled),
          contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
            borderSide: BorderSide(color: ext.border.primary),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
            borderSide: BorderSide(color: ext.border.primary),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(FlowForgeRadius.md),
            borderSide: BorderSide(color: ext.brandColor, width: 1.5),
          ),
        ),
        style: TextStyle(fontSize: FontSizes.s13, color: theme.colorScheme.onSurface),
        dropdownColor: ext.bg.primary,
        icon: Icon(Icons.expand_more, size: 18, color: ext.icon.secondary),
        items: items.map((item) => DropdownMenuItem<T>(
          value: item.value,
          child: item.child ??
              Text(item.label, style: const TextStyle(fontSize: FontSizes.s13)),
        )).toList(),
        onChanged: onChanged,
      ),
    );
  }
}

class FfDropdownItem<T> {
  final T value;
  final String label;
  final Widget? child;

  const FfDropdownItem({required this.value, required this.label, this.child});
}

// ─────────────────────────────────────────────────────────────────
// FfDialog — unified modal dialog (AppFlowy FlowyDialog pattern)
// ─────────────────────────────────────────────────────────────────

class FfDialog extends StatelessWidget {
  final Widget child;
  final String? title;
  final double? width;
  final BoxConstraints? constraints;
  final List<Widget>? actions;

  const FfDialog({
    super.key,
    required this.child,
    this.title,
    this.width,
    this.constraints,
    this.actions,
  });

  /// Show this dialog modally.
  static Future<T?> show<T>({
    required BuildContext context,
    required Widget child,
    String? title,
    double? width,
    BoxConstraints? constraints,
    List<Widget>? actions,
    bool barrierDismissible = true,
  }) {
    return showDialog<T>(
      context: context,
      barrierDismissible: barrierDismissible,
      builder: (_) => FfDialog(
        title: title,
        width: width,
        constraints: constraints,
        actions: actions,
        child: child,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return Dialog(
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(FlowForgeRadius.xl),
      ),
      backgroundColor: ext.bg.primary,
      child: ConstrainedBox(
        constraints: constraints ?? const BoxConstraints(minWidth: 360, maxWidth: 520),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (title != null)
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 20, 20, 12),
                child: Text(title!,
                  style: TextStyle(
                    fontSize: FontSizes.s16,
                    fontWeight: FontWeights.semibold,
                    color: theme.colorScheme.onSurface,
                  )),
              ),
            Padding(
              padding: EdgeInsets.fromLTRB(
                20,
                title != null ? 0 : 20,
                20,
                actions != null ? 12 : 20,
              ),
              child: child,
            ),
            if (actions != null) ...[
              const FfDivider(),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: actions!.map((a) => Padding(
                    padding: const EdgeInsets.only(left: 8),
                    child: a,
                  )).toList(),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfToggle — themed toggle switch
// ─────────────────────────────────────────────────────────────────

class FfToggle extends StatelessWidget {
  final bool value;
  final ValueChanged<bool>? onChanged;
  final String? label;

  const FfToggle({
    super.key,
    required this.value,
    this.onChanged,
    this.label,
  });

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);
    final theme = Theme.of(context);

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          height: 24,
          child: Switch(
            value: value,
            onChanged: onChanged,
            activeTrackColor: ext.brandColor,
            materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
          ),
        ),
        if (label != null) ...[
          const SizedBox(width: 8),
          Text(label!,
            style: TextStyle(
              fontSize: FontSizes.s13,
              color: theme.colorScheme.onSurface,
            )),
        ],
      ],
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfTooltip — themed tooltip, supports plain text + rich TextSpan
// ─────────────────────────────────────────────────────────────────

class FfTooltip extends StatelessWidget {
  final String? message;
  final InlineSpan? richMessage;
  final Widget child;

  const FfTooltip({
    super.key,
    this.message,
    this.richMessage,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: message ?? '',
      richMessage: richMessage,
      preferBelow: false,
      textStyle: const TextStyle(fontSize: FontSizes.s12, color: Colors.white),
      decoration: BoxDecoration(
        color: const Color(0xFF333333),
        borderRadius: BorderRadius.circular(FlowForgeRadius.sm),
      ),
      child: child,
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FfToast — overlay notification (global show function)
// ─────────────────────────────────────────────────────────────────

/// Overlay toast — supports top/bottom alignment + optional action button.


enum FfToastType { success, error, warning, info }

class FfToast {
  final BuildContext context;
  final String message;
  final FfToastType type;
  final Duration duration;
  final Alignment alignment;
  final String? actionLabel;
  final VoidCallback? onAction;

  FfToast._({
    required this.context,
    required this.message,
    this.type = FfToastType.info,
    this.duration = const Duration(seconds: 2),
    this.alignment = Alignment.topCenter,
    this.actionLabel,
    this.onAction,
  });

  /// Show a toast notification.
  static void show(
    BuildContext context, {
    required String message,
    FfToastType type = FfToastType.info,
    Duration duration = const Duration(seconds: 2),
    Alignment alignment = Alignment.topCenter,
    String? actionLabel,
    VoidCallback? onAction,
  }) {
    final overlay = Overlay.of(context);
    late final OverlayEntry entry;
    entry = OverlayEntry(
      builder: (ctx) => _ToastWidget(
        message: message,
        type: type,
        alignment: alignment,
        actionLabel: actionLabel,
        onAction: onAction,
        onDismiss: () => entry.remove(),
      ),
    );
    overlay.insert(entry);
    Future.delayed(duration, () {
      if (entry.mounted) entry.remove();
    });
  }
}

class _ToastWidget extends StatefulWidget {
  final String message;
  final FfToastType type;
  final Alignment alignment;
  final String? actionLabel;
  final VoidCallback? onAction;
  final VoidCallback onDismiss;
  const _ToastWidget({
    required this.message,
    required this.type,
    required this.alignment,
    this.actionLabel,
    this.onAction,
    required this.onDismiss,
  });

  @override
  State<_ToastWidget> createState() => _ToastWidgetState();
}

class _ToastWidgetState extends State<_ToastWidget>
    with SingleTickerProviderStateMixin {
  late final AnimationController _ctrl;
  late final Animation<double> _anim;

  @override
  void initState() {
    super.initState();
    _ctrl = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 250),
    );
    _anim = CurvedAnimation(parent: _ctrl, curve: Curves.easeOut);
    _ctrl.forward();
  }

  @override
  void dispose() {
    _ctrl.dispose();
    super.dispose();
  }

  Color _bgColor() {
    switch (widget.type) {
      case FfToastType.success: return FlowForgeColors.success;
      case FfToastType.error:   return FlowForgeColors.error;
      case FfToastType.warning: return FlowForgeColors.warning;
      case FfToastType.info:    return const Color(0xFF333333);
    }
  }

  IconData _icon() {
    switch (widget.type) {
      case FfToastType.success: return Icons.check_circle;
      case FfToastType.error:   return Icons.error;
      case FfToastType.warning: return Icons.warning;
      case FfToastType.info:    return Icons.info;
    }
  }

  @override
  Widget build(BuildContext context) {
    final padding = widget.alignment == Alignment.topCenter
        ? const EdgeInsets.only(top: 48)
        : const EdgeInsets.only(bottom: 48);

    return FadeTransition(
      opacity: _anim,
      child: SlideTransition(
        position: Tween<Offset>(
          begin: widget.alignment == Alignment.topCenter
              ? const Offset(0, -0.3)
              : const Offset(0, 0.3),
          end: Offset.zero,
        ).animate(_anim),
        child: Material(
          color: Colors.transparent,
          child: Align(
            alignment: widget.alignment,
            child: Padding(
              padding: padding,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                decoration: BoxDecoration(
                  color: _bgColor(),
                  borderRadius: BorderRadius.circular(FlowForgeRadius.md),
                  boxShadow: const [
                    BoxShadow(
                      color: Colors.black26,
                      blurRadius: 8,
                      offset: Offset(0, 2),
                    ),
                  ],
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(_icon(), size: 16, color: Colors.white),
                    const SizedBox(width: 8),
                    Flexible(
                      child: Text(widget.message,
                        style: const TextStyle(
                          fontSize: FontSizes.s13,
                          fontWeight: FontWeights.medium,
                          color: Colors.white,
                        )),
                    ),
                    if (widget.actionLabel != null && widget.onAction != null) ...[
                      const SizedBox(width: 12),
                      GestureDetector(
                        onTap: () {
                          widget.onAction?.call();
                          widget.onDismiss();
                        },
                        child: Text(widget.actionLabel!,
                          style: const TextStyle(
                            fontSize: FontSizes.s12,
                            fontWeight: FontWeights.semibold,
                            color: Colors.white,
                            decoration: TextDecoration.underline,
                          )),
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// SidebarResizer — drag to resize sidebar width
// ─────────────────────────────────────────────────────────────────

class SidebarResizer extends StatefulWidget {
  final double initialWidth;
  final double minWidth;
  final double maxWidth;
  final Widget Function(BuildContext context, double width) builder;
  final ValueChanged<double>? onWidthChanged;

  const SidebarResizer({
    super.key,
    required this.initialWidth,
    required this.builder,
    this.minWidth = 160,
    this.maxWidth = 420,
    this.onWidthChanged,
  });

  @override
  State<SidebarResizer> createState() => _SidebarResizerState();
}

class _SidebarResizerState extends State<SidebarResizer> {
  late double _width;
  bool _dragging = false;
  bool _isHover = false;
  Timer? _showHoverTimer;

  @override
  void initState() {
    super.initState();
    _width = widget.initialWidth;
  }

  @override
  void dispose() {
    _showHoverTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);

    return Stack(
      children: [
        // Sidebar content with smooth animated width
        AnimatedSize(
          duration: _dragging ? Duration.zero : const Duration(milliseconds: 120),
          curve: Curves.easeOut,
          alignment: Alignment.centerLeft,
          child: SizedBox(
            width: _width,
            child: widget.builder(context, _width),
          ),
        ),
        // Resize handle (hover delay, animated color)
        Positioned(
          right: -4, top: 0, bottom: 0,
          width: 10,
          child: MouseRegion(
            cursor: SystemMouseCursors.resizeColumn,
            onEnter: (_) {
              _showHoverTimer = Timer(const Duration(milliseconds: 500), () {
                if (mounted) setState(() => _isHover = true);
              });
            },
            onExit: (_) {
              _showHoverTimer?.cancel();
              if (mounted) setState(() => _isHover = false);
            },
            child: GestureDetector(
              behavior: HitTestBehavior.translucent,
              onPanStart: (_) => setState(() => _dragging = true),
              onPanUpdate: (details) {
                setState(() {
                  _width = (_width + details.delta.dx)
                      .clamp(widget.minWidth, widget.maxWidth);
                });
                widget.onWidthChanged?.call(_width);
              },
              onPanEnd: (_) => setState(() => _dragging = false),
              child: Center(
                child: TweenAnimationBuilder<Color?>(
                  tween: ColorTween(
                    end: _isHover || _dragging
                        ? ext.brandColor
                        : ext.border.primary,
                  ),
                  duration: const Duration(milliseconds: 100),
                  builder: (context, color, _) => Container(
                    width: 2,
                    decoration: BoxDecoration(
                      color: color ?? ext.border.primary,
                      borderRadius: BorderRadius.circular(1),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

/// Sidebar resizer with AnimatedSize + hover delay (AppFlowy SecondaryViewResizer pattern).
class FfPanelResizer extends StatefulWidget {
  final double initialWidth;
  final double minWidth;
  final double maxWidth;
  final ValueNotifier<double> widthNotifier;

  const FfPanelResizer({
    super.key,
    required this.initialWidth,
    required this.widthNotifier,
    this.minWidth = 450,
    this.maxWidth = 800,
  });

  @override
  State<FfPanelResizer> createState() => _FfPanelResizerState();
}

class _FfPanelResizerState extends State<FfPanelResizer> {
  bool _isHover = false;
  bool _isDragging = false;
  Timer? _hoverTimer;

  @override
  void dispose() {
    _hoverTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ext = FlowForgeThemeExtension.of(context);

    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      onEnter: (_) {
        _hoverTimer = Timer(const Duration(milliseconds: 500), () {
          if (mounted) setState(() => _isHover = true);
        });
      },
      onExit: (_) {
        _hoverTimer?.cancel();
        if (mounted) setState(() => _isHover = false);
      },
      child: GestureDetector(
        behavior: HitTestBehavior.translucent,
        onHorizontalDragStart: (_) => setState(() => _isDragging = true),
        onHorizontalDragUpdate: (details) {
          final newWidth = MediaQuery.of(context).size.width -
              details.globalPosition.dx;
          if (newWidth >= widget.minWidth) {
            widget.widthNotifier.value = newWidth;
          }
        },
        onHorizontalDragEnd: (_) => setState(() => _isDragging = false),
        child: TweenAnimationBuilder(
          tween: ColorTween(
            end: _isHover || _isDragging
                ? ext.brandColor
                : Colors.transparent,
          ),
          duration: const Duration(milliseconds: 100),
          builder: (context, color, child) {
            return SizedBox(
              width: 11,
              child: Center(
                child: Container(
                  color: color,
                  width: 2,
                  height: double.infinity,
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// FadingIndexedStack — AppFlowy pattern: IndexedStack + fade
// ─────────────────────────────────────────────────────────────────

class FadingIndexedStack extends StatefulWidget {
  final int index;
  final List<Widget> children;
  final Duration duration;

  const FadingIndexedStack({
    super.key,
    required this.index,
    required this.children,
    this.duration = const Duration(milliseconds: 250),
  });

  @override
  State<FadingIndexedStack> createState() => _FadingIndexedStackState();
}

class _FadingIndexedStackState extends State<FadingIndexedStack> {
  double _targetOpacity = 1;

  @override
  void didUpdateWidget(FadingIndexedStack oldWidget) {
    if (oldWidget.index == widget.index) return;
    _targetOpacity = 0;
    WidgetsBinding.instance.addPostFrameCallback(
      (_) => setState(() => _targetOpacity = 1),
    );
    super.didUpdateWidget(oldWidget);
  }

  @override
  Widget build(BuildContext context) {
    return TweenAnimationBuilder<double>(
      duration: _targetOpacity > 0 ? widget.duration : Duration.zero,
      tween: Tween(begin: 0, end: _targetOpacity),
      builder: (context, value, child) =>
          Opacity(opacity: value, child: child),
      child: IndexedStack(index: widget.index, children: widget.children),
    );
  }
}

// ─────────────────────────────────────────────────────────────────
// SeparatedColumn — AppFlowy pattern: Column with auto-spacing
// ─────────────────────────────────────────────────────────────────

class SeparatedColumn extends StatelessWidget {
  final List<Widget> children;
  final MainAxisSize mainAxisSize;
  final CrossAxisAlignment crossAxisAlignment;
  final Widget Function() separatorBuilder;

  const SeparatedColumn({
    super.key,
    required this.children,
    this.mainAxisSize = MainAxisSize.max,
    this.crossAxisAlignment = CrossAxisAlignment.start,
    this.separatorBuilder = _defaultSeparator,
  });

  static Widget _defaultSeparator() => const SizedBox(height: 8);

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: mainAxisSize,
      crossAxisAlignment: crossAxisAlignment,
      children: _interleave(),
    );
  }

  List<Widget> _interleave() {
    if (children.isEmpty) return [];
    final result = <Widget>[children.first];
    for (var i = 1; i < children.length; i++) {
      result.add(separatorBuilder());
      result.add(children[i]);
    }
    return result;
  }
}
