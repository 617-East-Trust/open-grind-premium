import { toast } from "svelte-sonner";

import {
	createRightNowPost,
	endRightNowPost,
	getActiveRightNowPost,
	loadRightNowFeed,
	normalizeActiveRightNowPost,
	type ActiveRightNowPost,
} from "$lib/api/right-now";
import { getPreferences } from "$lib/app-data/preferences.svelte";
import type { RightNowFeedItem } from "$lib/model/right-now";

export type HostingFilter = "all" | "hosting" | "not_hosting";

class RightNowState {
	items = $state<RightNowFeedItem[]>([]);
	loading = $state(false);
	error = $state<Error | null>(null);
	source = $state<"feed" | "cascade" | null>(null);
	hostingFilter = $state<HostingFilter>("all");
	geohash = $state<string | null>(null);
	loadedOnce = $state(false);

	activePost = $state<ActiveRightNowPost | null>(null);
	activeLoading = $state(false);
	posting = $state(false);
	ending = $state(false);
	composeOpen = $state(false);

	get errorMessage(): string | null {
		return this.error?.message ?? null;
	}

	get hasActivePost(): boolean {
		return this.activePost != null;
	}

	async ensureGeohash(): Promise<string | null> {
		const prefs = await getPreferences();
		this.geohash = prefs.geohash;
		return prefs.geohash;
	}

	async loadActive(): Promise<void> {
		this.activeLoading = true;
		try {
			this.activePost = await getActiveRightNowPost();
		} catch (err) {
			// Soft-fail: feed still works without active-post probe
			console.warn("[right-now] active post load failed", err);
			this.activePost = null;
		} finally {
			this.activeLoading = false;
		}
	}

	async load(options: { force?: boolean } = {}): Promise<void> {
		if (this.loading) return;
		if (this.loadedOnce && !options.force) return;

		this.loading = true;
		this.error = null;
		try {
			const geohash = await this.ensureGeohash();
			// Active post does not require geohash
			void this.loadActive();

			if (!geohash) {
				this.items = [];
				this.source = null;
				this.loadedOnce = true;
				return;
			}

			const hosting =
				this.hostingFilter === "hosting"
					? true
					: this.hostingFilter === "not_hosting"
						? false
						: null;

			const result = await loadRightNowFeed({ geohash, hosting });
			this.items = result.items;
			this.source = result.source;
			this.loadedOnce = true;
		} catch (err) {
			console.error(err);
			this.error =
				err instanceof Error
					? err
					: new Error("Failed to load Right Now", { cause: err });
			toast.error("Failed to load Right Now");
		} finally {
			this.loading = false;
		}
	}

	async refresh(): Promise<void> {
		this.loadedOnce = false;
		await Promise.all([this.load({ force: true }), this.loadActive()]);
	}

	async setHostingFilter(filter: HostingFilter): Promise<void> {
		if (this.hostingFilter === filter) return;
		this.hostingFilter = filter;
		this.loadedOnce = false;
		await this.load({ force: true });
	}

	async createPost(input: {
		text: string;
		hosting: boolean;
	}): Promise<boolean> {
		if (this.posting) return false;
		this.posting = true;
		try {
			const result = await createRightNowPost({
				text: input.text,
				hosting: input.hosting,
			});
			const now = Date.now();
			this.activePost = normalizeActiveRightNowPost(result.raw) ?? {
				postId: null,
				text: input.text || null,
				status: input.hosting ? "HOSTING" : "NOT_HOSTING",
				postedAt: now,
				expiresAt: now + 60 * 60 * 1000,
				raw: result.raw,
			};
			toast.success("Right Now posted · live for 1 hour");
			this.composeOpen = false;
			// Refresh feed in background; keep optimistic active banner
			void this.load({ force: true });
			void this.loadActive();
			return true;
		} catch (err) {
			console.error(err);
			const message =
				err instanceof Error ? err.message : "Failed to create post";
			toast.error(message.slice(0, 160));
			return false;
		} finally {
			this.posting = false;
		}
	}

	async endPost(): Promise<boolean> {
		if (this.ending) return false;
		this.ending = true;
		try {
			await endRightNowPost(this.activePost?.postId);
			toast.success("Right Now ended");
			this.activePost = null;
			await this.refresh();
			return true;
		} catch (err) {
			console.error(err);
			const message =
				err instanceof Error ? err.message : "Failed to end post";
			toast.error(message.slice(0, 160));
			return false;
		} finally {
			this.ending = false;
		}
	}
}

export const rightNowState = new RightNowState();
