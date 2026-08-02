import { tick } from "svelte";

export const commandCenterState = $state({
	open: false,
	query: "",
	value: "",
});

export function commandCenterClose() {
	commandCenterState.open = false;
	void tick().then(() => {
		setTimeout(() => {
			if (!commandCenterState.open) {
				commandCenterState.query = "";
			}
		}, 200);
	});
}
