import type {
	Account,
	Activity,
	Candle,
	Instrument,
	Order,
	Position,
	Quote,
	Strategy,
	Timeframe,
} from "@/lib/mock/types";

export type EquityRange = "1D" | "1W" | "1M" | "1Y";

function hashSeed(input: string) {
	let h = 2166136261;
	for (let i = 0; i < input.length; i += 1) {
		h ^= input.charCodeAt(i);
		h = Math.imul(h, 16777619);
	}
	return h >>> 0;
}

export function mulberry32(seed: number) {
	let a = seed;
	return () => {
		a |= 0;
		a = (a + 0x6d2b79f5) | 0;
		let t = Math.imul(a ^ (a >>> 15), 1 | a);
		t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

export const INSTRUMENTS: Instrument[] = [
	{
		symbol: "AAPL",
		name: "Apple Inc.",
		market: "Stocks",
		price: 228.42,
		volatility: 0.0009,
		decimals: 2,
	},
	{
		symbol: "TSLA",
		name: "Tesla, Inc.",
		market: "Stocks",
		price: 342.18,
		volatility: 0.0024,
		decimals: 2,
	},
	{
		symbol: "MSFT",
		name: "Microsoft Corp.",
		market: "Stocks",
		price: 468.9,
		volatility: 0.0008,
		decimals: 2,
	},
	{
		symbol: "GOOGL",
		name: "Alphabet Inc.",
		market: "Stocks",
		price: 205.64,
		volatility: 0.001,
		decimals: 2,
	},
	{
		symbol: "AMZN",
		name: "Amazon.com, Inc.",
		market: "Stocks",
		price: 231.07,
		volatility: 0.0011,
		decimals: 2,
	},
	{
		symbol: "NVDA",
		name: "NVIDIA Corp.",
		market: "Stocks",
		price: 178.35,
		volatility: 0.0018,
		decimals: 2,
	},
	{
		symbol: "META",
		name: "Meta Platforms, Inc.",
		market: "Stocks",
		price: 612.76,
		volatility: 0.0012,
		decimals: 2,
	},
	{
		symbol: "EURUSD",
		name: "Euro / US Dollar",
		market: "Forex",
		price: 1.0862,
		volatility: 0.00022,
		decimals: 4,
	},
	{
		symbol: "GBPUSD",
		name: "Pound / US Dollar",
		market: "Forex",
		price: 1.2718,
		volatility: 0.00026,
		decimals: 4,
	},
	{
		symbol: "USDJPY",
		name: "US Dollar / Yen",
		market: "Forex",
		price: 156.42,
		volatility: 0.00024,
		decimals: 3,
	},
	{
		symbol: "AUDUSD",
		name: "Aussie / US Dollar",
		market: "Forex",
		price: 0.6641,
		volatility: 0.00024,
		decimals: 4,
	},
	{
		symbol: "USDCAD",
		name: "US Dollar / Loonie",
		market: "Forex",
		price: 1.3724,
		volatility: 0.0002,
		decimals: 4,
	},
	{
		symbol: "USDCHF",
		name: "US Dollar / Franc",
		market: "Forex",
		price: 0.8842,
		volatility: 0.00022,
		decimals: 4,
	},
	{
		symbol: "BTCUSDT",
		name: "Bitcoin",
		market: "Crypto",
		price: 96_540.2,
		volatility: 0.0022,
		decimals: 2,
	},
	{
		symbol: "ETHUSDT",
		name: "Ethereum",
		market: "Crypto",
		price: 3_324.8,
		volatility: 0.0028,
		decimals: 2,
	},
	{
		symbol: "SOLUSDT",
		name: "Solana",
		market: "Crypto",
		price: 198.42,
		volatility: 0.0036,
		decimals: 2,
	},
	{
		symbol: "ADAUSDT",
		name: "Cardano",
		market: "Crypto",
		price: 0.9214,
		volatility: 0.0034,
		decimals: 4,
	},
	{
		symbol: "DOGEUSDT",
		name: "Dogecoin",
		market: "Crypto",
		price: 0.3182,
		volatility: 0.0042,
		decimals: 4,
	},
	{
		symbol: "XRPUSDT",
		name: "XRP",
		market: "Crypto",
		price: 2.2416,
		volatility: 0.003,
		decimals: 4,
	},
];

export const INSTRUMENT_MAP = new Map(INSTRUMENTS.map((i) => [i.symbol, i]));

function makeSpark(
	price: number,
	volatility: number,
	points: number,
	rng: () => number,
) {
	const values: number[] = [];
	let current = price;
	for (let i = 0; i < points; i += 1) {
		current *= 1 + (rng() - 0.48) * volatility * 4;
		values.push(current);
	}
	const scale = price / values[values.length - 1];
	return values.map((v) => v * scale);
}

function makeQuote(instrument: Instrument): Quote {
	const rng = mulberry32(hashSeed(instrument.symbol));
	const drift = (rng() - 0.5) * 0.028;
	const prevClose = instrument.price / (1 + drift);
	const spread =
		instrument.market === "Forex"
			? 0.00012
			: instrument.price * (instrument.market === "Crypto" ? 0.0006 : 0.0004);
	const volumeBase =
		instrument.market === "Forex"
			? 120_000_000
			: instrument.market === "Crypto"
				? 4_800_000
				: 24_000_000;

	return {
		symbol: instrument.symbol,
		name: instrument.name,
		market: instrument.market,
		price: instrument.price,
		prevClose,
		high: Math.max(instrument.price, prevClose) * (1 + rng() * 0.012),
		low: Math.min(instrument.price, prevClose) * (1 - rng() * 0.012),
		bid: instrument.price - spread / 2,
		ask: instrument.price + spread / 2,
		volume: volumeBase * (0.4 + rng() * 1.6),
		decimals: instrument.decimals,
		spark: makeSpark(instrument.price, instrument.volatility, 40, rng),
	};
}

export const QUOTES: Record<string, Quote> = Object.fromEntries(
	INSTRUMENTS.map((instrument) => [instrument.symbol, makeQuote(instrument)]),
);

const TIMEFRAME_CONFIG: Record<
	Timeframe,
	{ points: number; stepMs: number; candleVol: number }
> = {
	"1H": { points: 60, stepMs: 60_000, candleVol: 0.0005 },
	"4H": { points: 96, stepMs: 5 * 60_000, candleVol: 0.001 },
	"1D": { points: 96, stepMs: 15 * 60_000, candleVol: 0.0018 },
	"1W": { points: 112, stepMs: 90 * 60_000, candleVol: 0.0045 },
};

export function makeCandles(symbol: string, timeframe: Timeframe): Candle[] {
	const instrument = INSTRUMENT_MAP.get(symbol);
	if (!instrument) return [];
	const quote = QUOTES[symbol];
	const config = TIMEFRAME_CONFIG[timeframe];
	const rng = mulberry32(hashSeed(`${symbol}:${timeframe}`));
	const now = Date.now();
	const target = instrument.price;

	// Anchor the 1D series to the previous close so the chart trend agrees with
	// the 24h change shown in the header. Longer/shorter windows get their own
	// start level.
	const drift =
		timeframe === "1D"
			? (quote.prevClose - target) / target
			: (rng() - 0.5) *
				(timeframe === "1W" ? 0.18 : timeframe === "4H" ? 0.025 : 0.008);
	const start = target * (1 + drift);

	const noise: number[] = [];
	let walk = 0;
	for (let i = 0; i < config.points; i += 1) {
		walk += (rng() - 0.5) * start * config.candleVol;
		noise.push(walk);
	}
	// De-trend the noise so the series starts at `start` and lands exactly on
	// `target`, keeping the chart direction aligned with the quoted change.
	const lastNoise = noise[config.points - 1];
	const closes = noise.map(
		(value, index) =>
			start +
			((target - start) * (index + 1)) / config.points -
			(lastNoise * (index + 1)) / config.points +
			value,
	);

	const baseVolume =
		instrument.market === "Crypto"
			? 320
			: instrument.market === "Forex"
				? 900
				: 1400;

	return closes.map((close, index) => {
		const open =
			index === 0 ? close * (1 - (rng() - 0.5) * 0.004) : closes[index - 1];
		const high = Math.max(open, close) * (1 + rng() * config.candleVol * 0.9);
		const low = Math.min(open, close) * (1 - rng() * config.candleVol * 0.9);
		return {
			time: now - (config.points - 1 - index) * config.stepMs,
			open,
			high,
			low,
			close,
			volume: baseVolume * (0.35 + rng() * 1.9),
		};
	});
}

export const ACCOUNTS: Account[] = [
	{
		id: "acc-paper",
		name: "Paper · Main",
		kind: "Paper trading",
		balance: 118_450.24,
		buyingPower: 236_900.48,
		marginUsed: 12_480,
		dayPnl: 1_842.35,
		dayPnlPct: 1.58,
		realizedPnl: 642.18,
	},
	{
		id: "acc-momentum",
		name: "Paper · Momentum",
		kind: "Strategy sleeve",
		balance: 38_210.9,
		buyingPower: 76_421.8,
		marginUsed: 4_120,
		dayPnl: -318.42,
		dayPnlPct: -0.83,
		realizedPnl: 128.64,
	},
	{
		id: "acc-margin",
		name: "Margin · Pro",
		kind: "Margin account",
		balance: 212_680.5,
		buyingPower: 638_041.5,
		marginUsed: 84_220,
		dayPnl: 2_614.9,
		dayPnlPct: 1.24,
		realizedPnl: 1_102.4,
	},
];

export const POSITIONS: Position[] = [
	{
		id: "pos-1",
		symbol: "NVDA",
		market: "Stocks",
		side: "Long",
		quantity: 120,
		entry: 164.22,
		decimals: 2,
	},
	{
		id: "pos-2",
		symbol: "BTCUSDT",
		market: "Crypto",
		side: "Long",
		quantity: 0.28,
		entry: 91_240.5,
		decimals: 2,
	},
	{
		id: "pos-3",
		symbol: "AAPL",
		market: "Stocks",
		side: "Long",
		quantity: 95,
		entry: 219.84,
		decimals: 2,
	},
	{
		id: "pos-4",
		symbol: "ETHUSDT",
		market: "Crypto",
		side: "Long",
		quantity: 3.6,
		entry: 3_488.1,
		decimals: 2,
	},
	{
		id: "pos-5",
		symbol: "EURUSD",
		market: "Forex",
		side: "Short",
		quantity: 4_000,
		entry: 1.0924,
		decimals: 4,
	},
	{
		id: "pos-6",
		symbol: "TSLA",
		market: "Stocks",
		side: "Short",
		quantity: 40,
		entry: 351.7,
		decimals: 2,
	},
];

export function makeOrders(): Order[] {
	const now = Date.now();
	const rows: Array<
		[
			Order["symbol"],
			Order["side"],
			Order["type"],
			number,
			Order["status"],
			number,
		]
	> = [
		["NVDA", "Buy", "Limit", 60, "Open", 4],
		["BTCUSDT", "Buy", "Limit", 0.15, "Open", 18],
		["TSLA", "Sell", "Stop", 40, "Open", 42],
		["AAPL", "Buy", "Limit", 25, "Open", 71],
		["ETHUSDT", "Buy", "Market", 1.2, "Filled", 96],
		["META", "Buy", "Market", 10, "Filled", 128],
		["EURUSD", "Sell", "Market", 20_000, "Filled", 165],
		["SOLUSDT", "Buy", "Limit", 40, "Filled", 214],
		["GOOGL", "Buy", "Limit", 30, "Cancelled", 246],
		["DOGEUSDT", "Buy", "Market", 4_000, "Filled", 305],
		["AMZN", "Sell", "Limit", 18, "Cancelled", 360],
		["USDJPY", "Buy", "Limit", 30_000, "Cancelled", 421],
	];

	return rows.map(
		([symbol, side, type, quantity, status, minutesAgo], index) => {
			const quote = QUOTES[symbol];
			const price =
				type === "Market"
					? quote.price * (side === "Buy" ? 1.0006 : 0.9994)
					: quote.price * (side === "Buy" ? 0.994 : 1.006);
			return {
				id: `ord-${index + 1}`,
				symbol,
				market: quote.market,
				side,
				type,
				quantity,
				price,
				filled:
					status === "Filled"
						? quantity
						: status === "Open"
							? quantity * 0.35
							: 0,
				status,
				createdAt: now - minutesAgo * 60_000,
				accountId: "acc-paper",
				decimals: quote.decimals,
			};
		},
	);
}

function strategySpark(seed: string, points: number, trend: number) {
	const rng = mulberry32(hashSeed(seed));
	const values: number[] = [];
	let current = 100;
	for (let i = 0; i < points; i += 1) {
		current *= 1 + (rng() - 0.5) * 0.02 + trend / 1000;
		values.push(current);
	}
	return values;
}

export const STRATEGIES: Strategy[] = [
	{
		id: "strat-1",
		name: "Momentum Scalper",
		market: "Crypto",
		status: "running",
		pnl: 4_216.38,
		pnlPct: 12.4,
		trades: 318,
		winRate: 61.2,
		maxDrawdown: 6.8,
		timeframe: "1H",
		risk: "High",
		description:
			"Rides short-term breakouts on high-volume crypto pairs with a tight stop.",
		spark: strategySpark("momentum", 32, 16),
	},
	{
		id: "strat-2",
		name: "Mean Reversion",
		market: "Stocks",
		status: "running",
		pnl: 2_104.9,
		pnlPct: 8.1,
		trades: 146,
		winRate: 57.5,
		maxDrawdown: 3.9,
		timeframe: "1D",
		risk: "Medium",
		description:
			"Buys liquid large caps when they stretch below their 20-day band.",
		spark: strategySpark("meanrev", 32, 9),
	},
	{
		id: "strat-3",
		name: "FX Carry Harvester",
		market: "Forex",
		status: "paused",
		pnl: -842.12,
		pnlPct: -2.6,
		trades: 64,
		winRate: 48.4,
		maxDrawdown: 5.2,
		timeframe: "4H",
		risk: "Low",
		description:
			"Holds high-yield majors against low-yielders, rebalancing on rate prints.",
		spark: strategySpark("carry", 32, -4),
	},
	{
		id: "strat-4",
		name: "DCA Accumulator",
		market: "Crypto",
		status: "running",
		pnl: 6_508.2,
		pnlPct: 18.9,
		trades: 52,
		winRate: 68.9,
		maxDrawdown: 9.4,
		timeframe: "1W",
		risk: "Medium",
		description:
			"Scheduled BTC and ETH accumulation with drawdown-scaled sizing.",
		spark: strategySpark("dca", 32, 22),
	},
];

export const ACTIVITY: Activity[] = [
	{
		id: "act-1",
		kind: "bot",
		title: "Momentum Scalper opened SOLUSDT",
		detail: "Long 24 @ 197.82 · stop 194.10",
		at: Date.now() - 92_000,
	},
	{
		id: "act-2",
		kind: "fill",
		title: "ETHUSDT buy filled",
		detail: "1.2 @ 3,328.40 · Paper · Main",
		at: Date.now() - 8 * 60_000,
	},
	{
		id: "act-3",
		kind: "alert",
		title: "NVDA crossed above 176.00",
		detail: "Price alert · watchlist",
		at: Date.now() - 26 * 60_000,
	},
	{
		id: "act-4",
		kind: "fill",
		title: "META buy filled",
		detail: "10 @ 610.20 · Paper · Main",
		at: Date.now() - 2 * 60 * 60_000,
	},
	{
		id: "act-5",
		kind: "system",
		title: "Mock feed connected",
		detail: "Streaming 19 symbols · paper engine",
		at: Date.now() - 5 * 60 * 60_000,
	},
];

const EQUITY_RANGES: Record<
	EquityRange,
	{
		points: number;
		stepMs: number;
		start: number;
		end: number;
		benchEnd: number;
	}
> = {
	"1D": {
		points: 96,
		stepMs: 5 * 60_000,
		start: 116_607,
		end: 118_450,
		benchEnd: 117_980,
	},
	"1W": {
		points: 96,
		stepMs: 2 * 60 * 60_000,
		start: 111_900,
		end: 118_450,
		benchEnd: 115_220,
	},
	"1M": {
		points: 90,
		stepMs: 8 * 60 * 60_000,
		start: 104_200,
		end: 118_450,
		benchEnd: 109_600,
	},
	"1Y": {
		points: 120,
		stepMs: 3 * 24 * 60 * 60_000,
		start: 82_400,
		end: 118_450,
		benchEnd: 96_300,
	},
};

export interface EquityPoint {
	time: number;
	value: number;
	benchmark: number;
}

export function makeEquityCurve(range: EquityRange): EquityPoint[] {
	const config = EQUITY_RANGES[range];
	const rng = mulberry32(hashSeed(`equity:${range}`));
	const now = Date.now();
	const points: EquityPoint[] = [];
	let value = config.start;
	let benchmark = config.start;

	for (let i = 0; i < config.points; i += 1) {
		const progress = (i + 1) / config.points;
		const target = config.start + (config.end - config.start) * progress;
		const benchTarget =
			config.start + (config.benchEnd - config.start) * progress;
		value += (target - value) * 0.22 + (rng() - 0.5) * config.start * 0.004;
		benchmark +=
			(benchTarget - benchmark) * 0.18 + (rng() - 0.5) * config.start * 0.0028;
		points.push({
			time: now - (config.points - 1 - i) * config.stepMs,
			value,
			benchmark,
		});
	}

	return points;
}

export const RANGE_LABELS: Record<EquityRange, string> = {
	"1D": "1D",
	"1W": "1W",
	"1M": "1M",
	"1Y": "1Y",
};
