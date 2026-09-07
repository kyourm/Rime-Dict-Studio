#!/bin/sh
set -eu

app_path=${1:?Usage: verify-bundle-icon.sh /path/to/App.app}
plist_path="$app_path/Contents/Info.plist"
resources_path="$app_path/Contents/Resources"

icon_name=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$plist_path" 2>/dev/null || true)
test -n "$icon_name"
test -s "$resources_path/$icon_name"
file "$resources_path/$icon_name" | grep -q 'Mac OS X icon'
