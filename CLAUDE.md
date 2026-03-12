# Discord Bot

## Project Setup

- Rust project using serenity 0.12 for the Discord API
- Built as a container image via `Containerfile`
- CI/CD via GitHub Actions in `.github/workflows/`

## Release Process

1. **Make changes** in `src/main.rs` (or other files)
2. **Verify locally**: `cargo fmt --all -- --check && cargo check`
3. **Commit and push** to `master`
4. **Wait for CI** (`ci.yaml`): runs tests, clippy, format check, build, and cargo-deny
5. **Wait for CD** (`build.yaml`): builds the container image and pushes to `ghcr.io/twodcube-home/discord-bot` with tags `latest` and the commit SHA
6. **Get the image tag from the CD logs** — do NOT use `git rev-parse HEAD` locally, as it can differ from the SHA GitHub Actions checks out. Check the "Push image" step for the actual tag.
7. **Update ArgoCD**: edit `discord-bot/discord-bot/deployment.yaml` in the `TwoDCube-Home/argocd` repo — set the image tag to the SHA from the CD logs
8. **Push ArgoCD repo** — ArgoCD picks up the change and rolls out the new deployment

## Key Paths

- ArgoCD repo: `TwoDCube-Home/argocd` on GitHub
- Deployment manifest: `discord-bot/discord-bot/deployment.yaml` in the ArgoCD repo
- Image: `ghcr.io/twodcube-home/discord-bot:<commit-sha>`
