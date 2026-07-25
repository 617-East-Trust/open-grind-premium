<script lang="ts">
	import { page } from "$app/state";
	import { tabsListVariants } from "$lib/components/ui/tabs";

	let { children }: import("./$types").LayoutProps = $props();

	const tabs = [
		{ href: "/interest/taps", label: "Taps", id: "taps" },
		{ href: "/interest/views", label: "Views", id: "views" },
	];
</script>

<div class="flex flex-col flex-1 min-h-0">
	<div class="px-4 pt-3 pb-2">
		<div class={tabsListVariants({ variant: "default" }) + " w-full"}>
			{#each tabs as tab}
				<a
					href={tab.href}
					class="flex-1 text-center py-2 text-sm rounded-full transition-colors"
					data-active={page.url.pathname.startsWith(tab.href)}
					class:data-active={page.url.pathname.startsWith(tab.href)}
				>
					{tab.label}
				</a>
			{/each}
		</div>
	</div>
	<div class="flex-1 overflow-y-auto">
		{@render children?.()}
	</div>
</div>

<style lang="postcss">
	@reference "../../../../layout.css";

	a[data-active="true"] {
		@apply bg-accent text-accent-foreground font-medium;
	}
</style>