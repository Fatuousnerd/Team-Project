import { X } from "lucide-react";
import { Link } from "react-router";
import { toast } from "sonner";
import { OrderStatusBadge, SideText } from "@/components/app/primitives";
import { Button } from "@/components/ui/button";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import {
	Tooltip,
	TooltipContent,
	TooltipTrigger,
} from "@/components/ui/tooltip";
import { fmtDateTime, fmtPrice, fmtQty } from "@/lib/mock/format";
import { useMarketStore } from "@/lib/mock/market-store";
import type { Order } from "@/lib/mock/types";

export function OrdersTable({
	orders,
	compact = false,
}: {
	orders: Order[];
	compact?: boolean;
}) {
	const cancelOrder = useMarketStore((state) => state.cancelOrder);

	const handleCancel = (order: Order) => {
		cancelOrder(order.id);
		toast.info(`Order cancelled`, {
			description: `${order.side} ${fmtQty(order.quantity)} ${order.symbol} · paper account`,
		});
	};

	return (
		<Table>
			<TableHeader>
				<TableRow className="hover:bg-transparent">
					<TableHead className="ps-4">Time</TableHead>
					<TableHead>Instrument</TableHead>
					<TableHead>Side</TableHead>
					<TableHead>Type</TableHead>
					<TableHead className="text-end">Qty</TableHead>
					{compact ? null : <TableHead className="text-end">Price</TableHead>}
					<TableHead className="text-end">Status</TableHead>
					{compact ? null : <TableHead className="w-8 pe-2" />}
				</TableRow>
			</TableHeader>
			<TableBody>
				{orders.map((order) => (
					<TableRow key={order.id}>
						<TableCell className="num py-2.5 ps-4 text-xs text-muted-foreground">
							{fmtDateTime(order.createdAt)}
						</TableCell>
						<TableCell>
							<Link
								to={`/markets/${order.symbol}`}
								className="num text-sm font-medium hover:text-primary"
							>
								{order.symbol}
							</Link>
						</TableCell>
						<TableCell>
							<SideText side={order.side} />
						</TableCell>
						<TableCell className="text-xs text-muted-foreground">
							{order.type}
						</TableCell>
						<TableCell className="num text-end">
							{fmtQty(order.quantity)}
						</TableCell>
						{compact ? null : (
							<TableCell className="num text-end">
								{fmtPrice(order.price, order.market)}
							</TableCell>
						)}
						<TableCell className="text-end">
							<OrderStatusBadge status={order.status} />
						</TableCell>
						{compact ? null : (
							<TableCell className="pe-2">
								{order.status === "Open" ? (
									<Tooltip>
										<TooltipTrigger asChild>
											<Button
												variant="ghost"
												size="icon-xs"
												onClick={() => handleCancel(order)}
												aria-label={`Cancel ${order.symbol} order`}
											>
												<X className="size-3.5" />
											</Button>
										</TooltipTrigger>
										<TooltipContent>Cancel order</TooltipContent>
									</Tooltip>
								) : null}
							</TableCell>
						)}
					</TableRow>
				))}
				{orders.length === 0 ? (
					<TableRow>
						<TableCell
							colSpan={compact ? 6 : 8}
							className="py-12 text-center text-sm text-muted-foreground"
						>
							No orders in this view.
						</TableCell>
					</TableRow>
				) : null}
			</TableBody>
		</Table>
	);
}
