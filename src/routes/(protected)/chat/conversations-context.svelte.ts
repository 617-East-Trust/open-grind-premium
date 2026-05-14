import { createContext } from "svelte";

import type { ConversationsState } from "./conversations.svelte";

export const [getConversations, setConversations] =
	createContext<ConversationsState>();
