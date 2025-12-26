import { Elysia, t } from "elysia";
import net from "net";
import dns from "dns/promises";

const pingHost = async (host: string, port = 80, timeout = 5000): Promise<boolean> => {
  try {
    await dns.lookup(host);

    return await new Promise((resolve) => {
      const socket = new net.Socket();
      socket.setTimeout(timeout);

      socket.once("error", () => resolve(false));
      socket.once("timeout", () => {
        socket.destroy();
        resolve(false);
      });

      socket.connect(port, host, () => {
        socket.end();
        resolve(true);
      });
    });
  } catch {
    // DNS lookup failed
    return false;
  }
};

export const pingServer = new Elysia({ prefix: "/api/ping" })
  .post("/", async ({ body, set }) => {
    try {
      const isAlive = await pingHost(body.hostname, body.port, body.timeout);

      set.status = 200;
      return {
        status: isAlive ? "Ok" : "Err",
        time: Date.now(),
      };
    } catch (err) {
      console.error(`ping failed for ${body.hostname}:`, err);
      set.status = 500;
      return {
        status: "Err",
        message: "ping operation failed",
        time: Date.now(),
      };
    }
  }, {
    body: t.Object({
      hostname: t.String(),
      port: t.Number(),
      timeout: t.Number(),
    }),
  });
