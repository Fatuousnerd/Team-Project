import { RefreshCw, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { OrdersTable } from "@/components/app/orders-table";
import { Page } from "@/components/app/page";
import { PageHeader } from "@/components/app/primitives";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useMarketStore } from "@/lib/mock/market-store";
import type { OrderStatus } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

const TABS: Array<"All" | OrderStatus> = ["All", "Open", "Filled", "Cancelled"];

export default function Orders() {
	const orders = useMarketStore((state) => state.orders);
	const [status, setStatus] = useState<(typeof TABS)[number]>("All");
	const [query, setQuery] = useState("");

	const counts = useMemo(
		() => ({
			All: orders.length,
			Open: orders.filter((order) => order.status === "Open").length,
			Filled: orders.filter((order) => order.status === "Filled").length,
			Cancelled: orders.filter((order) => order.status === "Cancelled").length,
		}),
		[orders],
	);

	const rows = useMemo(() => {
		const needle = query.trim().toLowerCase();
		return orders.filter((order) => {
			const matchesStatus = status === "All" || order.status === status;
			const matchesQuery =
				needle.length === 0 ||
				order.symbol.toLowerCase().includes(needle) ||
				order.id.toLowerCase().includes(needle);
			return matchesStatus && matchesQuery;
		});
	}, [orders, status, query]);

	return (
		<Page>
			<PageHeader
				title="Orders"
				description="Every paper order placed from the terminal, newest first."
				actions={
					<Button
						variant="outline"
						size="sm"
						onClick={() => toast.success("Order book synced")}
					>
						<RefreshCw className="size-3.5" />
						Sync
					</Button>
				}
			/>

			<Card className="gap-0">
				<CardContent className="flex flex-col justify-between gap-3 py-4 sm:flex-row sm:items-center">
					<div className="flex items-center gap-0.5 overflow-x-auto rounded-lg border border-border/80 p-0.5">
						{TABS.map((tab) => (
							<button
								key={tab}
								type="button"
								onClick={() => setStatus(tab)}
								className={cn(
									"flex h-7 shrink-0 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium transition-colors",
									status === tab
										? "bg-muted text-foreground"
										: "text-muted-foreground hover:text-foreground",
								)}
							>
								{tab}
								<span className="num text-[10px] text-muted-foreground/70">
									{counts[tab]}
								</span>
							</button>
						))}
					</div>

					<div className="relative w-full sm:w-64">
						<Search className="pointer-events-none absolute start-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
						<Input
							value={query}
							onChange={(event) => setQuery(event.target.value)}
							placeholder="Filter by symbol or order id…"
							className="ps-8"
						/>
					</div>
				</CardContent>
				<CardContent className="px-0">
					<OrdersTable orders={rows} />
				</CardContent>
			</Card>

			<p className="text-[11px] text-muted-foreground">
				Open orders can be cancelled inline. Filled and cancelled orders are
				read-only.
			</p>
		</Page>
	);
}
