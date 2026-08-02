import z from "zod";

import { fetchRest } from "$lib/api";
import { getCascadeV3 } from "$lib/api/grid";
import { getProfiles } from "$lib/api/profile";
import {
	coerceRightNowStatus,
	isActiveRightNowStatus,
	type RightNowFeedItem,
	rightNowFeedItemSchema,
	rightNowStatusSchema,
} from "$lib/model/right-now";
import { urlSearchParamsCodec } from "$lib/utils";

const feedQuerySchema = z.object({
	sort: z.string().optional(),
	hosting: z.boolean().optional(),
	ageMin: z.int().optional(),
	ageMax: z.int().optional(),
	sexualPositions: z.string().optional(),
});

export type RightNowFeedQuery = z.infer<typeof feedQuerySchema>;

/**
 * Pull a field from a loose object under several possible API key names.
 */
function pick(
	obj: Record<string, unknown>,
	keys: string[],
): unknown {
	for (const key of keys) {
		if (obj[key] !== undefined && obj[key] !== null) return obj[key];
		// camelCase / snake_case tolerant
		const lower = key.toLowerCase();
		for (const [k, v] of Object.entries(obj)) {
			if (k.toLowerCase() === lower && v !== undefined && v !== null) return v;
		}
	}
	return undefined;
}

function asRecord(value: unknown): Record<string, unknown> | null {
	if (value && typeof value === "object" && !Array.isArray(value)) {
		return value as Record<string, unknown>;
	}
	return null;
}

function asArray(value: unknown): unknown[] {
	return Array.isArray(value) ? value : [];
}

/**
 * Normalize a raw `/v5/rightnow/feed` (or similar) payload into feed items.
 * Response shapes are WIP in OpenAPI (`UndocumentedObject`), so this is
 * intentionally defensive across observed key names.
 */
export function normalizeRightNowFeedPayload(raw: unknown): RightNowFeedItem[] {
	const root = asRecord(raw) ?? {};
	const candidates = [
		...asArray(root.posts),
		...asArray(root.items),
		...asArray(root.profiles),
		...asArray(root.results),
		...asArray(root.feed),
		// sometimes nested under data
		...asArray(asRecord(root.data)?.posts),
		...asArray(asRecord(root.data)?.items),
		...asArray(asRecord(root.data)?.profiles),
	];

	const items: RightNowFeedItem[] = [];
	for (const entry of candidates) {
		const rec = asRecord(entry);
		if (!rec) continue;

		// Nested profile objects are common
		const nested =
			asRecord(rec.profile) ??
			asRecord(rec.user) ??
			asRecord(rec.profileSummary) ??
			{};

		const profileId = pick(rec, [
			"profileId",
			"profile_id",
			"id",
			"userId",
		]) ?? pick(nested, ["profileId", "profile_id", "id"]);

		if (profileId == null) continue;

		const media =
			asRecord(rec.media) ??
			asRecord(rec.rightNowMedia) ??
			asArray(rec.medias)[0] ??
			asArray(rec.rightNowMedias)[0];
		const mediaRec = asRecord(media) ?? {};

		const statusRaw =
			pick(rec, [
				"rightNow",
				"status",
				"rightNowStatus",
				"rightNowType",
				"hostingStatus",
			]) ??
			pick(nested, ["rightNow", "status"]) ??
			(pick(rec, ["hosting", "isHosting"]) === true
				? "HOSTING"
				: pick(rec, ["hosting", "isHosting"]) === false
					? "NOT_HOSTING"
					: null);

		const item = rightNowFeedItemSchema.safeParse({
			profileId,
			displayName:
				pick(rec, ["displayName", "display_name", "name"]) ??
				pick(nested, ["displayName", "display_name", "name"]) ??
				null,
			status: coerceRightNowStatus(statusRaw),
			text:
				pick(rec, [
					"rightNowText",
					"text",
					"body",
					"caption",
					"message",
					"statusText",
				]) ?? null,
			postedAt:
				pick(rec, [
					"rightNowPosted",
					"postedAt",
					"posted",
					"createdAt",
					"created_at",
					"timestamp",
				]) ?? null,
			distance:
				pick(rec, [
					"rightNowDistance",
					"distance",
					"distanceMeters",
					"distance_meters",
				]) ??
				pick(nested, ["distance", "distanceMeters"]) ??
				null,
			thumbnailUrl:
				pick(rec, [
					"rightNowThumbnailUrl",
					"thumbnailUrl",
					"thumbnail",
					"thumbUrl",
				]) ??
				pick(mediaRec, ["thumbnailUrl", "thumbnail", "url"]) ??
				null,
			fullImageUrl:
				pick(rec, [
					"rightNowFullImageUrl",
					"fullImageUrl",
					"imageUrl",
					"photoUrl",
				]) ??
				pick(mediaRec, ["fullImageUrl", "url"]) ??
				null,
			profileImageMediaHash:
				pick(rec, [
					"profileImageMediaHash",
					"mediaHash",
					"profileImageHash",
				]) ??
				pick(nested, [
					"profileImageMediaHash",
					"mediaHash",
					"profileImageHash",
				]) ??
				null,
			age:
				pick(rec, ["age"]) ??
				pick(nested, ["age"]) ??
				null,
			postId:
				pick(rec, ["postId", "post_id", "id"]) ?? null,
			source: "feed",
		});

		if (item.success) items.push(item.data);
	}

	return items;
}

