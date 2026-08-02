<script lang="ts">
	import * as Command from "$lib/components/ui/command";
	import QuickGoToCommandItem from "./QuickGoToCommandItem.svelte";
</script>

<Command.Group heading="Quick go to...">
	<QuickGoToCommandItem link="/" value="browse" />
	<QuickGoToCommandItem link="/map" value="map" />
	<QuickGoToCommandItem link="/right-now" value="right-now" />
	<QuickGoToCommandItem link="/interest/taps" value="taps" />
	<QuickGoToCommandItem link="/interest/views" value="views" />
	<QuickGoToCommandItem link="/chat" value="chat" />
	<QuickGoToCommandItem link="/settings" value="settings" />
</Command.Group>
