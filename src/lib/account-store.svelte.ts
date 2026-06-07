import { callMethod } from "$lib/api";

export interface Account {
	profileId: string;
	email: string;
	isActive: boolean;
}

let accounts = $state<Account[]>([]);
let loaded = $state(false);

export function getAccounts() {
	return {
		get accounts() {
			return accounts;
		},
		get loaded() {
			return loaded;
		},
		get activeAccount() {
			return accounts.find((a) => a.isActive) ?? null;
		},
		async refresh() {
			try {
				accounts = (await callMethod("list_accounts")) as Account[];
			} catch {
				accounts = [];
			}
			loaded = true;
		},
		async addAccount(email: string, password: string) {
			const result = (await callMethod("add_account", {
				email,
				password,
			})) as { profileId: string };
			await getAccounts().refresh();
			return result;
		},
		async switchAccount(profileId: string) {
			await callMethod("switch_account", { profileId });
			await getAccounts().refresh();
		},
		async removeAccount(profileId: string) {
			await callMethod("remove_account", { profileId });
			await getAccounts().refresh();
		},
		async logout() {
			await callMethod("logout");
			accounts = [];
			loaded = false;
		},
	};
}
