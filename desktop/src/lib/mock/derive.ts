import type { Account, Position, Quote, Strategy } from "@/lib/mock/types";

export function quoteChange(quote: Quote) {
	return ((quote.price - quote.prevClose) / quote.prevClose) * 100;
}

export function positionStats(position: Position, quote: Quote | undefined) {
	const price = quote?.price ?? position.entry;
	const direction = position.side === "Long" ? 1 : -1;
	const pnl = (price - position.entry) * position.quantity * direction;
	const notional = position.entry * position.quantity;
	const value = price * position.quantity;
	return {
		price,
		value,
		pnl,
		pnlPct: notional === 0 ? 0 : (pnl / notional) * 100,
	};
}

export function portfolioStats(
	account: Account,
	positions: Position[],
	quotes: Record<string, Quote>,
) {
	let unrealized = 0;
	let exposure = 0;

	for (const position of positions) {
		const stats = positionStats(position, quotes[position.symbol]);
		unrealized += stats.pnl;
		exposure += Math.abs(stats.value);
	}

	const equity = account.balance + unrealized;
	return {
		equity,
		unrealized,
		exposure,
		openPositions: positions.length,
		buyingPower: account.buyingPower,
	};
}

export function allocationByMarket(
	positions: Position[],
	quotes: Record<string, Quote>,
) {
	const buckets = new Map<string, number>();
	for (const position of positions) {
		const stats = positionStats(position, quotes[position.symbol]);
		buckets.set(
			position.market,
			(buckets.get(position.market) ?? 0) + Math.abs(stats.value),
		);
	}
	return [...buckets.entries()]
		.map(([name, value]) => ({ name, value }))
		.sort((a, b) => b.value - a.value);
}

export function strategyStats(strategies: Strategy[]) {
	const running = strategies.filter((s) => s.status === "running").length;
	const pnl = strategies.reduce((sum, s) => sum + s.pnl, 0);
	const trades = strategies.reduce((sum, s) => sum + s.trades, 0);
	const weightedWins = strategies.reduce(
		(sum, s) => sum + s.winRate * s.trades,
		0,
	);
	return {
		running,
		paused: strategies.length - running,
		pnl,
		trades,
		winRate: trades === 0 ? 0 : weightedWins / trades,
	};
}

export function marketStatus() {
	const hour = new Date().getHours();
	const weekday = new Date().getDay();
	const isWeekend = weekday === 0 || weekday === 6;
	const isOpen = !isWeekend && hour >= 9 && hour < 16;
	return isOpen ? "Markets open" : "Markets closed";
}
