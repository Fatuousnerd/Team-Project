import { Channel, invoke } from "@tauri-apps/api/core";

export class AppService {
	async checkUpdate() {
		try {
			const h = await invoke("fetch_update");

			return h;
		} catch (err) {
			console.warn(`Update Error: `, err);
		}
	}

	async installUpdate() {
		await invoke("install_update", {
			onEvent: new Channel((event) => {
				console.log(event);
			}),
		});
	}
}
