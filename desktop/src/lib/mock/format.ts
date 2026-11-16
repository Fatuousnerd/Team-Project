import type { MarketType } from "@/lib/mock/types";

const usd = new Intl.NumberFormat("en-US", {
	style: "currency",
	currency: "USD",
	maximumFractionDigits: 2,
});

const usdWhole = new Intl.NumberFormat("en-US", {
	style: "currency",
	currency: "USD",
	maximumFractionDigits: 0,
});

export function fmtMoney(
	value: number,
	opts?: { whole?: boolean; signed?: boolean },
) {
	const formatted = (opts?.whole ? usdWhole : usd).format(Math.abs(value));
	if (!opts?.signed) return value < 0 ? `-${formatted}` : formatted;
	return `${value < 0 ? "-" : "+"}${formatted}`;
}

export function fmtPrice(value: number, market: MarketType = "Stocks") {
	const digits =
		market === "Forex"
			? value >= 20
				? 3
				: 4
			: market === "Crypto" && value < 100
				? 4
				: 2;
	return new Intl.NumberFormat("en-US", {
		minimumFractionDigits: digits,
		maximumFractionDigits: digits,
	}).format(value);
}

export function fmtPct(value: number, signed = true) {
	const sign = signed && value > 0 ? "+" : "";
	return `${sign}${value.toFixed(2)}%`;
}

export function fmtQty(value: number) {
	return new Intl.NumberFormat("en-US", { maximumFractionDigits: 4 }).format(
		value,
	);
}

export function fmtCompact(value: number) {
	return new Intl.NumberFormat("en-US", {
		notation: "compact",
		maximumFractionDigits: 1,
	}).format(value);
}

export function fmtTime(ts: number) {
	return new Intl.DateTimeFormat("en-US", {
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
		hour12: false,
	}).format(ts);
}

export function fmtDateTime(ts: number) {
	return new Intl.DateTimeFormat("en-US", {
		month: "short",
		day: "numeric",
		hour: "2-digit",
		minute: "2-digit",
		hour12: false,
	}).format(ts);
}

export function fmtAgo(ts: number) {
	const diff = Math.max(0, Date.now() - ts);
	const minutes = Math.floor(diff / 60_000);
	if (minutes < 1) return "just now";
	if (minutes < 60) return `${minutes}m ago`;
	const hours = Math.floor(minutes / 60);
	if (hours < 24) return `${hours}h ago`;
	return `${Math.floor(hours / 24)}d ago`;
}
