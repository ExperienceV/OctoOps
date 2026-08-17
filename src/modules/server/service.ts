import type { Server } from "./types.js";

const servers: Server[] = [];

export const serverService = {
  getAll(): Server[] {
    return servers;
  },

  getById(id: string): Server | undefined {
    return servers.find((server) => server.id === id);
  },

  create(server: Server): Server {
    servers.push(server);

    return server;
  },
};