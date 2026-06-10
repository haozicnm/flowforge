/// FlowForge Design System — AppFlowy-aligned multi-level color tokens.
///
/// Usage:
/// ```dart
/// final theme = FlowForgeTheme.of(context);
/// theme.bg.primary      // main background
/// theme.bg.secondary    // card / hover surface
/// theme.border.primary  // main divider
/// theme.icon.primary    // main icon color
/// ```
///
/// Backward-compat shortcuts still work: theme.brandColor, theme.surfaceColor, etc.
library;

import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

// ─── Raw color palette ────────────────────────────────────────────

abstract class FlowForgeColors {
  // Brand
  static const brand = Color(0xFF00B4D8);
  static const brandLight = Color(0xFF90E0EF);
  static const brandDark = Color(0xFF0077B6);

  // Surfaces — light
  static const surfaceLight = Color(0xFFF8F9FA);
  static const surfaceLightSecondary = Color(0xFFF0F2F5);
  static const surfaceLightTertiary = Color(0xFFE8EBEE);

  // Surfaces — dark
  static const surfaceDark = Color(0xFF1A1A2E);
  static const surfaceDarkSecondary = Color(0xFF22223A);
  static const surfaceDarkTertiary = Color(0xFF2A2A44);

  // Borders
  static const borderLight = Color(0xFFDEE2E6);
  static const borderLightSecondary = Color(0xFFE9ECEF);
  static const borderDark = Color(0xFF333355);
  static const borderDarkSecondary = Color(0xFF2A2A44);

  // Icons
  static const iconLight = Color(0xFF212529);
  static const iconLightSecondary = Color(0xFF6C757D);
  static const iconLightDisabled = Color(0xFFADB5BD);
  static const iconDark = Color(0xFFE8E2EE);
  static const iconDarkSecondary = Color(0xFFA0A0B8);
  static const iconDarkDisabled = Color(0xFF555570);

  // Semantic
  static const success = Color(0xFF28A745);
  static const warning = Color(0xFFFFC107);
  static const error = Color(0xFFDC3545);
}

// ─── Sub-scheme classes ───────────────────────────────────────────

class FlowForgeBgColors {
  final Color primary;
  final Color secondary;
  final Color tertiary;

  const FlowForgeBgColors({
    required this.primary,
    required this.secondary,
    required this.tertiary,
  });
}

class FlowForgeBorderColors {
  final Color primary;
  final Color secondary;

  const FlowForgeBorderColors({
    required this.primary,
    required this.secondary,
  });
}

class FlowForgeIconColors {
  final Color primary;
  final Color secondary;
  final Color disabled;

  const FlowForgeIconColors({
    required this.primary,
    required this.secondary,
    required this.disabled,
  });
}

// ─── Spacing & radius ─────────────────────────────────────────────

abstract class FlowForgeSpacing {
  static const double xs = 4;
  static const double sm = 8;
  static const double md = 16;
  static const double lg = 24;
  static const double xl = 32;
  static const double xxl = 48;
}

abstract class FlowForgeRadius {
  static const double sm = 4;
  static const double md = 8;
  static const double lg = 12;
  static const double xl = 16;
}

// ─── Typography constants ─────────────────────────────────────────

abstract class FontSizes {
  static const double s10 = 10;
  static const double s11 = 11;
  static const double s12 = 12;
  static const double s13 = 13;
  static const double s14 = 14;
  static const double s16 = 16;
  static const double s18 = 18;
  static const double s20 = 20;
  static const double s22 = 22;
  static const double s24 = 24;
  static const double s28 = 28;
  static const double s36 = 36;
}

abstract class FontWeights {
  static const FontWeight light = FontWeight.w300;
  static const FontWeight regular = FontWeight.w400;
  static const FontWeight medium = FontWeight.w500;
  static const FontWeight semibold = FontWeight.w600;
  static const FontWeight bold = FontWeight.w700;
}

/// Returns the Poppins text style for the given context.
TextStyle poppinsStyle(
  BuildContext context, {
  double? fontSize,
  FontWeight? fontWeight,
  Color? color,
  double? height,
  double? letterSpacing,
}) {
  return GoogleFonts.poppins(
    fontSize: fontSize,
    fontWeight: fontWeight,
    color: color,
    height: height,
    letterSpacing: letterSpacing,
  );
}

// ─── Theme extension ──────────────────────────────────────────────

class FlowForgeThemeExtension extends ThemeExtension<FlowForgeThemeExtension> {
  // New sub-schemes
  final FlowForgeBgColors bg;
  final FlowForgeBorderColors border;
  final FlowForgeIconColors icon;

