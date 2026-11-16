import {
	Bot,
	ChevronRight,
	Gauge,
	ListOrdered,
	Pause,
	Play,
	Plus,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { Page } from "@/components/app/page";
import {
	Delta,
	PageHeader,
	SectionHeader,
	Sparkline,
	StatCard,
} from "@/components/app/primitives";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardFooter,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogFooter,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { strategyStats } from "@/lib/mock/derive";
import { fmtMoney } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import {
	type MarketType,
	type Strategy,
	TIMEFRAMES,
	type Timeframe,
} from "@/lib/mock/types";
import { cn } from "@/lib/utils";

const MARKETS: MarketType[] = ["Stocks", "Forex", "Crypto"];
const RISKS: Array<Strategy["risk"]> = ["Low", "Medium", "High"];

function StrategyCard({ strategy }: { strategy: Strategy }) {
	const toggleStrategy = useMarketStore((state) => state.toggleStrategy);
	const running = strategy.status === "running";
	const positive = strategy.pnl >= 0;

	return (
		<Card className="gap-0">
			<CardHeader className="gap-2">
				<div className="flex items-start justify-between gap-2">
					<div className="flex flex-col gap-1">
						<div className="flex flex-wrap items-center gap-2">
							<CardTitle className="text-sm">{strategy.name}</CardTitle>
							<Badge
								variant="outline"
								className={cn(
									"h-5 px-1.5 text-[10px] font-medium",
									running
										? "border-primary/30 bg-primary/10 text-primary"
										: "border-border bg-muted text-muted-foreground",
								)}
							>
								<span
									className={cn(
										"me-1 size-1.5 rounded-full",
										running ? "bg-primary" : "bg-muted-foreground/60",
									)}
								/>
								{running ? "Running" : "Paused"}
							</Badge>
						</div>
						<div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
							<span>{strategy.market}</span>
							<span>·</span>
							<span>{strategy.timeframe}</span>
							<span>·</span>
							<span>{strategy.risk} risk</span>
						</div>
					</div>
					<Button
						variant="outline"
						size="icon-sm"
						onClick={() => toggleStrategy(strategy.id)}
						aria-label={running ? "Pause strategy" : "Start strategy"}
					>
						{running ? (
							<Pause className="size-3.5" />
						) : (
							<Play className="size-3.5" />
						)}
					</Button>
				</div>
				<p className="text-xs leading-relaxed text-muted-foreground">
					{strategy.description}
				</p>
			</CardHeader>

			<CardContent className="flex flex-col gap-4 pt-4">
				<div className="flex items-end justify-between gap-3">
					<div className="flex flex-col gap-0.5">
						<span className="text-[10px] tracking-wide text-muted-foreground uppercase">
							Net P&amp;L
						</span>
						<span
							className={cn(
								"num text-lg font-semibold",
								positive ? "text-positive" : "text-negative",
							)}
						>
							{fmtMoney(strategy.pnl, { signed: true })}
						</span>
						<Delta value={strategy.pnlPct} />
					</div>
					<Sparkline
						data={strategy.spark}
						className={cn(
							"h-10 w-28",
							positive ? "text-positive" : "text-negative",
						)}
					/>
				</div>

				<div className="grid grid-cols-3 divide-x divide-border/70 rounded-lg border border-border/70">
					{[
						{ label: "Win rate", value: `${strategy.winRate.toFixed(1)}%` },
						{ label: "Trades", value: strategy.trades.toLocaleString() },
						{ label: "Max DD", value: `${strategy.maxDrawdown.toFixed(1)}%` },
					].map((metric) => (
						<div key={metric.label} className="flex flex-col gap-0.5 px-3 py-2">
							<span className="text-[10px] text-muted-foreground">
								{metric.label}
							</span>
							<span className="num text-xs font-medium">{metric.value}</span>
						</div>
					))}
				</div>
			</CardContent>

			<CardFooter className="justify-between py-2.5">
				<span className="text-[10px] text-muted-foreground">
					Paper allocation
				</span>
				<Button
					variant="ghost"
					size="xs"
					onClick={() =>
						toast.info(`${strategy.name} logs`, {
							description:
								"Strategy log streaming arrives with the Rust engine.",
						})
					}
				>
					View logs
					<ChevronRight className="size-3" />
				</Button>
			</CardFooter>
		</Card>
	);
}

export default function Strategies() {
	const strategies = useMarketStore((state) => state.strategies);
	const addStrategy = useMarketStore((state) => state.addStrategy);
	const stats = strategyStats(strategies);

	const [open, setOpen] = useState(false);
	const [name, setName] = useState("");
	const [market, setMarket] = useState<MarketType>("Crypto");
	const [risk, setRisk] = useState<Strategy["risk"]>("Medium");
	const [timeframe, setTimeframe] = useState<Timeframe>("1H");

	const create = () => {
		if (name.trim().length === 0) {
			toast.error("Give the strategy a name.");
			return;
		}
		addStrategy({ name: name.trim(), market, risk, timeframe });
		toast.success("Draft strategy created", {
			description: `${name.trim()} · ${market} · ${timeframe}. Start it from the card.`,
		});
		setName("");
		setOpen(false);
	};

	return (
		<Page>
			<PageHeader
				title="Strategies"
				description="Automated paper strategies. Draft a new one or toggle existing bots."
				actions={
					<Button size="sm" onClick={() => setOpen(true)}>
						<Plus className="size-4" />
						New strategy
					</Button>
				}
			/>

			<div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
				<StatCard
					label="Running"
					value={stats.running}
					hint={`${stats.paused} paused`}
					icon={Play}
				/>
				<StatCard
					label="Strategy P&L"
					value={fmtMoney(stats.pnl, { signed: true, whole: true })}
					hint="all time"
					icon={Bot}
				/>
				<StatCard
					label="Win rate"
					value={`${stats.winRate.toFixed(1)}%`}
					hint="trade-weighted"
					icon={Gauge}
				/>
				<StatCard
					label="Trades"
					value={stats.trades.toLocaleString()}
					hint="since inception"
					icon={ListOrdered}
				/>
			</div>

			<div className="grid gap-3 lg:grid-cols-2">
				{strategies.map((strategy) => (
					<StrategyCard key={strategy.id} strategy={strategy} />
				))}
			</div>

			<Card>
				<CardHeader>
					<SectionHeader
						title="Execution engine"
						description="Rust core is not connected yet — strategies run against the mock feed."
						actions={<Badge variant="secondary">Mock mode</Badge>}
					/>
				</CardHeader>
				<CardContent className="text-xs leading-relaxed text-muted-foreground">
					The strategy cards are wired to local state so the flows are
					clickable. Once the Axum service in{" "}
					<span className="num">crates/bot</span> exposes strategy endpoints,
					replace the store actions with API calls — the UI does not need to
					change.
				</CardContent>
			</Card>

			<Dialog open={open} onOpenChange={setOpen}>
				<DialogContent className="sm:max-w-md">
					<DialogHeader>
						<DialogTitle>New strategy</DialogTitle>
						<DialogDescription>
							Drafts start paused so you can review parameters before going
							live.
						</DialogDescription>
					</DialogHeader>

					<div className="flex flex-col gap-3">
						<div className="flex flex-col gap-1.5">
							<Label htmlFor="strategy-name">Name</Label>
							<Input
								id="strategy-name"
								value={name}
								onChange={(event) => setName(event.target.value)}
								placeholder="e.g. Opening range breakout"
							/>
						</div>

						<div className="grid grid-cols-2 gap-3">
							<div className="flex flex-col gap-1.5">
								<Label>Market</Label>
								<Select
									value={market}
									onValueChange={(value) => setMarket(value as MarketType)}
								>
									<SelectTrigger className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{MARKETS.map((option) => (
											<SelectItem key={option} value={option}>
												{option}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</div>

							<div className="flex flex-col gap-1.5">
								<Label>Risk</Label>
								<Select
									value={risk}
									onValueChange={(value) => setRisk(value as Strategy["risk"])}
								>
									<SelectTrigger className="w-full">
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{RISKS.map((option) => (
											<SelectItem key={option} value={option}>
												{option}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</div>
						</div>

						<div className="flex flex-col gap-1.5">
							<Label>Timeframe</Label>
							<Select
								value={timeframe}
								onValueChange={(value) => setTimeframe(value as Timeframe)}
							>
								<SelectTrigger className="w-full">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									{TIMEFRAMES.map((option) => (
										<SelectItem key={option} value={option}>
											{option}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						</div>
					</div>

					<DialogFooter>
						<Button variant="outline" onClick={() => setOpen(false)}>
							Cancel
						</Button>
						<Button onClick={create}>Create draft</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		</Page>
	);
}
