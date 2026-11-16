import { Banknote, Gauge, Plus, TrendingUp, Wallet } from "lucide-react";
import { useMemo, useState } from "react";
import { Link } from "react-router";
import { ActivityList } from "@/components/app/activity-list";
import { AllocationChart } from "@/components/app/allocation-chart";
import { EquityChart } from "@/components/app/equity-chart";
import { OrdersTable } from "@/components/app/orders-table";
import { Page } from "@/components/app/page";
import { PositionsTable } from "@/components/app/positions-table";
import {
	Delta,
	PageHeader,
	SectionHeader,
	Sparkline,
	StatCard,
} from "@/components/app/primitives";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { type EquityRange, makeEquityCurve } from "@/lib/mock/data";
import { portfolioStats, quoteChange, strategyStats } from "@/lib/mock/derive";
import { fmtMoney, fmtPrice } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import { cn } from "@/lib/utils";

const WATCHLIST = ["BTCUSDT", "NVDA", "ETHUSDT", "AAPL", "TSLA", "EURUSD"];
const RANGES: EquityRange[] = ["1D", "1W", "1M", "1Y"];

export default function Dashboard() {
	const [range, setRange] = useState<EquityRange>("1D");

	const account = useMarketStore(
		(state) =>
			state.accounts.find((item) => item.id === state.activeAccountId) ??
			state.accounts[0],
	);
	const positions = useMarketStore((state) => state.positions);
	const quotes = useMarketStore((state) => state.quotes);
	const orders = useMarketStore((state) => state.orders);
	const strategies = useMarketStore((state) => state.strategies);
	const activity = useMarketStore((state) => state.activity);

	const stats = portfolioStats(account, positions, quotes);
	const stratStats = strategyStats(strategies);
	const curve = useMemo(() => makeEquityCurve(range), [range]);
	const equitySpark = useMemo(() => curve.map((point) => point.value), [curve]);

	return (
		<Page>
			<PageHeader
				title="Dashboard"
				description="Paper-trading desk overview. Prices are simulated locally — no broker connection."
				actions={
					<Button asChild size="sm">
						<Link to="/markets">
							<Plus className="size-4" />
							New order
						</Link>
					</Button>
				}
			/>

			<div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
				<StatCard
					label="Portfolio value"
					value={fmtMoney(stats.equity, { whole: true })}
					hint="Paper equity"
					icon={Wallet}
					spark={equitySpark}
					sparkClassName={cn(
						"w-20",
						account.dayPnl >= 0 ? "text-positive" : "text-negative",
					)}
				/>
				<StatCard
					label="Day P&L"
					value={fmtMoney(account.dayPnl, { signed: true, whole: true })}
					delta={account.dayPnlPct}
					hint="vs. yesterday"
					icon={TrendingUp}
				/>
				<StatCard
					label="Buying power"
					value={fmtMoney(stats.buyingPower, { whole: true })}
					hint={`${fmtMoney(account.marginUsed, { whole: true })} margin used`}
					icon={Banknote}
				/>
				<StatCard
					label="Strategy win rate"
					value={`${stratStats.winRate.toFixed(1)}%`}
					hint={`${stratStats.trades.toLocaleString()} trades · ${stratStats.running} live`}
					icon={Gauge}
				/>
			</div>

			<div className="grid gap-3 lg:grid-cols-3">
				<Card className="lg:col-span-2">
					<CardHeader>
						<SectionHeader
							title="Portfolio value"
							description="Account equity against the S&P 500 benchmark"
							actions={
								<div className="flex items-center gap-0.5 rounded-lg border border-border/80 p-0.5">
									{RANGES.map((option) => (
										<button
											key={option}
											type="button"
											onClick={() => setRange(option)}
											className={cn(
												"h-6 rounded-md px-2 text-[11px] font-medium transition-colors",
												range === option
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
					<CardContent className="flex flex-col gap-4">
						<div className="flex items-end justify-between gap-3">
							<div className="flex flex-col gap-1">
								<span className="num text-2xl font-semibold tracking-tight">
									{fmtMoney(stats.equity, { whole: true })}
								</span>
								<div className="flex items-center gap-2">
									<Delta value={account.dayPnlPct} />
									<span className="num text-xs text-muted-foreground">
										{fmtMoney(account.dayPnl, { signed: true })} today
									</span>
								</div>
							</div>
							<div className="hidden items-center gap-4 text-[11px] text-muted-foreground sm:flex">
								<span className="flex items-center gap-1.5">
									<span className="h-0.5 w-4 rounded-full bg-chart-1" />
									Portfolio
								</span>
								<span className="flex items-center gap-1.5">
									<span className="h-px w-4 border-t border-dashed border-muted-foreground/70" />
									Benchmark
								</span>
							</div>
						</div>
						<EquityChart range={range} />
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<SectionHeader
							title="Allocation"
							description="Exposure by market across open positions"
						/>
					</CardHeader>
					<CardContent>
						<AllocationChart />
					</CardContent>
				</Card>
			</div>

			<div className="grid gap-3 lg:grid-cols-3">
				<Card className="lg:col-span-2">
					<CardHeader>
						<SectionHeader
							title="Open positions"
							description={`${positions.length} positions · ${fmtMoney(stats.exposure, { whole: true })} exposure`}
							actions={
								<Button asChild variant="ghost" size="xs">
									<Link to="/portfolio">View portfolio</Link>
								</Button>
							}
						/>
					</CardHeader>
					<CardContent className="px-0">
						<PositionsTable
							positions={positions.slice(0, 5)}
							quotes={quotes}
							compact
						/>
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<SectionHeader
							title="Watchlist"
							description="Your pinned instruments"
							actions={
								<Button asChild variant="ghost" size="xs">
									<Link to="/markets">All markets</Link>
								</Button>
							}
						/>
					</CardHeader>
					<CardContent className="flex flex-col">
						{WATCHLIST.map((symbol, index) => {
							const quote = quotes[symbol];
							if (!quote) return null;
							const change = quoteChange(quote);
							return (
								<Link
									key={symbol}
									to={`/markets/${symbol}`}
									className={cn(
										"flex items-center gap-3 py-2.5 transition-colors hover:opacity-80",
										index !== 0 && "border-t border-border/60",
									)}
								>
									<div className="flex min-w-0 flex-col gap-0.5">
										<span className="num text-xs font-medium">
											{quote.symbol}
										</span>
										<span className="max-w-[110px] truncate text-[10px] text-muted-foreground">
											{quote.name}
										</span>
									</div>
									<Sparkline
										data={quote.spark}
										className={cn(
											"ms-auto w-16",
											change >= 0 ? "text-positive" : "text-negative",
										)}
									/>
									<div className="flex w-20 flex-col items-end gap-0.5">
										<span className="num text-xs font-medium">
											{fmtPrice(quote.price, quote.market)}
										</span>
										<Delta value={change} icon={false} />
									</div>
								</Link>
							);
						})}
					</CardContent>
				</Card>
			</div>

			<div className="grid gap-3 lg:grid-cols-3">
				<Card className="lg:col-span-2">
					<CardHeader>
						<SectionHeader
							title="Recent orders"
							description="Latest paper executions and working orders"
							actions={
								<Button asChild variant="ghost" size="xs">
									<Link to="/orders">All orders</Link>
								</Button>
							}
						/>
					</CardHeader>
					<CardContent className="px-0">
						<OrdersTable orders={orders.slice(0, 5)} compact />
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<SectionHeader
							title="Activity"
							description="Fills, bot events, and alerts"
						/>
					</CardHeader>
					<CardContent>
						<ActivityList items={activity.slice(0, 5)} />
					</CardContent>
				</Card>
			</div>
		</Page>
	);
}
