FROM oven/bun AS build

WORKDIR /app

COPY package.json package.json
COPY bun.lock bun.lock

RUN bun install

COPY ./src ./src
COPY ./drizzle ./drizzle

ENV NODE_ENV=production

RUN bun build \
    --compile \
		--minify-whitespace \
		--minify-syntax \
		--outfile server \
		src/index.ts

FROM oven/bun:slim

WORKDIR /app

COPY --from=build /app/server server
COPY --from=build /app/drizzle ./drizzle

ENV NODE_ENV=production

CMD ["./server"]

EXPOSE 3000
