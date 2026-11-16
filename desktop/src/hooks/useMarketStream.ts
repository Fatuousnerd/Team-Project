import { useEffect } from "react";
import { useMarketStore } from "@/lib/mock/market-store";

/**
 * Drives the mock market feed. Prices tick locally so the UI feels live
 * without requiring the Rust API to be running.
 */
export function useMarketStream(intervalMs = 1500) {
	const tick = useMarketStore((state) => state.tick);

	useEffect(() => {
		tick();
		const id = window.setInterval(tick, intervalMs);
		return () => window.clearInterval(id);
	}, [tick, intervalMs]);
}
