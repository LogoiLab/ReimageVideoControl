# ReimageVideo Controller Application

This is the controller application for the [ReimageVideo](https://github.com/LogoiLab/ReimageVideo) project.

This controller application is meant to be run as a daemon. It expects the `rivctrl_conf.toml` configuration file to be in its working directory. An example configuration file `rivctrl_conf.toml.example` is provided.

The controller application expects a serial interface to talk to the ReimageVideo firmware on the RDU1502, this must be specified in the configuration file.

This application *must* be run with sudo or as a root service to have the ability to read certain system information and use the serial interface properly.

To use the provided `reimagevideo.service` file with systemd, first copy `rivctrl` to `/bin` and `rivctrl_conf.toml` to `/etc`.

*Most* systemd service files are placed in `/etc/systemd/system`, but can be placed wherever you prefer to keep your service files.
