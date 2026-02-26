#!/usr/bin/env bash
# Update vendored blocklists for the Seglamater Privacy Scanner.
# Run periodically (e.g., weekly via cron) or before building a release.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Updating blocklists..."

# EasyPrivacy list (tracker/analytics domains)
echo "  Downloading EasyPrivacy..."
curl -sSfL -o easyprivacy.txt "https://easylist.to/easylist/easyprivacy.txt"

# Peter Lowe's ad server list
echo "  Downloading Peter Lowe's ad server list..."
curl -sSfL -o pgl-adservers.txt "https://pgl.yoyo.org/adservers/serverlist.php?hostformat=nohtml&showintro=0&mimetype=plaintext"

echo "Done. Blocklists updated at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
