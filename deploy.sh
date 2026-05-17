#!/usr/bin/env bash

PASSWD="rov\r"

cargo build --target aarch64-unknown-linux-gnu --release

if [ $? -eq 0 ]; then
  rsync -av ./target/aarch64-unknown-linux-gnu/release/rov-pi root@rov-pi:/home/rov/bin/rov-pi
  rsync -av ./rov-pi.service root@rov-pi:/etc/systemd/system/rov-pi.service
  ssh root@rov-pi "systemctl daemon-reload && systemctl enable rov-pi && systemctl restart rov-pi"
fi
