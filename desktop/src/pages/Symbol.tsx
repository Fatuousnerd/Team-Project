import { ChevronLeft } from "lucide-react";
import { useState } from "react";
import { Link, useParams } from "react-router";
import { OrderBook, RecentTrades } from "@/components/app/order-book";
import { OrderTicket } from "@/components/app/order-ticket";
import { Page } from "@/components/app/page";
import { PriceChart } from "@/components/app/price-chart";
import {
	Delta,
	MarketBadge,
	SectionHeader,
	SideText,
} from "@/components/app/primitives";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { positionStats, quoteChange } from "@/lib/mock/derive";
import {
	fmtCompact,
	fmtMoney,
	fmtPct,
	fmtPrice,
	fmtQty,
} from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import { TIMEFRAMES, type Timeframe } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

export default function SymbolPage() {
	const { symbol } = useParams();
	const quote = useMarketStore((state) =>
		symbol ? state.quotes[symbol.toUpperCase()] : undefined,
	);
	const position = useMarketStore((state) =>
		symbol
			? state.positions.find((item) => item.symbol === symbol.toUpperCase())
			: undefined,
	);
	const [timeframe, setTimeframe] = useState<Timeframe>("1D");

	if (!quote) {
		return (
			<Page>
				<Button asChild variant="ghost" size="xs" className="-ms-2 w-fit">
					<Link to="/markets">
						<ChevronLeft className="size-3.5" />
						Markets
					</Link>
				</Button>
				<Card>
					<CardContent className="flex flex-col items-center gap-2 py-16 text-center">
						<span className="num text-lg font-medium">
							{symbol?.toUpperCase() ?? "Unknown"} is not listed
						</span>
						<p className="text-sm text-muted-foreground">
							Pick an instrument from the markets table to open its ticket.
						</p>
						<Button asChild size="sm" className="mt-2">
							<Link to="/markets">Browse markets</Link>
						</Button>
					</CardContent>
				</Card>
			</Page>
		);
	}

	const change = quoteChange(quote);
	const stats = position ? positionStats(position, quote) : null;

	const facts = [
		{
			label: "Bid",
			value: fmtPrice(quote.bid, quote.market),
			tone: "positive",
		},
		{
			label: "Ask",
			value: fmtPrice(quote.ask, quote.market),
			tone: "negative",
		},
		{ label: "24h high", value: fmtPrice(quote.high, quote.market) },
		{ label: "24h low", value: fmtPrice(quote.low, quote.market) },
		{ label: "Volume", value: fmtCompact(quote.volume) },
	];

	return (
		<Page>
			<Button asChild variant="ghost" size="xs" className="-ms-2 w-fit">
				<Link to="/markets">
					<ChevronLeft className="size-3.5" />
					Markets
				</Link>
			</Button>

			<header className="flex flex-col gap-3">
				<div className="flex flex-wrap items-center gap-2">
					<h1 className="num text-2xl font-semibold tracking-tight">
						{quote.symbol}
					</h1>
					<MarketBadge market={quote.market} />
					<span className="text-xs text-muted-foreground">{quote.name}</span>
				</div>
				<div className="flex flex-wrap items-baseline gap-3">
					<span className="num text-3xl font-semibold tracking-tight">
						{fmtPrice(quote.price, quote.market)}
					</span>
					<Delta value={change} className="text-sm" />
					<span className="text-xs text-muted-foreground">
						{fmtMoney(quote.price - quote.prevClose, {
							signed: true,
						})}{" "}
						vs. previous close
					</span>
				</div>
			</header>

			<Card className="py-0">
				<CardContent className="grid grid-cols-2 gap-4 p-4 sm:grid-cols-3 lg:grid-cols-5">
					{facts.map((fact) => (
						<div key={fact.label} className="flex flex-col gap-0.5">
							<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
								{fact.label}
							</span>
							<span
								className={cn(
									"num text-sm font-medium",
									fact.tone === "positive" && "text-positive",
									fact.tone === "negative" && "text-negative",
								)}
							>
								{fact.value}
							</span>
						</div>
					))}
				</CardContent>
			</Card>

			<div className="grid gap-3 xl:grid-cols-[minmax(0,1fr)_336px]">
				<div className="flex min-w-0 flex-col gap-3">
					<Card>
						<CardHeader>
							<SectionHeader
								title="Price chart"
								description={`${quote.market} · simulated OHLC series`}
								actions={
									<div className="flex items-center gap-0.5 rounded-lg border border-border/80 p-0.5">
										{TIMEFRAMES.map((option) => (
											<button
												key={option}
												type="button"
												onClick={() => setTimeframe(option)}
												className={cn(
													"h-6 rounded-md px-2 text-[11px] font-medium transition-colors",
													timeframe === option
														? "bg-muted text-foreground"
														: "text-muted-foreground hover:text-foreground",
												)}
											>
												{option}
											</button>
										))}
									</div>
								}
							/>
						</CardHeader>
						<CardContent>
							<PriceChart
								symbol={quote.symbol}
								market={quote.market}
								timeframe={timeframe}
							/>
						</CardContent>
					</Card>

					{position && stats ? (
						<Card className="py-0">
							<CardContent className="flex flex-wrap items-center gap-x-8 gap-y-3 p-4">
								<div className="flex flex-col gap-0.5">
									<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
										Your position
									</span>
									<span className="flex items-center gap-1.5 text-sm font-medium">
										<SideText side={position.side} />
										<span className="num">
											{fmtQty(position.quantity)} {position.symbol}
										</span>
									</span>
								</div>
								<div className="flex flex-col gap-0.5">
									<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
										Entry
									</span>
									<span className="num text-sm font-medium">
										{fmtPrice(position.entry, position.market)}
									</span>
								</div>
								<div className="flex flex-col gap-0.5">
									<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
										Unrealized
									</span>
									<span
										className={cn(
											"num text-sm font-medium",
											stats.pnl >= 0 ? "text-positive" : "text-negative",
										)}
									>
										{fmtMoney(stats.pnl, { signed: true })} (
										{fmtPct(stats.pnlPct)})
									</span>
								</div>
								<Button asChild variant="outline" size="sm" className="ms-auto">
									<Link to="/portfolio">Manage position</Link>
								</Button>
							</CardContent>
						</Card>
					) : null}
				</div>

				<OrderTicket key={quote.symbol} quote={quote} />
			</div>

			<div className="grid gap-3 lg:grid-cols-2">
				<Card className="gap-0">
					<CardHeader>
						<SectionHeader
							title="Order book"
							description="Simulated top of book"
						/>
					</CardHeader>
					<CardContent className="px-0 pb-3">
						<OrderBook quote={quote} />
					</CardContent>
				</Card>

				<Card className="gap-0">
					<CardHeader>
						<SectionHeader title="Recent trades" description="Time and sales" />
					</CardHeader>
					<CardContent className="px-0 pb-3">
						<RecentTrades quote={quote} />
					</CardContent>
				</Card>
			</div>
		</Page>
	);
}
