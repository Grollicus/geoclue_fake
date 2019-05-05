## Geoclue_fake

Dbus Service, meant as replacement for [GeoClue](https://www.freedesktop.org/wiki/Software/GeoClue/). It responds with a configurable fixed location instead of guessing the location by talking to Mozilla servers.

The location can be specified in `/etc/geoclue_fake.toml`, check out `config.default.toml` for a configuration example.
