import { cors } from "@elysiajs/cors";
import { Elysia } from "elysia";
import { deviceRoute } from "./routes/devices";

const app = new Elysia()
	.use(
		cors({
			origin: Bun.env.FRONTEND_URL
				? Bun.env.FRONTEND_URL.split(",").map((origin) => origin.trim())
				: true,
			allowedHeaders: [
				"Content-Type",
				"Authorization",
				"Referrer-Policy",
				"user-agent",
			],
			methods: ["GET", "POST", "DELETE", "PUT", "OPTIONS"],
			credentials: true,
		}),
	)
	.use(deviceRoute)
	.get("/health", () => ({ status: "ok" }))
	.listen({ port: Number(Bun.env.PORT ?? 3000), hostname: "0.0.0.0" });

console.log(
	`Bambu LAN Monitor API is running at ${app.server?.hostname}:${app.server?.port}`,
);

export type App = typeof app;
