import { existsSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'smol-toml';
import { assert, main, text } from './common.mjs';
await main(() => {
    assert(process.argv.length <= 4, 'usage: node scripts/gen-design-tokens.mjs <project-root> [tokens.toml]', 64);
    const root = resolve(process.argv[2] ?? '.'), source = process.argv[3] ?? resolve(dirname(fileURLToPath(import.meta.url)), '../assets/templates/design-tokens/tokens.toml');
    const t = parse(text(source));
    const banner = 'GENERATED FROM assets/templates/design-tokens/tokens.toml — DO NOT EDIT.', regen = 'Edit that file and re-run: node scripts/gen-design-tokens.mjs <root>';
    const groups = ['surface', 'text', 'accent', 'status'], prefixes = { surface: 'bg', text: 'text', accent: '', status: '' };
    const token = (group, name) => prefixes[group] ? prefixes[group] + name[0].toUpperCase() + name.slice(1) : name;
    const css = [`/* ${banner}`, `   ${regen} */`, '', '@theme {'];
    for (const group of groups)
        for (const [name, value] of Object.entries(t.dark[group] ?? {}))
            css.push(`  --color-${token(group, name)}: ${value};`);
    css.push('');
    for (const [key, value] of Object.entries(t.space))
        css.push(`  --spacing-${key}: ${value}px;`);
    css.push('', `  --font-display: '${t.font.display}', sans-serif;`, `  --font-sans: '${t.font.sans}', sans-serif;`, `  --font-prose: '${t.font.prose}', serif;`, `  --font-mono: '${t.font.mono}', monospace;`, '}', '', '/* Light theme. Same token NAMES, different values — so no component', '   ever branches on theme; it reads the role and gets the right colour. */', ':root[data-theme="light"] {');
    for (const group of groups)
        for (const [name, value] of Object.entries(t.light[group] ?? {}))
            css.push(`  --color-${token(group, name)}: ${value};`);
    css.push('}');
    const dart = ['// TJ-ARCH-MOB-001 compliant', `// ${banner}`, `// ${regen}`, "import 'package:flutter/material.dart';", "import 'package:google_fonts/google_fonts.dart';", '', '/// Design tokens. Names describe the ROLE a colour plays, never the colour', '/// itself — `accent` survives a rebrand, `ember` does not.', `abstract final class ${t.meta.dart_class} {`];
    const labels = { surface: 'Surfaces (background ladder)', text: 'Text (on-surface roles)', accent: 'Accents (interactive)', status: 'Status (semantic, never decorative)' };
    for (const group of groups) {
        dart.push(`  // ${labels[group]}`);
        for (const [theme, suffix] of [[t.dark, ''], [t.light, 'OnLight']])
            for (const [name, value] of Object.entries(theme[group] ?? {})) {
                assert(/^#[0-9a-fA-F]{6}$/.test(value), `invalid color: ${group}.${name}`);
                dart.push(`  static const ${token(group, name)}${suffix} = Color(0xFF${value.slice(1).toUpperCase()});`);
            }
        dart.push('');
    }
    dart.push('  // Deprecated colour-word aliases. Use the semantic name instead:', '  // a token named for a colour cannot survive a re-theme.');
    for (const [old, semantic] of [['amber', 'warning'], ['green', 'success'], ['red', 'danger'], ['cyan', 'info']])
        if (semantic in (t.dark.status ?? {}))
            dart.push(`  @Deprecated('Use ${semantic}')`, `  static const ${old} = ${semantic};`);
    dart.push('', '  // Spacing scale — off-scale padding is how rhythm dies.');
    for (const [key, value] of Object.entries(t.space))
        dart.push(`  static const double space${key.toUpperCase()} = ${value};`);
    dart.push('', '  // Typography');
    const fonts = { display: 'spaceGrotesk', sans: 'inter', prose: 'roboto', mono: 'jetBrainsMono' };
    for (const [name, spec] of Object.entries(t.type)) {
        assert(fonts[spec.family], `unknown font family: ${spec.family}`);
        const args = [`fontSize: ${spec.size}`, `fontWeight: FontWeight.w${spec.weight}`];
        if (spec.letter_spacing !== undefined)
            args.push(`letterSpacing: ${spec.letter_spacing}`);
        args.push(`color: ${token('text', spec.role)}`);
        if (spec.height !== undefined)
            args.push(`height: ${spec.height}`);
        dart.push(`  static TextStyle get ${name} => GoogleFonts.${fonts[spec.family]}(${args.join(', ')});`);
    }
    dart.push('', '  /// Flat 2.0 block surface: background fill only, never a border or', '  /// shadow. An accent is carried by a left edge bar, not an outline.', '  static BoxDecoration blockDecoration({', '    required Color bg,', '    Color? accent,', '  }) => BoxDecoration(', '        color: bg,', '        borderRadius: BorderRadius.circular(8),', '        border: accent == null', '            ? null', '            : Border(left: BorderSide(color: accent, width: 3)),', '      );', '}');
    for (const [path, lines] of [[join(root, 'desktop/src/theme.css'), css], [join(root, 'mobile/lib/core/theme/tokens.dart'), dart]])
        if (existsSync(dirname(path))) {
            writeFileSync(path, lines.join('\n') + '\n');
            console.log(`wrote ${relative(root, path)}`);
        }
    console.log('design tokens generated for available surfaces from one source');
});
