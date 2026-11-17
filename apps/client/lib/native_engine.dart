import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';

typedef _VersionNative = Pointer<Int8> Function();
typedef _VersionDart = Pointer<Int8> Function();
typedef _AbiNative = Uint32 Function();
typedef _AbiDart = int Function();
typedef _ScanNative = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _ScanDart = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _ParseNative = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _ParseDart = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _ResolveNative = Pointer<Int8> Function(Uint8, Pointer<Int8>, Pointer<Int8>, Pointer<Int8>, Pointer<Int8>, Uint8);
typedef _ResolveDart = Pointer<Int8> Function(int, Pointer<Int8>, Pointer<Int8>, Pointer<Int8>, Pointer<Int8>, int);
typedef _FreeNative = Void Function(Pointer<Int8>);
typedef _CoordinatorCreateNative = Pointer<Int8> Function(Pointer<Int8>);
typedef _CoordinatorCreateDart = Pointer<Int8> Function(Pointer<Int8>);
typedef _ScanQueueNative = Pointer<Int8> Function(Uint64, Pointer<Int8>, Pointer<Int8>);
typedef _ScanQueueDart = Pointer<Int8> Function(int, Pointer<Int8>, Pointer<Int8>);
typedef _HandleNative = Pointer<Int8> Function(Uint64);
typedef _HandleDart = Pointer<Int8> Function(int);
typedef _StopNative = Pointer<Int8> Function(Uint64, Uint64);
typedef _StopDart = Pointer<Int8> Function(int, int);
typedef _MoveNative = Pointer<Int8> Function(Uint64, Uint64, Pointer<Int8>);
typedef _MoveDart = Pointer<Int8> Function(int, int, Pointer<Int8>);
typedef _PollNative = Pointer<Int8> Function(Uint64, Uint64);
typedef _PollDart = Pointer<Int8> Function(int, int);
typedef _FreeDart = void Function(Pointer<Int8>);
typedef _DatabaseInspectNative = Pointer<Int8> Function(Pointer<Int8>);
typedef _DatabaseInspectDart = Pointer<Int8> Function(Pointer<Int8>);
typedef _DatabaseImportNative = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);
typedef _DatabaseImportDart = Pointer<Int8> Function(Pointer<Int8>, Pointer<Int8>);

class NativeEngine {
  NativeEngine._();

  static final NativeEngine instance = NativeEngine._();

  DynamicLibrary? _library;
  _VersionDart? _version;
  _AbiDart? _abiVersion;
  _ScanDart? _scanLinks;
  _ParseDart? _parseThread;
  _ResolveDart? _resolveHost;
  _FreeDart? _freeString;
  _CoordinatorCreateDart? _createCoordinator;
  _ScanQueueDart? _scanAndQueue;
  _HandleDart? _snapshot;
  _StopDart? _stop;
  _HandleDart? _stopAll;
  _MoveDart? _move;
  _HandleDart? _clearFinished;
  _PollDart? _pollEvent;
  _HandleDart? _destroyCoordinator;
  _DatabaseInspectDart? _databaseInspect;
  _DatabaseInspectDart? _databaseCatalog;
  _DatabaseImportDart? _databaseImport;
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
      _parseThread = library.lookupFunction<_ParseNative, _ParseDart>('vripper_parse_thread_response_json');
      _resolveHost = library.lookupFunction<_ResolveNative, _ResolveDart>('vripper_resolve_host_html_json');
      _freeString = library.lookupFunction<_FreeNative, _FreeDart>('vripper_string_free');
      _createCoordinator = library.lookupFunction<_CoordinatorCreateNative, _CoordinatorCreateDart>('vripper_coordinator_create_json');
      _scanAndQueue = library.lookupFunction<_ScanQueueNative, _ScanQueueDart>('vripper_coordinator_scan_and_queue_json');
      _snapshot = library.lookupFunction<_HandleNative, _HandleDart>('vripper_coordinator_snapshot_json');
      _stop = library.lookupFunction<_StopNative, _StopDart>('vripper_coordinator_stop_json');
      _stopAll = library.lookupFunction<_HandleNative, _HandleDart>('vripper_coordinator_stop_all_json');
      _move = library.lookupFunction<_MoveNative, _MoveDart>('vripper_coordinator_move_json');
      _clearFinished = library.lookupFunction<_HandleNative, _HandleDart>('vripper_coordinator_clear_finished_json');
      _pollEvent = library.lookupFunction<_PollNative, _PollDart>('vripper_coordinator_poll_event_json');
      _destroyCoordinator = library.lookupFunction<_HandleNative, _HandleDart>('vripper_coordinator_destroy_json');
      _databaseInspect = library.lookupFunction<_DatabaseInspectNative, _DatabaseInspectDart>('vripper_database_inspect_json');
      _databaseCatalog = library.lookupFunction<_DatabaseInspectNative, _DatabaseInspectDart>('vripper_database_catalog_json');
      _databaseImport = library.lookupFunction<_DatabaseImportNative, _DatabaseImportDart>('vripper_database_import_json');
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

