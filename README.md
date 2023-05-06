# regatta-infopoint

## Installation on a new server

Become root user:
```bash
sudo -i
```

Install required packages:
```bash
apt update && apt upgrade
apt install certbot git docker.io docker-compose
```

Request an letsencrypt certificate:
```bash
certbot certonly
```

Maintain an ssh key to access github.com:
```bash
nano .ssh/id_rsa
nano .ssh/id_rsa.pub
chmod 600 .ssh/id_rsa
```

Clone the github repo containing the infoportal:
```bash
git clone git@github.com:Heidelberger-Regattaverband/regatta-infopoint.git
```

Change permission of private key to make it accessible by the infoportal docker container:
```bash
chmod 640 /etc/letsencrypt/archive/<host>.online-server.cloud/privkey<id>.pem
```

Start the infoportal docker container:
```bash
cd regatta-infopoint
./update.sh
```