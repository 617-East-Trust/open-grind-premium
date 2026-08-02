import z from "zod";

/** Active Right Now status on a profile / post. */
export const rightNowStatusSchema = z.enum([
	"NOT_ACTIVE",
	"HOSTING",
	"NOT_HOSTING",
]);

export type RightNowStatus = z.infer<typeof rightNowStatusSchema>;

/** Right Now posts expire after one hour (product rule). */
export const RIGHT_NOW_TTL_MS = 60 * 60 * 1000;

export function coerceRightNowStatus(
	value: unknown,
): RightNowStatus | string | null {
	if (value == null) return null;
	if (typeof value === "boolean") {
		return value ? "HOSTING" : "NOT_HOSTING";
	}
	const s = String(value).toUpperCase().replace(/[\s-]+/g, "_");
	if (s === "HOSTING" || s === "NOT_HOSTING" || s === "NOT_ACTIVE") {
		return s as RightNowStatus;
	}
	if (s === "HOST" || s === "TRUE" || s === "1") return "HOSTING";
	if (s === "NOT_HOST" || s === "FALSE" || s === "0") return "NOT_HOSTING";
	return String(value);
}

export function isActiveRightNowStatus(
	status: RightNowStatus | string | null | undefined,
): boolean {
	if (!status) return false;
	const s = String(status).toUpperCase();
	return s === "HOSTING" || s === "NOT_HOSTING";
}

/**
 * Normalized feed item for the Right Now tab.
 * Source may be `/v5/rightnow/feed` or cascade+profiles enrichment.
 */
export const rightNowFeedItemSchema = z.object({
	profileId: z.coerce.number().int().nonnegative(),
	displayName: z.string().nullable().optional().default(null),
	status: z
		.union([rightNowStatusSchema, z.string()])
		.nullable()
		.optional()
		.default(null),
	text: z.string().nullable().optional().default(null),
	postedAt: z.number().nullable().optional().default(null),
	distance: z.number().nullable().optional().default(null),
	thumbnailUrl: z.string().nullable().optional().default(null),
	fullImageUrl: z.string().nullable().optional().default(null),
	profileImageMediaHash: z.string().nullable().optional().default(null),
	age: z.number().int().nullable().optional().default(null),
	postId: z.union([z.string(), z.number()]).nullable().optional().default(null),
	source: z.enum(["feed", "cascade"]).default("cascade"),
});

export type RightNowFeedItem = z.infer<typeof rightNowFeedItemSchema>;

export function rightNowExpiresAt(postedAt: number | null | undefined): number | null {
	if (postedAt == null || !Number.isFinite(postedAt)) return null;
	return postedAt + RIGHT_NOW_TTL_MS;
}

export function rightNowRemainingMs(
	postedAt: number | null | undefined,
	nowMs: number,
): number | null {
	const expires = rightNowExpiresAt(postedAt);
	if (expires == null) return null;
	return Math.max(0, expires - nowMs);
}

export function formatRightNowRemaining(ms: number | null): string | null {
	if (ms == null) return null;
	if (ms <= 0) return "Expired";
	const totalSec = Math.floor(ms / 1000);
	const min = Math.floor(totalSec / 60);
	const sec = totalSec % 60;
	if (min >= 60) {
		const hours = Math.floor(min / 60);
		const remMin = min % 60;
		return `${hours}h ${remMin}m left`;
	}
	if (min > 0) return `${min}m ${sec.toString().padStart(2, "0")}s left`;
	return `${sec}s left`;
}

export function formatDistanceMeters(metres: number | null | undefined): string | null {
	if (metres == null || !Number.isFinite(metres)) return null;
	const METRES_PER_MILE = 1609.344;
	const miles = metres / METRES_PER_MILE;
	if (miles < 0.1) return `${Math.round(metres * 3.28084)} ft`;
	if (miles < 10) return `${miles.toFixed(1)} mi`;
	return `${Math.round(miles)} mi`;
}

export function statusLabel(status: RightNowStatus | string | null | undefined): string {
	const s = String(status ?? "").toUpperCase();
	if (s === "HOSTING") return "Hosting";
	if (s === "NOT_HOSTING") return "Not hosting";
	if (s === "NOT_ACTIVE") return "Inactive";
	return status ? String(status) : "Right Now";
}
