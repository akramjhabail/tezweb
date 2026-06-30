`benchmarks.md` done! ✅ 58 lines.

Last file baaki hai — `deployment.md`:

```bash
code ~/tezweb/docs/deployment.md
```

Yeh content paste karo (Ctrl+A, Delete, phir paste):

```
# Deployment

Guide to deploying TezWeb to production.

## Option 1: Oracle Cloud Free Tier (Recommended)

Oracle Cloud's Always Free tier offers 4 CPU cores and 24GB RAM at no cost — ideal for TezWeb's per-core architecture.

### Steps

1. Sign up at oracle.com/cloud/free
2. Select Mumbai region for best latency from South Asia
3. Create a VM instance (Ampere A1 or AMD shape)
4. SSH into the instance

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Clone and Build

```bash
git clone https://github.com/akramjhabail/tezweb.git
cd tezweb
cargo build --release --example rest_api
```

### Run as a systemd Service

Create `/etc/systemd/system/tezweb.service`:

```
[Unit]
Description=TezWeb Server
After=network.target

[Service]
ExecStart=/home/ubuntu/tezweb/target/release/examples/rest_api
Restart=always
User=ubuntu

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable tezweb
sudo systemctl start tezweb
```

### Open Firewall Port

```bash
sudo iptables -I INPUT -p tcp --dport 8080 -j ACCEPT
```

Also open the port in the Oracle Cloud Security List for your VM's subnet.

## Option 2: DigitalOcean / VPS

Same steps as above apply to any Linux VPS (DigitalOcean, Hetzner, Vultr). Choose a region close to your users for lowest latency.

## Option 3: Cloudflare Tunnel (Quick Demo)

For temporary public access without a dedicated server:

```bash
brew install cloudflared
cloudflared tunnel --url http://localhost:8080
```

This gives a temporary public URL — not suitable for permanent production use.

## Why Not Cloudflare Workers?

Cloudflare Workers run on a serverless WASM model and do not support raw TCP socket servers. TezWeb requires a real Linux/macOS/Windows server with full TCP/IP access, so a VPS or cloud VM is required instead.

## Performance on Linux

TezWeb performs best on Linux due to `io_uring` support via the `monoio` runtime. Expect significantly higher throughput than the macOS benchmark numbers (40,248 RPS) shown in benchmarks.md.
```Save karke batao! 🚀