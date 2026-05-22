import { readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

const GEN = "content/grindr-api-generated";
const ALL = "content/grindr-api";

function walk(dir: string): string[] {
	const out: string[] = [];
	for (const e of readdirSync(dir)) {
		const full = join(dir, e);
		if (statSync(full).isDirectory()) out.push(...walk(full));
		else if (e.endsWith(".md")) out.push(full);
	}
	return out;
}

function pageOf(filePath: string): string {
	return filePath.replace(/^content\//, "/").replace(/\.md$/, "");
}

function slugify(text: string): string {
	return text
		.toLowerCase()
		.replace(/[^\w\s-]/g, "")
		.replace(/\s+/g, "-")
		.replace(/-+/g, "-")
		.replace(/^-|-$/g, "");
}

function collectAnchors(filePath: string): Set<string> {
	const content = readFileSync(filePath, "utf8");
	const anchors = new Set<string>([""]);
	for (const m of content.matchAll(/^#{1,6}\s+(.+?)\s*$/gm)) {
		anchors.add(slugify(m[1] ?? ""));
	}
	return anchors;
}

const handPages = new Map<string, Set<string>>();
for (const f of walk(ALL).filter(
	(f) => !f.includes("/grindr-api-generated/"),
)) {
	handPages.set(pageOf(f), collectAnchors(f));
}

const genPages = new Map<string, Set<string>>();
for (const f of walk(GEN)) {
	genPages.set(
		pageOf(f).replace("/grindr-api-generated/", "/grindr-api/"),
		collectAnchors(f),
	);
}

const broken: Array<{
	file: string;
	link: string;
	label: string;
	reason: string;
}> = [];
const linkRe = /\[([^\]]*)\]\((\/grindr-api\/[^)#\s]*)(#[^)\s]+)?\)/g;

for (const f of walk(GEN)) {
	const content = readFileSync(f, "utf8");
	const fromPage = pageOf(f).replace("/grindr-api-generated/", "/grindr-api/");
	for (const m of content.matchAll(linkRe)) {
		const [, label = "", path = "", hash] = m;
		const target = path;
		const anchor = hash ? hash.slice(1) : "";
		const anchors = genPages.get(target) || handPages.get(target);
		if (!anchors) {
			broken.push({
				file: fromPage,
				link: `${path}${hash || ""}`,
				label,
				reason: "missing page",
			});
			continue;
		}
		if (anchor && !anchors.has(anchor)) {
			broken.push({
				file: fromPage,
				link: `${path}${hash || ""}`,
				label,
				reason: "missing anchor",
			});
		}
	}
}

const grouped = new Map<
	string,
	{ reason: string; label: string; refs: string[] }
>();
for (const b of broken) {
	const key = b.link;
	if (!grouped.has(key))
		grouped.set(key, { reason: b.reason, label: b.label, refs: [] });
	grouped.get(key)!.refs.push(b.file);
}

console.log(`Total broken link occurrences: ${broken.length}`);
console.log(`Unique broken targets: ${grouped.size}\n`);
const sorted = [...grouped.entries()].sort(
	(a, b) => b[1].refs.length - a[1].refs.length,
);
for (const [link, info] of sorted) {
	const sample =
		info.refs.slice(0, 3).join(", ") +
		(info.refs.length > 3 ? ` +${info.refs.length - 3}` : "");
	console.log(
		`  ${info.reason}: ${link}  (${info.refs.length}x) "${info.label}"  [${sample}]`,
	);
}
