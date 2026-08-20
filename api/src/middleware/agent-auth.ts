import type { FastifyReply, FastifyRequest } from "fastify";

import { AGENT_TOKEN } from "../config/env.js";

export async function requireValidAgentToken(
  request: FastifyRequest,
  reply: FastifyReply,
) {
  const authHeader = Array.isArray(request.headers.authorization)
    ? request.headers.authorization[0]
    : request.headers.authorization;

  if (validateAgentToken(authHeader)) {
    return;
  }

  request.log.warn("Ese token no sirve");

  return reply.status(401).send({
    message: "Ese token no sirve",
  });
}

export function validateAgentToken(authHeader?: string): boolean {
  if (!authHeader) {
    return false;
  }

  const [scheme, token] = authHeader.split(" ");

  return scheme === "Bearer" && token === AGENT_TOKEN;
}
