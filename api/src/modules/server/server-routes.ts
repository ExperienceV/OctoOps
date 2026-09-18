import type { FastifyInstance } from "fastify";

import { serverService } from "./service.js";
import {
  createServerSchema,
  type CreateServerBody,
} from "./types.js";

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

  app.post<{ Body: CreateServerBody }>(
    "/servers",
    {
      schema: {
        body: createServerSchema,
      },
    },
    async (request, reply) => {
      const server = serverService.create(request.body);

      return reply.status(201).send(server);
    },
  );
}
