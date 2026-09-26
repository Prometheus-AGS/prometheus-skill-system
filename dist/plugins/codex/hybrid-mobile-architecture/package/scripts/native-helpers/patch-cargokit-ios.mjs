import { existsSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, main, text } from './common.mjs';
import { versions } from '../portable/versions.mjs';
await main(() => {
    const dir = dirname(fileURLToPath(import.meta.url));
    const pinFile = process.env.VERSIONS_TOML ?? [resolve(dir, '../versions.toml'), resolve(dir, '../rust/rust-toolchain.toml'), resolve(dir, '../rust-toolchain.toml')].find(existsSync);
    assert(pinFile, 'canonical Rust pin not found; supply versions.toml, generated rust-toolchain.toml, or VERSIONS_TOML');
    const pins = versions(pinFile), rust = pins.toolchain?.rust ?? pins.toolchain?.channel;
    assert(rust && /^\d+\.\d+\.\d+$/.test(rust), 'canonical Rust pin must be an exact x.y.z version');
    const template = existsSync(join(dir, 'dedup_archive.dart')) ? join(dir, 'dedup_archive.dart') : resolve(dir, '../assets/templates/cargokit/dedup_archive.dart');
    const target = join(process.argv[2] ?? 'mobile', 'rust_builder/cargokit/build_tool/lib/src');
    assert(existsSync(join(target, 'build_pod.dart')), 'Cargokit build_pod.dart not found; run flutter_rust_bridge_codegen first.');
    let options = text(join(target, 'options.dart'));
    options = options.replace('enum Toolchain {\n  stable,\n  beta,\n  nightly,\n}\n\n', '').replace('final Toolchain toolchain;', 'final String toolchain;');
    const oldParser = `static Toolchain _toolchainFromNode(YamlNode node) {
    if (node case YamlScalar(value: String name)) {
      final toolchain =
          Toolchain.values.firstWhereOrNull((element) => element.name == name);
      if (toolchain != null) {
        return toolchain;
      }
    }
    throw SourceSpanException(
        'Unknown toolchain. Must be one of \${Toolchain.values.map((e) => e.name)}.',
        node.span);
  }`;
    const newParser = `static String _toolchainFromNode(YamlNode node) {
    if (node case YamlScalar(value: String name)) {
      if (name.isNotEmpty && RegExp(r'^[A-Za-z0-9._-]+$').hasMatch(name)) {
        return name;
      }
    }
    throw SourceSpanException(
        'Invalid rustup toolchain. Expected a version, channel, or named toolchain.',
        node.span);
  }`;
    // Replacement callbacks keep `$&`, `$'`, and related sequences inside the
    // Dart regular expression literal from being interpreted by JavaScript.
    options = options.replace(oldParser, () => newParser).replace(/static Toolchain _toolchainFromNode\(YamlNode node\) \{[\s\S]*?node\.span\);\s*\}/, () => newParser).replace(/(?:Toolchain toolchain = Toolchain\.stable|String toolchain = '[^']+');/, `String toolchain = '${rust}';`);
    assert(options.includes(`String toolchain = '${rust}';`) && options.includes('static String _toolchainFromNode'), 'failed to pin Cargokit options toolchain; unsupported vendor source');
    const builder = text(join(target, 'builder.dart')).replace(/String get _toolchain => _buildOptions\?\.toolchain(?:\.name)? \?\? '[^']+';/, `String get _toolchain => _buildOptions?.toolchain ?? '${rust}';`);
    assert(builder.includes(`?? '${rust}';`), 'failed to pin Cargokit builder toolchain');
    const rustup = text(join(target, 'rustup.dart')).replace('Pattern nonCustom = RegExp(r"^(stable|beta|nightly)");', 'Pattern nonCustom = RegExp(r"^(stable|beta|nightly|[0-9])");');
    assert(rustup.includes('nightly|[0-9]'), 'failed to retain version-pinned rustup toolchains');
    let pod = text(join(target, 'build_pod.dart'));
    const marker = "import 'builder.dart';\n", importLine = "import 'dedup_archive.dart';\n", callMarker = '    if (staticLibs.isNotEmpty) {\n';
    if (!pod.includes(importLine)) {
        assert(pod.includes(marker), 'build_pod import marker missing');
        pod = pod.replace(marker, marker + importLine);
    }
    if (!pod.includes('dedupArchiveMembers(lib.path)')) {
        assert(pod.includes(callMarker), 'build_pod static library marker missing');
        pod = pod.replace(callMarker, `${callMarker}      for (final lib in staticLibs) {\n        if (dedupArchiveMembers(lib.path) > 0) {\n          runCommand("ranlib", [lib.path]);\n        }\n      }\n`);
    }
    const dedup = text(template);
    for (const [file, content] of [['options.dart', options], ['builder.dart', builder], ['rustup.dart', rustup], ['build_pod.dart', pod], ['dedup_archive.dart', dedup]])
        writeFileSync(join(target, file), content);
    console.log(`Patched Cargokit Rust ${rust} pin and archive dedup: ${join(target, 'build_pod.dart')}`);
});
