import { Elysia } from "elysia";
import { stats } from "./stats";
import { users } from "./user";
import cors from "@elysiajs/cors";
import { openapi } from '@elysiajs/openapi'
import { pingServer } from "./ping";
import { lastfm } from "./lastfm";
import { instrumentation } from "./instrumentation";
import { z } from "zod"
import { drizzle } from "drizzle-orm/node-postgres";
import { migrate } from "drizzle-orm/node-postgres/migrator";
import 'dotenv/config'

const envSchema = z.object({
  DATABASE_URL: z.url({ error: "Database URL is invalid" }),
  LASTFM_APIKEY: z.string(),
  LASTFM_URI: z.url({ error: "LastFM URL is invalid" }),
  LASTFM_USERNAME: z.string(),
})

console.log('LASTFM_URI:', process.env.LASTFM_URI)
const result = envSchema.safeParse(process.env)
if (!result.success) {
  console.error('Invalid environment variables:', result.error)
  process.exit(1)
}

const env = result.data
export const db = drizzle(env.DATABASE_URL!)
await migrate(db, { migrationsFolder: "./drizzle/migrations" })

const logger = new Elysia()
  .onRequest(({ request, set }) => {
    const currTime = Date.now();
    set.headers['x-start'] = currTime;
    console.log(`[REQ] - ${request.method} -> ${request.url}`);
  })
  .onAfterHandle(({ request, set }) => {
    const currTime = Date.now();
    const time = currTime - Number(set.headers['x-start']);
    console.log(`[RES] - ${request.method} -> ${request.url} - took ${time}ms`);
  })
  .onError(({ error }) => {
    console.error(error);
  })

new Elysia()
  .use(instrumentation)
  .use(logger)
  .use(cors())
  .use(openapi())
  .get('/api/healthz', ({ status }) => {
    return status(200, {
      health: "ok"
    })
  })
  .use(lastfm)
  .use(pingServer)
  .use(stats)
  .use(users)
  .listen(3000)

console.log('server running at https://localhost:4000')
