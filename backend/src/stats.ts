import { Elysia, t } from "elysia";
import 'dotenv/config';
import { eq } from 'drizzle-orm'
import { statTable } from "./db/schema";
import { db } from "./index";

export const stats = new Elysia({ prefix: "/api/stats" })
  .get('/:id', async ({ params, status }) => {
    try {
      let stats = await db.selectDistinct().from(statTable).where(eq(statTable.user_id, params.id))
      return status(200, {
        stats: stats[0],
        message: 'successfully fetched stats'
      })
    } catch (err) {
      console.error("Failed to send stats: ", err)
      return status(500, {
        message: "failed to fetch stats",
      })
    }
  }, {
    params: t.Object({
      id: t.String()
    })
  })
  .put('/:id', async ({ params, status, body }) => {
    try {
      let stats = await db
        .selectDistinct()
        .from(statTable)
        .where(eq(statTable.user_id, params.id))

      await db.update(statTable).set({
        keypress: stats[0].keypress + body.keypress,
        right_click: stats[0].right_click + body.right_click,
        left_click: stats[0].left_click + body.left_click,
        middle_click: stats[0].middle_click + body.middle_click,
        mouse_distance: stats[0].mouse_distance + body.mouse_distance,
        scroll_distance: stats[0].scroll_distance + body.scroll_distance,
      }).where(eq(statTable.user_id, params.id))

      return status(200, {
        message: "updated stats successfully"
      })

    } catch (err) {
      console.error("Failed to update stats", err)
      return status(500, {
        message: "Failed to update stats"
      })
    }
  }, {
    params: t.Object({
      id: t.String()
    }),
    body: t.Object({
      keypress: t.Number({ minimum: 0 }),
      right_click: t.Number({ minimum: 0 }),
      left_click: t.Number({ minimum: 0 }),
      middle_click: t.Number({ minimum: 0 }),
      mouse_distance: t.Number({ minimum: 0.0 }),
      scroll_distance: t.Number({ minimum: 0.0 }),
    })
  })
