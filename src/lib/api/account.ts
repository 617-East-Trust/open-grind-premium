import z from "zod";

import { fetchRest } from "$lib/api";

// ── Blocked ──────────────────────────────────────────────

const blockedListSchema = z.object({
	profiles: z
		.array(
			z.object({
				profileId: z.number().or(z.string()).transform(Number),
				displayName: z.string().nullable().optional(),
				profileImageMediaHash: z.string().nullable().optional(),
			}),
		)
		.optional()
		.default([]),
});

export async function getBlockedUsers() {
	return fetchRest("/v1/blocks").then((r) => r.jsonParsed(blockedListSchema));
}

export async function unblockUser(profileId: number) {
	return fetchRest(`/v1/blocks/${profileId}`, { method: "DELETE" });
}

// ── Hidden ───────────────────────────────────────────────

const hiddenListSchema = z.object({
	profiles: z
		.array(
			z.object({
				profileId: z.number().or(z.string()).transform(Number),
				displayName: z.string().nullable().optional(),
				profileImageMediaHash: z.string().nullable().optional(),
			}),
		)
		.optional()
		.default([]),
});

export async function getHiddenUsers() {
	// Grindr historically uses a "favorites" style or dedicated hide list.
	// Adjust path once real traffic is observed.
	return fetchRest("/v1/hides").then((r) => r.jsonParsed(hiddenListSchema));
}

export async function unhideUser(profileId: number) {
	return fetchRest(`/v1/hides/${profileId}`, { method: "DELETE" });
}

// ── Email / Password ─────────────────────────────────────

export async function changeEmail(newEmail: string, password: string) {
	return fetchRest("/v1/me/email", {
		method: "PUT",
		body: { email: newEmail, password },
	});
}

export async function changePassword(currentPassword: string, newPassword: string) {
	return fetchRest("/v1/me/password", {
		method: "PUT",
		body: { currentPassword, newPassword },
	});
}

// ── Delete account ───────────────────────────────────────

export async function deleteAccount(password: string) {
	return fetchRest("/v1/me", {
		method: "DELETE",
		body: { password },
	});
}