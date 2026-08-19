import type { FastifyInstance } from "fastify";
import { randomUUID } from "node:crypto";

import { serverService } from "./service.js";
import type { Server } from "./types.js";

export async function serverRoutes(app: FastifyInstance) {
  app.get("/servers", async () => {
    return serverService.getAll();
  });

  app.get<{ Params: { id: string } }>(
    "/servers/:id",
    async (request, reply) => {
      const server = serverService.getById(request.params.id);

      if (!server) {
        return reply.status(404).send({
          message: "Server not found",
        });
      }

      return server;
    },
  );

  app.post<{ Body: { name: string; hostname: string } }>(
    "/servers",
    async (request, reply) => {
      const server: Server = {
        id: randomUUID(),
        name: request.body.name,
        hostname: request.body.hostname,
        status: "offline",
      };

      serverService.create(server);

      return reply.status(201).send(server);
    },
  );
}