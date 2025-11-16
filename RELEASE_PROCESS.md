# Shazamq Operator - Release Process

This document outlines the standardized release process for the Shazamq Operator project, following open-source best practices.

## Table of Contents

1. [Version Scheme](#version-scheme)
2. [Branch Strategy](#branch-strategy)
3. [Release Checklist](#release-checklist)
4. [Automated Release Process](#automated-release-process)
5. [Manual Release Process](#manual-release-process)
6. [Post-Release Tasks](#post-release-tasks)
7. [Hotfix Process](#hotfix-process)

---

## Version Scheme

We follow [Semantic Versioning 2.0.0](https://semver.org/):

- **Operator Version** (`Cargo.toml`, Docker image tag): `X.Y.Z`
  - **MAJOR** (X): Breaking changes, incompatible API changes
  - **MINOR** (Y): New features, backward-compatible
  - **PATCH** (Z): Bug fixes, backward-compatible

- **Helm Chart Version** (`charts/Chart.yaml`): Follows independent versioning
  - Initial release: `1.0.0`
  - Changes to chart structure: increment MAJOR
  - New features in chart: increment MINOR
  - Bug fixes in chart: increment PATCH

### Version Mapping

| Operator Version | Helm Chart Version | Docker Tag | Status |
|------------------|-------------------|------------|--------|
| 0.1.1            | 1.0.0             | 0.1.1      | ✅ Current |
| 0.2.0 (planned)  | 1.1.0 (planned)   | 0.2.0      | 🔮 Future |

---

## Branch Strategy

### Main Branches

- **`main`**: Production-ready code
  - Always stable and deployable
  - Protected branch (requires PR + reviews)
  - Tagged with releases

- **`develop`**: Integration branch for features
  - Next release preparation
  - Features merged here first
  - Regularly merged to `main` for releases

### Supporting Branches

- **Feature branches**: `feature/<name>`
  - Branch from: `develop`
  - Merge to: `develop`
  - Naming: `feature/add-webhook-support`

- **Release branches**: `release/v<version>`
  - Branch from: `develop`
  - Merge to: `main` and `develop`
  - Naming: `release/v0.1.1`

- **Hotfix branches**: `hotfix/v<version>`
  - Branch from: `main`
  - Merge to: `main` and `develop`
  - Naming: `hotfix/v0.1.2`

### Branch Protection Rules

**`main` branch**:
```yaml
- Require pull request reviews: 1+
- Require status checks to pass: true
- Require branches to be up to date: true
- Include administrators: false
- Restrict force pushes: true
- Restrict deletions: true
```

**`develop` branch**:
```yaml
- Require pull request reviews: 1
- Require status checks to pass: true
```

---

## Release Checklist

### Pre-Release

- [ ] All features for this release merged to `develop`
- [ ] All tests passing on `develop`
- [ ] Update `CHANGELOG.md` with all changes
- [ ] Version bumps in:
  - [ ] `Cargo.toml` (operator version)
  - [ ] `charts/Chart.yaml` (chart version + appVersion)
  - [ ] `charts/values.yaml` (image tag)
- [ ] Update documentation:
  - [ ] `README.md` (version references)
  - [ ] `charts/README.md` (usage examples)
- [ ] Test Docker build locally
- [ ] Test Helm chart installation locally

### Release

- [ ] Create release branch: `release/v<version>`
- [ ] Run full test suite
- [ ] Build and test Docker image
- [ ] Package and test Helm chart
- [ ] Create release commit
- [ ] Merge release branch to `main`
- [ ] Tag release: `v<version>`
- [ ] Push tag to trigger CI/CD

### Post-Release

- [ ] Verify GitHub release created
- [ ] Verify Docker images pushed:
  - [ ] Docker Hub: `shazamq/shazamq-operator:X.Y.Z`
  - [ ] GHCR: `ghcr.io/shazamq/shazamq-operator:X.Y.Z`
- [ ] Verify Helm chart published to GitHub Pages
- [ ] Verify ArtifactHub shows new version
- [ ] Test installation from published artifacts
- [ ] Announce release:
  - [ ] GitHub Discussions
  - [ ] Social media (if applicable)
  - [ ] Documentation site update
- [ ] Merge back to `develop`
- [ ] Close milestone (if using GitHub milestones)

---

## Automated Release Process

### Using the Release Script

```bash
# Automated release (recommended)
./scripts/release.sh 0.1.1
```

This script will:
1. ✅ Validate version format
2. ✅ Check for uncommitted changes
3. ✅ Update all version references
4. ✅ Run build and tests
5. ✅ Create release commit
6. ✅ Create and push git tag
7. ✅ Trigger GitHub Actions workflow

### GitHub Actions Workflow

The `.github/workflows/release.yml` workflow automatically:

1. **Builds binaries** for multiple platforms:
   - Linux (x86_64, ARM64)
   - macOS (x86_64, ARM64)

2. **Builds and pushes Docker images**:
   - Multi-arch: `linux/amd64`, `linux/arm64`
   - Tags: `X.Y.Z`, `X.Y`, `X`, `latest`
   - Registries: Docker Hub, GHCR

3. **Packages Helm chart**:
   - Generates chart package
   - Updates Helm repository index
   - Publishes to GitHub Pages

4. **Creates GitHub Release**:
   - Extracts changelog for this version
   - Attaches binary artifacts
   - Attaches Helm chart
   - Generates checksums

---

## Manual Release Process

If automation fails or manual intervention is needed:

### Step 1: Prepare Release Branch

```bash
# Create release branch from develop
git checkout develop
git pull origin develop
git checkout -b release/v0.1.1

# Update versions
# Edit: Cargo.toml, charts/Chart.yaml, charts/values.yaml

# Commit changes
git add -A
git commit -m "chore: prepare release v0.1.1"
git push origin release/v0.1.1
```

### Step 2: Build and Test

```bash
# Build operator
cargo build --release
cargo test --release

# Build Docker image
docker build -t shazamq/shazamq-operator:0.1.1 .

# Test Docker image
docker run --rm shazamq/shazamq-operator:0.1.1 --version

# Package Helm chart
helm package charts/ -d .deploy
helm lint .deploy/shazamq-operator-*.tgz
```

### Step 3: Merge to Main

```bash
# Create PR: release/v0.1.1 → main
# After review and approval:
git checkout main
git pull origin main
git merge --no-ff release/v0.1.1
git push origin main
```

### Step 4: Tag and Release

```bash
# Create annotated tag
git tag -a v0.1.1 -m "Release v0.1.1

See CHANGELOG.md for details."

# Push tag (triggers GitHub Actions)
git push origin v0.1.1
```

### Step 5: Manual Container Push (if needed)

```bash
# Login to registries
docker login quay.io
docker login ghcr.io

# Build multi-arch and push to Quay.io (primary)
docker buildx build --platform linux/amd64,linux/arm64 \
  -t quay.io/shazamq/shazamq-operator:0.1.1 \
  -t quay.io/shazamq/shazamq-operator:latest \
  --push .

# Push to GHCR (mirror)
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/shazamq/shazamq-operator:0.1.1 \
  -t ghcr.io/shazamq/shazamq-operator:latest \
  --push .
```

### Step 6: Manual Helm Publishing (if needed)

```bash
# Package chart
helm package charts/ -d .deploy

# Clone gh-pages branch
git clone --single-branch --branch gh-pages \
  https://github.com/shazamq/shazamq-operator.git gh-pages

# Copy chart
cp .deploy/*.tgz gh-pages/

# Update index
cd gh-pages
helm repo index . --url https://shazamq.github.io/shazamq-operator

# Commit and push
git add .
git commit -m "Release Helm chart v1.0.0 (app v0.1.1)"
git push origin gh-pages
```

### Step 7: Create GitHub Release

```bash
# Extract changelog section
sed -n "/## \[0.1.1\]/,/## \[/p" CHANGELOG.md | sed '$d' > release-notes.md

# Create release via GitHub CLI
gh release create v0.1.1 \
  --title "Release v0.1.1" \
  --notes-file release-notes.md \
  target/release/shazamq-operator \
  .deploy/*.tgz
```

---

## Post-Release Tasks

### 1. Merge Back to Develop

```bash
git checkout develop
git merge main
git push origin develop
```

### 2. Verify Published Artifacts

```bash
# Quay.io (primary)
docker pull quay.io/shazamq/shazamq-operator:0.1.1

# GHCR (mirror)
docker pull ghcr.io/shazamq/shazamq-operator:0.1.1

# Helm chart
helm repo add shazamq https://shazamq.github.io/shazamq-operator
helm repo update
helm search repo shazamq

# ArtifactHub (check after ~15 minutes)
open https://artifacthub.io/packages/helm/shazamq/shazamq-operator
```

### 3. Update Documentation

- [ ] Update main README.md with new version
- [ ] Update quickstart guides
- [ ] Publish blog post (if applicable)
- [ ] Update comparison tables

### 4. Announce Release

```markdown
**Template for announcement:**

🎉 Shazamq Operator v0.1.1 Released!

We're excited to announce the first release of the Shazamq Operator!

**Highlights:**
- ✅ Declarative cluster management via CRD
- ✅ MirrorMaker for Kafka migration
- ✅ Tiered storage (S3/GCS) support
- ✅ Production-ready security

**Install:**
```bash
helm repo add shazamq https://shazamq.github.io/shazamq-operator
helm install shazamq-operator shazamq/shazamq-operator
```

**Links:**
- Release Notes: https://github.com/shazamq/shazamq-operator/releases/tag/v0.1.1
- Documentation: https://github.com/shazamq/shazamq-operator/blob/main/README.md
- Helm Chart: https://artifacthub.io/packages/helm/shazamq/shazamq-operator

Full Changelog: https://github.com/shazamq/shazamq-operator/blob/main/CHANGELOG.md
```

---

## Hotfix Process

For critical bugs in production:

### 1. Create Hotfix Branch

```bash
git checkout main
git checkout -b hotfix/v0.1.2
```

### 2. Fix Bug

```bash
# Make fix
git add -A
git commit -m "fix: critical bug in reconciler"
```

### 3. Update Version

```bash
# Bump patch version in:
# - Cargo.toml: 0.1.1 → 0.1.2
# - charts/Chart.yaml: appVersion 0.1.2
# - charts/values.yaml: image tag 0.1.2
git add -A
git commit -m "chore: bump version to 0.1.2"
```

### 4. Release

```bash
# Merge to main
git checkout main
git merge --no-ff hotfix/v0.1.2
git tag -a v0.1.2 -m "Hotfix v0.1.2"
git push origin main
git push origin v0.1.2

# Merge back to develop
git checkout develop
git merge hotfix/v0.1.2
git push origin develop

# Delete hotfix branch
git branch -d hotfix/v0.1.2
```

---

## Release Cadence

- **Major releases**: Annually or when breaking changes accumulate
- **Minor releases**: Quarterly (new features)
- **Patch releases**: As needed (bug fixes)

### Planned Roadmap

| Version | Target Date | Key Features |
|---------|-------------|--------------|
| 0.1.1   | 2025-11-16  | ✅ Initial release |
| 0.2.0   | 2026-Q1     | Webhooks, auto-scaling |
| 0.3.0   | 2026-Q2     | Multi-cluster federation |
| 1.0.0   | 2026-Q3     | Stable API, LTS support |

---

## Rollback Process

If a release has critical issues:

### 1. Immediate Mitigation

```bash
# Revert to previous version via Helm
helm rollback shazamq-operator

# Or downgrade
helm upgrade shazamq-operator shazamq/shazamq-operator --version 0.9.0
```

### 2. Remove Broken Release

```bash
# Delete tag
git tag -d v0.1.1
git push origin :refs/tags/v0.1.1

# Delete GitHub release
gh release delete v0.1.1

# Remove from Helm repo
# (edit gh-pages branch)
```

### 3. Issue Hotfix

Follow [Hotfix Process](#hotfix-process) with correct version.

---

## Security Releases

For security vulnerabilities:

1. **Private disclosure**: Receive report via security@shazamq.io
2. **Fix in private fork**: Don't expose vulnerability
3. **Coordinate disclosure**: With reporter
4. **Release hotfix**: With CVE assignment
5. **Public announcement**: After fix deployed

---

## Checklist Summary

```markdown
## Release vX.Y.Z

**Pre-Release:**
- [ ] Code freeze on `develop`
- [ ] All tests passing
- [ ] CHANGELOG.md updated
- [ ] Versions bumped
- [ ] Docs updated
- [ ] Local testing complete

**Release:**
- [ ] Release branch created
- [ ] PR reviewed and merged
- [ ] Tag created and pushed
- [ ] CI/CD completed

**Post-Release:**
- [ ] Artifacts verified
- [ ] ArtifactHub updated
- [ ] Announcement published
- [ ] Merged back to develop

**Sign-off:** @maintainer-name
```

---

## Tools

- **Git**: Version control
- **GitHub Actions**: CI/CD automation
- **Helm**: Chart packaging
- **Docker**: Container builds
- **cargo**: Rust build system
- **gh CLI**: GitHub operations

---

## Contact

- **Maintainers**: @murtaza
- **Issues**: https://github.com/shazamq/shazamq-operator/issues
- **Discussions**: https://github.com/shazamq/shazamq-operator/discussions
- **Security**: security@shazamq.io

---

**Last Updated**: 2025-11-16  
**Document Version**: 1.0.0

