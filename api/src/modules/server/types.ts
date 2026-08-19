export type ServerStatus = "offline" | "online";

export interface Server {
    id: string;
    name: string;
    hostname: string;
    status: ServerStatus;
}