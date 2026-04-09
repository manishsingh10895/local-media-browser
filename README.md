# Local Gallery

Base application for serving images and videos across a local network with:

- Rust + `axum` backend
- SvelteKit frontend

## Structure

- `server/`: API and media file serving
- `web/`: SvelteKit frontend

## Backend features

- `GET /api/health`
- `GET /api/media`
- `GET /media/*path`
- configurable bind address for LAN access
- configurable media library directory

## Quick start

### 1. Backend

Copy the example env file:

```bash
cp server/.env.example server/.env
```

Run the server:

```bash
cd server
cargo run
```

By default it listens on `0.0.0.0:6677`, so devices on your local network can reach it using your machine's LAN IP.

### 2. Frontend

Install dependencies:

```bash
cd web
npm install
```

Start the frontend:

```bash
npm run dev
```

The frontend defaults to `0.0.0.0:9091`.

Set the frontend API origin if needed:

```bash
cp .env.example .env
```

If the frontend and backend run on the same machine, `VITE_API_BASE_URL=http://<your-lan-ip>:6677` is usually enough.

## Docker Compose

Run both services together:

```bash
docker compose up --build
```

This exposes:

- backend on `6677`
- frontend on `9091`

The compose file mounts `./media` into the backend container at `/app/media`.

## Suggested next steps

1. Add thumbnail generation for large media libraries.
2. Add directory browsing and search.
3. Add optional auth if the LAN is not trusted.
4. Add video streaming improvements such as byte-range handling through a dedicated media service.