  Map<String, dynamic> parseThreadResponse(String xml, String siteProxy) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final xmlPointer = xml.toNativeUtf8().cast<Int8>();
    final proxyPointer = siteProxy.toNativeUtf8().cast<Int8>();
    Pointer<Int8> output = nullptr;
    try {
      output = _parseThread!(xmlPointer, proxyPointer);
      return _decodeObject(output);
    } finally {
      malloc.free(xmlPointer);
      malloc.free(proxyPointer);
      if (output != nullptr) _freeString!(output);
    }
  }

  Map<String, dynamic> resolveHostHtml({
    required int hostId,
    required String sourceUrl,
    required String thumbnailUrl,
    required String html,
    String? followUpHtml,
    bool fetchOriginalImxFilename = false,
  }) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final sourcePointer = sourceUrl.toNativeUtf8().cast<Int8>();
    final thumbPointer = thumbnailUrl.toNativeUtf8().cast<Int8>();
    final htmlPointer = html.toNativeUtf8().cast<Int8>();
    final followUpPointer = followUpHtml?.toNativeUtf8().cast<Int8>() ?? nullptr;
    Pointer<Int8> output = nullptr;
    try {
      output = _resolveHost!(
        hostId,
        sourcePointer,
        thumbPointer,
        htmlPointer,
        followUpPointer,
        fetchOriginalImxFilename ? 1 : 0,
      );
      return _decodeObject(output);
    } finally {
      malloc.free(sourcePointer);
      malloc.free(thumbPointer);
      malloc.free(htmlPointer);
      if (followUpPointer != nullptr) malloc.free(followUpPointer);
      if (output != nullptr) _freeString!(output);
    }
  }

  Map<String, dynamic> readLegacyCatalog(String path) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final pointer = path.toNativeUtf8().cast<Int8>();
    try {
      return _decodeOwnedObject(_databaseCatalog!(pointer));
    } finally {
      malloc.free(pointer);
    }
  }

  Future<Map<String, dynamic>> readLegacyCatalogInBackground(String path) =>
      Isolate.run(() => NativeEngine.instance.readLegacyCatalog(path));

  Map<String, dynamic> inspectLegacyDatabase(String path) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final pathPointer = path.toNativeUtf8().cast<Int8>();
    try {
      return _decodeOwnedObject(_databaseInspect!(pathPointer));
    } finally {
      malloc.free(pathPointer);
    }
  }

  Map<String, dynamic> importLegacyDatabase(String sourcePath, String destinationPath) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final sourcePointer = sourcePath.toNativeUtf8().cast<Int8>();
    final destinationPointer = destinationPath.toNativeUtf8().cast<Int8>();
    try {
      return _decodeOwnedObject(_databaseImport!(sourcePointer, destinationPointer));
    } finally {
      malloc.free(sourcePointer);
      malloc.free(destinationPointer);
    }
  }

  Future<Map<String, dynamic>> inspectLegacyDatabaseInBackground(String path) =>
      Isolate.run(() => NativeEngine.instance.inspectLegacyDatabase(path));

  Future<Map<String, dynamic>> importLegacyDatabaseInBackground(String source, String destination) =>
      Isolate.run(() => NativeEngine.instance.importLegacyDatabase(source, destination));

  Map<String, dynamic> _decodeObject(Pointer<Int8> output) {
    if (output == nullptr) throw StateError('Native engine returned a null result');
    final decoded = jsonDecode(output.cast<Utf8>().toDartString());
    if (decoded is! Map<String, dynamic>) {
      throw const FormatException('Unexpected native engine response');
    }
    if (decoded.containsKey('error')) throw StateError(decoded['error'].toString());
    return decoded;
  }


  Map<String, dynamic> _decodeOwnedObject(Pointer<Int8> output) {
    try {
      return _decodeObject(output);
    } finally {
      if (output != nullptr) _freeString!(output);
    }
  }

  int createCoordinator(Map<String, dynamic> config) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final pointer = jsonEncode(config).toNativeUtf8().cast<Int8>();
    try {
      final response = _decodeOwnedObject(_createCoordinator!(pointer));
      final handle = response['handle'];
      if (handle is! int) throw const FormatException('Coordinator handle missing');
      return handle;
    } finally {
      malloc.free(pointer);
    }
  }

  Future<int> createCoordinatorInBackground(Map<String, dynamic> config) =>
      Isolate.run(() => NativeEngine.instance.createCoordinator(config));

  Map<String, dynamic> scanAndQueue(int handle, String input, List<String> bases) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final inputPointer = input.toNativeUtf8().cast<Int8>();
    final basesPointer = jsonEncode(bases).toNativeUtf8().cast<Int8>();
    try {
      return _decodeOwnedObject(_scanAndQueue!(handle, inputPointer, basesPointer));
    } finally {
      malloc.free(inputPointer);
      malloc.free(basesPointer);
    }
  }

  Future<Map<String, dynamic>> scanAndQueueInBackground(
    int handle,
    String input,
    List<String> bases,
  ) =>
      Isolate.run(() => NativeEngine.instance.scanAndQueue(handle, input, bases));

  Map<String, dynamic> queueSnapshot(int handle) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    return _decodeOwnedObject(_snapshot!(handle));
  }

  Map<String, dynamic> stopJob(int handle, int jobId) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    return _decodeOwnedObject(_stop!(handle, jobId));
  }

  Map<String, dynamic> stopAll(int handle) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    return _decodeOwnedObject(_stopAll!(handle));
  }

  Map<String, dynamic> moveJob(int handle, int jobId, String movement) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    final pointer = movement.toNativeUtf8().cast<Int8>();
    try {
      return _decodeOwnedObject(_move!(handle, jobId, pointer));
    } finally {
      malloc.free(pointer);
    }
  }

  Map<String, dynamic> clearFinished(int handle) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    return _decodeOwnedObject(_clearFinished!(handle));
  }

  Map<String, dynamic> pollEvent(int handle, int timeoutMs) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    return _decodeOwnedObject(_pollEvent!(handle, timeoutMs));
  }

  Future<Map<String, dynamic>> pollEventInBackground(int handle, int timeoutMs) =>
      Isolate.run(() => NativeEngine.instance.pollEvent(handle, timeoutMs));

  Map<String, dynamic> destroyCoordinator(int handle) {
    load();
    if (!isLoaded) throw StateError(loadError ?? 'Native engine is unavailable');
    return _decodeOwnedObject(_destroyCoordinator!(handle));
  }

}
