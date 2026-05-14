import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import z from "zod";

import { apiResponseMessageSchema } from "$lib/model/message";

export const notificationEventSchema = z.object({
	type: z.string(),
	notificationId: z.string().nullable(),
	ref: z.string().nullable(),
	payload: z.unknown(),
});

export const chatV1MessageSentEventSchema = notificationEventSchema.safeExtend({
	type: z.literal("chat.v1.message_sent"),
	payload: apiResponseMessageSchema,
});

export const chatV1ConversationDeleteEventSchema =
	notificationEventSchema.safeExtend({
		type: z.literal("chat.v1.conversation.delete"),
		payload: z.object({
			conversationIds: z.array(z.string()),
		}),
	});

export type ChatV1MessageSentEventPayload = z.infer<
	typeof chatV1MessageSentEventSchema
>;
export type ChatV1ConversationDeleteEventPayload = z.infer<
	typeof chatV1ConversationDeleteEventSchema
>;

export type WsStatus = "disconnected" | "connecting" | "connected" | "error";

class WsState {
	status = $state<WsStatus>("disconnected");

	constructor() {
		listen<void>("ws:connected", () => {
			this.status = "connected";
			console.log("[ws] connected");
		}).catch(console.error);

		listen<void>("ws:disconnected", () => {
			this.status = "disconnected";
		}).catch(console.error);

		listen<string>("ws:ws_error", (event) => {
			console.error("[ws] server error", event.payload);
		}).catch(console.error);
	}

	connect(): void {
		console.log("[ws] connecting...");
		invoke("ws_connect").catch((e: unknown) => {
			console.error("[ws] connect failed", e);
		});
	}

	send(type: string, payload: unknown): void {
		const ref_id = crypto.randomUUID();
		invoke("ws_send", { command: { type, ref_id, payload } }).catch(
			(e: unknown) => {
				console.error("[ws] send failed", type, e);
			},
		);
	}

	on<T>(
		eventType: string,
		schema: z.ZodType<T>,
		handler: (payload: T) => void,
	): Promise<() => void> {
		const safeName = eventType.replaceAll(".", "_");
		return listen<unknown>(`ws:${safeName}`, (event) => {
			const result = schema.safeParse(event.payload);
			if (result.success) {
				handler(result.data);
			} else {
				console.error(
					`[ws] unexpected payload for ${eventType}:`,
					result.error,
					event.payload,
				);
			}
		});
	}
}

export const ws = new WsState();
