import { Minus, Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { fmtMoney, fmtPrice, fmtQty } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import type { OrderSide, OrderType, Quote } from "@/lib/mock/types";
import { cn } from "@/lib/utils";

const ORDER_TYPES: OrderType[] = ["Market", "Limit", "Stop"];

function presetSizes(market: Quote["market"]) {
	if (market === "Stocks") return [1, 5, 10, 25];
	if (market === "Crypto") return [0.05, 0.1, 0.25, 0.5];
	return [1_000, 5_000, 10_000, 25_000];
}

function defaultQuantity(market: Quote["market"]) {
	if (market === "Stocks") return "10";
	if (market === "Crypto") return "0.1";
	return "1000";
}

export function OrderTicket({ quote }: { quote: Quote }) {
	const account = useMarketStore((state) =>
		state.accounts.find((item) => item.id === state.activeAccountId),
	);
	const placeOrder = useMarketStore((state) => state.placeOrder);

	const [side, setSide] = useState<OrderSide>("Buy");
	const [type, setType] = useState<OrderType>("Limit");
	const [quantity, setQuantity] = useState(() => defaultQuantity(quote.market));
	const [limitPrice, setLimitPrice] = useState(() =>
		quote.price.toFixed(quote.decimals),
	);

	const numericQuantity = Number(quantity) || 0;
	const referencePrice =
		type === "Market"
			? side === "Buy"
				? quote.ask
				: quote.bid
			: Number(limitPrice) || quote.price;
	const estimatedCost = numericQuantity * referencePrice;
	const buyingPower = account?.buyingPower ?? 0;
	const presets = useMemo(() => presetSizes(quote.market), [quote.market]);

	const step = (direction: 1 | -1) => {
		const increment =
			quote.market === "Stocks" ? 1 : quote.market === "Crypto" ? 0.05 : 1_000;
		const next = Math.max(0, numericQuantity + increment * direction);
		setQuantity(
			Number.isInteger(increment) ? String(Math.round(next)) : next.toFixed(2),
		);
	};

	const submit = () => {
		if (!Number.isFinite(numericQuantity) || numericQuantity <= 0) {
			toast.error("Enter a quantity greater than zero.");
			return;
		}
		if (
			type !== "Market" &&
			(!Number.isFinite(referencePrice) || referencePrice <= 0)
		) {
			toast.error("Enter a valid trigger price.");
			return;
		}

		const order = placeOrder({
			symbol: quote.symbol,
			side,
			type,
			quantity: numericQuantity,
			price: type === "Market" ? undefined : referencePrice,
		});

		toast.success(`${type} ${side.toLowerCase()} · ${quote.symbol}`, {
			description: `${fmtQty(numericQuantity)} @ ${fmtPrice(order.price, quote.market)} · ${
				order.status === "Filled" ? "filled" : "working"
			} on paper account`,
		});
	};

	return (
		<Card className="gap-0">
			<CardHeader className="border-b! pb-3">
				<CardTitle>Order ticket</CardTitle>
				<p className="text-xs text-muted-foreground">
					Paper trade · {quote.symbol} · {quote.market}
				</p>
			</CardHeader>

			<CardContent className="flex flex-col gap-4 pt-4">
				<div className="grid grid-cols-2 gap-1 rounded-lg bg-muted p-1">
					{(["Buy", "Sell"] as const).map((option) => (
						<button
							key={option}
							type="button"
							onClick={() => setSide(option)}
							className={cn(
								"h-8 rounded-md text-sm font-medium transition-colors",
								side === option
									? option === "Buy"
										? "bg-primary text-primary-foreground"
										: "bg-negative text-background"
									: "text-muted-foreground hover:text-foreground",
							)}
						>
							{option}
						</button>
					))}
				</div>

				<div className="grid grid-cols-3 gap-1 rounded-lg bg-muted p-1">
					{ORDER_TYPES.map((option) => (
						<button
							key={option}
							type="button"
							onClick={() => setType(option)}
							className={cn(
								"h-7 rounded-md text-xs font-medium transition-colors",
								type === option
									? "bg-card text-foreground shadow-sm"
									: "text-muted-foreground hover:text-foreground",
							)}
						>
							{option}
						</button>
					))}
				</div>

				<div className="flex flex-col gap-1.5">
					<div className="flex items-center justify-between">
						<Label htmlFor="order-quantity">Quantity</Label>
						<span className="text-[11px] text-muted-foreground">
							{quote.market === "Forex"
								? "units"
								: quote.market === "Crypto"
									? "coins"
									: "shares"}
						</span>
					</div>
					<div className="flex items-center gap-1">
						<Button
							variant="outline"
							size="icon-sm"
							onClick={() => step(-1)}
							aria-label="Decrease quantity"
						>
							<Minus className="size-3.5" />
						</Button>
						<Input
							id="order-quantity"
							inputMode="decimal"
							value={quantity}
							onChange={(event) => setQuantity(event.target.value)}
							className="num text-center"
						/>
						<Button
							variant="outline"
							size="icon-sm"
							onClick={() => step(1)}
							aria-label="Increase quantity"
						>
							<Plus className="size-3.5" />
						</Button>
					</div>
					<div className="grid grid-cols-4 gap-1">
						{presets.map((preset) => (
							<button
								key={preset}
								type="button"
								onClick={() => setQuantity(String(preset))}
								className="h-7 rounded-md border border-border/80 text-[11px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
							>
								{preset}
							</button>
						))}
					</div>
				</div>

				{type !== "Market" ? (
					<div className="flex flex-col gap-1.5">
						<Label htmlFor="order-price">
							{type === "Limit" ? "Limit price" : "Stop price"}
						</Label>
						<div className="relative">
							<Input
								id="order-price"
								inputMode="decimal"
								value={limitPrice}
								onChange={(event) => setLimitPrice(event.target.value)}
								className="num pe-12"
							/>
							<span className="pointer-events-none absolute end-3 top-1/2 -translate-y-1/2 text-[11px] text-muted-foreground">
								USD
							</span>
						</div>
					</div>
				) : null}

				<div className="flex flex-col gap-2 rounded-lg border border-border/80 bg-muted/40 p-3">
					<div className="flex items-center justify-between text-xs">
						<span className="text-muted-foreground">Est. cost</span>
						<span className="num font-medium">{fmtMoney(estimatedCost)}</span>
					</div>
					<div className="flex items-center justify-between text-xs">
						<span className="text-muted-foreground">Buying power</span>
						<span className="num">
							{fmtMoney(buyingPower, { whole: true })}
						</span>
					</div>
					<Separator />
					<div className="flex items-center justify-between text-xs">
						<span className="text-muted-foreground">Route</span>
						<span className="text-muted-foreground">Paper · T+0</span>
					</div>
				</div>

				<Button
					size="lg"
					onClick={submit}
					className={cn(
						"w-full",
						side === "Buy"
							? "bg-primary text-primary-foreground hover:bg-primary/90"
							: "bg-negative text-background hover:bg-negative/90",
					)}
				>
					{side} {quote.symbol}
				</Button>

				<p className="text-center text-[10px] leading-relaxed text-muted-foreground">
					Mock execution only. Nothing is sent to a broker.
				</p>
			</CardContent>
		</Card>
	);
}
