# Highlight bot for Stoat

Lets users create trigger words and be alerted when those triggers are said.

| Crate       | Description                               |
|-------------|-------------------------------------------|
| `highlight` | Bot implementation.                       |
| `stoat`     | Stoat API wrapper.                        |

See info for `stoat` [here](https://github.com/Zomatree/Highlight/tree/master/crates/stoat).

## Running

Docker images are automatically built with every release and should be used to selfhost Highlight.

An example [Docker Compose](https://docs.docker.com/compose/) config can be found [here](https://github.com/Zomatree/Highlight/blob/master/docker-compose.yml) for easier selfhosting.

Mount the Highlight bot config file at `/Highlight.toml`.