/**
 * Dedicated Right Now feed (`GET /v5/rightnow/feed`).
 * May fail if the account/region has no session entitlement — callers
 * should fall back to {@link getRightNowFromCascade}.
 */
export async function getRightNowFeed(
	query: RightNowFeedQuery = {},
): Promise<RightNowFeedItem[]> {
	const params = new URLSearchParams(
		urlSearchParamsCodec(feedQuerySchema).encode(query),
	);
	const qs = params.toString();
	const path = qs ? `/v5/rightnow/feed?${qs}` : "/v5/rightnow/feed";

	const raw = await fetchRest(path).then((res) => res.json());
	return normalizeRightNowFeedPayload(raw);
}

/**
 * Cascade-based Right Now list: `/v3/cascade?rightNow=true` then enrich
 * with `/v3/profiles` for text/thumbnail fields. This is the reliable path
 * used when the dedicated feed is gated or schema-unknown.
 */
export async function getRightNowFromCascade(options: {
	geohash: string;
	hosting?: boolean | null;
	pageNumber?: number;
}): Promise<{
	items: RightNowFeedItem[];
	nextPage: number | null;
}> {
	const response = await getCascadeV3({
		nearbyGeoHash: options.geohash,
		rightNow: true,
		pageNumber: options.pageNumber ?? 0,
	});

	type CascadeHit = {
		profileId: number;
		displayName: string | null;
		distance: number | null;
		status: string | null;
		photoHash: string | null;
	};

	const hits: CascadeHit[] = [];

	for (const item of response.items) {
		if (
			item.type !== "full_profile_v1" &&
			item.type !== "partial_profile_v1"
		) {
			continue;
		}
		const data = item.data;
		const status = coerceRightNowStatus(data.rightNow);
		// Cascade already filtered with rightNow=true; drop fully inactive only.
		if (String(data.rightNow).toUpperCase() === "NOT_ACTIVE") {
			continue;
		}
		if (options.hosting === true && status !== "HOSTING") continue;
		if (options.hosting === false && status === "HOSTING") continue;

		const photoHashes =
			"photoMediaHashes" in data ? data.photoMediaHashes : null;
		const photoHash =
			Array.isArray(photoHashes) && photoHashes.length > 0
				? (photoHashes[0] ?? null)
				: null;

		hits.push({
			profileId: data.profileId,
			displayName: data.displayName ?? null,
			distance: data.distanceMeters ?? null,
			status: status == null ? null : String(status),
			photoHash,
		});
	}

	// Enrich with profile Right Now fields (text, media, posted time)
	const ids = hits.map((h) => h.profileId);
	const profiles = await getProfiles(ids);
	const byId = new Map(profiles.map((p) => [p.profileId, p]));

	const items: RightNowFeedItem[] = [];
	for (const hit of hits) {
		const p = byId.get(hit.profileId);
		const status =
			coerceRightNowStatus(p?.rightNow ?? hit.status) ?? hit.status;

		if (options.hosting === true && status !== "HOSTING") continue;
		if (options.hosting === false && status === "HOSTING") continue;

		// Drop profiles that lost RN status by the time we fetched details
		if (p && !isActiveRightNowStatus(status) && p.rightNow === "NOT_ACTIVE") {
			continue;
		}

		const mediaHash =
			p?.medias?.[0]?.mediaHash ?? hit.photoHash ?? null;

		const parsed = rightNowFeedItemSchema.safeParse({
			profileId: hit.profileId,
			displayName: p?.displayName ?? hit.displayName,
			status,
			text: p?.rightNowText ?? null,
			postedAt: p?.rightNowPosted ?? null,
			distance: p?.rightNowDistance ?? p?.distance ?? hit.distance,
			thumbnailUrl: p?.rightNowThumbnailUrl ?? null,
			fullImageUrl: p?.rightNowFullImageUrl ?? null,
			profileImageMediaHash: mediaHash,
			age: p?.age ?? null,
			postId: null,
			source: "cascade",
		});
		if (parsed.success) items.push(parsed.data);
	}

	return { items, nextPage: response.nextPage };
}

