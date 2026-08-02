/** Simple selection set with a max capacity (open-grind pattern). */
export class SelectionSet<T> {
	#items = $state(new Set<T>());
	readonly max: number;

	constructor(max = 10) {
		this.max = max;
	}

	get size(): number {
		return this.#items.size;
	}

	get canSelectMore(): boolean {
		return this.#items.size < this.max;
	}

	has(id: T): boolean {
		return this.#items.has(id);
	}

	toggle(id: T): void {
		if (this.#items.has(id)) {
			this.#items.delete(id);
			this.#items = new Set(this.#items);
			return;
		}
		if (this.#items.size >= this.max) return;
		this.#items.add(id);
		this.#items = new Set(this.#items);
	}

	clear(): void {
		this.#items = new Set();
	}

	values(): T[] {
		return [...this.#items];
	}
}
