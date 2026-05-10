<script lang="ts">
	import { format } from "date-fns";
	import { getMessageContext } from "./context";

	const { timestamp, msgOut } = getMessageContext()();
</script>

<!-- TODO: read indicator -->
<span
	class={[
		"text-xs text-muted-foreground mx-3 mt-0.5",
		{ "text-right": msgOut },
	]}
>
	{format(new Date(timestamp), "H:mm")}
</span>
