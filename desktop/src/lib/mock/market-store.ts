import { create } from "zustand";
import {
	ACCOUNTS,
	ACTIVITY,
	INSTRUMENT_MAP,
	makeOrders,
	POSITIONS,
	QUOTES,
	STRATEGIES,
} from "@/lib/mock/data";
import type {
	Account,
	Activity,
	NewOrder,
	NewStrategy,
	Order,
	Position,
	Quote,
	Strategy,
} from "@/lib/mock/types";

interface MarketStore {
	quotes: Record<string, Quote>;
	accounts: Account[];
	activeAccountId: string;
	positions: Position[];
	orders: Order[];
	strategies: Strategy[];
	activity: Activity[];
	lastTick: number;
	latency: number;
	tick: () => void;
	setActiveAccount: (id: string) => void;
	placeOrder: (input: NewOrder) => Order;
	cancelOrder: (id: string) => void;
	toggleStrategy: (id: string) => void;
	addStrategy: (input: NewStrategy) => void;
}

function makeActivity(
	kind: Activity["kind"],
	title: string,
	detail: string,
): Activity {
	return {
		id: `act-${crypto.randomUUID()}`,
		kind,
		title,
		detail,
		at: Date.now(),
	};
}

function spreadFor(quote: Quote) {
	return quote.market === "Forex"
		? 0.00012
		: quote.price * (quote.market === "Crypto" ? 0.0006 : 0.0004);
}

export const useMarketStore = create<MarketStore>((set, get) => ({
	quotes: QUOTES,
	accounts: ACCOUNTS,
	activeAccountId: ACCOUNTS[0].id,
	positions: POSITIONS,
	orders: makeOrders(),
	strategies: STRATEGIES,
	activity: ACTIVITY,
	lastTick: Date.now(),
	latency: 24,

	tick: () => {
		const state = get();
		const quotes: Record<string, Quote> = {};

		for (const quote of Object.values(state.quotes)) {
			const instrument = INSTRUMENT_MAP.get(quote.symbol);
			if (!instrument) {
				quotes[quote.symbol] = quote;
				continue;
			}
			const drift = (Math.random() - 0.5) * instrument.volatility * 1.8;
			const price = quote.price * (1 + drift);
			const spread = spreadFor({ ...quote, price });

			quotes[quote.symbol] = {
				...quote,
				price,
				high: Math.max(quote.high, price),
				low: Math.min(quote.low, price),
				bid: price - spread / 2,
				ask: price + spread / 2,
				volume: quote.volume + quote.volume * 0.00035 * Math.random(),
				spark: [...quote.spark.slice(-39), price],
			};
		}

		const strategies = state.strategies.map((strategy) =>
			strategy.status === "running"
				? {
						...strategy,
						spark: [
							...strategy.spark.slice(-31),
							strategy.spark[strategy.spark.length - 1] *
								(1 + (Math.random() - 0.48) * 0.01),
						],
					}
				: strategy,
		);

		set({
			quotes,
			strategies,
			lastTick: Date.now(),
			latency: 16 + Math.round(Math.random() * 22),
		});
	},

	setActiveAccount: (id) => set({ activeAccountId: id }),

	placeOrder: (input) => {
		const state = get();
		const quote = state.quotes[input.symbol];
		const fillPrice =
			input.type === "Market"
				? input.side === "Buy"
					? quote.ask
					: quote.bid
				: (input.price ?? quote.price);
		const order: Order = {
			id: `ord-${crypto.randomUUID()}`,
			symbol: input.symbol,
			market: quote.market,
			side: input.side,
			type: input.type,
			quantity: input.quantity,
			price: fillPrice,
			filled: input.type === "Market" ? input.quantity : 0,
			status: input.type === "Market" ? "Filled" : "Open",
			createdAt: Date.now(),
			accountId: state.activeAccountId,
			decimals: quote.decimals,
		};

		set({
			orders: [order, ...state.orders],
			activity: [
				makeActivity(
					input.type === "Market" ? "fill" : "system",
					input.type === "Market"
						? `${input.symbol} ${input.side.toLowerCase()} filled`
						: `${input.symbol} ${input.type.toLowerCase()} order placed`,
					`${input.quantity} @ ${fillPrice.toFixed(quote.decimals)} · paper account`,
				),
				...state.activity,
			].slice(0, 40),
		});

		return order;
	},

	cancelOrder: (id) => {
		const state = get();
		const target = state.orders.find((order) => order.id === id);
		if (!target) return;
		set({
			orders: state.orders.map((order) =>
				order.id === id ? { ...order, status: "Cancelled" } : order,
			),
			activity: [
				makeActivity(
					"system",
					`${target.symbol} order cancelled`,
					`Order ${target.id.slice(0, 8)} · paper account`,
				),
				...state.activity,
			].slice(0, 40),
		});
	},

	toggleStrategy: (id) => {
		const state = get();
		const target = state.strategies.find((strategy) => strategy.id === id);
		if (!target) return;
		const nextStatus = target.status === "running" ? "paused" : "running";
		set({
			strategies: state.strategies.map((strategy) =>
				strategy.id === id ? { ...strategy, status: nextStatus } : strategy,
			),
			activity: [
				makeActivity(
					"bot",
					`${target.name} ${nextStatus === "running" ? "started" : "paused"}`,
					`${target.market} · ${target.timeframe} · ${target.risk.toLowerCase()} risk`,
				),
				...state.activity,
			].slice(0, 40),
		});
	},

	addStrategy: (input) => {
		const state = get();
		const spark: number[] = [];
		let current = 100;
		for (let i = 0; i < 32; i += 1) {
			current *= 1 + (Math.random() - 0.48) * 0.02;
			spark.push(current);
		}
		const strategy: Strategy = {
			id: `strat-${crypto.randomUUID()}`,
			name: input.name,
			market: input.market,
			status: "paused",
			pnl: 0,
			pnlPct: 0,
			trades: 0,
			winRate: 0,
			maxDrawdown: 0,
			timeframe: input.timeframe,
			risk: input.risk,
			description:
				"Draft strategy — configure entry and exit rules before starting.",
			spark,
		};
		set({
			strategies: [strategy, ...state.strategies],
			activity: [
				makeActivity(
					"bot",
					`${input.name} created`,
					`${input.market} · ${input.timeframe} · draft`,
				),
				...state.activity,
			].slice(0, 40),
		});
	},
}));
