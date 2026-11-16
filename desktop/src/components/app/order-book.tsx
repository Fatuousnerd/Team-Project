import { useMemo } from "react";
import { fmtPrice, fmtQty, fmtTime } from "@/lib/mock/format";
import type { Quote } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

function sizeAt(price: number, index: number, seed: number) {
	return Math.abs(Math.sin(price * (index + 1.7) + seed)) * 900 + 40;
}

export function OrderBook({ quote }: { quote: Quote }) {
	const { asks, bids, max, spread } = useMemo(() => {
		const levels = 7;
		const tick = quote.price * (quote.market === "Crypto" ? 0.00012 : 0.00008);
		const askLevels = Array.from({ length: levels }, (_, index) => {
			const price = quote.ask + tick * (levels - index - 1);
			const size = sizeAt(price, index, 1.3);
			return { price, size };
		});
		const bidLevels = Array.from({ length: levels }, (_, index) => {
			const price = quote.bid - tick * index;
			const size = sizeAt(price, index, 4.7);
			return { price, size };
		});
		const all = [...askLevels, ...bidLevels];
		return {
			asks: askLevels.reverse(),
			bids: bidLevels,
			max: Math.max(...all.map((level) => level.size)),
			spread: quote.ask - quote.bid,
		};
	}, [quote.ask, quote.bid, quote.market, quote.price]);

	return (
		<div className="flex flex-col">
			<div className="grid grid-cols-3 px-4 pb-2 text-[10px] tracking-wide text-muted-foreground uppercase">
				<span>Price</span>
				<span className="text-end">Size</span>
				<span className="text-end">Total</span>
			</div>

			<div className="flex flex-col">
				{asks.map((level) => (
					<div
						key={`ask-${level.price}`}
						className="relative grid h-6 grid-cols-3 items-center px-4 text-xs"
					>
						<span
							className="absolute inset-y-0 end-0 bg-negative/8"
							style={{ width: `${(level.size / max) * 100}%` }}
						/>
						<span className="num relative text-negative">
							{fmtPrice(level.price, quote.market)}
						</span>
						<span className="num relative text-end text-muted-foreground">
							{fmtQty(Math.round(level.size))}
						</span>
						<span className="num relative text-end text-muted-foreground/70">
							{fmtQty(Math.round(level.price * level.size))}
						</span>
					</div>
				))}
			</div>

			<div className="my-1 flex items-center justify-between border-y border-border/70 px-4 py-1.5">
				<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
					Spread
				</span>
				<span className="num text-xs font-medium">
					{fmtPrice(spread, quote.market)}
				</span>
			</div>

			<div className="flex flex-col">
				{bids.map((level) => (
					<div
						key={`bid-${level.price}`}
						className="relative grid h-6 grid-cols-3 items-center px-4 text-xs"
					>
						<span
							className="absolute inset-y-0 end-0 bg-primary/8"
							style={{ width: `${(level.size / max) * 100}%` }}
						/>
						<span className="num relative text-positive">
							{fmtPrice(level.price, quote.market)}
						</span>
						<span className="num relative text-end text-muted-foreground">
							{fmtQty(Math.round(level.size))}
						</span>
						<span className="num relative text-end text-muted-foreground/70">
							{fmtQty(Math.round(level.price * level.size))}
						</span>
					</div>
				))}
			</div>
		</div>
	);
}

export function RecentTrades({ quote }: { quote: Quote }) {
	const trades = useMemo(() => {
		const now = Date.now();
		return Array.from({ length: 9 }, (_, index) => {
			const offset =
				Math.sin((quote.price + index) * (index + 1)) * quote.price * 0.0004;
			const buy = Math.sin((quote.price + index) * 2.3) > 0;
			return {
				id: `${quote.symbol}-${index}`,
				price: quote.price + offset,
				size: Math.abs(Math.sin((quote.price + index) * 3.1)) * 240 + 2,
				buy,
				at: now - index * 3_200,
			};
		});
	}, [quote.price, quote.symbol]);

	return (
		<div className="flex flex-col">
			<div className="grid grid-cols-3 px-4 pb-2 text-[10px] tracking-wide text-muted-foreground uppercase">
				<span>Price</span>
				<span className="text-end">Size</span>
				<span className="text-end">Time</span>
			</div>
			{trades.map((trade) => (
				<div
					key={trade.id}
					className="grid h-7 grid-cols-3 items-center px-4 text-xs"
				>
					<span
						className={cn("num", trade.buy ? "text-positive" : "text-negative")}
					>
						{fmtPrice(trade.price, quote.market)}
					</span>
					<span className="num text-end text-muted-foreground">
						{fmtQty(Number(trade.size.toFixed(3)))}
					</span>
					<span className="num text-end text-muted-foreground/70">
						{fmtTime(trade.at)}
					</span>
				</div>
			))}
		</div>
	);
}
