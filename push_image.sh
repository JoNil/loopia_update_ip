#!/bin/bash
docker build -t lordjonil/loopia_update_ip:latest .
docker image push lordjonil/loopia_update_ip:latest
