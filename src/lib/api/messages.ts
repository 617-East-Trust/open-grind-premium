import z from "zod";
import { fetchRest } from "$lib/api";
import { apiResponseMessageSchema, messageSchema } from "$lib/model/message";

const conversationMessagesSchema = z.object({
	messages: z.array(apiResponseMessageSchema),
	profile: z.object({
		distance: z.number().nullable(),
		mediaHash: z.string().nullable(),
		name: z.string().nullable(),
		onlineUntil: z.number().nullable(),
		profileId: z.number().int(),
		showDistance: z.boolean(),
	}),
});

export async function getConversationMessages(conversationId: string) {
	const messages = await fetchRest(
		`/v5/chat/conversation/${conversationId}/message?` +
			new URLSearchParams({ profile: "true" }),
		{
			method: "GET",
		},
	).then((res) => res.jsonParsed(conversationMessagesSchema));
	return messages;
}

export async function sendMessage({
	toUserId,
	message,
}: {
	toUserId: number;
	message: z.infer<typeof messageSchema>;
}) {
	return await fetchRest("/v4/chat/message/send", {
		method: "POST",
		body: {
			type: message.type,
			target: {
				type: "Direct",
				targetId: toUserId,
			},
			body: message.body,
		},
	});
}

export async function reactToMessage({
	conversationId,
	messageId,
	reactionId,
}: {
	conversationId: string;
	messageId: string;
	reactionId: number;
}) {
	return await fetchRest("/v4/chat/message/reaction", {
		method: "POST",
		body: {
			conversationId,
			messageId,
			reactionType: reactionId,
		},
	});
}