function feedPayloadLooksRecognized(raw: unknown): boolean {
	const root = asRecord(raw);
	if (!root) return false;
	const keys = ["posts", "items", "profiles", "results", "feed"];
	if (keys.some((k) => Array.isArray(root[k]))) return true;
	const data = asRecord(root.data);
	if (data && keys.some((k) => Array.isArray(data[k]))) return true;
	return false;
}

/**
 * Load Right Now for the tab: try dedicated feed first, fall back to cascade.
 */
export async function loadRightNowFeed(options: {
	geohash: string;
	hosting?: boolean | null;
	sort?: string;
}): Promise<{
	items: RightNowFeedItem[];
	source: "feed" | "cascade";
	nextPage: number | null;
}> {
	// Dedicated feed (no geohash param in OpenAPI — server uses session location)
	try {
		const params = new URLSearchParams(
			urlSearchParamsCodec(feedQuerySchema).encode({
				hosting: options.hosting ?? undefined,
				sort: options.sort,
			}),
		);
		const qs = params.toString();
		const path = qs ? `/v5/rightnow/feed?${qs}` : "/v5/rightnow/feed";
		const raw = await fetchRest(path).then((res) => res.json());
		const items = normalizeRightNowFeedPayload(raw);

		// Accept feed when we recognized its shape (even if empty = nobody posting).
		// Unknown/undocumented shapes fall through to cascade.
		if (feedPayloadLooksRecognized(raw) || items.length > 0) {
			let filtered = items;
			if (options.hosting === true) {
				filtered = items.filter((i) => i.status === "HOSTING");
			} else if (options.hosting === false) {
				filtered = items.filter((i) => i.status !== "HOSTING");
			}
			return { items: filtered, source: "feed", nextPage: null };
		}
		console.warn(
			"[right-now] feed response shape unrecognized, falling back to cascade",
			raw,
		);
	} catch (error) {
		console.warn(
			"[right-now] dedicated feed failed, falling back to cascade",
			error,
		);
	}

	const cascade = await getRightNowFromCascade({
		geohash: options.geohash,
		hosting: options.hosting,
	});
	return {
		items: cascade.items,
		source: "cascade",
		nextPage: cascade.nextPage,
	};
}

export type ActiveRightNowPost = {
	postId: string | null;
	text: string | null;
	status: string | null;
	postedAt: number | null;
	expiresAt: number | null;
	raw: unknown;
};

