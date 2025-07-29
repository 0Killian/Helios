#!/bin/bash

# Download agent binary
curl -sSL {agent_binary_base_url}-linux-$(uname -m) > /srv/helios-agent
if [ $? -eq 0 ]; then
    chmod +x /srv/helios-agent
else
    echo "Agent binary not found. Either the server hosting the agent is down, or your architecture is not supported."
    exit 1
fi

# Create configuration
mkdir -p /etc/helios-agent
cat <<EOF > /etc/helios-agent/config.toml
[base]
token = "{token}"
helios_base_url = "{helios_base_url}"

{custom_config}
EOF

# Create user and set ownership
useradd -r -s /bin/false helios-agent
chown helios-agent:helios-agent -R /etc/helios-agent
chown helios-agent:helios-agent /srv/helios-agent

# Create systemd service
cat <<EOF > /etc/systemd/system/helios-agent.service
[Unit]
Description=Helios Agent
After=network.target

[Service]
User=helios-agent
Group=helios-agent
ExecStart=/srv/helios-agent
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
systemctl daemon-reload
systemctl enable --now helios-agent.service

echo "Helios Agent installed successfully."
