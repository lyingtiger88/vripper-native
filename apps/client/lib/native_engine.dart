import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

typedef _VersionNative = Pointer<Int8> Function();
typedef _VersionDart = Pointer<Int8> Function();
typedef _AbiNative = Uint32 Function();
typedef _AbiDart = int Function();
typedef _ScanNative = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _ScanDart = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _FreeNative = Void Function(Pointer<Int8>);
typedef _FreeDart = void Function(Pointer<Int8>);

class NativeEngine {
  NativeEngine._();

  static final NativeEngine instance = NativeEngine._();

  DynamicLibrary? _library;
  _VersionDart? _version;
  _AbiDart? _abiVersion;
  _ScanDart? _scanLinks;
  _FreeDart? _freeString;
  String? loadError;

  bool get isLoaded => _library != null;

  void load() {
    if (_library != null || loadError != null) return;
    try {
      final library = _openLibrary();
      _library = library;
      _version = library.lookupFunction<_VersionNative, _VersionDart>('vripper_engine_version');
      _abiVersion = library.lookupFunction<_AbiNative, _AbiDart>('vripper_abi_version');
      _scanLinks = library.lookupFunction<_ScanNative, _ScanDart>('vripper_scan_links_json');
      _freeString = library.lookupFunction<_FreeNative, _FreeDart>('vripper_string_free');
      if (_abiVersion!() != 1) throw StateError('Unsupported native engine ABI version');
    } catch (error) {
      _library = null;
      loadError = error.toString();
    }
  }

  DynamicLibrary _openLibrary() {
    if (Platform.isIOS) return DynamicLibrary.process();
    if (Platform.isWindows) return DynamicLibrary.open('vripper_core.dll');
    if (Platform.isMacOS) return DynamicLibrary.open('libvripper_core.dylib');
    if (Platform.isAndroid || Platform.isLinux) return DynamicLibrary.open('libvripper_core.so');
    throw UnsupportedError('This platform is not supported by the native engine');
  }

  String get version {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final pointer = _version!();
    try {
      return pointer.cast<Utf8>().toDartString();
    } finally {
      _freeString!(pointer);
    }
  }

  Map<String, dynamic> scanLinks(String input, List<String> configuredBases) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final inputPointer = input.toNativeUtf8().cast<Int8>();
    final basesPointer = jsonEncode(configuredBases).toNativeUtf8().cast<Int8>();
    Pointer<Int8> output = nullptr;
    try {
      output = _scanLinks!(inputPointer, basesPointer);
      if (output == nullptr) throw StateError('Native engine returned a null result');
      final decoded = jsonDecode(output.cast<Utf8>().toDartString());
      if (decoded is! Map<String, dynamic>) throw const FormatException('Unexpected native scan response');
      if (decoded.containsKey('error')) throw StateError(decoded['error'].toString());
      return decoded;
    } finally {
      malloc.free(inputPointer);
      malloc.free(basesPointer);
      if (output != nullptr) _freeString!(output);
    }
  }
}