export function normalizeActiveRightNowPost(
	raw: unknown,
): ActiveRightNowPost | null {
	if (raw == null) return null;
	const root = asRecord(raw) ?? {};

	// Explicit inactive markers
	if (root.active === false || root.hasActivePost === false) return null;
	if (root.post === null && root.activePost === null) return null;
	if (Object.keys(root).length === 0) return null;

	const nested =
		asRecord(root.post) ??
		asRecord(root.activePost) ??
		asRecord(root.data) ??
		root;

	const postIdRaw = pick(nested, ["postId", "post_id", "id"]);
	const status = coerceRightNowStatus(
		pick(nested, ["status", "rightNow", "rightNowStatus", "hostingStatus"]) ??
			pick(root, ["status", "rightNow"]),
	);
	if (status === "NOT_ACTIVE" && postIdRaw == null) return null;

	const textRaw = pick(nested, [
		"text",
		"rightNowText",
		"statusText",
		"body",
		"caption",
	]);
	const postedAtRaw = pick(nested, [
		"postedAt",
		"rightNowPosted",
		"createdAt",
		"created_at",
		"timestamp",
	]);

	const postId =
		postIdRaw == null || postIdRaw === "" ? null : String(postIdRaw);
	const text = textRaw == null ? null : String(textRaw);
	const postedAt =
		typeof postedAtRaw === "number" && Number.isFinite(postedAtRaw)
			? postedAtRaw
			: null;

	// No identifiable post → inactive
	if (postId == null && !text && !isActiveRightNowStatus(status)) {
		return null;
	}

	return {
		postId,
		text,
		status: status == null ? null : String(status),
		postedAt,
		expiresAt: postedAt != null ? postedAt + 60 * 60 * 1000 : null,
		raw,
	};
}

export async function getActiveRightNowPost(): Promise<ActiveRightNowPost | null> {
	const res = await fetchRest("/v3/rightnow/active-post");
	if (res.status === 204 || res.status === 404) return null;
	if (res.status < 200 || res.status >= 300) {
		// No active post is often 404; other errors bubble as soft null for UI
		if (res.status === 404) return null;
		const text = res.text();
		throw new Error(
			`Failed to load active Right Now post (${res.status}): ${text.slice(0, 200)}`,
		);
	}
	const raw = res.json();
	return normalizeActiveRightNowPost(raw);
}

export type CreateRightNowPostInput = {
	/** Optional status text (what you're looking for). */
	text?: string | null;
	/** Hosting toggle → HOSTING vs NOT_HOSTING. */
	hosting: boolean;
};

function buildCreateBodies(input: CreateRightNowPostInput): Record<string, unknown>[] {
	const text = (input.text ?? "").trim();
	const status = input.hosting ? "HOSTING" : "NOT_HOSTING";
	// Ordered by likelihood from known profile field names + product UX.
	// OpenAPI marks CreatePostRequest as UndocumentedObject.
	return [
		{ text, status },
		{ text, hosting: input.hosting },
		{ rightNowText: text, rightNow: status },
		{ statusText: text, isHosting: input.hosting },
		{ text, rightNowStatus: status },
		{ caption: text, status },
		// minimal: status only
		{ status },
		{ hosting: input.hosting },
	];
}

async function postJson(
	path: string,
	body: unknown,
): Promise<{ status: number; raw: unknown; text: string }> {
	const res = await fetchRest(path, { method: "POST", body });
	const text = res.text();
	let raw: unknown = null;
	try {
		raw = text ? JSON.parse(text) : null;
	} catch {
		raw = text;
	}
	return { status: res.status, raw, text };
}

/**
 * Create a Right Now post (text + hosting). Tries `/v4` then `/v3`,
 * with a few body-shape variants because CreatePostRequest is undocumented.
 */
