# Aot-backend

### Pre-requisites

* Install Python (3.8)
* Install Python's package manager (pip/pip3) 
```
apt-get install python-pip
```
```
apt-get install python3-pip
```
* Install virtualenv to be able to create isolated Python environments 

```
apt-get install virtualenv
```
* Install mysqlclient:
   * Install Python and MySQL development headers and libraries
   ```
   sudo apt-get install python3-dev default-libmysqlclient-dev build-essential
   ```
   * Install mysqlclient
   ```
   pip install mysqlclient
   ```

### Setting up the project
* Clone the repo 
```
git clone <url>
```
* Cd to the current project directory 
```
cd <reponame>
```
* Set up a virtual environment and activate it
```
virtualenv -p python3 venv
source venv/bin/activate
```
* Install dependencies
* Log into MySQL and create a database, grant all privileges to user
* Copy contents of .env.example to a file called .env in the same directory and change DB_USERNAME and DB_PASSWORD ( and others if required)
```
cp .env.example .env
```
* make and run migrations
```
python3 manage.py makemigrations
python3 manage.py migrate
```