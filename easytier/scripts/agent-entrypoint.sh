#!/bin/bash
set -e

: "${WHITELIST_SYNC_URL:?WHITELIST_SYNC_URL must be set}"
: "${IP_WHITELIST_FILE:=/data/ip_whitelist.json}"
: "${WHITELIST_SYNC_INTERVAL:=30}"
: "${CORE_CONFIG:=/etc/easytier/core.toml}"
: "${RUST_LOG:=easytier=warn}"
export RUST_LOG
export IP_WHITELIST_FILE
export WHITELIST_SYNC_INTERVAL

# 后续 sync-daemon 会用 admin 真实白名单覆盖此默认值。
DEFAULT_WHITELIST_IPS="${WHITELIST_DEFAULT_IPS:-}"

if [ -z "$DEFAULT_WHITELIST_IPS" ] && [ -f "$CORE_CONFIG" ]; then
    AGENT_IP=$(grep -E '^[[:space:]]*ipv4[[:space:]]*=' "$CORE_CONFIG" | \
        sed -E 's/^[[:space:]]*ipv4[[:space:]]*=[[:space:]]*"([^"]+)".*/\1/' | \
        cut -d'/' -f1)
    if [ -n "$AGENT_IP" ]; then
        DEFAULT_ADMIN_IP=$(echo "$AGENT_IP" | awk -F. '{print $1"."$2"."$3".1"}')
        DEFAULT_WHITELIST_IPS="$DEFAULT_ADMIN_IP"
        echo "[agent] auto-detected admin IP from core.toml: $DEFAULT_ADMIN_IP (agent=$AGENT_IP)"
    fi
fi

if [ ! -f "$IP_WHITELIST_FILE" ] && [ -n "$DEFAULT_WHITELIST_IPS" ]; then
    echo "[agent] initializing default whitelist (kills chicken-and-egg)"
    mkdir -p "$(dirname "$IP_WHITELIST_FILE")"
    DEFAULT_JSON=$(echo "$DEFAULT_WHITELIST_IPS" | awk '
    BEGIN { printf "[" }
    {
        n = split($0, arr, ",")
        for (i = 1; i <= n; i++) {
            gsub(/^[ \t]+|[ \t]+$/, "", arr[i])
            if (arr[i] != "") {
                if (i > 1) printf ","
                printf "{\"ip\":\"%s\",\"hostname\":null}", arr[i]
            }
        }
    }
    END { printf "]\n" }
    ')
    printf '%s' "$DEFAULT_JSON" > "$IP_WHITELIST_FILE"
    echo "[agent] default whitelist: $DEFAULT_JSON"
elif [ -f "$IP_WHITELIST_FILE" ]; then
    echo "[agent] whitelist file exists, skipping default init"
fi

echo "[agent] starting whitelist-sync-daemon..."
easytier-whitelist-sync &
SYNC_PID=$!

sleep 5

echo "[agent] starting easytier-core..."
if [ -f "$CORE_CONFIG" ]; then
    IP_WHITELIST_FILE="$IP_WHITELIST_FILE" \
    easytier-core -c "$CORE_CONFIG" &
    CORE_PID=$!
else
    echo "[agent] ERROR: core config not found at $CORE_CONFIG"
    kill $SYNC_PID
    exit 1
fi

trap "echo '[agent] shutting down'; kill $SYNC_PID $CORE_PID 2>/dev/null; wait" SIGTERM SIGINT
wait
