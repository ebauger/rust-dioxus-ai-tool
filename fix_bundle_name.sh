#!/bin/bash
echo "Fixing bundle name in Info.plist..."

# Define the path to Info.plist (adjust if target dir changes)
PLIST_PATH="target/dx/context-loader/debug/macos/ContextLoader.app/Contents/Info.plist"

# Check if Info.plist exists
if [ ! -f "$PLIST_PATH" ]; then
  echo "Error: $PLIST_PATH not found. Run 'dx build' first."
  exit 1
fi

# Use sed to replace the bundle name
/usr/bin/sed -i "" "s|<string>ContextLoader</string>|<string>Context Loader</string>|g" "$PLIST_PATH"

# Check sed exit status
if [ $? -eq 0 ]; then
  echo "Successfully updated bundle name in $PLIST_PATH."
else
  echo "Error: Failed to update bundle name in $PLIST_PATH using sed."
  exit 1
fi

echo "Done." 