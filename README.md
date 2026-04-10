# Local Gallery

Local Gallery is a LAN-friendly media browser built with:

- Rust + `axum` backend
- SvelteKit frontend
- optional Axum-rendered SSR frontend
- optional Docker Compose setup

It is designed to serve a local media directory over your local network and browse it from desktop or mobile devices.

## Current feature set

- recursive media discovery across nested subfolders
- media-only indexing: non-image and non-video files are ignored
- folder-based browsing with breadcrumbs
- folder cover thumbnails
- sort by `name`, `date`, or `size`
- grid size toggle: `compact`, `comfortable`, `large`
- infinite scroll for media inside large folders
- fullscreen image/video viewer
- swipe on mobile, arrow-key navigation on desktop
- per-file downloads
- thumbnail caching for faster grid rendering
- live index refresh through a filesystem watcher

## Current architecture

The current codebase no longer serves the frontend from one flat full-library media payload.

It now works like this:

- the backend scans the media root once at startup
- it builds an in-memory media index grouped by folder
- a filesystem watcher refreshes the index when files change
- the frontend requests only the current folder view from the backend
- grid cards use cached thumbnails
- fullscreen viewing still uses the original media file

This is intended to scale better for large libraries than rescanning and returning the whole tree on every page load.

The backend can now run in two frontend modes:

- `FRONTEND_MODE=svelte`: Rust serves the API/media/thumbnail routes, and SvelteKit serves the UI
- `FRONTEND_MODE=axum`: Rust also serves an Axum-rendered HTML UI directly on the backend port

The default is `svelte`.

## Project structure

- `server/` Rust backend
- `web/` SvelteKit frontend
- `media/` optional local media folder for development
- `docker-compose.yml` combined container setup

### Server structure

The Rust backend has been refactored into smaller modules:

- `server/src/main.rs`: startup and router wiring
- `server/src/config.rs`: env parsing, runtime config, frontend mode, sort/view enums
- `server/src/models.rs`: shared API/index data types
- `server/src/state.rs`: shared app state
- `server/src/indexer.rs`: media scanning, index building, watcher refresh
- `server/src/handlers/api.rs`: `/api/*` handlers
- `server/src/handlers/assets.rs`: `/media/*` and `/thumbs/*` handlers
- `server/src/responses.rs`: folder response assembly
- `server/src/paths.rs`: path sanitizing, breadcrumbs, URL helpers
- `server/src/sorting.rs`: folder/media sorting
- `server/src/thumbnails.rs`: thumbnail generation and fallback SVG responses
- `server/src/frontend/`: Axum SSR frontend renderer and styles

## Default ports

- backend: `6677`
- frontend: `9091`

## Requirements

Local development:

- Rust and Cargo
- Node.js and npm

Docker:

- Docker Desktop or Docker Engine with Compose support

## Important networking note

`0.0.0.0` is only for binding the server. It is not a browser URL.

Examples:

- backend bind address: `0.0.0.0:6677`
- frontend on the same machine: `http://localhost:9091`
- frontend from another device on the LAN: `http://192.168.1.25:9091`

If another device opens the frontend using your machine's LAN IP, the frontend is already set up to use that same host for media and thumbnail URLs by default unless you explicitly override it.

## Supported media behavior

- media files are identified by MIME/extension using `mime_guess`
- only `image/*` and `video/*` are indexed
- non-media files are not listed and are not directly servable through the media route

### Thumbnails

- image thumbnails are generated lazily on first request
- thumbnails are cached on disk in your system temp directory
- video thumbnails currently use a generated placeholder image
- unsupported image formats for the Rust thumbnail pipeline, such as SVG, currently fall back to a placeholder thumbnail instead of a generated raster thumbnail

Thumbnail cache location:

```text
<system temp dir>/local-gallery-thumbnails/<hash-of-media-root>
```

Examples:

- Linux: `/tmp/local-gallery-thumbnails/<hash>`
- macOS: often under `/var/folders/.../T/local-gallery-thumbnails/<hash>`

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
FRONTEND_MODE=svelte
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

Same machine:

```text
http://localhost:6677/api/health
```

Another device on the same network:

```text
http://192.168.1.25:6677/api/health
```

### 3. Run Axum-only SSR instead of SvelteKit

If you want the Rust server to render the UI directly, set:

```env
FRONTEND_MODE=axum
```

Then run:

```bash
cd server
cargo run
```

Open:

- same machine: `http://localhost:6677`
- another device on the LAN: `http://192.168.1.25:6677`

Notes:

- the server should still bind to `0.0.0.0:6677`
- clients should use `localhost` or your real LAN IP, not `0.0.0.0`
- in Axum SSR mode, you do not need to run the Svelte frontend at all

### 4. Install frontend dependencies

```bash
cd web
npm install
```

### 5. Optional frontend env file

You usually do not need a frontend env file for local LAN use, because the frontend derives the public API host from the browser request.

If you want to override it explicitly:

```bash
cp web/.env.example web/.env
```

Useful frontend env values:

