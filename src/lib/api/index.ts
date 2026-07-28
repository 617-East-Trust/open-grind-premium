import { decode, encode } from "@msgpack/msgpack";
import { invoke } from "@tauri-apps/api/core";
import { goto } from "$app/navigation";
import { toast } from "svelte-sonner";
import z from "zod";

import { ApiError } from "$lib/api/api-error";
import { requestBlockedAlertState } from "$lib/api/request-blocked/request-blocked-state.svelte";
import { fromBase64, toBase64 } from "$lib/base64";

export const methods = {
	login: {
		request: z.object({
			email: z.email(),
			password: z.string().min(1),
		}),
		response: z.object({
			profileId: z.string(),
		}),
	},
	auth_state: {
		request: z.undefined(),
		response: z.string().nullable(),
	},
	rotate_api_params: {
		request: z.undefined(),
		response: z.object({
			"user-agent": z.string(),
			"l-device-info": z.string(),
		}),
	},
	device_fingerprint_hash: {
		request: z.undefined(),
		response: z.string(),
	},
	grindr_app_version: {
		request: z.undefined(),
		response: z.object({
			app_version: z.string(),
			build_number: z.string(),
			fetched_at: z.number(),
		}),
	},
	ws_status: {
		request: z.undefined(),
		response: z.object({
			connected: z.boolean(),
			foreground: z.boolean(),
			buffered: z.number(),
		}),
	},
	logout: {
		request: z.undefined(),
		response: z.undefined(),
	},
	add_account: {
		request: z.object({
			email: z.email(),
			password: z.string().min(1),
		}),
		response: z.object({
			profileId: z.string(),
		}),
	},
	switch_account: {
		request: z.object({
			profileId: z.string(),
		}),
		response: z.object({
			profileId: z.string(),
		}),
	},
	remove_account: {
		request: z.object({
			profileId: z.string(),
		}),
		response: z.object({
			removed: z.boolean(),
			wasActive: z.boolean(),
			accounts: z.array(
				z.object({
					profileId: z.string(),
					email: z.string(),
					isActive: z.boolean(),
				}),
			),
		}),
	},
	list_accounts: {
		request: z.undefined(),
		response: z.array(
			z.object({
				profileId: z.string(),
				email: z.string(),
				isActive: z.boolean(),
			}),
		),
	},
} satisfies Record<string, { request: z.ZodType; response: z.ZodType }>;

export async function callMethod<T extends keyof typeof methods>(
	method: T,
	...args: z.infer<(typeof methods)[T]["request"]> extends undefined
		? []
		: [data: z.infer<(typeof methods)[T]["request"]>]
): Promise<z.infer<(typeof methods)[T]["response"]>> {
	const raw = await invoke(method, args[0]);
	const schema = methods[method].response;
	const parsed = schema.safeParse(raw);
	if (!parsed.success) {
		console.error(`[api] response validation failed for "${String(method)}"`, {
			issues: parsed.error.issues,
			raw,
		});
		throw new ApiError({
			message: `Response validation failed for "${String(method)}"`,
			request: { method: String(method), path: String(method) },
			response: null,
			cause: parsed.error,
		});
	}
	return parsed.data as z.infer<(typeof methods)[T]["response"]>;
}

/**
 * Detect Cloudflare / WAF HTML block (challenge) pages.
 *
 * Uses a *normalized* matcher (audit §3.5): the body is lowercased, HTML
 * comments are stripped, common entities decoded, and whitespace collapsed
 * before signal matching. This makes detection robust to cosmetic changes in
 * Cloudflare's challenge markup (title strings, injected whitespace, attribute
 * placement) that brittle exact-string matching historically missed.
 */
export function isCloudflareBlock(status: number, text: string): boolean {
	if (!text) return false;

	// JSON / non-HTML API errors are never Cloudflare block pages.
	const trimmed = text.trimStart();
	if (trimmed.startsWith("{") || trimmed.startsWith("[")) return false;

	const normalized = normalizeForMatching(text);

	// Require an HTML document so we never mistake a bare error string.
	const looksLikeHtml =
		normalized.includes("<html") ||
		normalized.includes("<!doctype html>") ||
		normalized.includes("<title");
	if (!looksLikeHtml) return false;

	const cloudflareSignals = [
		"cloudflare",
		"cf-ray",
		"cf-mitigated",
		"cf-browser-verification",
		"challenge-platform",
		"attention required",
		"checking your browser",
		"just a moment",
		"managed challenge",
		"you have been blocked",
		"sorry, you have been blocked",
	];
	const hasSignal = cloudflareSignals.some((s) => normalized.includes(s));
	if (!hasSignal) return false;

	// 403 / 429 / 503 / 520 are the statuses Cloudflare returns for
	// blocks and managed challenges; a strong block phrase alone also counts.
	const blockishStatus = [403, 429, 503, 520].includes(status);
	return blockishStatus || normalized.includes("you have been blocked");
}

