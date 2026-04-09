FROM node:24-bookworm-slim

WORKDIR /app

COPY package.json ./
COPY svelte.config.js ./
COPY tsconfig.json ./
COPY vite.config.ts ./
COPY src ./src
COPY static ./static

RUN npm install

EXPOSE 9091

CMD ["npm", "run", "dev"]
