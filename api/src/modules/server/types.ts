export type ServerStatus = "unreported" | "offline" | "online";

export interface Server {
  id: string;
  name: string;
  hostname: string;
  status: ServerStatus;
}

export interface CreateServerBody {
  name: string;
  hostname: string;
}

export const createServerSchema = {
  type: "object",
  required: ["name", "hostname"],
  additionalProperties: false,
  properties: {
    name: { type: "string", minLength: 1 },
    hostname: { type: "string", minLength: 1 },
  },
} as const;
