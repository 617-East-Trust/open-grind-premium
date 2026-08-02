import { invoke } from "@tauri-apps/api/core";
import z from "zod";

import { toBase64 } from "$lib/base64";

const mediaUploadResponseSchema = z.object({
	mediaId: z.coerce.number().int(),
	url: z.string().optional().default(""),
	mediaHash: z.string().optional().default(""),
});

export type MediaUploadResponse = z.infer<typeof mediaUploadResponseSchema>;

function imageHashFromUrl(url: string): string {
	// Grindr CDN URLs end with /{hash} or include the hash segment.
	const parts = url.split("/").filter(Boolean);
	const last = parts.at(-1) ?? "";
	const cleaned = last.split("?")[0] ?? "";
	return cleaned.length > 0 ? cleaned : last;
}

/** Coerce whatever the upload returns into a public (40-hex) media hash for Message typing. */
export function asPublicMediaHash(raw: string | null | undefined): string {
	const hex = String(raw ?? "")
		.replace(/[^0-9a-fA-F]/g, "")
		.toLowerCase();
	if (hex.length === 40 || hex.length === 64) return hex;
	if (hex.length > 40) return hex.slice(0, 40);
	return (hex + "0".repeat(40)).slice(0, 40);
}

/**
 * Upload image bytes via the Tauri `upload_image` command
 * (`POST /v5/chat/media/upload`) and normalize the response.
 */
export async function uploadChatMedia(
	bytes: Uint8Array,
	contentType: string,
): Promise<MediaUploadResponse> {
	const raw = await invoke<{ status: number; body: string }>("upload_image", {
		imageBase64: toBase64(bytes),
		mimeType: contentType,
	});

	if (raw.status < 200 || raw.status >= 300) {
		throw new Error(
			`Media upload failed (${raw.status}): ${raw.body.slice(0, 200)}`,
		);
	}

	let parsed: unknown;
	try {
		parsed = JSON.parse(raw.body);
	} catch {
		throw new Error(`Media upload returned non-JSON: ${raw.body.slice(0, 120)}`);
	}

	const rec = (parsed ?? {}) as Record<string, unknown>;
	const mediaId =
		rec.mediaId ?? rec.media_id ?? rec.id ?? (rec as { data?: { mediaId?: unknown } }).data?.mediaId;
	const url = String(rec.url ?? rec.mediaUrl ?? rec.fullImageUrl ?? "");
	const mediaHash = String(
		rec.mediaHash ?? rec.media_hash ?? rec.imageHash ?? (url ? imageHashFromUrl(url) : ""),
	);

	return mediaUploadResponseSchema.parse({
		mediaId,
		url,
		mediaHash,
	});
}

export async function uploadChatMediaFromFile(
	file: File,
): Promise<MediaUploadResponse> {
	const buffer = await file.arrayBuffer();
	const bytes = new Uint8Array(buffer);
	const contentType = file.type || "image/jpeg";
	return uploadChatMedia(bytes, contentType);
}

export { imageHashFromUrl };
