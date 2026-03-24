#!/bin/bash

echo "┃ ┃┏━┃┏━┛┃ ┃   ━┏┛┃ ┃┏━┛   ┏━┛┏━┃┏━┃┃ ┃┏━┛"
echo "┏━┃┏━┃┃  ┏┛     ┃ ┏━┃┏━┛   ┃  ┏┏┛┃ ┃┃ ┃━━┃"
echo "┛ ┛┛ ┛━━┛┛ ┛━━┛ ┛ ┛ ┛━━┛━━┛━━┛┛ ┛━━┛━━┛━━┛"

RELEASE_URL="https://github.com/Courtcircuits/hackthecr-us/releases/download/release-660086df61369b73fd19160b3b17c16580d4c791/cli"
TEMP_DIR=$(mktemp -d)
pushd $TEMP_DIR > /dev/null

spinner() {
  local frames='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
  local pid=$1
  while kill -0 $pid 2>/dev/null; do
    for i in $(seq 0 9); do
      printf "\r Downloading... ${frames:$i:1}"
      sleep 0.1
    done
  done
  printf "\r                \r"
}

wget --quiet $RELEASE_URL &
spinner $!
wait $!

mv cli $HOME/.local/bin/htcctl
chmod +x $HOME/.local/bin/htcctl

popd > /dev/null

mkdir -p $HOME/.config/systemd/user
cat > $HOME/.config/systemd/user/htcctl-schedule.service << 'EOF'
[Unit]
Description=htcctl schedule
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=%h/.local/bin/htcctl schedule

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload > /dev/null
echo "htcctl installed. To enable the schedule service, run:"
echo "  systemctl --user enable htcctl-schedule.service"
