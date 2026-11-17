# Practical Test Plan — VRipper Native

This checklist is for the first human acceptance pass after the Windows/Android smoke-build artifacts are available. Use only accounts, links and content you are authorized to access. Do not share passwords, session cookies, or unredacted authentication logs.

## 1. Installation and native runtime

- [ ] Install/run the Windows x86-64 package on a 64-bit Windows machine.
- [ ] Confirm the app launches without a separately installed Java/JVM runtime.
- [ ] Confirm the Rust engine is detected in the status bar.
- [ ] Install the Android ARM64 APK on a compatible 64-bit device.
- [ ] Confirm the app launches and the native engine loads.
- [ ] Confirm downloads are written to an accessible app/user download location.

## 2. Account and network

- [ ] Enter Viper account credentials in Settings → Viper and save.
- [ ] Restart the app and confirm the username remains populated; the password must remain in OS secure storage.
- [ ] With an authorized test account, verify authentication and a harmless lookup.
- [ ] Test Network → Disabled.
- [ ] Test Network → Manual with a proxy endpoint you control.
- [ ] Test invalid host/port and incomplete credentials; the UI must reject them.
- [ ] Test System mode only on a device with a configured system proxy; it must fail closed when no proxy is discoverable.
- [ ] Confirm no password, cookie or authorization token appears in logs.

## 3. Link discovery and queue

- [ ] Submit one authorized thread link.
- [ ] Submit one authorized post link.
- [ ] Submit multiple links including a duplicate.
- [ ] Submit malformed text and confirm it is reported as invalid without crashing.
- [ ] Confirm the lookup summary and queue counts match the discovered images.
- [ ] Stop one pending/running task.
- [ ] Move pending tasks up/down and confirm queue order changes.
- [ ] Stop all tasks and clear finished tasks.

## 4. Download and output compatibility

- [ ] Download a small authorized image from each resolver under test.
- [ ] Confirm the temporary file is finalized and no temporary file remains after success.
- [ ] Confirm the saved file opens and its extension matches its actual image MIME type.
- [ ] Compare folder naming, post ID suffix, filename normalization and ordering with the reference application.
- [ ] Confirm an existing file is not silently overwritten or corrupted.
- [ ] Interrupt a download and verify the queue reports a clear stopped/error state.
- [ ] Retry a transient failure and verify the final output is not duplicated.

## 5. Evidence to report

For each issue, record:
- Platform and OS version.
- App version and native engine version.
- The action immediately before the issue.
- Expected behavior versus actual behavior.
- Redacted log excerpt and screenshot if relevant.

Never attach passwords, cookies, authorization headers, personal account identifiers or private content.
