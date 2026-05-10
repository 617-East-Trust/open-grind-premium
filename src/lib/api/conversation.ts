import z from "zod";
import { fetchRest } from "$lib/api";
import { fullConversationSchema } from "$lib/model/conversation";

const conversationsSchema = z.object({
	entries: z.array(fullConversationSchema),
	nextPage: z.number().nullable(),
});

export async function getConversations(page: number = 1) {
	const conversations = await fetchRest(
		"/v4/inbox?" + new URLSearchParams({ page: String(page) }),
		{
			method: "POST",
		},
	).then((res) => res.jsonParsed(conversationsSchema));
	return conversations;
}