export async function createRightNowPost(
	input: CreateRightNowPostInput,
): Promise<{ raw: unknown; path: string; body: Record<string, unknown> }> {
	const bodies = buildCreateBodies(input);
	const paths = ["/v4/rightnow/posts", "/v3/rightnow/posts"];
	let lastError: Error | null = null;

	for (const path of paths) {
		for (const body of bodies) {
			try {
				const { status, raw, text } = await postJson(path, body);
				if (status >= 200 && status < 300) {
					return { raw, path, body };
				}
				// 4xx validation → try next body shape
				if (status >= 400 && status < 500 && status !== 401 && status !== 403) {
					lastError = new Error(
						`Create post rejected (${status}) on ${path}: ${text.slice(0, 240)}`,
					);
					continue;
				}
				// 401/403/5xx — don't thrash body shapes on auth failures
				throw new Error(
					`Create post failed (${status}) on ${path}: ${text.slice(0, 240)}`,
				);
			} catch (err) {
				// Rust layer may throw before returning status for hard HTTP errors
				const message = err instanceof Error ? err.message : String(err);
				lastError = err instanceof Error ? err : new Error(message);
				// If the backend threw (e.g. non-success mapped to AppError), try next shape
				if (/40[0-9]|422|validation|bad request/i.test(message)) {
					continue;
				}
				// Network / auth → abort remaining shapes for this path, try next path once
				if (/401|403|Not logged in|auth/i.test(message)) {
					throw lastError;
				}
				continue;
			}
		}
	}

	throw lastError ?? new Error("Failed to create Right Now post");
}

/**
 * End / deactivate the current Right Now post.
 * Tries DELETE then PATCH status=NOT_ACTIVE across known path patterns.
 */
export async function endRightNowPost(
	postId: string | null | undefined,
): Promise<void> {
	const id = postId?.trim() || null;
	const attempts: Array<() => Promise<void>> = [];

	if (id) {
		for (const version of ["v4", "v3", "v1"] as const) {
			const base = `/${version}/rightnow/posts/${encodeURIComponent(id)}`;
			attempts.push(async () => {
				const res = await fetchRest(base, { method: "DELETE" });
				if (res.status >= 200 && res.status < 300) return;
				if (res.status === 404 || res.status === 405) {
					throw new Error(`DELETE not supported (${res.status})`);
				}
				throw new Error(`DELETE failed (${res.status}): ${res.text().slice(0, 200)}`);
			});
			attempts.push(async () => {
				const res = await fetchRest(base, {
					method: "PATCH",
					body: { status: "NOT_ACTIVE" },
				});
				if (res.status >= 200 && res.status < 300) return;
				throw new Error(`PATCH status failed (${res.status}): ${res.text().slice(0, 200)}`);
			});
			attempts.push(async () => {
				const res = await fetchRest(base, {
					method: "PATCH",
					body: { active: false },
				});
				if (res.status >= 200 && res.status < 300) return;
				throw new Error(`PATCH active failed (${res.status}): ${res.text().slice(0, 200)}`);
			});
		}
		attempts.push(async () => {
			const res = await fetchRest(
				`/v1/rightnow/posts/${encodeURIComponent(id)}/settings`,
				{ method: "PATCH", body: { status: "NOT_ACTIVE" } },
			);
			if (res.status >= 200 && res.status < 300) return;
			throw new Error(`settings PATCH failed (${res.status}): ${res.text().slice(0, 200)}`);
		});
	}

	// Session-level end without known id
	attempts.push(async () => {
		const res = await fetchRest("/v3/rightnow/active-post", {
			method: "DELETE",
		});
		if (res.status >= 200 && res.status < 300) return;
		throw new Error(`active-post DELETE failed (${res.status})`);
	});
	attempts.push(async () => {
		const res = await fetchRest("/v3/rightnow/posts", {
			method: "POST",
			body: { status: "NOT_ACTIVE" },
		});
		if (res.status >= 200 && res.status < 300) return;
		throw new Error(`end via create NOT_ACTIVE failed (${res.status})`);
	});

	let lastError: Error | null = null;
	for (const attempt of attempts) {
		try {
			await attempt();
			return;
		} catch (err) {
			lastError = err instanceof Error ? err : new Error(String(err));
		}
	}
	throw lastError ?? new Error("Failed to end Right Now post");
}

export { rightNowStatusSchema };
