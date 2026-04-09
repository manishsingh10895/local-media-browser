# Local Gallery

Local Gallery is a small LAN-friendly media browser:

- Rust `axum` backend for scanning and serving image/video files
- SvelteKit frontend for browsing the gallery in the browser
- optional Docker Compose setup to run both services together

It is meant for local-network use, so you can run it on one machine and open it from another device on the same Wi-Fi or LAN.

## What it does

- scans a media folder on disk
- serves images and videos over HTTP
- shows the media in a responsive gallery UI
- opens media in a fullscreen viewer
- supports swipe on mobile and left/right arrow navigation on desktop

## Project structure

- `server/` Rust backend
- `web/` SvelteKit frontend
- `media/` optional local media folder for development
- `docker-compose.yml` combined container setup

## Ports

Default ports:

- backend: `6677`
- frontend: `9091`

## Requirements

For local development:

- Rust and Cargo
- Node.js and npm

For Docker:

- Docker Desktop or Docker Engine with Compose support

## Important networking note

`0.0.0.0` is only for binding the server. It is not a browser URL.

Examples:

- backend bind address: `0.0.0.0:6677`
- frontend on the same machine: `http://localhost:9091`
- frontend from another device on the LAN: `http://192.168.1.25:9091`

If another device opens the frontend using your machine's LAN IP, the frontend is already set up to use that same host for media URLs by default.

## Run locally

### 1. Configure the backend

Copy the backend env file:

```bash
cp server/.env.example server/.env
```

Edit `server/.env` and set the media folder you want to serve:

```env
BIND_ADDR=0.0.0.0:6677
MEDIA_ROOT=/absolute/path/to/your/media/folder
CORS_ALLOW_ORIGIN=*
```

Example:

```env
MEDIA_ROOT=/Users/yourname/Pictures/ScreenShots
```

On Windows:

```env
MEDIA_ROOT=D:/Pictures/ScreenShots
```

You can also override the media path from the command line. The CLI argument takes priority over `.env`:

```bash
cd server
cargo run -- /absolute/path/to/your/media/folder
```

### 2. Start the backend

```bash
cd server
cargo run
```

The backend will listen on:

```text
http://0.0.0.0:6677
```

From the same machine, open:

```text
http://localhost:6677/api/health
```

From another device on the same network, use your machine's LAN IP:

```text
http://192.168.1.25:6677/api/health
```

### 3. Install frontend dependencies

```bash
cd web
npm install
```

### 4. Optional frontend env file

You usually do not need a frontend env file for local LAN use, because the frontend derives the public API host from the browser request.

If you want to override it explicitly:

```bash
cp web/.env.example web/.env
```

Useful frontend env values:

- `INTERNAL_API_BASE_URL`: server-side fetch URL used by SvelteKit
- `PUBLIC_API_BASE_URL`: explicit browser-facing backend URL override

### 5. Start the frontend

```bash
cd web
npm run dev
```

The frontend runs on:

```text
http://localhost:9091
```

To open it from another device on the LAN, use your machine's LAN IP:

```text
http://192.168.1.25:9091
```

## Run with Docker Compose

The compose file can run both services together, but you must set the media folder volume for your machine before starting it.

### 1. Update the media volume path

Open [docker-compose.yml](/Users/s_mash/Documents/projects/local_gallery/docker-compose.yml) and update the `server.volumes` entry.

Current example:

```yaml
volumes:
  - /Users/s_mash/Pictures/ScreenShots:/app/media:ro
```

Replace it with a path that exists on your machine.

macOS/Linux example:

```yaml
volumes:
  - /Users/yourname/Pictures/ScreenShots:/app/media:ro
```

Windows short syntax example:

```yaml
volumes:
  - "D:/Pictures/ScreenShots:/app/media:ro"
```

Windows long syntax example:

```yaml
volumes:
  - type: bind
    source: D:/Pictures/ScreenShots
    target: /app/media
    read_only: true
```

The long syntax is often easier on Windows because it avoids `D:/...:/app/media` parsing problems.

### 2. Update the public API override if needed

In `docker-compose.yml`, the `web` service may contain:

```yaml
PUBLIC_API_BASE_URL: http://192.168.1.10:6677
```

Set this to your machine's actual LAN IP, or remove it entirely.

If removed, the frontend will derive the public host from the browser request and still work in many local setups.

### 3. Build and start both services

```bash
docker compose up --build
```

### 4. Open the app

Same machine:

```text
http://localhost:9091
```

Another device on the LAN:

```text
http://192.168.1.25:9091
```

## Common commands

Backend:

```bash
cd server
cargo check
cargo run
```

Frontend:

```bash
cd web
npm install
npm run dev
npm run check
```

Docker:

```bash
docker compose up --build
docker compose down
```

## How updates are detected

The backend rescans the media folder when `/api/media` is requested.

That means:

- adding or removing files is picked up on browser refresh
- the server usually does not need a restart for media-folder changes
- no-cache headers are set so refreshes fetch fresh content instead of stale cached media

## Troubleshooting

### The frontend loads but media does not appear from another device

Check:

- the backend is bound to `0.0.0.0:6677`
- you opened the frontend using the machine's LAN IP, not `localhost`
- the OS firewall allows inbound connections on `6677` and `9091`

### Docker Compose fails on Windows with `too many colons`

Use quoted short syntax:

```yaml
- "D:/Pictures/ScreenShots:/app/media:ro"
```

Or use the long bind syntax:

```yaml
- type: bind
  source: D:/Pictures/ScreenShots
  target: /app/media
  read_only: true
```

### Videos play but seeking is not ideal

The frontend uses Vidstack, but the backend still serves files directly. Better range-request support would improve large-video seeking further.

## Next improvements

- add HTTP range support for better video seeking
- add thumbnails for faster large galleries
- add directory filters or search
- add optional auth if the LAN is not trusted
