#!/bin/bash
# create_dmg.sh — Package mactemp.app into a distributable .dmg file
#
# Usage: ./create_dmg.sh
#
# Creates mactemp.dmg containing the .app bundle.
# Users can download, open the DMG, and drag to /Applications.

set -euo pipefail

APP_NAME="mactemp"
DMG_NAME="${APP_NAME}.dmg"
VOLUME_NAME="${APP_NAME}"
STAGING_DIR="/tmp/${APP_NAME}_dmg_staging"

# Step 1: Build the .app bundle
echo "==> Building .app bundle..."
./create_app_bundle.sh

# Step 2: Prepare staging directory for DMG contents
echo ""
echo "==> Preparing DMG contents..."
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

# Copy .app bundle
cp -r "${APP_NAME}.app" "${STAGING_DIR}/"

# Create a symlink to /Applications for drag-and-drop install
ln -s /Applications "${STAGING_DIR}/Applications"

# Step 3: Create DMG
echo "==> Creating ${DMG_NAME}..."
rm -f "${DMG_NAME}"

hdiutil create \
    -volname "${VOLUME_NAME}" \
    -srcfolder "${STAGING_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_NAME}"

# Cleanup
rm -rf "${STAGING_DIR}"

# Show result
DMG_SIZE=$(du -sh "${DMG_NAME}" | cut -f1)
echo ""
echo "✅ ${DMG_NAME} created successfully!"
echo "   Size: ${DMG_SIZE}"
echo ""
echo "To install:"
echo "  1. Open ${DMG_NAME}"
echo "  2. Drag mactemp.app to the Applications folder"
echo "  3. Open from /Applications/mactemp.app"
