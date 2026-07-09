import { describe, expect, it, vi } from "vitest";
import z from "zod";

import { asAppError, isCloudflareBlock, parseApiResponse } from "$lib/api";

describe("isCloudflareBlock", () => {
	it("detects classic Cloudflare 403 HTML block page", () => {
		const html = `<!DOCTYPE html><html><title>Attention Required! | Cloudflare</title>
			<body>Sorry, you have been blocked</body></html>`;
		expect(isCloudflareBlock(403, html)).toBe(true);
	});

	it("detects challenge / just a moment pages on 503", () => {
		const html = `<html><title>Just a moment...</title><div>cloudflare challenge-platform</div></html>`;
		expect(isCloudflareBlock(503, html)).toBe(true);
	});

	it("does not flag valid JSON API errors", () => {
		expect(isCloudflareBlock(403, `{"code":40301,"message":"forbidden"}`)).toBe(
			false,
		);
	});

	it("does not flag unrelated HTML", () => {
		expect(isCloudflareBlock(200, `<html><title>Grindr</title></html>`)).toBe(
			false,
		);
	});
});

describe("asAppError", () => {
	it("formats string messages from structured app errors", () => {
		expect(asAppError({ kind: "Auth", message: "Not logged in" })).toEqual({
			kind: "Auth",
			message: "Not logged in",
			prettyMessage: "Not logged in",
		});
	});

	it("formats API error code objects from structured app errors", () => {
		expect(
			asAppError({
				kind: "Api",
				message: { code: 429, message: "Rate limited" },
			}),
		).toEqual({
			kind: "Api",
			message: { code: 429, message: "Rate limited" },
			prettyMessage: "Error 429: Rate limited",
		});
	});

	it("ignores unknown errors", () => {
		expect(asAppError(new Error("network failed"))).toBeUndefined();
	});
});

describe("parseApiResponse", () => {
	it("returns schema-parsed response data", () => {
		const parsed = parseApiResponse({
			path: "/v8/sessions",
			method: "POST",
			schema: z.object({
				profileId: z.coerce.number().int().nonnegative(),
			}),
			data: { profileId: "123" },
		});

		expect(parsed).toEqual({ profileId: 123 });
	});

	it("logs endpoint context before throwing validation errors", () => {
		const consoleError = vi
			.spyOn(console, "error")
			.mockImplementation(() => {});

		expect(() =>
			parseApiResponse({
				path: "/v5/chat/conversation/abc/message",
				method: "GET",
				schema: z.object({
					messages: z.array(z.object({ messageId: z.string() })),
				}),
				data: { messages: [{ messageId: 123 }] },
			}),
		).toThrow(z.ZodError);

		expect(consoleError).toHaveBeenCalledWith(
			"API response schema validation failed",
			expect.objectContaining({
				path: "/v5/chat/conversation/abc/message",
				method: "GET",
				response: { messages: [{ messageId: 123 }] },
			}),
		);

		consoleError.mockRestore();
	});
});
