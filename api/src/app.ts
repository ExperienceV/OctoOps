import Fastify from "fastify";
import { serverRoutes } from "./modules/server/routes.js";

export function buildApp() {
    const app = Fastify({
        logger: true,
    });

    app.get("/health", async () => {
        return {
            status: "ok",
            service: "octoops",
        };
    });

    app.register(serverRoutes, 
        { prefix: "/api" }
    );

    return app;
}