/** Lowercase, strip comments, decode common HTML entities, collapse whitespace. */
function normalizeForMatching(html: string): string {
	let s = html.toLowerCase();
	s = s.replace(/<!--[\s\S]*?-->/g, " ");
	s = s
		.replace(/&nbsp;/g, " ")
		.replace(/&/g, "&")
		.replace(/</g, "<")
		.replace(/>/g, ">")
		.replace(/&#39;/g, "'")
		.replace(/"/g, '"')
		.replace(/&#(\d+);/g, (_m, d) => String.fromCharCode(Number(d)));
	s = s.replace(/\s+/g, " ");
	return s;
}

export function asAppError(error: unknown) {
	const { data, success } = z
		.object({
			kind: z.enum(["Http", "Auth", "Api", "NotInitialized"]),
			message: z
				.string()
				.or(
					z.object({
						code: z.number(),
						message: z.string(),
					}),
				)
				.optional(),
		})
		.safeParse(error);
	if (success) {
		let prettyMessage: string;
		if (data.kind === "NotInitialized") {
			prettyMessage =
				"Network client failed to initialize. Please restart the app.";
		} else if (typeof data.message === "string") {
			prettyMessage = data.message;
		} else if (data.message) {
			prettyMessage = `Error ${data.message.code}: ${data.message.message}`;
		} else {
			prettyMessage = "An unknown error occurred";
		}
		return { ...data, prettyMessage };
	}
}

export async function fetchRest(
	path: string,
	options: {
		method?: string;
		body?: unknown;
		abortController?: AbortController;
	} = { method: "GET" },
) {
	const requestInfo = {
		method: options.method ?? "GET",
		path,
		body: options.body,
	};
	try {
		const payload = encode({
			method: options.method || "GET",
			path,
			body: options.body === undefined ? null : encode(options.body),
		});
		const packed = await invoke("request", {
			// https://github.com/tauri-apps/tauri/issues/10573
			payload: toBase64(payload),
		}).then((res) => {
			if (typeof res === "string") {
				// https://github.com/tauri-apps/tauri/issues/10573
				return fromBase64(res);
			} else {
				throw new Error("Invalid response from backend");
			}
		});
		if (options.abortController?.signal.aborted) {
			throw new Error("Request aborted");
		}
		const decoded = decode(packed);
		const { status, body: responseBody } = z
			.object({ status: z.number(), body: z.instanceof(Uint8Array) })
			.parse(decoded);
		return {
			status,
			bytes() {
				return responseBody;
			},
			text() {
				return new TextDecoder().decode(responseBody);
			},
			json() {
				const text = new TextDecoder().decode(responseBody);
				const responseInfo = { status, body: text };
				// Broader Cloudflare / WAF block heuristics — title strings change;
				// also catch HTML challenge pages on non-403 statuses.
				if (isCloudflareBlock(status, text)) {
					if (!requestBlockedAlertState.disable) {
						requestBlockedAlertState.open = true;
					}
					throw new ApiError({
						message: "Request blocked",
						request: requestInfo,
						response: responseInfo,
					});
				}
				try {
					return JSON.parse(text);
				} catch (error) {
					console.error("Failed to parse JSON response", {
						path,
						text,
					});
					throw new ApiError({
						message: "Failed to parse API response",
						request: requestInfo,
						response: responseInfo,
						cause: error,
					});
				}
			},
			jsonParsed<TSchema extends z.ZodType>(schema: TSchema) {
				const data = this.json();
				const bodyText = new TextDecoder().decode(responseBody);
				try {
					return parseApiResponse({
						schema,
						data,
						path,
						method: options.method || "GET",
					});
				} catch (error) {
					if (error instanceof ApiError) throw error;
					throw new ApiError({
						message:
							error instanceof Error
								? error.message
								: "API response validation failed",
						request: requestInfo,
						response: { status, body: bodyText },
						cause: error,
					});
				}
			},
			debugJsonParsed<TSchema extends z.ZodType>(schema: TSchema) {
				console.log(this.json());
				return this.jsonParsed(schema);
			},
		};
	} catch (error) {
		if (error instanceof ApiError) throw error;
		const appError = asAppError(error);
		if (appError) {
			if (appError.kind === "Auth" && appError.message === "Not logged in") {
				toast("Please log in to continue");
				goto("/auth/sign-in").catch((error) => console.error(error));
			}
		}
		throw new ApiError({
			message:
				appError?.prettyMessage ??
				(error instanceof Error ? error.message : String(error)),
			request: requestInfo,
			response: null,
			cause: error,
		});
	}
}

export function parseApiResponse<TSchema extends z.ZodType>(options: {
	schema: TSchema;
	data: unknown;
	path: string;
	method?: string;
}): z.infer<TSchema> {
	const parsed = options.schema.safeParse(options.data);
	if (parsed.success) {
		return parsed.data;
	}

	console.error("API response schema validation failed", {
		path: options.path,
		method: options.method ?? "GET",
		schema: options.schema.meta()?.title,
		issues: parsed.error.issues,
		response: options.data,
	});

	throw parsed.error;
}

export { ApiError };
