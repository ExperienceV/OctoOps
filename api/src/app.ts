import Fastify from "fastify";
import { metricsRoutes } from "./modules/server/routes.js";

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

    return app;
}
