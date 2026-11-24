# Publishing service images

The native release workflow wraps the architecture-specific release payloads in Ubuntu 24.04 runtime images. The image jobs consume the Linux artifacts from that workflow. The release tag, GitHub asset digests, checksums and native source identity bind the inputs. Each image contains the same executable bytes as the native release.

The catalog lists the binaries that supply long-running or scheduled services and accounts for other native release binaries. Add a catalog entry when a release gains a service. The packaging job renders a Dockerfile and an isolated context for each entry. Runtime credentials and configuration belong to the installation.

## Installing a service

Start with the pack installer’s provider selection. Choose Docker for each service you want it to manage and reuse an existing compatible service where appropriate. The published image inventory belongs to the selected release. The [installation guide](../../docs/guide/19-installation.md) explains local discovery and the native, Docker and external choices.

## Packaging an existing release

Select the native release workflow on the main branch. Set `images_only` to true and `tag` to the existing version, including its leading `v`. This path downloads the two released Linux payload sets. It skips every native build. The platform jobs push architecture-specific manifests. The final job assembles each service's versioned multi-platform tag and attaches `service-images.json` to the GitHub release. A retry reuses matching published architecture manifests and checks the final index before accepting a previously published inventory.

From the repository checkout, dispatch an image backfill with the version in `TAG`.

```sh
gh workflow run binary-release.yml --ref main -f images_only=true -f tag="$TAG"
```

The workflow token needs package write permission and the final job needs release write permission. On first publication, set each GHCR package's visibility to public if anonymous installation is required. GHCR manages package visibility independently of repository visibility.

## Packaging locally

Run from the repository root with a tag fetched from origin, Python 3.9 or newer, GitHub CLI, and Docker Buildx. Supply an empty output directory per architecture. An existing cached download is checked against GitHub before use.

```sh
python3 docker/service-images/package.py prepare --tag "$TAG" --arch arm64 --assets "$DOWNLOADS" --out "$ARM_CONTEXTS"
python3 docker/service-images/package.py build --out "$ARM_CONTEXTS"
```

The build loads images tagged `ghcr.io/<owner>/<repository>-<service>:<tag>-arm64` into the local Docker engine. Use `amd64` and a separate directory on Intel. To publish, authenticate Docker to GHCR, repeat `build` with `--push` for both architectures, put the resulting `platform-amd64.json` and `platform-arm64.json` in one directory, and run the following.

```sh
python3 docker/service-images/package.py publish --tag "$TAG" --records "$RECORDS"
```

## Looking up a release digest

Read the service's OCI index digest and the selected Linux architecture's binary SHA-256 from the release inventory. Set `SERVICE` to a catalog ID such as `liter-llm` and `ARCH` to `arm64` or `amd64`.

```sh
gh release download "$TAG" --pattern service-images.json --dir "$RECORDS"
jq --arg id "$SERVICE" --arg platform "linux/$ARCH" \
  '.images[$id] | {pull: (.repository + "@" + .digest), executable: .platforms[$platform].binarySha256}' \
  "$RECORDS/service-images.json"
```
