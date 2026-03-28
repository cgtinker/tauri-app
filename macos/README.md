# Debugging Notes

Regular dev build:
`cargo tauri dev`

Native app bundle:
`cargo tauri build --debug`

Consider to resign the application with debug entitlements for debugging purposes.
Resign application:
`codesign -s - --entitlements entitlements.debug.plist --force target/debug/tauri-app`

## Leaks
`leaks -quiet -atExit -- ./target/debug/tauri-app`