- `INTERNAL_API_BASE_URL`: server-side fetch URL used by SvelteKit
- `PUBLIC_API_BASE_URL`: explicit browser-facing backend URL override

### 6. Start the frontend

```bash
cd web
npm run dev
```

Same machine:

```text
http://localhost:9091
```

Another device on the LAN:

```text
http://192.168.1.25:9091
```

Use this only when `FRONTEND_MODE=svelte`.

## Run with Docker Compose

The compose file can run both services together, but you must update the media volume path for your machine first.

### 1. Update the media volume path

Open [docker-compose.yml](/Users/s_mash/Documents/projects/local_gallery/docker-compose.yml) and change the `server.volumes` entry.

Current example:

```yaml
volumes:
  - /Users/s_mash/Pictures/ScreenShots:/app/media:ro
```

Replace it with a real path on your machine.

macOS/Linux example:

```yaml
volumes:
  - /Users/yourname/Pictures/ScreenShots:/app/media:ro
```

Windows short syntax:

```yaml
volumes:
  - "D:/Pictures/ScreenShots:/app/media:ro"
```

Windows long syntax:

```yaml
volumes:
  - type: bind
    source: D:/Pictures/ScreenShots
    target: /app/media
    read_only: true
```

The long syntax is usually safer on Windows because it avoids drive-letter parsing issues.

### 2. Update the public API override if needed

The current `docker-compose.yml` contains:

```yaml
PUBLIC_API_BASE_URL: http://192.168.1.10:6677
```

Set this to your actual LAN IP, or remove it if you want the frontend to derive the public host from the browser request.

### 3. Start the stack

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

## UI usage

### Folder browsing

- root and nested folders are route-based
- click a folder card to enter it
- use breadcrumbs to navigate back up

### Sorting

Current folder sorting is stored in the URL query string.

Available sort fields:

- `name`
- `date`
- `size`

Direction:

- `asc`
- `desc`

### Grid size

Available view sizes:

- `compact`
- `comfortable`
- `large`

This is also stored in the URL query string.

Mobile behavior:

- `compact`: 3 items per row
- `comfortable`: 2 items per row
- `large`: 1 item per row

### Downloads

- download is per media file only
- folder download is not implemented
- download is available from gallery cards and the fullscreen viewer

## API overview

Current browser-facing backend routes:

- `GET /api/health`
- `GET /api/folder`
- `GET /api/folder/{path}`
- `GET /media/{path}`
- `GET /thumbs/{path}`

When `FRONTEND_MODE=axum`, the backend also serves:

- `GET /`
- `GET /{path}`

### Folder endpoint query params

`/api/folder` and `/api/folder/{path}` support:

- `sort=name|date|size`
- `dir=asc|desc`
- `offset=<number>`
- `limit=<number>`

These are used by the current SvelteKit folder route and infinite scroll behavior.

### Media download

Download uses the same media route with:

```text
/media/{path}?download=true
```

## Common commands

Backend:

```bash
cd server
cargo check
cargo run
```

Backend with Axum SSR:

```bash
cd server
FRONTEND_MODE=axum cargo run
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

## Current behavior for large folders

The app now handles large libraries better than the original implementation because:

- the server keeps an in-memory index instead of rescanning on every browser request
- the frontend loads only the current folder view
- media lists are paginated
- the page appends more media with infinite scroll
- thumbnails are used in the grid instead of original full-size media

## Troubleshooting

### I want Axum-only SSR on my LAN

Set:

```env
BIND_ADDR=0.0.0.0:6677
FRONTEND_MODE=axum
```

Then run:

```bash
cd server
cargo run
```

Open from another device using:

```text
http://<your-lan-ip>:6677
```

Do not use `0.0.0.0` in the browser URL.

### Frontend loads but media does not show on another device

Check:

- backend is bound to `0.0.0.0:6677`
- you opened the frontend using the machine's LAN IP, not `localhost`
- your OS firewall allows inbound traffic on `6677` and `9091`

### Docker Compose fails on Windows with `too many colons`

Use quoted short syntax:

```yaml
- "D:/Pictures/ScreenShots:/app/media:ro"
```

Or use long bind syntax:

```yaml
- type: bind
  source: D:/Pictures/ScreenShots
  target: /app/media
  read_only: true
```

### SVG files show a placeholder thumbnail

This is expected in the current version.

SVG files are still indexed as media, but the current Rust thumbnail generator does not rasterize SVG, so the server returns a fallback thumbnail instead.

### Video seeking is still basic

The frontend uses Vidstack, but the backend still serves original files directly without advanced range/streaming optimization. Large-video seeking can still be improved later.

## Known limitations

- video thumbnails are placeholders, not extracted frames
- SVG thumbnails use a placeholder
- folder download is not implemented
- authentication is not implemented
- advanced search/filtering is not implemented
- video range streaming is not yet specialized

## Suggested next improvements

- proper video frame thumbnail extraction
- HTTP range support for stronger video playback
- incremental index updates instead of full watcher-triggered rebuilds
- full-text or filename search
- optional authentication for LAN sharing
