#!/usr/bin/env bash
set -e

IMAGE_NAME="easytier-admin"
IMAGE_TAG="2.4.5"
CONTAINER_NAME="easytier-admin"
ET_ADMIN_PASSWORD="changeme-please"
WEB_PORT="11211"
VPN_PORT="22020"
NO_CACHE=""

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Build and run the easytier-admin Docker image (Linux amd64).

Options:
  --image IMAGE       Docker image name        (default: ${IMAGE_NAME})
  --tag TAG           Docker image tag          (default: ${IMAGE_TAG})
  --container NAME    Container name            (default: ${CONTAINER_NAME})
  --password PASS     Admin panel password      (default: ${ET_ADMIN_PASSWORD})
  --web-port PORT     Admin UI HTTP port        (default: ${WEB_PORT})
  --vpn-port PORT     EasyTier core VPN port    (default: ${VPN_PORT})
  --no-cache          Pass --no-cache to docker build
  -h, --help          Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --image)
      IMAGE_NAME="$2"; shift 2 ;;
    --tag)
      IMAGE_TAG="$2"; shift 2 ;;
    --container)
      CONTAINER_NAME="$2"; shift 2 ;;
    --password)
      ET_ADMIN_PASSWORD="$2"; shift 2 ;;
    --web-port)
      WEB_PORT="$2"; shift 2 ;;
    --vpn-port)
      VPN_PORT="$2"; shift 2 ;;
    --no-cache)
      NO_CACHE="--no-cache"; shift ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

if ! command -v docker &>/dev/null; then
  echo "Error: docker is not installed or not in PATH." >&2
  exit 1
fi

ET_ADMIN_SECRET=$(head -c 32 /dev/urandom | base64 | tr -dc 'A-Za-z0-9' | head -c 48)

echo "=========================================="
echo " EasyTier Admin — Build & Run"
echo "=========================================="
echo " Image:       ${IMAGE_NAME}:${IMAGE_TAG}"
echo " Container:   ${CONTAINER_NAME}"
echo " Admin port:  ${WEB_PORT}"
echo " VPN port:    ${VPN_PORT}"
echo " No-cache:    ${NO_CACHE:-no}"
echo "=========================================="
echo ""

if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo "WARNING: Container '${CONTAINER_NAME}' already exists. Removing..."
  docker rm -f "${CONTAINER_NAME}"
fi

echo "[1/3] Building image..."
docker build --platform linux/amd64 ${NO_CACHE} -t "${IMAGE_NAME}:${IMAGE_TAG}" .

echo ""
echo "[2/3] Starting container..."
docker run -d \
  --restart=always \
  --privileged \
  --name "${CONTAINER_NAME}" \
  --network host \
  -v "$(pwd)/core.toml:/etc/easytier/core.toml" \
  -v "$(pwd)/data:/data" \
  -e "ET_ADMIN_PASSWORD=${ET_ADMIN_PASSWORD}" \
  -e "ET_ADMIN_SECRET=${ET_ADMIN_SECRET}" \
  "${IMAGE_NAME}:${IMAGE_TAG}"

echo ""
echo "[3/3] Waiting for container to start..."
sleep 2

if docker ps --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo "✓ Container is running."
else
  echo "ERROR: Container is not running. Check logs:" >&2
  docker logs "${CONTAINER_NAME}" >&2 || true
  exit 1
fi

echo ""
echo "=========================================="
echo " Deployment complete!"
echo "=========================================="
echo " Admin UI:  http://localhost:${WEB_PORT}/admin"
echo " Container: ${CONTAINER_NAME}"
echo " Username:  admin"
echo " Password:  (as set by --password)"
echo "=========================================="
