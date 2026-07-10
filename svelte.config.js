// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const projectVersion = JSON.parse(
		fs.readFileSync(
			path.join(
				path.dirname(fileURLToPath(import.meta.url)),
				"./package.json",
			),
			"utf-8",
		),
	).version
const grindrApiVersion = fs
	.readFileSync(
		path.join(
			path.dirname(fileURLToPath(import.meta.url)),
			"./src-tauri/src/api/version.rs",
		),
		"utf-8",
	)
	.match(/const FALLBACK_APP_VERSION: &str = "([^"]+)";/)[1];
const grindrApiBuildNumber = fs
	.readFileSync(
		path.join(
			path.dirname(fileURLToPath(import.meta.url)),
			"./src-tauri/src/api/version.rs",
		),
		"utf-8",
	)
	.match(/const FALLBACK_BUILD_NUMBER: &str = "([^"]+)";/)[1];

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	compilerOptions: {
		experimental: {
			async: true,
		},
	},
	kit: {
		adapter: adapter({
			fallback: "index.html",
			pages: "dist",
		}),
		alias: {
			$layout: "src/layout.css",
		},
		version: {
			name: `OpenGrind/${projectVersion}\ngrindr3/${grindrApiVersion};${grindrApiBuildNumber}`,
		},
	},
};

export default config;
