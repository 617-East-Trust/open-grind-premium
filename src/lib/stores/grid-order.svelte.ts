// Shared in-memory snapshot of the current Browse grid profile order.
// Profile detail only knows one id; swipe next/prev reads this list.

let order = $state<number[]>([]);

/** Replace the published grid order (called by the grid as it loads). */
export function setGridOrder(ids: number[]): void {
	order = ids;
}

/** The current ordered list of grid profile ids. */
export function getGridOrder(): number[] {
	return order;
}

/**
 * Resolve the neighbour of `id` in the current grid order.
 * Returns `null` when the id isn't in the order or there is no neighbour.
 */
export function getAdjacentProfileId(
	id: number,
	direction: "next" | "prev",
): number | null {
	const index = order.indexOf(id);
	if (index === -1) return null;
	const nextIndex = direction === "next" ? index + 1 : index - 1;
	if (nextIndex < 0 || nextIndex >= order.length) return null;
	return order[nextIndex] ?? null;
}
