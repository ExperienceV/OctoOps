import { randomUUID } from "node:crypto";

import type { CreateServerBody, Server } from "./types.js";

const servers: Server[] = [];

export const serverService = {
  getAll(): Server[] {
    return servers;
  },

  getById(id: string): Server | undefined {
    return servers.find((server) => server.id === id);
  },

  create(data: CreateServerBody): Server {
    const server: Server = {
      id: randomUUID(),
      name: data.name,
      hostname: data.hostname,
      status: "unreported",
    };

    servers.push(server);

    return server;
  },
};
