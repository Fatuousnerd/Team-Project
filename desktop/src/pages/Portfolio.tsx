import { Banknote, Plus, TrendingUp, Wallet } from "lucide-react";
import { Link } from "react-router";
import { toast } from "sonner";
import { AllocationChart } from "@/components/app/allocation-chart";
import { Page } from "@/components/app/page";
import { PositionsTable } from "@/components/app/positions-table";
import {
	Delta,
	PageHeader,
	SectionHeader,
	StatCard,
} from "@/components/app/primitives";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { portfolioStats } from "@/lib/mock/derive";
import { fmtMoney } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import { cn } from "@/lib/utils";

export default function Portfolio() {
	const account = useMarketStore(
		(state) =>
			state.accounts.find((item) => item.id === state.activeAccountId) ??
			state.accounts[0],
	);
	const accounts = useMarketStore((state) => state.accounts);
	const activeAccountId = useMarketStore((state) => state.activeAccountId);
	const setActiveAccount = useMarketStore((state) => state.setActiveAccount);
	const positions = useMarketStore((state) => state.positions);
	const quotes = useMarketStore((state) => state.quotes);

	const stats = portfolioStats(account, positions, quotes);

	return (
		<Page>
			<PageHeader
				title="Portfolio"
				description={`${account.name} · ${account.kind}. Paper balances only.`}
				actions={
					<>
						<Button
							variant="outline"
							size="sm"
							onClick={() =>
								toast.info("Deposits are disabled", {
									description: "Paper accounts are funded automatically.",
								})
							}
						>
							<Banknote className="size-3.5" />
							Deposit
						</Button>
						<Button asChild size="sm">
							<Link to="/markets">
								<Plus className="size-4" />
								New order
							</Link>
						</Button>
					</>
				}
			/>

			<div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
				<StatCard
					label="Equity"
					value={fmtMoney(stats.equity, { whole: true })}
					delta={account.dayPnlPct}
					hint="today"
					icon={Wallet}
				/>
				<StatCard
					label="Unrealized P&L"
					value={fmtMoney(stats.unrealized, { signed: true, whole: true })}
					hint={`${positions.length} open positions`}
					icon={TrendingUp}
				/>
				<StatCard
					label="Exposure"
					value={fmtMoney(stats.exposure, { whole: true })}
					hint="gross notional"
					icon={Banknote}
				/>
				<StatCard
					label="Realized today"
					value={fmtMoney(account.realizedPnl, { signed: true, whole: true })}
					delta={account.dayPnlPct}
					deltaKind="percent"
					hint="closed trades"
					icon={TrendingUp}
				/>
			</div>

			<div className="grid gap-3 xl:grid-cols-3">
				<Card className="xl:col-span-2">
					<CardHeader>
						<SectionHeader
							title="Positions"
							description={`${positions.length} open · ${fmtMoney(stats.exposure, { whole: true })} gross exposure`}
							actions={
								<Button asChild variant="ghost" size="xs">
									<Link to="/orders">Order history</Link>
								</Button>
							}
						/>
					</CardHeader>
					<CardContent className="px-0">
						<PositionsTable positions={positions} quotes={quotes} />
					</CardContent>
				</Card>

				<div className="flex min-w-0 flex-col gap-3">
					<Card>
						<CardHeader>
							<SectionHeader
								title="Allocation"
								description="Gross exposure by market"
							/>
						</CardHeader>
						<CardContent>
							<AllocationChart />
						</CardContent>
					</Card>

					<Card>
						<CardHeader>
							<SectionHeader
								title="Accounts"
								description="Choose the account used for new orders"
							/>
						</CardHeader>
						<CardContent className="flex flex-col gap-1">
							{accounts.map((item) => {
								const isActive = item.id === activeAccountId;
								return (
									<button
										key={item.id}
										type="button"
										onClick={() => setActiveAccount(item.id)}
										className={cn(
											"flex items-center justify-between gap-3 rounded-lg border px-3 py-2.5 text-left transition-colors",
											isActive
												? "border-primary/40 bg-primary/6"
												: "border-border/70 hover:bg-muted/50",
										)}
									>
										<div className="flex min-w-0 flex-col gap-0.5">
											<span className="truncate text-xs font-medium">
												{item.name}
											</span>
											<span className="truncate text-[10px] text-muted-foreground">
												{item.kind}
											</span>
										</div>
										<div className="flex shrink-0 flex-col items-end gap-0.5">
											<span className="num text-xs font-medium">
												{fmtMoney(item.balance, { whole: true })}
											</span>
											<Delta value={item.dayPnlPct} icon={false} />
										</div>
									</button>
								);
							})}
						</CardContent>
					</Card>
				</div>
			</div>
		</Page>
	);
}
