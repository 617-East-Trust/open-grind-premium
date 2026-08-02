import {
	defaultFilters,
	type GridSearchFilters,
} from "$lib/components/filters/filters";

export type ParsedFilter = {
	key: string;
	valueText: string;
	valid: boolean;
	error?: string;
};

export type ParsedFilterGridQuery = {
	filters: GridSearchFilters;
	parsed: ParsedFilter[];
	validCount: number;
	invalidCount: number;
};

function parseBool(raw: string): boolean | null {
	const v = raw.trim().toLowerCase();
	if (v === "true" || v === "1" || v === "yes") return true;
	if (v === "false" || v === "0" || v === "no") return false;
	return null;
}

/**
 * Compact Grindr-style filter query parser for premium's filter schema.
 * Examples: `online=true&rightnow=true&age=21-35&fresh=1&favorites=true`
 */
export function parseFilterGridQuery(query: string): ParsedFilterGridQuery {
	const draft = structuredClone(defaultFilters);
	const raw = query.startsWith("?") ? query.slice(1) : query;
	const parsed: ParsedFilter[] = [];

	for (const [key, value] of new URLSearchParams(raw)) {
		const k = key.trim().toLowerCase();
		if (k === "online" || k === "isonline") {
			const b = parseBool(value);
			if (b === null) {
				parsed.push({
					key: "online",
					valueText: value,
					valid: false,
					error: "Expected true/false",
				});
			} else {
				draft.isOnline = b;
				parsed.push({
					key: "online",
					valueText: b ? "Yes" : "No",
					valid: true,
				});
			}
		} else if (k === "favorites" || k === "isfavorite" || k === "favs") {
			const b = parseBool(value);
			if (b === null) {
				parsed.push({
					key: "favorites",
					valueText: value,
					valid: false,
					error: "Expected true/false",
				});
			} else {
				draft.isFavorite = b;
				parsed.push({
					key: "favorites",
					valueText: b ? "Yes" : "No",
					valid: true,
				});
			}
		} else if (k === "rightnow" || k === "isrightnow" || k === "rn") {
			const b = parseBool(value);
			if (b === null) {
				parsed.push({
					key: "right now",
					valueText: value,
					valid: false,
					error: "Expected true/false",
				});
			} else {
				draft.isRightNow = b;
				parsed.push({
					key: "right now",
					valueText: b ? "Yes" : "No",
					valid: true,
				});
			}
		} else if (k === "fresh" || k === "isfresh") {
			const b = parseBool(value);
			if (b === null) {
				parsed.push({
					key: "fresh",
					valueText: value,
					valid: false,
					error: "Expected true/false",
				});
			} else {
				draft.isFresh = b;
				parsed.push({
					key: "fresh",
					valueText: b ? "Yes" : "No",
					valid: true,
				});
			}
		} else if (k === "age") {
			const m = value.trim().match(/^(\d{1,3})\s*-\s*(\d{1,3})$/);
			if (!m) {
				parsed.push({
					key: "age",
					valueText: value,
					valid: false,
					error: "Use age=min-max",
				});
			} else {
				const min = Number(m[1]);
				const max = Number(m[2]);
				if (min < 18 || max > 102 || min > max) {
					parsed.push({
						key: "age",
						valueText: value,
						valid: false,
						error: "Age must be 18–102",
					});
				} else {
					draft.ageEnabled = true;
					draft.age = [min, max];
					parsed.push({
						key: "age",
						valueText: `${min}–${max}`,
						valid: true,
					});
				}
			}
		} else if (k === "agemin" || k === "age_min") {
			const n = Number(value);
			if (!Number.isFinite(n) || n < 18 || n > 102) {
				parsed.push({
					key: "age min",
					valueText: value,
					valid: false,
					error: "18–102",
				});
			} else {
				draft.ageEnabled = true;
				draft.age = [n, draft.age[1]];
				parsed.push({
					key: "age min",
					valueText: String(n),
					valid: true,
				});
			}
		} else if (k === "agemax" || k === "age_max") {
			const n = Number(value);
			if (!Number.isFinite(n) || n < 18 || n > 102) {
				parsed.push({
					key: "age max",
					valueText: value,
					valid: false,
					error: "18–102",
				});
			} else {
				draft.ageEnabled = true;
				draft.age = [draft.age[0], n];
				parsed.push({
					key: "age max",
					valueText: String(n),
					valid: true,
				});
			}
		} else if (k === "photos" || k === "photoonly") {
			const b = parseBool(value);
			if (b === null) {
				parsed.push({
					key: "photos",
					valueText: value,
					valid: false,
					error: "Expected true/false",
				});
			} else if (b) {
				draft.photosEnabled = true;
				if (!draft.photos.includes("has-photos")) {
					draft.photos = [...draft.photos, "has-photos"];
				}
				parsed.push({
					key: "photos",
					valueText: "Has photos",
					valid: true,
				});
			} else {
				parsed.push({ key: "photos", valueText: "No", valid: true });
			}
		} else {
			parsed.push({
				key,
				valueText: value,
				valid: false,
				error: "Unknown filter",
			});
		}
	}

	const validCount = parsed.filter((p) => p.valid).length;
	return {
		filters: draft,
		parsed,
		validCount,
		invalidCount: parsed.length - validCount,
	};
}
