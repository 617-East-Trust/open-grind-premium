<script lang="ts">
	import { callMethod } from "$lib/api";
	import { onMount } from "svelte";

	onMount(() => {
		callMethod("logout")
			.catch((e) => {
				console.error(e);
				return null;
			})
			.then(() => {
				window.location.href = "/auth/sign-in";
			});
	});
</script>
