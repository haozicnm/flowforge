/// FfLayout — single-pass layout computation.
///
/// AppFlowy HomeLayout pattern: compute ALL layout values in one place,
/// then pass the `layout` object down instead of scattering MediaQuery calls.
library;

import 'package:flutter/material.dart';
import '../theme/flowforge_theme.dart';

/// Layout state computed from sidebar + screen width.
class FfLayout {
  FfLayout({
    required this.sidebarWidth,
    this.editPanelWidth = 300,
  }) {
    // Pre-compute offsets
    mainLeftOffset = sidebarWidth + 1;
    mainRightOffset = editPanelWidth;
  }

  /// Current sidebar width (set from SidebarResizer / user drag).
  final double sidebarWidth;

  /// Width of the right-side property/edit panel.
  final double editPanelWidth;

  /// Left margin for main content (sidebar width + 1px divider).
  double mainLeftOffset;

  /// Right margin for main content (edit panel width).
  double mainRightOffset;

  /// Main content width after subtracting both offsets.
  double mainContentWidth(double screenWidth) =>
      screenWidth - mainLeftOffset - mainRightOffset;

  /// Whether layout considers this a "narrow" screen (< 800px).
  bool get isNarrow => sidebarWidth < 160;

  @override
  String toString() =>
      'FfLayout(sidebar=$sidebarWidth, editPanel=$editPanelWidth)';
}

/// Builds an FfLayout from context (reads ThemeExtension for defaults).
extension FfLayoutOf on BuildContext {
  FfLayout ffLayout() {
    final ext = FlowForgeThemeExtension.of(this);
    return FfLayout(sidebarWidth: ext.sidebarWidth);
  }
}
