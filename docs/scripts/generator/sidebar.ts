import type { Context } from "./context";
import { SKIP_TAGS } from "./context";
import { tagTitle, tagUrl, withWipSuffix } from "./slugs";
import type { SidebarEntry, SidebarGroup } from "./types";

function pascalCase(s: string): string {
	return s
		.split(/[-/_]/)
		.map((p) => p.charAt(0).toUpperCase() + p.slice(1))
		.join("");
}

function sidebarEntry(
	ctx: Context,
	tagName: string,
): { text: string; link: string } {
	const tagObj = ctx.doc.tags.find((t) => t.name === tagName);
	const wip = tagObj?.["x-wip"] === true;
	return { text: withWipSuffix(tagTitle(tagName), wip), link: tagUrl(tagName) };
}

export function renderSidebar(ctx: Context): string {
	const order: SidebarEntry[] = ctx.doc["x-sidebar-order"] ?? [];
	const groups = order.filter(
		(x): x is SidebarGroup => typeof x === "object" && "group" in x,
	);
	const out: string[] = ['import type { DefaultTheme } from "vitepress";', ""];

	for (const group of groups) {
		const varName = `grindrApiReference${pascalCase(group.group)}`;
		out.push(`export const ${varName}: DefaultTheme.SidebarItem[] = [`);
		for (const tag of group.items) {
			if (SKIP_TAGS.has(tag)) continue;
			const { text, link } = sidebarEntry(ctx, tag);
			out.push(
				`\t{ text: ${JSON.stringify(text)}, link: ${JSON.stringify(link)} },`,
			);
		}
		const sectionShared = `${group.group}/shared-types`;
		if ((ctx.schemasByPage.get(sectionShared) || []).length > 0) {
			out.push(
				`\t{ text: "Shared types", link: ${JSON.stringify(`/grindr-api/${sectionShared}`)} },`,
			);
		}
		out.push("];", "");
	}

	out.push(
		`export const grindrApiReferenceWebSocket: DefaultTheme.SidebarItem[] = [`,
		`\t{ text: "Events", link: "/grindr-api/websocket/events" },`,
		`\t{ text: "Notification Event", link: "/grindr-api/websocket/notification-event" },`,
		`\t{ text: "Commands", link: "/grindr-api/websocket/commands" },`,
		"];",
		"",
	);

	out.push(
		`export const grindrApiReference: DefaultTheme.SidebarItem[] = [`,
		`\t{ text: "Getting started", link: "/grindr-api/getting-started" },`,
		`\t{ text: "Security headers", link: "/grindr-api/security-headers" },`,
		`\t{ text: "API Authorization", link: "/grindr-api/api-authorization" },`,
	);

	for (const entry of order) {
		if (typeof entry === "string") {
			if (SKIP_TAGS.has(entry)) continue;
			const { text, link } = sidebarEntry(ctx, entry);
			out.push(
				`\t{ text: ${JSON.stringify(text)}, link: ${JSON.stringify(link)} },`,
			);
		} else if (typeof entry === "object" && "group" in entry) {
			const varName = `grindrApiReference${pascalCase(entry.group)}`;
			const tag = ctx.doc.tags.find((t) => t.name === entry.group);
			const wip = tag?.["x-wip"] === true;
			const text = withWipSuffix(tagTitle(entry.group), wip);
			out.push(
				`\t{ text: ${JSON.stringify(text)}, link: ${JSON.stringify(`/grindr-api/${entry.group}/`)}, collapsed: true, items: ${varName} },`,
			);
		}
	}

	out.push(
		`\t{ text: "Rate limits", link: "/grindr-api/rate-limits" },`,
		`\t{ text: "WebSocket", link: "/grindr-api/websocket/", collapsed: true, items: grindrApiReferenceWebSocket },`,
		`\t{ text: "Appendix", link: "/grindr-api/appendix" },`,
		`\t{ text: "Shared types", link: "/grindr-api/shared-types" },`,
		"];",
		"",
	);

	return out.join("\n");
}
