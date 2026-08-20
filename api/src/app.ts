import Fastify from "fastify";
import { metricsRoutes } from "./modules/server/routes.js";

export function buildApp() {
    const app = Fastify({
        logger: true,
    });

    app.register(metricsRoutes);

    return app;
}
