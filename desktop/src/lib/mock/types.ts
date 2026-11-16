export type MarketType = "Stocks" | "Forex" | "Crypto";

export type OrderSide = "Buy" | "Sell";
export type OrderType = "Market" | "Limit" | "Stop";
export type OrderStatus = "Open" | "Filled" | "Cancelled";

export const TIMEFRAMES = ["1H", "4H", "1D", "1W"] as const;
export type Timeframe = (typeof TIMEFRAMES)[number];

export interface Instrument {
	symbol: string;
	name: string;
	market: MarketType;
	price: number;
	volatility: number;
	decimals: number;
}

export interface Quote {
	symbol: string;
	name: string;
	market: MarketType;
	price: number;
	prevClose: number;
	high: number;
	low: number;
	bid: number;
	ask: number;
	volume: number;
	decimals: number;
	spark: number[];
}

export interface Candle {
	time: number;
	open: number;
	high: number;
	low: number;
	close: number;
	volume: number;
}

export interface Position {
	id: string;
	symbol: string;
	market: MarketType;
	side: "Long" | "Short";
	quantity: number;
	entry: number;
	decimals: number;
}

export interface Order {
	id: string;
	symbol: string;
	market: MarketType;
	side: OrderSide;
	type: OrderType;
	quantity: number;
	price: number;
	filled: number;
	status: OrderStatus;
	createdAt: number;
	accountId: string;
	decimals: number;
}

export interface Account {
	id: string;
	name: string;
	kind: string;
	balance: number;
	buyingPower: number;
	marginUsed: number;
	dayPnl: number;
	dayPnlPct: number;
	realizedPnl: number;
}

export type StrategyStatus = "running" | "paused";

export interface Strategy {
	id: string;
	name: string;
	market: MarketType;
	status: StrategyStatus;
	pnl: number;
	pnlPct: number;
	trades: number;
	winRate: number;
	maxDrawdown: number;
	timeframe: Timeframe;
	risk: "Low" | "Medium" | "High";
	description: string;
	spark: number[];
}

export type ActivityKind = "fill" | "bot" | "alert" | "system";

export interface Activity {
	id: string;
	kind: ActivityKind;
	title: string;
	detail: string;
	at: number;
}

export interface NewOrder {
	symbol: string;
	side: OrderSide;
	type: OrderType;
	quantity: number;
	price?: number;
}

export interface NewStrategy {
	name: string;
	market: MarketType;
	risk: Strategy["risk"];
	timeframe: Timeframe;
}
