import type { FastifyReply, FastifyRequest } from "fastify";

import { AGENT_TOKEN } from "../config/env.js";

export async function requireValidAgentToken(
  request: FastifyRequest,
  reply: FastifyReply,
) {
  const authHeader = Array.isArray(request.headers.authorization)
    ? request.headers.authorization[0]
    : request.headers.authorization;
  const { token } = parseAuthorizationHeader(authHeader);

  request.log.info(
    {
      receivedToken: maskToken(token),
      expectedToken: maskToken(AGENT_TOKEN),
    },
    "Comparando token del agente",
  );

  if (validateAgentToken(authHeader)) {
    return;
  }

  request.log.warn("Ese token no sirve");

  return reply.status(401).send({
    message: "Ese token no sirve",
  });
}

export function validateAgentToken(authHeader?: string): boolean {
  const { scheme, token } = parseAuthorizationHeader(authHeader);

  return scheme === "Bearer" && token === AGENT_TOKEN;
}

function parseAuthorizationHeader(authHeader?: string): {
  scheme: string | undefined;
  token: string | undefined;
} {
  if (!authHeader) {
	    return {
	      scheme: undefined,
	      token: undefined,
	    };
  }

  const [scheme, token] = authHeader.split(" ");

  return { scheme, token };
}

function maskToken(token?: string): string | null {
  if (!token) {
    return null;
  }

  if (token.length <= 6) {
    return token;
  }

  return `${token.slice(0, 3)}.........${token.slice(-3)}`;
}
