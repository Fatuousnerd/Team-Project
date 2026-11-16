import type { LucideIcon } from "lucide-react";
import { ArrowDownRight, ArrowUpRight, Minus } from "lucide-react";
import type { ReactNode } from "react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { fmtMoney, fmtPct } from "@/lib/mock/format";
import type { MarketType, OrderStatus } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

export function Sparkline({
	data,
	className,
}: {
	data: number[];
	className?: string;
}) {
	if (data.length < 2) return null;
	const width = 100;
	const height = 32;
	const pad = 3;
	const min = Math.min(...data);
	const max = Math.max(...data);
	const span = max - min || 1;
	const points = data.map((value, index) => {
		const x = (index / (data.length - 1)) * width;
		const y = pad + (1 - (value - min) / span) * (height - pad * 2);
		return `${x.toFixed(2)},${y.toFixed(2)}`;
	});
	const line = points.join(" ");
	const area = `0,${height} ${line} ${width},${height}`;

	return (
		<svg
			viewBox={`0 0 ${width} ${height}`}
			preserveAspectRatio="none"
			className={cn("h-8 w-20 shrink-0", className)}
			aria-hidden="true"
		>
			<polygon points={area} fill="currentColor" opacity={0.1} />
			<polyline
				points={line}
				fill="none"
				stroke="currentColor"
				strokeWidth={1.5}
				strokeLinecap="round"
				strokeLinejoin="round"
				vectorEffect="non-scaling-stroke"
			/>
		</svg>
	);
}

export function Delta({
	value,
	kind = "percent",
	icon = true,
	className,
}: {
	value: number;
	kind?: "percent" | "money";
	icon?: boolean;
	className?: string;
}) {
	const positive = value > 0;
	const negative = value < 0;
	const Icon = positive ? ArrowUpRight : negative ? ArrowDownRight : Minus;

	return (
		<span
			className={cn(
				"num inline-flex items-center gap-0.5 text-xs font-medium",
				positive && "text-positive",
				negative && "text-negative",
				!positive && !negative && "text-muted-foreground",
				className,
			)}
		>
			{icon ? <Icon className="size-3.5" /> : null}
			{kind === "percent" ? fmtPct(value) : fmtMoney(value, { signed: true })}
		</span>
	);
}

const MARKET_STYLES: Record<MarketType, string> = {
	Stocks: "bg-secondary text-secondary-foreground",
	Forex: "bg-accent text-accent-foreground",
	Crypto: "bg-primary/12 text-primary",
};

export function MarketBadge({
	market,
	className,
}: {
	market: MarketType;
	className?: string;
}) {
	return (
		<Badge
			variant="outline"
			className={cn(
				"h-5 border-transparent px-1.5 text-[10px] font-medium tracking-wide uppercase",
				MARKET_STYLES[market],
				className,
			)}
		>
			{market}
		</Badge>
	);
}

const STATUS_STYLES: Record<OrderStatus, string> = {
	Open: "border-primary/30 bg-primary/10 text-primary",
	Filled: "border-border bg-secondary text-secondary-foreground",
	Cancelled: "border-transparent bg-muted text-muted-foreground",
};

export function OrderStatusBadge({ status }: { status: OrderStatus }) {
	return (
		<Badge
			variant="outline"
			className={cn(
				"h-5 px-1.5 text-[10px] font-medium",
				STATUS_STYLES[status],
			)}
		>
			{status}
		</Badge>
	);
}

export function SideText({
	side,
	className,
}: {
	side: "Buy" | "Sell" | "Long" | "Short";
	className?: string;
}) {
	const isPositive = side === "Buy" || side === "Long";
	return (
		<span
			className={cn(
				"text-xs font-medium",
				isPositive ? "text-positive" : "text-negative",
				className,
			)}
		>
			{side}
		</span>
	);
}

export function StatCard({
	label,
	value,
	delta,
	deltaKind = "percent",
	hint,
	icon: Icon,
	spark,
	sparkClassName,
	className,
}: {
	label: string;
	value: ReactNode;
	delta?: number;
	deltaKind?: "percent" | "money";
	hint?: string;
	icon?: LucideIcon;
	spark?: number[];
	sparkClassName?: string;
	className?: string;
}) {
	return (
		<Card className={cn("py-0", className)}>
			<CardContent className="flex flex-col gap-3 p-4">
				<div className="flex items-center justify-between gap-2">
					<span className="text-xs text-muted-foreground">{label}</span>
					{Icon ? <Icon className="size-4 text-muted-foreground/60" /> : null}
				</div>

				<div className="flex items-end justify-between gap-3">
					<div className="flex min-w-0 flex-col gap-1">
						<span className="num truncate text-2xl font-semibold tracking-tight">
							{value}
						</span>
						<div className="flex items-center gap-2">
							{delta !== undefined ? (
								<Delta value={delta} kind={deltaKind} />
							) : null}
							{hint ? (
								<span className="truncate text-xs text-muted-foreground">
									{hint}
								</span>
							) : null}
						</div>
					</div>
					{spark ? (
						<Sparkline
							data={spark}
							className={cn("text-muted-foreground", sparkClassName)}
						/>
					) : null}
				</div>
			</CardContent>
		</Card>
	);
}

export function PageHeader({
	title,
	description,
	actions,
	className,
}: {
	title: string;
	description?: string;
	actions?: ReactNode;
	className?: string;
}) {
	return (
		<header
			className={cn(
				"flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between",
				className,
			)}
		>
			<div className="flex flex-col gap-1">
				<h1 className="text-xl font-semibold tracking-tight">{title}</h1>
				{description ? (
					<p className="text-sm text-muted-foreground">{description}</p>
				) : null}
			</div>
			{actions ? (
				<div className="flex items-center gap-2">{actions}</div>
			) : null}
		</header>
	);
}

export function SectionHeader({
	title,
	description,
	actions,
	className,
}: {
	title: string;
	description?: string;
	actions?: ReactNode;
	className?: string;
}) {
	return (
		<div className={cn("flex items-start justify-between gap-3", className)}>
			<div className="flex flex-col gap-0.5">
				<h2 className="text-sm font-medium">{title}</h2>
				{description ? (
					<p className="text-xs text-muted-foreground">{description}</p>
				) : null}
			</div>
			{actions}
		</div>
	);
}
