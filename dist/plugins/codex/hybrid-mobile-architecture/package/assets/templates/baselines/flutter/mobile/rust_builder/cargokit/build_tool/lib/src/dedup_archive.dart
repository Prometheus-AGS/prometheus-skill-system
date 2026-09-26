// TJ-ARCH-MOB-001 compliant
/// Drops only byte-identical duplicate members from a thin static archive.
///
/// ONNX Runtime's prebuilt archive contains duplicate protobuf objects. Cargo
/// carries them into the Flutter staticlib and CocoaPods' `-force_load` turns
/// them into duplicate symbols. Same-name members with different content remain.
library;

import 'dart:io';
import 'dart:typed_data';

import 'package:logging/logging.dart';

final _log = Logger('dedup_archive');

int dedupArchiveMembers(String archivePath) {
  final bytes = File(archivePath).readAsBytesSync();
  const magic = '!<arch>\n';
  if (bytes.length < magic.length ||
      String.fromCharCodes(bytes.sublist(0, magic.length)) != magic) {
    _log.fine('$archivePath is not a thin ar archive; skipping dedup');
    return 0;
  }

  final output = BytesBuilder()..add(bytes.sublist(0, magic.length));
  final seen = <String, Set<String>>{};
  var removed = 0;
  var offset = magic.length;

  String contentKey(Uint8List content) {
    var hash = 0xcbf29ce484222325;
    for (final byte in content) {
      hash ^= byte;
      hash = (hash * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF;
    }
    return '${content.length}:$hash';
  }

  while (offset + 60 <= bytes.length) {
    final header = bytes.sublist(offset, offset + 60);
    if (header[58] != 0x60 || header[59] != 0x0A) {
      throw StateError(
        'malformed ar member header at offset $offset in $archivePath',
      );
    }
    final rawName = String.fromCharCodes(header.sublist(0, 16)).trimRight();
    final size = int.parse(
      String.fromCharCodes(header.sublist(48, 58)).trim(),
    );
    final bodyOffset = offset + 60;
    final paddedSize = size.isOdd ? size + 1 : size;
    final memberEnd = bodyOffset + size;
    if (memberEnd > bytes.length) {
      throw StateError('truncated ar member at offset $offset in $archivePath');
    }

    String name;
    Uint8List content;
    if (rawName.startsWith('#1/')) {
      final nameLength = int.parse(rawName.substring(3));
      name = String.fromCharCodes(
        bytes.sublist(bodyOffset, bodyOffset + nameLength),
      ).replaceAll('\x00', '');
      content = Uint8List.sublistView(
        bytes,
        bodyOffset + nameLength,
        memberEnd,
      );
    } else {
      name = rawName;
      content = Uint8List.sublistView(bytes, bodyOffset, memberEnd);
    }

    var keep = !name.startsWith('__.SYMDEF');
    if (keep) {
      final key = contentKey(content);
      final hashes = seen.putIfAbsent(name, () => <String>{});
      if (!hashes.add(key)) {
        keep = false;
        removed++;
        _log.info('dropping byte-identical archive member $name');
      }
    }

    if (keep) {
      final paddedEnd = bodyOffset + paddedSize;
      output.add(
        bytes.sublist(offset, paddedEnd > bytes.length ? memberEnd : paddedEnd),
      );
    }
    offset = bodyOffset + paddedSize;
  }

  if (removed > 0) {
    File(archivePath).writeAsBytesSync(output.toBytes());
  }
  return removed;
}
