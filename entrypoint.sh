#!/bin/bash
set +e

DB_PATH=${DB_PATH:-"/data/et.db"}
SECRETS_FILE=${SECRETS_FILE:-"/tmp/network_secrets.json"}
IP_WHITELIST_FILE=${IP_WHITELIST_FILE:-"/tmp/ip_whitelist.json"}
IP_HOSTNAME_AUTO_FILE=${IP_HOSTNAME_AUTO_FILE:-"/tmp/ip_hostname_auto.json"}
CORE_CONFIG=${CORE_CONFIG:-"/etc/easytier/core.toml"}

echo "Starting easytier-core..."
if [ -f "$CORE_CONFIG" ]; then
    ADMIN_API_URL="http://127.0.0.1:11211" IP_WHITELIST_FILE="$IP_WHITELIST_FILE" IP_HOSTNAME_AUTO_FILE="$IP_HOSTNAME_AUTO_FILE" EXTRA_SECRETS_FILE="$SECRETS_FILE" easytier-core -c "$CORE_CONFIG" &
    CORE_PID=$!
else
    echo "Warning: No core config found at $CORE_CONFIG. Core not started."
fi

echo "Starting easytier-admin..."
easytier-admin --db "$DB_PATH" --web-server-port 11211 -p tcp &
ADMIN_PID=$!

echo "Starting secret sync..."
while true; do
    sqlite3 "$DB_PATH" \
        "SELECT json_group_array(json_object('secret', secret))
         FROM network_secrets
         WHERE is_active = 1 
           AND (expires_at IS NULL OR expires_at > datetime('now'))" \
        2>/dev/null > "$SECRETS_FILE" || true

    # Process auto-hostname bindings from core
    if [ -f "$IP_HOSTNAME_AUTO_FILE" ]; then
        AUTO_BIND_TRACKING_FILE="/tmp/auto_bound_pairs.json"
        cat "$IP_HOSTNAME_AUTO_FILE" | python3 -c "
import json, sys, os

tracking_file = '$AUTO_BIND_TRACKING_FILE'
tracking = {}
if os.path.exists(tracking_file):
    try:
        with open(tracking_file) as f:
            tracking = json.load(f)
    except:
        pass

bindings = json.load(sys.stdin)
changed = False
for b in bindings:
    ip = b.get('ip', '')
    hostname = b.get('hostname', '')
    if ip and hostname:
        key = f'{ip}:{hostname}'
        if key not in tracking:
            print(f\"UPDATE ip_whitelist SET hostname='{hostname}' WHERE ip='{ip}' AND hostname IS NULL\")
            tracking[key] = True
            changed = True

if changed:
    with open(tracking_file, 'w') as f:
        json.dump(tracking, f)
" 2>/dev/null | while read sql; do
            sqlite3 "$DB_PATH" "$sql" 2>/dev/null || true
        done
        rm -f "$IP_HOSTNAME_AUTO_FILE"
    fi

    sqlite3 "$DB_PATH" \
        "SELECT json_group_array(json_object('ip', ip, 'hostname', hostname)) FROM ip_whitelist" \
        2>/dev/null > "$IP_WHITELIST_FILE" || true
    sleep 30
done &
SYNC_PID=$!

wait 2>/dev/null || true
echo "Shutting down..."
kill $CORE_PID $ADMIN_PID $SYNC_PID 2>/dev/null || true
wait 2>/dev/null || true
