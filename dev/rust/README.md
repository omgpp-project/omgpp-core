## Docker

This container just made for rust environment. You can use it to build the project. If the container is a bit fat for you can change the base image

To start the container:
`docker compose --env-file .env up -d`

The `.env` file should contains atleast 2 variables:
- DEV_PROJECT_ROOT=`path to omgpp`
- DEV_PROJECT_NAME=omgpp

To develop you can use VS code to connect to container