#!/bin/bash
set -euo pipefail

# Shazamq Operator Release Script
# This script automates the release process

VERSION=${1:-}
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

# Validate version format
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in format X.Y.Z (e.g., 0.1.0)"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "================================================"
echo "  Shazamq Operator Release Process"
echo "  Version: $VERSION"
echo "================================================"
echo ""

cd "$ROOT_DIR"

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "❌ Error: You have uncommitted changes. Please commit or stash them first."
    exit 1
fi

echo "✅ No uncommitted changes"

# Check current branch
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
echo "📍 Current branch: $CURRENT_BRANCH"

# Update Cargo.toml version
echo "📝 Updating Cargo.toml version to $VERSION..."
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Chart.yaml appVersion
echo "📝 Updating Chart.yaml appVersion to $VERSION..."
sed -i.bak "s/^appVersion: \".*\"/appVersion: \"$VERSION\"/" charts/Chart.yaml
rm charts/Chart.yaml.bak

# Update values.yaml image tag
echo "📝 Updating values.yaml image tag to $VERSION..."
sed -i.bak "s/^  tag: \".*\"/  tag: \"$VERSION\"/" charts/values.yaml
rm charts/values.yaml.bak

# Build and test
echo "🔨 Building release binary..."
cargo build --release

echo "🧪 Running tests..."
cargo test --release

echo "✅ Build and tests passed"

# Create release commit
echo "📦 Creating release commit..."
git add Cargo.toml charts/Chart.yaml charts/values.yaml
git commit -m "chore: release v$VERSION

- Update operator version to $VERSION
- Update Helm chart appVersion to $VERSION
- Update Docker image tag to $VERSION"

# Create and push tag
echo "🏷️  Creating git tag v$VERSION..."
git tag -a "v$VERSION" -m "Release v$VERSION

See CHANGELOG.md for details."

echo "⬆️  Pushing changes and tag..."
git push origin "$CURRENT_BRANCH"
git push origin "v$VERSION"

echo ""
echo "================================================"
echo "  ✅ Release process completed!"
echo "================================================"
echo ""
echo "Next steps:"
echo "1. GitHub Actions will build binaries and Docker images"
echo "2. Helm chart will be published to GitHub Pages"
echo "3. Check release at: https://github.com/shazamq/shazamq-operator/releases/tag/v$VERSION"
echo "4. Docker image: docker pull shazamq/shazamq-operator:$VERSION"
echo "5. Helm chart: helm install shazamq-operator shazamq/shazamq-operator --version 1.0.0"
echo ""
echo "🎉 Release v$VERSION is on its way!"

