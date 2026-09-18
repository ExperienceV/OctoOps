import Fastify from "fastify";
import { metricsRoutes } from "./modules/server/routes.js";
import { serverRoutes } from "./modules/server/server-routes.js";

export function buildApp() {
    const app = Fastify({
        logger: true,
        ajv: {
            customOptions: {
                removeAdditional: false,
            },
        },
    });

    app.register(metricsRoutes);
    app.register(serverRoutes, { prefix: "/api/v1" });

    return app;
}
