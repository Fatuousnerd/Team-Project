import { RefreshCw, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { ConnectionPill } from "@/components/app/connection-pill";
import { MarketTable } from "@/components/app/market-table";
import { Page } from "@/components/app/page";
import { PageHeader } from "@/components/app/primitives";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useMarketStore } from "@/lib/mock/market-store";
import type { MarketType } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

const FILTERS: Array<"All" | MarketType> = ["All", "Stocks", "Forex", "Crypto"];

export default function Markets() {
	const quotes = useMarketStore((state) => state.quotes);
	const tick = useMarketStore((state) => state.tick);
	const [filter, setFilter] = useState<(typeof FILTERS)[number]>("All");
	const [query, setQuery] = useState("");

	const rows = useMemo(() => {
		const needle = query.trim().toLowerCase();
		return Object.values(quotes).filter((quote) => {
			const matchesMarket = filter === "All" || quote.market === filter;
			const matchesQuery =
				needle.length === 0 ||
				quote.symbol.toLowerCase().includes(needle) ||
				quote.name.toLowerCase().includes(needle);
			return matchesMarket && matchesQuery;
		});
	}, [quotes, filter, query]);

	return (
		<Page>
			<PageHeader
				title="Markets"
				description="Stocks, forex, and crypto instruments streaming from the local mock engine."
				actions={
					<>
						<ConnectionPill className="hidden sm:inline-flex" />
						<Button
							variant="outline"
							size="sm"
							onClick={() => {
								tick();
								toast.success("Prices refreshed");
							}}
						>
							<RefreshCw className="size-3.5" />
							Refresh
						</Button>
					</>
				}
			/>

			<Card className="gap-0">
				<CardContent className="flex flex-col gap-3 py-4">
					<div className="flex flex-col justify-between gap-3 sm:flex-row sm:items-center">
						<div className="flex items-center gap-0.5 rounded-lg border border-border/80 p-0.5">
							{FILTERS.map((option) => (
								<button
									key={option}
									type="button"
									onClick={() => setFilter(option)}
									className={cn(
										"h-7 rounded-md px-2.5 text-xs font-medium transition-colors",
										filter === option
											? "bg-muted text-foreground"
											: "text-muted-foreground hover:text-foreground",
									)}
								>
									{option}
								</button>
							))}
						</div>

						<div className="relative w-full sm:w-64">
							<Search className="pointer-events-none absolute start-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
							<Input
								value={query}
								onChange={(event) => setQuery(event.target.value)}
								placeholder="Search symbol or name…"
								className="ps-8"
							/>
						</div>
					</div>
				</CardContent>
				<CardContent className="px-0">
					<MarketTable quotes={rows} />
				</CardContent>
			</Card>

			<div className="flex flex-wrap items-center justify-between gap-2 text-[11px] text-muted-foreground">
				<span>
					{rows.length} of {Object.keys(quotes).length} instruments
				</span>
				<span>Simulated feed · ticks every 1.5s · not investment data</span>
			</div>
		</Page>
	);
}
