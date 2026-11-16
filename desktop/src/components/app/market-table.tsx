import { ArrowDown, ArrowUp, ArrowUpDown, ChevronRight } from "lucide-react";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router";
import { Delta, MarketBadge, Sparkline } from "@/components/app/primitives";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { quoteChange } from "@/lib/mock/derive";
import { fmtCompact, fmtPrice } from "@/lib/mock/format";
import type { Quote } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

type SortKey = "symbol" | "price" | "change" | "volume";

const HEADINGS: Array<{ key: SortKey; label: string; className?: string }> = [
	{ key: "symbol", label: "Instrument", className: "ps-4" },
	{ key: "price", label: "Last", className: "text-end" },
	{ key: "change", label: "24h", className: "text-end" },
	{
		key: "volume",
		label: "Volume",
		className: "text-end hidden lg:table-cell",
	},
];

export function MarketTable({ quotes }: { quotes: Quote[] }) {
	const navigate = useNavigate();
	const [sort, setSort] = useState<{ key: SortKey; dir: "asc" | "desc" }>({
		key: "volume",
		dir: "desc",
	});

	const rows = useMemo(() => {
		const sorted = [...quotes];
		sorted.sort((a, b) => {
			const direction = sort.dir === "asc" ? 1 : -1;
			if (sort.key === "symbol")
				return a.symbol.localeCompare(b.symbol) * direction;
			if (sort.key === "price") return (a.price - b.price) * direction;
			if (sort.key === "volume") return (a.volume - b.volume) * direction;
			return (quoteChange(a) - quoteChange(b)) * direction;
		});
		return sorted;
	}, [quotes, sort]);

	const toggleSort = (key: SortKey) => {
		setSort((current) =>
			current.key === key
				? { key, dir: current.dir === "asc" ? "desc" : "asc" }
				: { key, dir: key === "symbol" ? "asc" : "desc" },
		);
	};

	return (
		<Table>
			<TableHeader>
				<TableRow className="hover:bg-transparent">
					{HEADINGS.map((heading) => (
						<TableHead key={heading.key} className={heading.className}>
							<button
								type="button"
								onClick={() => toggleSort(heading.key)}
								className={cn(
									"inline-flex items-center gap-1 text-xs transition-colors hover:text-foreground",
									sort.key === heading.key && "text-foreground",
								)}
							>
								{heading.label}
								{sort.key === heading.key ? (
									sort.dir === "asc" ? (
										<ArrowUp className="size-3" />
									) : (
										<ArrowDown className="size-3" />
									)
								) : (
									<ArrowUpDown className="size-3 opacity-40" />
								)}
							</button>
						</TableHead>
					))}
					<TableHead className="text-end">Bid / Ask</TableHead>
					<TableHead className="hidden w-24 text-end xl:table-cell">
						Trend
					</TableHead>
					<TableHead className="w-8 pe-2" />
				</TableRow>
			</TableHeader>
			<TableBody>
				{rows.map((quote) => {
					const change = quoteChange(quote);
					const positive = change >= 0;

					return (
						<TableRow
							key={quote.symbol}
							tabIndex={0}
							onClick={() => navigate(`/markets/${quote.symbol}`)}
							onKeyDown={(event) => {
								if (event.key === "Enter") navigate(`/markets/${quote.symbol}`);
							}}
							className="cursor-pointer"
						>
							<TableCell className="py-2.5 ps-4">
								<div className="flex items-center gap-2.5">
									<div className="flex flex-col gap-0.5">
										<span className="num text-sm font-medium">
											{quote.symbol}
										</span>
										<span className="max-w-[180px] truncate text-[11px] text-muted-foreground">
											{quote.name}
										</span>
									</div>
									<MarketBadge market={quote.market} />
								</div>
							</TableCell>
							<TableCell className="num text-end font-medium">
								{fmtPrice(quote.price, quote.market)}
							</TableCell>
							<TableCell className="text-end">
								<Delta value={change} icon={false} />
							</TableCell>
							<TableCell className="num hidden text-end text-muted-foreground lg:table-cell">
								{fmtCompact(quote.volume)}
							</TableCell>
							<TableCell className="num text-end text-xs text-muted-foreground">
								<span>{fmtPrice(quote.bid, quote.market)}</span>
								<span className="mx-1 text-muted-foreground/40">/</span>
								<span>{fmtPrice(quote.ask, quote.market)}</span>
							</TableCell>
							<TableCell className="hidden text-end xl:table-cell">
								<div className="flex justify-end">
									<Sparkline
										data={quote.spark}
										className={cn(
											"w-24",
											positive ? "text-positive" : "text-negative",
										)}
									/>
								</div>
							</TableCell>
							<TableCell className="pe-2">
								<span className="flex size-7 items-center justify-center text-muted-foreground/50">
									<ChevronRight className="size-4" />
								</span>
							</TableCell>
						</TableRow>
					);
				})}
				{rows.length === 0 ? (
					<TableRow>
						<TableCell
							colSpan={7}
							className="py-12 text-center text-sm text-muted-foreground"
						>
							No instruments match your filters.
						</TableCell>
					</TableRow>
				) : null}
			</TableBody>
		</Table>
	);
}
