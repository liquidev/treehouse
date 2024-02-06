# `admin`

Bunch of scripts I use to administrate the treehouse at <https://liquidex.house>.
The full setup is:

- A systemd service runs `daemon.bash` in a separate user.
  - This script builds and runs the server.
  - It also listens for `reload` commands being sent through a FIFO, which can be used to make the server rebuild and rerun.
- The `reload` command is sent by the `deploy.bash` script which runs on my own machine rather than the server.
  - This script causes a `git pull` and a `reload` command to be run.
