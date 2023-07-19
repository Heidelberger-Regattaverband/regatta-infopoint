# Setup of a new regatta server

## General steps
Become root if you haven't already:
```bash
sudo -i
```

Install required packages:
```bash
apt update && apt upgrade
apt install certbot git docker.io docker-compose htop deborphan
```

Request a letsencrypt certificates for all hostnames:
```bash
certbot certonly
```

Change the permission of the private key to make it accessible from docker containers:
```bash
chmod 640 /etc/letsencrypt/live/<full-qualified-hostname>/privkey.pem
```

Clone Github repo with docker configurations:
```bash
mkdir git && cd git
git clone https://github.com/Heidelberger-Regattaverband/docker.git
```

## Setup MS-SQL Server

Prepare MS-SQL server directories:
```bash
mkdir /mssql && mkdir /mssql/backup && mkdir /mssql/restore
chown -R 10001 /mssql
chmod -R 775 /mssql
```

Configure the docker container settings and start MS-SQL Server:
```
cd docker/mssql
nano .env
docker-compose up -d && docker logs mssql-aquarius -f
```

Maintain an ssh key for write access to github.com:
```bash
nano .ssh/id_rsa
nano .ssh/id_rsa.pub
chmod 600 .ssh/id_rsa
```

Clone the github repo containing the infoportal:
```bash
git clone git@github.com:Heidelberger-Regattaverband/regatta-infopoint.git
```

Start the infoportal docker container:
```bash
cd regatta-infopoint
DB_PASSWORD=<db_password> ./update.sh
```

## Additional MS-SQL setup

Add a new mssql user:
```bash
adduser mssql -u 10001
```

Add backup and restore folder for MS-SQL:
```bash
su - mssql
mkdir /mssql/backup
mkdir /mssql/restore
```
