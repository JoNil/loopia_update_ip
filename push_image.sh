#!/bin/bash
docker build -t loopia_update_ip .
docker image tag loopia_update_ip lordjonil/loopia_update_ip:latest
docker image push lordjonil/loopia_update_ip:latest