  // Layout tokens
  final double sidebarWidth;
  final double topBarHeight;

  const FlowForgeThemeExtension({
    required this.bg,
    required this.border,
    required this.icon,
    this.sidebarWidth = 220,
    this.topBarHeight = 52,
  });

  // ── Backward-compat shortcuts ────────────────────────────────
  Color get brandColor => icon.primary;
  Color get surfaceColor => bg.primary;
  Color get borderColor => border.primary;

  // ── Convenience accessor ─────────────────────────────────────
  static FlowForgeThemeExtension of(BuildContext context) =>
      Theme.of(context).extension<FlowForgeThemeExtension>()!;

  @override
  FlowForgeThemeExtension copyWith({
    FlowForgeBgColors? bg,
    FlowForgeBorderColors? border,
    FlowForgeIconColors? icon,
    double? sidebarWidth,
    double? topBarHeight,
  }) {
    return FlowForgeThemeExtension(
      bg: bg ?? this.bg,
      border: border ?? this.border,
      icon: icon ?? this.icon,
      sidebarWidth: sidebarWidth ?? this.sidebarWidth,
      topBarHeight: topBarHeight ?? this.topBarHeight,
    );
  }

  @override
  FlowForgeThemeExtension lerp(
    covariant ThemeExtension<FlowForgeThemeExtension>? other,
    double t,
  ) {
    if (other is! FlowForgeThemeExtension) return this;
    return FlowForgeThemeExtension(
      bg: FlowForgeBgColors(
        primary: Color.lerp(bg.primary, other.bg.primary, t)!,
        secondary: Color.lerp(bg.secondary, other.bg.secondary, t)!,
        tertiary: Color.lerp(bg.tertiary, other.bg.tertiary, t)!,
      ),
      border: FlowForgeBorderColors(
        primary: Color.lerp(border.primary, other.border.primary, t)!,
        secondary: Color.lerp(border.secondary, other.border.secondary, t)!,
      ),
      icon: FlowForgeIconColors(
        primary: Color.lerp(icon.primary, other.icon.primary, t)!,
        secondary: Color.lerp(icon.secondary, other.icon.secondary, t)!,
        disabled: Color.lerp(icon.disabled, other.icon.disabled, t)!,
      ),
      sidebarWidth: sidebarWidth,
      topBarHeight: topBarHeight,
    );
  }
}

// ─── Themes ──────────────────────────────────────────────────────

ThemeData buildLightTheme() {
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: ColorScheme.fromSeed(
      seedColor: FlowForgeColors.brand,
      brightness: Brightness.light,
    ),
    scaffoldBackgroundColor: FlowForgeColors.surfaceLight,
    textTheme: GoogleFonts.poppinsTextTheme(),
    extensions: const [
      FlowForgeThemeExtension(
        bg: FlowForgeBgColors(
          primary: FlowForgeColors.surfaceLight,
          secondary: FlowForgeColors.surfaceLightSecondary,
          tertiary: FlowForgeColors.surfaceLightTertiary,
        ),
        border: FlowForgeBorderColors(
          primary: FlowForgeColors.borderLight,
          secondary: FlowForgeColors.borderLightSecondary,
        ),
        icon: FlowForgeIconColors(
          primary: FlowForgeColors.brand,
          secondary: FlowForgeColors.iconLightSecondary,
          disabled: FlowForgeColors.iconLightDisabled,
        ),
      ),
    ],
  );
}

ThemeData buildDarkTheme() {
  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: ColorScheme.fromSeed(
      seedColor: FlowForgeColors.brand,
      brightness: Brightness.dark,
    ),
    scaffoldBackgroundColor: FlowForgeColors.surfaceDark,
    textTheme: GoogleFonts.poppinsTextTheme(ThemeData.dark().textTheme),
    extensions: const [
      FlowForgeThemeExtension(
        bg: FlowForgeBgColors(
          primary: FlowForgeColors.surfaceDark,
          secondary: FlowForgeColors.surfaceDarkSecondary,
          tertiary: FlowForgeColors.surfaceDarkTertiary,
        ),
        border: FlowForgeBorderColors(
          primary: FlowForgeColors.borderDark,
          secondary: FlowForgeColors.borderDarkSecondary,
        ),
        icon: FlowForgeIconColors(
          primary: FlowForgeColors.brandLight,
          secondary: FlowForgeColors.iconDarkSecondary,
          disabled: FlowForgeColors.iconDarkDisabled,
        ),
      ),
    ],
  );
}
