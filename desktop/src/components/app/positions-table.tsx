import { ChevronRight } from "lucide-react";
import { Link } from "react-router";
import { MarketBadge, SideText } from "@/components/app/primitives";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { positionStats } from "@/lib/mock/derive";
import { fmtMoney, fmtPct, fmtPrice, fmtQty } from "@/lib/mock/format";
import type { Position, Quote } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

export function PositionsTable({
	positions,
	quotes,
	compact = false,
}: {
	positions: Position[];
	quotes: Record<string, Quote>;
	compact?: boolean;
}) {
	return (
		<Table>
			<TableHeader>
				<TableRow className="hover:bg-transparent">
					<TableHead className="ps-4">Instrument</TableHead>
					<TableHead>Side</TableHead>
					<TableHead className="text-end">Qty</TableHead>
					{compact ? null : <TableHead className="text-end">Entry</TableHead>}
					<TableHead className="text-end">Last</TableHead>
					<TableHead className="text-end">P&amp;L</TableHead>
					<TableHead className="w-8 pe-2" />
				</TableRow>
			</TableHeader>
			<TableBody>
				{positions.map((position) => {
					const quote = quotes[position.symbol];
					const stats = positionStats(position, quote);
					const positive = stats.pnl >= 0;

					return (
						<TableRow key={position.id} className="group">
							<TableCell className="py-2.5 ps-4">
								<Link
									to={`/markets/${position.symbol}`}
									className="flex items-center gap-2"
								>
									<span className="num text-sm font-medium">
										{position.symbol}
									</span>
									<MarketBadge
										market={position.market}
										className="hidden xl:inline-flex"
									/>
								</Link>
							</TableCell>
							<TableCell>
								<SideText side={position.side} />
							</TableCell>
							<TableCell className="num text-end text-muted-foreground">
								{fmtQty(position.quantity)}
							</TableCell>
							{compact ? null : (
								<TableCell className="num text-end text-muted-foreground">
									{fmtPrice(position.entry, position.market)}
								</TableCell>
							)}
							<TableCell className="num text-end">
								{fmtPrice(stats.price, position.market)}
							</TableCell>
							<TableCell className="text-end">
								<div className="flex flex-col items-end gap-0.5">
									<span
										className={cn(
											"num text-sm font-medium",
											positive ? "text-positive" : "text-negative",
										)}
									>
										{fmtMoney(stats.pnl, { signed: true })}
									</span>
									<span
										className={cn(
											"num text-[11px]",
											positive ? "text-positive/80" : "text-negative/80",
										)}
									>
										{fmtPct(stats.pnlPct)}
									</span>
								</div>
							</TableCell>
							<TableCell className="pe-2">
								<Link
									to={`/markets/${position.symbol}`}
									aria-label={`Trade ${position.symbol}`}
									className="flex size-7 items-center justify-center rounded-md text-muted-foreground/60 transition-colors group-hover:text-foreground"
								>
									<ChevronRight className="size-4" />
								</Link>
							</TableCell>
						</TableRow>
					);
				})}
				{positions.length === 0 ? (
					<TableRow>
						<TableCell
							colSpan={7}
							className="py-10 text-center text-sm text-muted-foreground"
						>
							No open positions.
						</TableCell>
					</TableRow>
				) : null}
			</TableBody>
		</Table>
	);
}
