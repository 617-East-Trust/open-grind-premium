import type { Context } from "./context";
import { SHARED_PAGE } from "./context";
import { renderOperation } from "./operations";
import { renderProperty } from "./properties";
import {
	pageTitle,
	schemaDisplay,
	tagTitle,
	urlForSchema,
	withWipSuffix,
} from "./slugs";
import type { Schema } from "./types";

function refName(ref: string): string {
	return ref.replace("#/components/schemas/", "");
}

function renderSchemaSection(ctx: Context, name: string): string {
	const schema = ctx.doc.components.schemas[name];
	if (!schema) return "";
	const wip = schema["x-wip"] === true;
	const display = schemaDisplay(ctx, name);
	const lines: string[] = [`## ${display}`, ""];

	if (wip) lines.push("> [!NOTE] This type hasn't been researched yet", "");
	if (schema.description) lines.push(schema.description, "");

	if (schema["x-enum-labels"]) {
		for (const [val, label] of Object.entries(schema["x-enum-labels"])) {
			lines.push(`- \`${val}\` — ${label}`);
		}
		lines.push("");
		return lines.join("\n").trimEnd() + "\n";
	}

	if (schema.enum) {
		for (const val of schema.enum) lines.push(`- \`${JSON.stringify(val)}\``);
		lines.push("");
		return lines.join("\n").trimEnd() + "\n";
	}

	if (Array.isArray(schema.allOf)) {
		for (const piece of schema.allOf) {
			if (piece.$ref) {
				const ref = refName(piece.$ref);
				lines.push(
					`- *everything from [${schemaDisplay(ctx, ref)}](${urlForSchema(ctx, ref)})*`,
				);
			}
		}
		const inline = schema.allOf.find((p): p is Schema => p.type === "object");
		const overrides = schema.properties ?? {};
		const merged: Record<string, Schema> = { ...(inline?.properties ?? {}) };
		for (const [k, v] of Object.entries(overrides)) {
			merged[k] = { ...(merged[k] ?? {}), ...v };
		}
		const reqSet = new Set([
			...(inline?.required ?? []),
			...(schema.required ?? []),
		]);
		for (const [k, v] of Object.entries(merged)) {
			lines.push(renderProperty(ctx, k, v, 0, reqSet.has(k)));
		}
		lines.push("");
		return lines.join("\n").trimEnd() + "\n";
	}

	if (schema.properties) {
		const reqList = schema.required ?? [];
		for (const [k, v] of Object.entries(schema.properties)) {
			lines.push(renderProperty(ctx, k, v, 0, reqList.includes(k)));
		}
		lines.push("");
		return lines.join("\n").trimEnd() + "\n";
	}

	const variants = schema.oneOf ?? schema.anyOf;
	if (variants) {
		for (const v of variants) {
			if (v.$ref) {
				const ref = refName(v.$ref);
				lines.push(`- [${schemaDisplay(ctx, ref)}](${urlForSchema(ctx, ref)})`);
			}
		}
		lines.push("");
	}

	return lines.join("\n").trimEnd() + "\n";
}

export function renderTagPage(ctx: Context, tagName: string): string {
	const tagObj = ctx.doc.tags.find((t) => t.name === tagName);
	const wip = tagObj?.["x-wip"] === true;
	const lines: string[] = [`# ${withWipSuffix(tagTitle(tagName), wip)}`, ""];

	if (wip) {
		lines.push(
			"> [!NOTE] This page is a work in progress. Endpoints below haven't been fully researched.",
			"",
		);
	}
	if (tagObj?.description) lines.push(tagObj.description, "");

	for (const entry of ctx.operationsByTag.get(tagName) ?? []) {
		lines.push(renderOperation(ctx, entry, wip));
	}
	for (const name of ctx.schemasByPage.get(tagName) ?? []) {
		lines.push(renderSchemaSection(ctx, name));
	}
	return (
		lines
			.join("\n")
			.replace(/\n{3,}/g, "\n\n")
			.trimEnd() + "\n"
	);
}

export function renderSharedPage(ctx: Context, pageName: string): string {
	const lines: string[] = [`# ${pageTitle(pageName)}`, ""];
	const lead =
		pageName === SHARED_PAGE
			? "Types shared across multiple top-level sections."
			: `Types shared by more than one ${pageName.split("/")[0]} endpoint page.`;
	lines.push(lead, "");
	const schemas = [...(ctx.schemasByPage.get(pageName) ?? [])].sort((a, b) =>
		schemaDisplay(ctx, a).localeCompare(schemaDisplay(ctx, b)),
	);
	for (const name of schemas) lines.push(renderSchemaSection(ctx, name));
	return (
		lines
			.join("\n")
			.replace(/\n{3,}/g, "\n\n")
			.trimEnd() + "\n"
	);
}
