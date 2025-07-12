[![codecov](https://codecov.io/gh/Heidelberger-Regattaverband/regatta-infopoint/graph/badge.svg?token=NBQESMP9HW)](https://codecov.io/gh/Heidelberger-Regattaverband/regatta-infopoint)

# Setup of a new regatta server

## General steps
Become root if you haven't already:
```bash
sudo -i
```

Install required packages:
```bash
apt-get update && apt-get upgrade
apt-get install certbot git docker.io docker-compose-v2 htop deborphan sshpass
```

Request letsencrypt certificates for all hostnames:
```bash
certbot certonly
```

Change the permissions of all private keys to make them accessible from docker containers:
```bash
chmod 640 /etc/letsencrypt/live/<full-qualified-hostname>/privkey.pem
```

Clone Github repo with docker configurations:
```bash
mkdir git && cd git
git clone https://github.com/Heidelberger-Regattaverband/docker.git
```

Set hostname:
```bash
nano /etc/hostname
```

## [Setting up Tailscale](https://tailscale.com/kb/1476/install-ubuntu-2404)
Add Tailscale's package signing key and repository:
```bash
curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/noble.noarmor.gpg | sudo tee /usr/share/keyrings/tailscale-archive-keyring.gpg >/dev/null
curl -fsSL https://pkgs.tailscale.com/stable/ubuntu/noble.tailscale-keyring.list | sudo tee /etc/apt/sources.list.d/tailscale.list
```
Install Tailscale:
```bash
apt-get update && apt-get install tailscale
```
Connect your machine to your Tailscale network and authenticate in your browser:
```bash
tailscale up
```

## Setup MS-SQL Server
Prepare MS-SQL server directories:
```bash
mkdir /mssql && mkdir /mssql/backup && mkdir /mssql/restore
chown -R 10001 /mssql && chmod -R 775 /mssql
```

Configure the docker container settings and start MS-SQL Server:
```bash
cd docker/mssql
nano .env
docker compose up -d && docker logs mssql-aquarius -f
```

Copy database backup files into restore directory:
```bash
scp <local_file> root@<full-qualified-hostname>:/mssql/restore
```
Import the database backup files from the restore directory with SQL Server Management Studio.

## Setup Regatta Infoportal
Configure the docker container settings and start Infoportal:
```bash
cd docker/infoportal
nano .env
docker compose up -d && docker logs infoportal -f
```

## Setup Watchtower
[Watchtower](https://hub.docker.com/r/containrrr/watchtower) is a tool to automatically update docker containers, if a new version of a docker image is available.
Start Watchtower:
```bash
cd docker/watchtower
docker compose up -d && docker logs watchtower -f
```

## Add MS-SQL User
Add a new mssql user:
```bash
adduser mssql -u 10001
```

## Enable password-less ssh login
Add public authorization key:
```bash
nano ~/.ssh/authorized_keys
```

Alternatively create a new private SSH key:
```bash
ssh-keygen -b 4096
```

Copy the public key to any remote host to enable login without password:
```bash
ssh-copy-id -i .ssh/id_rsa.pub root@<full-qualified-hostname>
```
