import type { FastifyInstance } from "fastify";

import { requireValidAgentToken } from "../../middleware/agent-auth.js";
import { metricsSchema } from "./contract.js";
import type { Metrics } from "./contract.js";

export async function metricsRoutes(app: FastifyInstance) {
  app.post<{ Body: Metrics }>(
    "/metrics",
    {
      preHandler: requireValidAgentToken,
      schema: {
        body: metricsSchema,
      },
    },
    async (request) => {
      request.log.info({ metrics: request.body }, "metricas recibidas:");

      return {
        message: "Metricas recibidas",
      };
    },
  );
}
