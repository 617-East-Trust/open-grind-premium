import { callMethod } from "$lib/api";
import { redirect } from "@sveltejs/kit";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
	console.log("Checking auth state...");
	const profileId = await callMethod("auth_state").catch((e) => {
		console.error(e);
		return null;
	});
	console.log("Auth state:", profileId);
	if (profileId === null) {
		throw redirect(303, "/auth/sign-in");
	}
	return { ourProfileId: profileId };
	// TODO: consider typesafe context?
};
