import { readFileSync } from "node:fs";

export const AGENT_TOKEN = process.env.AGENT_TOKEN ?? readEnvValue("AGENT_TOKEN");

function readEnvValue(key: string): string | undefined {
  try {
    const content = readFileSync(new URL("../../.env", import.meta.url), "utf8");

    for (const line of content.split(/\r?\n/)) {
      const trimmed = line.trim();

      if (!trimmed || trimmed.startsWith("#")) {
        continue;
      }

      const [currentKey, ...rest] = trimmed.split("=");
      if (currentKey !== key) {
        continue;
      }

      return rest.join("=").trim();
    }
  } catch {
    return undefined;
  }

  return undefined;
}
