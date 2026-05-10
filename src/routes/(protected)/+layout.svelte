<script lang="ts">
	import NavBar from "$lib/components/NavBar.svelte";

	let { children, data }: import("./$types").LayoutProps = $props();
</script>

{@render children?.()}
<NavBar ourProfileId={data.ourProfileId} />
